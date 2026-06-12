//! Workflow side of the assignment candidate pair picker.

use super::super::state::{filtered_workflow_candidates, selected_workflow_summary};
use super::super::types::WorkflowAssignmentCandidate;
use crate::features::workflows::workflow_assignment_revision_label;
use icons::{Search, X};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub(super) fn WorkflowAssignmentWorkflowPicker(
    candidates: RwSignal<Vec<WorkflowAssignmentCandidate>>,
    selected_candidate_id: RwSignal<String>,
    selected_workflow_version_id: RwSignal<String>,
    selected_node_id: RwSignal<String>,
    selected_account_ids: RwSignal<HashSet<String>>,
    workflow_search: RwSignal<String>,
    candidates_loading: RwSignal<bool>,
) -> impl IntoView {
    let filtered_workflow_candidates = move || {
        filtered_workflow_candidates(
            candidates.get(),
            &workflow_search.get(),
            &selected_node_id.get(),
        )
    };
    let selected_workflow_summary =
        move || selected_workflow_summary(candidates.get(), &selected_workflow_version_id.get());

    view! {
        <section class="workflow-assignment-pair-list" aria-labelledby="workflow-assignment-workflow-list">
            <div class="workflow-assignment-pair-list__header">
                <h4 id="workflow-assignment-workflow-list">"Workflow"</h4>
            </div>
            {move || {
                if let Some((workflow_name, version)) = selected_workflow_summary() {
                    view! {
                        <div class="workflow-assignment-selected-option">
                            <div>
                                <strong>{workflow_name}</strong>
                                <span>{version}</span>
                            </div>
                            <button
                                class="icon-button icon-button--control"
                                type="button"
                                aria-label="Clear selected workflow"
                                on:click=move |_| {
                                    selected_workflow_version_id.set(String::new());
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
                            <span class="sr-only">"Search workflows"</span>
                            <input
                                type="search"
                                placeholder="Search workflows"
                                prop:value=move || workflow_search.get()
                                on:input=move |event| workflow_search.set(event_target_value(&event))
                            />
                        </label>
                        <div class="workflow-assignment-pair-list__options">
                            {move || {
                                let options = filtered_workflow_candidates();
                                if options.is_empty() {
                                    view! { <p class="workflow-assignee-picker__empty">"No Workflows to Display"</p> }.into_any()
                                } else {
                                    options.into_iter().map(|candidate| {
                                        let workflow_version_id = candidate.workflow_version_id.clone();
                                        let workflow_version_id_for_class = workflow_version_id.clone();
                                        let revision = workflow_assignment_revision_label(candidate.workflow_version_label.as_deref());
                                        view! {
                                            <button
                                                class=move || if selected_workflow_version_id.get() == workflow_version_id_for_class {
                                                    "workflow-assignment-pair-option is-selected"
                                                } else {
                                                    "workflow-assignment-pair-option"
                                                }
                                                type="button"
                                                disabled=move || candidates_loading.get()
                                                on:click=move |_| {
                                                    let workflow_version_id = workflow_version_id.clone();
                                                    selected_workflow_version_id.set(workflow_version_id.clone());
                                                    let selected_node = selected_node_id.get_untracked();
                                                    if !selected_node.is_empty()
                                                        && !candidates.get_untracked().into_iter().any(|candidate| {
                                                            candidate.workflow_version_id == workflow_version_id
                                                                && candidate.node_id == selected_node
                                                        })
                                                    {
                                                        selected_node_id.set(String::new());
                                                    }
                                                }
                                            >
                                                <strong>{candidate.workflow_name}</strong>
                                                <span>{format!("Revision {revision}")}</span>
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
