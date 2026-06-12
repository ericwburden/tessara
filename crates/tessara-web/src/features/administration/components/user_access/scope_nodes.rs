//! Scope-node access display and picker components.

use super::super::super::state::toggle_string_selection;
use crate::features::administration::models::AdminScopeNodeSummary;
use crate::utils::text::text_matches;
use icons::Search;
use leptos::prelude::*;

#[component]
pub(crate) fn AdminScopeNodeList(nodes: Vec<AdminScopeNodeSummary>) -> impl IntoView {
    if nodes.is_empty() {
        view! { <p>"No scope nodes assigned."</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                    {nodes
                        .into_iter()
                        .map(|node| {
                            let node_context = admin_scope_node_context(&node);
                            view! {
                                <tr>
                                    <th scope="row">
                                        <a class="data-table__primary-link" href=format!("/organization/{}", node.node_id)>
                                            {node.node_name}
                                        </a>
                                    </th>
                                    <td>{node_context}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

#[component]
pub(crate) fn AdminScopeNodeChecklist(
    nodes: Vec<AdminScopeNodeSummary>,
    selected_node_ids: RwSignal<Vec<String>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let node_count = nodes.len();
    if nodes.is_empty() {
        view! { <p>"No scope nodes are available."</p> }.into_any()
    } else {
        view! {
            <div class="permission-picker">
                <label class="searchable-data-table__search searchable-data-table__control">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search scope nodes"</span>
                    <input
                        type="search"
                        placeholder=format!("Search {node_count} scope nodes")
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </label>
                <table class="info-list-table permission-picker__table">
                    <tbody>
                    {move || {
                        let query = search.get();
                        nodes
                            .iter()
                            .filter(|node| {
                                text_matches(
                                    &query,
                                    &[
                                        node.node_name.as_str(),
                                        node.node_type_name.as_str(),
                                        node.parent_node_name.as_deref().unwrap_or(""),
                                    ],
                                )
                            })
                            .cloned()
                            .map(|node| {
                                let node_id = node.node_id.clone();
                                let node_context = admin_scope_node_context(&node);
                                let selected = selected_node_ids
                                    .get()
                                    .iter()
                                    .any(|selected_id| selected_id == &node.node_id);
                                let checkbox_label =
                                    format!("Assign scope node {} ({node_context})", node.node_name);
                                view! {
                                    <tr>
                                        <td class="data-table__cell--center">
                                            <input
                                                type="checkbox"
                                                aria-label=checkbox_label
                                                prop:checked=selected
                                                on:change=move |event| {
                                                    toggle_string_selection(
                                                        selected_node_ids,
                                                        node_id.clone(),
                                                        event_target_checked(&event),
                                                    );
                                                }
                                            />
                                        </td>
                                        <th scope="row">{node.node_name}</th>
                                        <td>{node_context}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                    </tbody>
                </table>
            </div>
        }
        .into_any()
    }
}

fn admin_scope_node_context(node: &AdminScopeNodeSummary) -> String {
    match node.parent_node_name.as_deref() {
        Some(parent) if !parent.is_empty() => {
            format!("{} - Parent: {parent}", node.node_type_name)
        }
        _ => format!("{} - No parent", node.node_type_name),
    }
}
