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
    hierarchy::{IdResponse, require_text},
};

#[derive(Deserialize)]
pub struct CreateChartRequest {
    name: String,
    report_id: Option<Uuid>,
    aggregation_id: Option<Uuid>,
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
    report_name: Option<String>,
    report_form_name: Option<String>,
    report_url: Option<String>,
    aggregation_id: Option<Uuid>,
    aggregation_name: Option<String>,
    aggregation_report_name: Option<String>,
    aggregation_url: Option<String>,
}

#[derive(Serialize)]
pub struct DashboardSummary {
    id: Uuid,
    name: String,
    component_count: i64,
}

pub async fn create_chart(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateChartRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_text("chart name", &payload.name)?;

    let chart_type = parse_chart_type(payload.chart_type.as_deref())?;

    require_single_chart_data_source(payload.report_id, payload.aggregation_id)?;
    if let Some(report_id) = payload.report_id {
        require_report_exists(&state.pool, report_id).await?;
    }
    if let Some(aggregation_id) = payload.aggregation_id {
        require_aggregation_exists(&state.pool, aggregation_id).await?;
    }

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO charts (name, report_id, aggregation_id, chart_type)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(payload.name)
    .bind(payload.report_id)
    .bind(payload.aggregation_id)
    .bind(chart_type.as_str())
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

/// Updates an existing chart definition.
pub async fn update_chart(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(chart_id): Path<Uuid>,
    Json(payload): Json<CreateChartRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_chart_exists(&state.pool, chart_id).await?;
    require_text("chart name", &payload.name)?;
    let chart_type = parse_chart_type(payload.chart_type.as_deref())?;

    require_single_chart_data_source(payload.report_id, payload.aggregation_id)?;
    if let Some(report_id) = payload.report_id {
        require_report_exists(&state.pool, report_id).await?;
    }
    if let Some(aggregation_id) = payload.aggregation_id {
        require_aggregation_exists(&state.pool, aggregation_id).await?;
    }

    sqlx::query(
        r#"
        UPDATE charts
        SET name = $1, report_id = $2, aggregation_id = $3, chart_type = $4
        WHERE id = $5
        "#,
    )
    .bind(payload.name)
    .bind(payload.report_id)
    .bind(payload.aggregation_id)
    .bind(chart_type.as_str())
    .bind(chart_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(IdResponse { id: chart_id }))
}

/// Deletes an existing chart definition and removes any dashboard components using it.
pub async fn delete_chart(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(chart_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_chart_exists(&state.pool, chart_id).await?;

    sqlx::query("DELETE FROM charts WHERE id = $1")
        .bind(chart_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: chart_id }))
}

/// Lists chart definitions for dashboard builder screens.
pub async fn list_charts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ChartResponse>>> {
    auth::require_capability(&state.pool, &headers, "reports:read").await?;

    let rows = sqlx::query(
        r#"
        SELECT
            charts.id,
            charts.name,
            charts.chart_type::text AS chart_type,
            charts.report_id,
            charts.aggregation_id,
            reports.name AS report_name,
            forms.name AS report_form_name,
            aggregations.name AS aggregation_name,
            aggregation_reports.name AS aggregation_report_name
        FROM charts
        LEFT JOIN reports ON reports.id = charts.report_id
        LEFT JOIN forms ON forms.id = reports.form_id
        LEFT JOIN aggregations ON aggregations.id = charts.aggregation_id
        LEFT JOIN reports AS aggregation_reports
            ON aggregation_reports.id = aggregations.report_id
        ORDER BY charts.name, charts.id
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let charts = rows
        .into_iter()
        .map(|row| {
            let report_id: Option<Uuid> = row.try_get("report_id")?;
            let aggregation_id: Option<Uuid> = row.try_get("aggregation_id")?;
            Ok(ChartResponse {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                chart_type: row.try_get("chart_type")?,
                report_id,
                report_name: row.try_get("report_name")?,
                report_form_name: row.try_get("report_form_name")?,
                report_url: report_id.map(|id| format!("/api/reports/{id}/table")),
                aggregation_id,
                aggregation_name: row.try_get("aggregation_name")?,
                aggregation_report_name: row.try_get("aggregation_report_name")?,
                aggregation_url: aggregation_id.map(|id| format!("/api/aggregations/{id}/table")),
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(charts))
}

pub async fn create_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDashboardRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_text("dashboard name", &payload.name)?;

    let id = sqlx::query_scalar("INSERT INTO dashboards (name) VALUES ($1) RETURNING id")
        .bind(payload.name)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(IdResponse { id }))
}

/// Updates an existing dashboard definition.
pub async fn update_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(payload): Json<CreateDashboardRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_dashboard_exists(&state.pool, dashboard_id).await?;
    require_text("dashboard name", &payload.name)?;

    sqlx::query("UPDATE dashboards SET name = $1 WHERE id = $2")
        .bind(payload.name)
        .bind(dashboard_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: dashboard_id }))
}

/// Deletes an existing dashboard and its component layout.
pub async fn delete_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
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
    auth::require_capability(&state.pool, &headers, "reports:write").await?;

    require_dashboard_exists(&state.pool, dashboard_id).await?;
    require_chart_exists(&state.pool, payload.chart_id).await?;

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
    .bind(normalize_component_config(payload.config))
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

