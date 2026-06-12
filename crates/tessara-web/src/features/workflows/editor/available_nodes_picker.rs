//! Available-node picker component for workflow editing.

use crate::features::organization::OrganizationNode;
use crate::features::shared::node_display_path;
use icons::Search;
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub(in crate::features::workflows) fn WorkflowAvailableNodesPicker(
    nodes: Vec<OrganizationNode>,
    selected_node_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let filtered_nodes = move || {
        let query = search.get();
        nodes
            .clone()
            .into_iter()
            .filter(|node| {
                crate::utils::text::text_matches(
                    &query,
                    &[
                        node.name.as_str(),
                        node.node_type_singular_label.as_str(),
                        node.parent_node_name.as_deref().unwrap_or(""),
                    ],
                )
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="permission-picker workflow-available-node-picker">
            <label class="searchable-data-table__search searchable-data-table__control">
                <Search class="searchable-data-table__control-icon"/>
                <span class="sr-only">"Search available nodes"</span>
                <input
                    type="search"
                    placeholder="Search available nodes"
                    prop:value=move || search.get()
                    on:input=move |event| search.set(event_target_value(&event))
                />
            </label>
            <div class="checkbox-list permission-picker__list permission-picker__list--compact">
                {move || {
                    let nodes = filtered_nodes();
                    if nodes.is_empty() {
                        return view! {
                            <section class="organization-state">
                                <h3>"No nodes found"</h3>
                                <p>"Adjust the search to choose where this workflow is available."</p>
                            </section>
                        }
                        .into_any();
                    }

                    nodes
                        .into_iter()
                        .map(|node| {
                            let node_id = node.id.clone();
                            let node_id_for_checked = node_id.clone();
                            let node_id_for_change = node_id.clone();
                            let node_name = node.name.clone();
                            let node_type = node.node_type_singular_label.clone();
                            let node_path = node_display_path(&node);
                            view! {
                                <label class="checkbox-list__item permission-picker__item">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || selected_node_ids.get().contains(&node_id_for_checked)
                                        on:change=move |event| {
                                            let checked = event_target_checked(&event);
                                            selected_node_ids.update(|ids| {
                                                if checked {
                                                    ids.insert(node_id_for_change.clone());
                                                } else {
                                                    ids.remove(&node_id_for_change);
                                                }
                                            });
                                        }
                                    />
                                    <span>
                                        <strong>{node_name}</strong>
                                        <small>{format!("{node_type} - {node_path}")}</small>
                                    </span>
                                </label>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
    }
}
