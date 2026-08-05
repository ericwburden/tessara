//! Dashboard endpoint orchestration for hydrated feature content.

#[cfg(feature = "hydrate")]
use crate::http::{fetch_json, send_json, send_without_response};
#[cfg(feature = "hydrate")]
use crate::types::{
    Dashboard, DashboardComposition, DashboardDependencyActionRequest,
    DashboardDependencyActionResponse, DashboardDependencyHealth, DashboardMetadataRequest,
    DashboardSummary, IdResponse, ReconcileDashboardCompositionRequest, SessionAccount,
    VisibilityNodeOption,
};

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_account() -> Result<SessionAccount, String> {
    fetch_json("/api/me", "account").await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_dashboards() -> Result<Vec<DashboardSummary>, String> {
    fetch_json("/api/dashboards", "Dashboard list").await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_dashboard(dashboard_id: &str) -> Result<Dashboard, String> {
    fetch_json(
        &format!("/api/dashboards/{dashboard_id}"),
        "Dashboard detail",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_composition(dashboard_id: &str) -> Result<DashboardComposition, String> {
    fetch_json(
        &format!("/api/admin/dashboards/{dashboard_id}/composition"),
        "Dashboard composition",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_visibility_nodes() -> Result<Vec<VisibilityNodeOption>, String> {
    fetch_json(
        "/api/admin/dashboards/visibility-nodes",
        "Dashboard visibility nodes",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn create_dashboard(
    payload: &DashboardMetadataRequest,
) -> Result<IdResponse, String> {
    send_json(
        gloo_net::http::Request::post("/api/admin/dashboards"),
        payload,
        "Dashboard creation",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn update_dashboard(
    dashboard_id: &str,
    payload: &DashboardMetadataRequest,
) -> Result<IdResponse, String> {
    send_json(
        gloo_net::http::Request::put(&format!("/api/admin/dashboards/{dashboard_id}")),
        payload,
        "Dashboard settings",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn delete_dashboard(dashboard_id: &str) -> Result<(), String> {
    send_without_response(
        gloo_net::http::Request::delete(&format!("/api/admin/dashboards/{dashboard_id}")),
        "Dashboard deletion",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn save_composition(
    dashboard_id: &str,
    payload: &ReconcileDashboardCompositionRequest,
) -> Result<DashboardComposition, String> {
    send_json(
        gloo_net::http::Request::put(&format!("/api/admin/dashboards/{dashboard_id}/composition")),
        payload,
        "Dashboard layout save",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_dependency_health(
    dashboard_id: &str,
) -> Result<DashboardDependencyHealth, String> {
    fetch_json(
        &format!("/api/admin/dashboards/{dashboard_id}/dependencies"),
        "Dashboard dependency health",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn refresh_dependency_health(
    dashboard_id: &str,
) -> Result<DashboardDependencyHealth, String> {
    send_json(
        gloo_net::http::Request::post(&format!(
            "/api/admin/dashboards/{dashboard_id}/dependencies/refresh"
        )),
        &serde_json::json!({}),
        "Dashboard dependency refresh",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn act_on_dependency(
    dashboard_id: &str,
    finding_id: &str,
    request: &DashboardDependencyActionRequest,
    idempotency_key: &str,
) -> Result<DashboardDependencyActionResponse, String> {
    send_json(
        gloo_net::http::Request::post(&format!(
            "/api/admin/dashboards/{dashboard_id}/dependencies/{finding_id}/actions"
        ))
        .header("X-Idempotency-Key", idempotency_key),
        request,
        "Dashboard dependency action",
    )
    .await
}
