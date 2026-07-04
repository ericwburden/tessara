//! Component authoring and read endpoints.
//!
//! Components are presentation assets over dataset major lines. This module keeps
//! route behavior and scope checks together while the public wire types live in
//! `dto`.

use std::collections::{BTreeMap, HashMap};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::{Column, Postgres, Row, Transaction};
use tessara_data_ops::{
    AggregateFunction, AggregateMetric, AggregationPlan, DataField, FieldType, FilterOperator,
    ValidatedAggregationPlan, validate_aggregation_plan,
};
use uuid::Uuid;

mod dto;

pub use dto::{
    ComponentDefinition, ComponentSummary, ComponentTable, ComponentTableColumn,
    ComponentTablePagination, ComponentTableRow, ComponentValidationFinding,
    ComponentValidationResponse, ComponentVersionSummary, CreateComponentRequest,
    CreateComponentVersionRequest, UpdateComponentRequest,
};

use crate::{
    auth, datasets,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, require_text},
};

struct ComponentDatasetBinding {
    dataset_id: Uuid,
    dataset_version_major: i32,
    legacy_dataset_revision_id: Option<Uuid>,
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admin/components",
            get(list_admin_components).post(create_component),
        )
        .route(
            "/api/admin/components/{component_id}",
            get(get_admin_component).patch(update_component),
        )
        .route("/api/admin/components/validate", post(validate_component))
        .route(
            "/api/admin/components/{component_id}/versions",
            post(create_component_version),
        )
        .route(
            "/api/admin/components/{component_id}/versions/{version_id}",
            axum::routing::patch(update_component_version),
        )
        .route(
            "/api/admin/components/{component_id}/versions/{version_id}/publish",
            post(publish_component_version),
        )
        .route("/api/components", get(list_components))
        .route("/api/components/{component_ref}", get(get_component_by_ref))
        .route(
            "/api/components/{component_ref}/table",
            get(run_component_table),
        )
        .route(
            "/api/components/{component_ref}/versions/{version_id}/table",
            get(run_component_version_table),
        )
}

/// Creates a component shell before versions are attached.
pub async fn create_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    require_text("component name", &payload.name)?;
    require_text("component slug", &payload.slug)?;
    let version_binding = if let Some(version) = payload.version.as_ref() {
        let binding = resolve_component_dataset_binding(&state.pool, version).await?;
        require_dataset_major_line_exists(
            &state.pool,
            binding.dataset_id,
            binding.dataset_version_major,
        )
        .await?;
        require_dataset_fully_in_capability_scope(
            &state.pool,
            &account,
            "components:manage",
            binding.dataset_id,
        )
        .await?;
        validate_component_type(&version.component_type)?;
        let dataset_fields = load_dataset_major_line_fields(
            &state.pool,
            binding.dataset_id,
            binding.dataset_version_major,
        )
        .await?;
        validate_component_config(&version.component_type, &version.config, &dataset_fields)?;
        Some(binding)
    } else {
        None
    };

    let mut tx = state.pool.begin().await?;
    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO components (name, slug, description)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.slug.trim())
    .bind(&payload.description)
    .fetch_one(&mut *tx)
    .await?;
    if let (Some(version), Some(binding)) = (payload.version.as_ref(), version_binding.as_ref()) {
        let version_id = upsert_component_draft_version(&mut tx, id, binding, version).await?;
        if version.publish.unwrap_or(false) {
            publish_component_version_in_tx(&mut tx, id, version_id).await?;
        }
    }
    tx.commit().await?;

    Ok(Json(IdResponse { id }))
}

/// Lists components visible to the caller's component-management scope.
pub async fn list_admin_components(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ComponentSummary>>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    load_component_summaries(&state.pool, &account, "components:manage")
        .await
        .map(Json)
}

/// Loads one component for management when any version is in manageable scope.
pub async fn get_admin_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> ApiResult<Json<ComponentDefinition>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    let component_id = parse_component_ref(&state.pool, &component_ref).await?;
    load_component_definition(&state.pool, &account, component_id, "components:manage")
        .await
        .map(Json)
}

/// Updates mutable component shell metadata without touching version history.
pub async fn update_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    Json(payload): Json<UpdateComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    require_component_exists(&state.pool, component_id).await?;
    let boundary = auth::capability_boundary(&state.pool, &account, "components:manage").await?;
    require_component_visible_for_boundary(
        &state.pool,
        component_id,
        &boundary,
        "components:manage",
    )
    .await?;
    require_text("component name", &payload.name)?;
    require_text("component slug", &payload.slug)?;
    sqlx::query(
        r#"
        UPDATE components
        SET name = $1,
            slug = $2,
            description = $3
        WHERE id = $4
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.slug.trim())
    .bind(&payload.description)
    .bind(component_id)
    .execute(&state.pool)
    .await?;
    Ok(Json(IdResponse { id: component_id }))
}

/// Validates a component version payload against the bound Dataset major-line contract.
pub async fn validate_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateComponentVersionRequest>,
) -> ApiResult<Json<ComponentValidationResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    let mut findings = Vec::new();
    let binding = match resolve_component_dataset_binding(&state.pool, &payload).await {
        Ok(binding) => binding,
        Err(error) => {
            findings.push(component_validation_finding_from_error(
                "DATASET_MAJOR_LINE_NOT_FOUND",
                "dataset",
                error,
            ));
            return Ok(Json(component_validation_response(findings)));
        }
    };
    if let Err(error) = require_dataset_major_line_exists(
        &state.pool,
        binding.dataset_id,
        binding.dataset_version_major,
    )
    .await
    {
        findings.push(component_validation_finding_from_error(
            "DATASET_MAJOR_LINE_NOT_FOUND",
            "dataset",
            error,
        ));
        return Ok(Json(component_validation_response(findings)));
    }
    require_dataset_fully_in_capability_scope(
        &state.pool,
        &account,
        "components:manage",
        binding.dataset_id,
    )
    .await?;
    let dataset_fields = match load_dataset_major_line_fields(
        &state.pool,
        binding.dataset_id,
        binding.dataset_version_major,
    )
    .await
    {
        Ok(fields) => fields,
        Err(error) => {
            findings.push(component_validation_finding_from_error(
                "DATASET_MAJOR_LINE_CONTRACT_UNAVAILABLE",
                "dataset",
                error,
            ));
            return Ok(Json(component_validation_response(findings)));
        }
    };
    if let Err(error) = validate_component_type(&payload.component_type) {
        findings.push(component_validation_finding_from_error(
            "COMPONENT_UNSUPPORTED_KIND",
            "component_type",
            error,
        ));
    } else if let Err(error) =
        validate_component_config(&payload.component_type, &payload.config, &dataset_fields)
    {
        findings.push(component_config_validation_finding(error));
    }

    Ok(Json(component_validation_response(findings)))
}

/// Creates a draft or published component version over a dataset major line.
pub async fn create_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    Json(payload): Json<CreateComponentVersionRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    require_component_exists(&state.pool, component_id).await?;
    let binding = resolve_component_dataset_binding(&state.pool, &payload).await?;
    require_dataset_major_line_exists(
        &state.pool,
        binding.dataset_id,
        binding.dataset_version_major,
    )
    .await?;
    require_dataset_fully_in_capability_scope(
        &state.pool,
        &account,
        "components:manage",
        binding.dataset_id,
    )
    .await?;
    validate_component_type(&payload.component_type)?;
    let dataset_fields = load_dataset_major_line_fields(
        &state.pool,
        binding.dataset_id,
        binding.dataset_version_major,
    )
    .await?;
    validate_component_config(&payload.component_type, &payload.config, &dataset_fields)?;

    let mut tx = state.pool.begin().await?;
    let id = upsert_component_draft_version(&mut tx, component_id, &binding, &payload).await?;
    if payload.publish.unwrap_or(false) {
        publish_component_version_in_tx(&mut tx, component_id, id).await?;
    }
    tx.commit().await?;
    Ok(Json(IdResponse { id }))
}

/// Updates a specific draft component version without creating a new version row.
pub async fn update_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateComponentVersionRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    require_component_exists(&state.pool, component_id).await?;
    require_component_version_draft_row(&state.pool, component_id, version_id).await?;
    let binding = resolve_component_dataset_binding(&state.pool, &payload).await?;
    require_dataset_major_line_exists(
        &state.pool,
        binding.dataset_id,
        binding.dataset_version_major,
    )
    .await?;
    require_dataset_fully_in_capability_scope(
        &state.pool,
        &account,
        "components:manage",
        binding.dataset_id,
    )
    .await?;
    validate_component_type(&payload.component_type)?;
    let dataset_fields = load_dataset_major_line_fields(
        &state.pool,
        binding.dataset_id,
        binding.dataset_version_major,
    )
    .await?;
    validate_component_config(&payload.component_type, &payload.config, &dataset_fields)?;

    sqlx::query(
        r#"
        UPDATE component_versions
        SET dataset_id = $1,
            dataset_version_major = $2,
            binding_mode = 'major_line',
            dataset_revision_id = $3,
            component_type = $4::component_type,
            config = $5
        WHERE component_id = $6
          AND id = $7
          AND status = 'draft'::component_version_status
        "#,
    )
    .bind(binding.dataset_id)
    .bind(binding.dataset_version_major)
    .bind(binding.legacy_dataset_revision_id)
    .bind(&payload.component_type)
    .bind(&payload.config)
    .bind(component_id)
    .bind(version_id)
    .execute(&state.pool)
    .await?;
    Ok(Json(IdResponse { id: version_id }))
}

