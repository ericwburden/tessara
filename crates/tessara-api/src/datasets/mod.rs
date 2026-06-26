//! Dataset definition, visibility, and table execution endpoints.
//!
//! Datasets are row-level analytical assets. This module owns validation,
//! visibility enforcement, and table execution; public request/response shapes
//! live in `dto`.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
};
use sqlx::{Column, Postgres, Row, Transaction};
use tessara_datasets::DatasetGrain;
use uuid::Uuid;

mod dto;

pub use dto::{
    CreateDatasetRequest, DatasetAggregationRequest, DatasetCalculatedFieldRequest,
    DatasetDefinition, DatasetFieldDefinition, DatasetOperationRequest,
    DatasetProjectionFieldRequest, DatasetRestrictionPolicyRequest, DatasetRowFilterRequest,
    DatasetSourceDefinition, DatasetSourceRequest, DatasetSqlPreview, DatasetSummary, DatasetTable,
    DatasetTableRow, DatasetVisibilityNodeSummary,
};

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, require_text},
};

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/admin/datasets", post(create_dataset))
        .route("/api/admin/datasets/sql-preview", post(preview_dataset_sql))
        .route(
            "/api/admin/datasets/{dataset_id}/sql-preview",
            post(preview_existing_dataset_sql),
        )
        .route(
            "/api/admin/datasets/{dataset_id}",
            axum::routing::put(update_dataset).delete(delete_dataset),
        )
        .route("/api/datasets", get(list_datasets))
        .route("/api/datasets/{dataset_id}", get(get_dataset))
        .route("/api/datasets/{dataset_id}/table", get(run_dataset_table))
}

#[derive(Clone)]
struct ValidatedDatasetSource {
    source_alias: String,
    form_id: Option<Uuid>,
    form_version_id: Option<Uuid>,
    dataset_revision_id: Option<Uuid>,
    position: i32,
}

#[derive(Clone)]
struct ValidatedDatasetField {
    id: Option<Uuid>,
    key: String,
    label: String,
    source_alias: String,
    source_field_key: String,
    source_field_id: Option<Uuid>,
    field_type: String,
    position: i32,
}

struct CompiledDataset {
    initial_source: DatasetSourceRequest,
    operations: Vec<DatasetOperationRequest>,
    restriction_policy: Option<DatasetRestrictionPolicyRequest>,
    generated_sql: String,
    sources: Vec<ValidatedDatasetSource>,
    fields: Vec<ValidatedDatasetField>,
}

struct QuerySpecBuilder<'a> {
    pool: &'a sqlx::PgPool,
    account: &'a auth::AccountContext,
    dataset_id: Option<Uuid>,
    ctes: Vec<String>,
    current_cte: String,
    fields: Vec<ValidatedDatasetField>,
    cte_index: usize,
    aliases: BTreeSet<String>,
    sources: Vec<ValidatedDatasetSource>,
}

#[derive(Clone)]
struct CompiledSource {
    cte_name: String,
    fields: Vec<ValidatedDatasetField>,
}

#[derive(Clone)]
struct ValidatedAggregation {
    group_fields: Vec<String>,
    metrics: Vec<ValidatedAggregationMetric>,
    row_picker: Option<ValidatedRowPicker>,
}

#[derive(Clone)]
struct ValidatedAggregationMetric {
    key: String,
    label: String,
    function: AggregationFunction,
    source_field_key: Option<String>,
    field_type: String,
    position: i32,
}

#[derive(Clone)]
struct ValidatedRowPicker {
    sort_fields: Vec<ValidatedRowPickerSort>,
    direction: String,
}

#[derive(Clone)]
struct ValidatedRowPickerSort {
    field_key: String,
    field_type: String,
}

#[derive(Clone)]
struct ValidatedRowFilter {
    field_key: String,
    field_type: String,
    operator: RowFilterOperator,
    value: Option<String>,
    value_field_key: Option<String>,
}

#[derive(Clone)]
struct ValidatedCalculatedField {
    key: String,
    label: String,
    base_field_key: String,
    functions: Vec<ValidatedCalculationFunction>,
    field_type: String,
}

#[derive(Clone)]
struct ValidatedCalculationFunction {
    function: CalculationFunction,
    argument: Option<String>,
    argument_field_key: Option<String>,
    input_type: String,
}

#[derive(Clone, Copy)]
enum CalculationFunction {
    Trim,
    Uppercase,
    Lowercase,
    Prefix,
    Suffix,
    Concat,
    Coalesce,
    Constant,
    MapValue,
    Add,
    Subtract,
    Multiply,
    Divide,
    Round,
    FormatDate,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    IsEmpty,
    IsNotEmpty,
    ToText,
    ToNumber,
    ToBoolean,
    ToDate,
}

#[derive(Clone, Default)]
struct ValidatedRestrictionPolicy {
    internal_field_key: Option<String>,
    restricted_field_key: Option<String>,
    confidential_field_key: Option<String>,
}

#[derive(Clone, Copy)]
enum RowFilterOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    IsEmpty,
    IsNotEmpty,
}

#[derive(Clone, Copy)]
enum AggregationFunction {
    CountRows,
    CountValues,
    Sum,
    Average,
    Min,
    Max,
}

#[derive(Clone)]
struct SourceCompileField {
    key: String,
    source_field_key: String,
    source_field_id: Option<Uuid>,
}

struct PublishedFormVersionIdentity {
    id: Uuid,
}

fn internal_dataset_columns() -> BTreeSet<String> {
    ["__row_id"].into_iter().map(String::from).collect()
}

fn ordered_columns(columns: &BTreeSet<String>) -> Vec<String> {
    let mut ordered = ["__row_id"]
        .into_iter()
        .filter(|column| columns.contains(*column))
        .map(String::from)
        .collect::<Vec<_>>();
    let internal = ordered.iter().cloned().collect::<BTreeSet<_>>();
    ordered.extend(
        columns
            .iter()
            .filter(|column| !internal.contains(*column))
            .cloned(),
    );
    ordered
}

fn select_expression_from_cte(
    table_alias: Option<&str>,
    source_columns: &BTreeSet<String>,
    column: &str,
) -> String {
    let quoted = quote_identifier(column);
    if source_columns.contains(column) {
        match table_alias {
            Some(alias) => format!("{alias}.{quoted} AS {quoted}"),
            None => quoted,
        }
    } else {
        format!("NULL::text AS {quoted}")
    }
}

fn coalesced_join_expression(
    left_columns: &BTreeSet<String>,
    right_columns: &BTreeSet<String>,
    column: &str,
) -> String {
    if column == "__row_id" {
        return "CASE WHEN l.__row_id IS NOT NULL AND r.__row_id IS NOT NULL THEN md5(concat_ws('|', l.__row_id, r.__row_id)) ELSE COALESCE(l.__row_id, r.__row_id) END AS __row_id".to_string();
    }
    let quoted = quote_identifier(column);
    match (
        left_columns.contains(column),
        right_columns.contains(column),
    ) {
        (true, true) => format!("COALESCE(l.{quoted}, r.{quoted}) AS {quoted}"),
        (true, false) => format!("l.{quoted} AS {quoted}"),
        (false, true) => format!("r.{quoted} AS {quoted}"),
        (false, false) => format!("NULL::text AS {quoted}"),
    }
}

/// Compiles a dataset draft and returns generated SQL without saving it.
pub async fn preview_dataset_sql(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDatasetRequest>,
) -> ApiResult<Json<DatasetSqlPreview>> {
    preview_dataset_sql_inner(state, headers, None, payload).await
}

/// Compiles an existing dataset draft and returns generated SQL without saving it.
pub async fn preview_existing_dataset_sql(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
    Json(payload): Json<CreateDatasetRequest>,
) -> ApiResult<Json<DatasetSqlPreview>> {
    preview_dataset_sql_inner(state, headers, Some(dataset_id), payload).await
}

async fn preview_dataset_sql_inner(
    state: AppState,
    headers: HeaderMap,
    dataset_id: Option<Uuid>,
    payload: CreateDatasetRequest,
) -> ApiResult<Json<DatasetSqlPreview>> {
    let account = auth::require_capability(&state.pool, &headers, "datasets:manage").await?;
    require_text("dataset name", &payload.name)?;
    require_text("dataset slug", &payload.slug)?;
    let grain = DatasetGrain::parse(&payload.grain)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    if grain != DatasetGrain::Submission {
        return Err(ApiError::BadRequest(
            "dataset query designer currently supports submission grain".into(),
        ));
    }
    let compiled = compile_dataset_definition(&state.pool, &account, dataset_id, &payload).await?;
    Ok(Json(DatasetSqlPreview {
        generated_sql: compiled.generated_sql,
    }))
}

/// Creates a semantic dataset definition and its first immutable revision.
pub async fn create_dataset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDatasetRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "datasets:manage").await?;
    require_text("dataset name", &payload.name)?;
    require_text("dataset slug", &payload.slug)?;
    require_dataset_slug_available(&state.pool, &payload.slug).await?;
    require_node_ids_exist(&state.pool, &payload.visibility_node_ids).await?;
    auth::require_capability_contains_nodes(
        &state.pool,
        &account,
        "datasets:manage",
        &payload.visibility_node_ids,
    )
    .await?;
    let grain = DatasetGrain::parse(&payload.grain)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    if grain != DatasetGrain::Submission {
        return Err(ApiError::BadRequest(
            "dataset query designer currently supports submission grain".into(),
        ));
    }
    let mut tx = state.pool.begin().await?;
    let dataset_id: Uuid = sqlx::query_scalar(
        "INSERT INTO datasets (name, slug, grain) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(grain.as_str())
    .fetch_one(&mut *tx)
    .await?;

    let compiled =
        compile_dataset_definition(&state.pool, &account, Some(dataset_id), &payload).await?;
    insert_dataset_sources(&mut tx, dataset_id, &compiled.sources).await?;
    insert_dataset_fields(&mut tx, dataset_id, &compiled.fields).await?;
    replace_dataset_scope_nodes_tx(&mut tx, dataset_id, &payload.visibility_node_ids).await?;
    let revision_id = insert_dataset_revision(
        &mut tx,
        dataset_id,
        "Initial published definition",
        &compiled,
    )
    .await?;
    materialize_dataset_revision(&mut tx, revision_id, &compiled).await?;
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
    let account = auth::require_capability(&state.pool, &headers, "datasets:manage").await?;
    require_dataset_exists(&state.pool, dataset_id).await?;
    require_dataset_fully_in_capability_scope(&state.pool, &account, "datasets:manage", dataset_id)
        .await?;
    require_text("dataset name", &payload.name)?;
    require_text("dataset slug", &payload.slug)?;
    require_dataset_slug_available_for_update(&state.pool, dataset_id, &payload.slug).await?;
    require_node_ids_exist(&state.pool, &payload.visibility_node_ids).await?;
    auth::require_capability_contains_nodes(
        &state.pool,
        &account,
        "datasets:manage",
        &payload.visibility_node_ids,
    )
    .await?;
    let grain = DatasetGrain::parse(&payload.grain)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    if grain != DatasetGrain::Submission {
        return Err(ApiError::BadRequest(
            "dataset query designer currently supports submission grain".into(),
        ));
    }
    let compiled =
        compile_dataset_definition(&state.pool, &account, Some(dataset_id), &payload).await?;

    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE datasets SET name = $1, slug = $2, grain = $3 WHERE id = $4")
        .bind(payload.name)
        .bind(payload.slug)
        .bind(grain.as_str())
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
    insert_dataset_sources(&mut tx, dataset_id, &compiled.sources).await?;
    insert_dataset_fields(&mut tx, dataset_id, &compiled.fields).await?;
    replace_dataset_scope_nodes_tx(&mut tx, dataset_id, &payload.visibility_node_ids).await?;
    let revision_id = insert_dataset_revision(
        &mut tx,
        dataset_id,
        "Published definition update",
        &compiled,
    )
    .await?;
    materialize_dataset_revision(&mut tx, revision_id, &compiled).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: dataset_id }))
}

