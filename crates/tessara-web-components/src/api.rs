//! API client helpers for the Components feature.

#[cfg(feature = "hydrate")]
use super::http::{fetch_json_request, send_json_request};
#[cfg(feature = "hydrate")]
use super::types::{
    ComponentDefinition, ComponentSummary, ComponentTable, ComponentValidationResponse,
    CreateComponentRequest, CreateComponentVersionRequest, DatasetSummary, IdResponse,
    UpdateComponentRequest,
};

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_components() -> Result<Option<Vec<ComponentSummary>>, String> {
    fetch_json_request("/api/components", "Component list").await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_component(
    component_ref: &str,
) -> Result<Option<ComponentDefinition>, String> {
    fetch_json_request(
        &format!("/api/components/{component_ref}"),
        "Component detail",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_admin_component(
    component_ref: &str,
) -> Result<Option<ComponentDefinition>, String> {
    fetch_json_request(
        &format!("/api/admin/components/{component_ref}"),
        "Component detail",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_component_table(
    component_ref: &str,
    query: &str,
) -> Result<Option<ComponentTable>, String> {
    let suffix = if query.is_empty() {
        String::new()
    } else {
        format!("?{query}")
    };
    fetch_json_request(
        &format!("/api/components/{component_ref}/table{suffix}"),
        "Component table",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_datasets() -> Result<Option<Vec<DatasetSummary>>, String> {
    fetch_json_request("/api/datasets", "Dataset list").await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn create_component(
    payload: CreateComponentRequest,
) -> Result<IdResponse, String> {
    send_json_request(
        gloo_net::http::Request::post("/api/admin/components"),
        serde_json::to_string(&payload).map_err(|_| "Component payload is invalid.".to_string())?,
        "Create component",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn update_component(
    component_id: &str,
    payload: UpdateComponentRequest,
) -> Result<IdResponse, String> {
    send_json_request(
        gloo_net::http::Request::patch(&format!("/api/admin/components/{component_id}")),
        serde_json::to_string(&payload)
            .map_err(|_| "Component metadata payload is invalid.".to_string())?,
        "Update component",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn save_component_version(
    component_id: &str,
    payload: CreateComponentVersionRequest,
) -> Result<IdResponse, String> {
    send_json_request(
        gloo_net::http::Request::post(&format!("/api/admin/components/{component_id}/versions")),
        serde_json::to_string(&payload)
            .map_err(|_| "Component version payload is invalid.".to_string())?,
        "Save component draft",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn update_component_version(
    component_id: &str,
    version_id: &str,
    payload: CreateComponentVersionRequest,
) -> Result<IdResponse, String> {
    send_json_request(
        gloo_net::http::Request::patch(&format!(
            "/api/admin/components/{component_id}/versions/{version_id}"
        )),
        serde_json::to_string(&payload)
            .map_err(|_| "Component version payload is invalid.".to_string())?,
        "Update component draft",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn validate_component_version(
    payload: CreateComponentVersionRequest,
) -> Result<ComponentValidationResponse, String> {
    send_json_request(
        gloo_net::http::Request::post("/api/admin/components/validate"),
        serde_json::to_string(&payload)
            .map_err(|_| "Component validation payload is invalid.".to_string())?,
        "Validate component",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn publish_component_version(
    component_id: &str,
    version_id: &str,
) -> Result<IdResponse, String> {
    send_json_request(
        gloo_net::http::Request::post(&format!(
            "/api/admin/components/{component_id}/versions/{version_id}/publish"
        )),
        "{}".into(),
        "Publish component",
    )
    .await
}