/// Updates an existing dashboard component.
pub async fn update_dashboard_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    Json(payload): Json<AddDashboardComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_dashboard_component_exists(&state.pool, component_id).await?;
    require_chart_exists(&state.pool, payload.chart_id).await?;

    sqlx::query(
        r#"
        UPDATE dashboard_components
        SET chart_id = $1, position = $2, config = $3
        WHERE id = $4
        "#,
    )
    .bind(payload.chart_id)
    .bind(payload.position)
    .bind(normalize_component_config(payload.config))
    .bind(component_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(IdResponse { id: component_id }))
}

/// Removes a dashboard component from its dashboard.
pub async fn delete_dashboard_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_dashboard_component_exists(&state.pool, component_id).await?;

    sqlx::query("DELETE FROM dashboard_components WHERE id = $1")
        .bind(component_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: component_id }))
}

pub async fn list_dashboards(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<DashboardSummary>>> {
    let rows = sqlx::query(
        r#"
        SELECT dashboards.id, dashboards.name, COUNT(dashboard_components.id) AS component_count
        FROM dashboards
        LEFT JOIN dashboard_components ON dashboard_components.dashboard_id = dashboards.id
        GROUP BY dashboards.id, dashboards.name
        ORDER BY dashboards.name, dashboards.id
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let dashboards = rows
        .into_iter()
        .map(|row| {
            Ok(DashboardSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                component_count: row.try_get("component_count")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(dashboards))
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
            charts.report_id,
            charts.aggregation_id,
            reports.name AS report_name,
            forms.name AS report_form_name,
            aggregations.name AS aggregation_name,
            aggregation_reports.name AS aggregation_report_name
        FROM dashboard_components
        JOIN charts ON charts.id = dashboard_components.chart_id
        LEFT JOIN reports ON reports.id = charts.report_id
        LEFT JOIN forms ON forms.id = reports.form_id
        LEFT JOIN aggregations ON aggregations.id = charts.aggregation_id
        LEFT JOIN reports AS aggregation_reports
            ON aggregation_reports.id = aggregations.report_id
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
        let aggregation_id: Option<Uuid> = row.try_get("aggregation_id")?;
        components.push(DashboardComponentResponse {
            id: row.try_get("component_id")?,
            position: row.try_get("position")?,
            config: row.try_get("config")?,
            chart: ChartResponse {
                id: row.try_get("chart_id")?,
                name: row.try_get("chart_name")?,
                chart_type: row.try_get("chart_type")?,
                report_id,
                report_name: row.try_get("report_name")?,
                report_form_name: row.try_get("report_form_name")?,
                report_url: report_id.map(|id| format!("/api/reports/{id}/table")),
                aggregation_id,
                aggregation_name: row.try_get("aggregation_name")?,
                aggregation_report_name: row.try_get("aggregation_report_name")?,
                aggregation_url: aggregation_id.map(|id| format!("/api/aggregations/{id}/table")),
            },
        });
    }

    Ok(Json(DashboardResponse {
        id: dashboard.try_get("id")?,
        name: dashboard.try_get("name")?,
        components,
    }))
}

fn parse_chart_type(chart_type: Option<&str>) -> ApiResult<ChartType> {
    chart_type
        .map(ChartType::from_str)
        .transpose()
        .map_err(|error| ApiError::BadRequest(error.to_string()))
        .map(|chart_type| chart_type.unwrap_or(ChartType::Table))
}

fn normalize_component_config(config: Value) -> Value {
    if config.is_null() {
        serde_json::json!({})
    } else {
        config
    }
}

fn require_single_chart_data_source(
    report_id: Option<Uuid>,
    aggregation_id: Option<Uuid>,
) -> ApiResult<()> {
    if report_id.is_some() && aggregation_id.is_some() {
        Err(ApiError::BadRequest(
            "a chart can target either a report or an aggregation, not both".into(),
        ))
    } else {
        Ok(())
    }
}

async fn require_report_exists(pool: &sqlx::PgPool, report_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM reports WHERE id = $1)")
        .bind(report_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("report {report_id}")))
    }
}

async fn require_aggregation_exists(pool: &sqlx::PgPool, aggregation_id: Uuid) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM aggregations WHERE id = $1)")
            .bind(aggregation_id)
            .fetch_one(pool)
            .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("aggregation {aggregation_id}")))
    }
}

async fn require_dashboard_exists(pool: &sqlx::PgPool, dashboard_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM dashboards WHERE id = $1)")
        .bind(dashboard_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("dashboard {dashboard_id}")))
    }
}

async fn require_chart_exists(pool: &sqlx::PgPool, chart_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM charts WHERE id = $1)")
        .bind(chart_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("chart {chart_id}")))
    }
}

async fn require_dashboard_component_exists(
    pool: &sqlx::PgPool,
    component_id: Uuid,
) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM dashboard_components WHERE id = $1)")
            .bind(component_id)
            .fetch_one(pool)
            .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!(
            "dashboard component {component_id}"
        )))
    }
}