/// Publishes a draft component version and atomically supersedes the current published version.
pub async fn publish_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    require_component_exists(&state.pool, component_id).await?;
    let version = load_component_version_for_publish(&state.pool, component_id, version_id).await?;
    require_dataset_fully_in_capability_scope(
        &state.pool,
        &account,
        "components:manage",
        version.dataset_id,
    )
    .await?;
    let dataset_fields = load_dataset_major_line_fields(
        &state.pool,
        version.dataset_id,
        version.dataset_version_major,
    )
    .await?;
    validate_component_config(&version.component_type, &version.config, &dataset_fields)?;

    let mut tx = state.pool.begin().await?;
    publish_component_version_in_tx(&mut tx, component_id, version_id).await?;
    tx.commit().await?;
    Ok(Json(IdResponse { id: version_id }))
}

async fn upsert_component_draft_version(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    binding: &ComponentDatasetBinding,
    payload: &CreateComponentVersionRequest,
) -> ApiResult<Uuid> {
    let id = sqlx::query_scalar(
        r#"
        WITH next_version AS (
            SELECT COALESCE(MAX(version_number), 0) + 1 AS version_number
            FROM component_versions
            WHERE component_id = $1
        )
        INSERT INTO component_versions
            (component_id, dataset_id, dataset_version_major, binding_mode, dataset_revision_id,
             component_type, version_number, version_label, status, config)
        SELECT $1, $2, $3, 'major_line', $4, $5::component_type,
               next_version.version_number, next_version.version_number::text,
               'draft'::component_version_status, $6
        FROM next_version
        ON CONFLICT (component_id) WHERE status = 'draft'::component_version_status
        DO UPDATE SET dataset_id = EXCLUDED.dataset_id,
                      dataset_version_major = EXCLUDED.dataset_version_major,
                      binding_mode = EXCLUDED.binding_mode,
                      dataset_revision_id = EXCLUDED.dataset_revision_id,
                      component_type = EXCLUDED.component_type,
                      config = EXCLUDED.config
        RETURNING id
        "#,
    )
    .bind(component_id)
    .bind(binding.dataset_id)
    .bind(binding.dataset_version_major)
    .bind(binding.legacy_dataset_revision_id)
    .bind(&payload.component_type)
    .bind(&payload.config)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

async fn publish_component_version_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    version_id: Uuid,
) -> ApiResult<()> {
    let status: Option<String> = sqlx::query_scalar(
        r#"
        SELECT status::text
        FROM component_versions
        WHERE component_id = $1
          AND id = $2
        "#,
    )
    .bind(component_id)
    .bind(version_id)
    .fetch_optional(&mut **tx)
    .await?;
    match status.as_deref() {
        Some(status) => require_component_version_draft(version_id, status)?,
        None => {
            return Err(ApiError::NotFound(format!(
                "component version {version_id}"
            )));
        }
    }
    sqlx::query(
        r#"
        UPDATE component_versions
        SET status = 'superseded'::component_version_status
        WHERE component_id = $1
          AND status = 'published'::component_version_status
        "#,
    )
    .bind(component_id)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        r#"
        UPDATE component_versions
        SET status = 'published'::component_version_status,
            published_at = now()
        WHERE component_id = $1
          AND id = $2
          AND status = 'draft'::component_version_status
        "#,
    )
    .bind(component_id)
    .bind(version_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Lists components visible to the caller's component-read capability scope.
pub async fn list_components(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ComponentSummary>>> {
    let account = auth::require_capability(&state.pool, &headers, "components:read").await?;
    load_component_summaries(&state.pool, &account, "components:read")
        .await
        .map(Json)
}

/// Loads a component by UUID or slug when its dataset major line is readable.
pub async fn get_component_by_ref(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> ApiResult<Json<ComponentDefinition>> {
    let account = auth::require_capability(&state.pool, &headers, "components:read").await?;
    let component_id = parse_component_ref(&state.pool, &component_ref).await?;
    load_component_definition(&state.pool, &account, component_id, "components:read")
        .await
        .map(Json)
}

/// Executes the current published table version for a component.
pub async fn run_component_table(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
    Query(query): Query<ComponentTableQuery>,
) -> ApiResult<Json<ComponentTable>> {
    let account = auth::require_capability(&state.pool, &headers, "components:read").await?;
    let version =
        load_component_version_for_table(&state.pool, &account, &component_ref, None).await?;
    execute_component_table(&state.pool, &account, version, query.into_runtime_query()?)
        .await
        .map(Json)
}

/// Executes a specific published table component version.
pub async fn run_component_version_table(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_ref, version_id)): Path<(String, Uuid)>,
    Query(query): Query<ComponentTableQuery>,
) -> ApiResult<Json<ComponentTable>> {
    let account = auth::require_capability(&state.pool, &headers, "components:read").await?;
    let version =
        load_component_version_for_table(&state.pool, &account, &component_ref, Some(version_id))
            .await?;
    execute_component_table(&state.pool, &account, version, query.into_runtime_query()?)
        .await
        .map(Json)
}

struct ComponentVersionForTable {
    id: Uuid,
    component_id: Uuid,
    dataset_id: Uuid,
    dataset_version_major: i32,
    component_type: String,
    config: serde_json::Value,
}

#[derive(Default, Deserialize)]
pub struct ComponentTableQuery {
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    page_size: Option<usize>,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    sort: Option<String>,
    #[serde(default)]
    visible_columns: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

impl ComponentTableQuery {
    fn into_runtime_query(self) -> ApiResult<ComponentRuntimeQuery> {
        Ok(ComponentRuntimeQuery {
            search: self
                .q
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            page_size: self.page_size.map(|value| value.clamp(1, 200)),
            offset: parse_component_cursor(self.cursor.as_deref())?,
            sort: self.sort.as_deref().map(parse_component_sort).transpose()?,
            visible_columns: self
                .visible_columns
                .as_deref()
                .map(csv_keys)
                .unwrap_or_default(),
            filters: parse_component_query_filters(&self.extra)?,
        })
    }
}

#[derive(Default)]
struct ComponentRuntimeQuery {
    search: Option<String>,
    page_size: Option<usize>,
    offset: usize,
    sort: Option<ComponentSortConfig>,
    visible_columns: Vec<String>,
    filters: Vec<ComponentFilterConfig>,
}

struct ComponentVersionForPublish {
    dataset_id: Uuid,
    dataset_version_major: i32,
    component_type: String,
    config: serde_json::Value,
}

struct MajorLineMaterialization {
    schema: String,
    table: String,
    state: String,
}

async fn execute_component_table(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    version: ComponentVersionForTable,
    query: ComponentRuntimeQuery,
) -> ApiResult<ComponentTable> {
    let fields =
        load_dataset_major_line_fields(pool, version.dataset_id, version.dataset_version_major)
            .await?;
    validate_component_config(&version.component_type, &version.config, &fields)?;
    let Some(materialization) =
        load_major_line_materialization(pool, version.dataset_id, version.dataset_version_major)
            .await?
    else {
        return Ok(empty_component_table(version, "pending", Vec::new()));
    };
    if materialization.state != "ready" {
        return Ok(empty_component_table(
            version,
            &materialization.state,
            Vec::new(),
        ));
    }
    match version.component_type.as_str() {
        "detail_table" => {
            execute_detail_table(pool, account, version, materialization, &fields, query).await
        }
        "aggregate_table" => {
            execute_aggregate_table(pool, account, version, materialization, &fields, query).await
        }
        other => Err(ApiError::BadRequest(format!(
            "unsupported component type '{other}'"
        ))),
    }
}

async fn load_major_line_materialization(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
    version_major: i32,
) -> ApiResult<Option<MajorLineMaterialization>> {
    let row = sqlx::query(
        r#"
        SELECT materialized_schema, materialized_table, rebuild_status
        FROM dataset_major_materializations
        WHERE dataset_id = $1
          AND version_major = $2
        "#,
    )
    .bind(dataset_id)
    .bind(version_major)
    .fetch_optional(pool)
    .await?;
    if let Some(row) = row {
        Ok(Some(MajorLineMaterialization {
            schema: row.try_get("materialized_schema")?,
            table: row.try_get("materialized_table")?,
            state: row.try_get("rebuild_status")?,
        }))
    } else {
        Ok(None)
    }
}

fn empty_component_table(
    version: ComponentVersionForTable,
    materialization_state: &str,
    columns: Vec<ComponentTableColumn>,
) -> ComponentTable {
    ComponentTable {
        component_id: version.component_id,
        component_version_id: version.id,
        dataset_id: version.dataset_id,
        dataset_version_major: version.dataset_version_major,
        component_type: version.component_type,
        materialization_state: materialization_state.into(),
        columns,
        rows: Vec::new(),
        pagination: ComponentTablePagination {
            page_size: 0,
            next_cursor: None,
            has_more: false,
        },
    }
}

async fn execute_detail_table(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    version: ComponentVersionForTable,
    materialization: MajorLineMaterialization,
    fields: &[DataField],
    query: ComponentRuntimeQuery,
) -> ApiResult<ComponentTable> {
    let config: DetailTableConfig =
        serde_json::from_value(version.config.clone()).map_err(|error| {
            ApiError::BadRequest(format!("detail table config is invalid: {error}"))
        })?;
    let configured_fields = config
        .columns
        .iter()
        .map(|column| require_component_field(fields, column.field_key(), "detail table column"))
        .collect::<ApiResult<Vec<_>>>()?;
    let selected_fields = visible_table_fields(&configured_fields, &query.visible_columns)?;
    let columns = selected_fields
        .iter()
        .map(|field| component_table_column(field))
        .collect::<Vec<_>>();
    let select_columns = selected_fields
        .iter()
        .map(|field| quote_identifier(&field.key))
        .collect::<Vec<_>>();
    let mut predicates =
        vec![tier_access_predicate_for_materialization(pool, account, &materialization).await?];
    predicates.extend(component_filter_sql(&config.default_filters, fields)?);
    predicates.extend(component_filter_sql(&query.filters, fields)?);
    if let Some(search) = query.search.as_deref() {
        let search_fields = detail_search_fields(&config, &configured_fields, fields)?;
        if !search_fields.is_empty() {
            predicates.push(search_predicate_sql(&search_fields, search));
        }
    }
    let full_name = materialized_full_name(&materialization);
    let sort = query.sort.or(config.default_sort);
    let order_by = table_order_by_sql(sort.as_ref(), &configured_fields, "__row_id")?;
    let page_size = effective_component_page_size(query.page_size, config.page_size);
    let page = component_pagination_sql(query.offset, page_size);
    let sql = format!(
        "SELECT __row_id, {} FROM {full_name} WHERE {}{order_by}{page}",
        select_columns.join(", "),
        predicates.join(" AND ")
    );
    let (rows, pagination) = component_rows_from_query(pool, &sql, query.offset, page_size).await?;
    Ok(ComponentTable {
        component_id: version.component_id,
        component_version_id: version.id,
        dataset_id: version.dataset_id,
        dataset_version_major: version.dataset_version_major,
        component_type: version.component_type,
        materialization_state: "ready".into(),
        columns,
        rows,
        pagination,
    })
}

async fn execute_aggregate_table(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    version: ComponentVersionForTable,
    materialization: MajorLineMaterialization,
    fields: &[DataField],
    query: ComponentRuntimeQuery,
) -> ApiResult<ComponentTable> {
    let config: AggregateTableConfig =
        serde_json::from_value(version.config.clone()).map_err(|error| {
            ApiError::BadRequest(format!("aggregate table config is invalid: {error}"))
        })?;
    let validated = validated_aggregation_plan_from_config(&config, fields)?;
    let mut aggregate_fields = validated
        .group_fields
        .iter()
        .filter_map(|key| fields.iter().find(|field| field.key == *key).cloned())
        .collect::<Vec<_>>();
    aggregate_fields.extend(validated.metrics.iter().map(|metric| DataField {
        key: metric.key.clone(),
        label: metric.label.clone(),
        field_type: metric.output_field_type.clone(),
        position: metric.position,
    }));
    let aggregate_field_refs = aggregate_fields.iter().collect::<Vec<_>>();
    let selected_fields = visible_table_fields(&aggregate_field_refs, &query.visible_columns)?;
    let columns = selected_fields
        .iter()
        .map(|field| component_table_column(field))
        .collect::<Vec<_>>();
    let mut source_predicates =
        vec![tier_access_predicate_for_materialization(pool, account, &materialization).await?];
    source_predicates.extend(component_filter_sql(&config.pre_filters, fields)?);
    source_predicates.extend(source_runtime_filters(
        &query.filters,
        fields,
        &aggregate_fields,
    )?);
    let group_selects = validated
        .group_fields
        .iter()
        .map(|key| quote_identifier(key))
        .collect::<Vec<_>>();
    let metric_selects = validated
        .metrics
        .iter()
        .map(aggregate_metric_sql)
        .collect::<Vec<_>>();
    let mut select_parts = group_selects.clone();
    select_parts.extend(metric_selects);
    let group_by = if group_selects.is_empty() {
        String::new()
    } else {
        format!(" GROUP BY {}", group_selects.join(", "))
    };
    let full_name = materialized_full_name(&materialization);
    let sort = query.sort.or(config.default_sort);
    let order_by = table_order_by_sql(sort.as_ref(), &aggregate_field_refs, "__row_id")?;
    let inner_sql = format!(
        "SELECT md5(concat_ws('|', {})) AS __row_id, {} FROM {full_name} WHERE {}{group_by}",
        aggregate_row_id_parts(&validated.group_fields),
        select_parts.join(", "),
        source_predicates.join(" AND ")
    );
    let post_predicates = component_filter_sql(&config.post_filters, &aggregate_fields)?;
    let mut post_predicates = post_predicates;
    post_predicates.extend(aggregate_runtime_filters(
        &query.filters,
        fields,
        &aggregate_fields,
    )?);
    if let Some(search) = query.search.as_deref() {
        let search_fields = aggregate_search_fields(&aggregate_field_refs);
        if !search_fields.is_empty() {
            post_predicates.push(search_predicate_sql(&search_fields, search));
        }
    }
    let outer_where = if post_predicates.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", post_predicates.join(" AND "))
    };
    let page_size = effective_component_page_size(query.page_size, config.page_size);
    let page = component_pagination_sql(query.offset, page_size);
    let sql = format!("SELECT * FROM ({inner_sql}) AS aggregate_rows{outer_where}{order_by}{page}");
    let (rows, pagination) = component_rows_from_query(pool, &sql, query.offset, page_size).await?;
    Ok(ComponentTable {
        component_id: version.component_id,
        component_version_id: version.id,
        dataset_id: version.dataset_id,
        dataset_version_major: version.dataset_version_major,
        component_type: version.component_type,
        materialization_state: "ready".into(),
        columns,
        rows,
        pagination,
    })
}

async fn component_rows_from_query(
    pool: &sqlx::PgPool,
    sql: &str,
    offset: usize,
    page_size: usize,
) -> ApiResult<(Vec<ComponentTableRow>, ComponentTablePagination)> {
    let mut rows = sqlx::query(sql).fetch_all(pool).await?;
    let has_more = rows.len() > page_size;
    if has_more {
        rows.truncate(page_size);
    }
    let next_cursor = has_more.then(|| format!("offset:{}", offset + page_size));
    let rows = rows
        .into_iter()
        .map(|row| {
            let row_id: String = row.try_get("__row_id")?;
            let mut values = BTreeMap::new();
            for column in row.columns() {
                let name = column.name();
                if name.starts_with("__") {
                    continue;
                }
                values.insert(name.to_string(), row.try_get(name)?);
            }
            Ok(ComponentTableRow { row_id, values })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(ApiError::from)?;
    Ok((
        rows,
        ComponentTablePagination {
            page_size,
            next_cursor,
            has_more,
        },
    ))
}

fn component_table_column(field: &DataField) -> ComponentTableColumn {
    ComponentTableColumn {
        key: field.key.clone(),
        label: field.label.clone(),
        field_type: field.field_type.as_str().to_string(),
    }
}

fn visible_table_fields<'a>(
    fields: &[&'a DataField],
    visible_columns: &[String],
) -> ApiResult<Vec<&'a DataField>> {
    if visible_columns.is_empty() {
        return Ok(fields.to_vec());
    }
    let mut selected = Vec::new();
    for key in visible_columns {
        let field = fields
            .iter()
            .find(|field| field.key == *key)
            .copied()
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "visible column '{key}' is outside the component table contract"
                ))
            })?;
        selected.push(field);
    }
    Ok(selected)
}

