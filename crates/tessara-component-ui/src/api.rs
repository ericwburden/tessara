//! API client helpers for the Components feature.

#[cfg(feature = "hydrate")]
use super::http::{fetch_json_request, send_json_request};
#[cfg(feature = "hydrate")]
use super::types::{
    ComponentDefinition, ComponentLifecycleRequest, ComponentLifecycleResponse, ComponentSummary,
    ComponentValidationResponse, CreateComponentVersionRequest, DatasetDistinctValues,
    DatasetSummary, IdResponse, SaveComponentEditRequest, dataset_reference,
    remember_dataset_references,
};
#[cfg(feature = "hydrate")]
use serde::Deserialize;
#[cfg(feature = "hydrate")]
use tessara_component_viewer_ui::ComponentRenderResponse;
#[cfg(feature = "hydrate")]
use tessara_datasets_contract::DATASET_CONTRACT_SCHEMA_VERSION;

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
pub(crate) async fn preview_component(
    payload: CreateComponentVersionRequest,
) -> Result<ComponentRenderResponse, String> {
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
    #[derive(Deserialize)]
    struct Catalog {
        datasets: Vec<DatasetSummary>,
    }
    let response =
        fetch_json_request::<Catalog>("/api/admin/components/datasets", "Dataset list").await?;
    Ok(response.map(|catalog| {
        remember_dataset_references(&catalog.datasets);
        catalog.datasets
    }))
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_dataset_distinct_values(
    dataset_id: &str,
    version_major: i32,
    field: &str,
) -> Result<Option<DatasetDistinctValues>, String> {
    #[derive(Deserialize)]
    struct DistinctResponse {
        values: Vec<serde_json::Value>,
    }
    let reference = dataset_reference(dataset_id, version_major)
        .ok_or_else(|| "Selected Dataset reference is unavailable.".to_string())?;
    let response: DistinctResponse = send_json_request(
        gloo_net::http::Request::post("/api/admin/components/datasets/distinct-values"),
        serde_json::json!({"schema_version":DATASET_CONTRACT_SCHEMA_VERSION,"action":"distinct_values","reference":reference,"field_key":field,"limit":100}).to_string(),
        "Dataset distinct values",
    ).await?;
    Ok(Some(DatasetDistinctValues {
        dataset_id: dataset_id.into(),
        version_major,
        field: field.into(),
        values: response
            .values
            .into_iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string())
            })
            .collect(),
    }))
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
pub(crate) async fn change_component_version_lifecycle(
    component_id: &str,
    version_id: &str,
    action: &str,
    expected_resource_revision: i64,
) -> Result<ComponentLifecycleResponse, String> {
    let payload = ComponentLifecycleRequest {
        schema_version: 1,
        action: action.into(),
        expected_resource_revision,
    };
    send_json_request(
        gloo_net::http::Request::post(&format!(
            "/api/admin/components/{component_id}/versions/{version_id}/lifecycle"
        )),
        serde_json::to_string(&payload)
            .map_err(|_| "Component lifecycle payload is invalid.".to_string())?,
        "Component lifecycle",
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
