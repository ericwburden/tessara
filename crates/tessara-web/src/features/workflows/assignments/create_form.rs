//! Workflow assignment creation form.

use super::WorkflowAssignmentAssigneePicker;
use super::state::{
    filtered_node_candidates, filtered_workflow_candidates, selected_node_summary,
    selected_workflow_summary, workflow_assignment_pair_is_valid,
};
use super::types::{
    WorkflowAssigneeOption, WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};
use crate::features::workflows::{
    submit_workflow_assignment_bulk, workflow_assignment_revision_label,
};
use crate::ui::empty_view;
use icons::{Search, X};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
/// Renders the workflow assignment creation form.
pub(in crate::features::workflows) fn WorkflowAssignmentCreateForm(
    assignments: RwSignal<Vec<WorkflowAssignmentSummary>>,
    candidates: RwSignal<Vec<WorkflowAssignmentCandidate>>,
    assignees: RwSignal<Vec<WorkflowAssigneeOption>>,
    selected_candidate_id: RwSignal<String>,
    selected_workflow_version_id: RwSignal<String>,
    selected_node_id: RwSignal<String>,
    selected_account_ids: RwSignal<HashSet<String>>,
    workflow_search: RwSignal<String>,
    node_search: RwSignal<String>,
    assignee_search: RwSignal<String>,
    assignments_loading: RwSignal<bool>,
    assignments_error: RwSignal<Option<String>>,
    candidates_loading: RwSignal<bool>,
    candidates_error: RwSignal<Option<String>>,
    assignees_loading: RwSignal<bool>,
    assignees_error: RwSignal<Option<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let filtered_workflow_candidates = move || {
        filtered_workflow_candidates(
            candidates.get(),
            &workflow_search.get(),
            &selected_node_id.get(),
        )
    };
    let filtered_node_candidates = move || {
        filtered_node_candidates(
            candidates.get(),
            &node_search.get(),
            &selected_workflow_version_id.get(),
        )
    };
    let selected_pair_is_valid = move || {
        workflow_assignment_pair_is_valid(
            &candidates.get(),
            &selected_workflow_version_id.get(),
            &selected_node_id.get(),
        )
    };
    let invalid_pair_message = move || {
        if selected_workflow_version_id.get().is_empty()
            || selected_node_id.get().is_empty()
            || selected_pair_is_valid()
        {
            None
        } else {
            Some("That workflow is not valid for the selected node.".to_string())
        }
    };
    let selected_workflow_summary =
        move || selected_workflow_summary(candidates.get(), &selected_workflow_version_id.get());
    let selected_node_summary =
        move || selected_node_summary(candidates.get(), &selected_node_id.get());
    let can_create = move || {
        !is_saving.get()
            && !selected_candidate_id.get().is_empty()
            && !selected_account_ids.get().is_empty()
    };

    view! {
        <form
            class="native-form workflow-assignment-create-form"
            on:submit=move |event| {
                event.prevent_default();
                submit_workflow_assignment_bulk(
                    selected_candidate_id,
                    candidates,
                    selected_account_ids,
                    assignments,
                    assignments_loading,
                    assignments_error,
                    is_saving,
                    message,
                );
            }
        >
            <section class="form-section">
                <div class="form-builder-section-card__header">
                    <h3>"Create Assignment"</h3>
                </div>
                <div class="workflow-assignment-create-grid">
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
                </div>
                {move || invalid_pair_message().map(|message| view! {
                    <p class="form-message" role="alert">{message}</p>
                })}
                {move || {
                    if let Some(message) = candidates_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Assignment candidates unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if candidates_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading candidates"</h3>
                                <p>"Fetching eligible workflow and node combinations."</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        empty_view()
                    }
                }}
                <WorkflowAssignmentAssigneePicker
                    assignees
                    selected_candidate_id
                    selected_account_ids
                    assignee_search
                    assignees_loading
                    assignees_error
                />
                <div class="form-actions">
                    <button class="button button--secondary" type="submit" disabled=move || !can_create()>
                        {move || if is_saving.get() { "Creating..." } else { "Create Assignments" }}
                    </button>
                </div>
            </section>
        </form>
    }
}
