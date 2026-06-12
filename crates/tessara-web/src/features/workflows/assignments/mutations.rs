//! Workflow assignment mutation orchestration.

#[cfg(feature = "hydrate")]
use crate::features::workflows::assignments::api::{
    create_workflow_assignments_bulk, update_workflow_assignment,
};
#[cfg(feature = "hydrate")]
use crate::features::workflows::assignments::{
    BulkWorkflowAssignmentPayload, UpdateWorkflowAssignmentPayload, load_workflow_assignments,
};
use crate::features::workflows::assignments::{
    WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};
#[cfg(feature = "hydrate")]
use crate::features::workflows::workflow_assignment_candidate_key;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;
use std::collections::HashSet;

#[cfg(feature = "hydrate")]
use super::errors::WorkflowAssignmentMutationError;

pub(crate) fn submit_workflow_assignment_bulk(
    selected_candidate_id: RwSignal<String>,
    candidates: RwSignal<Vec<WorkflowAssignmentCandidate>>,
    selected_account_ids: RwSignal<HashSet<String>>,
    assignments: RwSignal<Vec<WorkflowAssignmentSummary>>,
    assignments_loading: RwSignal<bool>,
    assignments_error: RwSignal<Option<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let candidate_id = selected_candidate_id.get();
        let Some(candidate) = candidates
            .get_untracked()
            .into_iter()
            .find(|candidate| workflow_assignment_candidate_key(candidate) == candidate_id)
        else {
            message.set(Some("Select a workflow and node candidate.".into()));
            return;
        };

        let account_ids = selected_account_ids
            .get_untracked()
            .into_iter()
            .collect::<Vec<_>>();

        if account_ids.is_empty() {
            message.set(Some("Select at least one assignee.".into()));
            return;
        }

        let payload = BulkWorkflowAssignmentPayload {
            workflow_version_id: candidate.workflow_version_id,
            node_id: candidate.node_id,
            account_ids,
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            match create_workflow_assignments_bulk(payload).await {
                Ok(()) => {
                    selected_account_ids.set(HashSet::new());
                    selected_candidate_id.set(String::new());
                    message.set(Some("Assignments created.".into()));
                    is_saving.set(false);
                    load_workflow_assignments(assignments, assignments_loading, assignments_error);
                }
                Err(WorkflowAssignmentMutationError::Unauthorized) => {
                    is_saving.set(false);
                    redirect_to_login();
                }
                Err(WorkflowAssignmentMutationError::Message(error)) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
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
}

#[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
pub(crate) fn toggle_workflow_assignment(
    assignment: WorkflowAssignmentSummary,
    assignments: RwSignal<Vec<WorkflowAssignmentSummary>>,
    assignments_loading: RwSignal<bool>,
    assignments_error: RwSignal<Option<String>>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        let payload = UpdateWorkflowAssignmentPayload {
            node_id: assignment.node_id,
            account_id: assignment.account_id,
            is_active: !assignment.is_active,
        };
        let assignment_id = assignment.id;
        let next_is_active = payload.is_active;

        leptos::task::spawn_local(async move {
            message.set(None);
            assignments_error.set(None);

            match update_workflow_assignment(&assignment_id, payload).await {
                Ok(()) => {
                    assignments.update(|items| {
                        if let Some(item) = items.iter_mut().find(|item| item.id == assignment_id) {
                            item.is_active = next_is_active;
                        }
                    });
                    assignments_loading.set(false);
                    message.set(Some("Assignment updated.".into()));
                }
                Err(WorkflowAssignmentMutationError::Unauthorized) => redirect_to_login(),
                Err(WorkflowAssignmentMutationError::Message(error)) => {
                    message.set(Some(error));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            assignment,
            assignments,
            assignments_loading,
            assignments_error,
            message,
        );
    }
}
