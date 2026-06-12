//! Edit-page loaders for the organization node editor.

use crate::features::organization::types::{
    NodeMetadataFieldSummary, NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail,
};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
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

            let node_type_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let node_response = gloo_net::http::Request::get("/api/nodes").send().await;
            let detail_response = gloo_net::http::Request::get(&format!("/api/nodes/{node_id}"))
                .send()
                .await;

            match (node_type_response, node_response, detail_response) {
                (Ok(response), _, _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response), _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, Ok(response)) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_type_response), Ok(node_response), Ok(detail_response))
                    if node_type_response.ok() && node_response.ok() && detail_response.ok() =>
                {
                    let loaded_node_types =
                        node_type_response.json::<Vec<NodeTypeCatalogEntry>>().await;
                    let loaded_nodes = node_response.json::<Vec<OrganizationNode>>().await;
                    let loaded_detail = detail_response.json::<OrganizationNodeDetail>().await;

                    match (loaded_node_types, loaded_nodes, loaded_detail) {
                        (Ok(loaded_node_types), Ok(loaded_nodes), Ok(loaded_detail)) => {
                            match fetch_node_type_definition(&loaded_detail.node_type_id).await {
                                Ok(definition) => {
                                    let (text_values, boolean_values) = metadata_input_state(
                                        &definition.metadata_fields,
                                        &loaded_detail.metadata,
                                    );

                                    selected_parent_node_id.set(
                                        loaded_detail.parent_node_id.clone().unwrap_or_default(),
                                    );
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
                        _ => {
                            is_loading.set(false);
                            message.set(Some("Edit options could not be read.".into()));
                        }
                    }
                }
                (Ok(_), Ok(_), Ok(_)) => {
                    is_loading.set(false);
                    message.set(Some("Edit options returned an unexpected response.".into()));
                }
                _ => {
                    is_loading.set(false);
                    message.set(Some("Could not reach the organization APIs.".into()));
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
