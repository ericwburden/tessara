//! Transport calls for organization tree screens.

#[cfg(feature = "hydrate")]
use super::errors::OrganizationTreeApiError;
#[cfg(feature = "hydrate")]
use crate::types::{NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail};

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_tree_context()
-> Result<(Vec<OrganizationNode>, Vec<NodeTypeCatalogEntry>), OrganizationTreeApiError> {
    let nodes = tessara_web_http::fetch_json("/api/nodes", "Organization hierarchy")
        .await
        .map_err(OrganizationTreeApiError::from_request)?;
    let node_types = tessara_web_http::fetch_json("/api/node-types", "Organization node types")
        .await
        .map_err(OrganizationTreeApiError::from_request)?;
    Ok((nodes, node_types))
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_organization_detail(
    node_id: &str,
) -> Result<OrganizationNodeDetail, OrganizationTreeApiError> {
    tessara_web_http::fetch_json(&format!("/api/nodes/{node_id}"), "Organization detail")
        .await
        .map_err(OrganizationTreeApiError::from_request)
}
