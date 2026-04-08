use std::collections::{BTreeMap, HashMap};

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use tessara_datasets::{
    DatasetCompositionMode, DatasetGrain, DatasetSelectionRule, validate_dataset_shape,
};
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, require_text},
};

#[derive(Deserialize)]
pub struct CreateDatasetRequest {
    name: String,
    slug: String,
    grain: String,
    #[serde(default = "default_dataset_composition_mode")]
    composition_mode: String,
    sources: Vec<CreateDatasetSourceRequest>,
    fields: Vec<CreateDatasetFieldRequest>,
}

#[derive(Deserialize)]
pub struct CreateDatasetSourceRequest {
    source_alias: String,
    form_id: Option<Uuid>,
    compatibility_group_id: Option<Uuid>,
    selection_rule: String,
}

#[derive(Deserialize)]
pub struct CreateDatasetFieldRequest {
    key: String,
    label: String,
    source_alias: String,
    source_field_key: String,
    position: i32,
}

#[derive(Serialize)]
pub struct DatasetSummary {
    id: Uuid,
    name: String,
    slug: String,
    grain: String,
    composition_mode: String,
    source_count: i64,
    field_count: i64,
}

#[derive(Serialize)]
pub struct DatasetDefinition {
    id: Uuid,
    name: String,
    slug: String,
    grain: String,
    composition_mode: String,
    sources: Vec<DatasetSourceDefinition>,
    fields: Vec<DatasetFieldDefinition>,
}

#[derive(Serialize)]
pub struct DatasetSourceDefinition {
    id: Uuid,
    source_alias: String,
    form_id: Option<Uuid>,
    form_name: Option<String>,
    compatibility_group_id: Option<Uuid>,
    compatibility_group_name: Option<String>,
    selection_rule: String,
    position: i32,
}

#[derive(Serialize)]
pub struct DatasetFieldDefinition {
    id: Uuid,
    key: String,
    label: String,
    source_alias: String,
    source_field_key: String,
    field_type: String,
    position: i32,
}

#[derive(Serialize)]
pub struct DatasetTable {
    dataset_id: Uuid,
    rows: Vec<DatasetTableRow>,
}

#[derive(Serialize)]
pub struct DatasetTableRow {
    pub(crate) submission_id: String,
    pub(crate) node_name: String,
    pub(crate) source_alias: String,
    pub(crate) values: BTreeMap<String, Option<String>>,
}

struct ValidatedDatasetSource {
    source_alias: String,
    form_id: Option<Uuid>,
    compatibility_group_id: Option<Uuid>,
    selection_rule: DatasetSelectionRule,
    position: i32,
}

struct ValidatedDatasetField {
    key: String,
    label: String,
    source_alias: String,
    source_field_key: String,
    field_type: String,
    position: i32,
}

/// Creates a semantic dataset definition for report row modeling.
pub async fn create_dataset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDatasetRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "datasets:write").await?;
    require_text("dataset name", &payload.name)?;
    require_text("dataset slug", &payload.slug)?;
    require_dataset_slug_available(&state.pool, &payload.slug).await?;
    let grain = DatasetGrain::parse(&payload.grain)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let composition_mode = DatasetCompositionMode::parse(&payload.composition_mode)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    validate_dataset_shape(
        payload
            .sources
            .iter()
            .map(|source| source.source_alias.as_str()),
        payload.fields.iter().map(|field| field.key.as_str()),
    )
    .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let sources = validate_dataset_sources(&state.pool, payload.sources).await?;
    let fields = validate_dataset_fields(&state.pool, &sources, payload.fields).await?;

    let mut tx = state.pool.begin().await?;
    let dataset_id: Uuid = sqlx::query_scalar(
        "INSERT INTO datasets (name, slug, grain, composition_mode) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(payload.name)
    .bind(payload.slug)
    .bind(grain.as_str())
    .bind(composition_mode.as_str())
    .fetch_one(&mut *tx)
    .await?;

    insert_dataset_sources(&mut tx, dataset_id, &sources).await?;
    insert_dataset_fields(&mut tx, dataset_id, &fields).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: dataset_id }))
}

