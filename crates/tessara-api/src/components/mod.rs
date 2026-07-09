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
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use serde::Deserialize;
use sqlx::{Column, Postgres, Row, Transaction};
use tessara_data_ops::{DataField, FieldType, FilterOperator};
use uuid::Uuid;

mod dto;

pub use dto::{
    ComponentDefinition, ComponentSummary, ComponentTable, ComponentTableColumn,
    ComponentTablePagination, ComponentTableRow, ComponentValidationFinding,
    ComponentValidationResponse, ComponentVersionSummary, CreateComponentRequest,
    CreateComponentVersionRequest, SaveComponentEditAction, SaveComponentEditRequest,
    UpdateComponentRequest,
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

    let mut tx = state.pool.begin().await?;
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
                let version_id = payload.published_version_id.ok_or_else(|| {
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
    lock_component_in_tx(&mut tx, component_id).await?;
    require_component_fully_manageable_in_tx(&mut tx, &state.pool, &account, component_id).await?;
    require_component_version_status_row_in_tx(&mut tx, component_id, version_id, "published")
        .await?;
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
          AND status = 'published'::component_version_status
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
            "component version {version_id} could not be updated because it is no longer published"
        )));
    }

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
    execute_table_component(pool, account, version, materialization, &fields, query).await
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

async fn execute_table_component(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    version: ComponentVersionForTable,
    materialization: MajorLineMaterialization,
    fields: &[DataField],
    query: ComponentRuntimeQuery,
) -> ApiResult<ComponentTable> {
    validate_component_type(&version.component_type)?;
    let config: TableComponentConfig =
        serde_json::from_value(version.config.clone()).map_err(|error| {
            ApiError::BadRequest(format!("table component config is invalid: {error}"))
        })?;
    let field_refs = fields.iter().collect::<Vec<_>>();
    let configured_visible_columns = config
        .visible_columns
        .iter()
        .map(|column| column.field_key().to_string())
        .collect::<Vec<_>>();
    let component_contract_fields = visible_table_fields(&field_refs, &configured_visible_columns)?;
    let component_contract_refs = component_contract_fields.to_vec();
    let selected_fields = visible_table_fields(&component_contract_refs, &query.visible_columns)?;
    let columns = selected_fields
        .iter()
        .map(|field| component_table_column(field, &config.display_labels))
        .collect::<Vec<_>>();
    let select_columns = selected_fields
        .iter()
        .map(|field| quote_identifier(&field.key))
        .collect::<Vec<_>>();
    let mut predicates =
        vec![tier_access_predicate_for_materialization(pool, account, &materialization).await?];
    predicates.extend(component_filter_sql(&config.filters, &field_refs)?);
    predicates.extend(component_filter_sql(
        &query.filters,
        &component_contract_fields,
    )?);
    if let Some(search) = query.search.as_deref() {
        let search_fields = table_search_fields(&config, &component_contract_fields)?;
        if !search_fields.is_empty() {
            predicates.push(search_predicate_sql(&search_fields, search));
        }
    }
    let full_name = materialized_full_name(&materialization);
    let sort = query.sort.or(config.default_sort);
    let order_by = table_order_by_sql(sort.as_ref(), &component_contract_refs, "__row_id")?;
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

fn component_table_column(
    field: &DataField,
    display_labels: &BTreeMap<String, String>,
) -> ComponentTableColumn {
    ComponentTableColumn {
        key: field.key.clone(),
        label: display_labels
            .get(&field.key)
            .cloned()
            .unwrap_or_else(|| field.label.clone()),
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

fn table_search_fields<'a>(
    config: &TableComponentConfig,
    fields: &[&'a DataField],
) -> ApiResult<Vec<&'a DataField>> {
    // Blank search fields means search every projected component field using
    // text coercion, matching the shared interactive table viewer behavior.
    if config.search_fields.is_empty() {
        return Ok(fields.to_vec());
    }
    config
        .search_fields
        .iter()
        .map(|key| require_component_field_ref(fields, key, "table search field"))
        .collect()
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

fn filter_to_sql(
    filter: &ComponentFilterConfig,
    fields: &[&DataField],
    label: &str,
) -> ApiResult<String> {
    let field = require_component_field_ref(fields, &filter.field_key, label)?;
    let operator = FilterOperator::parse(&filter.operator).map_err(component_data_op_error)?;
    operator
        .validate_for_field(field)
        .map_err(component_data_op_error)?;
    validate_component_filter_value(field, operator, filter.value.as_deref(), label)?;
    Ok(filter_predicate_sql(
        field,
        operator,
        filter.value.as_deref(),
    ))
}

fn validate_component_filter_value(
    field: &DataField,
    operator: FilterOperator,
    value: Option<&str>,
    label: &str,
) -> ApiResult<()> {
    if !operator.requires_value() {
        return Ok(());
    }
    if matches!(
        operator,
        FilterOperator::Between | FilterOperator::NotBetween
    ) {
        let value = required_filter_value(field, value, operator, label)?;
        let (lower, upper) = parse_filter_range(value).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "{label} for field '{}' requires two values for operator '{}'",
                field.key,
                operator.as_str()
            ))
        })?;
        validate_filter_scalar(field, lower, operator, label)?;
        validate_filter_scalar(field, upper, operator, label)?;
        return Ok(());
    }
    let value = required_filter_value(field, value, operator, label)?;
    validate_filter_scalar(field, value, operator, label)
}

