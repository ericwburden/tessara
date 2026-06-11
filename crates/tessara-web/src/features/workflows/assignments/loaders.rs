//! Signal-aware loaders for workflow assignments.

use crate::features::workflows::assignments::types::{
    PendingWorkflowWork, WorkflowAssigneeOption, WorkflowAssignmentCandidate,
    WorkflowAssignmentSummary,
};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::api::{
    WorkflowAssignmentApiError, fetch_pending_work, fetch_workflow_assignment_assignees,
    fetch_workflow_assignment_candidates, fetch_workflow_assignments,
};

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn load_pending_work(
    pending_work: RwSignal<Vec<PendingWorkflowWork>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_pending_work().await {
                Ok(loaded_work) => {
                    pending_work.set(loaded_work);
                    is_loading.set(false);
                }
                Err(WorkflowAssignmentApiError::Unauthorized) => {
                    pending_work.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(WorkflowAssignmentApiError::Message(error)) => {
                    pending_work.set(Vec::new());
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (pending_work, is_loading, load_error);
    }
}

pub(crate) fn load_workflow_assignments(
    assignments: RwSignal<Vec<WorkflowAssignmentSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_workflow_assignments().await {
                Ok(loaded_assignments) => {
                    assignments.set(loaded_assignments);
                    is_loading.set(false);
                }
                Err(WorkflowAssignmentApiError::Unauthorized) => {
                    assignments.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(WorkflowAssignmentApiError::Message(error)) => {
                    assignments.set(Vec::new());
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (assignments, is_loading, load_error);
    }
}

pub(crate) fn load_workflow_assignment_candidates(
    candidates: RwSignal<Vec<WorkflowAssignmentCandidate>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_workflow_assignment_candidates().await {
                Ok(loaded_candidates) => {
                    candidates.set(loaded_candidates);
                    is_loading.set(false);
                }
                Err(WorkflowAssignmentApiError::Unauthorized) => {
                    candidates.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(WorkflowAssignmentApiError::Message(error)) => {
                    candidates.set(Vec::new());
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (candidates, is_loading, load_error);
    }
}

pub(crate) fn load_workflow_assignment_assignees(
    workflow_version_id: String,
    node_id: String,
    assignees: RwSignal<Vec<WorkflowAssigneeOption>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if workflow_version_id.trim().is_empty() || node_id.trim().is_empty() {
                assignees.set(Vec::new());
                is_loading.set(false);
                load_error.set(None);
                return;
            }

            is_loading.set(true);
            load_error.set(None);

            match fetch_workflow_assignment_assignees(&workflow_version_id, &node_id).await {
                Ok(loaded_assignees) => {
                    assignees.set(loaded_assignees);
                    is_loading.set(false);
                }
                Err(WorkflowAssignmentApiError::Unauthorized) => {
                    assignees.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(WorkflowAssignmentApiError::Message(error)) => {
                    assignees.set(Vec::new());
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            workflow_version_id,
            node_id,
            assignees,
            is_loading,
            load_error,
        );
    }
}
