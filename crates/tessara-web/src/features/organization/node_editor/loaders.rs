//! Signal-aware loaders for organization node editor pages.

#[cfg(feature = "hydrate")]
use crate::features::organization::types::NodeTypeDefinition;
use crate::features::organization::types::{
    NodeMetadataFieldSummary, NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail,
};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;
use leptos::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use super::super::node_metadata::metadata_input_state;
#[cfg(feature = "hydrate")]
use super::super::node_options::available_node_types_for_parent;

/// Loads node types and visible nodes for the create-node page.
pub(crate) fn load_organization_create_options(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    nodes: RwSignal<Vec<OrganizationNode>>,
    selected_node_type_id: RwSignal<String>,
    selected_parent_node_id: RwSignal<String>,
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

            match (node_type_response, node_response) {
                (Ok(response), _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response)) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_type_response), Ok(node_response))
                    if node_type_response.ok() && node_response.ok() =>
                {
                    let loaded_node_types =
                        node_type_response.json::<Vec<NodeTypeCatalogEntry>>().await;
                    let loaded_nodes = node_response.json::<Vec<OrganizationNode>>().await;

                    match (loaded_node_types, loaded_nodes) {
                        (Ok(loaded_node_types), Ok(loaded_nodes)) => {
                            let requested_node_type_id = current_search_param("node_type_id");
                            let requested_parent_id = current_search_param("parent_node_id")
                                .or_else(|| current_search_param("parent_id"));
                            let selected_parent = requested_parent_id
                                .filter(|requested| {
                                    loaded_nodes.iter().any(|node| node.id == *requested)
                                })
                                .unwrap_or_default();
                            let available_types = available_node_types_for_parent(
                                &selected_parent,
                                &loaded_node_types,
                                &loaded_nodes,
                            );
                            let selected_type = requested_node_type_id
                                .filter(|requested| {
                                    available_types
                                        .iter()
                                        .any(|node_type| node_type.id == *requested)
                                })
                                .or_else(|| {
                                    available_types
                                        .first()
                                        .map(|node_type| node_type.id.clone())
                                });

                            nodes.set(loaded_nodes);
                            node_types.set(loaded_node_types);
                            selected_node_type_id.set(selected_type.unwrap_or_default());
                            selected_parent_node_id.set(selected_parent);
                            is_loading.set(false);
                        }
                        _ => {
                            is_loading.set(false);
                            message.set(Some("Create options could not be read.".into()));
                        }
                    }
                }
                (Ok(_), Ok(_)) => {
                    is_loading.set(false);
                    message.set(Some(
                        "Create options returned an unexpected response.".into(),
                    ));
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
            node_types,
            nodes,
            selected_node_type_id,
            selected_parent_node_id,
            is_loading,
            message,
        );
    }
}

/// Loads metadata field definitions for the selected node type.
pub(crate) fn load_node_type_metadata(
    node_type_id: String,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            let response =
                gloo_net::http::Request::get(&format!("/api/admin/node-types/{node_type_id}"))
                    .send()
                    .await;

            match response {
                Ok(response) if response.status() == 401 => redirect_to_login(),
                Ok(response) if response.ok() => {
                    match response.json::<NodeTypeDefinition>().await {
                        Ok(definition) => {
                            metadata_fields.set(definition.metadata_fields);
                            metadata_values.set(HashMap::new());
                            metadata_booleans.set(HashMap::new());
                        }
                        Err(_) => {
                            metadata_fields.set(Vec::new());
                            message.set(Some("Metadata fields could not be read.".into()));
                        }
                    }
                }
                Ok(_) => {
                    metadata_fields.set(Vec::new());
                    message.set(Some(
                        "Metadata fields returned an unexpected response.".into(),
                    ));
                }
                Err(_) => {
                    metadata_fields.set(Vec::new());
                    message.set(Some("Could not reach the node type API.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_type_id,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            message,
        );
    }
}

/// Loads the node detail, editable metadata state, and parent/type options.
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
                            let metadata_response = gloo_net::http::Request::get(&format!(
                                "/api/admin/node-types/{}",
                                loaded_detail.node_type_id
                            ))
                            .send()
                            .await;

                            match metadata_response {
                                Ok(response) if response.status() == 401 => {
                                    is_loading.set(false);
                                    redirect_to_login();
                                }
                                Ok(response) if response.ok() => {
                                    match response.json::<NodeTypeDefinition>().await {
                                        Ok(definition) => {
                                            let (text_values, boolean_values) =
                                                metadata_input_state(
                                                    &definition.metadata_fields,
                                                    &loaded_detail.metadata,
                                                );

                                            selected_parent_node_id.set(
                                                loaded_detail
                                                    .parent_node_id
                                                    .clone()
                                                    .unwrap_or_default(),
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
                                        Err(_) => {
                                            is_loading.set(false);
                                            message.set(Some(
                                                "Metadata fields could not be read.".into(),
                                            ));
                                        }
                                    }
                                }
                                Ok(_) => {
                                    is_loading.set(false);
                                    message.set(Some(
                                        "Metadata fields returned an unexpected response.".into(),
                                    ));
                                }
                                Err(_) => {
                                    is_loading.set(false);
                                    message.set(Some("Could not reach the node type API.".into()));
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