fn required_filter_value<'a>(
    field: &DataField,
    value: Option<&'a str>,
    operator: FilterOperator,
    label: &str,
) -> ApiResult<&'a str> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "{label} for field '{}' requires a value for operator '{}'",
                field.key,
                operator.as_str()
            ))
        })
}

fn parse_filter_range(value: &str) -> Option<(&str, &str)> {
    value
        .split_once("..")
        .or_else(|| value.split_once(','))
        .and_then(|(lower, upper)| {
            let lower = lower.trim();
            let upper = upper.trim();
            if lower.is_empty() || upper.is_empty() {
                None
            } else {
                Some((lower, upper))
            }
        })
}

fn validate_filter_scalar(
    field: &DataField,
    value: &str,
    operator: FilterOperator,
    label: &str,
) -> ApiResult<()> {
    let valid = match &field.field_type {
        FieldType::Number => value.parse::<f64>().is_ok(),
        FieldType::Boolean => matches!(
            value.to_ascii_lowercase().as_str(),
            "true" | "t" | "1" | "yes" | "y" | "false" | "f" | "0" | "no" | "n"
        ),
        FieldType::Date => NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok(),
        FieldType::DateTime | FieldType::Timestamp => parse_filter_timestamp(value),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} for field '{}' has invalid value '{}' for operator '{}' and type '{}'",
            field.key,
            value,
            operator.as_str(),
            field.field_type.as_str()
        )))
    }
}

fn parse_filter_timestamp(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value).is_ok()
        || DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f%#z").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f").is_ok()
        || NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
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