fn detail_search_fields<'a>(
    config: &DetailTableConfig,
    selected_fields: &[&'a DataField],
    fields: &'a [DataField],
) -> ApiResult<Vec<&'a DataField>> {
    if config.search_fields.is_empty() {
        return Ok(selected_fields.to_vec());
    }
    config
        .search_fields
        .iter()
        .map(|key| require_component_field(fields, key, "detail table search field"))
        .collect()
}

fn aggregate_search_fields<'a>(aggregate_fields: &[&'a DataField]) -> Vec<&'a DataField> {
    aggregate_fields.to_vec()
}

fn search_predicate_sql(fields: &[&DataField], search: &str) -> String {
    let value = sql_literal(search);
    let predicates = fields
        .iter()
        .map(|field| {
            format!(
                "POSITION(LOWER({value}) IN LOWER(COALESCE({}::text, ''))) > 0",
                quote_identifier(&field.key)
            )
        })
        .collect::<Vec<_>>();
    format!("({})", predicates.join(" OR "))
}

fn table_order_by_sql(
    sort: Option<&ComponentSortConfig>,
    fields: &[&DataField],
    fallback: &str,
) -> ApiResult<String> {
    let Some(sort) = sort else {
        return Ok(format!(" ORDER BY {}", quote_identifier(fallback)));
    };
    let field = fields
        .iter()
        .find(|field| field.key == sort.field_key)
        .copied()
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "component table sort references field '{}' outside the table contract",
                sort.field_key
            ))
        })?;
    let direction = if sort.direction.eq_ignore_ascii_case("desc") {
        "DESC"
    } else {
        "ASC"
    };
    Ok(format!(
        " ORDER BY {} {direction}, {}",
        typed_orderable_sql(&quote_identifier(&field.key), field.field_type.as_str()),
        quote_identifier(fallback)
    ))
}