/// Deletes a dataset definition.
pub async fn delete_dataset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "datasets:manage").await?;
    require_dataset_exists(&state.pool, dataset_id).await?;
    require_dataset_fully_in_capability_scope(&state.pool, &account, "datasets:manage", dataset_id)
        .await?;

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
    let account = auth::require_capability(&state.pool, &headers, "datasets:read").await?;
    let boundary = auth::capability_boundary(&state.pool, &account, "datasets:read").await?;

    let rows = match &boundary {
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            sqlx::query(
                r#"
        SELECT
            datasets.id,
            current_revisions.id AS current_revision_id,
            datasets.name,
            datasets.slug,
            datasets.grain,
            current_revisions.materialized_row_count,
            current_revisions.materialized_at,
            COUNT(DISTINCT dataset_sources.id) AS source_count,
            COUNT(DISTINCT dataset_fields.id) AS field_count
        FROM datasets
        JOIN dataset_scope_nodes ON dataset_scope_nodes.dataset_id = datasets.id
        LEFT JOIN dataset_revisions AS current_revisions
            ON current_revisions.dataset_id = datasets.id
           AND current_revisions.status = 'published'::dataset_revision_status
        LEFT JOIN dataset_sources ON dataset_sources.dataset_id = datasets.id
        LEFT JOIN dataset_fields ON dataset_fields.dataset_id = datasets.id
        WHERE dataset_scope_nodes.node_id = ANY($1)
        GROUP BY
            datasets.id,
            current_revisions.id,
            current_revisions.materialized_row_count,
            current_revisions.materialized_at,
            datasets.name,
            datasets.slug,
            datasets.grain,
            datasets.created_at
        ORDER BY datasets.created_at, datasets.name
        "#,
            )
            .bind(scope_ids)
            .fetch_all(&state.pool)
            .await?
        }
        auth::CapabilityBoundary::Global => {
            sqlx::query(
                r#"
        SELECT
            datasets.id,
            current_revisions.id AS current_revision_id,
            datasets.name,
            datasets.slug,
            datasets.grain,
            current_revisions.materialized_row_count,
            current_revisions.materialized_at,
            COUNT(DISTINCT dataset_sources.id) AS source_count,
            COUNT(DISTINCT dataset_fields.id) AS field_count
        FROM datasets
        LEFT JOIN dataset_revisions AS current_revisions
            ON current_revisions.dataset_id = datasets.id
           AND current_revisions.status = 'published'::dataset_revision_status
        LEFT JOIN dataset_sources ON dataset_sources.dataset_id = datasets.id
        LEFT JOIN dataset_fields ON dataset_fields.dataset_id = datasets.id
        GROUP BY
            datasets.id,
            current_revisions.id,
            current_revisions.materialized_row_count,
            current_revisions.materialized_at,
            datasets.name,
            datasets.slug,
            datasets.grain,
            datasets.created_at
        ORDER BY datasets.created_at, datasets.name
        "#,
            )
            .fetch_all(&state.pool)
            .await?
        }
        auth::CapabilityBoundary::None => return Err(ApiError::Forbidden("datasets:read".into())),
    };
    let dataset_ids = rows
        .iter()
        .map(|row| row.try_get::<Uuid, _>("id"))
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let visible_node_filter = match &boundary {
        auth::CapabilityBoundary::Scoped(scope_ids) => Some(scope_ids.as_slice()),
        _ => None,
    };
    let visibility_nodes =
        load_dataset_visibility_nodes(&state.pool, &dataset_ids, visible_node_filter).await?;

    let datasets = rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.try_get("id")?;
            Ok(DatasetSummary {
                id,
                current_revision_id: row.try_get("current_revision_id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                grain: row.try_get("grain")?,
                materialized_row_count: row.try_get("materialized_row_count")?,
                materialized_at: row.try_get("materialized_at")?,
                visibility_nodes: visibility_nodes.get(&id).cloned().unwrap_or_default(),
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
    let account = auth::require_capability(&state.pool, &headers, "datasets:read").await?;
    let boundary = auth::capability_boundary(&state.pool, &account, "datasets:read").await?;
    require_dataset_visible_for_boundary(&state.pool, dataset_id, &boundary, "datasets:read")
        .await?;
    let visible_node_filter = match &boundary {
        auth::CapabilityBoundary::Scoped(scope_ids) => Some(scope_ids.as_slice()),
        _ => None,
    };

    let dataset = sqlx::query(
        r#"
        SELECT datasets.id, current_revisions.id AS current_revision_id,
               datasets.name, datasets.slug, datasets.grain,
               current_revisions.initial_source,
               current_revisions.operations,
               current_revisions.restriction_policy,
               current_revisions.generated_sql,
               current_revisions.materialized_schema,
               current_revisions.materialized_table,
               current_revisions.materialized_row_count,
               current_revisions.materialized_at
        FROM datasets
        LEFT JOIN dataset_revisions AS current_revisions
            ON current_revisions.dataset_id = datasets.id
           AND current_revisions.status = 'published'::dataset_revision_status
        WHERE datasets.id = $1
        "#,
    )
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
            dataset_sources.form_version_id,
            dataset_sources.dataset_revision_id,
            dataset_sources.position
        FROM dataset_sources
        LEFT JOIN forms ON forms.id = dataset_sources.form_id
        WHERE dataset_sources.dataset_id = $1
        ORDER BY dataset_sources.position, dataset_sources.source_alias
        "#,
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;

    let field_rows = sqlx::query(
        r#"
        SELECT id, key, label, source_alias, source_field_key, source_field_id, field_type::text AS field_type, position
        FROM dataset_fields
        WHERE dataset_id = $1
        ORDER BY position, key
        "#,
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;

    let visibility_nodes =
        load_dataset_visibility_nodes(&state.pool, &[dataset_id], visible_node_filter).await?;
    let operations = dataset
        .try_get::<Option<serde_json::Value>, _>("operations")?
        .map(serde_json::from_value::<Vec<DatasetOperationRequest>>)
        .transpose()
        .map_err(|error| {
            ApiError::Internal(anyhow::anyhow!(
                "stored dataset operations are invalid: {error}"
            ))
        })?
        .unwrap_or_default();
    let restriction_policy = dataset
        .try_get::<Option<serde_json::Value>, _>("restriction_policy")?
        .map(serde_json::from_value::<DatasetRestrictionPolicyRequest>)
        .transpose()
        .map_err(|error| {
            ApiError::Internal(anyhow::anyhow!(
                "stored dataset restriction policy is invalid: {error}"
            ))
        })?;
    let fields = field_rows
        .into_iter()
        .map(|row| {
            Ok(ValidatedDatasetField {
                id: row.try_get("id")?,
                key: row.try_get("key")?,
                label: row.try_get("label")?,
                source_alias: row.try_get("source_alias")?,
                source_field_key: row.try_get("source_field_key")?,
                source_field_id: row.try_get("source_field_id")?,
                field_type: row.try_get("field_type")?,
                position: row.try_get("position")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let output_fields = fields.clone();

    Ok(Json(DatasetDefinition {
        id: dataset.try_get("id")?,
        current_revision_id: dataset.try_get("current_revision_id")?,
        name: dataset.try_get("name")?,
        slug: dataset.try_get("slug")?,
        grain: dataset.try_get("grain")?,
        initial_source: dataset
            .try_get::<Option<serde_json::Value>, _>("initial_source")?
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| {
                ApiError::Internal(anyhow::anyhow!(
                    "stored dataset initial source is invalid: {error}"
                ))
            })?,
        operations,
        restriction_policy,
        generated_sql: dataset.try_get("generated_sql")?,
        materialized_schema: dataset.try_get("materialized_schema")?,
        materialized_table: dataset.try_get("materialized_table")?,
        materialized_row_count: dataset.try_get("materialized_row_count")?,
        materialized_at: dataset.try_get("materialized_at")?,
        visibility_nodes: visibility_nodes
            .get(&dataset_id)
            .cloned()
            .unwrap_or_default(),
        sources: source_rows
            .into_iter()
            .map(|row| {
                Ok(DatasetSourceDefinition {
                    id: row.try_get("id")?,
                    source_alias: row.try_get("source_alias")?,
                    form_id: row.try_get("form_id")?,
                    form_name: row.try_get("form_name")?,
                    form_version_id: row.try_get("form_version_id")?,
                    dataset_revision_id: row.try_get("dataset_revision_id")?,
                    position: row.try_get("position")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
        fields: fields.iter().map(dataset_field_definition).collect(),
        output_fields: output_fields.iter().map(dataset_field_definition).collect(),
    }))
}

/// Executes a submission-grain dataset as either a union or a node-aligned join of sources.
pub async fn run_dataset_table(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
) -> ApiResult<Json<DatasetTable>> {
    let account = auth::require_capability(&state.pool, &headers, "datasets:read").await?;
    let boundary = auth::capability_boundary(&state.pool, &account, "datasets:read").await?;
    require_dataset_visible_for_boundary(&state.pool, dataset_id, &boundary, "datasets:read")
        .await?;
    match boundary {
        auth::CapabilityBoundary::Global | auth::CapabilityBoundary::Scoped(_) => {}
        auth::CapabilityBoundary::None => return Err(ApiError::Forbidden("datasets:read".into())),
    }
    let table_rows = load_dataset_table_rows(&state.pool, &account, dataset_id).await?;

    Ok(Json(DatasetTable {
        dataset_id,
        rows: table_rows,
    }))
}

async fn compile_dataset_definition(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    dataset_id: Option<Uuid>,
    payload: &CreateDatasetRequest,
) -> ApiResult<CompiledDataset> {
    let mut spec =
        QuerySpecBuilder::init(pool, account, dataset_id, &payload.initial_source).await?;
    spec.apply_operations(&payload.operations).await?;
    let restriction_policy =
        validate_dataset_restriction_policy(payload.restriction_policy.clone(), spec.fields())?;
    require_dataset_output_fields(spec.fields())?;
    let generated_sql = spec.final_sql(&restriction_policy);
    Ok(CompiledDataset {
        initial_source: payload.initial_source.clone(),
        operations: payload.operations.clone(),
        restriction_policy: payload.restriction_policy.clone(),
        generated_sql,
        sources: spec.sources,
        fields: spec.fields,
    })
}

impl<'a> QuerySpecBuilder<'a> {
    async fn init(
        pool: &'a sqlx::PgPool,
        account: &'a auth::AccountContext,
        dataset_id: Option<Uuid>,
        initial_source: &DatasetSourceRequest,
    ) -> ApiResult<Self> {
        let mut builder = Self {
            pool,
            account,
            dataset_id,
            ctes: Vec::new(),
            current_cte: String::new(),
            fields: Vec::new(),
            cte_index: 0,
            aliases: BTreeSet::new(),
            sources: Vec::new(),
        };
        let source = builder.compile_source(initial_source).await?;
        builder.current_cte = source.cte_name;
        builder.fields = source.fields;
        Ok(builder)
    }

    fn fields(&self) -> &[ValidatedDatasetField] {
        &self.fields
    }

    #[cfg(test)]
    fn new_for_test(
        ctes: Vec<String>,
        current_cte: String,
        fields: Vec<ValidatedDatasetField>,
        cte_index: usize,
    ) -> Self {
        let pool = Box::leak(Box::new(
            sqlx::PgPool::connect_lazy("postgres://localhost/tessara_query_spec_test")
                .expect("lazy pool should be constructible for QuerySpecBuilder tests"),
        ));
        let account = Box::leak(Box::new(auth::AccountContext {
            account_id: Uuid::nil(),
            email: "test@example.com".into(),
            display_name: "QuerySpec Test".into(),
            is_active: true,
            roles: Vec::new(),
            capabilities: Vec::new(),
            capability_scopes: Vec::new(),
            scope_nodes: Vec::new(),
            delegations: Vec::new(),
        }));
        Self {
            pool,
            account,
            dataset_id: None,
            ctes,
            current_cte,
            fields,
            cte_index,
            aliases: BTreeSet::new(),
            sources: Vec::new(),
        }
    }

    async fn apply_operations(&mut self, operations: &[DatasetOperationRequest]) -> ApiResult<()> {
        for operation in operations {
            self.apply_operation(operation).await?;
        }
        Ok(())
    }

    async fn apply_operation(&mut self, operation: &DatasetOperationRequest) -> ApiResult<()> {
        match operation {
            DatasetOperationRequest::JoinSource {
                source,
                operation,
                join_keys,
                ..
            } => {
                let right = self.compile_source(source).await?;
                self.apply_join_source(right, operation, join_keys)
            }
            DatasetOperationRequest::UnionSource { source, .. } => {
                let right = self.compile_source(source).await?;
                self.apply_union_source(right, false)
            }
            DatasetOperationRequest::UnionAllSource { source, .. } => {
                let right = self.compile_source(source).await?;
                self.apply_union_source(right, true)
            }
            DatasetOperationRequest::Projection { fields, .. } => self.apply_projection(fields),
            DatasetOperationRequest::Aggregation {
                group_fields,
                metrics,
                row_picker,
                ..
            } => self.apply_aggregation(group_fields, metrics, row_picker),
            DatasetOperationRequest::CalculatedFields { fields, .. } => {
                self.apply_calculated_fields(fields)
            }
            DatasetOperationRequest::Filter { filters, .. } => self.apply_filter(filters),
        }
    }

    async fn compile_source(&mut self, source: &DatasetSourceRequest) -> ApiResult<CompiledSource> {
        match source {
            DatasetSourceRequest::Form {
                alias,
                form_id,
                form_version_id,
            } => {
                self.compile_form_source(alias, *form_id, *form_version_id)
                    .await
            }
            DatasetSourceRequest::Dataset {
                alias,
                dataset_id,
                dataset_revision_id,
            } => {
                self.compile_dataset_source(alias, *dataset_id, *dataset_revision_id)
                    .await
            }
        }
    }

    async fn compile_form_source(
        &mut self,
        alias: &str,
        form_id: Uuid,
        form_version_id: Uuid,
    ) -> ApiResult<CompiledSource> {
        require_identifier("dataset source alias", alias)?;
        if form_id.is_nil() {
            return Err(ApiError::BadRequest(
                "dataset form source must reference a form".into(),
            ));
        }
        if form_version_id.is_nil() {
            return Err(ApiError::BadRequest(
                "dataset form source must reference a form version".into(),
            ));
        }
        if !self.aliases.insert(alias.to_string()) {
            return Err(ApiError::BadRequest(format!(
                "dataset expression alias '{alias}' is duplicated"
            )));
        }
        require_form_exists(self.pool, form_id).await?;
        require_form_readable_by_account(self.pool, self.account, form_id).await?;
        let form_version =
            load_published_form_version_identity(self.pool, form_id, form_version_id).await?;
        let source = ValidatedDatasetSource {
            source_alias: alias.to_string(),
            form_id: Some(form_id),
            form_version_id: Some(form_version.id),
            dataset_revision_id: None,
            position: self.sources.len() as i32,
        };
        let cte_name = self.next_cte_name(alias)?;
        let fields = load_form_source_catalog(self.pool, &source, form_version.id).await?;
        let source_fields = fields
            .iter()
            .map(|field| SourceCompileField {
                key: field.key.clone(),
                source_field_key: field.source_field_key.clone(),
                source_field_id: field.source_field_id,
            })
            .collect::<Vec<_>>();
        let mut value_field_ids = BTreeSet::new();
        let needs_node_dim = source_fields
            .iter()
            .any(|field| field.source_field_key == "__node_name");
        let mut group_by_expressions = BTreeSet::from([
            "submission_fact.form_version_id".to_string(),
            "submission_fact.submission_id".to_string(),
        ]);
        let select_columns = source_fields
            .iter()
            .map(|field| {
                let column = quote_identifier(&field.key);
                if let Some(expression) = system_source_field_expression(&field.source_field_key) {
                    group_by_expressions.insert(expression.to_string());
                    Ok(format!("{expression} AS {column}"))
                } else {
                    let Some(source_field_id) = field.source_field_id else {
                        return Err(ApiError::BadRequest(format!(
                            "dataset field '{}' cannot be projected without a stable source field id",
                            field.key
                        )));
                    };
                    value_field_ids.insert(source_field_id);
                    Ok(format!(
                        "MAX(submission_value_fact.value_text) FILTER (WHERE submission_value_fact.field_id = {}::uuid) AS {column}",
                        sql_literal(&source_field_id.to_string())
                    ))
                }
            })
            .collect::<ApiResult<Vec<_>>>()?
            .join(",\n                ");
        let extra_select_columns = if select_columns.is_empty() {
            String::new()
        } else {
            format!(",\n                {select_columns}")
        };
        let value_join = if value_field_ids.is_empty() {
            String::new()
        } else {
            let field_id_filter = value_field_ids
                .iter()
                .map(|field_id| format!("{}::uuid", sql_literal(&field_id.to_string())))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                r#"
            LEFT JOIN analytics.submission_value_fact
              ON submission_value_fact.submission_id = submission_fact.submission_id
             AND submission_value_fact.form_version_id = submission_fact.form_version_id
             AND submission_value_fact.field_id IN ({field_id_filter})"#
            )
        };
        let group_by = if value_field_ids.is_empty() {
            String::new()
        } else {
            format!(
                r#"
            GROUP BY {}"#,
                group_by_expressions
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let node_join = if needs_node_dim {
            "\n            JOIN analytics.node_dim ON node_dim.node_id = submission_fact.node_id"
        } else {
            ""
        };
        let sql = format!(
            r#"{cte_name} AS (
            SELECT
                md5(concat_ws('|', 'form', submission_fact.form_version_id::text, submission_fact.submission_id::text)) AS __row_id{extra_select_columns}
            FROM analytics.submission_fact{node_join}{value_join}
            WHERE submission_fact.status = 'submitted'
              AND submission_fact.form_version_id = {}::uuid{group_by}
        )"#,
            sql_literal(&form_version.id.to_string())
        );
        self.ctes.push(sql);
        self.sources.push(source);
        Ok(CompiledSource { cte_name, fields })
    }

    async fn compile_dataset_source(
        &mut self,
        alias: &str,
        dataset_id: Uuid,
        dataset_revision_id: Uuid,
    ) -> ApiResult<CompiledSource> {
        require_identifier("dataset source alias", alias)?;
        if !self.aliases.insert(alias.to_string()) {
            return Err(ApiError::BadRequest(format!(
                "dataset expression alias '{alias}' is duplicated"
            )));
        }
        if Some(dataset_id) == self.dataset_id {
            return Err(ApiError::BadRequest(
                "dataset definitions cannot reference themselves".into(),
            ));
        }
        require_dataset_visible_for_account(self.pool, self.account, dataset_id).await?;
        let row = sqlx::query(
            r#"
            SELECT materialized_schema, materialized_table
            FROM dataset_revisions
            WHERE id = $1
              AND dataset_id = $2
              AND materialized_table IS NOT NULL
            "#,
        )
        .bind(dataset_revision_id)
        .bind(dataset_id)
        .fetch_optional(self.pool)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "dataset revision {dataset_revision_id} is not materialized"
            ))
        })?;
        let schema: String = row.try_get("materialized_schema")?;
        let table: String = row.try_get("materialized_table")?;
        let cte_name = self.next_cte_name(alias)?;
        let source = ValidatedDatasetSource {
            source_alias: alias.to_string(),
            form_id: None,
            form_version_id: None,
            dataset_revision_id: Some(dataset_revision_id),
            position: self.sources.len() as i32,
        };
        let fields = load_dataset_source_catalog(self.pool, &source).await?;
        let source_fields = fields
            .iter()
            .map(|field| SourceCompileField {
                key: field.key.clone(),
                source_field_key: field.source_field_key.clone(),
                source_field_id: None,
            })
            .collect::<Vec<_>>();
        let select_columns = source_fields
            .iter()
            .map(|field| {
                format!(
                    "{}::text AS {}",
                    quote_identifier(&field.source_field_key),
                    quote_identifier(&field.key)
                )
            })
            .collect::<Vec<_>>()
            .join(",\n                ");
        let extra_select_columns = if select_columns.is_empty() {
            String::new()
        } else {
            format!(",\n                {select_columns}")
        };
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                __row_id{extra_select_columns}
            FROM {schema}.{table}
        )"#,
            schema = quote_identifier(&schema),
            table = quote_identifier(&table)
        ));
        self.sources.push(source);
        Ok(CompiledSource { cte_name, fields })
    }

    fn apply_join_source(
        &mut self,
        right: CompiledSource,
        operation: &str,
        join_keys: &[dto::DatasetJoinKeyRequest],
    ) -> ApiResult<()> {
        let join = match operation.trim() {
            "left_join" => "LEFT JOIN",
            "inner_join" => "INNER JOIN",
            "outer_join" => "FULL OUTER JOIN",
            "union" | "union_all" => {
                return Err(ApiError::BadRequest(
                    "join_source operations require a join type".into(),
                ));
            }
            other => {
                return Err(ApiError::BadRequest(format!(
                    "unsupported join type '{other}'"
                )));
            }
        };
        let cte_name = self.next_cte_name("op")?;
        let left_columns = catalog_columns(&self.fields);
        let right_columns = catalog_columns(&right.fields);
        let output_fields = merge_field_catalogs(&self.fields, &right.fields)?;
        let output_columns = catalog_columns(&output_fields);
        if join_keys.is_empty() {
            return Err(ApiError::BadRequest(
                "join operations require at least one explicit join key".into(),
            ));
        }
        let predicates = join_keys
            .iter()
            .map(|key| {
                if !left_columns.contains(&key.left_field) {
                    return Err(ApiError::BadRequest(format!(
                        "left join key '{}' is not available from the current input",
                        key.left_field
                    )));
                }
                if !right_columns.contains(&key.right_field) {
                    return Err(ApiError::BadRequest(format!(
                        "right join key '{}' is not available from the joined source",
                        key.right_field
                    )));
                }
                Ok(format!(
                    "l.{} = r.{}",
                    quote_identifier(&key.left_field),
                    quote_identifier(&key.right_field)
                ))
            })
            .collect::<ApiResult<Vec<_>>>()?
            .join(" AND ");
        let columns = ordered_columns(&output_columns)
            .iter()
            .map(|column| coalesced_join_expression(&left_columns, &right_columns, column))
            .collect::<Vec<_>>()
            .join(",\n                ");
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                {columns}
            FROM {} l
            {join} {} r ON {predicates}
        )"#,
            quote_identifier(&self.current_cte),
            quote_identifier(&right.cte_name)
        ));
        self.current_cte = cte_name;
        self.fields = output_fields;
        Ok(())
    }

    fn apply_union_source(&mut self, right: CompiledSource, union_all: bool) -> ApiResult<()> {
        let cte_name = self.next_cte_name("op")?;
        let left_columns = catalog_columns(&self.fields);
        let right_columns = catalog_columns(&right.fields);
        let output_fields = merge_field_catalogs(&self.fields, &right.fields)?;
        let output_columns = catalog_columns(&output_fields);
        let operation = if union_all { "UNION ALL" } else { "UNION" };
        let left_selects = ordered_columns(&output_columns)
            .iter()
            .map(|column| select_expression_from_cte(None, &left_columns, column))
            .collect::<Vec<_>>()
            .join(",\n                ");
        let right_selects = ordered_columns(&output_columns)
            .iter()
            .map(|column| select_expression_from_cte(None, &right_columns, column))
            .collect::<Vec<_>>()
            .join(",\n                ");
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                {left_selects}
            FROM {}
            {operation}
            SELECT
                {right_selects}
            FROM {}
        )"#,
            quote_identifier(&self.current_cte),
            quote_identifier(&right.cte_name)
        ));
        self.current_cte = cte_name;
        self.fields = output_fields;
        Ok(())
    }

    fn next_cte_name(&mut self, seed: &str) -> ApiResult<String> {
        require_identifier("dataset expression alias", seed)?;
        self.cte_index += 1;
        Ok(format!("{}_{}", sanitize_identifier(seed), self.cte_index))
    }
}

