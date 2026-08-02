//! Component authoring and read endpoints.
//!
//! Components are presentation assets over dataset major lines. This module keeps
//! route behavior and scope checks together while the public wire types live in
//! `dto`.

use std::collections::{BTreeMap, HashMap};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::{Postgres, Row, Transaction};
use tessara_data_ops::{DataField, FieldType};
use uuid::Uuid;

mod dashboard_compatibility;
mod dto;
mod runtime;

use runtime::{
    component_filter_sql, csv_keys, execute_component_table, execute_component_visual,
    parse_component_cursor, parse_component_query_filters, parse_component_sort,
    visible_table_fields,
};

#[cfg(test)]
use runtime::{
    component_pagination_sql, component_visual_source_limit_clause, effective_component_page_size,
    empty_component_table, table_order_by_sql, table_search_fields, visual_from_rows,
};

pub use dto::{
    ComponentDefinition, ComponentStatValue, ComponentSummary, ComponentTable,
    ComponentTableColumn, ComponentTablePagination, ComponentTableRow, ComponentValidationFinding,
    ComponentValidationResponse, ComponentVersionSummary, ComponentVisual, ComponentVisualPoint,
    ComponentVisualSlice, CreateComponentRequest, CreateComponentVersionRequest,
    SaveComponentEditAction, SaveComponentEditRequest, UpdateComponentRequest,
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
}

fn parse_component_payload<T>(body: &[u8]) -> ApiResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_slice(body)
        .map_err(|error| ApiError::BadRequest(format!("invalid component payload: {error}")))
}

pub(crate) fn routes() -> Router<AppState> {
    // The edit-screen authoring workflow is POST /api/admin/components/save.
    // The granular admin routes remain internal API/setup primitives for tests,
    // migrations, and narrowly scoped lifecycle operations such as draft delete.
    Router::new()
        .route(
            "/api/admin/components",
            get(list_admin_components).post(create_component),
        )
        .route("/api/admin/components/save", post(save_component_edit))
        .route("/api/admin/components/preview", post(preview_component))
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
            axum::routing::patch(update_component_version).delete(delete_component_version),
        )
        .route(
            "/api/admin/components/{component_id}/versions/{version_id}/published",
            axum::routing::patch(update_published_component_version),
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
        .route(
            "/api/components/{component_ref}/bar",
            get(run_component_bar),
        )
        .route(
            "/api/components/{component_ref}/versions/{version_id}/bar",
            get(run_component_version_bar),
        )
        .route(
            "/api/components/{component_ref}/line",
            get(run_component_line),
        )
        .route(
            "/api/components/{component_ref}/versions/{version_id}/line",
            get(run_component_version_line),
        )
        .route(
            "/api/components/{component_ref}/pie",
            get(run_component_pie),
        )
        .route(
            "/api/components/{component_ref}/versions/{version_id}/pie",
            get(run_component_version_pie),
        )
        .route(
            "/api/components/{component_ref}/donut",
            get(run_component_donut),
        )
        .route(
            "/api/components/{component_ref}/versions/{version_id}/donut",
            get(run_component_version_donut),
        )
        .route(
            "/api/components/{component_ref}/stat-card",
            get(run_component_stat_card),
        )
        .route(
            "/api/components/{component_ref}/versions/{version_id}/stat-card",
            get(run_component_version_stat_card),
        )
}

/// Creates a component shell before versions are attached.
pub async fn create_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<IdResponse>> {
    let payload = parse_component_payload::<CreateComponentRequest>(&body)?;
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
        upsert_component_draft_version(&mut tx, id, binding, version).await?;
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
    body: Bytes,
) -> ApiResult<Json<IdResponse>> {
    let payload = parse_component_payload::<UpdateComponentRequest>(&body)?;
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    require_component_exists(&state.pool, component_id).await?;
    require_component_fully_manageable(&state.pool, &account, component_id).await?;
    require_text("component name", &payload.name)?;
    require_text("component slug", &payload.slug)?;
    let mut tx = state.pool.begin().await?;
    update_component_shell_in_tx(&mut tx, component_id, &payload).await?;
    tx.commit().await?;
    Ok(Json(IdResponse { id: component_id }))
}