fn component_pagination_sql(offset: usize, page_size: usize) -> String {
    format!(" LIMIT {} OFFSET {}", page_size + 1, offset)
}

fn effective_component_page_size(
    query_page_size: Option<usize>,
    config_page_size: Option<usize>,
) -> usize {
    query_page_size
        .or(config_page_size)
        .unwrap_or(50)
        .clamp(1, 200)
}

fn source_runtime_filters(
    filters: &[ComponentFilterConfig],
    source_fields: &[DataField],
    aggregate_fields: &[DataField],
) -> ApiResult<Vec<String>> {
    filters
        .iter()
        .filter(|filter| {
            source_fields
                .iter()
                .any(|field| field.key == filter.field_key)
                && !aggregate_fields
                    .iter()
                    .any(|field| field.key == filter.field_key)
        })
        .map(|filter| filter_to_sql(filter, source_fields, "component source filter"))
        .collect()
}

fn aggregate_runtime_filters(
    filters: &[ComponentFilterConfig],
    source_fields: &[DataField],
    aggregate_fields: &[DataField],
) -> ApiResult<Vec<String>> {
    filters
        .iter()
        .map(|filter| {
            if aggregate_fields
                .iter()
                .any(|field| field.key == filter.field_key)
            {
                filter_to_sql(filter, aggregate_fields, "component aggregate filter").map(Some)
            } else if source_fields
                .iter()
                .any(|field| field.key == filter.field_key)
            {
                Ok(None)
            } else {
                Err(ApiError::BadRequest(format!(
                    "component table filter references field '{}' outside the table contract",
                    filter.field_key
                )))
            }
        })
        .filter_map(Result::transpose)
        .collect()
}

fn filter_to_sql(
    filter: &ComponentFilterConfig,
    fields: &[DataField],
    label: &str,
) -> ApiResult<String> {
    let field = require_component_field(fields, &filter.field_key, label)?;
    let operator = FilterOperator::parse(&filter.operator).map_err(component_data_op_error)?;
    operator
        .validate_for_field(field)
        .map_err(component_data_op_error)?;
    Ok(filter_predicate_sql(
        field,
        operator,
        filter.value.as_deref(),
    ))
}

fn parse_component_cursor(cursor: Option<&str>) -> ApiResult<usize> {
    let Some(cursor) = cursor.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(0);
    };
    let value = cursor.strip_prefix("offset:").unwrap_or(cursor);
    value
        .parse::<usize>()
        .map_err(|_| ApiError::BadRequest("component table cursor is invalid".into()))
}

fn parse_component_sort(value: &str) -> ApiResult<ComponentSortConfig> {
    let (field_key, direction) = value.split_once(':').unwrap_or((value, "asc"));
    let direction = match direction.trim().to_ascii_lowercase().as_str() {
        "asc" | "" => "asc",
        "desc" => "desc",
        _ => {
            return Err(ApiError::BadRequest(
                "component table sort direction must be asc or desc".into(),
            ));
        }
    };
    Ok(ComponentSortConfig {
        field_key: field_key.trim().to_string(),
        direction: direction.into(),
    })
}

fn parse_component_query_filters(
    values: &HashMap<String, String>,
) -> ApiResult<Vec<ComponentFilterConfig>> {
    let mut by_field = BTreeMap::<String, (Option<String>, Option<String>)>::new();
    for (key, value) in values {
        let Some(remainder) = key.strip_prefix("filter[") else {
            continue;
        };
        let Some((field_key, suffix)) = remainder.split_once("][") else {
            continue;
        };
        let Some(kind) = suffix.strip_suffix(']') else {
            continue;
        };
        let entry = by_field.entry(field_key.to_string()).or_default();
        match kind {
            "operator" => entry.0 = Some(value.clone()),
            "value" => entry.1 = Some(value.clone()),
            _ => {}
        }
    }
    by_field
        .into_iter()
        .map(|(field_key, (operator, value))| {
            let operator = operator.ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "component table filter for '{field_key}' is missing an operator"
                ))
            })?;
            Ok(ComponentFilterConfig {
                field_key,
                operator,
                value,
            })
        })
        .collect()
}

fn csv_keys(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn materialized_full_name(materialization: &MajorLineMaterialization) -> String {
    format!(
        "{}.{}",
        quote_identifier(&materialization.schema),
        quote_identifier(&materialization.table)
    )
}

fn aggregate_metric_sql(metric: &tessara_data_ops::ValidatedAggregateMetric) -> String {
    let key = quote_identifier(&metric.key);
    match metric.function {
        AggregateFunction::Count => format!("COUNT(*)::text AS {key}"),
        AggregateFunction::CountValues => format!(
            "COUNT(NULLIF({}, ''))::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregateFunction::CountDistinct => format!(
            "COUNT(DISTINCT NULLIF({}, ''))::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregateFunction::Sum => format!(
            "SUM(NULLIF({}, '')::numeric)::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregateFunction::Avg => format!(
            "AVG(NULLIF({}, '')::numeric)::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregateFunction::Min => format!(
            "MIN({})::text AS {key}",
            typed_orderable_sql(
                &quote_identifier(metric.source_field_key.as_deref().unwrap_or_default()),
                metric.output_field_type.as_str()
            )
        ),
        AggregateFunction::Max => format!(
            "MAX({})::text AS {key}",
            typed_orderable_sql(
                &quote_identifier(metric.source_field_key.as_deref().unwrap_or_default()),
                metric.output_field_type.as_str()
            )
        ),
    }
}

fn aggregate_row_id_parts(group_fields: &[String]) -> String {
    if group_fields.is_empty() {
        return sql_literal("all");
    }
    group_fields
        .iter()
        .map(|field| format!("COALESCE({}::text, '')", quote_identifier(field)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn component_filter_sql(
    filters: &[ComponentFilterConfig],
    fields: &[DataField],
) -> ApiResult<Vec<String>> {
    filters
        .iter()
        .map(|filter| filter_to_sql(filter, fields, "component filter"))
        .collect()
}

fn filter_predicate_sql(
    field: &DataField,
    operator: FilterOperator,
    value: Option<&str>,
) -> String {
    let field_sql = quote_identifier(&field.key);
    let value_sql = sql_literal(value.unwrap_or_default());
    match operator {
        FilterOperator::Equals => equality_sql(&field_sql, &value_sql, field.field_type.as_str()),
        FilterOperator::NotEquals => {
            let equality = equality_sql(&field_sql, &value_sql, field.field_type.as_str());
            format!("NOT ({equality})")
        }
        FilterOperator::Contains => {
            format!("POSITION(LOWER({value_sql}) IN LOWER(COALESCE({field_sql}, ''))) > 0")
        }
        FilterOperator::NotContains => {
            format!("POSITION(LOWER({value_sql}) IN LOWER(COALESCE({field_sql}, ''))) = 0")
        }
        FilterOperator::Lt => {
            comparison_sql(&field_sql, "<", &value_sql, field.field_type.as_str())
        }
        FilterOperator::Lte => {
            comparison_sql(&field_sql, "<=", &value_sql, field.field_type.as_str())
        }
        FilterOperator::Gt => {
            comparison_sql(&field_sql, ">", &value_sql, field.field_type.as_str())
        }
        FilterOperator::Gte => {
            comparison_sql(&field_sql, ">=", &value_sql, field.field_type.as_str())
        }
        FilterOperator::Between | FilterOperator::NotBetween => {
            between_filter_sql(&field_sql, field.field_type.as_str(), value, operator)
        }
        FilterOperator::IsEmpty => format!("NULLIF({field_sql}, '') IS NULL"),
        FilterOperator::IsNotEmpty => format!("NULLIF({field_sql}, '') IS NOT NULL"),
        FilterOperator::IsNull => format!("{field_sql} IS NULL"),
        FilterOperator::IsNotNull => format!("{field_sql} IS NOT NULL"),
    }
}

fn between_filter_sql(
    field_sql: &str,
    field_type: &str,
    value: Option<&str>,
    operator: FilterOperator,
) -> String {
    let (lower, upper) = value
        .unwrap_or_default()
        .split_once("..")
        .or_else(|| value.unwrap_or_default().split_once(','))
        .unwrap_or(("", ""));
    let lower = sql_literal(lower.trim());
    let upper = sql_literal(upper.trim());
    let predicate = format!(
        "({} AND {})",
        comparison_sql(field_sql, ">=", &lower, field_type),
        comparison_sql(field_sql, "<=", &upper, field_type)
    );
    if operator == FilterOperator::NotBetween {
        format!("NOT {predicate}")
    } else {
        predicate
    }
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

fn typed_orderable_sql(expression: &str, field_type: &str) -> String {
    typed_comparable_sql(expression, field_type)
        .unwrap_or_else(|| format!("NULLIF({expression}, '')"))
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

fn tier_access_predicate(account: &auth::AccountContext) -> &'static str {
    if account.has_capability("admin:all") || account.has_capability("datasets:read_confidential") {
        "TRUE"
    } else if account.has_capability("datasets:read_restricted") {
        "COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal', 'restricted')"
    } else {
        "COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal')"
    }
}

async fn tier_access_predicate_for_materialization(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    materialization: &MajorLineMaterialization,
) -> ApiResult<String> {
    let has_restriction_tier: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = $1
              AND table_name = $2
              AND column_name = '__restriction_tier'
        )
        "#,
    )
    .bind(&materialization.schema)
    .bind(&materialization.table)
    .fetch_one(pool)
    .await?;

    if has_restriction_tier {
        Ok(tier_access_predicate(account).to_string())
    } else {
        Ok("TRUE".into())
    }
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

async fn load_component_version_for_table(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    component_ref: &str,
    version_id: Option<Uuid>,
) -> ApiResult<ComponentVersionForTable> {
    let boundary = auth::capability_boundary(pool, account, "components:read").await?;
    let mut query = String::from(
        r#"
        SELECT component_versions.id,
               component_versions.component_id,
               component_versions.dataset_id,
               component_versions.dataset_version_major,
               component_versions.component_type::text AS component_type,
               component_versions.config
        FROM components
        JOIN component_versions ON component_versions.component_id = components.id
        WHERE (components.id::text = $1 OR components.slug = $1)
          AND component_versions.status = 'published'::component_version_status
        "#,
    );
    if version_id.is_some() {
        query.push_str(" AND component_versions.id = $2");
    }
    query.push_str(" ORDER BY component_versions.version_number DESC LIMIT 1");

    let mut sql = sqlx::query(&query).bind(component_ref);
    if let Some(version_id) = version_id {
        sql = sql.bind(version_id);
    }
    let row = sql
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("published component {component_ref}")))?;
    let component_id = row.try_get("component_id")?;
    require_component_visible_for_boundary(pool, component_id, &boundary, "components:read")
        .await?;
    Ok(ComponentVersionForTable {
        id: row.try_get("id")?,
        component_id,
        dataset_id: row.try_get("dataset_id")?,
        dataset_version_major: row.try_get("dataset_version_major")?,
        component_type: row.try_get("component_type")?,
        config: row.try_get("config")?,
    })
}

async fn load_component_version_for_publish(
    pool: &sqlx::PgPool,
    component_id: Uuid,
    version_id: Uuid,
) -> ApiResult<ComponentVersionForPublish> {
    let row = sqlx::query(
        r#"
        SELECT dataset_id, dataset_version_major, component_type::text AS component_type, config,
               status::text AS status
        FROM component_versions
        WHERE component_id = $1
          AND id = $2
        "#,
    )
    .bind(component_id)
    .bind(version_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("component version {version_id}")))?;
    let status: String = row.try_get("status")?;
    require_component_version_draft(version_id, &status)?;
    Ok(ComponentVersionForPublish {
        dataset_id: row.try_get("dataset_id")?,
        dataset_version_major: row.try_get("dataset_version_major")?,
        component_type: row.try_get("component_type")?,
        config: row.try_get("config")?,
    })
}

fn require_component_version_draft(version_id: Uuid, status: &str) -> ApiResult<()> {
    if status == "draft" {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "component version {version_id} is immutable with status '{status}'"
        )))
    }
}

