//! Workflow and node candidate picker for assignment creation.

mod node_list;
mod workflow_list;

use super::filtering::workflow_assignment_pair_is_valid;
use super::types::WorkflowAssignmentCandidate;
use leptos::prelude::*;
use node_list::WorkflowAssignmentNodePicker;
use std::collections::HashSet;
use workflow_list::WorkflowAssignmentWorkflowPicker;

#[component]
pub(in crate::features::workflows) fn WorkflowAssignmentCandidatePairPicker(
    candidates: RwSignal<Vec<WorkflowAssignmentCandidate>>,
    selected_candidate_id: RwSignal<String>,
    selected_workflow_version_id: RwSignal<String>,
    selected_node_id: RwSignal<String>,
    selected_account_ids: RwSignal<HashSet<String>>,
    workflow_search: RwSignal<String>,
    node_search: RwSignal<String>,
    candidates_loading: RwSignal<bool>,
) -> impl IntoView {
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

    view! {
        <div class="workflow-assignment-create-grid">
            <WorkflowAssignmentWorkflowPicker
                candidates
                selected_candidate_id
                selected_workflow_version_id
                selected_node_id
                selected_account_ids
                workflow_search
                candidates_loading
            />
            <WorkflowAssignmentNodePicker
                candidates
                selected_candidate_id
                selected_workflow_version_id
                selected_node_id
                selected_account_ids
                node_search
                candidates_loading
            />
        </div>
        {move || invalid_pair_message().map(|message| view! {
            <p class="form-message" role="alert">{message}</p>
        })}
    }
}
