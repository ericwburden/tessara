//! Organization tree loading and rendering helpers.
//!
//! Keep hierarchy construction, branch expansion, tree navigation, and node detail loading here.

use crate::features::organization::types::{
    CreateChildLink, NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail,
    OrganizationTreeNode,
};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use crate::ui::DropdownMenu;
use icons::{ChevronDown, ChevronRight, PanelRight, Pencil, Plus};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

/// Toggles the toggle organization branch state.
pub(crate) fn toggle_organization_branch(
    expanded_nodes: RwSignal<HashSet<String>>,
    node_id: String,
    lineage: Vec<String>,
) {
    expanded_nodes.update(|nodes| {
        let was_open = nodes.contains(&node_id);
        let lineage: HashSet<String> = lineage.into_iter().collect();

        nodes.retain(|open_id| lineage.contains(open_id));

        if !was_open {
            nodes.insert(node_id);
        }
    });
}

/// Loads the load organization tree data.
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

            let node_response = gloo_net::http::Request::get("/api/nodes").send().await;
            let node_type_response = gloo_net::http::Request::get("/api/node-types").send().await;

            match (node_response, node_type_response) {
                (Ok(response), _) if response.status() == 401 => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response)) if response.status() == 401 => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_response), Ok(node_type_response))
                    if node_response.ok() && node_type_response.ok() =>
                {
                    let loaded_nodes = node_response.json::<Vec<OrganizationNode>>().await;
                    let loaded_node_types =
                        node_type_response.json::<Vec<NodeTypeCatalogEntry>>().await;

                    match (loaded_nodes, loaded_node_types) {
                        (Ok(nodes), Ok(loaded_node_types)) => {
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
                        _ => {
                            tree.set(Vec::new());
                            node_types.set(Vec::new());
                            load_error
                                .set(Some("The hierarchy response could not be read.".into()));
                            is_loading.set(false);
                        }
                    }
                }
                (Ok(_), Ok(_)) => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    load_error.set(Some(
                        "The hierarchy API returned an unexpected response.".into(),
                    ));
                    is_loading.set(false);
                }
                _ => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    load_error.set(Some("Could not reach the hierarchy API.".into()));
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

