//! API client helpers for the Components feature.

#[cfg(feature = "hydrate")]
use super::http::{fetch_json_request, send_json_request};
#[cfg(feature = "hydrate")]
use super::types::{
    ComponentDefinition, ComponentSummary, ComponentTable, ComponentValidationResponse,
    ComponentVisual, CreateComponentVersionRequest, DatasetDistinctValues, DatasetSummary,
    IdResponse, SaveComponentEditRequest,
};

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_components() -> Result<Option<Vec<ComponentSummary>>, String> {
    fetch_json_request("/api/components", "Component list").await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_admin_components() -> Result<Option<Vec<ComponentSummary>>, String> {
    fetch_json_request("/api/admin/components", "Component list").await
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
pub(crate) async fn fetch_component_visual(
    component_ref: &str,
    component_type: &str,
) -> Result<Option<ComponentVisual>, String> {
    let path_kind = match component_type {
        "stat_card" => "stat-card",
        "bar" | "line" | "pie" | "donut" => component_type,
        _ => return Ok(None),
    };
    fetch_json_request(
        &format!("/api/components/{component_ref}/{path_kind}"),
        "Component visual",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn preview_component_visual(
    payload: CreateComponentVersionRequest,
) -> Result<ComponentVisual, String> {
    send_json_request(
        gloo_net::http::Request::post("/api/admin/components/preview"),
        serde_json::to_string(&payload)
            .map_err(|_| "Component preview payload is invalid.".to_string())?,
        "Component preview",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_datasets() -> Result<Option<Vec<DatasetSummary>>, String> {
    fetch_json_request("/api/datasets", "Dataset list").await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_dataset_distinct_values(
    dataset_id: &str,
    version_major: i32,
    field: &str,
) -> Result<Option<DatasetDistinctValues>, String> {
    fetch_json_request(
        &format!(
            "/api/datasets/{dataset_id}/distinct-values?version_major={version_major}&field={field}"
        ),
        "Dataset distinct values",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn save_component_edit(
    payload: SaveComponentEditRequest,
) -> Result<IdResponse, String> {
    send_json_request(
        gloo_net::http::Request::post("/api/admin/components/save"),
        serde_json::to_string(&payload)
            .map_err(|_| "Component save payload is invalid.".to_string())?,
        "Save component",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn delete_component_version(
    component_id: &str,
    version_id: &str,
) -> Result<IdResponse, String> {
    send_json_request(
        gloo_net::http::Request::delete(&format!(
            "/api/admin/components/{component_id}/versions/{version_id}"
        )),
        "{}".into(),
        "Delete component draft",
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
