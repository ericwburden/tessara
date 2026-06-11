//! Client-side API orchestration for the Workflows feature.
//!
//! Keep endpoint calls, request assembly, and response handling for Workflows screens here; pure DTOs and display formatting belong in sibling modules.

use crate::features::workflows::assignments::types::{
    PendingWorkflowWork, WorkflowAssigneeOption, WorkflowAssignmentCandidate,
    WorkflowAssignmentSummary,
};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Loads the load pending work data.
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

            match gloo_net::http::Request::get("/api/workflow-assignments/pending")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    pending_work.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<PendingWorkflowWork>>().await {
                        Ok(loaded_work) => {
                            pending_work.set(loaded_work);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            pending_work.set(Vec::new());
                            load_error.set(Some(format!("Unable to parse assigned work: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    pending_work.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load assigned work. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    pending_work.set(Vec::new());
                    load_error.set(Some(format!("Unable to load assigned work: {error}")));
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

/// Loads the load workflow assignments data.
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

            match gloo_net::http::Request::get("/api/workflow-assignments")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    assignments.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<WorkflowAssignmentSummary>>().await {
                        Ok(loaded_assignments) => {
                            assignments.set(loaded_assignments);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            assignments.set(Vec::new());
                            load_error.set(Some(format!(
                                "Unable to parse workflow assignments: {error}"
                            )));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    assignments.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load workflow assignments. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    assignments.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load workflow assignments: {error}"
                    )));
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

/// Loads the load workflow assignment candidates data.
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

            match gloo_net::http::Request::get("/api/workflow-assignment-candidates")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    candidates.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<WorkflowAssignmentCandidate>>().await {
                        Ok(loaded_candidates) => {
                            candidates.set(loaded_candidates);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            candidates.set(Vec::new());
                            load_error.set(Some(format!(
                                "Unable to parse assignment candidates: {error}"
                            )));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    candidates.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load assignment candidates. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    candidates.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load assignment candidates: {error}"
                    )));
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

/// Loads the load workflow assignment assignees data.
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
            let url = format!(
                "/api/workflow-assignment-candidates/assignees?workflow_version_id={workflow_version_id}&node_id={node_id}"
            );

            match gloo_net::http::Request::get(&url).send().await {
                Ok(response) if response.status() == 401 => {
                    assignees.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<WorkflowAssigneeOption>>().await {
                        Ok(loaded_assignees) => {
                            assignees.set(loaded_assignees);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            assignees.set(Vec::new());
                            load_error
                                .set(Some(format!("Unable to parse eligible assignees: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    assignees.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load eligible assignees. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    assignees.set(Vec::new());
                    load_error.set(Some(format!("Unable to load eligible assignees: {error}")));
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
