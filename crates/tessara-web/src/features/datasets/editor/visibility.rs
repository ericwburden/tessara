//! Dataset editor visibility controls.

use crate::features::datasets::types::NodeResponse;
use crate::features::datasets::validation::node_matches_visibility_query;
use crate::utils::text::sentence_label;
use icons::{
    ArrowBigDownDash, ArrowBigUpDash, ChevronDown, ChevronRight, Search, Square, SquareCheckBig,
};
use leptos::prelude::*;
use std::collections::{BTreeSet, HashMap};

/// Renders the node visibility picker for dataset access.
#[component]
pub(crate) fn DatasetVisibilityEditor(
    nodes: RwSignal<Vec<NodeResponse>>,
    visibility_node_ids: RwSignal<BTreeSet<String>>,
    visibility_search: RwSignal<String>,
    expanded_node_ids: RwSignal<BTreeSet<String>>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <h3>"Visibility"</h3>
                <label class="searchable-data-table__search searchable-data-table__control">
                    <Search class="searchable-data-table__control-icon"/>
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
                            expanded_node_ids,
                            !query.trim().is_empty(),
                            query.clone(),
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
    expanded_node_ids: RwSignal<BTreeSet<String>>,
    force_expanded: bool,
    query: String,
    depth: usize,
) -> AnyView {
    let node_id_for_class = node.id.clone();
    let node_id_for_toggle = node.id.clone();
    let node_id_for_expanded = node.id.clone();
    let node_id_for_children = node.id.clone();
    let parents_node = node.clone();
    let descendants_node = node.clone();
    let all_nodes_for_parents = all_nodes.clone();
    let all_nodes_for_descendants = all_nodes.clone();
    let children = child_nodes(&visible_nodes, &node.id);
    let has_children = !children.is_empty();
    let children_for_selected_count = children.clone();
    let node_scope = vec![node.id.clone()];
    let parent_scope = node_and_parent_ids(&all_nodes_for_parents, &parents_node);
    let descendant_scope = node_and_descendant_ids(&all_nodes_for_descendants, &descendants_node);
    let node_scope_for_class = node_scope.clone();
    let node_scope_for_pressed = node_scope.clone();
    let node_scope_for_icon = node_scope.clone();
    let node_scope_for_click = node_scope.clone();
    let parent_scope_for_class = parent_scope.clone();
    let parent_scope_for_pressed = parent_scope.clone();
    let parent_scope_for_click = parent_scope.clone();
    let descendant_scope_for_class = descendant_scope.clone();
    let descendant_scope_for_pressed = descendant_scope.clone();
    let descendant_scope_for_click = descendant_scope.clone();
    let is_search_match = !query.trim().is_empty() && node_matches_visibility_query(&node, &query);
    view! {
        <section class="dataset-visibility-branch" style=format!("--visibility-depth: {depth};")>
            <div class=move || {
                visibility_node_class(
                    visibility_node_ids.get().contains(&node_id_for_class),
                    is_search_match,
                )
            }>
                <button
                    class="dataset-visibility-node__main"
                    type="button"
                    aria-expanded=move || {
                        (has_children && (force_expanded || expanded_node_ids.get().contains(&node_id_for_expanded))).to_string()
                    }
                    disabled=!has_children
                    on:click=move |_| {
                        if has_children {
                            expanded_node_ids.update(|ids| {
                                if ids.contains(&node_id_for_toggle) {
                                    ids.remove(&node_id_for_toggle);
                                } else {
                                    ids.insert(node_id_for_toggle.clone());
                                }
                            });
                        }
                    }
                >
                    <span class="dataset-visibility-node__copy">
                        <strong>{node.name.clone()}</strong>
                        <span>{format!(
                            "{} · {}",
                            sentence_label(&node.node_type_name),
                            node.parent_node_name.clone().unwrap_or_else(|| "Top-level".into()),
                        )}</span>
                    </span>
                    <span class="dataset-visibility-node__count">
                        {move || {
                            let selected_count = selected_direct_child_count(
                                &children_for_selected_count,
                                &visibility_node_ids.get(),
                            );
                            if selected_count == 0 {
                                "No selected children".to_string()
                            } else if selected_count == 1 {
                                "1 selected child".to_string()
                            } else {
                                format!("{selected_count} selected children")
                            }
                        }}
                    </span>
                    <span class="dataset-visibility-node__toggle" aria-hidden="true">
                        {move || if has_children {
                            if force_expanded || expanded_node_ids.get().contains(&node.id) {
                                view! { <ChevronDown class="dataset-visibility-node__toggle-icon"/> }.into_any()
                            } else {
                                view! { <ChevronRight class="dataset-visibility-node__toggle-icon"/> }.into_any()
                            }
                        } else {
                            view! { <span class="dataset-visibility-node__toggle-placeholder"></span> }.into_any()
                        }}
                    </span>
                </button>
                <span class="dataset-visibility-node__actions">
                    <button
                        class=move || visibility_action_class(
                            scope_is_selected(&visibility_node_ids.get(), &node_scope_for_class)
                        )
                        type="button"
                        aria-label="Toggle this node"
                        aria-pressed=move || scope_is_selected(&visibility_node_ids.get(), &node_scope_for_pressed).to_string()
                        title="Toggle this node"
                        on:click=move |_| {
                            visibility_node_ids.update(|selected| toggle_visibility_scope(selected, &node_scope_for_click));
                        }
                    >
                        {move || {
                            if scope_is_selected(&visibility_node_ids.get(), &node_scope_for_icon) {
                                view! { <SquareCheckBig class="icon-button__icon"/> }.into_any()
                            } else {
                                view! { <Square class="icon-button__icon"/> }.into_any()
                            }
                        }}
                    </button>
                    <button
                        class=move || visibility_action_class(
                            scope_is_selected(&visibility_node_ids.get(), &parent_scope_for_class)
                        )
                        type="button"
                        aria-label="Toggle node and parents"
                        aria-pressed=move || scope_is_selected(&visibility_node_ids.get(), &parent_scope_for_pressed).to_string()
                        title="Toggle node and parents"
                        on:click=move |_| {
                            visibility_node_ids.update(|selected| toggle_visibility_scope(selected, &parent_scope_for_click));
                        }
                    >
                        <ArrowBigUpDash class="icon-button__icon"/>
                    </button>
                    <button
                        class=move || visibility_action_class(
                            scope_is_selected(&visibility_node_ids.get(), &descendant_scope_for_class)
                        )
                        type="button"
                        aria-label="Toggle node and descendants"
                        aria-pressed=move || scope_is_selected(&visibility_node_ids.get(), &descendant_scope_for_pressed).to_string()
                        title="Toggle node and descendants"
                        on:click=move |_| {
                            visibility_node_ids.update(|selected| toggle_visibility_scope(selected, &descendant_scope_for_click));
                        }
                    >
                        <ArrowBigDownDash class="icon-button__icon"/>
                    </button>
                </span>
            </div>
            <Show when=move || has_children && (force_expanded || expanded_node_ids.get().contains(&node_id_for_children))>
                <div class="dataset-visibility-children" role="group">
                    {children.clone().into_iter().map(|child| {
                        visibility_tree_branch(
                            child,
                            visible_nodes.clone(),
                            all_nodes.clone(),
                            visibility_node_ids,
                            expanded_node_ids,
                            force_expanded,
                            query.clone(),
                            depth + 1,
                        )
                    }).collect_view()}
                </div>
            </Show>
        </section>
    }
    .into_any()
}

fn visibility_node_class(is_selected: bool, is_search_match: bool) -> &'static str {
    match (is_selected, is_search_match) {
        (true, true) => "dataset-visibility-node is-selected is-search-match",
        (true, false) => "dataset-visibility-node is-selected",
        (false, true) => "dataset-visibility-node is-search-match",
        (false, false) => "dataset-visibility-node",
    }
}

fn visibility_action_class(scope_selected: bool) -> &'static str {
    if scope_selected {
        "icon-button icon-button--danger dataset-visibility-node-action"
    } else {
        "icon-button dataset-visibility-node-action"
    }
}

fn scope_is_selected(selected: &BTreeSet<String>, scope: &[String]) -> bool {
    !scope.is_empty() && scope.iter().all(|id| selected.contains(id))
}

fn toggle_visibility_scope(selected: &mut BTreeSet<String>, scope: &[String]) {
    if scope_is_selected(selected, scope) {
        for id in scope {
            selected.remove(id);
        }
    } else {
        for id in scope {
            selected.insert(id.clone());
        }
    }
}

fn selected_direct_child_count(children: &[NodeResponse], selected: &BTreeSet<String>) -> usize {
    children
        .iter()
        .filter(|child| selected.contains(&child.id))
        .count()
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