impl<'a> QuerySpecBuilder<'a> {
    fn apply_projection(&mut self, requests: &[DatasetProjectionFieldRequest]) -> ApiResult<()> {
        let mut requests = requests
            .iter()
            .filter(|field| {
                !field.key.trim().is_empty()
                    || !field
                        .input_field_key
                        .as_deref()
                        .unwrap_or_default()
                        .trim()
                        .is_empty()
            })
            .cloned()
            .collect::<Vec<_>>();
        requests.sort_by_key(|field| (field.position, field.key.clone()));
        if requests.is_empty() {
            return Err(ApiError::BadRequest(
                "projection operation requires at least one field".into(),
            ));
        }
        let input_by_key = self
            .fields
            .iter()
            .map(|field| (field.key.as_str(), field.clone()))
            .collect::<HashMap<_, _>>();
        let mut seen_output_keys = BTreeSet::new();
        let mut seen_input_keys = BTreeSet::new();
        let mut output_fields = Vec::new();
        let mut select_fields = vec![quote_identifier("__row_id")];

        for (index, request) in requests.into_iter().enumerate() {
            require_text("projection field key", &request.key)?;
            require_text("projection field label", &request.label)?;
            require_identifier("projection field key", &request.key)?;
            if internal_dataset_columns().contains(&request.key) {
                return Err(ApiError::BadRequest(format!(
                    "projection field key '{}' conflicts with an internal dataset column",
                    request.key
                )));
            }
            if !seen_output_keys.insert(request.key.clone()) {
                return Err(ApiError::BadRequest(format!(
                    "projection field key '{}' is duplicated",
                    request.key
                )));
            }
            let input_key = projection_input_field_key(&request)?;
            if !seen_input_keys.insert(input_key.clone()) {
                return Err(ApiError::BadRequest(format!(
                    "projection input field '{}' is duplicated",
                    input_key
                )));
            }
            let input = input_by_key.get(input_key.as_str()).ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "projection field '{}' references unavailable field '{}'",
                    request.key, input_key
                ))
            })?;
            select_fields.push(format!(
                "{} AS {}",
                quote_identifier(&input.key),
                quote_identifier(&request.key)
            ));
            output_fields.push(ValidatedDatasetField {
                id: input.id,
                key: request.key,
                label: request.label,
                source_alias: input.source_alias.clone(),
                source_field_key: input.source_field_key.clone(),
                source_field_id: input.source_field_id,
                field_type: input.field_type.clone(),
                position: index as i32,
            });
        }

        let cte_name = self.next_cte_name("projection")?;
        let select_list = select_fields.join(",\n                ");
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                {select_list}
            FROM {}
        )"#,
            quote_identifier(&self.current_cte)
        ));
        self.current_cte = cte_name;
        self.fields = output_fields;
        Ok(())
    }

    fn apply_aggregation(
        &mut self,
        group_fields: &[String],
        metrics: &[dto::DatasetAggregationMetricRequest],
        row_picker: &Option<dto::DatasetRowPickerRequest>,
    ) -> ApiResult<()> {
        let request = DatasetAggregationRequest {
            group_fields: group_fields.to_vec(),
            metrics: metrics.to_vec(),
            row_picker: row_picker.clone(),
        };
        let aggregation = validate_dataset_aggregation(Some(request), &self.fields)?;
        let cte_name = self.next_cte_name("aggregation")?;
        let sql_body = aggregation
            .as_ref()
            .map(|aggregation| {
                aggregation_sql_from_source(&self.current_cte, &self.fields, aggregation)
            })
            .unwrap_or_else(|| format!("SELECT * FROM {}", quote_identifier(&self.current_cte)));
        self.ctes.push(format!(
            r#"{cte_name} AS (
            {sql_body}
        )"#
        ));
        if let Some(aggregation) = aggregation.as_ref() {
            self.fields = fields_after_aggregation(&self.fields, Some(aggregation));
        }
        self.current_cte = cte_name;
        Ok(())
    }

    fn apply_calculated_fields(
        &mut self,
        requests: &[DatasetCalculatedFieldRequest],
    ) -> ApiResult<()> {
        let calculated = validate_dataset_calculated_fields(requests.to_vec(), &self.fields)?;
        let cte_name = self.next_cte_name("calculated_fields")?;
        let base_selects = self
            .fields
            .iter()
            .map(|field| quote_identifier(&field.key))
            .collect::<Vec<_>>();
        let calculated_selects = calculated
            .iter()
            .map(calculated_field_sql)
            .collect::<Vec<_>>();
        let select_list = ["__row_id"]
            .into_iter()
            .map(quote_identifier)
            .chain(base_selects)
            .chain(calculated_selects)
            .collect::<Vec<_>>()
            .join(",\n                ");
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                {select_list}
            FROM {}
        )"#,
            quote_identifier(&self.current_cte)
        ));
        let mut calculated_output_fields = calculated_fields_for_dataset(&calculated);
        let offset = self.fields.len() as i32;
        for (index, field) in calculated_output_fields.iter_mut().enumerate() {
            field.position = offset + index as i32;
        }
        self.fields.extend(calculated_output_fields);
        self.current_cte = cte_name;
        Ok(())
    }

    fn apply_filter(&mut self, requests: &[DatasetRowFilterRequest]) -> ApiResult<()> {
        let filters = validate_dataset_row_filters(requests.to_vec(), &self.fields)?;
        let cte_name = self.next_cte_name("filtered_fields")?;
        let where_clause = row_filters_sql(&filters);
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                *
            FROM {}{where_clause}
        )"#,
            quote_identifier(&self.current_cte)
        ));
        self.current_cte = cte_name;
        Ok(())
    }

    fn final_sql(&self, restriction_policy: &ValidatedRestrictionPolicy) -> String {
        let field_selects = self.fields.iter().map(|field| quote_identifier(&field.key));
        let final_selects = ["__row_id".to_string()]
            .into_iter()
            .chain(std::iter::once(format!(
                "{} AS {}",
                restriction_policy_tier_sql(restriction_policy),
                quote_identifier("__restriction_tier")
            )))
            .chain(field_selects)
            .collect::<Vec<_>>()
            .join(",\n            ");

        format!(
            r#"WITH
        {}
        SELECT
            {final_selects}
        FROM {}"#,
            self.ctes.join(",\n        "),
            quote_identifier(&self.current_cte)
        )
    }
}

fn projection_input_field_key(request: &DatasetProjectionFieldRequest) -> ApiResult<String> {
    if let Some(input_field_key) = request
        .input_field_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(input_field_key.to_string());
    }
    require_text("projection input field", &request.key)?;
    Ok(request.key.clone())
}

fn canonical_source_column_key(source_alias: &str, source_field_key: &str) -> String {
    format!(
        "{source_alias}__{}",
        source_field_key.trim_start_matches('_')
    )
}

fn catalog_columns(fields: &[ValidatedDatasetField]) -> BTreeSet<String> {
    let mut columns = internal_dataset_columns();
    columns.extend(fields.iter().map(|field| field.key.clone()));
    columns
}

fn merge_field_catalogs(
    left: &[ValidatedDatasetField],
    right: &[ValidatedDatasetField],
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let mut output = left.to_vec();
    let mut index_by_key = output
        .iter()
        .enumerate()
        .map(|(index, field)| (field.key.clone(), index))
        .collect::<HashMap<_, _>>();
    for field in right {
        if let Some(index) = index_by_key.get(&field.key).copied() {
            let existing = &output[index];
            if existing.field_type != field.field_type {
                return Err(ApiError::BadRequest(format!(
                    "field '{}' has incompatible types '{}' and '{}'",
                    field.key, existing.field_type, field.field_type
                )));
            }
            continue;
        }
        index_by_key.insert(field.key.clone(), output.len());
        output.push(field.clone());
    }
    for (index, field) in output.iter_mut().enumerate() {
        field.position = index as i32;
    }
    Ok(output)
}

async fn load_form_source_catalog(
    pool: &sqlx::PgPool,
    source: &ValidatedDatasetSource,
    form_version_id: Uuid,
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let mut fields = system_source_field_keys()
        .into_iter()
        .enumerate()
        .map(|(index, key)| ValidatedDatasetField {
            id: None,
            key: canonical_source_column_key(&source.source_alias, key),
            label: system_source_field_label(key).to_string(),
            source_alias: source.source_alias.clone(),
            source_field_key: key.to_string(),
            source_field_id: None,
            field_type: system_source_field_type(key).unwrap_or("text").to_string(),
            position: index as i32,
        })
        .collect::<Vec<_>>();

    let rows = sqlx::query(
        r#"
        SELECT
            form_fields.key,
            form_fields.label,
            form_fields.field_id,
            form_fields.field_type::text AS field_type,
            form_sections.position AS section_position,
            form_fields.position AS field_position,
            form_fields.grid_row,
            form_fields.grid_column
        FROM form_fields
        JOIN form_sections ON form_sections.id = form_fields.section_id
        WHERE form_fields.form_version_id = $1
        ORDER BY
            form_sections.position,
            form_fields.position,
            form_fields.grid_row,
            form_fields.grid_column,
            form_fields.label,
            form_fields.key
        "#,
    )
    .bind(form_version_id)
    .fetch_all(pool)
    .await?;
    let offset = fields.len() as i32;
    for (index, row) in rows.into_iter().enumerate() {
        let source_field_key: String = row.try_get("key")?;
        fields.push(ValidatedDatasetField {
            id: None,
            key: canonical_source_column_key(&source.source_alias, &source_field_key),
            label: row.try_get("label")?,
            source_alias: source.source_alias.clone(),
            source_field_key,
            source_field_id: Some(row.try_get("field_id")?),
            field_type: row.try_get("field_type")?,
            position: offset + index as i32,
        });
    }
    Ok(fields)
}

async fn load_dataset_source_catalog(
    pool: &sqlx::PgPool,
    source: &ValidatedDatasetSource,
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let Some(dataset_revision_id) = source.dataset_revision_id else {
        return Err(ApiError::BadRequest(format!(
            "dataset source '{}' must reference a dataset revision",
            source.source_alias
        )));
    };
    let mut fields = Vec::new();
    for (index, field) in load_dataset_revision_output_fields(pool, dataset_revision_id)
        .await?
        .into_iter()
        .enumerate()
    {
        fields.push(ValidatedDatasetField {
            id: None,
            key: canonical_source_column_key(&source.source_alias, &field.key),
            label: field.label,
            source_alias: source.source_alias.clone(),
            source_field_key: field.key,
            source_field_id: None,
            field_type: field.field_type,
            position: index as i32,
        });
    }
    Ok(fields)
}

