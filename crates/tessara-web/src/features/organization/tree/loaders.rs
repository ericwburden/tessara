//! Organization tree and detail loaders.

#[cfg(feature = "hydrate")]
use super::api::{fetch_organization_detail, fetch_tree_context};
#[cfg(feature = "hydrate")]
use super::build_organization_tree;
#[cfg(feature = "hydrate")]
use super::errors::OrganizationTreeApiError;
use crate::features::organization::types::{
    NodeTypeCatalogEntry, OrganizationNodeDetail, OrganizationTreeNode,
};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;
use std::collections::HashSet;

pub(crate) fn load_organization_tree(
    tree: RwSignal<Vec<OrganizationTreeNode>>,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    expanded_nodes: RwSignal<HashSet<String>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_tree_context().await {
                Ok((nodes, loaded_node_types)) => {
                    let branches = build_organization_tree(nodes);
                    let first_open = branches
                        .iter()
                        .find(|branch| !branch.children.is_empty())
                        .map(|branch| branch.node.id.clone());

                    expanded_nodes.set(first_open.into_iter().collect());
                    tree.set(branches);
                    node_types.set(loaded_node_types);
                    is_loading.set(false);
                }
                Err(OrganizationTreeApiError::Unauthorized) => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(OrganizationTreeApiError::Message(error)) => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (tree, node_types, expanded_nodes, is_loading, load_error);
    }
}

pub(crate) fn load_organization_detail(
    node_id: String,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    is_loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            error.set(None);

            match fetch_organization_detail(&node_id).await {
                Ok(payload) => {
                    detail.set(Some(payload));
                    is_loading.set(false);
                }
                Err(OrganizationTreeApiError::Unauthorized) => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(OrganizationTreeApiError::Message(message)) => {
                    error.set(Some(message));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (node_id, detail, is_loading, error);
    }
}
