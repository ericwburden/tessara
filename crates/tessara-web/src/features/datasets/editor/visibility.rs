//! Dataset editor visibility controls.

use crate::features::datasets::types::NodeResponse;
use crate::features::datasets::validation::node_matches_visibility_query;
use crate::ui::DataTable;
use crate::utils::text::sentence_label;
use icons::Search;
use leptos::prelude::*;
use std::collections::BTreeSet;

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
            <div class="table-wrap dataset-visibility-table">
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Visible"</th>
                            <th scope="col">"Node"</th>
                            <th scope="col">"Type"</th>
                            <th scope="col">"Parent"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let query = visibility_search.get();
                            let mut visible_nodes = nodes.get();
                            visible_nodes.sort_by(|left, right| {
                                left.node_type_name
                                    .cmp(&right.node_type_name)
                                    .then_with(|| left.parent_node_name.cmp(&right.parent_node_name))
                                    .then_with(|| left.name.cmp(&right.name))
                            });
                            visible_nodes
                                .into_iter()
                                .filter(|node| node_matches_visibility_query(node, &query))
                                .map(|node| {
                                    let node_id = node.id.clone();
                                    let checked = visibility_node_ids.get().contains(&node_id);
                                    view! {
                                        <tr>
                                            <td>
                                                <input
                                                    type="checkbox"
                                                    checked=checked
                                                    aria-label=format!("Toggle visibility for {}", node.name)
                                                    on:change=move |event| {
                                                        let is_checked = event_target_checked(&event);
                                                        visibility_node_ids.update(|ids| {
                                                            if is_checked {
                                                                ids.insert(node_id.clone());
                                                            } else {
                                                                ids.remove(&node_id);
                                                            }
                                                        });
                                                    }
                                                />
                                            </td>
                                            <th scope="row">{node.name}</th>
                                            <td>{sentence_label(&node.node_type_name)}</td>
                                            <td>{node.parent_node_name.unwrap_or_else(|| "Top-level".into())}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                        }}
                    </tbody>
                </DataTable>
            </div>
        </section>
    }
}
