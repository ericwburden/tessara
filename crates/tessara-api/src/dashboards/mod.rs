//! Dashboard transport boundary.
//!
//! Handlers authenticate and adapt HTTP only. Dashboard policy and
//! authorization live in `service`; SQL lives in `repository`.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
};
use uuid::Uuid;

mod dto;
mod error;
mod native;
mod projection;
mod reconciliation;
mod repository;
mod scope;
mod service;

pub(crate) use native::viewer as native_viewer;
pub(crate) use native::{create as native_create, detail as native_detail};
pub(crate) use native::{directory as native_directory, editor as native_editor};

use dto::{
    CreateDashboardRequest, DashboardCompositionResponse, DashboardResponse, DashboardSummary,
    DashboardVisibilityNodeOption, ReconcileDashboardCompositionRequest,
};
use reconciliation::reconcile_composition;

use crate::{auth, db::AppState, hierarchy::IdResponse};
use error::DashboardResult;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/admin/dashboards", post(create_dashboard))
        .route(
            "/api/admin/dashboards/visibility-nodes",
            get(list_dashboard_visibility_nodes),
        )
        .route(
            "/api/admin/dashboards/{dashboard_id}",
            axum::routing::put(update_dashboard).delete(delete_dashboard),
        )
        .route(
            "/api/admin/dashboards/{dashboard_id}/composition",
            get(get_dashboard_composition).put(reconcile_dashboard_composition),
        )
        .route("/api/dashboards/{dashboard_id}", get(get_dashboard))
        .route("/api/dashboards", get(list_dashboards))
}

async fn list_dashboard_visibility_nodes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> DashboardResult<Json<Vec<DashboardVisibilityNodeOption>>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    Ok(Json(
        service::list_visibility_node_options(&state.pool, &account).await?,
    ))
}

async fn create_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDashboardRequest>,
) -> DashboardResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    Ok(Json(
        service::create_dashboard(&state.pool, &account, payload).await?,
    ))
}

async fn update_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(payload): Json<CreateDashboardRequest>,
) -> DashboardResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    Ok(Json(
        service::update_dashboard(&state.pool, &account, dashboard_id, payload).await?,
    ))
}

async fn delete_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> DashboardResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    Ok(Json(
        service::delete_dashboard(&state.pool, &account, dashboard_id).await?,
    ))
}

async fn list_dashboards(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> DashboardResult<Json<Vec<DashboardSummary>>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:read").await?;
    Ok(Json(service::list_dashboards(&state.pool, &account).await?))
}

async fn get_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> DashboardResult<Json<DashboardResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:read").await?;
    Ok(Json(
        service::get_dashboard(&state.pool, &account, dashboard_id).await?,
    ))
}

async fn get_dashboard_composition(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> DashboardResult<Json<DashboardCompositionResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    Ok(Json(
        service::load_composition(&state.pool, &account, dashboard_id).await?,
    ))
}

async fn reconcile_dashboard_composition(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(payload): Json<ReconcileDashboardCompositionRequest>,
) -> DashboardResult<Json<DashboardCompositionResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    Ok(Json(
        reconcile_composition(&state.pool, &account, dashboard_id, payload).await?,
    ))
}
