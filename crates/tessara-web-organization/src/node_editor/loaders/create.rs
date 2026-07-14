//! Create-page loaders for the organization node editor.

#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use crate::types::{NodeTypeCatalogEntry, OrganizationNode};
#[cfg(feature = "hydrate")]
use crate::url::current_search_param;
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

            let loaded: Result<
                (Vec<NodeTypeCatalogEntry>, Vec<OrganizationNode>),
                tessara_web_http::RequestError,
            > = async {
                let node_types =
                    tessara_web_http::fetch_json("/api/node-types", "Organization node types")
                        .await?;
                let nodes =
                    tessara_web_http::fetch_json("/api/nodes", "Organization nodes").await?;
                Ok((node_types, nodes))
            }
            .await;

            match loaded {
                Ok((loaded_node_types, loaded_nodes)) => {
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
            node_types,
            nodes,
            selected_node_type_id,
            selected_parent_node_id,
            is_loading,
            message,
        );
    }
}