async fn require_component_version_draft_row(
    pool: &sqlx::PgPool,
    component_id: Uuid,
    version_id: Uuid,
) -> ApiResult<()> {
    let status: String = sqlx::query_scalar(
        r#"
        SELECT status::text
        FROM component_versions
        WHERE component_id = $1
          AND id = $2
        "#,
    )
    .bind(component_id)
    .bind(version_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("component version {version_id}")))?;
    require_component_version_draft(version_id, &status)
}

async fn load_component_summaries(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    capability: &str,
) -> ApiResult<Vec<ComponentSummary>> {
    let rows = match auth::capability_boundary(pool, account, capability).await? {
        auth::CapabilityBoundary::Scoped(scope_ids) if capability == "components:manage" => {
            sqlx::query(
                r#"
        SELECT DISTINCT ON (components.id)
            components.id,
            components.name,
            components.slug,
            components.description,
            current_versions.id AS current_version_id,
            current_versions.component_type::text AS current_component_type
        FROM components
        JOIN component_versions AS visible_versions
            ON visible_versions.component_id = components.id
        JOIN dataset_scope_nodes
            ON dataset_scope_nodes.dataset_id = visible_versions.dataset_id
           AND dataset_scope_nodes.node_id = ANY($1)
        LEFT JOIN component_versions AS current_versions
            ON current_versions.component_id = components.id
           AND current_versions.status = 'published'::component_version_status
        ORDER BY components.id, components.name
        "#,
            )
            .bind(scope_ids)
            .fetch_all(pool)
            .await?
        }
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            sqlx::query(
                r#"
        SELECT
            components.id,
            components.name,
            components.slug,
            components.description,
            current_versions.id AS current_version_id,
            current_versions.component_type::text AS current_component_type
        FROM components
        LEFT JOIN component_versions AS current_versions
            ON current_versions.component_id = components.id
           AND current_versions.status = 'published'::component_version_status
        JOIN dataset_scope_nodes
            ON dataset_scope_nodes.dataset_id = current_versions.dataset_id
           AND dataset_scope_nodes.node_id = ANY($1)
        ORDER BY components.name, components.id
        "#,
            )
            .bind(scope_ids)
            .fetch_all(pool)
            .await?
        }
        auth::CapabilityBoundary::Global => {
            sqlx::query(
                r#"
        SELECT
            components.id,
            components.name,
            components.slug,
            components.description,
            current_versions.id AS current_version_id,
            current_versions.component_type::text AS current_component_type
        FROM components
        LEFT JOIN component_versions AS current_versions
            ON current_versions.component_id = components.id
           AND current_versions.status = 'published'::component_version_status
        WHERE ($1 OR current_versions.id IS NOT NULL)
        ORDER BY components.name, components.id
        "#,
            )
            .bind(capability != "components:read")
            .fetch_all(pool)
            .await?
        }
        auth::CapabilityBoundary::None => return Err(ApiError::Forbidden(capability.into())),
    };

    rows.into_iter()
        .map(|row| {
            Ok(ComponentSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                description: row.try_get("description")?,
                current_version_id: row.try_get("current_version_id")?,
                current_component_type: row.try_get("current_component_type")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(ApiError::from)
}

async fn load_component_definition(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    component_id: Uuid,
    capability: &str,
) -> ApiResult<ComponentDefinition> {
    let boundary = auth::capability_boundary(pool, account, capability).await?;
    require_component_visible_for_boundary(pool, component_id, &boundary, capability).await?;
    let component = sqlx::query("SELECT id, name, slug, description FROM components WHERE id = $1")
        .bind(component_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("component {component_id}")))?;
    let versions = load_component_versions(pool, account, component_id, capability).await?;
    Ok(ComponentDefinition {
        id: component.try_get("id")?,
        name: component.try_get("name")?,
        slug: component.try_get("slug")?,
        description: component.try_get("description")?,
        versions,
    })
}

async fn parse_component_ref(pool: &sqlx::PgPool, component_ref: &str) -> ApiResult<Uuid> {
    sqlx::query_scalar("SELECT id FROM components WHERE id::text = $1 OR slug = $1")
        .bind(component_ref)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("component {component_ref}")))
}

fn component_validation_response(
    findings: Vec<ComponentValidationFinding>,
) -> ComponentValidationResponse {
    ComponentValidationResponse {
        valid: findings.iter().all(|finding| finding.severity != "error"),
        findings,
    }
}

fn component_validation_finding_from_error(
    code: &str,
    field_path: &str,
    error: ApiError,
) -> ComponentValidationFinding {
    ComponentValidationFinding {
        code: code.into(),
        severity: "error".into(),
        field_path: Some(field_path.into()),
        message: validation_error_message(error),
    }
}

fn component_config_validation_finding(error: ApiError) -> ComponentValidationFinding {
    let message = validation_error_message(error);
    let lower = message.to_ascii_lowercase();
    let (code, field_path) = if lower.contains("unsupported component type") {
        ("COMPONENT_UNSUPPORTED_KIND", "component_type")
    } else if lower.contains("duplicate") && lower.contains("metric") {
        ("COMPONENT_DUPLICATE_METRIC_KEY", "config.metrics")
    } else if lower.contains("unsupported aggregate function") {
        ("COMPONENT_UNSUPPORTED_AGGREGATE_FUNCTION", "config.metrics")
    } else if lower.contains("aggregate table pre-filter")
        || lower.contains("aggregate table post-filter")
        || lower.contains("detail table filter")
        || lower.contains("filter operator")
    {
        ("COMPONENT_FILTER_FIELD_NOT_IN_MAJOR_LINE", "config.filters")
    } else if lower.contains("sort") {
        (
            "COMPONENT_SORT_FIELD_NOT_IN_MAJOR_LINE",
            "config.default_sort",
        )
    } else if lower.contains("aggregation")
        || lower.contains("aggregate")
        || lower.contains("metric")
    {
        (
            "COMPONENT_AGGREGATE_FIELD_NOT_IN_MAJOR_LINE",
            "config.metrics",
        )
    } else {
        ("COMPONENT_FIELD_NOT_IN_MAJOR_LINE", "config")
    };
    ComponentValidationFinding {
        code: code.into(),
        severity: "error".into(),
        field_path: Some(field_path.into()),
        message,
    }
}

