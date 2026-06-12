//! Workflow assignment creation form.

use super::types::{
    WorkflowAssigneeOption, WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};
use super::{WorkflowAssignmentAssigneePicker, WorkflowAssignmentCandidatePairPicker};
use crate::ui::empty_view;
use leptos::prelude::*;
use std::collections::HashSet;

use super::mutations::submit_workflow_assignment_bulk;

#[component]
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
                <WorkflowAssignmentCandidatePairPicker
                    candidates
                    selected_candidate_id
                    selected_workflow_version_id
                    selected_node_id
                    selected_account_ids
                    workflow_search
                    node_search
                    candidates_loading
                />
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
