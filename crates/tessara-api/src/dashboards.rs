use std::str::FromStr;

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use tessara_dashboards::ChartType;
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::IdResponse,
};

#[derive(Deserialize)]
pub struct CreateChartRequest {
    name: String,
    report_id: Option<Uuid>,
    chart_type: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateDashboardRequest {
    name: String,
}

#[derive(Deserialize)]
pub struct AddDashboardComponentRequest {
    chart_id: Uuid,
    position: i32,
    #[serde(default)]
    config: Value,
}

#[derive(Serialize)]
pub struct DashboardResponse {
    id: Uuid,
    name: String,
    components: Vec<DashboardComponentResponse>,
}

#[derive(Serialize)]
pub struct DashboardComponentResponse {
    id: Uuid,
    position: i32,
    config: Value,
    chart: ChartResponse,
}

#[derive(Serialize)]
pub struct ChartResponse {
    id: Uuid,
    name: String,
    chart_type: String,
    report_id: Option<Uuid>,
    report_url: Option<String>,
}

pub async fn create_chart(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateChartRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;

    let chart_type = payload
        .chart_type
        .as_deref()
        .map(ChartType::from_str)
        .transpose()
        .map_err(|error| ApiError::BadRequest(error.to_string()))?
        .unwrap_or(ChartType::Table);

    let id = sqlx::query_scalar(
        "INSERT INTO charts (name, report_id, chart_type) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(payload.name)
    .bind(payload.report_id)
    .bind(chart_type.as_str())
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

pub async fn create_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDashboardRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;

    let id = sqlx::query_scalar("INSERT INTO dashboards (name) VALUES ($1) RETURNING id")
        .bind(payload.name)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(IdResponse { id }))
}

pub async fn add_dashboard_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(payload): Json<AddDashboardComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO dashboard_components (dashboard_id, chart_id, position, config)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(dashboard_id)
    .bind(payload.chart_id)
    .bind(payload.position)
    .bind(if payload.config.is_null() {
        serde_json::json!({})
    } else {
        payload.config
    })
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
) -> ApiResult<Json<DashboardResponse>> {
    let dashboard = sqlx::query("SELECT id, name FROM dashboards WHERE id = $1")
        .bind(dashboard_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dashboard {dashboard_id}")))?;

    let rows = sqlx::query(
        r#"
        SELECT
            dashboard_components.id AS component_id,
            dashboard_components.position,
            dashboard_components.config,
            charts.id AS chart_id,
            charts.name AS chart_name,
            charts.chart_type,
            charts.report_id
        FROM dashboard_components
        JOIN charts ON charts.id = dashboard_components.chart_id
        WHERE dashboard_components.dashboard_id = $1
        ORDER BY dashboard_components.position, charts.name
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(&state.pool)
    .await?;

    let mut components = Vec::new();
    for row in rows {
        let report_id: Option<Uuid> = row.try_get("report_id")?;
        components.push(DashboardComponentResponse {
            id: row.try_get("component_id")?,
            position: row.try_get("position")?,
            config: row.try_get("config")?,
            chart: ChartResponse {
                id: row.try_get("chart_id")?,
                name: row.try_get("chart_name")?,
                chart_type: row.try_get("chart_type")?,
                report_id,
                report_url: report_id.map(|id| format!("/api/reports/{id}/table")),
            },
        });
    }

    Ok(Json(DashboardResponse {
        id: dashboard.try_get("id")?,
        name: dashboard.try_get("name")?,
        components,
    }))
}
