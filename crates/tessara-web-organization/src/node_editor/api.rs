//! Transport calls for organization node editor pages.

#[cfg(feature = "hydrate")]
use crate::http::{IdResponse, send_json_request};
#[cfg(feature = "hydrate")]
use crate::types::NodeMetadataFieldSummary;
#[cfg(feature = "hydrate")]
use crate::types::{CreateNodePayload, UpdateNodePayload};

/// Node editor API failure.
#[cfg(feature = "hydrate")]
pub(super) enum NodeEditorApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
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

fn node_type_metadata_fields_path(node_type_id: &str) -> String {
    format!("/api/node-types/{node_type_id}/metadata-fields")
}

/// Fetches the scope-authorized metadata fields used by Organization node editors.
#[cfg(feature = "hydrate")]
pub(super) async fn fetch_node_type_metadata_fields(
    node_type_id: &str,
) -> Result<Vec<NodeMetadataFieldSummary>, NodeEditorApiError> {
    tessara_web_http::fetch_json(
        &node_type_metadata_fields_path(node_type_id),
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

#[cfg(test)]
mod tests {
    use super::node_type_metadata_fields_path;

    #[test]
    fn organization_editor_metadata_uses_the_scope_authorized_read_route() {
        assert_eq!(
            node_type_metadata_fields_path("node-type-1"),
            "/api/node-types/node-type-1/metadata-fields"
        );
    }
}