/// Updates a dataset definition and replaces its sources and exposed fields.
pub async fn update_dataset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
    Json(payload): Json<CreateDatasetRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "datasets:write").await?;
    require_dataset_exists(&state.pool, dataset_id).await?;
    require_text("dataset name", &payload.name)?;
    require_text("dataset slug", &payload.slug)?;
    require_dataset_slug_available_for_update(&state.pool, dataset_id, &payload.slug).await?;
    let grain = DatasetGrain::parse(&payload.grain)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let composition_mode = DatasetCompositionMode::parse(&payload.composition_mode)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    validate_dataset_shape(
        payload
            .sources
            .iter()
            .map(|source| source.source_alias.as_str()),
        payload.fields.iter().map(|field| field.key.as_str()),
    )
    .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let sources = validate_dataset_sources(&state.pool, payload.sources).await?;
    let fields = validate_dataset_fields(&state.pool, &sources, payload.fields).await?;

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "UPDATE datasets SET name = $1, slug = $2, grain = $3, composition_mode = $4 WHERE id = $5",
    )
    .bind(payload.name)
    .bind(payload.slug)
    .bind(grain.as_str())
    .bind(composition_mode.as_str())
    .bind(dataset_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM dataset_sources WHERE dataset_id = $1")
        .bind(dataset_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM dataset_fields WHERE dataset_id = $1")
        .bind(dataset_id)
        .execute(&mut *tx)
        .await?;
    insert_dataset_sources(&mut tx, dataset_id, &sources).await?;
    insert_dataset_fields(&mut tx, dataset_id, &fields).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: dataset_id }))
}

/// Deletes a dataset definition.
pub async fn delete_dataset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "datasets:write").await?;
    require_dataset_exists(&state.pool, dataset_id).await?;

    sqlx::query("DELETE FROM datasets WHERE id = $1")
        .bind(dataset_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: dataset_id }))
}