fn validation_error_message(error: ApiError) -> String {
    match error {
        ApiError::BadRequest(message) | ApiError::NotFound(message) => message,
        ApiError::Forbidden(capability) => {
            format!("The current account is missing required capability '{capability}'.")
        }
        ApiError::Unauthorized
        | ApiError::InvalidCredentials
        | ApiError::SessionExpired
        | ApiError::SessionRevoked => "Authentication is required.".into(),
        ApiError::Database(_) | ApiError::Internal(_) => {
            "An internal server error occurred.".into()
        }
    }
}

async fn load_component_versions(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    component_id: Uuid,
    capability: &str,
) -> ApiResult<Vec<ComponentVersionSummary>> {
    let rows = match auth::capability_boundary(pool, account, capability).await? {
        auth::CapabilityBoundary::Scoped(scope_ids) => sqlx::query(
            r#"
        SELECT component_versions.id, component_versions.component_id,
               component_versions.dataset_id, component_versions.dataset_version_major,
               component_versions.binding_mode, component_versions.dataset_revision_id,
               component_versions.component_type::text AS component_type,
               component_versions.status::text AS status, component_versions.version_label, component_versions.config
        FROM component_versions
        JOIN dataset_scope_nodes ON dataset_scope_nodes.dataset_id = component_versions.dataset_id
        WHERE component_id = $1
          AND dataset_scope_nodes.node_id = ANY($2)
          AND ($3 OR component_versions.status = 'published'::component_version_status)
        ORDER BY component_versions.version_number DESC, component_versions.created_at DESC
        "#,
        )
        .bind(component_id)
        .bind(scope_ids)
        .bind(capability != "components:read")
        .fetch_all(pool)
        .await?,
        auth::CapabilityBoundary::Global => sqlx::query(
            r#"
        SELECT id, component_id, dataset_id, dataset_version_major, binding_mode, dataset_revision_id, component_type::text AS component_type,
               status::text AS status, version_label, config
        FROM component_versions
        WHERE component_id = $1
          AND ($2 OR status = 'published'::component_version_status)
        ORDER BY component_versions.version_number DESC, component_versions.created_at DESC
        "#,
        )
        .bind(component_id)
        .bind(capability != "components:read")
        .fetch_all(pool)
        .await?,
        auth::CapabilityBoundary::None => return Err(ApiError::Forbidden(capability.into())),
    };
    Ok(rows
        .into_iter()
        .map(|row| {
            Ok(ComponentVersionSummary {
                id: row.try_get("id")?,
                component_id: row.try_get("component_id")?,
                dataset_id: row.try_get("dataset_id")?,
                dataset_version_major: row.try_get("dataset_version_major")?,
                binding_mode: row.try_get("binding_mode")?,
                dataset_revision_id: row.try_get("dataset_revision_id")?,
                component_type: row.try_get("component_type")?,
                status: row.try_get("status")?,
                version_label: row.try_get("version_label")?,
                config: row.try_get("config")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

fn validate_component_type(component_type: &str) -> ApiResult<()> {
    match component_type {
        "detail_table" | "aggregate_table" => Ok(()),
        other => Err(ApiError::BadRequest(format!(
            "unsupported component type '{other}'"
        ))),
    }
}

#[derive(Deserialize)]
struct DetailTableConfig {
    #[serde(default)]
    columns: Vec<ComponentFieldRef>,
    #[serde(default)]
    default_filters: Vec<ComponentFilterConfig>,
    #[serde(default)]
    search_fields: Vec<String>,
    #[serde(default)]
    default_sort: Option<ComponentSortConfig>,
    #[serde(default)]
    page_size: Option<usize>,
}

#[derive(Deserialize)]
struct AggregateTableConfig {
    #[serde(default)]
    pre_filters: Vec<ComponentFilterConfig>,
    #[serde(default)]
    group_fields: Vec<String>,
    #[serde(default)]
    metrics: Vec<ComponentAggregateMetricConfig>,
    #[serde(default)]
    post_filters: Vec<ComponentFilterConfig>,
    #[serde(default)]
    default_sort: Option<ComponentSortConfig>,
    #[serde(default)]
    page_size: Option<usize>,
}

#[derive(Clone, Deserialize)]
struct ComponentSortConfig {
    field_key: String,
    #[serde(default = "default_sort_direction")]
    direction: String,
}

fn default_sort_direction() -> String {
    "asc".into()
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ComponentFieldRef {
    Key(String),
    FieldKey { field_key: String },
    ObjectKey { key: String },
}

impl ComponentFieldRef {
    fn field_key(&self) -> &str {
        match self {
            Self::Key(key) => key,
            Self::FieldKey { field_key } => field_key,
            Self::ObjectKey { key } => key,
        }
    }
}

#[derive(Deserialize)]
struct ComponentFilterConfig {
    field_key: String,
    operator: String,
    #[serde(default)]
    value: Option<String>,
}

#[derive(Deserialize)]
struct ComponentAggregateMetricConfig {
    key: String,
    label: String,
    function: String,
    #[serde(default)]
    source_field_key: Option<String>,
    #[serde(default)]
    position: Option<i32>,
}

fn validate_component_config(
    component_type: &str,
    config: &serde_json::Value,
    fields: &[DataField],
) -> ApiResult<()> {
    match component_type {
        "detail_table" => validate_detail_table_config(config, fields),
        "aggregate_table" => validate_aggregate_table_config(config, fields),
        _ => validate_component_type(component_type),
    }
}

fn validate_detail_table_config(config: &serde_json::Value, fields: &[DataField]) -> ApiResult<()> {
    let config: DetailTableConfig = serde_json::from_value(config.clone()).map_err(|error| {
        ApiError::BadRequest(format!("detail table config is invalid: {error}"))
    })?;
    if config.columns.is_empty() {
        return Err(ApiError::BadRequest(
            "detail table config requires at least one column".into(),
        ));
    }
    for column in &config.columns {
        require_component_field(fields, column.field_key(), "detail table column")?;
    }
    for field_key in &config.search_fields {
        require_component_field(fields, field_key, "detail table search field")?;
    }
    validate_component_filters(&config.default_filters, fields, "detail table filter")?;
    validate_component_sort(&config.default_sort, fields, "detail table sort")
}

fn validate_aggregate_table_config(
    config: &serde_json::Value,
    fields: &[DataField],
) -> ApiResult<()> {
    let config: AggregateTableConfig = serde_json::from_value(config.clone()).map_err(|error| {
        ApiError::BadRequest(format!("aggregate table config is invalid: {error}"))
    })?;
    if config.metrics.is_empty() {
        return Err(ApiError::BadRequest(
            "aggregate table config requires at least one metric".into(),
        ));
    }
    validate_component_filters(&config.pre_filters, fields, "aggregate table pre-filter")?;
    let validated_plan = validated_aggregation_plan_from_config(&config, fields)?;
    let mut aggregate_fields = validated_plan
        .group_fields
        .into_iter()
        .filter_map(|key| fields.iter().find(|field| field.key == key).cloned())
        .collect::<Vec<_>>();
    aggregate_fields.extend(validated_plan.metrics.into_iter().map(|metric| DataField {
        key: metric.key,
        label: metric.label,
        field_type: metric.output_field_type,
        position: metric.position,
    }));
    validate_component_filters(
        &config.post_filters,
        &aggregate_fields,
        "aggregate table post-filter",
    )?;
    validate_component_sort(
        &config.default_sort,
        &aggregate_fields,
        "aggregate table sort",
    )
}

fn validate_component_sort(
    sort: &Option<ComponentSortConfig>,
    fields: &[DataField],
    label: &str,
) -> ApiResult<()> {
    if let Some(sort) = sort {
        require_component_field(fields, &sort.field_key, label)?;
        match sort.direction.to_ascii_lowercase().as_str() {
            "asc" | "desc" => {}
            _ => {
                return Err(ApiError::BadRequest(format!(
                    "{label} direction must be asc or desc"
                )));
            }
        }
    }
    Ok(())
}

fn validate_component_filters(
    filters: &[ComponentFilterConfig],
    fields: &[DataField],
    label: &str,
) -> ApiResult<()> {
    for filter in filters {
        let field = require_component_field(fields, &filter.field_key, label)?;
        let operator = FilterOperator::parse(&filter.operator).map_err(component_data_op_error)?;
        operator
            .validate_for_field(field)
            .map_err(component_data_op_error)?;
        if operator.requires_value()
            && filter
                .value
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
        {
            return Err(ApiError::BadRequest(format!(
                "{label} on '{}' requires a value for operator '{}'",
                filter.field_key,
                operator.as_str()
            )));
        }
    }
    Ok(())
}

fn validated_aggregation_plan_from_config(
    config: &AggregateTableConfig,
    fields: &[DataField],
) -> ApiResult<ValidatedAggregationPlan> {
    let plan = AggregationPlan {
        group_fields: config.group_fields.clone(),
        metrics: config
            .metrics
            .iter()
            .enumerate()
            .map(|(index, metric)| {
                let function = component_aggregate_function(&metric.function)?;
                Ok(AggregateMetric {
                    key: metric.key.clone(),
                    label: metric.label.clone(),
                    function,
                    source_field_key: metric.source_field_key.clone(),
                    position: metric.position.unwrap_or(index as i32),
                })
            })
            .collect::<ApiResult<Vec<_>>>()?,
    };
    validate_aggregation_plan(plan, fields).map_err(component_data_op_error)
}

fn component_aggregate_function(function: &str) -> ApiResult<AggregateFunction> {
    AggregateFunction::parse(function).map_err(component_data_op_error)
}

fn require_component_field<'a>(
    fields: &'a [DataField],
    field_key: &str,
    label: &str,
) -> ApiResult<&'a DataField> {
    fields
        .iter()
        .find(|field| field.key == field_key)
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "{label} references field '{field_key}' outside the dataset major-line contract"
            ))
        })
}

fn component_data_op_error(error: tessara_data_ops::DataOpError) -> ApiError {
    ApiError::BadRequest(error.message().to_string())
}

async fn load_dataset_major_line_fields(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
    dataset_version_major: i32,
) -> ApiResult<Vec<DataField>> {
    let output_fields: Option<serde_json::Value> = sqlx::query_scalar(
        r#"
        SELECT output_fields
        FROM dataset_revisions
        WHERE dataset_id = $1
          AND version_major = $2
          AND status IN ('published'::dataset_revision_status, 'superseded'::dataset_revision_status)
        ORDER BY COALESCE(version_minor, 0) DESC,
                 COALESCE(version_patch, 0) DESC,
                 version_number DESC
        LIMIT 1
        "#,
    )
    .bind(dataset_id)
    .bind(dataset_version_major)
    .fetch_optional(pool)
    .await?
    .flatten();
    let output_fields = output_fields.ok_or_else(|| {
        ApiError::BadRequest(format!(
            "dataset {dataset_id} major version {dataset_version_major} has no field contract"
        ))
    })?;
    let fields = serde_json::from_value::<Vec<datasets::DatasetFieldDefinition>>(output_fields)
        .map_err(|error| {
            ApiError::Internal(anyhow::anyhow!(
                "stored dataset major-line field contract is invalid: {error}"
            ))
        })?;
    Ok(fields
        .into_iter()
        .map(|field| DataField {
            key: field.key,
            label: field.label,
            field_type: FieldType::parse(&field.field_type),
            position: field.position,
        })
        .collect())
}

async fn require_component_exists(pool: &sqlx::PgPool, component_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM components WHERE id = $1)")
        .bind(component_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("component {component_id}")))
    }
}

