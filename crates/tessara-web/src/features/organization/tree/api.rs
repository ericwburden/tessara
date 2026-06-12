//! Transport calls for organization tree screens.

#[cfg(feature = "hydrate")]
use super::errors::OrganizationTreeApiError;
#[cfg(feature = "hydrate")]
use crate::features::organization::types::{
    NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail,
};

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_tree_context()
-> Result<(Vec<OrganizationNode>, Vec<NodeTypeCatalogEntry>), OrganizationTreeApiError> {
    let node_response = gloo_net::http::Request::get("/api/nodes").send().await;
    let node_type_response = gloo_net::http::Request::get("/api/node-types").send().await;

    match (node_response, node_type_response) {
        (Ok(response), _) if response.status() == 401 => {
            Err(OrganizationTreeApiError::Unauthorized)
        }
        (_, Ok(response)) if response.status() == 401 => {
            Err(OrganizationTreeApiError::Unauthorized)
        }
        (Ok(node_response), Ok(node_type_response))
            if node_response.ok() && node_type_response.ok() =>
        {
            let loaded_nodes = node_response.json::<Vec<OrganizationNode>>().await;
            let loaded_node_types = node_type_response.json::<Vec<NodeTypeCatalogEntry>>().await;

            match (loaded_nodes, loaded_node_types) {
                (Ok(nodes), Ok(node_types)) => Ok((nodes, node_types)),
                _ => Err(OrganizationTreeApiError::message(
                    "The hierarchy response could not be read.",
                )),
            }
        }
        (Ok(_), Ok(_)) => Err(OrganizationTreeApiError::message(
            "The hierarchy API returned an unexpected response.",
        )),
        _ => Err(OrganizationTreeApiError::message(
            "Could not reach the hierarchy API.",
        )),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_organization_detail(
    node_id: &str,
) -> Result<OrganizationNodeDetail, OrganizationTreeApiError> {
    let response = gloo_net::http::Request::get(&format!("/api/nodes/{node_id}"))
        .send()
        .await;

    match response {
        Ok(response) if response.status() == 401 => Err(OrganizationTreeApiError::Unauthorized),
        Ok(response) if response.ok() => {
            response
                .json::<OrganizationNodeDetail>()
                .await
                .map_err(|_| {
                    OrganizationTreeApiError::message("The detail response could not be read.")
                })
        }
        Ok(_) => Err(OrganizationTreeApiError::message(
            "The detail API returned an unexpected response.",
        )),
        Err(_) => Err(OrganizationTreeApiError::message(
            "Could not reach the detail API.",
        )),
    }
}