/// Loads the load organization detail data.
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

            let response = gloo_net::http::Request::get(&format!("/api/nodes/{node_id}"))
                .send()
                .await;

            match response {
                Ok(response) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<OrganizationNodeDetail>().await {
                        Ok(payload) => {
                            detail.set(Some(payload));
                            is_loading.set(false);
                        }
                        Err(_) => {
                            error.set(Some("The detail response could not be read.".into()));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(_) => {
                    error.set(Some(
                        "The detail API returned an unexpected response.".into(),
                    ));
                    is_loading.set(false);
                }
                Err(_) => {
                    error.set(Some("Could not reach the detail API.".into()));
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

/// Builds the build organization tree value.
pub(crate) fn build_organization_tree(nodes: Vec<OrganizationNode>) -> Vec<OrganizationTreeNode> {
    let visible_ids = nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>();
    let mut children_by_parent = HashMap::<Option<String>, Vec<OrganizationNode>>::new();

    for node in nodes {
        let parent_id = node
            .parent_node_id
            .clone()
            .filter(|parent_id| visible_ids.contains(parent_id));
        children_by_parent.entry(parent_id).or_default().push(node);
    }

    for siblings in children_by_parent.values_mut() {
        siblings.sort_by(|left, right| {
            left.node_type_name
                .cmp(&right.node_type_name)
                .then(left.name.cmp(&right.name))
        });
    }

    build_organization_branches(None, &mut children_by_parent)
}

/// Builds the build organization branches value.
pub(crate) fn build_organization_branches(
    parent_id: Option<String>,
    children_by_parent: &mut HashMap<Option<String>, Vec<OrganizationNode>>,
) -> Vec<OrganizationTreeNode> {
    children_by_parent
        .remove(&parent_id)
        .unwrap_or_default()
        .into_iter()
        .map(|node| {
            let children = build_organization_branches(Some(node.id.clone()), children_by_parent);
            OrganizationTreeNode { node, children }
        })
        .collect()
}

/// Handles the child create links behavior.
pub(crate) fn child_create_links(
    parent_node_type_id: &str,
    node_types: &[NodeTypeCatalogEntry],
    parent_node_id: &str,
) -> Vec<CreateChildLink> {
    let Some(parent_type) = node_types
        .iter()
        .find(|node_type| node_type.id == parent_node_type_id)
    else {
        return Vec::new();
    };

    parent_type
        .child_relationships
        .iter()
        .map(|relationship| CreateChildLink {
            href: format!(
                "/organization/new?parent_node_id={parent_node_id}&node_type_id={}",
                relationship.node_type_id
            ),
            label: format!("Create {}", relationship.singular_label),
        })
        .collect()
}

/// Handles the organization tree view behavior.
pub(crate) fn organization_tree_view(
    nodes: Vec<OrganizationTreeNode>,
    node_types: Vec<NodeTypeCatalogEntry>,
    expanded_nodes: RwSignal<HashSet<String>>,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    detail_is_loading: RwSignal<bool>,
    detail_error: RwSignal<Option<String>>,
    depth: usize,
    lineage: Vec<String>,
) -> AnyView {
    view! {
        <div class="organization-tree" role=if depth == 0 { "tree" } else { "group" }>
            {nodes
                .into_iter()
                .map(|branch| {
                    organization_branch_view(
                        branch,
                        node_types.clone(),
                        expanded_nodes,
                        detail,
                        detail_is_loading,
                        detail_error,
                        depth,
                        lineage.clone(),
                    )
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

/// Handles the organization branch view behavior.
pub(crate) fn organization_branch_view(
    branch: OrganizationTreeNode,
    node_types: Vec<NodeTypeCatalogEntry>,
    expanded_nodes: RwSignal<HashSet<String>>,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    detail_is_loading: RwSignal<bool>,
    detail_error: RwSignal<Option<String>>,
    depth: usize,
    lineage: Vec<String>,
) -> AnyView {
    let node = branch.node;
    let children = branch.children;
    let node_id = node.id.clone();
    let row_id = node.id.clone();
    let row_class_id = node.id.clone();
    let child_link_node_type_id = node.node_type_id.clone();
    let expanded_id = node.id.clone();
    let toggle_icon_id = node.id.clone();
    let child_visibility_id = node.id.clone();
    let details_id = node.id.clone();
    let action_label = format!("Open actions for {}", node.name);
    let has_children = !children.is_empty();
    let child_count = children.len();
    let child_lineage = {
        let mut next_lineage = lineage.clone();
        next_lineage.push(node.id.clone());
        next_lineage
    };
    let row_class = move || {
        if has_children && expanded_nodes.with(|nodes| nodes.contains(&row_class_id)) {
            "organization-node is-open"
        } else {
            "organization-node"
        }
    };
    let edit_href = format!("/organization/{node_id}/edit");
    let create_links = child_create_links(&child_link_node_type_id, &node_types, &node_id);
    let child_label = visible_child_label(child_count);

    view! {
        <section class="organization-branch" style=format!("--organization-depth: {depth};")>
            <div class=row_class>
                <button
                    class="organization-node__main"
                    type="button"
                    aria-expanded=move || {
                        (has_children && expanded_nodes.with(|nodes| nodes.contains(&expanded_id))).to_string()
                    }
                    on:click=move |_| {
                        if has_children {
                            toggle_organization_branch(
                                expanded_nodes,
                                row_id.clone(),
                                lineage.clone(),
                            );
                        }
                    }
                >
                    <span class="organization-node__toggle" aria-hidden="true">
                        {move || {
                            if has_children && expanded_nodes.with(|nodes| nodes.contains(&toggle_icon_id)) {
                                view! { <ChevronDown class="organization-node__toggle-icon"/> }.into_any()
                            } else {
                                view! { <ChevronRight class="organization-node__toggle-icon"/> }.into_any()
                            }
                        }}
                    </span>
                    <span class="organization-node__copy">
                        <span class="organization-node__type">{node.node_type_singular_label}</span>
                        <strong>{node.name}</strong>
                        <span class="organization-node__context">
                            {node.parent_node_name.unwrap_or_else(|| "Top-level".to_string())}
                        </span>
                    </span>
                    <span class="organization-node__count">{child_label}</span>
                </button>
                <DropdownMenu label=action_label>
                    <button
                        class="dropdown-menu__item"
                        type="button"
                        role="menuitem"
                        on:click=move |_| {
                            load_organization_detail(
                                details_id.clone(),
                                detail,
                                detail_is_loading,
                                detail_error,
                            );
                        }
                    >
                        <PanelRight class="dropdown-menu__item-icon"/>
                        <span>"Details"</span>
                    </button>
                    <a class="dropdown-menu__item" role="menuitem" href=edit_href>
                        <Pencil class="dropdown-menu__item-icon"/>
                        <span>"Edit"</span>
                    </a>
                    {create_links
                        .into_iter()
                        .map(|link| {
                            view! {
                                <a class="dropdown-menu__item" role="menuitem" href=link.href>
                                    <Plus class="dropdown-menu__item-icon"/>
                                    <span>{link.label}</span>
                                </a>
                            }
                        })
                        .collect_view()}
                </DropdownMenu>
            </div>

            <Show when=move || has_children && expanded_nodes.with(|nodes| nodes.contains(&child_visibility_id))>
                {organization_tree_view(
                    children.clone(),
                    node_types.clone(),
                    expanded_nodes,
                    detail,
                    detail_is_loading,
                    detail_error,
                    depth + 1,
                    child_lineage.clone(),
                )}
            </Show>
        </section>
    }
    .into_any()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the visible child label behavior.
pub(crate) fn visible_child_label(count: usize) -> String {
    match count {
        0 => "No visible children".to_string(),
        1 => "1 visible child".to_string(),
        count => format!("{count} visible children"),
    }
}