fn aggregation_sql_from_source(
    source_cte: &str,
    fields: &[ValidatedDatasetField],
    aggregation: &ValidatedAggregation,
) -> String {
    let group_selects = aggregation
        .group_fields
        .iter()
        .map(|field| {
            let quoted = quote_identifier(field);
            format!("{quoted} AS {quoted}")
        })
        .collect::<Vec<_>>();
    let metric_selects = aggregation
        .metrics
        .iter()
        .map(aggregation_metric_sql)
        .collect::<Vec<_>>();
    let row_pick_selects = aggregation
        .row_picker
        .as_ref()
        .map(|_| {
            fields
                .iter()
                .filter(|field| !aggregation.group_fields.contains(&field.key))
                .map(|field| {
                    let quoted = quote_identifier(&field.key);
                    format!("MAX({quoted}) FILTER (WHERE __pick_rank = 1) AS {quoted}")
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let output_selects = group_selects
        .into_iter()
        .chain(row_pick_selects)
        .chain(metric_selects)
        .collect::<Vec<_>>();
    let field_select = if output_selects.is_empty() {
        "COUNT(*)::text AS row_count".to_string()
    } else {
        output_selects.join(",\n            ")
    };
    let group_columns = aggregation
        .group_fields
        .iter()
        .map(|field| quote_identifier(field))
        .collect::<Vec<_>>();
    let all_group_columns = group_columns;
    let group_by = if all_group_columns.is_empty() {
        String::new()
    } else {
        format!("\n        GROUP BY {}", all_group_columns.join(", "))
    };
    let row_id = if all_group_columns.is_empty() {
        "'aggregate'::text".to_string()
    } else {
        format!(
            "md5(concat_ws('|', {}))",
            all_group_columns
                .iter()
                .map(|column| format!("{column}::text"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let source = if let Some(row_picker) = &aggregation.row_picker {
        let partition = if all_group_columns.is_empty() {
            String::new()
        } else {
            format!("PARTITION BY {}", all_group_columns.join(", "))
        };
        let order_by = row_picker
            .sort_fields
            .iter()
            .map(|sort| {
                let direction = if row_picker.direction == "highest" {
                    "DESC"
                } else {
                    "ASC"
                };
                let expression =
                    typed_orderable_sql(&quote_identifier(&sort.field_key), &sort.field_type);
                format!("{expression} {direction} NULLS LAST")
            })
            .chain(std::iter::once("__row_id".to_string()))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"(SELECT
                {}.*,
                ROW_NUMBER() OVER (
                    {partition}
                    ORDER BY {order_by}
                ) AS __pick_rank
            FROM {})"#,
            quote_identifier(source_cte),
            quote_identifier(source_cte)
        )
    } else {
        quote_identifier(source_cte)
    };
    format!(
        r#"SELECT
            {row_id} AS __row_id,
            {field_select}
        FROM {source}{group_by}"#
    )
}

fn row_filters_sql(filters: &[ValidatedRowFilter]) -> String {
    if filters.is_empty() {
        return String::new();
    }
    let predicates = filters
        .iter()
        .map(row_filter_sql)
        .collect::<Vec<_>>()
        .join("\n              AND ");
    format!("\n            WHERE {predicates}")
}

fn restriction_policy_tier_sql(restriction_policy: &ValidatedRestrictionPolicy) -> String {
    let internal_predicate = restriction_policy
        .internal_field_key
        .as_deref()
        .map(boolean_tier_predicate_sql)
        .unwrap_or_else(|| "FALSE".to_string());
    let restricted_predicate = restriction_policy
        .restricted_field_key
        .as_deref()
        .map(boolean_tier_predicate_sql)
        .unwrap_or_else(|| "FALSE".to_string());
    let confidential_predicate = restriction_policy
        .confidential_field_key
        .as_deref()
        .map(boolean_tier_predicate_sql)
        .unwrap_or_else(|| "FALSE".to_string());

    format!(
        r#"CASE
                    WHEN {internal_predicate} THEN 'internal'
                    WHEN {restricted_predicate} THEN 'restricted'
                    WHEN {confidential_predicate} THEN 'confidential'
                    ELSE 'public'
                END"#
    )
}

fn boolean_tier_predicate_sql(field_key: &str) -> String {
    boolean_expression_sql(&quote_identifier(field_key))
}

fn validate_dataset_row_filters(
    mut requests: Vec<DatasetRowFilterRequest>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<Vec<ValidatedRowFilter>> {
    requests
        .retain(|filter| !filter.field_key.trim().is_empty() || !filter.operator.trim().is_empty());
    requests.sort_by_key(|filter| (filter.position, filter.field_key.clone()));
    let field_by_key = fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<HashMap<_, _>>();
    let mut filters = Vec::new();
    for filter in requests {
        require_text("row filter field", &filter.field_key)?;
        require_text("row filter operator", &filter.operator)?;
        let field = field_by_key.get(filter.field_key.as_str()).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "row filter field '{}' is not projected",
                filter.field_key
            ))
        })?;
        let operator = RowFilterOperator::parse(&filter.operator)?;
        operator.validate_field_type(&field.field_type, &filter.field_key)?;
        let value_mode = filter.value_mode.trim().to_string();
        if !matches!(value_mode.as_str(), "value" | "field") {
            return Err(ApiError::BadRequest(format!(
                "row filter on '{}' has unsupported value mode '{}'",
                filter.field_key, filter.value_mode
            )));
        }
        let value_field_key = if value_mode == "field" {
            Some(
                filter
                    .value_field_key
                    .map(|key| key.trim().to_string())
                    .filter(|key| !key.is_empty())
                    .ok_or_else(|| {
                        ApiError::BadRequest(format!(
                            "row filter on '{}' requires a value field",
                            filter.field_key
                        ))
                    })?,
            )
        } else {
            None
        };
        if let Some(value_field_key) = &value_field_key {
            let value_field = field_by_key.get(value_field_key.as_str()).ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "row filter value field '{value_field_key}' is not projected"
                ))
            })?;
            if value_field.field_type != field.field_type {
                return Err(ApiError::BadRequest(format!(
                    "row filter value field '{value_field_key}' has type '{}' but '{}' has type '{}'",
                    value_field.field_type, filter.field_key, field.field_type
                )));
            }
        }
        let value = if value_mode == "value" {
            filter.value.map(|value| value.trim().to_string())
        } else {
            None
        };
        if operator.requires_value()
            && value_mode == "value"
            && value.as_deref().unwrap_or_default().is_empty()
        {
            return Err(ApiError::BadRequest(format!(
                "row filter on '{}' requires a value",
                filter.field_key
            )));
        }
        if operator.requires_value() && value_mode == "value" {
            validate_filter_literal(&field.field_type, value.as_deref().unwrap_or_default())?;
        }
        filters.push(ValidatedRowFilter {
            field_key: filter.field_key,
            field_type: field.field_type.clone(),
            operator,
            value,
            value_field_key,
        });
    }
    Ok(filters)
}

fn validate_filter_literal(field_type: &str, value: &str) -> ApiResult<()> {
    validate_typed_literal("row filter", field_type, value)
}

fn validate_typed_literal(label: &str, field_type: &str, value: &str) -> ApiResult<()> {
    match field_type {
        "number" => {
            value.parse::<f64>().map_err(|_| {
                ApiError::BadRequest(format!(
                    "{label} requires a numeric value for field type '{field_type}'"
                ))
            })?;
        }
        "date" => validate_date_literal(label, value)?,
        "datetime" | "timestamp" => validate_datetime_literal(label, value)?,
        "boolean" => validate_boolean_literal(label, value)?,
        _ => {}
    }
    Ok(())
}

fn validate_boolean_literal(label: &str, value: &str) -> ApiResult<()> {
    if matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "false" | "t" | "f" | "1" | "0" | "yes" | "no" | "y" | "n"
    ) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} requires a boolean value"
        )))
    }
}

fn validate_dataset_calculated_fields(
    mut requests: Vec<DatasetCalculatedFieldRequest>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<Vec<ValidatedCalculatedField>> {
    requests
        .retain(|field| !field.key.trim().is_empty() || !field.base_field_key.trim().is_empty());
    requests.sort_by_key(|field| (field.position, field.key.clone()));
    let field_by_key = fields
        .iter()
        .map(|field| (field.key.as_str(), field.clone()))
        .collect::<HashMap<_, _>>();
    let mut seen_keys = field_by_key
        .keys()
        .map(|key| (*key).to_string())
        .collect::<BTreeSet<_>>();
    let mut calculated = Vec::new();
    for request in requests {
        require_text("calculated field key", &request.key)?;
        require_text("calculated field label", &request.label)?;
        require_text("calculated field base field", &request.base_field_key)?;
        require_identifier("calculated field key", &request.key)?;
        if !seen_keys.insert(request.key.clone()) {
            return Err(ApiError::BadRequest(format!(
                "calculated field key '{}' conflicts with an existing field",
                request.key
            )));
        }
        let base_field = field_by_key
            .get(request.base_field_key.as_str())
            .cloned()
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "calculated field '{}' references unknown base field '{}'",
                    request.key, request.base_field_key
                ))
            })?;
        let mut field_type = base_field.field_type.clone();
        let mut functions = request.functions.clone();
        functions.sort_by_key(|function| (function.position, function.function.clone()));
        let mut validated_functions = Vec::new();
        for function in functions {
            let calculation_function = CalculationFunction::parse(&function.function)?;
            let argument_field_key = match function.argument_mode.as_str() {
                "value" => None,
                "field" => {
                    let key = function
                        .argument_field_key
                        .clone()
                        .filter(|key| !key.trim().is_empty())
                        .ok_or_else(|| {
                            ApiError::BadRequest(format!(
                                "calculated field '{}' function requires an argument field",
                                request.key
                            ))
                        })?;
                    let argument_field = field_by_key.get(key.as_str()).ok_or_else(|| {
                        ApiError::BadRequest(format!(
                            "calculated field '{}' references unknown argument field '{}'",
                            request.key, key
                        ))
                    })?;
                    calculation_function.validate_argument_field(
                        &argument_field.field_type,
                        &field_type,
                        &request.key,
                    )?;
                    Some(key)
                }
                other => {
                    return Err(ApiError::BadRequest(format!(
                        "calculated field '{}' has unsupported argument mode '{}'",
                        request.key, other
                    )));
                }
            };
            if argument_field_key.is_none() {
                calculation_function.validate_argument(
                    &function.argument,
                    &field_type,
                    &request.key,
                )?;
            } else if !calculation_function.accepts_field_argument() {
                return Err(ApiError::BadRequest(format!(
                    "calculated field '{}' function does not accept a field argument",
                    request.key
                )));
            }
            let input_type = field_type.clone();
            field_type = calculation_function.output_field_type(&field_type, &request.key)?;
            validated_functions.push(ValidatedCalculationFunction {
                function: calculation_function,
                argument: function.argument,
                argument_field_key,
                input_type,
            });
        }
        let validated = ValidatedCalculatedField {
            key: request.key.clone(),
            label: request.label.clone(),
            base_field_key: request.base_field_key.clone(),
            functions: validated_functions,
            field_type: field_type.clone(),
        };
        calculated.push(validated);
    }
    Ok(calculated)
}

fn calculated_fields_for_dataset(
    calculated: &[ValidatedCalculatedField],
) -> Vec<ValidatedDatasetField> {
    calculated
        .iter()
        .enumerate()
        .map(|(index, field)| ValidatedDatasetField {
            id: None,
            key: field.key.clone(),
            label: field.label.clone(),
            source_alias: "calculated".into(),
            source_field_key: field.base_field_key.clone(),
            source_field_id: None,
            field_type: field.field_type.clone(),
            position: i32::MAX - 10_000 + index as i32,
        })
        .collect()
}

fn validate_dataset_restriction_policy(
    request: Option<DatasetRestrictionPolicyRequest>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<ValidatedRestrictionPolicy> {
    let Some(request) = request else {
        return Ok(ValidatedRestrictionPolicy::default());
    };
    let internal_field_key =
        validate_restriction_boolean_field("internal", request.internal_field_key, fields)?;
    let restricted_field_key =
        validate_restriction_boolean_field("restricted", request.restricted_field_key, fields)?;
    let confidential_field_key =
        validate_restriction_boolean_field("confidential", request.confidential_field_key, fields)?;
    Ok(ValidatedRestrictionPolicy {
        internal_field_key,
        restricted_field_key,
        confidential_field_key,
    })
}

fn validate_restriction_boolean_field(
    tier: &str,
    field_key: Option<String>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<Option<String>> {
    let field_key = field_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let Some(field_key) = field_key else {
        return Ok(None);
    };
    let field = fields
        .iter()
        .find(|field| field.key == field_key)
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "{tier} restriction field '{field_key}' is not projected"
            ))
        })?;
    if field.field_type != "boolean" {
        return Err(ApiError::BadRequest(format!(
            "{tier} restriction field '{}' must be boolean, got '{}'",
            field.key, field.field_type
        )));
    }
    Ok(Some(field_key))
}

fn calculated_field_sql(field: &ValidatedCalculatedField) -> String {
    let mut expression = quote_identifier(&field.base_field_key);
    for function in &field.functions {
        expression = calculation_function_sql(function, &expression);
    }
    format!("{} AS {}", expression, quote_identifier(&field.key))
}