fn component_filter_sql(
    filters: &[ComponentFilterConfig],
    fields: &[&DataField],
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
    } else if lower.contains("filter operator") {
        ("COMPONENT_FILTER_FIELD_NOT_IN_MAJOR_LINE", "config.filters")
    } else if lower.contains("sort") {
        (
            "COMPONENT_SORT_FIELD_NOT_IN_MAJOR_LINE",
            "config.default_sort",
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
        "table" => Ok(()),
        other => Err(ApiError::BadRequest(format!(
            "unsupported component type '{other}'"
        ))),
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentFilterConfig {
    field_key: String,
    operator: String,
    #[serde(default)]
    value: Option<String>,
}

fn validate_component_config(
    component_type: &str,
    config: &serde_json::Value,
    fields: &[DataField],
) -> ApiResult<()> {
    match component_type {
        "table" => validate_table_component_config(config, fields),
        _ => validate_component_type(component_type),
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
mod tests {
    use serde_json::json;
    use std::collections::{BTreeMap, HashMap};
    use tessara_data_ops::{DataField, FieldType};

    use uuid::Uuid;

    use super::{
        ComponentSummary, ComponentTableQuery, ComponentVersionForTable, CreateComponentRequest,
        CreateComponentVersionRequest, UpdateComponentRequest, component_filter_sql,
        component_pagination_sql, effective_component_page_size, parse_component_query_filters,
        parse_component_sort, require_component_version_draft, table_order_by_sql,
        table_search_fields, validate_component_config, visible_table_fields,
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
    fn reader_component_summary_omits_absent_draft_metadata() {
        let summary = ComponentSummary {
            id: Uuid::nil(),
            name: "Published Table".into(),
            slug: "published_table".into(),
            description: None,
            current_version_id: Some(Uuid::nil()),
            current_version_label: Some("1".into()),
            current_component_type: Some("table".into()),
            draft_version_id: None,
            draft_version_label: None,
        };

        let value = serde_json::to_value(summary).expect("summary should serialize");

        assert!(value.get("draft_version_id").is_none());
        assert!(value.get("draft_version_label").is_none());
    }

    #[test]
    fn table_config_validates_presentation_fields() {
        let fields = vec![
            field("program", FieldType::Text),
            field("amount", FieldType::Number),
        ];
        let config = json!({
            "visible_columns": ["program", "amount"],
            "filters": [
                {
                    "field_key": "program",
                    "operator": "not_contains",
                    "value": "archived"
                }
            ],
            "search_fields": ["program"],
            "default_sort": {
                "field_key": "amount",
                "direction": "desc"
            },
            "page_size": 25,
            "display_labels": {
                "amount": "Award Amount"
            }
        });

        validate_component_config("table", &config, &fields)
            .expect("valid table component config should pass");
    }

    #[test]
    fn table_config_rejects_stale_analytical_keys() {
        let fields = vec![field("program", FieldType::Text)];
        let config = json!({
            "visible_columns": ["program"],
            "metrics": [
                {
                    "function": "count",
                    "field_key": "program"
                }
            ]
        });

        let error = validate_component_config("table", &config, &fields)
            .expect_err("component analytical keys should fail");
        assert!(error.to_string().contains("unknown field `metrics`"));
    }

    #[test]
    fn table_config_validates_saved_filters() {
        let fields = vec![field("score", FieldType::Number)];
        let config = json!({
            "visible_columns": ["score"],
            "filters": [
                {
                    "field_key": "score",
                    "operator": "contains",
                    "value": "10"
                }
            ]
        });

        let error = validate_component_config("table", &config, &fields)
            .expect_err("invalid saved filter should fail");
        assert!(
            error
                .to_string()
                .contains("filter operator 'contains' is not supported")
        );
    }

    #[test]
    fn table_config_rejects_invalid_numeric_saved_filter_value() {
        let fields = vec![field("score", FieldType::Number)];
        let config = json!({
            "visible_columns": ["score"],
            "filters": [
                {
                    "field_key": "score",
                    "operator": "equals",
                    "value": "not-a-number"
                }
            ]
        });

        let error = validate_component_config("table", &config, &fields)
            .expect_err("invalid numeric saved filter should fail");
        assert!(error.to_string().contains("invalid value 'not-a-number'"));
    }

    #[test]
    fn table_config_rejects_invalid_date_saved_filter_range() {
        let fields = [field("submitted_on", FieldType::Date)];
        let config = json!({
            "visible_columns": ["submitted_on"],
            "filters": [
                {
                    "field_key": "submitted_on",
                    "operator": "between",
                    "value": "2026-01-01..soon"
                }
            ]
        });

        let error = validate_component_config("table", &config, &fields)
            .expect_err("invalid date saved filter range should fail");
        assert!(error.to_string().contains("invalid value 'soon'"));
    }

    #[test]
    fn component_filter_sql_rejects_invalid_runtime_filter_value() {
        let fields = [field("submitted_on", FieldType::Date)];
        let filters = vec![super::ComponentFilterConfig {
            field_key: "submitted_on".into(),
            operator: "gte".into(),
            value: Some("not-a-date".into()),
        }];
        let refs = fields.iter().collect::<Vec<_>>();

        let error = component_filter_sql(&filters, &refs)
            .expect_err("invalid runtime filter literal should fail");
        assert!(error.to_string().contains("invalid value 'not-a-date'"));
    }

    #[test]
    fn table_config_rejects_field_mode_component_filters() {
        let fields = [field("program", FieldType::Text)];
        let config = json!({
            "visible_columns": ["program"],
            "filters": [
                {
                    "field_key": "program",
                    "operator": "equals",
                    "value_field_key": "other_program"
                }
            ]
        });

        let error = validate_component_config("table", &config, &fields)
            .expect_err("field-mode component filter should fail");
        assert!(
            error
                .to_string()
                .contains("unknown field `value_field_key`")
        );
    }

    #[test]
    fn table_config_rejects_missing_visible_column() {
        let fields = [field("program", FieldType::Text)];
        let config = json!({
            "visible_columns": ["program", "amount"]
        });

        let error = validate_component_config("table", &config, &fields)
            .expect_err("unknown visible column should fail");
        assert!(
            error
                .to_string()
                .contains("table visible column references field 'amount'")
        );
    }

    #[test]
    fn old_component_table_kinds_are_rejected() {
        let fields = [field("program", FieldType::Text)];
        let config = json!({ "visible_columns": ["program"] });

        let detail_error = validate_component_config("detail_table", &config, &fields)
            .expect_err("old detail kind should fail");
        let aggregate_error = validate_component_config("aggregate_table", &config, &fields)
            .expect_err("old aggregate kind should fail");

        assert!(
            detail_error
                .to_string()
                .contains("unsupported component type")
        );
        assert!(
            aggregate_error
                .to_string()
                .contains("unsupported component type")
        );
    }

    #[test]
    fn component_filter_sql_supports_negative_operator() {
        let fields = [field("program", FieldType::Text)];
        let filters = vec![super::ComponentFilterConfig {
            field_key: "program".into(),
            operator: "not_contains".into(),
            value: Some("archived".into()),
        }];
        let refs = fields.iter().collect::<Vec<_>>();

        let sql = component_filter_sql(&filters, &refs).expect("filter should compile");
        assert_eq!(
            sql,
            vec!["POSITION(LOWER('archived') IN LOWER(COALESCE(\"program\", ''))) = 0"]
        );
    }

    #[test]
    fn component_filter_sql_validates_operator_field_compatibility() {
        let fields = [field("score", FieldType::Number)];
        let filters = vec![super::ComponentFilterConfig {
            field_key: "score".into(),
            operator: "contains".into(),
            value: Some("10".into()),
        }];
        let refs = fields.iter().collect::<Vec<_>>();

        let error = component_filter_sql(&filters, &refs)
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
        let fields = [
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
        let fields = [
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
    fn table_search_defaults_to_component_projection_contract() {
        let config = super::TableComponentConfig {
            visible_columns: vec![super::ComponentFieldRef::Key("score".into())],
            filters: Vec::new(),
            search_fields: Vec::new(),
            default_sort: None,
            page_size: None,
            display_labels: BTreeMap::new(),
        };
        let fields = [
            field("program", FieldType::Text),
            field("score", FieldType::Number),
        ];

        let refs = fields.iter().collect::<Vec<_>>();
        let selected = visible_table_fields(&refs, &["score".into()])
            .expect("visible column projection should pass");

        let search_fields = table_search_fields(&config, &selected).expect("search fields");

        assert_eq!(
            selected
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["score"]
        );
        assert_eq!(
            search_fields
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["score"]
        );
    }

    #[test]
    fn runtime_visible_columns_can_only_narrow_component_projection() {
        let fields = [
            field("program", FieldType::Text),
            field("score", FieldType::Number),
            field("hidden", FieldType::Text),
        ];
        let refs = fields.iter().collect::<Vec<_>>();
        let component_contract = visible_table_fields(&refs, &["program".into(), "score".into()])
            .expect("configured projection should pass");

        let selected = visible_table_fields(&component_contract, &["score".into()])
            .expect("query projection can narrow component projection");
        assert_eq!(
            selected
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["score"]
        );

        let error = visible_table_fields(&component_contract, &["hidden".into()])
            .expect_err("query projection cannot expand component projection");
        assert!(
            error
                .to_string()
                .contains("visible column 'hidden' is outside")
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
    fn new_component_versions_require_notes() {
        assert!(super::require_new_version_note("changed displayed fields").is_ok());
        let error =
            super::require_new_version_note("   ").expect_err("blank new-version note should fail");
        assert!(error.to_string().contains("require a version note"));
    }

    #[test]
    fn component_table_without_materialization_uses_pending_state() {
        let version = ComponentVersionForTable {
            id: Uuid::new_v4(),
            component_id: Uuid::new_v4(),
            dataset_id: Uuid::new_v4(),
            dataset_version_major: 1,
            component_type: "table".into(),
            config: json!({ "visible_columns": ["program"] }),
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
            component_type: "table".into(),
            config: json!({ "visible_columns": ["program"] }),
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
                "component_type": "table",
                "config": {
                    "visible_columns": ["program"]
                }
            }
        }))
        .expect("atomic create payload should deserialize");

        assert_eq!(payload.name, "Program table");
        let version = payload.version.expect("version should be present");
        assert_eq!(version.dataset_id, Some(dataset_id));
        assert_eq!(version.dataset_version_major, Some(1));
        assert_eq!(version.component_type, "table");
    }

    #[test]
    fn component_shell_payloads_reject_unknown_fields() {
        let create_error = match serde_json::from_value::<CreateComponentRequest>(json!({
            "name": "Program table",
            "slug": "program-table",
            "description": "A first table component",
            "dataset_revision_id": Uuid::nil()
        })) {
            Ok(_) => panic!("create component shell should reject legacy revision fields"),
            Err(error) => error,
        };
        assert!(create_error.to_string().contains("dataset_revision_id"));

        let update_error = match serde_json::from_value::<UpdateComponentRequest>(json!({
            "name": "Program table",
            "slug": "program-table",
            "description": "Updated table component",
            "dataset_revision_id": Uuid::nil()
        })) {
            Ok(_) => panic!("update component shell should reject legacy revision fields"),
            Err(error) => error,
        };
        assert!(update_error.to_string().contains("dataset_revision_id"));
    }

    #[test]
    fn atomic_component_version_payload_rejects_legacy_revision_binding() {
        let error = match serde_json::from_value::<CreateComponentRequest>(json!({
            "name": "Program table",
            "slug": "program-table",
            "description": "A first table component",
            "version": {
                "dataset_id": Uuid::nil(),
                "dataset_version_major": 1,
                "dataset_revision_id": Uuid::nil(),
                "component_type": "table",
                "config": {
                    "visible_columns": ["program"]
                }
            }
        })) {
            Ok(_) => panic!("atomic create version should reject legacy revision fields"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("dataset_revision_id"));
    }

    #[test]
    fn component_version_payload_rejects_legacy_revision_binding() {
        let error = match serde_json::from_value::<CreateComponentVersionRequest>(json!({
            "dataset_revision_id": Uuid::nil(),
            "component_type": "table",
            "config": {
                "visible_columns": ["program"]
            }
        })) {
            Ok(_) => panic!("legacy revision-bound payload should be rejected"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("dataset_revision_id"));
    }

    #[test]
    fn component_version_payload_rejects_inline_publish_flag() {
        let error = match serde_json::from_value::<CreateComponentVersionRequest>(json!({
            "dataset_id": Uuid::nil(),
            "dataset_version_major": 1,
            "component_type": "table",
            "config": {
                "visible_columns": ["program"]
            },
            "publish": true
        })) {
            Ok(_) => panic!("inline publish flag should be rejected"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("publish"));
    }
}
