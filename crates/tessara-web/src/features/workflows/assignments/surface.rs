//! Workflow assignments page surface.

use super::{
    WorkflowAssignmentCreateForm, WorkflowAssignmentsList, WorkflowAssignmentsPageState,
    assignee_filter_options, filtered_assignments,
};
use crate::ui::PageHeader;
use leptos::prelude::*;

#[component]
pub(in crate::features::workflows) fn WorkflowAssignmentsSurface(
    state: WorkflowAssignmentsPageState,
) -> impl IntoView {
    let WorkflowAssignmentsPageState {
        assignments,
        candidates,
        assignees,
        selected_candidate_id,
        selected_workflow_version_id,
        selected_node_id,
        requested_workflow_id: _,
        selected_account_ids,
        workflow_search,
        node_search,
        assignee_search,
        assignment_search,
        status_filter,
        state_filter,
        assignee_filter,
        assignments_loading,
        assignments_error,
        candidates_loading,
        candidates_error,
        assignees_loading,
        assignees_error,
        is_saving,
        message,
    } = state;

    let filtered_assignments = move || {
        filtered_assignments(
            assignments.get(),
            &assignment_search.get(),
            &status_filter.get(),
            &state_filter.get(),
            &assignee_filter.get(),
        )
    };
    let assignee_filter_options = move || assignee_filter_options(&assignments.get());

    view! {
        <section class="route-panel workflows-page workflow-assignments-page">
            <PageHeader title="Workflow Assignments"/>

            <WorkflowAssignmentCreateForm
                assignments=assignments
                candidates=candidates
                assignees=assignees
                selected_candidate_id=selected_candidate_id
                selected_workflow_version_id=selected_workflow_version_id
                selected_node_id=selected_node_id
                selected_account_ids=selected_account_ids
                workflow_search=workflow_search
                node_search=node_search
                assignee_search=assignee_search
                assignments_loading=assignments_loading
                assignments_error=assignments_error
                candidates_loading=candidates_loading
                candidates_error=candidates_error
                assignees_loading=assignees_loading
                assignees_error=assignees_error
                is_saving=is_saving
                message=message
            />

            {move || message.get().map(|message| view! {
                <p class="form-message" role="status">{message}</p>
            })}

            {move || {
                if assignments_loading.get() {
                    view! {
                        <section class="organization-state" aria-live="polite">
                            <h3>"Loading assignments"</h3>
                            <p>"Fetching workflow assignment records."</p>
                        </section>
                    }
                    .into_any()
                } else if let Some(message) = assignments_error.get() {
                    view! {
                        <section class="organization-state is-error" role="alert">
                            <h3>"Workflow assignments unavailable"</h3>
                            <p>{message}</p>
                        </section>
                    }
                    .into_any()
                } else {
                    view! {
                        <WorkflowAssignmentsList
                            assignments=filtered_assignments()
                            search=assignment_search
                            status_filter=status_filter
                            state_filter=state_filter
                            assignee_filter=assignee_filter
                            assignee_options=assignee_filter_options()
                            assignments_signal=assignments
                            assignments_loading=assignments_loading
                            assignments_error=assignments_error
                            message=message
                        />
                    }
                    .into_any()
                }
            }}
        </section>
    }
}