fn calculation_function_sql(function: &ValidatedCalculationFunction, input: &str) -> String {
    let argument = function.argument.as_deref().unwrap_or_default();
    let argument_sql = calculation_argument_sql(function);
    match function.function {
        CalculationFunction::Trim => format!("BTRIM({input})"),
        CalculationFunction::Uppercase => format!("UPPER({input})"),
        CalculationFunction::Lowercase => format!("LOWER({input})"),
        CalculationFunction::Prefix => {
            format!("CONCAT({argument_sql}, COALESCE({input}, ''))")
        }
        CalculationFunction::Suffix | CalculationFunction::Concat => {
            format!("CONCAT(COALESCE({input}, ''), {argument_sql})")
        }
        CalculationFunction::Coalesce => {
            format!("COALESCE(NULLIF({input}, ''), {argument_sql})")
        }
        CalculationFunction::Constant => argument_sql,
        CalculationFunction::MapValue => {
            let branches = split_map_arguments(argument)
                .into_iter()
                .map(|(from, to)| {
                    format!(
                        "WHEN COALESCE({input}, '') = {} THEN {}",
                        sql_literal(&from),
                        sql_literal(&to)
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!("CASE {branches} ELSE {input} END")
        }
        CalculationFunction::Add => format!(
            "(NULLIF({input}, '')::numeric + {})::text",
            numeric_operand_sql(function, argument)
        ),
        CalculationFunction::Subtract => {
            format!(
                "(NULLIF({input}, '')::numeric - {})::text",
                numeric_operand_sql(function, argument)
            )
        }
        CalculationFunction::Multiply => {
            format!(
                "(NULLIF({input}, '')::numeric * {})::text",
                numeric_operand_sql(function, argument)
            )
        }
        CalculationFunction::Divide => {
            format!(
                "(NULLIF({input}, '')::numeric / NULLIF({}, 0))::text",
                numeric_operand_sql(function, argument)
            )
        }
        CalculationFunction::Round => format!(
            "ROUND(NULLIF({input}, '')::numeric, {})::text",
            integer_literal(argument)
        ),
        CalculationFunction::FormatDate => {
            let typed_input = date_format_input_sql(input, &function.input_type);
            format!("TO_CHAR({typed_input}, {})", sql_literal(argument))
        }
        CalculationFunction::GreaterThan => comparison_function_sql(input, ">", function),
        CalculationFunction::GreaterThanOrEqual => comparison_function_sql(input, ">=", function),
        CalculationFunction::LessThan => comparison_function_sql(input, "<", function),
        CalculationFunction::LessThanOrEqual => comparison_function_sql(input, "<=", function),
        CalculationFunction::Equal => {
            let equality = equality_sql(input, &argument_sql, &function.input_type);
            format!("CASE WHEN {equality} THEN 'true' ELSE 'false' END")
        }
        CalculationFunction::NotEqual => {
            let equality = equality_sql(input, &argument_sql, &function.input_type);
            format!("CASE WHEN NOT ({equality}) THEN 'true' ELSE 'false' END")
        }
        CalculationFunction::IsEmpty => {
            format!("CASE WHEN NULLIF({input}, '') IS NULL THEN 'true' ELSE 'false' END")
        }
        CalculationFunction::IsNotEmpty => {
            format!("CASE WHEN NULLIF({input}, '') IS NOT NULL THEN 'true' ELSE 'false' END")
        }
        CalculationFunction::ToText => input.to_string(),
        CalculationFunction::ToNumber => format!("NULLIF({input}, '')::numeric::text"),
        CalculationFunction::ToBoolean => {
            format!(
                "CASE WHEN {} THEN 'true' ELSE 'false' END",
                boolean_expression_sql(input)
            )
        }
        CalculationFunction::ToDate => format!("NULLIF({input}, '')::date::text"),
    }
}

fn calculation_argument_sql(function: &ValidatedCalculationFunction) -> String {
    function
        .argument_field_key
        .as_deref()
        .map(quote_identifier)
        .unwrap_or_else(|| sql_literal(function.argument.as_deref().unwrap_or_default()))
}

fn numeric_operand_sql(function: &ValidatedCalculationFunction, argument: &str) -> String {
    function
        .argument_field_key
        .as_deref()
        .map(|field_key| format!("NULLIF({}, '')::numeric", quote_identifier(field_key)))
        .unwrap_or_else(|| numeric_literal(argument))
}

fn comparison_function_sql(
    input: &str,
    operator: &str,
    function: &ValidatedCalculationFunction,
) -> String {
    let argument_sql = calculation_argument_sql(function);
    let predicate = comparison_sql(input, operator, &argument_sql, &function.input_type);
    format!("CASE WHEN {predicate} THEN 'true' ELSE 'false' END")
}

fn comparison_sql(left: &str, operator: &str, right: &str, field_type: &str) -> String {
    typed_comparable_sql(left, field_type)
        .zip(typed_comparable_sql(right, field_type))
        .map(|(left, right)| format!("{left} {operator} {right}"))
        .unwrap_or_else(|| "FALSE /* unsupported comparison */".to_string())
}

fn equality_sql(left: &str, right: &str, field_type: &str) -> String {
    typed_comparable_sql(left, field_type)
        .zip(typed_comparable_sql(right, field_type))
        .map(|(left, right)| {
            format!("COALESCE({left} = {right}, {left} IS NULL AND {right} IS NULL)")
        })
        .unwrap_or_else(|| format!("COALESCE({left}, '') = COALESCE({right}, '')"))
}

fn typed_comparable_sql(expression: &str, field_type: &str) -> Option<String> {
    match field_type {
        "number" => Some(format!("NULLIF({expression}, '')::numeric")),
        "date" => Some(format!("NULLIF({expression}, '')::date")),
        "datetime" | "timestamp" => Some(format!("NULLIF({expression}, '')::timestamptz")),
        "boolean" => Some(nullable_boolean_expression_sql(expression)),
        _ => None,
    }
}

fn boolean_expression_sql(expression: &str) -> String {
    format!("LOWER(COALESCE({expression}, '')) IN ('true', 't', '1', 'yes', 'y')")
}

fn nullable_boolean_expression_sql(expression: &str) -> String {
    format!(
        "CASE WHEN NULLIF({expression}, '') IS NULL THEN NULL ELSE {} END",
        boolean_expression_sql(expression)
    )
}

fn typed_orderable_sql(expression: &str, field_type: &str) -> String {
    typed_comparable_sql(expression, field_type)
        .unwrap_or_else(|| format!("NULLIF({expression}, '')"))
}

fn date_format_input_sql(expression: &str, field_type: &str) -> String {
    match field_type {
        "date" => format!("NULLIF({expression}, '')::date"),
        "datetime" | "timestamp" => format!("NULLIF({expression}, '')::timestamptz"),
        _ => format!("NULLIF({expression}, '')::timestamptz"),
    }
}

impl CalculationFunction {
    fn parse(value: &str) -> ApiResult<Self> {
        match value {
            "trim" => Ok(Self::Trim),
            "uppercase" => Ok(Self::Uppercase),
            "lowercase" => Ok(Self::Lowercase),
            "prefix" => Ok(Self::Prefix),
            "suffix" => Ok(Self::Suffix),
            "concat" => Ok(Self::Concat),
            "coalesce" => Ok(Self::Coalesce),
            "constant" => Ok(Self::Constant),
            "map_value" => Ok(Self::MapValue),
            "add" => Ok(Self::Add),
            "subtract" => Ok(Self::Subtract),
            "multiply" => Ok(Self::Multiply),
            "divide" => Ok(Self::Divide),
            "round" => Ok(Self::Round),
            "format_date" => Ok(Self::FormatDate),
            "greater_than" => Ok(Self::GreaterThan),
            "greater_than_or_equal" => Ok(Self::GreaterThanOrEqual),
            "less_than" => Ok(Self::LessThan),
            "less_than_or_equal" => Ok(Self::LessThanOrEqual),
            "equal" => Ok(Self::Equal),
            "not_equal" => Ok(Self::NotEqual),
            "is_empty" => Ok(Self::IsEmpty),
            "is_not_empty" => Ok(Self::IsNotEmpty),
            "to_text" => Ok(Self::ToText),
            "to_number" => Ok(Self::ToNumber),
            "to_boolean" => Ok(Self::ToBoolean),
            "to_date" => Ok(Self::ToDate),
            other => Err(ApiError::BadRequest(format!(
                "unsupported calculation function '{other}'"
            ))),
        }
    }

    fn validate_argument(
        self,
        argument: &Option<String>,
        input_type: &str,
        field_key: &str,
    ) -> ApiResult<()> {
        let value = argument.as_deref().unwrap_or_default().trim();
        if matches!(self, Self::MapValue) && split_map_arguments(value).is_empty() {
            return Err(ApiError::BadRequest(format!(
                "calculated field '{field_key}' map function requires a from=>to argument"
            )));
        }
        match self {
            Self::Trim
            | Self::Uppercase
            | Self::Lowercase
            | Self::ToText
            | Self::ToNumber
            | Self::ToBoolean
            | Self::ToDate
            | Self::IsEmpty
            | Self::IsNotEmpty => {
                if value.is_empty() {
                    Ok(())
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' function does not accept an argument"
                    )))
                }
            }
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide => {
                value.parse::<f64>().map_err(|_| {
                    ApiError::BadRequest(format!(
                        "calculated field '{field_key}' requires a numeric argument"
                    ))
                })?;
                Ok(())
            }
            Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::LessThan
            | Self::LessThanOrEqual => {
                if input_type == "number" {
                    value.parse::<f64>().map_err(|_| {
                        ApiError::BadRequest(format!(
                            "calculated field '{field_key}' requires a numeric argument"
                        ))
                    })?;
                    Ok(())
                } else if matches!(input_type, "date" | "datetime" | "timestamp") {
                    validate_typed_comparison_argument(input_type, value, field_key)
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' comparison function requires a number or date input"
                    )))
                }
            }
            Self::Equal | Self::NotEqual => {
                if input_type == "number" {
                    value.parse::<f64>().map_err(|_| {
                        ApiError::BadRequest(format!(
                            "calculated field '{field_key}' requires a numeric argument"
                        ))
                    })?;
                    Ok(())
                } else if matches!(input_type, "date" | "datetime" | "timestamp") {
                    validate_typed_comparison_argument(input_type, value, field_key)
                } else if input_type == "boolean" {
                    validate_boolean_literal("calculated field", value)
                } else {
                    validate_non_empty_comparison_argument(value, field_key)
                }
            }
            Self::Round => {
                value.parse::<i32>().map_err(|_| {
                    ApiError::BadRequest(format!(
                        "calculated field '{field_key}' requires an integer argument"
                    ))
                })?;
                Ok(())
            }
            Self::Prefix | Self::Suffix | Self::Concat | Self::MapValue | Self::FormatDate => {
                if value.is_empty() {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' function requires an argument"
                    )))
                } else {
                    Ok(())
                }
            }
            Self::Coalesce | Self::Constant => {
                if value.is_empty() {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' function requires an argument"
                    )))
                } else {
                    validate_typed_literal("calculated field", input_type, value)
                }
            }
        }
    }

    fn accepts_field_argument(self) -> bool {
        matches!(
            self,
            Self::Prefix
                | Self::Suffix
                | Self::Concat
                | Self::Coalesce
                | Self::Constant
                | Self::Add
                | Self::Subtract
                | Self::Multiply
                | Self::Divide
                | Self::GreaterThan
                | Self::GreaterThanOrEqual
                | Self::LessThan
                | Self::LessThanOrEqual
                | Self::Equal
                | Self::NotEqual
        )
    }

    fn validate_argument_field(
        self,
        argument_type: &str,
        input_type: &str,
        field_key: &str,
    ) -> ApiResult<()> {
        if !self.accepts_field_argument() {
            return Err(ApiError::BadRequest(format!(
                "calculated field '{field_key}' function does not accept a field argument"
            )));
        }
        let valid = match self {
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide => argument_type == "number",
            Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::LessThan
            | Self::LessThanOrEqual => {
                argument_type == input_type && input_type == "number"
                    || date_like_type(input_type) && date_like_type(argument_type)
            }
            Self::Prefix | Self::Suffix | Self::Concat => {
                matches!(argument_type, "text" | "static_text")
            }
            Self::Equal | Self::NotEqual | Self::Coalesce | Self::Constant => {
                argument_type == input_type
                    || date_like_type(input_type) && date_like_type(argument_type)
            }
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(ApiError::BadRequest(format!(
                "calculated field '{field_key}' argument field type '{argument_type}' is not compatible with input type '{input_type}'"
            )))
        }
    }

    fn output_field_type(self, input_type: &str, field_key: &str) -> ApiResult<String> {
        match self {
            Self::Trim
            | Self::Uppercase
            | Self::Lowercase
            | Self::Prefix
            | Self::Suffix
            | Self::Concat
            | Self::MapValue => Ok("text".into()),
            Self::Coalesce | Self::Constant => Ok(input_type.into()),
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide | Self::Round => {
                if input_type == "number" {
                    Ok("number".into())
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' numeric function requires a number input"
                    )))
                }
            }
            Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::LessThan
            | Self::LessThanOrEqual => {
                if matches!(input_type, "number" | "date" | "datetime" | "timestamp") {
                    Ok("boolean".into())
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' comparison function requires a number or date input"
                    )))
                }
            }
            Self::Equal | Self::NotEqual | Self::IsEmpty | Self::IsNotEmpty => Ok("boolean".into()),
            Self::ToText => Ok("text".into()),
            Self::ToNumber => Ok("number".into()),
            Self::ToBoolean => Ok("boolean".into()),
            Self::ToDate => Ok("date".into()),
            Self::FormatDate => {
                if matches!(input_type, "date" | "datetime" | "timestamp") {
                    Ok("text".into())
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' date formatting requires a date input"
                    )))
                }
            }
        }
    }
}

fn date_like_type(field_type: &str) -> bool {
    matches!(field_type, "date" | "datetime" | "timestamp")
}

fn validate_non_empty_comparison_argument(value: &str, field_key: &str) -> ApiResult<()> {
    if value.is_empty() {
        Err(ApiError::BadRequest(format!(
            "calculated field '{field_key}' comparison function requires an argument"
        )))
    } else {
        Ok(())
    }
}

fn validate_typed_comparison_argument(
    input_type: &str,
    value: &str,
    field_key: &str,
) -> ApiResult<()> {
    validate_non_empty_comparison_argument(value, field_key)?;
    match input_type {
        "date" => validate_date_literal("calculated field", value),
        "datetime" | "timestamp" => validate_datetime_literal("calculated field", value),
        _ => Ok(()),
    }
}

fn validate_date_literal(label: &str, value: &str) -> ApiResult<()> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| ApiError::BadRequest(format!("{label} requires a date value as YYYY-MM-DD")))
}

fn validate_datetime_literal(label: &str, value: &str) -> ApiResult<()> {
    if chrono::DateTime::parse_from_rfc3339(value).is_ok()
        || chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").is_ok()
        || chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").is_ok()
    {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} requires a datetime value as RFC3339 or YYYY-MM-DDTHH:MM"
        )))
    }
}

fn numeric_literal(value: &str) -> String {
    value
        .parse::<f64>()
        .expect("numeric literal should be validated before SQL generation")
        .to_string()
}

fn integer_literal(value: &str) -> String {
    value
        .parse::<i32>()
        .expect("integer literal should be validated before SQL generation")
        .to_string()
}

fn split_map_arguments(value: &str) -> Vec<(String, String)> {
    value
        .split(',')
        .filter_map(|entry| {
            entry
                .split_once("=>")
                .or_else(|| entry.split_once('='))
                .map(|(from, to)| (from.trim().to_string(), to.trim().to_string()))
        })
        .filter(|(from, to)| !from.is_empty() && !to.is_empty())
        .collect()
}

fn row_filter_sql(filter: &ValidatedRowFilter) -> String {
    let field = quote_identifier(&filter.field_key);
    let operand = filter_operand_sql(filter);
    match filter.operator {
        RowFilterOperator::Equals => equality_sql(&field, &operand, &filter.field_type),
        RowFilterOperator::NotEquals => {
            let equality = equality_sql(&field, &operand, &filter.field_type);
            format!("NOT ({equality})")
        }
        RowFilterOperator::Contains => format!(
            "POSITION(LOWER({}) IN LOWER(COALESCE({field}, ''))) > 0",
            operand
        ),
        RowFilterOperator::GreaterThan => comparison_filter_sql(&field, ">", filter),
        RowFilterOperator::GreaterThanOrEqual => comparison_filter_sql(&field, ">=", filter),
        RowFilterOperator::LessThan => comparison_filter_sql(&field, "<", filter),
        RowFilterOperator::LessThanOrEqual => comparison_filter_sql(&field, "<=", filter),
        RowFilterOperator::IsEmpty => format!("NULLIF({field}, '') IS NULL"),
        RowFilterOperator::IsNotEmpty => format!("NULLIF({field}, '') IS NOT NULL"),
    }
}

