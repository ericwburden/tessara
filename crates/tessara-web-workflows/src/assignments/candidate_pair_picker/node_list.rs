//! Node side of the assignment candidate pair picker.

use super::super::filtering::{filtered_node_candidates, selected_node_summary};
use super::super::types::WorkflowAssignmentCandidate;
use icons::{Search, X};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub(super) fn WorkflowAssignmentNodePicker(
    candidates: RwSignal<Vec<WorkflowAssignmentCandidate>>,
    selected_candidate_id: RwSignal<String>,
    selected_workflow_version_id: RwSignal<String>,
    selected_node_id: RwSignal<String>,
    selected_account_ids: RwSignal<HashSet<String>>,
    node_search: RwSignal<String>,
    candidates_loading: RwSignal<bool>,
) -> impl IntoView {
    let filtered_node_candidates = move || {
        filtered_node_candidates(
            candidates.get(),
            &node_search.get(),
            &selected_workflow_version_id.get(),
        )
    };
    let selected_node_summary =
        move || selected_node_summary(candidates.get(), &selected_node_id.get());

    view! {
        <section class="workflow-assignment-pair-list" aria-labelledby="workflow-assignment-node-list">
            <div class="workflow-assignment-pair-list__header">
                <h4 id="workflow-assignment-node-list">"Node"</h4>
            </div>
            {move || {
                if let Some((node_name, node_path)) = selected_node_summary() {
                    view! {
                        <div class="workflow-assignment-selected-option">
                            <div>
                                <strong>{node_name}</strong>
                                <span>{node_path}</span>
                            </div>
                            <button
                                class="icon-button icon-button--control"
                                type="button"
                                aria-label="Clear selected node"
                                on:click=move |_| {
                                    selected_node_id.set(String::new());
                                    selected_candidate_id.set(String::new());
                                    selected_account_ids.set(HashSet::new());
                                }
                            >
                                <X/>
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <label class="searchable-data-table__search searchable-data-table__control workflow-assignment-candidate-search">
                            <Search class="searchable-data-table__control-icon"/>
                            <span class="sr-only">"Search nodes"</span>
                            <input
                                type="search"
                                placeholder="Search nodes"
                                prop:value=move || node_search.get()
                                on:input=move |event| node_search.set(event_target_value(&event))
                            />
                        </label>
                        <div class="workflow-assignment-pair-list__options">
                            {move || {
                                let options = filtered_node_candidates();
                                if options.is_empty() {
                                    view! { <p class="workflow-assignee-picker__empty">"No Nodes to Display"</p> }.into_any()
                                } else {
                                    options.into_iter().map(|candidate| {
                                        let node_id = candidate.node_id.clone();
                                        let node_id_for_class = node_id.clone();
                                        view! {
                                            <button
                                                class=move || if selected_node_id.get() == node_id_for_class {
                                                    "workflow-assignment-pair-option is-selected"
                                                } else {
                                                    "workflow-assignment-pair-option"
                                                }
                                                type="button"
                                                disabled=move || candidates_loading.get()
                                                on:click=move |_| {
                                                    let node_id = node_id.clone();
                                                    selected_node_id.set(node_id.clone());
                                                    let selected_workflow = selected_workflow_version_id.get_untracked();
                                                    if !selected_workflow.is_empty()
                                                        && !candidates.get_untracked().into_iter().any(|candidate| {
                                                            candidate.workflow_version_id == selected_workflow
                                                                && candidate.node_id == node_id
                                                        })
                                                    {
                                                        selected_workflow_version_id.set(String::new());
                                                    }
                                                }
                                            >
                                                <strong>{candidate.node_name}</strong>
                                                <span>{candidate.node_path}</span>
                                            </button>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </div>
                    }.into_any()
                }
            }}
        </section>
    }
}
