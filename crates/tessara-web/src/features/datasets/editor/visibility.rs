//! Dataset editor visibility controls.

use crate::features::datasets::types::NodeResponse;
use crate::features::datasets::validation::node_matches_visibility_query;
use crate::utils::text::sentence_label;
use icons::Search;
use leptos::prelude::*;
use std::collections::{BTreeSet, HashMap};

/// Renders the node visibility picker for dataset access.
#[component]
pub(crate) fn DatasetVisibilityEditor(
    nodes: RwSignal<Vec<NodeResponse>>,
    visibility_node_ids: RwSignal<BTreeSet<String>>,
    visibility_search: RwSignal<String>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <h3>"Visibility"</h3>
                <label class="searchable-data-table__search">
                    <Search class="searchable-data-table__search-icon"/>
                    <span class="sr-only">"Search visibility nodes"</span>
                    <input
                        type="search"
                        placeholder="Search nodes"
                        prop:value=move || visibility_search.get()
                        on:input=move |event| visibility_search.set(event_target_value(&event))
                    />
                </label>
            </div>
            <div class="dataset-visibility-tree" role="tree">
                {move || {
                    let query = visibility_search.get();
                    let all_nodes = sorted_nodes(nodes.get());
                    let visible_nodes = visible_tree_nodes(&all_nodes, &query);
                    let roots = root_nodes(&visible_nodes);
                    roots.into_iter().map(|node| {
                        visibility_tree_branch(
                            node,
                            visible_nodes.clone(),
                            all_nodes.clone(),
                            visibility_node_ids,
                            0,
                        )
                    }).collect_view()
                }}
            </div>
        </section>
    }
}

fn visibility_tree_branch(
    node: NodeResponse,
    visible_nodes: Vec<NodeResponse>,
    all_nodes: Vec<NodeResponse>,
    visibility_node_ids: RwSignal<BTreeSet<String>>,
    depth: usize,
) -> AnyView {
    let node_id = node.id.clone();
    let node_id_for_class = node.id.clone();
    let node_id_for_pressed = node.id.clone();
    let parents_node = node.clone();
    let descendants_node = node.clone();
    let all_nodes_for_parents = all_nodes.clone();
    let all_nodes_for_descendants = all_nodes.clone();
    let children = child_nodes(&visible_nodes, &node.id);
    view! {
        <section class="dataset-visibility-branch" style=format!("--visibility-depth: {depth};")>
            <div class=move || {
                if visibility_node_ids.get().contains(&node_id_for_class) {
                    "dataset-visibility-node is-selected"
                } else {
                    "dataset-visibility-node"
                }
            }>
                <span class="dataset-visibility-node__copy">
                    <strong>{node.name.clone()}</strong>
                    <span>{format!(
                        "{} · {}",
                        sentence_label(&node.node_type_name),
                        node.parent_node_name.clone().unwrap_or_else(|| "Top-level".into()),
                    )}</span>
                </span>
                <span class="dataset-visibility-node__actions">
                    <button
                        class="button button--secondary button--compact"
                        type="button"
                        aria-pressed=move || visibility_node_ids.get().contains(&node_id_for_pressed).to_string()
                        on:click=move |_| {
                            visibility_node_ids.update(|ids| {
                                if ids.contains(&node_id) {
                                    ids.remove(&node_id);
                                } else {
                                    ids.insert(node_id.clone());
                                }
                            });
                        }
                    >
                        "Node"
                    </button>
                    <button
                        class="button button--secondary button--compact"
                        type="button"
                        on:click=move |_| {
                            let ids = node_and_parent_ids(&all_nodes_for_parents, &parents_node);
                            visibility_node_ids.update(|selected| {
                                for id in ids {
                                    selected.insert(id);
                                }
                            });
                        }
                    >
                        "+ Parents"
                    </button>
                    <button
                        class="button button--secondary button--compact"
                        type="button"
                        on:click=move |_| {
                            let ids = node_and_descendant_ids(&all_nodes_for_descendants, &descendants_node);
                            visibility_node_ids.update(|selected| {
                                for id in ids {
                                    selected.insert(id);
                                }
                            });
                        }
                    >
                        "+ Descendants"
                    </button>
                </span>
            </div>
            <div class="dataset-visibility-children" role="group">
                {children.into_iter().map(|child| {
                    visibility_tree_branch(
                        child,
                        visible_nodes.clone(),
                        all_nodes.clone(),
                        visibility_node_ids,
                        depth + 1,
                    )
                }).collect_view()}
            </div>
        </section>
    }
    .into_any()
}

fn sorted_nodes(mut nodes: Vec<NodeResponse>) -> Vec<NodeResponse> {
    nodes.sort_by(|left, right| {
        left.parent_node_id
            .cmp(&right.parent_node_id)
            .then_with(|| left.node_type_name.cmp(&right.node_type_name))
            .then_with(|| left.name.cmp(&right.name))
    });
    nodes
}

fn visible_tree_nodes(nodes: &[NodeResponse], query: &str) -> Vec<NodeResponse> {
    if query.trim().is_empty() {
        return nodes.to_vec();
    }

    let nodes_by_id = nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<HashMap<_, _>>();
    let mut visible_ids = BTreeSet::new();
    for node in nodes {
        if node_matches_visibility_query(node, query) {
            visible_ids.insert(node.id.clone());
            let mut current = node;
            while let Some(parent) = current
                .parent_node_id
                .as_deref()
                .and_then(|parent_id| nodes_by_id.get(parent_id).copied())
            {
                visible_ids.insert(parent.id.clone());
                current = parent;
            }
        }
    }

    nodes
        .iter()
        .filter(|node| visible_ids.contains(&node.id))
        .cloned()
        .collect()
}

fn root_nodes(nodes: &[NodeResponse]) -> Vec<NodeResponse> {
    nodes
        .iter()
        .filter(|node| {
            node.parent_node_id
                .as_deref()
                .is_none_or(|parent_id| !nodes.iter().any(|candidate| candidate.id == parent_id))
        })
        .cloned()
        .collect()
}

fn child_nodes(nodes: &[NodeResponse], parent_id: &str) -> Vec<NodeResponse> {
    nodes
        .iter()
        .filter(|node| node.parent_node_id.as_deref() == Some(parent_id))
        .cloned()
        .collect()
}

fn node_and_parent_ids(nodes: &[NodeResponse], node: &NodeResponse) -> Vec<String> {
    let nodes_by_id = nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<HashMap<_, _>>();
    let mut ids = vec![node.id.clone()];
    let mut current = node;
    while let Some(parent) = current
        .parent_node_id
        .as_deref()
        .and_then(|parent_id| nodes_by_id.get(parent_id).copied())
    {
        ids.push(parent.id.clone());
        current = parent;
    }
    ids
}

fn node_and_descendant_ids(nodes: &[NodeResponse], node: &NodeResponse) -> Vec<String> {
    let mut ids = vec![node.id.clone()];
    for child in child_nodes(nodes, &node.id) {
        ids.extend(node_and_descendant_ids(nodes, &child));
    }
    ids
}
