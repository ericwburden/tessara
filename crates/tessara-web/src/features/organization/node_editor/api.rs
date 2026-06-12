//! Transport calls for organization node editor pages.

use crate::features::organization::types::NodeTypeDefinition;

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
