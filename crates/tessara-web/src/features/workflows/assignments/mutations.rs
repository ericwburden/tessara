//! Workflow assignment mutation orchestration.

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

/// Submits the submit workflow assignment bulk request.
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

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Assignment request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            let response = gloo_net::http::Request::post("/api/workflow-assignments/bulk")
                .header("Content-Type", "application/json")
                .body(body)
                .map_err(|_| "Assignment request could not be prepared.".to_string());

            match response {
                Ok(request) => match request.send().await {
                    Ok(response) if response.status() == 401 => {
                        is_saving.set(false);
                        redirect_to_login();
                    }
                    Ok(response) if response.ok() => {
                        selected_account_ids.set(HashSet::new());
                        selected_candidate_id.set(String::new());
                        message.set(Some("Assignments created.".into()));
                        is_saving.set(false);
                        load_workflow_assignments(
                            assignments,
                            assignments_loading,
                            assignments_error,
                        );
                    }
                    Ok(response) => {
                        message.set(Some(format!(
                            "Create assignments failed with status {}.",
                            response.status()
                        )));
                        is_saving.set(false);
                    }
                    Err(error) => {
                        message.set(Some(format!(
                            "Could not reach the assignments API: {error}"
                        )));
                        is_saving.set(false);
                    }
                },
                Err(error) => {
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
/// Toggles the toggle workflow assignment state.
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
            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Update request could not be prepared.".into()));
                    return;
                }
            };
            let response =
                gloo_net::http::Request::put(&format!("/api/workflow-assignments/{assignment_id}"))
                    .header("Content-Type", "application/json")
                    .body(body)
                    .map_err(|_| "Update request could not be prepared.".to_string());

            match response {
                Ok(request) => match request.send().await {
                    Ok(response) if response.status() == 401 => redirect_to_login(),
                    Ok(response) if response.ok() => {
                        assignments.update(|items| {
                            if let Some(item) =
                                items.iter_mut().find(|item| item.id == assignment_id)
                            {
                                item.is_active = next_is_active;
                            }
                        });
                        assignments_loading.set(false);
                        message.set(Some("Assignment updated.".into()));
                    }
                    Ok(response) => {
                        message.set(Some(format!(
                            "Update assignment failed with status {}.",
                            response.status()
                        )));
                    }
                    Err(error) => {
                        message.set(Some(format!(
                            "Could not reach the assignments API: {error}"
                        )));
                    }
                },
                Err(error) => message.set(Some(error)),
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
