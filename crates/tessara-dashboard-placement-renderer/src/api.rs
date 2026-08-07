//! Exact-version endpoint clients for Dashboard placement rendering.

#[cfg(feature = "hydrate")]
use crate::http::{ComponentRequestError, fetch_json_request};
#[cfg(feature = "hydrate")]
use tessara_components_contract::ComponentRenderResponse;

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_component_table_endpoint(
    endpoint: &str,
    query: &str,
) -> Result<Option<ComponentRenderResponse>, ComponentRequestError> {
    let suffix = if query.is_empty() {
        String::new()
    } else {
        format!("?{query}")
    };
    fetch_json_request(&format!("{endpoint}{suffix}"), "Component table").await
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_component_visual_endpoint(
    endpoint: &str,
) -> Result<Option<ComponentRenderResponse>, ComponentRequestError> {
    fetch_json_request(endpoint, "Component visual").await
}
