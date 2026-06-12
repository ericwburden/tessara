//! Transport calls for organization node editor pages.

#[cfg(feature = "hydrate")]
use crate::features::administration::{CreateNodePayload, UpdateNodePayload};
use crate::features::organization::types::NodeTypeDefinition;
#[cfg(feature = "hydrate")]
use crate::http::{IdResponse, redirect_to_login};

/// Node editor API failure.
pub(super) enum NodeEditorApiError {
    Unauthorized,
    Message(String),
}

impl NodeEditorApiError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

/// Fetches the full node type definition, including metadata fields.
pub(super) async fn fetch_node_type_definition(
    node_type_id: &str,
) -> Result<NodeTypeDefinition, NodeEditorApiError> {
    match gloo_net::http::Request::get(&format!("/api/admin/node-types/{node_type_id}"))
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(NodeEditorApiError::Unauthorized),
        Ok(response) if response.ok() => response
            .json::<NodeTypeDefinition>()
            .await
            .map_err(|_| NodeEditorApiError::message("Metadata fields could not be read.")),
        Ok(_) => Err(NodeEditorApiError::message(
            "Metadata fields returned an unexpected response.",
        )),
        Err(_) => Err(NodeEditorApiError::message(
            "Could not reach the node type API.",
        )),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn create_node(payload: CreateNodePayload) -> Result<IdResponse, String> {
    let body = serde_json::to_string(&payload)
        .map_err(|_| "Create request could not be prepared.".to_string())?;

    let response = gloo_net::http::Request::post("/api/admin/nodes")
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|_| "Create request could not be prepared.".to_string())?
        .send()
        .await;

    match response {
        Ok(response) if response.status() == 401 => {
            redirect_to_login();
            Err("Authentication is required.".into())
        }
        Ok(response) if response.ok() => response
            .json::<IdResponse>()
            .await
            .map_err(|_| "Create response could not be read.".to_string()),
        Ok(response) => Err(format!("Create failed with status {}.", response.status())),
        Err(_) => Err("Could not reach the create node API.".into()),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn update_node(
    node_id: &str,
    payload: UpdateNodePayload,
) -> Result<IdResponse, String> {
    let body = serde_json::to_string(&payload)
        .map_err(|_| "Update request could not be prepared.".to_string())?;

    let response = gloo_net::http::Request::put(&format!("/api/admin/nodes/{node_id}"))
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|_| "Update request could not be prepared.".to_string())?
        .send()
        .await;

    match response {
        Ok(response) if response.status() == 401 => {
            redirect_to_login();
            Err("Authentication is required.".into())
        }
        Ok(response) if response.ok() => response
            .json::<IdResponse>()
            .await
            .map_err(|_| "Update response could not be read.".to_string()),
        Ok(response) => Err(format!("Update failed with status {}.", response.status())),
        Err(_) => Err("Could not reach the update node API.".into()),
    }
}