async fn resolve_component_dataset_binding(
    pool: &sqlx::PgPool,
    payload: &CreateComponentVersionRequest,
) -> ApiResult<ComponentDatasetBinding> {
    match (
        payload.dataset_id,
        payload.dataset_version_major,
        payload.dataset_revision_id,
    ) {
        (Some(dataset_id), Some(dataset_version_major), legacy_dataset_revision_id) => {
            Ok(ComponentDatasetBinding {
                dataset_id,
                dataset_version_major,
                legacy_dataset_revision_id,
            })
        }
        (None, None, Some(dataset_revision_id)) => {
            let row = sqlx::query(
                r#"
                SELECT dataset_id, version_major
                FROM dataset_revisions
                WHERE id = $1
                "#,
            )
            .bind(dataset_revision_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("dataset revision {dataset_revision_id}")))?;
            let dataset_version_major: Option<i32> = row.try_get("version_major")?;
            Ok(ComponentDatasetBinding {
                dataset_id: row.try_get("dataset_id")?,
                dataset_version_major: dataset_version_major.ok_or_else(|| {
                    ApiError::BadRequest(format!(
                        "dataset revision {dataset_revision_id} has no major version"
                    ))
                })?,
                legacy_dataset_revision_id: Some(dataset_revision_id),
            })
        }
        _ => Err(ApiError::BadRequest(
            "component version requires dataset_id and dataset_version_major".into(),
        )),
    }
}

async fn require_dataset_major_line_exists(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
    dataset_version_major: i32,
) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM dataset_revisions
            WHERE dataset_id = $1
              AND version_major = $2
              AND status IN ('published'::dataset_revision_status, 'superseded'::dataset_revision_status)
        )
        "#,
    )
    .bind(dataset_id)
    .bind(dataset_version_major)
    .fetch_one(pool)
    .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!(
            "dataset {dataset_id} major version {dataset_version_major}"
        )))
    }
}

async fn require_dataset_fully_in_capability_scope(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    capability: &str,
    dataset_id: Uuid,
) -> ApiResult<()> {
    let node_ids = datasets::load_dataset_scope_node_ids(pool, dataset_id).await?;
    auth::require_capability_contains_nodes(pool, account, capability, &node_ids).await
}

