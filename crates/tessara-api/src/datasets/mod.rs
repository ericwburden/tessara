//! Dataset definition, visibility, and table execution endpoints.
//!
//! Datasets are row-level analytical assets. This module owns validation,
//! visibility enforcement, and table execution; public request/response shapes
//! live in `dto`.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    future::Future,
    pin::Pin,
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
};
use sqlx::{Column, Postgres, Row, Transaction};
use tessara_datasets::{
    DatasetCompositionMode, DatasetGrain, DatasetSelectionRule, validate_dataset_shape,
};
use uuid::Uuid;

mod dto;

pub use dto::{
    CreateDatasetFieldRequest, CreateDatasetRequest, DatasetAggregationRequest,
    DatasetAggregationResponse, DatasetDefinition, DatasetExpressionRequest,
    DatasetFieldDefinition, DatasetRowPickerRequest, DatasetSourceDefinition, DatasetSqlPreview,
    DatasetSummary, DatasetTable, DatasetTableRow, DatasetVisibilityNodeSummary,
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
    form_version_major: Option<i32>,
    dataset_revision_id: Option<Uuid>,
    selection_rule: DatasetSelectionRule,
    position: i32,
}

#[derive(Clone)]
struct ValidatedDatasetField {
    id: Option<Uuid>,
    key: String,
    label: String,
    source_alias: String,
    source_field_key: String,
    field_type: String,
    position: i32,
}

struct CompiledDataset {
    definition_ast: DatasetExpressionRequest,
    aggregation: Option<DatasetAggregationRequest>,
    generated_sql: String,
    sources: Vec<ValidatedDatasetSource>,
    fields: Vec<ValidatedDatasetField>,
}

struct QueryCompiler<'a> {
    pool: &'a sqlx::PgPool,
    account: &'a auth::AccountContext,
    dataset_id: Option<Uuid>,
    output_fields: &'a [CreateDatasetFieldRequest],
    aggregation: Option<ValidatedAggregation>,
    field_keys: Vec<String>,
    ctes: Vec<String>,
    cte_index: usize,
    aliases: BTreeSet<String>,
    sources: Vec<ValidatedDatasetSource>,
}

#[derive(Clone)]
struct ValidatedAggregation {
    group_fields: Vec<String>,
    metrics: Vec<ValidatedAggregationMetric>,
    row_picker: Option<DatasetRowPickerRequest>,
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

#[derive(Clone, Copy)]
enum AggregationFunction {
    CountRows,
    CountValues,
    Sum,
    Average,
    Min,
    Max,
}

struct CompiledExpression {
    cte_name: String,
    columns: BTreeSet<String>,
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
        compile_dataset_definition(&state.pool, &account, dataset_id, &payload, &payload.fields)
            .await?;
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
    let composition_mode = top_level_composition_mode(&payload)?;
    let field_requests = payload.fields.clone();