/// Atomically applies the edit-screen component metadata plus version action.
pub async fn save_component_edit(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<IdResponse>> {
    let payload = parse_component_payload::<SaveComponentEditRequest>(&body)?;
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    require_text("component name", &payload.component.name)?;
    require_text("component slug", &payload.component.slug)?;
    validate_component_version_note(payload.version.version_note.as_deref())?;
    if payload.action == SaveComponentEditAction::CreateNewVersion {
        require_new_version_note(payload.version.version_note.as_deref().unwrap_or_default())?;
    }

    let binding = resolve_component_dataset_binding(&state.pool, &payload.version).await?;
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
    validate_component_type(&payload.version.component_type)?;
    let dataset_fields = load_dataset_major_line_fields(
        &state.pool,
        binding.dataset_id,
        binding.dataset_version_major,
    )
    .await?;
    validate_component_config(
        &payload.version.component_type,
        &payload.version.config,
        &dataset_fields,
    )?;

    let published_update_id = if payload.component_id.is_some()
        && payload.action == SaveComponentEditAction::UpdateExistingVersion
    {
        Some(payload.published_version_id.ok_or_else(|| {
            ApiError::BadRequest(
                "updating an existing component version requires a published version".into(),
            )
        })?)
    } else {
        None
    };
    let mut tx = state.pool.begin().await?;
    if let Some(version_id) = published_update_id {
        dashboard_compatibility::prepare_published_update(&mut tx, version_id).await?;
    }
    let component_id = if let Some(component_id) = payload.component_id {
        lock_component_in_tx(&mut tx, component_id).await?;
        require_component_fully_manageable_in_tx(&mut tx, &state.pool, &account, component_id)
            .await?;
        update_component_shell_in_tx(&mut tx, component_id, &payload.component).await?;
        match payload.action {
            SaveComponentEditAction::SaveDraft => {
                if let Some(version_id) = payload.draft_version_id {
                    update_component_version_row_in_tx(
                        &mut tx,
                        component_id,
                        version_id,
                        "draft",
                        &binding,
                        &payload.version,
                    )
                    .await?;
                } else {
                    upsert_component_draft_version(
                        &mut tx,
                        component_id,
                        &binding,
                        &payload.version,
                    )
                    .await?;
                }
            }
            SaveComponentEditAction::UpdateExistingVersion => {
                let version_id = published_update_id.ok_or_else(|| {
                    ApiError::BadRequest(
                        "updating an existing component version requires a published version"
                            .into(),
                    )
                })?;
                update_component_version_row_in_tx(
                    &mut tx,
                    component_id,
                    version_id,
                    "published",
                    &binding,
                    &payload.version,
                )
                .await?;
                delete_component_drafts_in_tx(&mut tx, component_id).await?;
            }
            SaveComponentEditAction::CreateNewVersion => {
                let version_id = if let Some(version_id) = payload.draft_version_id {
                    update_component_version_row_in_tx(
                        &mut tx,
                        component_id,
                        version_id,
                        "draft",
                        &binding,
                        &payload.version,
                    )
                    .await?;
                    version_id
                } else {
                    upsert_component_draft_version(
                        &mut tx,
                        component_id,
                        &binding,
                        &payload.version,
                    )
                    .await?
                };
                publish_component_version_in_tx(
                    &mut tx,
                    &state.pool,
                    &account,
                    component_id,
                    version_id,
                )
                .await?;
            }
        }
        component_id
    } else {
        let component_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO components (name, slug, description)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(payload.component.name.trim())
        .bind(payload.component.slug.trim())
        .bind(&payload.component.description)
        .fetch_one(&mut *tx)
        .await?;
        let version_id =
            upsert_component_draft_version(&mut tx, component_id, &binding, &payload.version)
                .await?;
        match payload.action {
            SaveComponentEditAction::SaveDraft => {}
            SaveComponentEditAction::CreateNewVersion => {
                publish_component_version_in_tx(
                    &mut tx,
                    &state.pool,
                    &account,
                    component_id,
                    version_id,
                )
                .await?;
            }
            SaveComponentEditAction::UpdateExistingVersion => {
                return Err(ApiError::BadRequest(
                    "updating an existing component version requires an existing component".into(),
                ));
            }
        }
        component_id
    };

    tx.commit().await?;
    Ok(Json(IdResponse { id: component_id }))
}

async fn update_component_shell_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    payload: &UpdateComponentRequest,
) -> ApiResult<()> {
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
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Validates a component version payload against the bound Dataset major-line contract.
pub async fn validate_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<ComponentValidationResponse>> {
    let payload = parse_component_payload::<CreateComponentVersionRequest>(&body)?;
    validate_component_version_note(payload.version_note.as_deref())?;
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

/// Executes an unsaved visual component config against its bound Dataset major line.
pub async fn preview_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<ComponentVisual>> {
    let payload = parse_component_payload::<CreateComponentVersionRequest>(&body)?;
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
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

    let visual_kind = match payload.component_type.as_str() {
        "bar" => "bar",
        "line" => "line",
        "pie" => "pie",
        "donut" => "donut",
        "stat_card" => "stat_card",
        component_type => {
            return Err(ApiError::BadRequest(format!(
                "component preview does not support type '{component_type}'"
            )));
        }
    };
    let version = ComponentVersionForTable {
        id: Uuid::nil(),
        component_id: Uuid::nil(),
        dataset_id: binding.dataset_id,
        dataset_version_major: binding.dataset_version_major,
        component_type: payload.component_type,
        config: payload.config,
    };
    execute_component_visual(&state.pool, &account, version, visual_kind, Some(100))
        .await
        .map(Json)
}

/// Creates or updates a draft component version over a dataset major line.
pub async fn create_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    body: Bytes,
) -> ApiResult<Json<IdResponse>> {
    let payload = parse_component_payload::<CreateComponentVersionRequest>(&body)?;
    validate_component_version_note(payload.version_note.as_deref())?;
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
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
    lock_component_in_tx(&mut tx, component_id).await?;
    require_component_fully_manageable_in_tx(&mut tx, &state.pool, &account, component_id).await?;
    let id = upsert_component_draft_version(&mut tx, component_id, &binding, &payload).await?;
    tx.commit().await?;
    Ok(Json(IdResponse { id }))
}

/// Updates a specific draft component version without creating a new version row.
pub async fn update_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
    body: Bytes,
) -> ApiResult<Json<IdResponse>> {
    let payload = parse_component_payload::<CreateComponentVersionRequest>(&body)?;
    validate_component_version_note(payload.version_note.as_deref())?;
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
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

    let mut tx = state.pool.begin().await?;
    lock_component_in_tx(&mut tx, component_id).await?;
    require_component_fully_manageable(&state.pool, &account, component_id).await?;
    require_component_version_draft_row_in_tx(&mut tx, component_id, version_id).await?;
    let dataset_fields = load_dataset_major_line_fields(
        &state.pool,
        binding.dataset_id,
        binding.dataset_version_major,
    )
    .await?;
    validate_component_config(&payload.component_type, &payload.config, &dataset_fields)?;
    let version_note = normalized_component_version_note(payload.version_note.as_deref())?;

    let update_result = sqlx::query(
        r#"
        UPDATE component_versions
        SET dataset_id = $1,
            dataset_version_major = $2,
            binding_mode = 'major_line',
            component_type = $3::component_type,
            config = $4,
            version_note = COALESCE($7, version_note)
        WHERE component_id = $5
          AND id = $6
          AND status = 'draft'::component_version_status
        "#,
    )
    .bind(binding.dataset_id)
    .bind(binding.dataset_version_major)
    .bind(&payload.component_type)
    .bind(&payload.config)
    .bind(component_id)
    .bind(version_id)
    .bind(version_note)
    .execute(&mut *tx)
    .await?;
    if update_result.rows_affected() != 1 {
        return Err(ApiError::BadRequest(format!(
            "component version {version_id} could not be updated because it is no longer a draft"
        )));
    }
    tx.commit().await?;
    Ok(Json(IdResponse { id: version_id }))
}

