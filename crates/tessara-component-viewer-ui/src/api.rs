//! Exact-version endpoint clients for the reusable Component viewer.

#[cfg(feature = "hydrate")]
use crate::http::{ComponentRequestError, fetch_json_request};
#[cfg(feature = "hydrate")]
use crate::types::{ComponentTable, ComponentVisual};

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_component_table_endpoint(
    endpoint: &str,
    query: &str,
) -> Result<Option<ComponentTable>, ComponentRequestError> {
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
) -> Result<Option<ComponentVisual>, ComponentRequestError> {
    fetch_json_request(endpoint, "Component visual").await
}
