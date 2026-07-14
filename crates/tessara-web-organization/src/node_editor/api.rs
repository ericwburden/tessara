//! Transport calls for organization node editor pages.

#[cfg(feature = "hydrate")]
use crate::http::{IdResponse, send_json_request};
#[cfg(feature = "hydrate")]
use crate::types::NodeTypeDefinition;
#[cfg(feature = "hydrate")]
use crate::types::{CreateNodePayload, UpdateNodePayload};

/// Node editor API failure.
pub(super) enum NodeEditorApiError {
    Unauthorized,
    Message(String),
}

impl NodeEditorApiError {
    #[cfg(feature = "hydrate")]
    fn from_request(error: tessara_web_http::RequestError) -> Self {
        if error.is_authentication() {
            Self::Unauthorized
        } else {
            Self::Message(error.into_message())
        }
    }
}

/// Fetches the full node type definition, including metadata fields.
#[cfg(feature = "hydrate")]
pub(super) async fn fetch_node_type_definition(
    node_type_id: &str,
) -> Result<NodeTypeDefinition, NodeEditorApiError> {
    tessara_web_http::fetch_json(
        &format!("/api/admin/node-types/{node_type_id}"),
        "Node type metadata fields",
    )
    .await
    .map_err(NodeEditorApiError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn create_node(payload: CreateNodePayload) -> Result<IdResponse, String> {
    send_json_request(
        gloo_net::http::Request::post("/api/admin/nodes"),
        &payload,
        "Create node",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn update_node(
    node_id: &str,
    payload: UpdateNodePayload,
) -> Result<IdResponse, String> {
    send_json_request(
        gloo_net::http::Request::put(&format!("/api/admin/nodes/{node_id}")),
        &payload,
        "Update node",
    )
    .await
}