/// Updates the current published component version in place and clears any pending draft.
pub async fn update_published_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
    body: Bytes,
) -> ApiResult<Json<IdResponse>> {
    let payload = parse_component_payload::<CreateComponentVersionRequest>(&body)?;
    validate_component_version_note(payload.version_note.as_deref())?;
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
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
    dashboard_compatibility::prepare_published_update(&mut tx, version_id).await?;
    lock_component_in_tx(&mut tx, component_id).await?;
    require_component_fully_manageable_in_tx(&mut tx, &state.pool, &account, component_id).await?;
    update_component_version_row_in_tx(
        &mut tx,
        component_id,
        version_id,
        "published",
        &binding,
        &payload,
    )
    .await?;

    sqlx::query(
        r#"
        DELETE FROM component_versions
        WHERE component_id = $1
          AND status = 'draft'::component_version_status
        "#,
    )
    .bind(component_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(IdResponse { id: version_id }))
}

/// Deletes a draft component version without affecting published history.
pub async fn delete_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    let mut tx = state.pool.begin().await?;
    lock_component_in_tx(&mut tx, component_id).await?;
    require_component_fully_manageable_in_tx(&mut tx, &state.pool, &account, component_id).await?;
    require_component_version_draft_row_in_tx(&mut tx, component_id, version_id).await?;

    let delete_result = sqlx::query(
        r#"
        DELETE FROM component_versions
        WHERE component_id = $1
          AND id = $2
          AND status = 'draft'::component_version_status
        "#,
    )
    .bind(component_id)
    .bind(version_id)
    .execute(&mut *tx)
    .await?;
    if delete_result.rows_affected() != 1 {
        return Err(ApiError::BadRequest(format!(
            "component version {version_id} could not be deleted because it is no longer a draft"
        )));
    }
    tx.commit().await?;
    Ok(Json(IdResponse { id: version_id }))
}

/// Publishes a draft component version and atomically supersedes the current published version.
pub async fn publish_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    let mut tx = state.pool.begin().await?;
    publish_component_version_in_tx(&mut tx, &state.pool, &account, component_id, version_id)
        .await?;
    tx.commit().await?;
    Ok(Json(IdResponse { id: version_id }))
}

async fn upsert_component_draft_version(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    binding: &ComponentDatasetBinding,
    payload: &CreateComponentVersionRequest,
) -> ApiResult<Uuid> {
    let version_note = normalized_component_version_note(payload.version_note.as_deref())?;
    let id = sqlx::query_scalar(
        r#"
        WITH next_version AS (
            SELECT COALESCE(MAX(version_number), 0) + 1 AS version_number
            FROM component_versions
            WHERE component_id = $1
        )
        INSERT INTO component_versions
            (component_id, dataset_id, dataset_version_major, binding_mode,
             component_type, version_number, version_label, version_note, status, config)
        SELECT $1, $2, $3, 'major_line', $4::component_type,
               next_version.version_number, next_version.version_number::text,
               COALESCE($6, ''), 'draft'::component_version_status, $5
        FROM next_version
        ON CONFLICT (component_id) WHERE status = 'draft'::component_version_status
        DO UPDATE SET dataset_id = EXCLUDED.dataset_id,
                      dataset_version_major = EXCLUDED.dataset_version_major,
                      binding_mode = EXCLUDED.binding_mode,
                      component_type = EXCLUDED.component_type,
                      config = EXCLUDED.config,
                      version_note = COALESCE($6, component_versions.version_note)
        RETURNING id
        "#,
    )
    .bind(component_id)
    .bind(binding.dataset_id)
    .bind(binding.dataset_version_major)
    .bind(&payload.component_type)
    .bind(&payload.config)
    .bind(version_note)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

async fn update_component_version_row_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    version_id: Uuid,
    status: &str,
    binding: &ComponentDatasetBinding,
    payload: &CreateComponentVersionRequest,
) -> ApiResult<()> {
    require_component_version_status_row_in_tx(tx, component_id, version_id, status).await?;
    let version_note = normalized_component_version_note(payload.version_note.as_deref())?;
    let update_result = sqlx::query(
        r#"
        UPDATE component_versions
        SET dataset_id = $1,
            dataset_version_major = $2,
            binding_mode = 'major_line',
            component_type = $3::component_type,
            config = $4,
            version_note = COALESCE($8, version_note)
        WHERE component_id = $5
          AND id = $6
          AND status = $7::component_version_status
        "#,
    )
    .bind(binding.dataset_id)
    .bind(binding.dataset_version_major)
    .bind(&payload.component_type)
    .bind(&payload.config)
    .bind(component_id)
    .bind(version_id)
    .bind(status)
    .bind(version_note)
    .execute(&mut **tx)
    .await?;
    if update_result.rows_affected() != 1 {
        return Err(ApiError::BadRequest(format!(
            "component version {version_id} could not be updated because it is no longer {status}"
        )));
    }
    if status == "published" {
        dashboard_compatibility::validate_published_update(tx, version_id).await?;
    }
    Ok(())
}

