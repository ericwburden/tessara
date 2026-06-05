use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, require_text},
};

#[derive(Deserialize)]
pub struct CreateDashboardRequest {
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct AddDashboardComponentRequest {
    component_version_id: Uuid,
    position: i32,
    #[serde(default)]
    config: Value,
}

#[derive(Serialize)]
pub struct DashboardSummary {
    id: Uuid,
    name: String,
    description: Option<String>,
    component_count: i64,
}

#[derive(Serialize)]
pub struct DashboardResponse {
    id: Uuid,
    name: String,
    description: Option<String>,
    components: Vec<DashboardComponentResponse>,
}

#[derive(Serialize)]
pub struct DashboardComponentResponse {
    id: Uuid,
    position: i32,
    config: Value,
    component_version_id: Uuid,
    component_id: Uuid,
    component_name: String,
    component_slug: String,
    component_type: String,
    dataset_revision_id: Uuid,
}

pub async fn create_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDashboardRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "dashboards:write").await?;
    require_text("dashboard name", &payload.name)?;
    let id = sqlx::query_scalar(
        "INSERT INTO dashboards (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(payload.name.trim())
    .bind(payload.description)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(IdResponse { id }))
}

pub async fn update_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(payload): Json<CreateDashboardRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "dashboards:write").await?;
    require_dashboard_exists(&state.pool, dashboard_id).await?;
    require_text("dashboard name", &payload.name)?;
    sqlx::query("UPDATE dashboards SET name = $1, description = $2 WHERE id = $3")
        .bind(payload.name.trim())
        .bind(payload.description)
        .bind(dashboard_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(IdResponse { id: dashboard_id }))
}

pub async fn delete_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "dashboards:write").await?;
    require_dashboard_exists(&state.pool, dashboard_id).await?;
    sqlx::query("DELETE FROM dashboards WHERE id = $1")
        .bind(dashboard_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(IdResponse { id: dashboard_id }))
}

pub async fn add_dashboard_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(payload): Json<AddDashboardComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "dashboards:write").await?;
    require_dashboard_exists(&state.pool, dashboard_id).await?;
    require_component_version_exists(&state.pool, payload.component_version_id).await?;
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO dashboard_components (dashboard_id, component_version_id, position, config)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(dashboard_id)
    .bind(payload.component_version_id)
    .bind(payload.position)
    .bind(payload.config)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(IdResponse { id }))
}

pub async fn update_dashboard_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    Json(payload): Json<AddDashboardComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "dashboards:write").await?;
    require_component_version_exists(&state.pool, payload.component_version_id).await?;
    sqlx::query(
        r#"
        UPDATE dashboard_components
        SET component_version_id = $2, position = $3, config = $4
        WHERE id = $1
        "#,
    )
    .bind(component_id)
    .bind(payload.component_version_id)
    .bind(payload.position)
    .bind(payload.config)
    .execute(&state.pool)
    .await?;
    Ok(Json(IdResponse { id: component_id }))
}

pub async fn delete_dashboard_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "dashboards:write").await?;
    sqlx::query("DELETE FROM dashboard_components WHERE id = $1")
        .bind(component_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(IdResponse { id: component_id }))
}

pub async fn list_dashboards(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<DashboardSummary>>> {
    auth::require_capability(&state.pool, &headers, "dashboards:read").await?;
    let rows = sqlx::query(
        r#"
        SELECT dashboards.id, dashboards.name, dashboards.description,
               COUNT(dashboard_components.id) AS component_count
        FROM dashboards
        LEFT JOIN dashboard_components ON dashboard_components.dashboard_id = dashboards.id
        GROUP BY dashboards.id, dashboards.name, dashboards.description
        ORDER BY dashboards.name, dashboards.id
        "#,
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| {
                Ok(DashboardSummary {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    component_count: row.try_get("component_count")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
    ))
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> ApiResult<Json<DashboardResponse>> {
    auth::require_capability(&state.pool, &headers, "dashboards:read").await?;
    let dashboard = sqlx::query("SELECT id, name, description FROM dashboards WHERE id = $1")
        .bind(dashboard_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dashboard {dashboard_id}")))?;
    let component_rows = sqlx::query(
        r#"
        SELECT
            dashboard_components.id,
            dashboard_components.position,
            dashboard_components.config,
            component_versions.id AS component_version_id,
            component_versions.component_id,
            component_versions.component_type::text AS component_type,
            component_versions.dataset_revision_id,
            components.name AS component_name,
            components.slug AS component_slug
        FROM dashboard_components
        JOIN component_versions ON component_versions.id = dashboard_components.component_version_id
        JOIN components ON components.id = component_versions.component_id
        WHERE dashboard_components.dashboard_id = $1
        ORDER BY dashboard_components.position, dashboard_components.id
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(DashboardResponse {
        id: dashboard.try_get("id")?,
        name: dashboard.try_get("name")?,
        description: dashboard.try_get("description")?,
        components: component_rows
            .into_iter()
            .map(|row| {
                Ok(DashboardComponentResponse {
                    id: row.try_get("id")?,
                    position: row.try_get("position")?,
                    config: row.try_get("config")?,
                    component_version_id: row.try_get("component_version_id")?,
                    component_id: row.try_get("component_id")?,
                    component_name: row.try_get("component_name")?,
                    component_slug: row.try_get("component_slug")?,
                    component_type: row.try_get("component_type")?,
                    dataset_revision_id: row.try_get("dataset_revision_id")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
    }))
}

async fn require_dashboard_exists(pool: &sqlx::PgPool, dashboard_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM dashboards WHERE id = $1)")
        .bind(dashboard_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("dashboard {dashboard_id}")))
    }
}

async fn require_component_version_exists(
    pool: &sqlx::PgPool,
    component_version_id: Uuid,
) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM component_versions WHERE id = $1)")
            .bind(component_version_id)
            .fetch_one(pool)
            .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!(
            "component version {component_version_id}"
        )))
    }
}