fn filter_operand_sql(filter: &ValidatedRowFilter) -> String {
    filter
        .value_field_key
        .as_deref()
        .map(quote_identifier)
        .unwrap_or_else(|| sql_literal(filter.value.as_deref().unwrap_or_default()))
}

fn comparison_filter_sql(field: &str, operator: &str, filter: &ValidatedRowFilter) -> String {
    let operand = filter_operand_sql(filter);
    comparison_sql(field, operator, &operand, &filter.field_type)
}

impl RowFilterOperator {
    fn parse(value: &str) -> ApiResult<Self> {
        match value {
            "equals" => Ok(Self::Equals),
            "not_equals" => Ok(Self::NotEquals),
            "contains" => Ok(Self::Contains),
            "greater_than" => Ok(Self::GreaterThan),
            "greater_than_or_equal" => Ok(Self::GreaterThanOrEqual),
            "less_than" => Ok(Self::LessThan),
            "less_than_or_equal" => Ok(Self::LessThanOrEqual),
            "is_empty" => Ok(Self::IsEmpty),
            "is_not_empty" => Ok(Self::IsNotEmpty),
            other => Err(ApiError::BadRequest(format!(
                "unsupported row filter operator '{other}'"
            ))),
        }
    }

    fn requires_value(self) -> bool {
        matches!(
            self,
            Self::Equals
                | Self::NotEquals
                | Self::Contains
                | Self::GreaterThan
                | Self::GreaterThanOrEqual
                | Self::LessThan
                | Self::LessThanOrEqual
        )
    }

    fn validate_field_type(self, field_type: &str, field_key: &str) -> ApiResult<()> {
        if matches!(self, Self::Contains) && !matches!(field_type, "text" | "static_text") {
            return Err(ApiError::BadRequest(format!(
                "row filter operator 'contains' is not supported for field '{}' with type '{}'",
                field_key, field_type
            )));
        }
        if matches!(
            self,
            Self::GreaterThan | Self::GreaterThanOrEqual | Self::LessThan | Self::LessThanOrEqual
        ) && !matches!(field_type, "number" | "date" | "datetime" | "timestamp")
        {
            return Err(ApiError::BadRequest(format!(
                "row filter comparison is not supported for field '{}' with type '{}'",
                field_key, field_type
            )));
        }
        Ok(())
    }
}

fn validate_dataset_aggregation(
    request: Option<DatasetAggregationRequest>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<Option<ValidatedAggregation>> {
    let Some(request) = request else {
        return Ok(None);
    };
    let active = !request.group_fields.is_empty()
        || !request.metrics.is_empty()
        || request.row_picker.is_some();
    if !active {
        return Ok(None);
    }
    let field_by_key = fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<HashMap<_, _>>();
    let mut group_fields = Vec::new();
    let mut seen_groups = BTreeSet::new();
    for key in request.group_fields {
        require_text("aggregation group field", &key)?;
        if !field_by_key.contains_key(key.as_str()) {
            return Err(ApiError::BadRequest(format!(
                "aggregation group field '{key}' is not projected"
            )));
        }
        if seen_groups.insert(key.clone()) {
            group_fields.push(key);
        }
    }
    let mut seen_metric_keys = BTreeSet::new();
    let mut metrics = Vec::new();
    for metric in request.metrics {
        require_text("aggregation metric key", &metric.key)?;
        require_text("aggregation metric label", &metric.label)?;
        require_identifier("aggregation metric key", &metric.key)?;
        if !seen_metric_keys.insert(metric.key.clone()) {
            return Err(ApiError::BadRequest(format!(
                "aggregation metric key '{}' is duplicated",
                metric.key
            )));
        }
        if field_by_key.contains_key(metric.key.as_str()) || seen_groups.contains(&metric.key) {
            return Err(ApiError::BadRequest(format!(
                "aggregation metric key '{}' conflicts with a projected field",
                metric.key
            )));
        }
        let function = AggregationFunction::parse(&metric.function)?;
        let source_field = metric
            .source_field_key
            .as_deref()
            .unwrap_or_default()
            .trim();
        let source = if function.requires_source_field() {
            if source_field.is_empty() {
                return Err(ApiError::BadRequest(format!(
                    "aggregation metric '{}' requires a source field",
                    metric.key
                )));
            }
            let source = field_by_key.get(source_field).ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "aggregation metric '{}' references unprojected field '{}'",
                    metric.key, source_field
                ))
            })?;
            function.validate_field_type(&source.field_type, &metric.key)?;
            Some((*source).clone())
        } else {
            if !source_field.is_empty() {
                return Err(ApiError::BadRequest(format!(
                    "aggregation metric '{}' does not use a source field",
                    metric.key
                )));
            }
            None
        };
        let field_type = function.output_field_type(source.as_ref());
        metrics.push(ValidatedAggregationMetric {
            key: metric.key,
            label: metric.label,
            function,
            source_field_key: source.map(|field| field.key),
            field_type,
            position: metric.position,
        });
    }
    let row_picker = request
        .row_picker
        .map(|mut row_picker| {
            if row_picker.sort_fields.is_empty() {
                return Err(ApiError::BadRequest(
                    "row picker requires at least one sort field".into(),
                ));
            }
            row_picker
                .sort_fields
                .sort_by_key(|sort| (sort.position, sort.field_key.clone()));
            if !matches!(row_picker.direction.as_str(), "lowest" | "highest") {
                return Err(ApiError::BadRequest(
                    "row picker direction must be 'lowest' or 'highest'".into(),
                ));
            }
            let mut seen_sort_fields = BTreeSet::new();
            let mut sort_fields = Vec::new();
            for sort in &row_picker.sort_fields {
                require_text("row picker sort field", &sort.field_key)?;
                let field = field_by_key.get(sort.field_key.as_str()).ok_or_else(|| {
                    ApiError::BadRequest(format!(
                        "row picker sort field '{}' is not projected",
                        sort.field_key
                    ))
                })?;
                if !seen_sort_fields.insert(sort.field_key.clone()) {
                    return Err(ApiError::BadRequest(format!(
                        "row picker sort field '{}' is duplicated",
                        sort.field_key
                    )));
                }
                sort_fields.push(ValidatedRowPickerSort {
                    field_key: sort.field_key.clone(),
                    field_type: field.field_type.clone(),
                });
            }
            Ok(ValidatedRowPicker {
                sort_fields,
                direction: row_picker.direction,
            })
        })
        .transpose()?;
    Ok(Some(ValidatedAggregation {
        group_fields,
        metrics,
        row_picker,
    }))
}

fn fields_after_aggregation(
    fields: &[ValidatedDatasetField],
    aggregation: Option<&ValidatedAggregation>,
) -> Vec<ValidatedDatasetField> {
    let Some(aggregation) = aggregation else {
        return fields.to_vec();
    };
    let field_by_key = fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    let mut seen = BTreeSet::new();
    for key in &aggregation.group_fields {
        if let Some(field) = field_by_key.get(key.as_str())
            && seen.insert(field.key.clone())
        {
            output.push((*field).clone());
        }
    }
    if aggregation.row_picker.is_some() {
        for field in fields {
            if seen.insert(field.key.clone()) {
                output.push(field.clone());
            }
        }
    }
    let metric_offset = output.len() as i32;
    let mut metrics = aggregation.metrics.clone();
    metrics.sort_by_key(|metric| (metric.position, metric.key.clone()));
    for (index, metric) in metrics.into_iter().enumerate() {
        output.push(ValidatedDatasetField {
            id: None,
            key: metric.key,
            label: metric.label,
            source_alias: "aggregation".into(),
            source_field_key: metric.source_field_key.unwrap_or_default(),
            source_field_id: None,
            field_type: metric.field_type,
            position: metric_offset + index as i32,
        });
    }
    for (index, field) in output.iter_mut().enumerate() {
        field.position = index as i32;
    }
    output
}

fn require_dataset_output_fields(fields: &[ValidatedDatasetField]) -> ApiResult<()> {
    if fields.is_empty() {
        Err(ApiError::BadRequest(
            "dataset output requires at least one field".into(),
        ))
    } else {
        Ok(())
    }
}

fn aggregation_metric_sql(metric: &ValidatedAggregationMetric) -> String {
    let key = quote_identifier(&metric.key);
    match metric.function {
        AggregationFunction::CountRows => format!("COUNT(*)::text AS {key}"),
        AggregationFunction::CountValues => format!(
            "COUNT(NULLIF({}, ''))::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregationFunction::Sum => format!(
            "SUM(NULLIF({}, '')::numeric)::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregationFunction::Average => format!(
            "AVG(NULLIF({}, '')::numeric)::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregationFunction::Min => format!(
            "MIN({})::text AS {key}",
            aggregation_source_value_sql(metric)
        ),
        AggregationFunction::Max => format!(
            "MAX({})::text AS {key}",
            aggregation_source_value_sql(metric)
        ),
    }
}

fn aggregation_source_value_sql(metric: &ValidatedAggregationMetric) -> String {
    let source = quote_identifier(metric.source_field_key.as_deref().unwrap_or_default());
    typed_orderable_sql(&source, &metric.field_type)
}

impl AggregationFunction {
    fn parse(value: &str) -> ApiResult<Self> {
        match value {
            "count_rows" => Ok(Self::CountRows),
            "count_values" => Ok(Self::CountValues),
            "sum" => Ok(Self::Sum),
            "average" => Ok(Self::Average),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            other => Err(ApiError::BadRequest(format!(
                "unsupported aggregation function '{other}'"
            ))),
        }
    }

    fn requires_source_field(self) -> bool {
        !matches!(self, Self::CountRows)
    }

    fn validate_field_type(self, field_type: &str, metric_key: &str) -> ApiResult<()> {
        let allowed = match self {
            Self::Sum | Self::Average => matches!(field_type, "number"),
            Self::Min | Self::Max => {
                matches!(
                    field_type,
                    "number" | "date" | "datetime" | "timestamp" | "single_choice" | "multi_choice"
                )
            }
            Self::CountRows | Self::CountValues => true,
        };
        if allowed {
            Ok(())
        } else {
            Err(ApiError::BadRequest(format!(
                "aggregation metric '{metric_key}' cannot use field type '{field_type}'"
            )))
        }
    }

    fn output_field_type(self, source: Option<&ValidatedDatasetField>) -> String {
        match self {
            Self::CountRows | Self::CountValues | Self::Sum | Self::Average => "number".into(),
            Self::Min | Self::Max => source
                .map(|field| field.field_type.clone())
                .unwrap_or_else(|| "text".into()),
        }
    }
}

fn dataset_field_definition(field: &ValidatedDatasetField) -> DatasetFieldDefinition {
    DatasetFieldDefinition {
        id: field.id.unwrap_or_else(Uuid::nil),
        key: field.key.clone(),
        label: field.label.clone(),
        source_alias: field.source_alias.clone(),
        source_field_key: field.source_field_key.clone(),
        field_type: field.field_type.clone(),
        position: field.position,
    }
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
                (dataset_id, source_alias, form_id, form_version_id, dataset_revision_id, position)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(dataset_id)
        .bind(&source.source_alias)
        .bind(source.form_id)
        .bind(source.form_version_id)
        .bind(source.dataset_revision_id)
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
                (dataset_id, key, label, source_alias, source_field_key, source_field_id, field_type, position)
            VALUES ($1, $2, $3, $4, $5, $6, $7::field_type, $8)
            "#,
        )
        .bind(dataset_id)
        .bind(&field.key)
        .bind(&field.label)
        .bind(&field.source_alias)
        .bind(&field.source_field_key)
        .bind(field.source_field_id)
        .bind(&field.field_type)
        .bind(field.position)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn insert_dataset_revision(
    tx: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    version_label: &str,
    compiled: &CompiledDataset,
) -> ApiResult<Uuid> {
    let version_number: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_number), 0) + 1 FROM dataset_revisions WHERE dataset_id = $1",
    )
    .bind(dataset_id)
    .fetch_one(&mut **tx)
    .await?;
    sqlx::query(
        r#"
        UPDATE dataset_revisions
        SET status = 'superseded'::dataset_revision_status
        WHERE dataset_id = $1
          AND status = 'published'::dataset_revision_status
        "#,
    )
    .bind(dataset_id)
    .execute(&mut **tx)
    .await?;
    let revision_id = sqlx::query_scalar(
        r#"
        INSERT INTO dataset_revisions
            (dataset_id, version_number, version_label, status, published_at, initial_source, operations, restriction_policy, generated_sql)
        VALUES ($1, $2, $3, 'published'::dataset_revision_status, now(), $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(dataset_id)
    .bind(version_number)
    .bind(version_label)
    .bind(serde_json::to_value(&compiled.initial_source).map_err(|error| {
        ApiError::Internal(anyhow::anyhow!(
            "dataset initial source could not be serialized: {error}"
        ))
    })?)
    .bind(serde_json::to_value(&compiled.operations).map_err(|error| {
        ApiError::Internal(anyhow::anyhow!(
            "dataset operations could not be serialized: {error}"
        ))
    })?)
    .bind(
        compiled
            .restriction_policy
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| {
                ApiError::Internal(anyhow::anyhow!(
                    "dataset restriction policy could not be serialized: {error}"
                ))
            })?,
    )
    .bind(&compiled.generated_sql)
    .fetch_one(&mut **tx)
    .await?;

    Ok(revision_id)
}

async fn materialize_dataset_revision(
    tx: &mut Transaction<'_, Postgres>,
    revision_id: Uuid,
    compiled: &CompiledDataset,
) -> ApiResult<()> {
    let table_name = format!("dataset_{}", revision_id.simple());
    let full_name = format!(
        "{}.{}",
        quote_identifier("dataset_materialized"),
        quote_identifier(&table_name)
    );
    sqlx::query(&format!("DROP TABLE IF EXISTS {full_name}"))
        .execute(&mut **tx)
        .await?;
    sqlx::query(&format!(
        "CREATE TABLE {full_name} AS {}",
        compiled.generated_sql
    ))
    .execute(&mut **tx)
    .await?;
    sqlx::query(&format!("CREATE INDEX ON {full_name} (__row_id)"))
        .execute(&mut **tx)
        .await?;
    let row_count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {full_name}"))
        .fetch_one(&mut **tx)
        .await?;
    sqlx::query(
        r#"
        UPDATE dataset_revisions
        SET materialized_schema = 'dataset_materialized',
            materialized_table = $1,
            materialized_row_count = $2,
            materialized_at = now()
        WHERE id = $3
        "#,
    )
    .bind(&table_name)
    .bind(row_count)
    .bind(revision_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn load_dataset_revision_output_fields(
    pool: &sqlx::PgPool,
    dataset_revision_id: Uuid,
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let revision = sqlx::query(
        r#"
        SELECT dataset_id
        FROM dataset_revisions
        WHERE id = $1
        "#,
    )
    .bind(dataset_revision_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ApiError::BadRequest(format!(
            "dataset revision {dataset_revision_id} is not available"
        ))
    })?;
    let dataset_id: Uuid = revision.try_get("dataset_id")?;
    let fields = sqlx::query(
        r#"
        SELECT id, key, label, source_alias, source_field_key, source_field_id, field_type::text AS field_type, position
        FROM dataset_fields
        WHERE dataset_id = $1
        ORDER BY position, key
        "#,
    )
    .bind(dataset_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ValidatedDatasetField {
            id: row.try_get("id")?,
            key: row.try_get("key")?,
            label: row.try_get("label")?,
            source_alias: row.try_get("source_alias")?,
            source_field_key: row.try_get("source_field_key")?,
            source_field_id: row.try_get("source_field_id")?,
            field_type: row.try_get("field_type")?,
            position: row.try_get("position")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;
    Ok(fields)
}

fn system_source_field_expression(source_field_key: &str) -> Option<&'static str> {
    match source_field_key {
        "__submission_id" => Some("submission_fact.submission_id::text"),
        "__form_version_id" => Some("submission_fact.form_version_id::text"),
        "__node_id" => Some("submission_fact.node_id::text"),
        "__node_name" => Some("node_dim.node_name"),
        "__submission_status" => Some("submission_fact.status"),
        "__submitted_at" => Some("submission_fact.submitted_at::text"),
        "__submission_created_at" => Some("submission_fact.created_at::text"),
        "__last_updated_at" => Some("submission_fact.last_modified_at::text"),
        "__last_updated_by_user_name" => Some("submission_fact.last_modified_by_user_name"),
        _ => None,
    }
}

fn system_source_field_keys() -> [&'static str; 9] {
    [
        "__submission_id",
        "__form_version_id",
        "__node_id",
        "__node_name",
        "__submission_status",
        "__submitted_at",
        "__submission_created_at",
        "__last_updated_at",
        "__last_updated_by_user_name",
    ]
}

fn system_source_field_label(source_field_key: &str) -> &'static str {
    match source_field_key {
        "__submission_id" => "Submission ID",
        "__form_version_id" => "Form Version ID",
        "__node_id" => "Attached Node ID",
        "__node_name" => "Attached Node Name",
        "__submission_status" => "Submission Status",
        "__submitted_at" => "Submitted Date",
        "__submission_created_at" => "Created Date",
        "__last_updated_at" => "Updated Date",
        "__last_updated_by_user_name" => "Updated By User Name",
        _ => "System Field",
    }
}

fn system_source_field_type(source_field_key: &str) -> Option<&'static str> {
    match source_field_key {
        "__submitted_at" | "__submission_created_at" | "__last_updated_at" => Some("date"),
        "__submission_status" => Some("single_choice"),
        "__submission_id"
        | "__form_version_id"
        | "__node_id"
        | "__node_name"
        | "__last_updated_by_user_name" => Some("text"),
        _ => None,
    }
}