async fn delete_component_drafts_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        DELETE FROM component_versions
        WHERE component_id = $1
          AND status = 'draft'::component_version_status
        "#,
    )
    .bind(component_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn publish_component_version_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    component_id: Uuid,
    version_id: Uuid,
) -> ApiResult<()> {
    lock_component_in_tx(tx, component_id).await?;
    let row = sqlx::query(
        r#"
        SELECT dataset_id, dataset_version_major, component_type::text AS component_type,
               config, status::text AS status, version_note
        FROM component_versions
        WHERE component_id = $1
          AND id = $2
        FOR UPDATE
        "#,
    )
    .bind(component_id)
    .bind(version_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("component version {version_id}")))?;
    let status: String = row.try_get("status")?;
    require_component_version_draft(version_id, &status)?;
    let dataset_id: Uuid = row.try_get("dataset_id")?;
    let dataset_version_major = row.try_get("dataset_version_major")?;
    let component_type: String = row.try_get("component_type")?;
    let config = row.try_get("config")?;
    let version_note: String = row.try_get("version_note")?;

    require_component_fully_manageable_in_tx(tx, pool, account, component_id).await?;
    require_dataset_fully_in_capability_scope(pool, account, "components:manage", dataset_id)
        .await?;
    validate_component_type(&component_type)?;
    let dataset_fields =
        load_dataset_major_line_fields(pool, dataset_id, dataset_version_major).await?;
    validate_component_config(&component_type, &config, &dataset_fields)?;
    require_new_version_note_when_replacing_published(tx, component_id, &version_note).await?;

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
    let publish_result = sqlx::query(
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
    if publish_result.rows_affected() != 1 {
        return Err(ApiError::BadRequest(format!(
            "component version {version_id} could not be published because it is no longer a draft"
        )));
    }
    Ok(())
}

async fn require_new_version_note_when_replacing_published(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    version_note: &str,
) -> ApiResult<()> {
    let replaces_published: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM component_versions
            WHERE component_id = $1
              AND status IN (
                  'published'::component_version_status,
                  'superseded'::component_version_status
              )
        )
        "#,
    )
    .bind(component_id)
    .fetch_one(&mut **tx)
    .await?;
    if replaces_published && version_note.trim().is_empty() {
        require_new_version_note(version_note)?;
    }
    Ok(())
}

fn require_new_version_note(version_note: &str) -> ApiResult<()> {
    if version_note.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "new component versions require a version note".into(),
        ));
    }
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

async fn run_component_visual_kind(
    state: AppState,
    headers: HeaderMap,
    component_ref: String,
    version_id: Option<Uuid>,
    visual_kind: &'static str,
) -> ApiResult<Json<ComponentVisual>> {
    let account = auth::require_capability(&state.pool, &headers, "components:read").await?;
    let version =
        load_component_version_for_table(&state.pool, &account, &component_ref, version_id).await?;
    execute_component_visual(&state.pool, &account, version, visual_kind, None)
        .await
        .map(Json)
}

pub async fn run_component_bar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, None, "bar").await
}

pub async fn run_component_version_bar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_ref, version_id)): Path<(String, Uuid)>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, Some(version_id), "bar").await
}

pub async fn run_component_line(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, None, "line").await
}

pub async fn run_component_version_line(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_ref, version_id)): Path<(String, Uuid)>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, Some(version_id), "line").await
}

pub async fn run_component_pie(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, None, "pie").await
}

pub async fn run_component_version_pie(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_ref, version_id)): Path<(String, Uuid)>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, Some(version_id), "pie").await
}

pub async fn run_component_donut(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, None, "donut").await
}

pub async fn run_component_version_donut(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_ref, version_id)): Path<(String, Uuid)>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, Some(version_id), "donut").await
}

pub async fn run_component_stat_card(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, None, "stat_card").await
}

pub async fn run_component_version_stat_card(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((component_ref, version_id)): Path<(String, Uuid)>,
) -> ApiResult<Json<ComponentVisual>> {
    run_component_visual_kind(state, headers, component_ref, Some(version_id), "stat_card").await
}

pub(crate) async fn render_version_for_account(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    version_id: Uuid,
    expected_kind: &'static str,
    raw_query: &str,
) -> ApiResult<serde_json::Value> {
    let component_ref: String = sqlx::query_scalar(
        "SELECT components.slug FROM component_versions
         JOIN components ON components.id=component_versions.component_id
         WHERE component_versions.id=$1",
    )
    .bind(version_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("component version not found".into()))?;
    let version =
        load_component_version_for_table(pool, account, &component_ref, Some(version_id)).await?;
    if version.component_type != expected_kind {
        return Err(ApiError::NotFound("component render kind not found".into()));
    }
    if expected_kind == "table" {
        let uri: axum::http::Uri = format!("/?{raw_query}")
            .parse()
            .map_err(|_| ApiError::BadRequest("invalid component query".into()))?;
        let query = Query::<ComponentTableQuery>::try_from_uri(&uri)
            .map_err(|_| ApiError::BadRequest("invalid component query".into()))?
            .0;
        let table =
            execute_component_table(pool, account, version, query.into_runtime_query()?).await?;
        serde_json::to_value(table).map_err(|error| ApiError::Internal(error.into()))
    } else {
        let visual = execute_component_visual(pool, account, version, expected_kind, None).await?;
        serde_json::to_value(visual).map_err(|error| ApiError::Internal(error.into()))
    }
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

struct MajorLineMaterialization {
    schema: String,
    table: String,
    state: String,
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
        "#,
    );
    if version_id.is_some() {
        query.push_str(
            r#"
          AND component_versions.id = $2
          AND component_versions.status IN (
              'published'::component_version_status,
              'superseded'::component_version_status
          )
        "#,
        );
    } else {
        query.push_str(" AND component_versions.status = 'published'::component_version_status");
    }
    query.push_str(" ORDER BY component_versions.version_number DESC LIMIT 1");

    let mut sql = sqlx::query(&query).bind(component_ref);
    if let Some(version_id) = version_id {
        sql = sql.bind(version_id);
    }
    let row = sql.fetch_optional(pool).await?.ok_or_else(|| {
        if let Some(version_id) = version_id {
            ApiError::NotFound(format!("published-history component version {version_id}"))
        } else {
            ApiError::NotFound(format!("published component {component_ref}"))
        }
    })?;
    let component_id = row.try_get("component_id")?;
    let dataset_id = row.try_get("dataset_id")?;
    if version_id.is_some() {
        require_dataset_visible_for_boundary(pool, dataset_id, &boundary, "components:read")
            .await?;
    } else {
        require_component_visible_for_boundary(pool, component_id, &boundary, "components:read")
            .await?;
    }
    Ok(ComponentVersionForTable {
        id: row.try_get("id")?,
        component_id,
        dataset_id,
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

async fn lock_component_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
) -> ApiResult<()> {
    let component_locked: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM components
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(component_id)
    .fetch_optional(&mut **tx)
    .await?;
    if component_locked.is_some() {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("component {component_id}")))
    }
}

