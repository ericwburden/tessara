//! Edit-page loaders for the organization node editor.

#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use crate::types::{
    NodeMetadataFieldSummary, NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail,
};
use leptos::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use super::super::super::node_metadata::metadata_input_state;
#[cfg(feature = "hydrate")]
use super::super::api::{NodeEditorApiError, fetch_node_type_definition};

#[allow(clippy::too_many_arguments)]
pub(crate) fn load_organization_edit_options(
    node_id: String,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    nodes: RwSignal<Vec<OrganizationNode>>,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    selected_parent_node_id: RwSignal<String>,
    name: RwSignal<String>,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let loaded: Result<
                (
                    Vec<NodeTypeCatalogEntry>,
                    Vec<OrganizationNode>,
                    OrganizationNodeDetail,
                ),
                tessara_web_http::RequestError,
            > = async {
                let node_types =
                    tessara_web_http::fetch_json("/api/node-types", "Organization node types")
                        .await?;
                let nodes =
                    tessara_web_http::fetch_json("/api/nodes", "Organization nodes").await?;
                let detail = tessara_web_http::fetch_json(
                    &format!("/api/nodes/{node_id}"),
                    "Organization detail",
                )
                .await?;
                Ok((node_types, nodes, detail))
            }
            .await;

            match loaded {
                Ok((loaded_node_types, loaded_nodes, loaded_detail)) => {
                    match fetch_node_type_definition(&loaded_detail.node_type_id).await {
                        Ok(definition) => {
                            let (text_values, boolean_values) = metadata_input_state(
                                &definition.metadata_fields,
                                &loaded_detail.metadata,
                            );

                            selected_parent_node_id
                                .set(loaded_detail.parent_node_id.clone().unwrap_or_default());
                            name.set(loaded_detail.name.clone());
                            metadata_fields.set(definition.metadata_fields);
                            metadata_values.set(text_values);
                            metadata_booleans.set(boolean_values);
                            detail.set(Some(loaded_detail));
                            nodes.set(loaded_nodes);
                            node_types.set(loaded_node_types);
                            is_loading.set(false);
                        }
                        Err(NodeEditorApiError::Unauthorized) => {
                            is_loading.set(false);
                            redirect_to_login();
                        }
                        Err(NodeEditorApiError::Message(error)) => {
                            is_loading.set(false);
                            message.set(Some(error));
                        }
                    }
                }
                Err(error) if error.is_authentication() => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(error) => {
                    is_loading.set(false);
                    message.set(Some(error.into_message()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_id,
            node_types,
            nodes,
            detail,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_loading,
            message,
        );
    }
}