pub(crate) async fn load_dataset_table_rows(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    dataset_id: Uuid,
) -> ApiResult<Vec<DatasetTableRow>> {
    load_materialized_dataset_table_rows(pool, account, dataset_id).await
}

async fn load_materialized_dataset_table_rows(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    dataset_id: Uuid,
) -> ApiResult<Vec<DatasetTableRow>> {
    let Some(row) = sqlx::query(
        r#"
        SELECT materialized_schema, materialized_table
        FROM dataset_revisions
        WHERE dataset_id = $1
          AND status = 'published'::dataset_revision_status
          AND materialized_table IS NOT NULL
        "#,
    )
    .bind(dataset_id)
    .fetch_optional(pool)
    .await?
    else {
        return Err(ApiError::BadRequest(
            "dataset preview requires a materialized published revision".into(),
        ));
    };
    let schema: String = row.try_get("materialized_schema")?;
    let table: String = row.try_get("materialized_table")?;
    let full_name = format!("{}.{}", quote_identifier(&schema), quote_identifier(&table));
    let rows = sqlx::query(&format!(
        "SELECT * FROM {full_name} WHERE {} ORDER BY __row_id LIMIT 200",
        tier_access_predicate(account)
    ))
    .fetch_all(pool)
    .await?;
    let mut table_rows = Vec::new();
    for row in rows {
        let submission_id: String = row.try_get("__row_id")?;
        let mut values = BTreeMap::new();
        for column in row.columns() {
            let name = column.name();
            if name.starts_with("__") {
                continue;
            }
            let value: Option<String> = row.try_get(name)?;
            values.insert(name.to_string(), value);
        }
        table_rows.push(DatasetTableRow {
            submission_id,
            node_name: String::new(),
            source_alias: "materialized".into(),
            values,
        });
    }
    Ok(table_rows)
}

fn tier_access_predicate(account: &auth::AccountContext) -> &'static str {
    if account.has_capability("admin:all") || account.has_capability("datasets:read_confidential") {
        "TRUE"
    } else if account.has_capability("datasets:read_restricted") {
        "COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal', 'restricted')"
    } else {
        "COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal')"
    }
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

async fn require_node_ids_exist(pool: &sqlx::PgPool, node_ids: &[Uuid]) -> ApiResult<()> {
    if node_ids.is_empty() {
        return Err(ApiError::BadRequest(
            "at least one visibility node is required".into(),
        ));
    }
    let existing_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE id = ANY($1)")
        .bind(node_ids)
        .fetch_one(pool)
        .await?;
    if existing_count as usize == node_ids.len() {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "one or more visibility nodes do not exist".into(),
        ))
    }
}

async fn replace_dataset_scope_nodes_tx(
    tx: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    node_ids: &[Uuid],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM dataset_scope_nodes WHERE dataset_id = $1")
        .bind(dataset_id)
        .execute(&mut **tx)
        .await?;
    for node_id in node_ids {
        sqlx::query(
            "INSERT INTO dataset_scope_nodes (dataset_id, node_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(dataset_id)
        .bind(node_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn load_dataset_visibility_nodes(
    pool: &sqlx::PgPool,
    dataset_ids: &[Uuid],
    visible_node_filter: Option<&[Uuid]>,
) -> ApiResult<BTreeMap<Uuid, Vec<DatasetVisibilityNodeSummary>>> {
    if dataset_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let rows = if let Some(node_ids) = visible_node_filter {
        sqlx::query(
            r#"
            SELECT
                dataset_scope_nodes.dataset_id,
                nodes.id AS node_id,
                nodes.name AS node_name,
                nodes.parent_node_id,
                node_types.name AS node_type_name,
                COALESCE(parent_nodes.name || ' / ' || nodes.name, nodes.name) AS node_path
            FROM dataset_scope_nodes
            JOIN nodes ON nodes.id = dataset_scope_nodes.node_id
            JOIN node_types ON node_types.id = nodes.node_type_id
            LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
            WHERE dataset_scope_nodes.dataset_id = ANY($1)
              AND dataset_scope_nodes.node_id = ANY($2)
            ORDER BY node_path, nodes.name, nodes.id
            "#,
        )
        .bind(dataset_ids)
        .bind(node_ids)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                dataset_scope_nodes.dataset_id,
                nodes.id AS node_id,
                nodes.name AS node_name,
                nodes.parent_node_id,
                node_types.name AS node_type_name,
                COALESCE(parent_nodes.name || ' / ' || nodes.name, nodes.name) AS node_path
            FROM dataset_scope_nodes
            JOIN nodes ON nodes.id = dataset_scope_nodes.node_id
            JOIN node_types ON node_types.id = nodes.node_type_id
            LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
            WHERE dataset_scope_nodes.dataset_id = ANY($1)
            ORDER BY node_path, nodes.name, nodes.id
            "#,
        )
        .bind(dataset_ids)
        .fetch_all(pool)
        .await?
    };
    let mut visibility_nodes = BTreeMap::<Uuid, Vec<DatasetVisibilityNodeSummary>>::new();
    for row in rows {
        let dataset_id: Uuid = row.try_get("dataset_id")?;
        visibility_nodes
            .entry(dataset_id)
            .or_default()
            .push(DatasetVisibilityNodeSummary {
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                node_type_name: row.try_get("node_type_name")?,
                parent_node_id: row.try_get("parent_node_id")?,
                node_path: row.try_get("node_path")?,
            });
    }
    Ok(visibility_nodes)
}

async fn require_dataset_fully_in_capability_scope(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    capability: &str,
    dataset_id: Uuid,
) -> ApiResult<()> {
    let node_ids = load_dataset_scope_node_ids(pool, dataset_id).await?;
    auth::require_capability_contains_nodes(pool, account, capability, &node_ids).await
}

async fn require_dataset_visible_for_boundary(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
    boundary: &auth::CapabilityBoundary,
    capability: &str,
) -> ApiResult<()> {
    match boundary {
        auth::CapabilityBoundary::Global => Ok(()),
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            let node_ids = load_dataset_scope_node_ids(pool, dataset_id).await?;
            if node_ids.iter().any(|node_id| scope_ids.contains(node_id)) {
                Ok(())
            } else {
                Err(ApiError::Forbidden(capability.into()))
            }
        }
        auth::CapabilityBoundary::None => Err(ApiError::Forbidden(capability.into())),
    }
}

async fn require_dataset_visible_for_account(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    dataset_id: Uuid,
) -> ApiResult<()> {
    let boundary = auth::capability_boundary(pool, account, "datasets:read").await?;
    require_dataset_visible_for_boundary(pool, dataset_id, &boundary, "datasets:read").await
}

pub(crate) async fn load_dataset_scope_node_ids(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
) -> ApiResult<Vec<Uuid>> {
    sqlx::query_scalar(
        "SELECT node_id FROM dataset_scope_nodes WHERE dataset_id = $1 ORDER BY node_id",
    )
    .bind(dataset_id)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
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

async fn require_form_readable_by_account(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    form_id: Uuid,
) -> ApiResult<()> {
    match auth::capability_boundary(pool, account, "forms:read").await? {
        auth::CapabilityBoundary::Global => Ok(()),
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            let visible: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM form_scope_nodes
                    WHERE form_id = $1
                      AND node_id = ANY($2)
                )
                "#,
            )
            .bind(form_id)
            .bind(scope_ids)
            .fetch_one(pool)
            .await?;
            if visible {
                Ok(())
            } else {
                Err(ApiError::Forbidden("forms:read".into()))
            }
        }
        auth::CapabilityBoundary::None => Err(ApiError::Forbidden("forms:read".into())),
    }
}

async fn load_published_form_version_identity(
    pool: &sqlx::PgPool,
    form_id: Uuid,
    form_version_id: Uuid,
) -> ApiResult<PublishedFormVersionIdentity> {
    let row = sqlx::query(
        r#"
        SELECT id
        FROM form_versions
        WHERE id = $1
          AND form_id = $2
          AND status = 'published'::form_version_status
        "#,
    )
    .bind(form_version_id)
    .bind(form_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ApiError::BadRequest(format!(
            "dataset form source must reference published form version {form_version_id} for form {form_id}"
        ))
    })?;

    Ok(PublishedFormVersionIdentity {
        id: row.try_get("id")?,
    })
}

fn require_identifier(label: &str, value: &str) -> ApiResult<()> {
    let valid = !value.is_empty()
        && value.len() <= 63
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        && value
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic() || character == '_');
    if valid {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} must use only letters, numbers, and underscores, and must start with a letter or underscore"
        )))
    }
}

fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validated_field(key: &str, field_type: &str) -> ValidatedDatasetField {
        ValidatedDatasetField {
            id: None,
            key: key.into(),
            label: key.into(),
            source_alias: "program".into(),
            source_field_key: key.into(),
            source_field_id: None,
            field_type: field_type.into(),
            position: 0,
        }
    }

    fn projection_field(key: &str, input_field_key: &str) -> DatasetProjectionFieldRequest {
        DatasetProjectionFieldRequest {
            key: key.into(),
            label: key.into(),
            input_field_key: Some(input_field_key.into()),
            position: 0,
        }
    }

    fn projection_operation(fields: Vec<DatasetProjectionFieldRequest>) -> DatasetOperationRequest {
        DatasetOperationRequest::Projection {
            fields: fields
                .into_iter()
                .enumerate()
                .map(|(position, mut field)| {
                    field.position = position as i32;
                    field
                })
                .collect(),
            position: 0,
        }
    }

    fn calculation_function(
        function: &str,
        argument: Option<&str>,
    ) -> dto::DatasetCalculationFunctionRequest {
        dto::DatasetCalculationFunctionRequest {
            function: function.into(),
            argument: argument.map(str::to_string),
            argument_mode: "value".into(),
            argument_field_key: None,
            position: 0,
        }
    }

    fn calculated_field_operation(
        key: &str,
        base_field_key: &str,
        function: &str,
        argument: Option<&str>,
    ) -> DatasetOperationRequest {
        DatasetOperationRequest::CalculatedFields {
            fields: vec![DatasetCalculatedFieldRequest {
                key: key.into(),
                label: key.into(),
                base_field_key: base_field_key.into(),
                functions: vec![calculation_function(function, argument)],
                position: 0,
            }],
            position: 0,
        }
    }

    fn filter_operation(
        field_key: &str,
        operator: &str,
        value: Option<&str>,
    ) -> DatasetOperationRequest {
        DatasetOperationRequest::Filter {
            filters: vec![DatasetRowFilterRequest {
                field_key: field_key.into(),
                operator: operator.into(),
                value_mode: "value".into(),
                value: value.map(str::to_string),
                value_field_key: None,
                position: 0,
            }],
            position: 0,
        }
    }

    fn count_rows_aggregation_operation(key: &str) -> DatasetOperationRequest {
        DatasetOperationRequest::Aggregation {
            group_fields: Vec::new(),
            metrics: vec![dto::DatasetAggregationMetricRequest {
                key: key.into(),
                label: key.into(),
                function: "count_rows".into(),
                source_field_key: None,
                position: 0,
            }],
            row_picker: None,
            position: 0,
        }
    }

    #[test]
    fn date_calculation_comparison_uses_field_argument_and_date_casts() {
        let function = ValidatedCalculationFunction {
            function: CalculationFunction::LessThanOrEqual,
            argument: None,
            argument_field_key: Some("program2__session_date".into()),
            input_type: "date".into(),
        };

        assert_eq!(
            calculation_function_sql(&function, "\"program__session_date\""),
            "CASE WHEN NULLIF(\"program__session_date\", '')::date <= NULLIF(\"program2__session_date\", '')::date THEN 'true' ELSE 'false' END"
        );
    }

    #[test]
    fn date_format_calculation_uses_date_cast_for_date_input() {
        let function = ValidatedCalculationFunction {
            function: CalculationFunction::FormatDate,
            argument: Some("%Y-%m-%d".into()),
            argument_field_key: None,
            input_type: "date".into(),
        };

        assert_eq!(
            calculation_function_sql(&function, "\"program__session_date\""),
            "TO_CHAR(NULLIF(\"program__session_date\", '')::date, '%Y-%m-%d')"
        );
    }

    #[test]
    fn invalid_date_filter_literals_are_rejected_before_sql_generation() {
        assert!(validate_filter_literal("date", "2026-06-03").is_ok());
        assert!(validate_filter_literal("date", "06/03/2026").is_err());
        assert!(validate_filter_literal("datetime", "2026-06-03T13:45").is_ok());
        assert!(validate_filter_literal("datetime", "not-a-date").is_err());
    }

    #[test]
    fn invalid_boolean_filter_literals_are_rejected_before_sql_generation() {
        assert!(validate_filter_literal("boolean", "yes").is_ok());
        assert!(validate_filter_literal("boolean", "0").is_ok());
        assert!(validate_filter_literal("boolean", "maybe").is_err());
    }

    #[test]
    fn numeric_min_max_aggregation_uses_numeric_casts() {
        let min_metric = ValidatedAggregationMetric {
            key: "minimum_target".into(),
            label: "Minimum Target".into(),
            function: AggregationFunction::Min,
            source_field_key: Some("program__participant_target".into()),
            field_type: "number".into(),
            position: 0,
        };
        let max_metric = ValidatedAggregationMetric {
            function: AggregationFunction::Max,
            key: "maximum_target".into(),
            label: "Maximum Target".into(),
            source_field_key: Some("program__participant_target".into()),
            field_type: "number".into(),
            position: 1,
        };

        assert_eq!(
            aggregation_metric_sql(&min_metric),
            "MIN(NULLIF(\"program__participant_target\", '')::numeric)::text AS \"minimum_target\""
        );
        assert_eq!(
            aggregation_metric_sql(&max_metric),
            "MAX(NULLIF(\"program__participant_target\", '')::numeric)::text AS \"maximum_target\""
        );
    }

    #[test]
    fn row_picker_ordering_uses_typed_expression() {
        assert_eq!(
            typed_orderable_sql("\"program__participant_target\"", "number"),
            "NULLIF(\"program__participant_target\", '')::numeric"
        );
        assert_eq!(
            typed_orderable_sql("\"program__review_started\"", "date"),
            "NULLIF(\"program__review_started\", '')::date"
        );
    }

    #[test]
    fn map_value_calculation_parses_multiple_pairs() {
        let function = ValidatedCalculationFunction {
            function: CalculationFunction::MapValue,
            argument: Some("draft=>booger, submitted=snot".into()),
            argument_field_key: None,
            input_type: "text".into(),
        };

        let sql = calculation_function_sql(&function, "\"program__submission_status\"");

        assert!(
            sql.contains(
                "WHEN COALESCE(\"program__submission_status\", '') = 'draft' THEN 'booger'"
            )
        );
        assert!(sql.contains(
            "WHEN COALESCE(\"program__submission_status\", '') = 'submitted' THEN 'snot'"
        ));
        assert!(sql.ends_with("ELSE \"program__submission_status\" END"));
    }

    #[test]
    fn numeric_row_filter_can_compare_against_another_field() {
        let filter = ValidatedRowFilter {
            field_key: "program__participants".into(),
            field_type: "number".into(),
            operator: RowFilterOperator::GreaterThanOrEqual,
            value: None,
            value_field_key: Some("program2__participants".into()),
        };

        assert_eq!(
            row_filter_sql(&filter),
            "NULLIF(\"program__participants\", '')::numeric >= NULLIF(\"program2__participants\", '')::numeric"
        );
    }

    #[test]
    fn row_filter_equality_uses_field_type_casts() {
        let numeric_filter = ValidatedRowFilter {
            field_key: "program__participants".into(),
            field_type: "number".into(),
            operator: RowFilterOperator::Equals,
            value: Some("135.0".into()),
            value_field_key: None,
        };
        let date_filter = ValidatedRowFilter {
            field_key: "program__review_window_start".into(),
            field_type: "date".into(),
            operator: RowFilterOperator::Equals,
            value: Some("2026-06-03".into()),
            value_field_key: None,
        };

        assert!(row_filter_sql(&numeric_filter).contains("::numeric"));
        let date_sql = row_filter_sql(&date_filter);
        assert!(date_sql.contains("::date"));
        assert!(!date_sql.contains("::timestamptz"));
    }

    #[test]
    fn numeric_calculation_equality_rejects_non_numeric_literals() {
        let result = CalculationFunction::Equal.validate_argument(
            &Some("george".into()),
            "number",
            "calculated_1",
        );

        assert!(result.is_err());
    }

    #[test]
    fn typed_default_literals_reject_invalid_values() {
        assert!(
            CalculationFunction::Constant
                .validate_argument(&Some("banana".into()), "number", "calculated_1")
                .is_err()
        );
        assert!(
            CalculationFunction::Coalesce
                .validate_argument(&Some("not-a-date".into()), "date", "calculated_1")
                .is_err()
        );
        assert!(
            CalculationFunction::Equal
                .validate_argument(&Some("maybe".into()), "boolean", "calculated_1")
                .is_err()
        );
    }

    #[test]
    fn coalesce_preserves_carry_forward_type_for_numeric_pipeline() {
        let fields = vec![validated_field("response_count", "number")];
        let calculated = validate_dataset_calculated_fields(
            vec![DatasetCalculatedFieldRequest {
                key: "defaulted_plus_one".into(),
                label: "Defaulted Plus One".into(),
                base_field_key: "response_count".into(),
                functions: vec![
                    dto::DatasetCalculationFunctionRequest {
                        function: "coalesce".into(),
                        argument: Some("0".into()),
                        argument_mode: "value".into(),
                        argument_field_key: None,
                        position: 0,
                    },
                    dto::DatasetCalculationFunctionRequest {
                        function: "add".into(),
                        argument: Some("1".into()),
                        argument_mode: "value".into(),
                        argument_field_key: None,
                        position: 1,
                    },
                ],
                position: 0,
            }],
            &fields,
        )
        .expect("calculated field should validate");

        assert_eq!(calculated[0].field_type, "number");
        let sql = calculated_field_sql(&calculated[0]);
        assert!(sql.contains("COALESCE(NULLIF(\"response_count\", ''), '0')"));
        assert!(sql.contains("::numeric + 1"));
    }

    #[test]
    fn boolean_equality_uses_nullable_boolean_normalization() {
        let filter = ValidatedRowFilter {
            field_key: "program__funding_confirmed".into(),
            field_type: "boolean".into(),
            operator: RowFilterOperator::Equals,
            value: Some("yes".into()),
            value_field_key: None,
        };

        let sql = row_filter_sql(&filter);

        assert!(sql.contains("CASE WHEN NULLIF(\"program__funding_confirmed\", '') IS NULL"));
        assert!(sql.contains("CASE WHEN NULLIF('yes', '') IS NULL"));
        assert!(sql.contains("LOWER(COALESCE(\"program__funding_confirmed\", '')) IN"));
        assert!(sql.contains("LOWER(COALESCE('yes', '')) IN"));
    }

    #[tokio::test]
    async fn query_spec_renders_ctes_in_saved_operation_order() {
        let field = validated_field("program__participant_target", "number");
        let mut spec = QuerySpecBuilder::new_for_test(
            vec![r#"program_1 AS (SELECT "__row_id", "program__participant_target")"#.into()],
            "program_1".into(),
            vec![field],
            1,
        );

        spec.apply_operations(&[
            DatasetOperationRequest::Projection {
                fields: vec![DatasetProjectionFieldRequest {
                    key: "program__participant_target".into(),
                    label: "Participant Target".into(),
                    input_field_key: Some("program__participant_target".into()),
                    position: 0,
                }],
                position: 0,
            },
            DatasetOperationRequest::Aggregation {
                group_fields: vec![],
                metrics: vec![dto::DatasetAggregationMetricRequest {
                    key: "target_average".into(),
                    label: "Target Average".into(),
                    function: "average".into(),
                    source_field_key: Some("program__participant_target".into()),
                    position: 0,
                }],
                row_picker: None,
                position: 1,
            },
            DatasetOperationRequest::CalculatedFields {
                fields: vec![DatasetCalculatedFieldRequest {
                    key: "target_plus_one".into(),
                    label: "Target Plus One".into(),
                    base_field_key: "target_average".into(),
                    functions: vec![dto::DatasetCalculationFunctionRequest {
                        function: "add".into(),
                        argument: Some("1".into()),
                        argument_mode: "value".into(),
                        argument_field_key: None,
                        position: 0,
                    }],
                    position: 0,
                }],
                position: 2,
            },
            DatasetOperationRequest::Filter {
                filters: vec![DatasetRowFilterRequest {
                    field_key: "target_plus_one".into(),
                    operator: "greater_than_or_equal".into(),
                    value_mode: "value".into(),
                    value: Some("100".into()),
                    value_field_key: None,
                    position: 0,
                }],
                position: 3,
            },
        ])
        .await
        .expect("operation pipeline should validate");
        let sql = spec.final_sql(&ValidatedRestrictionPolicy::default());

        let source = sql.find("program_1 AS").expect("source cte");
        let projection = sql.find("projection_2 AS").expect("projection cte");
        let aggregated = sql.find("aggregation_3 AS").expect("aggregation cte");
        let calculated = sql.find("calculated_fields_4 AS").expect("calculation cte");
        let filtered = sql.find("filtered_fields_5 AS").expect("filter cte");
        let final_select = sql.rfind("SELECT").expect("final select");
        assert!(source < projection);
        assert!(projection < aggregated);
        assert!(aggregated < calculated);
        assert!(calculated < filtered);
        assert!(filtered < final_select);
    }

    #[tokio::test]
    async fn projection_rejects_unavailable_input_keys_without_legacy_resolution() {
        let mut spec = QuerySpecBuilder::new_for_test(
            vec![r#"program_1 AS (SELECT "__row_id", "program__participant_target")"#.into()],
            "program_1".into(),
            vec![validated_field("program__participant_target", "number")],
            1,
        );

        let error = spec
            .apply_operations(&[projection_operation(vec![projection_field(
                "participant_target",
                "participant_target",
            )])])
            .await
            .expect_err("unprefixed input should not resolve to a canonical field")
            .to_string();

        assert!(
            error.contains(
                "projection field 'participant_target' references unavailable field 'participant_target'"
            )
        );
    }

    #[tokio::test]
    async fn query_spec_supports_repeated_operations_in_saved_order() {
        let mut spec = QuerySpecBuilder::new_for_test(
            vec![r#"source_1 AS (SELECT "__row_id", "a")"#.into()],
            "source_1".into(),
            vec![validated_field("a", "number")],
            1,
        );

        spec.apply_operations(&[
            projection_operation(vec![projection_field("a", "a")]),
            calculated_field_operation("a_plus_one", "a", "add", Some("1")),
            filter_operation("a_plus_one", "greater_than", Some("1")),
            projection_operation(vec![projection_field("renamed_a", "a_plus_one")]),
            calculated_field_operation("renamed_a_plus_one", "renamed_a", "add", Some("1")),
            filter_operation("renamed_a_plus_one", "less_than_or_equal", Some("10")),
            count_rows_aggregation_operation("row_count"),
            calculated_field_operation("row_count_plus_one", "row_count", "add", Some("1")),
        ])
        .await
        .expect("repeated operations should validate");
        let sql = spec.final_sql(&ValidatedRestrictionPolicy::default());

        let expected_order = [
            "source_1 AS",
            "projection_2 AS",
            "calculated_fields_3 AS",
            "filtered_fields_4 AS",
            "projection_5 AS",
            "calculated_fields_6 AS",
            "filtered_fields_7 AS",
            "aggregation_8 AS",
            "calculated_fields_9 AS",
            "FROM \"calculated_fields_9\"",
        ];
        let mut previous_index = 0;
        for needle in expected_order {
            let index = sql
                .find(needle)
                .unwrap_or_else(|| panic!("expected SQL to contain {needle}"));
            assert!(
                index >= previous_index,
                "{needle} should appear after the prior operation"
            );
            previous_index = index;
        }
        assert_eq!(
            spec.fields()
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["row_count", "row_count_plus_one"]
        );
    }

    #[tokio::test]
    async fn query_spec_rejects_references_removed_by_projection() {
        let mut spec = QuerySpecBuilder::new_for_test(
            vec![r#"source_1 AS (SELECT "__row_id", "kept", "removed")"#.into()],
            "source_1".into(),
            vec![
                validated_field("kept", "text"),
                validated_field("removed", "text"),
            ],
            1,
        );

        let error = spec
            .apply_operations(&[
                projection_operation(vec![projection_field("kept", "kept")]),
                filter_operation("removed", "is_not_empty", None),
            ])
            .await
            .expect_err("removed field should not be available after projection")
            .to_string();

        assert!(error.contains("row filter field 'removed' is not projected"));
    }

    #[tokio::test]
    async fn calculated_output_types_feed_later_filters_and_aggregations() {
        let mut spec = QuerySpecBuilder::new_for_test(
            vec![r#"source_1 AS (SELECT "__row_id", "score")"#.into()],
            "source_1".into(),
            vec![validated_field("score", "number")],
            1,
        );

        spec.apply_operations(&[
            projection_operation(vec![projection_field("score", "score")]),
            calculated_field_operation("score_plus_one", "score", "add", Some("1")),
            calculated_field_operation(
                "score_is_large",
                "score_plus_one",
                "greater_than",
                Some("10"),
            ),
            filter_operation("score_is_large", "equals", Some("true")),
            DatasetOperationRequest::Aggregation {
                group_fields: Vec::new(),
                metrics: vec![dto::DatasetAggregationMetricRequest {
                    key: "average_score".into(),
                    label: "Average Score".into(),
                    function: "average".into(),
                    source_field_key: Some("score_plus_one".into()),
                    position: 0,
                }],
                row_picker: None,
                position: 3,
            },
        ])
        .await
        .expect("calculated field types should feed later validation");

        assert_eq!(spec.fields()[0].key, "average_score");
        assert_eq!(spec.fields()[0].field_type, "number");
    }

    #[tokio::test]
    async fn query_spec_rejects_duplicate_projection_output_keys() {
        let mut spec = QuerySpecBuilder::new_for_test(
            vec![r#"source_1 AS (SELECT "__row_id", "a", "b")"#.into()],
            "source_1".into(),
            vec![validated_field("a", "text"), validated_field("b", "text")],
            1,
        );

        let error = spec
            .apply_operations(&[projection_operation(vec![
                projection_field("dup", "a"),
                projection_field("dup", "b"),
            ])])
            .await
            .expect_err("duplicate projection keys should fail")
            .to_string();

        assert!(error.contains("projection field key 'dup' is duplicated"));
    }

    #[tokio::test]
    async fn query_spec_rejects_duplicate_projection_input_keys() {
        let mut spec = QuerySpecBuilder::new_for_test(
            vec![r#"source_1 AS (SELECT "__row_id", "a")"#.into()],
            "source_1".into(),
            vec![validated_field("a", "text")],
            1,
        );

        let error = spec
            .apply_operations(&[projection_operation(vec![
                projection_field("a", "a"),
                projection_field("a_copy", "a"),
            ])])
            .await
            .expect_err("duplicate projection inputs should fail")
            .to_string();

        assert!(error.contains("projection input field 'a' is duplicated"));
    }

    #[tokio::test]
    async fn restrictions_render_after_all_operations_and_keep_restriction_fields_available() {
        let restriction_flag = validated_field("internal_flag", "boolean");
        let mut spec = QuerySpecBuilder::new_for_test(
            vec![r#"source_1 AS (SELECT "__row_id", "title", "internal_flag")"#.into()],
            "source_1".into(),
            vec![validated_field("title", "text"), restriction_flag],
            1,
        );

        spec.apply_operations(&[
            projection_operation(vec![
                projection_field("title", "title"),
                projection_field("internal_flag", "internal_flag"),
            ]),
            filter_operation("title", "contains", Some("Ready")),
        ])
        .await
        .expect("restriction source field should remain available after final operation");
        let sql = spec.final_sql(&ValidatedRestrictionPolicy {
            internal_field_key: Some("internal_flag".into()),
            restricted_field_key: None,
            confidential_field_key: None,
        });

        assert!(
            sql.find("filtered_fields_3 AS").expect("filter cte")
                < sql.find("__restriction_tier").expect("restriction select")
        );
        assert!(sql.contains("WHEN LOWER(COALESCE(\"internal_flag\", '')) IN"));
        assert!(sql.contains("\"internal_flag\""));
        assert!(
            spec.fields()
                .iter()
                .any(|field| field.key == "internal_flag")
        );
    }
}