async fn require_component_version_draft_row_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    version_id: Uuid,
) -> ApiResult<()> {
    require_component_version_status_row_in_tx(tx, component_id, version_id, "draft").await
}

async fn require_component_version_status_row_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    version_id: Uuid,
    expected_status: &str,
) -> ApiResult<()> {
    let status: String = sqlx::query_scalar(
        r#"
        SELECT status::text
        FROM component_versions
        WHERE component_id = $1
          AND id = $2
        FOR UPDATE
        "#,
    )
    .bind(component_id)
    .bind(version_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("component version {version_id}")))?;
    if status == expected_status {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "component version {version_id} has status '{status}', expected '{expected_status}'"
        )))
    }
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
        SELECT
            components.id,
            components.name,
            components.slug,
            components.description,
            current_versions.id AS current_version_id,
            current_versions.version_label AS current_version_label,
            current_versions.component_type::text AS current_component_type,
            draft_versions.id AS draft_version_id,
            draft_versions.version_label AS draft_version_label
        FROM components
        LEFT JOIN component_versions AS current_versions
            ON current_versions.component_id = components.id
           AND current_versions.status = 'published'::component_version_status
        LEFT JOIN component_versions AS draft_versions
            ON draft_versions.component_id = components.id
           AND draft_versions.status = 'draft'::component_version_status
        WHERE NOT EXISTS (
            SELECT 1
            FROM component_versions AS governed_versions
            JOIN dataset_scope_nodes
              ON dataset_scope_nodes.dataset_id = governed_versions.dataset_id
            WHERE governed_versions.component_id = components.id
              AND NOT (dataset_scope_nodes.node_id = ANY($1))
        )
        ORDER BY components.name, components.id
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
            current_versions.version_label AS current_version_label,
            current_versions.component_type::text AS current_component_type,
            NULL::uuid AS draft_version_id,
            NULL::text AS draft_version_label
        FROM components
        LEFT JOIN component_versions AS current_versions
            ON current_versions.component_id = components.id
           AND current_versions.status = 'published'::component_version_status
        WHERE EXISTS (
            SELECT 1
            FROM dataset_scope_nodes
            WHERE dataset_scope_nodes.dataset_id = current_versions.dataset_id
              AND dataset_scope_nodes.node_id = ANY($1)
        )
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
            current_versions.version_label AS current_version_label,
            current_versions.component_type::text AS current_component_type,
            CASE WHEN $1 THEN draft_versions.id ELSE NULL::uuid END AS draft_version_id,
            CASE WHEN $1 THEN draft_versions.version_label ELSE NULL::text END AS draft_version_label
        FROM components
        LEFT JOIN component_versions AS current_versions
            ON current_versions.component_id = components.id
           AND current_versions.status = 'published'::component_version_status
        LEFT JOIN component_versions AS draft_versions
            ON draft_versions.component_id = components.id
           AND draft_versions.status = 'draft'::component_version_status
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
                current_version_label: row.try_get("current_version_label")?,
                current_component_type: row.try_get("current_component_type")?,
                draft_version_id: row.try_get("draft_version_id")?,
                draft_version_label: row.try_get("draft_version_label")?,
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
    if capability == "components:manage" {
        require_component_fully_manageable(pool, account, component_id).await?;
    } else {
        require_component_visible_for_boundary(pool, component_id, &boundary, capability).await?;
    }
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
    } else if lower.contains("requires numeric summary field") {
        (
            "COMPONENT_SUMMARY_FIELD_TYPE_MISMATCH",
            "config.summary_field",
        )
    } else if lower.contains("summary field references field") {
        (
            "COMPONENT_SUMMARY_FIELD_NOT_IN_MAJOR_LINE",
            "config.summary_field",
        )
    } else if lower.contains("bar category field") || lower.contains("pie category field") {
        (
            "COMPONENT_CATEGORY_FIELD_NOT_IN_MAJOR_LINE",
            "config.category_field",
        )
    } else if lower.contains("bar comparison field") {
        (
            "COMPONENT_COMPARISON_FIELD_NOT_IN_MAJOR_LINE",
            "config.comparison_field",
        )
    } else if lower.contains("line x field") {
        ("COMPONENT_X_FIELD_NOT_IN_MAJOR_LINE", "config.x_field")
    } else if lower.contains("table visible column references field") {
        ("COMPONENT_FIELD_NOT_IN_MAJOR_LINE", "config")
    } else if lower.contains("component filter") && lower.contains("outside") {
        ("COMPONENT_FILTER_FIELD_NOT_IN_MAJOR_LINE", "config.filters")
    } else if lower.contains("filter operator") {
        ("COMPONENT_FILTER_OPERATOR_INVALID", "config.filters")
    } else if lower.contains("stacked bar comparison layout") {
        (
            "COMPONENT_COMPARISON_LAYOUT_INCOMPATIBLE",
            "config.comparison_layout",
        )
    } else if lower.contains("sort") {
        ("COMPONENT_SORT_INVALID", "config.sort_field")
    } else {
        ("COMPONENT_CONFIG_INVALID", "config")
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
        ApiError::BadRequest(message)
        | ApiError::NotFound(message)
        | ApiError::ServiceUnavailable(message) => message,
        ApiError::MixedCapabilityScopeModes => {
            "A role cannot mix scope-aware and installation-global capabilities.".into()
        }
        ApiError::GlobalCapabilityRequiresGlobalRoleAssignment => {
            "Installation-global capabilities require a global role assignment.".into()
        }
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
               component_versions.binding_mode,
               component_versions.component_type::text AS component_type,
               component_versions.status::text AS status, component_versions.version_label,
               component_versions.version_note, component_versions.config
        FROM component_versions
        WHERE component_id = $1
          AND ($3 OR component_versions.status = 'published'::component_version_status)
          AND EXISTS (
              SELECT 1
              FROM dataset_scope_nodes
              WHERE dataset_scope_nodes.dataset_id = component_versions.dataset_id
                AND dataset_scope_nodes.node_id = ANY($2)
          )
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
        SELECT id, component_id, dataset_id, dataset_version_major, binding_mode, component_type::text AS component_type,
               status::text AS status, version_label, version_note, config
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
                component_type: row.try_get("component_type")?,
                status: row.try_get("status")?,
                version_label: row.try_get("version_label")?,
                version_note: row.try_get("version_note")?,
                config: row.try_get("config")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

fn validate_component_type(component_type: &str) -> ApiResult<()> {
    match component_type {
        "table" | "bar" | "line" | "pie" | "donut" | "stat_card" => Ok(()),
        other => Err(ApiError::BadRequest(format!(
            "unsupported component type '{other}'"
        ))),
    }
}

fn require_component_kind(component_type: &str, expected: &str, label: &str) -> ApiResult<()> {
    if component_type == expected {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} expected component type '{expected}' but found '{component_type}'"
        )))
    }
}