    let mut tx = state.pool.begin().await?;
    let dataset_id: Uuid = sqlx::query_scalar(
        "INSERT INTO datasets (name, slug, grain, composition_mode) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&payload.name)
    .bind(&payload.slug)
    .bind(grain.as_str())
    .bind(composition_mode.as_str())
    .fetch_one(&mut *tx)
    .await?;

    let compiled = compile_dataset_definition(
        &state.pool,
        &account,
        Some(dataset_id),
        &payload,
        &field_requests,
    )
    .await?;
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
    let composition_mode = top_level_composition_mode(&payload)?;
    let field_requests = payload.fields.clone();
    let compiled = compile_dataset_definition(
        &state.pool,
        &account,
        Some(dataset_id),
        &payload,
        &field_requests,
    )
    .await?;

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
            datasets.composition_mode,
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
            datasets.composition_mode,
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
            datasets.composition_mode,
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
            datasets.composition_mode,
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
                composition_mode: row.try_get("composition_mode")?,
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
               datasets.name, datasets.slug, datasets.grain, datasets.composition_mode,
               current_revisions.definition_ast,
               current_revisions.aggregation,
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
            dataset_sources.form_version_major,
            dataset_sources.dataset_revision_id,
            dataset_sources.selection_rule,
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
        SELECT id, key, label, source_alias, source_field_key, field_type::text AS field_type, position
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
    let aggregation_request = dataset
        .try_get::<Option<serde_json::Value>, _>("aggregation")?
        .map(serde_json::from_value::<DatasetAggregationRequest>)
        .transpose()
        .map_err(|error| {
            ApiError::Internal(anyhow::anyhow!(
                "stored dataset aggregation is invalid: {error}"
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
                field_type: row.try_get("field_type")?,
                position: row.try_get("position")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let aggregation = validate_dataset_aggregation(aggregation_request.clone(), &fields)?;
    let output_fields = output_fields_for_dataset(&fields, aggregation.as_ref());

    Ok(Json(DatasetDefinition {
        id: dataset.try_get("id")?,
        current_revision_id: dataset.try_get("current_revision_id")?,
        name: dataset.try_get("name")?,
        slug: dataset.try_get("slug")?,
        grain: dataset.try_get("grain")?,
        composition_mode: dataset.try_get("composition_mode")?,
        definition_ast: dataset
            .try_get::<Option<serde_json::Value>, _>("definition_ast")?
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| {
                ApiError::Internal(anyhow::anyhow!(
                    "stored dataset definition is invalid: {error}"
                ))
            })?,
        aggregation: aggregation_response_from_validated(aggregation_request, aggregation),
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
                    form_version_major: row.try_get("form_version_major")?,
                    dataset_revision_id: row.try_get("dataset_revision_id")?,
                    selection_rule: row.try_get("selection_rule")?,
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
    let table_rows = load_dataset_table_rows(&state.pool, dataset_id).await?;

    Ok(Json(DatasetTable {
        dataset_id,
        rows: table_rows,
    }))
}

fn top_level_composition_mode(payload: &CreateDatasetRequest) -> ApiResult<DatasetCompositionMode> {
    if let DatasetExpressionRequest::Operation { operation, .. } = &payload.definition_ast {
        return DatasetCompositionMode::parse(operation)
            .map_err(|error| ApiError::BadRequest(error.to_string()));
    }
    DatasetCompositionMode::parse(&payload.composition_mode)
        .map_err(|error| ApiError::BadRequest(error.to_string()))
}

async fn compile_dataset_definition(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    dataset_id: Option<Uuid>,
    payload: &CreateDatasetRequest,
    field_requests: &[CreateDatasetFieldRequest],
) -> ApiResult<CompiledDataset> {
    let ast = payload.definition_ast.clone();
    validate_dataset_shape(
        collect_expression_aliases(&ast),
        field_requests.iter().map(|field| field.key.as_str()),
    )
    .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let mut compiler = QueryCompiler {
        pool,
        account,
        dataset_id,
        output_fields: field_requests,
        aggregation: None,
        field_keys: field_requests
            .iter()
            .map(|field| field.key.clone())
            .collect(),
        ctes: Vec::new(),
        cte_index: 0,
        aliases: BTreeSet::new(),
        sources: Vec::new(),
    };
    let root = compiler.compile_expression(&ast).await?;
    let fields = validate_dataset_fields(pool, &compiler.sources, field_requests.to_vec()).await?;
    let aggregation = validate_dataset_aggregation(payload.aggregation.clone(), &fields)?;
    compiler.aggregation = aggregation.clone();
    let generated_sql = compiler.final_sql(&root);
    Ok(CompiledDataset {
        definition_ast: ast,
        aggregation: payload.aggregation.clone(),
        generated_sql,
        sources: compiler.sources,
        fields,
    })
}

fn collect_expression_aliases(ast: &DatasetExpressionRequest) -> Vec<&str> {
    match ast {
        DatasetExpressionRequest::Form { alias, .. }
        | DatasetExpressionRequest::Dataset { alias, .. } => vec![alias.as_str()],
        DatasetExpressionRequest::Operation { left, right, .. } => {
            let mut aliases = Vec::new();
            aliases.extend(collect_expression_aliases(left));
            aliases.extend(collect_expression_aliases(right));
            aliases
        }
    }
}

impl<'a> QueryCompiler<'a> {
    fn compile_expression<'b>(
        &'b mut self,
        ast: &'b DatasetExpressionRequest,
    ) -> Pin<Box<dyn Future<Output = ApiResult<CompiledExpression>> + Send + 'b>> {
        Box::pin(async move {
            match ast {
                DatasetExpressionRequest::Form {
                    alias,
                    form_id,
                    form_version_major,
                    selection_rule,
                } => {
                    self.compile_form_source(alias, *form_id, *form_version_major, selection_rule)
                        .await
                }
                DatasetExpressionRequest::Dataset {
                    alias,
                    dataset_id,
                    dataset_revision_id,
                } => {
                    self.compile_dataset_source(alias, *dataset_id, *dataset_revision_id)
                        .await
                }
                DatasetExpressionRequest::Operation {
                    operation,
                    left,
                    right,
                    join_keys,
                    ..
                } => {
                    let left = self.compile_expression(left).await?;
                    let right = self.compile_expression(right).await?;
                    self.compile_operation(operation, left, right, join_keys)
                }
            }
        })
    }

    async fn compile_form_source(
        &mut self,
        alias: &str,
        form_id: Uuid,
        form_version_major: Option<i32>,
        selection_rule: &str,
    ) -> ApiResult<CompiledExpression> {
        require_identifier("dataset source alias", alias)?;
        if form_id.is_nil() {
            return Err(ApiError::BadRequest(
                "dataset form source must reference a form".into(),
            ));
        }
        if !self.aliases.insert(alias.to_string()) {
            return Err(ApiError::BadRequest(format!(
                "dataset expression alias '{alias}' is duplicated"
            )));
        }
        let selection_rule = DatasetSelectionRule::parse(selection_rule)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        require_form_exists(self.pool, form_id).await?;
        require_form_readable_by_account(self.pool, self.account, form_id).await?;
        let form_version_major = match form_version_major {
            Some(value) => Some(value),
            None => load_latest_published_form_major(self.pool, form_id).await?,
        };
        let source = ValidatedDatasetSource {
            source_alias: alias.to_string(),
            form_id: Some(form_id),
            form_version_major,
            dataset_revision_id: None,
            selection_rule,
            position: self.sources.len() as i32,
        };
        let cte_name = self.next_cte_name(alias)?;
        let select_columns = self
            .output_fields
            .iter()
            .map(|field| {
                let column = quote_identifier(&field.key);
                if field.source_alias == alias {
                    if let Some(expression) =
                        system_source_field_expression(&field.source_field_key)
                    {
                        Ok(format!("{expression} AS {column}"))
                    } else {
                        Ok(format!(
                            "MAX(submission_value_fact.value_text) FILTER (WHERE submission_value_fact.field_key = {}) AS {column}",
                            sql_literal(&field.source_field_key)
                        ))
                    }
                } else {
                    Ok(format!("NULL::text AS {column}"))
                }
            })
            .collect::<ApiResult<Vec<_>>>()?
            .join(",\n                ");
        let form_predicate = match form_version_major {
            Some(major) => format!(
                "form_versions.form_id = {}::uuid AND form_versions.version_major = {major}",
                sql_literal(&form_id.to_string())
            ),
            None => format!(
                "form_versions.form_id = {}::uuid",
                sql_literal(&form_id.to_string())
            ),
        };
        let order = match selection_rule {
            DatasetSelectionRule::Earliest => "submission_fact.submitted_at ASC NULLS LAST",
            DatasetSelectionRule::All | DatasetSelectionRule::Latest => {
                "submission_fact.submitted_at DESC NULLS LAST"
            }
        };
        let selected_filter = if selection_rule == DatasetSelectionRule::All {
            "TRUE"
        } else {
            "selection_rank = 1"
        };
        let sql = format!(
            r#"{cte_name} AS (
            WITH ranked AS (
                SELECT
                    submission_fact.submission_id,
                    submission_fact.form_version_id,
                    submission_fact.node_id,
                    node_dim.node_name,
                    submission_fact.status,
                    submission_fact.submitted_at,
                    submission_fact.created_at,
                    submission_fact.last_modified_at,
                    submission_fact.last_modified_by_user_name,
                    ROW_NUMBER() OVER (
                        PARTITION BY submission_fact.node_id
                        ORDER BY {order}, submission_fact.submission_id
                    ) AS selection_rank
                FROM analytics.submission_fact
                JOIN form_versions ON form_versions.id = submission_fact.form_version_id
                JOIN analytics.node_dim ON node_dim.node_id = submission_fact.node_id
                WHERE submission_fact.status = 'submitted'
                  AND {form_predicate}
            )
            SELECT
                ranked.submission_id::text AS __row_id,
                ranked.node_id AS __node_id,
                ranked.node_name AS __node_name,
                {select_columns}
            FROM ranked
            LEFT JOIN analytics.submission_value_fact
              ON submission_value_fact.submission_id = ranked.submission_id
            WHERE {selected_filter}
            GROUP BY ranked.submission_id, ranked.node_id, ranked.node_name
                , ranked.form_version_id, ranked.status, ranked.submitted_at
                , ranked.created_at, ranked.last_modified_at, ranked.last_modified_by_user_name
        )"#
        );
        self.ctes.push(sql);
        self.sources.push(source);
        Ok(CompiledExpression {
            cte_name,
            columns: self.field_keys.iter().cloned().collect(),
        })
    }

    async fn compile_dataset_source(
        &mut self,
        alias: &str,
        dataset_id: Uuid,
        dataset_revision_id: Uuid,
    ) -> ApiResult<CompiledExpression> {
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
        let select_columns = self
            .field_keys
            .iter()
            .map(|field| {
                format!(
                    "{}::text AS {}",
                    quote_identifier(field),
                    quote_identifier(field)
                )
            })
            .collect::<Vec<_>>()
            .join(",\n                ");
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                __row_id,
                __node_id,
                __node_name,
                {select_columns}
            FROM {}.{}
        )"#,
            quote_identifier(&schema),
            quote_identifier(&table)
        ));
        self.sources.push(ValidatedDatasetSource {
            source_alias: alias.to_string(),
            form_id: None,
            form_version_major: None,
            dataset_revision_id: Some(dataset_revision_id),
            selection_rule: DatasetSelectionRule::Latest,
            position: self.sources.len() as i32,
        });
        Ok(CompiledExpression {
            cte_name,
            columns: self.field_keys.iter().cloned().collect(),
        })
    }

    fn compile_operation(
        &mut self,
        operation: &str,
        left: CompiledExpression,
        right: CompiledExpression,
        join_keys: &[dto::DatasetJoinKeyRequest],
    ) -> ApiResult<CompiledExpression> {
        let mode = DatasetCompositionMode::parse(operation)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        let cte_name = self.next_cte_name("op")?;
        let columns = self
            .field_keys
            .iter()
            .map(|field| {
                let quoted = quote_identifier(field);
                format!("COALESCE(l.{quoted}, r.{quoted}) AS {quoted}")
            })
            .collect::<Vec<_>>()
            .join(",\n                ");
        let sql = match mode {
            DatasetCompositionMode::Union | DatasetCompositionMode::UnionAll => {
                let op = if mode == DatasetCompositionMode::Union {
                    "UNION"
                } else {
                    "UNION ALL"
                };
                format!(
                    r#"{cte_name} AS (
            SELECT * FROM {}
            {op}
            SELECT * FROM {}
        )"#,
                    quote_identifier(&left.cte_name),
                    quote_identifier(&right.cte_name)
                )
            }
            DatasetCompositionMode::LeftJoin
            | DatasetCompositionMode::InnerJoin
            | DatasetCompositionMode::OuterJoin => {
                if join_keys.is_empty() {
                    return Err(ApiError::BadRequest(
                        "join operations require at least one explicit join key".into(),
                    ));
                }
                let join = match mode {
                    DatasetCompositionMode::LeftJoin => "LEFT JOIN",
                    DatasetCompositionMode::InnerJoin => "INNER JOIN",
                    DatasetCompositionMode::OuterJoin => "FULL OUTER JOIN",
                    _ => unreachable!(),
                };
                let predicates = join_keys
                    .iter()
                    .map(|key| {
                        if !left.columns.contains(&key.left_field) {
                            return Err(ApiError::BadRequest(format!(
                                "left join key '{}' is not projected by the left input",
                                key.left_field
                            )));
                        }
                        if !right.columns.contains(&key.right_field) {
                            return Err(ApiError::BadRequest(format!(
                                "right join key '{}' is not projected by the right input",
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
                format!(
                    r#"{cte_name} AS (
            SELECT
                COALESCE(l.__row_id, r.__row_id) AS __row_id,
                COALESCE(l.__node_id, r.__node_id) AS __node_id,
                COALESCE(l.__node_name, r.__node_name) AS __node_name,
                {columns}
            FROM {} l
            {join} {} r ON {predicates}
        )"#,
                    quote_identifier(&left.cte_name),
                    quote_identifier(&right.cte_name)
                )
            }
        };
        self.ctes.push(sql);
        Ok(CompiledExpression {
            cte_name,
            columns: self.field_keys.iter().cloned().collect(),
        })
    }

    fn next_cte_name(&mut self, seed: &str) -> ApiResult<String> {
        require_identifier("dataset expression alias", seed)?;
        self.cte_index += 1;
        Ok(format!("{}_{}", sanitize_identifier(seed), self.cte_index))
    }

    fn final_sql(&self, root: &CompiledExpression) -> String {
        let field_select = self
            .field_keys
            .iter()
            .map(|field| quote_identifier(field))
            .collect::<Vec<_>>()
            .join(",\n    ");
        let selected_cte = format!(
            r#"selected_fields AS (
            SELECT
                __row_id,
                __node_id,
                __node_name,
                {field_select}
            FROM {}
        )"#,
            quote_identifier(&root.cte_name)
        );
        let mut ctes = self.ctes.clone();
        ctes.push(selected_cte);

        let final_select = self
            .aggregation
            .as_ref()
            .map(|aggregation| self.aggregation_sql(aggregation))
            .unwrap_or_else(|| {
                format!(
                    r#"SELECT
            __row_id,
            __node_id,
            __node_name,
            {field_select}
        FROM selected_fields"#
                )
            });

        format!(
            r#"WITH
        {}
        {final_select}"#,
            ctes.join(",\n        ")
        )
    }

    fn aggregation_sql(&self, aggregation: &ValidatedAggregation) -> String {
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
                self.field_keys
                    .iter()
                    .filter(|field| !aggregation.group_fields.contains(field))
                    .map(|field| {
                        let quoted = quote_identifier(field);
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
        let internal_group_columns = Vec::<String>::new();
        let all_group_columns = internal_group_columns
            .iter()
            .cloned()
            .chain(group_columns.iter().cloned())
            .collect::<Vec<_>>();
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
        let node_id = "NULL::uuid";
        let node_name = "'Aggregated dataset'";
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
                    let direction = if sort.direction == "highest" {
                        "DESC"
                    } else {
                        "ASC"
                    };
                    format!(
                        "{} {direction} NULLS LAST",
                        quote_identifier(&sort.field_key)
                    )
                })
                .chain(std::iter::once("__row_id".to_string()))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                r#"(SELECT
                selected_fields.*,
                ROW_NUMBER() OVER (
                    {partition}
                    ORDER BY {order_by}
                ) AS __pick_rank
            FROM selected_fields)"#
            )
        } else {
            "selected_fields".to_string()
        };

        format!(
            r#"SELECT
            {row_id} AS __row_id,
            {node_id} AS __node_id,
            {node_name} AS __node_name,
            {field_select}
        FROM {source}{group_by}"#
        )
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
            let mut seen_sort_fields = BTreeSet::new();
            for sort in &row_picker.sort_fields {
                require_text("row picker sort field", &sort.field_key)?;
                if !field_by_key.contains_key(sort.field_key.as_str()) {
                    return Err(ApiError::BadRequest(format!(
                        "row picker sort field '{}' is not projected",
                        sort.field_key
                    )));
                }
                if !seen_sort_fields.insert(sort.field_key.clone()) {
                    return Err(ApiError::BadRequest(format!(
                        "row picker sort field '{}' is duplicated",
                        sort.field_key
                    )));
                }
                if !matches!(sort.direction.as_str(), "lowest" | "highest") {
                    return Err(ApiError::BadRequest(
                        "row picker direction must be 'lowest' or 'highest'".into(),
                    ));
                }
            }
            Ok(row_picker)
        })
        .transpose()?;
    Ok(Some(ValidatedAggregation {
        group_fields,
        metrics,
        row_picker,
    }))
}

fn output_fields_for_dataset(
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
    for key in &aggregation.group_fields {
        if let Some(field) = field_by_key.get(key.as_str()) {
            output.push((*field).clone());
        }
    }
    if aggregation.row_picker.is_some() {
        for field in fields {
            if !aggregation.group_fields.contains(&field.key) {
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
            field_type: metric.field_type,
            position: metric_offset + index as i32,
        });
    }
    output
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
            "MIN(NULLIF({}, '')) AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregationFunction::Max => format!(
            "MAX(NULLIF({}, '')) AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
    }
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
                    "number" | "date" | "text" | "single_choice" | "multi_choice" | "boolean"
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

fn aggregation_response_from_validated(
    aggregation: Option<DatasetAggregationRequest>,
    validated: Option<ValidatedAggregation>,
) -> Option<DatasetAggregationResponse> {
    let request = aggregation?;
    let validated = validated?;
    Some(DatasetAggregationResponse {
        group_fields: validated.group_fields,
        metrics: request.metrics,
        row_picker: request.row_picker,
    })
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
        let field_type = if source.dataset_revision_id.is_some() {
            "text".to_string()
        } else {
            require_source_field_exists(pool, source, &field.source_field_key).await?
        };
        validated.push(ValidatedDatasetField {
            id: None,
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
                (dataset_id, source_alias, form_id, form_version_major, dataset_revision_id, selection_rule, position)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(dataset_id)
        .bind(&source.source_alias)
        .bind(source.form_id)
        .bind(source.form_version_major)
        .bind(source.dataset_revision_id)
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
            (dataset_id, version_number, version_label, status, published_at, definition_ast, aggregation, generated_sql)
        VALUES ($1, $2, $3, 'published'::dataset_revision_status, now(), $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(dataset_id)
    .bind(version_number)
    .bind(version_label)
    .bind(serde_json::to_value(&compiled.definition_ast).map_err(|error| {
        ApiError::Internal(anyhow::anyhow!("dataset definition could not be serialized: {error}"))
    })?)
    .bind(
        compiled
            .aggregation
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| {
                ApiError::Internal(anyhow::anyhow!(
                    "dataset aggregation could not be serialized: {error}"
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
    sqlx::query(&format!("CREATE INDEX ON {full_name} (__node_id)"))
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

async fn require_source_field_exists(
    pool: &sqlx::PgPool,
    source: &ValidatedDatasetSource,
    source_field_key: &str,
) -> ApiResult<String> {
    if let Some(field_type) = system_source_field_type(source_field_key) {
        return Ok(field_type.to_string());
    }

    let row = if let Some(form_id) = source.form_id {
        if let Some(form_version_major) = source.form_version_major {
            sqlx::query(
                r#"
                SELECT form_fields.field_type::text AS field_type
                FROM form_fields
                JOIN form_versions ON form_versions.id = form_fields.form_version_id
                WHERE form_versions.form_id = $1
                  AND form_versions.version_major = $2
                  AND form_versions.status = 'published'::form_version_status
                  AND form_fields.key = $3
                ORDER BY
                    form_versions.version_minor DESC NULLS LAST,
                    form_versions.version_patch DESC NULLS LAST,
                    form_versions.published_at DESC NULLS LAST,
                    form_fields.position
                LIMIT 1
                "#,
            )
            .bind(form_id)
            .bind(form_version_major)
            .bind(source_field_key)
            .fetch_optional(pool)
            .await?
        } else {
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
        }
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

fn system_source_field_expression(source_field_key: &str) -> Option<&'static str> {
    match source_field_key {
        "__submission_id" => Some("ranked.submission_id::text"),
        "__form_version_id" => Some("ranked.form_version_id::text"),
        "__node_id" => Some("ranked.node_id::text"),
        "__node_name" => Some("ranked.node_name"),
        "__submission_status" => Some("ranked.status"),
        "__submitted_at" => Some("ranked.submitted_at::text"),
        "__submission_created_at" => Some("ranked.created_at::text"),
        "__last_updated_at" => Some("ranked.last_modified_at::text"),
        "__last_updated_by_user_name" => Some("ranked.last_modified_by_user_name"),
        _ => None,
    }
}

fn system_source_field_type(source_field_key: &str) -> Option<&'static str> {
    match source_field_key {
        "__submitted_at" | "__submission_created_at" | "__last_updated_at" => Some("date"),
        "__submission_id"
        | "__form_version_id"
        | "__node_id"
        | "__node_name"
        | "__submission_status"
        | "__last_updated_by_user_name" => Some("text"),
        _ => None,
    }
}

pub(crate) async fn load_dataset_table_rows(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
) -> ApiResult<Vec<DatasetTableRow>> {
    load_materialized_dataset_table_rows(pool, dataset_id).await
}

async fn load_materialized_dataset_table_rows(
    pool: &sqlx::PgPool,
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
        "SELECT * FROM {full_name} ORDER BY __row_id LIMIT 200"
    ))
    .fetch_all(pool)
    .await?;
    let mut table_rows = Vec::new();
    for row in rows {
        let submission_id: String = row.try_get("__row_id")?;
        let node_name: String = row.try_get("__node_name")?;
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
            node_name,
            source_alias: "materialized".into(),
            values,
        });
    }
    Ok(table_rows)
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

async fn load_latest_published_form_major(
    pool: &sqlx::PgPool,
    form_id: Uuid,
) -> ApiResult<Option<i32>> {
    sqlx::query_scalar(
        r#"
        SELECT version_major
        FROM form_versions
        WHERE form_id = $1
          AND status = 'published'::form_version_status
          AND version_major IS NOT NULL
        ORDER BY
            version_major DESC,
            version_minor DESC NULLS LAST,
            version_patch DESC NULLS LAST,
            published_at DESC NULLS LAST,
            created_at DESC
        LIMIT 1
        "#,
    )
    .bind(form_id)
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
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