async fn require_component_visible_for_boundary(
    pool: &sqlx::PgPool,
    component_id: Uuid,
    boundary: &auth::CapabilityBoundary,
    capability: &str,
) -> ApiResult<()> {
    match boundary {
        auth::CapabilityBoundary::Global if capability == "components:read" => {
            let visible = sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM component_versions
                    WHERE component_id = $1
                      AND status = 'published'::component_version_status
                )
                "#,
            )
            .bind(component_id)
            .fetch_one(pool)
            .await?;
            if visible {
                Ok(())
            } else {
                Err(ApiError::Forbidden(capability.into()))
            }
        }
        auth::CapabilityBoundary::Global => Ok(()),
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            let visible = sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM component_versions
                    JOIN dataset_scope_nodes
                      ON dataset_scope_nodes.dataset_id = component_versions.dataset_id
                    WHERE component_versions.component_id = $1
                      AND dataset_scope_nodes.node_id = ANY($2)
                      AND ($3 OR component_versions.status = 'published'::component_version_status)
                )
                "#,
            )
            .bind(component_id)
            .bind(scope_ids)
            .bind(capability != "components:read")
            .fetch_one(pool)
            .await?;
            if visible {
                Ok(())
            } else {
                Err(ApiError::Forbidden(capability.into()))
            }
        }
        auth::CapabilityBoundary::None => Err(ApiError::Forbidden(capability.into())),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::collections::HashMap;
    use tessara_data_ops::{DataField, FieldType};

    use uuid::Uuid;

    use super::{
        ComponentTableQuery, ComponentVersionForTable, CreateComponentRequest,
        aggregate_metric_sql, aggregate_search_fields, component_filter_sql,
        component_pagination_sql, detail_search_fields, effective_component_page_size,
        parse_component_query_filters, parse_component_sort, require_component_version_draft,
        table_order_by_sql, validate_component_config, visible_table_fields,
    };

    fn field(key: &str, field_type: FieldType) -> DataField {
        DataField {
            key: key.into(),
            label: key.into(),
            field_type,
            position: 0,
        }
    }

    #[test]
    fn aggregate_table_config_validates_with_shared_data_ops() {
        let fields = vec![
            field("program", FieldType::Text),
            field("amount", FieldType::Number),
        ];
        let config = json!({
            "group_fields": ["program"],
            "metrics": [{
                "key": "total_amount",
                "label": "Total amount",
                "function": "sum",
                "source_field_key": "amount"
            }],
            "pre_filters": [{
                "field_key": "program",
                "operator": "not_contains",
                "value": "archived"
            }],
            "post_filters": [{
                "field_key": "total_amount",
                "operator": "gt",
                "value": "0"
            }]
        });

        validate_component_config("aggregate_table", &config, &fields)
            .expect("valid aggregate table config should pass");
    }

    #[test]
    fn aggregate_table_config_accepts_public_component_functions() {
        let fields = vec![
            field("program", FieldType::Text),
            field("amount", FieldType::Number),
        ];

        for (function, source_field_key) in [
            ("count", None),
            ("count_values", Some("program")),
            ("count_distinct", Some("program")),
            ("sum", Some("amount")),
            ("avg", Some("amount")),
            ("min", Some("program")),
            ("max", Some("program")),
        ] {
            let mut metric = json!({
                "key": format!("{function}_metric"),
                "label": format!("{function} metric"),
                "function": function,
            });
            if let Some(source_field_key) = source_field_key {
                metric["source_field_key"] = json!(source_field_key);
            }
            let config = json!({
                "group_fields": ["program"],
                "metrics": [metric]
            });

            validate_component_config("aggregate_table", &config, &fields)
                .unwrap_or_else(|error| panic!("{function} should validate: {error}"));
        }
    }

    #[test]
    fn aggregate_table_config_accepts_count_values_for_present_source_values() {
        let fields = vec![field("program", FieldType::Text)];
        let config = json!({
            "group_fields": ["program"],
            "metrics": [{
                "key": "value_count",
                "label": "Values",
                "function": "count_values",
                "source_field_key": "program"
            }]
        });

        validate_component_config("aggregate_table", &config, &fields)
            .expect("count_values should be exposed for component aggregate tables");
    }

    #[test]
    fn count_values_aggregation_counts_non_empty_source_values() {
        let metric = tessara_data_ops::ValidatedAggregateMetric {
            key: "present_count".into(),
            label: "Present Values".into(),
            function: tessara_data_ops::AggregateFunction::CountValues,
            source_field_key: Some("program".into()),
            output_field_type: FieldType::Number,
            position: 0,
        };

        assert_eq!(
            aggregate_metric_sql(&metric),
            "COUNT(NULLIF(\"program\", ''))::text AS \"present_count\""
        );
    }

    #[test]
    fn aggregate_table_config_rejects_missing_metric_source_field() {
        let fields = vec![field("program", FieldType::Text)];
        let config = json!({
            "group_fields": ["program"],
            "metrics": [{
                "key": "total_amount",
                "label": "Total amount",
                "function": "sum",
                "source_field_key": "amount"
            }]
        });

        let error = validate_component_config("aggregate_table", &config, &fields)
            .expect_err("missing metric source should fail");
        assert!(
            error
                .to_string()
                .contains("references field 'amount' outside the field contract")
        );
    }

    #[test]
    fn detail_table_config_rejects_missing_columns() {
        let fields = vec![field("program", FieldType::Text)];
        let config = json!({
            "columns": ["program", "amount"]
        });

        let error = validate_component_config("detail_table", &config, &fields)
            .expect_err("unknown detail column should fail");
        assert!(
            error
                .to_string()
                .contains("detail table column references field 'amount'")
        );
    }

    #[test]
    fn component_filter_sql_supports_negative_operator() {
        let fields = vec![field("program", FieldType::Text)];
        let filters = vec![super::ComponentFilterConfig {
            field_key: "program".into(),
            operator: "not_contains".into(),
            value: Some("archived".into()),
        }];

        let sql = component_filter_sql(&filters, &fields).expect("filter should compile");
        assert_eq!(
            sql,
            vec!["POSITION(LOWER('archived') IN LOWER(COALESCE(\"program\", ''))) = 0"]
        );
    }

    #[test]
    fn component_filter_sql_validates_operator_field_compatibility() {
        let fields = vec![field("score", FieldType::Number)];
        let filters = vec![super::ComponentFilterConfig {
            field_key: "score".into(),
            operator: "contains".into(),
            value: Some("10".into()),
        }];

        let error = component_filter_sql(&filters, &fields)
            .expect_err("text operator on numeric field should fail");
        assert!(
            error
                .to_string()
                .contains("filter operator 'contains' is not supported")
        );
    }

    #[test]
    fn component_table_query_parses_runtime_filters_and_cursor() {
        let mut extra = HashMap::new();
        extra.insert("filter[program][operator]".into(), "not_contains".into());
        extra.insert("filter[program][value]".into(), "archived".into());
        let query = ComponentTableQuery {
            q: Some(" demo ".into()),
            page_size: Some(500),
            cursor: Some("offset:25".into()),
            sort: Some("program:desc".into()),
            visible_columns: Some("program, row_count".into()),
            extra,
        }
        .into_runtime_query()
        .expect("query should parse");

        assert_eq!(query.search.as_deref(), Some("demo"));
        assert_eq!(query.page_size, Some(200));
        assert_eq!(query.offset, 25);
        assert_eq!(query.visible_columns, vec!["program", "row_count"]);
        assert_eq!(query.filters[0].field_key, "program");
        assert_eq!(query.filters[0].operator, "not_contains");
        assert_eq!(query.filters[0].value.as_deref(), Some("archived"));
        assert_eq!(query.sort.expect("sort").direction, "desc");
    }

    #[test]
    fn component_table_sort_and_page_sql_are_server_driven() {
        let fields = vec![
            field("program", FieldType::Text),
            field("score", FieldType::Number),
        ];
        let refs = fields.iter().collect::<Vec<_>>();
        let sort = parse_component_sort("score:desc").expect("sort should parse");
        let order_by =
            table_order_by_sql(Some(&sort), &refs, "__row_id").expect("sort should compile");

        assert!(order_by.contains("\"score\""));
        assert!(order_by.contains("DESC"));
        assert_eq!(component_pagination_sql(25, 50), " LIMIT 51 OFFSET 25");
        assert_eq!(effective_component_page_size(None, Some(500)), 200);
        assert_eq!(effective_component_page_size(Some(25), Some(500)), 25);
    }

    #[test]
    fn visible_table_fields_preserves_requested_order_and_rejects_unknown_columns() {
        let fields = vec![
            field("program", FieldType::Text),
            field("score", FieldType::Number),
        ];
        let refs = fields.iter().collect::<Vec<_>>();
        let selected = visible_table_fields(&refs, &["score".into(), "program".into()])
            .expect("known visible columns should pass");
        assert_eq!(
            selected
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["score", "program"]
        );

        let error = visible_table_fields(&refs, &["missing".into()])
            .expect_err("unknown visible column should fail");
        assert!(
            error
                .to_string()
                .contains("visible column 'missing' is outside")
        );
    }

    #[test]
    fn detail_search_defaults_to_configured_columns_not_visible_subset() {
        let config = super::DetailTableConfig {
            columns: vec![
                super::ComponentFieldRef::Key("program".into()),
                super::ComponentFieldRef::Key("score".into()),
            ],
            default_sort: None,
            default_filters: Vec::new(),
            search_fields: Vec::new(),
            page_size: None,
        };
        let fields = vec![
            field("program", FieldType::Text),
            field("score", FieldType::Number),
        ];
        let configured = fields.iter().collect::<Vec<_>>();

        let search_fields =
            detail_search_fields(&config, &configured, &fields).expect("search fields");

        assert_eq!(
            search_fields
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["program", "score"]
        );
    }

    #[test]
    fn aggregate_search_defaults_to_output_contract_not_visible_subset() {
        let fields = vec![
            field("program", FieldType::Text),
            field("row_count", FieldType::Number),
        ];
        let refs = fields.iter().collect::<Vec<_>>();
        let selected = visible_table_fields(&refs, &["row_count".into()])
            .expect("visible column projection should pass");

        let search_fields = aggregate_search_fields(&refs);

        assert_eq!(
            selected
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["row_count"]
        );
        assert_eq!(
            search_fields
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["program", "row_count"]
        );
    }

    #[test]
    fn component_query_filters_require_operators() {
        let mut extra = HashMap::new();
        extra.insert("filter[program][value]".into(), "demo".into());

        let error = match parse_component_query_filters(&extra) {
            Ok(_) => panic!("missing operator should fail"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("missing an operator"));
    }

    #[test]
    fn component_publish_guard_rejects_immutable_versions() {
        let version_id = Uuid::nil();

        assert!(require_component_version_draft(version_id, "draft").is_ok());
        assert!(require_component_version_draft(version_id, "published").is_err());
        assert!(require_component_version_draft(version_id, "superseded").is_err());
    }

    #[test]
    fn component_table_without_materialization_uses_pending_state() {
        let version = ComponentVersionForTable {
            id: Uuid::new_v4(),
            component_id: Uuid::new_v4(),
            dataset_id: Uuid::new_v4(),
            dataset_version_major: 1,
            component_type: "detail_table".into(),
            config: json!({ "columns": ["program"] }),
        };

        let table = super::empty_component_table(version, "pending", Vec::new());

        assert_eq!(table.materialization_state, "pending");
        assert!(table.rows.is_empty());
        assert_eq!(table.pagination.page_size, 0);
        assert!(!table.pagination.has_more);
    }

    #[test]
    fn component_table_materialization_failure_is_render_state() {
        let version = ComponentVersionForTable {
            id: Uuid::new_v4(),
            component_id: Uuid::new_v4(),
            dataset_id: Uuid::new_v4(),
            dataset_version_major: 1,
            component_type: "aggregate_table".into(),
            config: json!({ "metrics": [{ "key": "row_count", "label": "Rows", "function": "count" }] }),
        };

        let table = super::empty_component_table(version, "failed", Vec::new());

        assert_eq!(table.materialization_state, "failed");
        assert!(table.rows.is_empty());
        assert!(!table.pagination.has_more);
    }

    #[test]
    fn create_component_request_accepts_first_version_payload() {
        let dataset_id = Uuid::nil();
        let payload: CreateComponentRequest = serde_json::from_value(json!({
            "name": "Program table",
            "slug": "program-table",
            "description": "A first table component",
            "version": {
                "dataset_id": dataset_id,
                "dataset_version_major": 1,
                "component_type": "detail_table",
                "config": {
                    "columns": ["program"]
                }
            }
        }))
        .expect("atomic create payload should deserialize");

        assert_eq!(payload.name, "Program table");
        let version = payload.version.expect("version should be present");
        assert_eq!(version.dataset_id, Some(dataset_id));
        assert_eq!(version.dataset_version_major, Some(1));
        assert_eq!(version.component_type, "detail_table");
    }
}