fn validate_component_version_note(note: Option<&str>) -> ApiResult<()> {
    if note.map(str::trim).unwrap_or_default().len() > 2_000 {
        return Err(ApiError::BadRequest(
            "component version note must be 2000 characters or fewer".into(),
        ));
    }
    Ok(())
}

fn normalized_component_version_note(note: Option<&str>) -> ApiResult<Option<String>> {
    validate_component_version_note(note)?;
    Ok(note.map(|value| value.trim().to_string()))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TableComponentConfig {
    #[serde(default)]
    visible_columns: Vec<ComponentFieldRef>,
    #[serde(default)]
    filters: Vec<ComponentFilterConfig>,
    #[serde(default)]
    search_fields: Vec<String>,
    #[serde(default)]
    default_sort: Option<ComponentSortConfig>,
    #[serde(default)]
    page_size: Option<usize>,
    #[serde(default)]
    display_labels: BTreeMap<String, String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentFilterConfig {
    field_key: String,
    operator: String,
    #[serde(default)]
    value: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct VisualSharedConfig {
    summary_field: String,
    summary_type: String,
    #[serde(default = "default_value_format")]
    value_format: String,
    #[serde(default = "default_missing_policy")]
    missing_policy: String,
    #[serde(default)]
    value_missing_policy: Option<String>,
    #[serde(default)]
    sort_field: Option<String>,
    #[serde(default = "default_sort_direction")]
    sort_direction: String,
    #[serde(default)]
    filters: Vec<ComponentFilterConfig>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct StatCardComponentConfig {
    #[serde(flatten)]
    shared: VisualSharedConfig,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    supporting_text: Option<String>,
    #[serde(default = "default_panel_style")]
    panel_style: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BarComponentConfig {
    #[serde(flatten)]
    shared: VisualSharedConfig,
    mode: String,
    category_field: String,
    #[serde(default)]
    category_missing_policy: Option<String>,
    #[serde(default)]
    comparison_field: Option<String>,
    #[serde(default)]
    comparison_missing_policy: Option<String>,
    #[serde(default = "default_bar_orientation")]
    orientation: String,
    #[serde(default = "default_bar_comparison_layout")]
    comparison_layout: String,
    #[serde(default = "default_visual_limit")]
    number_of_points: usize,
    #[serde(default)]
    category_labels: BTreeMap<String, String>,
    #[serde(default)]
    category_colors: BTreeMap<String, String>,
    #[serde(default)]
    legend_title: Option<String>,
    #[serde(default)]
    x_axis_label: Option<String>,
    #[serde(default)]
    y_axis_label: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LineComponentConfig {
    #[serde(flatten)]
    shared: VisualSharedConfig,
    x_field: String,
    #[serde(default)]
    x_missing_policy: Option<String>,
    #[serde(default = "default_line_smoothing")]
    smoothing: bool,
    #[serde(default = "default_visual_limit")]
    number_of_points: usize,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PieComponentConfig {
    #[serde(flatten)]
    shared: VisualSharedConfig,
    category_field: String,
    #[serde(default)]
    category_missing_policy: Option<String>,
    #[serde(default = "default_visual_limit")]
    max_slices: usize,
    #[serde(default)]
    category_labels: BTreeMap<String, String>,
    #[serde(default)]
    category_colors: BTreeMap<String, String>,
    #[serde(default)]
    legend_title: Option<String>,
}

enum VisualComponentConfig {
    StatCard(StatCardComponentConfig),
    Bar(BarComponentConfig),
    Line(LineComponentConfig),
    Pie(PieComponentConfig),
    Donut(PieComponentConfig),
}

impl VisualComponentConfig {
    fn parse(component_type: &str, config: &serde_json::Value) -> ApiResult<Self> {
        match component_type {
            "stat_card" => Ok(Self::StatCard(
                serde_json::from_value(config.clone()).map_err(|error| {
                    ApiError::BadRequest(format!("stat card component config is invalid: {error}"))
                })?,
            )),
            "bar" => Ok(Self::Bar(serde_json::from_value(config.clone()).map_err(
                |error| ApiError::BadRequest(format!("bar component config is invalid: {error}")),
            )?)),
            "line" => Ok(Self::Line(serde_json::from_value(config.clone()).map_err(
                |error| ApiError::BadRequest(format!("line component config is invalid: {error}")),
            )?)),
            "pie" => Ok(Self::Pie(serde_json::from_value(config.clone()).map_err(
                |error| ApiError::BadRequest(format!("pie component config is invalid: {error}")),
            )?)),
            "donut" => Ok(Self::Donut(
                serde_json::from_value(config.clone()).map_err(|error| {
                    ApiError::BadRequest(format!("donut component config is invalid: {error}"))
                })?,
            )),
            _ => Err(ApiError::BadRequest(format!(
                "unsupported component type '{component_type}'"
            ))),
        }
    }

    fn shared(&self) -> &VisualSharedConfig {
        match self {
            Self::StatCard(config) => &config.shared,
            Self::Bar(config) => &config.shared,
            Self::Line(config) => &config.shared,
            Self::Pie(config) | Self::Donut(config) => &config.shared,
        }
    }

    fn value_format(&self) -> String {
        self.shared().value_format.clone()
    }

    fn referenced_fields(&self) -> Vec<String> {
        let mut fields = Vec::new();
        if self.shared().summary_type != "row_count" {
            fields.push(self.shared().summary_field.clone());
        }
        fields.extend(
            self.shared()
                .filters
                .iter()
                .map(|filter| filter.field_key.clone()),
        );
        match self {
            Self::StatCard(_) => {}
            Self::Bar(config) => {
                fields.push(config.category_field.clone());
                fields.extend(config.comparison_field.clone());
            }
            Self::Line(config) => fields.push(config.x_field.clone()),
            Self::Pie(config) | Self::Donut(config) => fields.push(config.category_field.clone()),
        }
        fields
    }
}

fn default_value_format() -> String {
    "plain".into()
}

fn default_missing_policy() -> String {
    "omit".into()
}

fn default_panel_style() -> String {
    "default".into()
}

fn default_bar_orientation() -> String {
    "horizontal".into()
}

fn default_bar_comparison_layout() -> String {
    "grouped".into()
}

fn default_visual_limit() -> usize {
    20
}

fn default_line_smoothing() -> bool {
    true
}

fn validate_component_config(
    component_type: &str,
    config: &serde_json::Value,
    fields: &[DataField],
) -> ApiResult<()> {
    match component_type {
        "table" => validate_table_component_config(config, fields),
        "bar" | "line" | "pie" | "donut" | "stat_card" => {
            validate_visual_component_config(component_type, config, fields)
        }
        _ => validate_component_type(component_type),
    }
}

fn validate_visual_component_config(
    component_type: &str,
    config: &serde_json::Value,
    fields: &[DataField],
) -> ApiResult<()> {
    let config = VisualComponentConfig::parse(component_type, config)?;
    validate_visual_shared_config(config.shared(), fields)?;
    match &config {
        VisualComponentConfig::StatCard(config) => {
            validate_enum(
                &config.panel_style,
                &["default", "muted", "accent"],
                "stat card panel style",
            )?;
            if config.shared.sort_field.is_some() {
                return Err(ApiError::BadRequest(
                    "stat card config does not support sort_field".into(),
                ));
            }
        }
        VisualComponentConfig::Bar(config) => {
            require_component_field(fields, &config.category_field, "bar category field")?;
            validate_optional_missing_policy(
                config.category_missing_policy.as_deref(),
                &["omit", "explicit_missing"],
                "bar category missing policy",
            )?;
            validate_optional_missing_policy(
                config.comparison_missing_policy.as_deref(),
                &["omit", "explicit_missing"],
                "bar comparison missing policy",
            )?;
            validate_enum(&config.mode, &["summary", "comparison"], "bar mode")?;
            validate_enum(
                &config.orientation,
                &["vertical", "horizontal"],
                "bar orientation",
            )?;
            validate_enum(
                &config.comparison_layout,
                &["grouped", "stacked"],
                "bar comparison layout",
            )?;
            if config.mode == "comparison"
                && config.comparison_layout == "stacked"
                && !matches!(
                    config.shared.summary_type.as_str(),
                    "row_count" | "count" | "sum"
                )
            {
                return Err(ApiError::BadRequest(format!(
                    "stacked bar comparison layout requires row_count, count, or sum calculation; found '{}'",
                    config.shared.summary_type
                )));
            }
            validate_visual_limit(config.number_of_points, "bar number_of_points")?;
            match config.mode.as_str() {
                "summary" if config.comparison_field.is_some() => {
                    return Err(ApiError::BadRequest(
                        "bar summary config must not include comparison_field".into(),
                    ));
                }
                "comparison" => {
                    let comparison_field = config.comparison_field.as_deref().ok_or_else(|| {
                        ApiError::BadRequest(
                            "bar comparison config requires comparison_field".into(),
                        )
                    })?;
                    require_component_field(fields, comparison_field, "bar comparison field")?;
                }
                _ => {}
            }
            validate_visual_sort_field(
                config.shared.sort_field.as_deref(),
                if config.mode == "comparison" {
                    &["category", "comparison", "summary_value"]
                } else {
                    &["category", "summary_value"]
                },
                "bar sort field",
            )?;
        }
        VisualComponentConfig::Line(config) => {
            require_component_field(fields, &config.x_field, "line x field")?;
            validate_optional_missing_policy(
                config.x_missing_policy.as_deref(),
                &["omit", "explicit_missing"],
                "line x missing policy",
            )?;
            validate_visual_limit(config.number_of_points, "line number_of_points")?;
            validate_visual_sort_field(
                config.shared.sort_field.as_deref(),
                &["x", "summary_value"],
                "line sort field",
            )?;
        }
        VisualComponentConfig::Pie(config) | VisualComponentConfig::Donut(config) => {
            require_component_field(fields, &config.category_field, "pie category field")?;
            validate_optional_missing_policy(
                config.category_missing_policy.as_deref(),
                &["omit", "explicit_missing"],
                "pie category missing policy",
            )?;
            validate_visual_limit(config.max_slices, "pie max_slices")?;
            validate_visual_sort_field(
                config.shared.sort_field.as_deref(),
                &["category", "summary_value"],
                "pie sort field",
            )?;
        }
    }
    Ok(())
}

fn validate_visual_shared_config(
    config: &VisualSharedConfig,
    fields: &[DataField],
) -> ApiResult<()> {
    validate_enum(
        &config.summary_type,
        &[
            "row_count",
            "count",
            "unique_count",
            "sum",
            "average",
            "median",
            "none",
        ],
        "summary type",
    )?;
    let summary_field = if config.summary_type == "row_count" {
        None
    } else {
        Some(require_component_field(
            fields,
            &config.summary_field,
            "summary field",
        )?)
    };
    validate_enum(
        &config.value_format,
        &["plain", "integer", "decimal", "percent"],
        "value format",
    )?;
    validate_enum(
        &config.missing_policy,
        &["omit", "zero", "explicit_missing"],
        "missing policy",
    )?;
    validate_optional_missing_policy(
        config.value_missing_policy.as_deref(),
        &["omit", "zero", "explicit_missing"],
        "value missing policy",
    )?;
    validate_enum(&config.sort_direction, &["asc", "desc"], "sort direction")?;
    if matches!(
        config.summary_type.as_str(),
        "sum" | "average" | "median" | "none"
    ) && summary_field.is_some_and(|field| field.field_type != FieldType::Number)
    {
        return Err(ApiError::BadRequest(format!(
            "summary type '{}' requires numeric summary field '{}'",
            config.summary_type, config.summary_field
        )));
    }
    let field_refs = fields.iter().collect::<Vec<_>>();
    component_filter_sql(&config.filters, &field_refs)?;
    Ok(())
}

fn validate_optional_missing_policy(
    value: Option<&str>,
    allowed: &[&str],
    label: &str,
) -> ApiResult<()> {
    if let Some(value) = value {
        validate_enum(value, allowed, label)?;
    }
    Ok(())
}

fn validate_visual_sort_field(
    sort_field: Option<&str>,
    allowed: &[&str],
    label: &str,
) -> ApiResult<()> {
    if let Some(sort_field) = sort_field {
        validate_enum(sort_field, allowed, label)?;
    }
    Ok(())
}

fn validate_visual_limit(value: usize, label: &str) -> ApiResult<()> {
    if (1..=100).contains(&value) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} must be between 1 and 100"
        )))
    }
}

fn validate_enum(value: &str, allowed: &[&str], label: &str) -> ApiResult<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} has unsupported value '{value}'"
        )))
    }
}

