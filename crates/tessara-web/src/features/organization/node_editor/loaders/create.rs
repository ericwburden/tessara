//! Create-page loaders for the organization node editor.

use crate::features::organization::types::{NodeTypeCatalogEntry, OrganizationNode};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::super::options::organization_create_selection;

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
                            let selection = organization_create_selection(
                                requested_node_type_id,
                                requested_parent_id,
                                &loaded_node_types,
                                &loaded_nodes,
                            );

                            nodes.set(loaded_nodes);
                            node_types.set(loaded_node_types);
                            selected_node_type_id.set(selection.node_type_id);
                            selected_parent_node_id.set(selection.parent_node_id);
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