/// Lists dataset definitions for the admin reporting workbench.
pub async fn list_datasets(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<DatasetSummary>>> {
    auth::require_capability(&state.pool, &headers, "datasets:read").await?;

    let rows = sqlx::query(
        r#"
        SELECT
            datasets.id,
            datasets.name,
            datasets.slug,
            datasets.grain,
            datasets.composition_mode,
            COUNT(DISTINCT dataset_sources.id) AS source_count,
            COUNT(DISTINCT dataset_fields.id) AS field_count
        FROM datasets
        LEFT JOIN dataset_sources ON dataset_sources.dataset_id = datasets.id
        LEFT JOIN dataset_fields ON dataset_fields.dataset_id = datasets.id
        GROUP BY
            datasets.id,
            datasets.name,
            datasets.slug,
            datasets.grain,
            datasets.composition_mode,
            datasets.created_at
        ORDER BY datasets.created_at, datasets.name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let datasets = rows
        .into_iter()
        .map(|row| {
            Ok(DatasetSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                grain: row.try_get("grain")?,
                composition_mode: row.try_get("composition_mode")?,
                source_count: row.try_get("source_count")?,
                field_count: row.try_get("field_count")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(datasets))
}

/// Returns one dataset definition with sources and exposed semantic fields.
pub async fn get_dataset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
) -> ApiResult<Json<DatasetDefinition>> {
    auth::require_capability(&state.pool, &headers, "datasets:read").await?;

    let dataset =
        sqlx::query("SELECT id, name, slug, grain, composition_mode FROM datasets WHERE id = $1")
            .bind(dataset_id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("dataset {dataset_id}")))?;

    let source_rows = sqlx::query(
        r#"
        SELECT
            dataset_sources.id,
            dataset_sources.source_alias,
            dataset_sources.form_id,
            forms.name AS form_name,
            dataset_sources.compatibility_group_id,
            compatibility_groups.name AS compatibility_group_name,
            dataset_sources.selection_rule,
            dataset_sources.position
        FROM dataset_sources
        LEFT JOIN forms ON forms.id = dataset_sources.form_id
        LEFT JOIN compatibility_groups
            ON compatibility_groups.id = dataset_sources.compatibility_group_id
        WHERE dataset_sources.dataset_id = $1
        ORDER BY dataset_sources.position, dataset_sources.source_alias
        "#,
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;

    let field_rows = sqlx::query(
        r#"
        SELECT id, key, label, source_alias, source_field_key, field_type::text AS field_type, position
        FROM dataset_fields
        WHERE dataset_id = $1
        ORDER BY position, key
        "#,
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(DatasetDefinition {
        id: dataset.try_get("id")?,
        name: dataset.try_get("name")?,
        slug: dataset.try_get("slug")?,
        grain: dataset.try_get("grain")?,
        composition_mode: dataset.try_get("composition_mode")?,
        sources: source_rows
            .into_iter()
            .map(|row| {
                Ok(DatasetSourceDefinition {
                    id: row.try_get("id")?,
                    source_alias: row.try_get("source_alias")?,
                    form_id: row.try_get("form_id")?,
                    form_name: row.try_get("form_name")?,
                    compatibility_group_id: row.try_get("compatibility_group_id")?,
                    compatibility_group_name: row.try_get("compatibility_group_name")?,
                    selection_rule: row.try_get("selection_rule")?,
                    position: row.try_get("position")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
        fields: field_rows
            .into_iter()
            .map(|row| {
                Ok(DatasetFieldDefinition {
                    id: row.try_get("id")?,
                    key: row.try_get("key")?,
                    label: row.try_get("label")?,
                    source_alias: row.try_get("source_alias")?,
                    source_field_key: row.try_get("source_field_key")?,
                    field_type: row.try_get("field_type")?,
                    position: row.try_get("position")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
    }))
}

/// Executes a submission-grain dataset as either a union or a node-aligned join of sources.
pub async fn run_dataset_table(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
) -> ApiResult<Json<DatasetTable>> {
    auth::require_capability(&state.pool, &headers, "datasets:read").await?;
    let table_rows = load_dataset_table_rows(&state.pool, dataset_id).await?;

    Ok(Json(DatasetTable {
        dataset_id,
        rows: table_rows,
    }))
}

async fn validate_dataset_sources(
    pool: &sqlx::PgPool,
    sources: Vec<CreateDatasetSourceRequest>,
) -> ApiResult<Vec<ValidatedDatasetSource>> {
    let mut validated = Vec::new();
    for (position, source) in sources.into_iter().enumerate() {
        require_text("dataset source alias", &source.source_alias)?;
        let selection_rule = DatasetSelectionRule::parse(&source.selection_rule)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        match (source.form_id, source.compatibility_group_id) {
            (Some(form_id), None) => {
                require_form_exists(pool, form_id).await?;
                validated.push(ValidatedDatasetSource {
                    source_alias: source.source_alias,
                    form_id: Some(form_id),
                    compatibility_group_id: None,
                    selection_rule,
                    position: position as i32,
                });
            }
            (None, Some(compatibility_group_id)) => {
                require_compatibility_group_exists(pool, compatibility_group_id).await?;
                validated.push(ValidatedDatasetSource {
                    source_alias: source.source_alias,
                    form_id: None,
                    compatibility_group_id: Some(compatibility_group_id),
                    selection_rule,
                    position: position as i32,
                });
            }
            _ => {
                return Err(ApiError::BadRequest(
                    "dataset source must reference exactly one form or compatibility group".into(),
                ));
            }
        }
    }

    Ok(validated)
}

async fn validate_dataset_fields(
    pool: &sqlx::PgPool,
    sources: &[ValidatedDatasetSource],
    fields: Vec<CreateDatasetFieldRequest>,
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let source_by_alias = sources
        .iter()
        .map(|source| (source.source_alias.as_str(), source))
        .collect::<HashMap<_, _>>();
    let mut validated = Vec::new();

    for field in fields {
        require_text("dataset field key", &field.key)?;
        require_text("dataset field label", &field.label)?;
        require_text("dataset field source alias", &field.source_alias)?;
        require_text("dataset field source key", &field.source_field_key)?;
        let source = source_by_alias
            .get(field.source_alias.as_str())
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "dataset field '{}' references unknown source alias '{}'",
                    field.key, field.source_alias
                ))
            })?;
        let field_type = require_source_field_exists(pool, source, &field.source_field_key).await?;
        validated.push(ValidatedDatasetField {
            key: field.key,
            label: field.label,
            source_alias: field.source_alias,
            source_field_key: field.source_field_key,
            field_type,
            position: field.position,
        });
    }

    Ok(validated)
}

async fn insert_dataset_sources(
    tx: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    sources: &[ValidatedDatasetSource],
) -> ApiResult<()> {
    for source in sources {
        sqlx::query(
            r#"
            INSERT INTO dataset_sources
                (dataset_id, source_alias, form_id, compatibility_group_id, selection_rule, position)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(dataset_id)
        .bind(&source.source_alias)
        .bind(source.form_id)
        .bind(source.compatibility_group_id)
        .bind(source.selection_rule.as_str())
        .bind(source.position)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn insert_dataset_fields(
    tx: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    fields: &[ValidatedDatasetField],
) -> ApiResult<()> {
    for field in fields {
        sqlx::query(
            r#"
            INSERT INTO dataset_fields
                (dataset_id, key, label, source_alias, source_field_key, field_type, position)
            VALUES ($1, $2, $3, $4, $5, $6::field_type, $7)
            "#,
        )
        .bind(dataset_id)
        .bind(&field.key)
        .bind(&field.label)
        .bind(&field.source_alias)
        .bind(&field.source_field_key)
        .bind(&field.field_type)
        .bind(field.position)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn require_source_field_exists(
    pool: &sqlx::PgPool,
    source: &ValidatedDatasetSource,
    source_field_key: &str,
) -> ApiResult<String> {
    let row = if let Some(form_id) = source.form_id {
        sqlx::query(
            r#"
            SELECT form_fields.field_type::text AS field_type
            FROM form_fields
            JOIN form_versions ON form_versions.id = form_fields.form_version_id
            WHERE form_versions.form_id = $1 AND form_fields.key = $2
            ORDER BY form_versions.created_at DESC, form_fields.position
            LIMIT 1
            "#,
        )
        .bind(form_id)
        .bind(source_field_key)
        .fetch_optional(pool)
        .await?
    } else if let Some(compatibility_group_id) = source.compatibility_group_id {
        sqlx::query(
            r#"
            SELECT form_fields.field_type::text AS field_type
            FROM form_fields
            JOIN form_versions ON form_versions.id = form_fields.form_version_id
            WHERE form_versions.compatibility_group_id = $1 AND form_fields.key = $2
            ORDER BY form_versions.created_at DESC, form_fields.position
            LIMIT 1
            "#,
        )
        .bind(compatibility_group_id)
        .bind(source_field_key)
        .fetch_optional(pool)
        .await?
    } else {
        None
    };

    row.map(|row| row.try_get("field_type"))
        .transpose()?
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "dataset source field '{source_field_key}' is not available on source '{}'",
                source.source_alias
            ))
        })
}

pub(crate) async fn load_dataset_table_rows(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
) -> ApiResult<Vec<DatasetTableRow>> {
    let composition_mode = require_executable_submission_dataset(pool, dataset_id).await?;
    match composition_mode {
        DatasetCompositionMode::Union => run_union_dataset_table(pool, dataset_id).await,
        DatasetCompositionMode::Join => run_join_dataset_table(pool, dataset_id).await,
    }
}

async fn run_union_dataset_table(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
) -> ApiResult<Vec<DatasetTableRow>> {
    let rows = sqlx::query(
        r#"
        WITH ranked_submissions AS (
            SELECT
                dataset_sources.dataset_id,
                dataset_sources.source_alias,
                dataset_sources.selection_rule,
                submission_fact.submission_id,
                submission_fact.node_id,
                node_dim.node_name,
                ROW_NUMBER() OVER (
                    PARTITION BY dataset_sources.dataset_id, dataset_sources.source_alias, submission_fact.node_id
                    ORDER BY
                        CASE
                            WHEN dataset_sources.selection_rule = 'earliest' THEN submission_fact.submitted_at
                        END ASC NULLS LAST,
                        CASE
                            WHEN dataset_sources.selection_rule <> 'earliest' THEN submission_fact.submitted_at
                        END DESC NULLS LAST,
                        submission_fact.submission_id
                ) AS selection_rank
            FROM dataset_sources
            JOIN analytics.form_version_dim
                ON (
                    dataset_sources.form_id IS NOT NULL
                    AND form_version_dim.form_id = dataset_sources.form_id
                )
                OR (
                    dataset_sources.compatibility_group_id IS NOT NULL
                    AND form_version_dim.compatibility_group_id = dataset_sources.compatibility_group_id
                )
            JOIN analytics.submission_fact
                ON submission_fact.form_version_id = form_version_dim.form_version_id
            JOIN analytics.node_dim
                ON node_dim.node_id = submission_fact.node_id
            WHERE dataset_sources.dataset_id = $1
              AND submission_fact.status = 'submitted'
        )
        SELECT
            ranked_submissions.submission_id::text AS submission_id,
            ranked_submissions.node_name,
            ranked_submissions.source_alias,
            dataset_fields.key,
            submission_value_fact.value_text
        FROM ranked_submissions
        JOIN dataset_fields
            ON dataset_fields.dataset_id = ranked_submissions.dataset_id
           AND dataset_fields.source_alias = ranked_submissions.source_alias
        LEFT JOIN analytics.submission_value_fact
            ON submission_value_fact.submission_id = ranked_submissions.submission_id
           AND submission_value_fact.field_key = dataset_fields.source_field_key
        WHERE ranked_submissions.selection_rule = 'all'
           OR ranked_submissions.selection_rank = 1
        ORDER BY ranked_submissions.submission_id, dataset_fields.position, dataset_fields.key
        "#,
    )
    .bind(dataset_id)
    .fetch_all(pool)
    .await?;

    let mut table_rows = BTreeMap::<String, DatasetTableRow>::new();
    for row in rows {
        let submission_id: String = row.try_get("submission_id")?;
        let node_name: String = row.try_get("node_name")?;
        let field_key: String = row.try_get("key")?;
        let source_alias: String = row.try_get("source_alias")?;
        let value: Option<String> = row.try_get("value_text")?;

        table_rows
            .entry(format!("{source_alias}:{submission_id}"))
            .or_insert_with(|| DatasetTableRow {
                submission_id,
                node_name,
                source_alias,
                values: BTreeMap::new(),
            })
            .values
            .insert(field_key, value);
    }

    Ok(table_rows.into_values().collect())
}

async fn run_join_dataset_table(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
) -> ApiResult<Vec<DatasetTableRow>> {
    let rows = sqlx::query(
        r#"
        WITH ranked_submissions AS (
            SELECT
                dataset_sources.dataset_id,
                dataset_sources.source_alias,
                dataset_sources.selection_rule,
                submission_fact.submission_id,
                submission_fact.node_id,
                node_dim.node_name,
                ROW_NUMBER() OVER (
                    PARTITION BY dataset_sources.dataset_id, dataset_sources.source_alias, submission_fact.node_id
                    ORDER BY
                        CASE
                            WHEN dataset_sources.selection_rule = 'earliest' THEN submission_fact.submitted_at
                        END ASC NULLS LAST,
                        CASE
                            WHEN dataset_sources.selection_rule <> 'earliest' THEN submission_fact.submitted_at
                        END DESC NULLS LAST,
                        submission_fact.submission_id
                ) AS selection_rank
            FROM dataset_sources
            JOIN analytics.form_version_dim
                ON (
                    dataset_sources.form_id IS NOT NULL
                    AND form_version_dim.form_id = dataset_sources.form_id
                )
                OR (
                    dataset_sources.compatibility_group_id IS NOT NULL
                    AND form_version_dim.compatibility_group_id = dataset_sources.compatibility_group_id
                )
            JOIN analytics.submission_fact
                ON submission_fact.form_version_id = form_version_dim.form_version_id
            JOIN analytics.node_dim
                ON node_dim.node_id = submission_fact.node_id
            WHERE dataset_sources.dataset_id = $1
              AND submission_fact.status = 'submitted'
        ),
        selected_submissions AS (
            SELECT *
            FROM ranked_submissions
            WHERE selection_rank = 1
        )
        SELECT
            selected_submissions.node_id::text AS node_id,
            selected_submissions.submission_id::text AS submission_id,
            selected_submissions.node_name,
            selected_submissions.source_alias,
            dataset_fields.key,
            submission_value_fact.value_text
        FROM selected_submissions
        JOIN dataset_fields
            ON dataset_fields.dataset_id = selected_submissions.dataset_id
           AND dataset_fields.source_alias = selected_submissions.source_alias
        LEFT JOIN analytics.submission_value_fact
            ON submission_value_fact.submission_id = selected_submissions.submission_id
           AND submission_value_fact.field_key = dataset_fields.source_field_key
        ORDER BY selected_submissions.node_id, selected_submissions.source_alias, dataset_fields.position, dataset_fields.key
        "#,
    )
    .bind(dataset_id)
    .fetch_all(pool)
    .await?;

    let mut table_rows = BTreeMap::<String, DatasetTableRow>::new();
    let mut joined_submissions = BTreeMap::<String, BTreeMap<String, String>>::new();
    for row in rows {
        let node_id: String = row.try_get("node_id")?;
        let submission_id: String = row.try_get("submission_id")?;
        let node_name: String = row.try_get("node_name")?;
        let field_key: String = row.try_get("key")?;
        let source_alias: String = row.try_get("source_alias")?;
        let value: Option<String> = row.try_get("value_text")?;

        joined_submissions
            .entry(node_id.clone())
            .or_default()
            .insert(source_alias.clone(), submission_id);

        table_rows
            .entry(node_id)
            .or_insert_with(|| DatasetTableRow {
                submission_id: String::new(),
                node_name,
                source_alias: "join".into(),
                values: BTreeMap::new(),
            })
            .values
            .insert(field_key, value);
    }

    for (node_id, row) in &mut table_rows {
        if let Some(submissions) = joined_submissions.get(node_id) {
            row.submission_id = submissions
                .iter()
                .map(|(source_alias, submission_id)| format!("{source_alias}:{submission_id}"))
                .collect::<Vec<_>>()
                .join(" | ");
        }
    }

    Ok(table_rows.into_values().collect())
}

pub(crate) async fn require_executable_submission_dataset(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
) -> ApiResult<DatasetCompositionMode> {
    let (dataset_grain, composition_mode): (String, String) =
        sqlx::query_as("SELECT grain, composition_mode FROM datasets WHERE id = $1")
            .bind(dataset_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("dataset {dataset_id}")))?;

    if dataset_grain != DatasetGrain::Submission.as_str() {
        return Err(ApiError::BadRequest(
            "dataset table execution currently supports only submission grain".into(),
        ));
    }
    let composition_mode = DatasetCompositionMode::parse(&composition_mode)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    let source_rows = sqlx::query(
        r#"
        SELECT source_alias, form_id, compatibility_group_id, selection_rule
        FROM dataset_sources
        WHERE dataset_id = $1
        ORDER BY position, source_alias
        "#,
    )
    .bind(dataset_id)
    .fetch_all(pool)
    .await?;

    if source_rows.is_empty() {
        return Err(ApiError::BadRequest(
            "dataset table execution requires at least one source".into(),
        ));
    }

    let mut source_count = 0usize;
    let mut join_has_all_selection_rule = false;
    for source in source_rows {
        source_count += 1;
        let form_id: Option<Uuid> = source.try_get("form_id")?;
        let compatibility_group_id: Option<Uuid> = source.try_get("compatibility_group_id")?;
        let selection_rule: String = source.try_get("selection_rule")?;
        if form_id.is_none() && compatibility_group_id.is_none() {
            return Err(ApiError::BadRequest(
                "dataset table execution currently requires form or compatibility-group sources"
                    .into(),
            ));
        }
        if composition_mode == DatasetCompositionMode::Join && selection_rule == "all" {
            join_has_all_selection_rule = true;
        }
    }
    if composition_mode == DatasetCompositionMode::Join && source_count < 2 {
        return Err(ApiError::BadRequest(
            "join composition mode requires at least two sources".into(),
        ));
    }
    if composition_mode == DatasetCompositionMode::Join && join_has_all_selection_rule {
        return Err(ApiError::BadRequest(
            "join composition mode requires latest or earliest selection rules for every source"
                .into(),
        ));
    }

    let field_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM dataset_fields WHERE dataset_id = $1")
            .bind(dataset_id)
            .fetch_one(pool)
            .await?;

    if field_count == 0 {
        return Err(ApiError::BadRequest(
            "dataset table execution requires at least one field".into(),
        ));
    }

    Ok(composition_mode)
}

fn default_dataset_composition_mode() -> String {
    DatasetCompositionMode::Union.as_str().to_string()
}

async fn require_dataset_slug_available(pool: &sqlx::PgPool, slug: &str) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM datasets WHERE slug = $1)")
        .bind(slug)
        .fetch_one(pool)
        .await?;

    if exists {
        Err(ApiError::BadRequest(format!(
            "dataset slug '{slug}' is already in use"
        )))
    } else {
        Ok(())
    }
}

async fn require_dataset_slug_available_for_update(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
    slug: &str,
) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM datasets WHERE slug = $1 AND id <> $2)")
            .bind(slug)
            .bind(dataset_id)
            .fetch_one(pool)
            .await?;

    if exists {
        Err(ApiError::BadRequest(format!(
            "dataset slug '{slug}' is already in use"
        )))
    } else {
        Ok(())
    }
}

async fn require_dataset_exists(pool: &sqlx::PgPool, dataset_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM datasets WHERE id = $1)")
        .bind(dataset_id)
        .fetch_one(pool)
        .await?;

    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("dataset {dataset_id}")))
    }
}

async fn require_form_exists(pool: &sqlx::PgPool, form_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM forms WHERE id = $1)")
        .bind(form_id)
        .fetch_one(pool)
        .await?;

    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("form {form_id}")))
    }
}

async fn require_compatibility_group_exists(
    pool: &sqlx::PgPool,
    compatibility_group_id: Uuid,
) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM compatibility_groups WHERE id = $1)")
            .bind(compatibility_group_id)
            .fetch_one(pool)
            .await?;

    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!(
            "compatibility group {compatibility_group_id}"
        )))
    }
}