fn validate_table_component_config(
    config: &serde_json::Value,
    fields: &[DataField],
) -> ApiResult<()> {
    let config: TableComponentConfig = serde_json::from_value(config.clone()).map_err(|error| {
        ApiError::BadRequest(format!("table component config is invalid: {error}"))
    })?;
    for column in &config.visible_columns {
        require_component_field(fields, column.field_key(), "table visible column")?;
    }
    let field_refs = fields.iter().collect::<Vec<_>>();
    let configured_visible_columns = config
        .visible_columns
        .iter()
        .map(|column| column.field_key().to_string())
        .collect::<Vec<_>>();
    let component_contract_fields = visible_table_fields(&field_refs, &configured_visible_columns)?;
    let component_filter_fields = fields.iter().collect::<Vec<_>>();
    component_filter_sql(&config.filters, &component_filter_fields)?;
    for field_key in &config.search_fields {
        require_component_field_ref(&component_contract_fields, field_key, "table search field")?;
    }
    for field_key in config.display_labels.keys() {
        require_component_field_ref(&component_contract_fields, field_key, "table display label")?;
    }
    validate_component_sort(
        &config.default_sort,
        &component_contract_fields,
        "table sort",
    )
}

fn validate_component_sort(
    sort: &Option<ComponentSortConfig>,
    fields: &[&DataField],
    label: &str,
) -> ApiResult<()> {
    if let Some(sort) = sort {
        require_component_field_ref(fields, &sort.field_key, label)?;
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

fn require_component_field_ref<'a>(
    fields: &[&'a DataField],
    field_key: &str,
    label: &str,
) -> ApiResult<&'a DataField> {
    fields
        .iter()
        .copied()
        .find(|field| field.key == field_key)
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "{label} references field '{field_key}' outside the component table contract"
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
    _pool: &sqlx::PgPool,
    payload: &CreateComponentVersionRequest,
) -> ApiResult<ComponentDatasetBinding> {
    match (payload.dataset_id, payload.dataset_version_major) {
        (Some(dataset_id), Some(dataset_version_major)) => Ok(ComponentDatasetBinding {
            dataset_id,
            dataset_version_major,
        }),
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

async fn require_component_fully_manageable(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    component_id: Uuid,
) -> ApiResult<()> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT dataset_scope_nodes.node_id
        FROM component_versions
        JOIN dataset_scope_nodes
          ON dataset_scope_nodes.dataset_id = component_versions.dataset_id
        WHERE component_versions.component_id = $1
        "#,
    )
    .bind(component_id)
    .fetch_all(pool)
    .await?;
    let node_ids = rows
        .into_iter()
        .map(|row| row.try_get("node_id"))
        .collect::<Result<Vec<Uuid>, sqlx::Error>>()?;
    if node_ids.is_empty() {
        require_component_exists(pool, component_id).await
    } else {
        auth::require_capability_contains_nodes(pool, account, "components:manage", &node_ids).await
    }
}

async fn require_component_fully_manageable_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    component_id: Uuid,
) -> ApiResult<()> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT dataset_scope_nodes.node_id
        FROM component_versions
        JOIN dataset_scope_nodes
          ON dataset_scope_nodes.dataset_id = component_versions.dataset_id
        WHERE component_versions.component_id = $1
        "#,
    )
    .bind(component_id)
    .fetch_all(&mut **tx)
    .await?;
    let node_ids = rows
        .into_iter()
        .map(|row| row.try_get("node_id"))
        .collect::<Result<Vec<Uuid>, sqlx::Error>>()?;
    if node_ids.is_empty() {
        Ok(())
    } else {
        auth::require_capability_contains_nodes(pool, account, "components:manage", &node_ids).await
    }
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
            let visible = sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM dataset_scope_nodes
                    WHERE dataset_id = $1
                      AND node_id = ANY($2)
                )
                "#,
            )
            .bind(dataset_id)
            .bind(scope_ids)
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
mod tests;
