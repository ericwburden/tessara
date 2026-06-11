//! Workflows support for the Organization feature.
//!
//! Keep functionality here when it is owned by Organization and specifically supports the Workflows concern.

#[cfg(feature = "hydrate")]
use crate::api::client::{redirect_to_login, send_json_id_request};
#[cfg(feature = "hydrate")]
use crate::features::shared::{existing_workflow_slugs, unique_slug_from_label};
#[cfg(feature = "hydrate")]
use crate::features::workflows::assignments::{
    BulkWorkflowAssignmentPayload, UpdateWorkflowAssignmentPayload, load_workflow_assignments,
};
use crate::features::workflows::assignments::{
    WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};
use crate::features::workflows::types::WorkflowSummary;
#[cfg(feature = "hydrate")]
use crate::features::workflows::workflow_assignment_candidate_key;
#[cfg(feature = "hydrate")]
use crate::features::workflows::{
    CreateWorkflowPayload, CreateWorkflowRevisionPayload, UpdateWorkflowPayload,
    UpdateWorkflowRevisionStepsPayload,
};
use crate::features::workflows::{
    CreateWorkflowStepPayload, WorkflowSaveIntent, WorkflowStepDraft,
};
use leptos::prelude::*;
use std::collections::HashSet;

use super::IntoNonemptyString;
#[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
/// Submits the submit create workflow request.
pub(crate) fn submit_create_workflow(
    name: RwSignal<String>,
    available_node_ids: RwSignal<HashSet<String>>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
    description: RwSignal<String>,
    existing_workflows: RwSignal<Vec<WorkflowSummary>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let workflow_name = name.get().trim().to_string();
        if workflow_name.is_empty() {
            message.set(Some("Workflow name is required.".into()));
            return;
        }
        let mut selected_available_node_ids =
            available_node_ids.get().into_iter().collect::<Vec<_>>();
        selected_available_node_ids.sort();
        if selected_available_node_ids.is_empty() {
            message.set(Some("Select at least one available node.".into()));
            return;
        }

        let current_steps = steps.get();
        if current_steps.is_empty() {
            message.set(Some("Add at least one workflow step.".into()));
            return;
        }
        if current_steps
            .iter()
            .any(|step| step.form_version_id.trim().is_empty())
        {
            message.set(Some("Select a form version for each workflow step.".into()));
            return;
        }

        let workflow_steps = current_steps
            .into_iter()
            .enumerate()
            .map(|(index, step)| CreateWorkflowStepPayload {
                title: step
                    .title
                    .trim()
                    .to_string()
                    .into_nonempty()
                    .unwrap_or_else(|| format!("Step {}", index + 1)),
                form_version_id: step.form_version_id,
            })
            .collect::<Vec<_>>();

        let workflow_slug = unique_slug_from_label(
            &workflow_name,
            &existing_workflow_slugs(existing_workflows.get_untracked().as_slice()),
        );
        if workflow_slug.is_empty() {
            message.set(Some(
                "Workflow name must contain letters or numbers.".into(),
            ));
            return;
        }

        let payload = CreateWorkflowPayload {
            available_node_ids: selected_available_node_ids,
            name: workflow_name,
            slug: workflow_slug,
            description: description.get().trim().to_string().into_nonempty(),
        };
        let version_payload = CreateWorkflowRevisionPayload {
            steps: workflow_steps,
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let workflow_body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Create request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };
            let version_body = match serde_json::to_string(&version_payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Workflow step request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_id_request(
                gloo_net::http::Request::post("/api/workflows"),
                Some(workflow_body),
                "Create workflow",
            )
            .await
            {
                Ok(created) => {
                    let version_url = format!("/api/workflows/{}/versions", created.id);
                    match send_json_id_request(
                        gloo_net::http::Request::post(&version_url),
                        Some(version_body),
                        "Create workflow steps",
                    )
                    .await
                    {
                        Ok(_) => {
                            if let Some(window) = web_sys::window() {
                                let _ = window
                                    .location()
                                    .set_href(&format!("/workflows/{}", created.id));
                            }
                        }
                        Err(error) => {
                            if error != "Authentication is required." {
                                message.set(Some(error));
                            }
                            is_saving.set(false);
                        }
                    }
                }
                Err(error) => {
                    if error != "Authentication is required." {
                        message.set(Some(error));
                    }
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            name,
            available_node_ids,
            steps,
            description,
            existing_workflows,
            is_saving,
            message,
        );
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the workflow step payloads from drafts behavior.
pub(crate) fn workflow_step_payloads_from_drafts(
    steps: Vec<WorkflowStepDraft>,
) -> Vec<CreateWorkflowStepPayload> {
    steps
        .into_iter()
        .enumerate()
        .map(|(index, step)| CreateWorkflowStepPayload {
            title: step
                .title
                .trim()
                .to_string()
                .into_nonempty()
                .unwrap_or_else(|| format!("Step {}", index + 1)),
            form_version_id: step.form_version_id,
        })
        .collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the workflow step signature behavior.
pub(crate) fn workflow_step_signature(steps: &[WorkflowStepDraft]) -> Vec<(String, String)> {
    steps
        .iter()
        .map(|step| {
            (
                step.title.trim().to_string(),
                step.form_version_id.trim().to_string(),
            )
        })
        .collect()
}

/// Handles the workflow step title by id behavior.
pub(crate) fn workflow_step_title_by_id(steps: &[WorkflowStepDraft], step_id: usize) -> String {
    steps
        .iter()
        .find(|step| step.id == step_id)
        .map(|step| step.title.clone())
        .unwrap_or_default()
}

/// Handles the workflow step form version id by id behavior.
pub(crate) fn workflow_step_form_version_id_by_id(
    steps: &[WorkflowStepDraft],
    step_id: usize,
) -> String {
    steps
        .iter()
        .find(|step| step.id == step_id)
        .map(|step| step.form_version_id.clone())
        .unwrap_or_default()
}

#[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
/// Submits the submit update workflow request.
pub(crate) fn submit_update_workflow(
    workflow_id: String,
    version_id: Option<String>,
    version_is_draft: bool,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    available_node_ids: RwSignal<HashSet<String>>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
    original_steps: RwSignal<Vec<WorkflowStepDraft>>,
    description: RwSignal<String>,
    is_saving: RwSignal<bool>,
    save_intent: RwSignal<Option<WorkflowSaveIntent>>,
    message: RwSignal<Option<String>>,
    intent: WorkflowSaveIntent,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let workflow_name = name.get().trim().to_string();
        if workflow_name.is_empty() {
            message.set(Some("Workflow name is required.".into()));
            return;
        }

        let workflow_slug = slug.get().trim().to_string();
        if workflow_slug.is_empty() {
            message.set(Some(
                "Workflow slug is missing. Reload the workflow and try again.".into(),
            ));
            return;
        }
        let mut selected_available_node_ids =
            available_node_ids.get().into_iter().collect::<Vec<_>>();
        selected_available_node_ids.sort();
        if selected_available_node_ids.is_empty() {
            message.set(Some("Select at least one available node.".into()));
            return;
        }

        let current_steps = steps.get();
        let steps_changed = workflow_step_signature(&current_steps)
            != workflow_step_signature(&original_steps.get_untracked());
        if intent == WorkflowSaveIntent::Publish && !version_is_draft && !steps_changed {
            message.set(Some(
                "Make a workflow step change before publishing a new revision.".into(),
            ));
            return;
        }

        let step_payload = if steps_changed {
            if current_steps.is_empty() {
                message.set(Some("Add at least one workflow step.".into()));
                return;
            }
            if current_steps
                .iter()
                .any(|step| step.form_version_id.trim().is_empty())
            {
                message.set(Some("Select a form version for each workflow step.".into()));
                return;
            }

            Some(workflow_step_payloads_from_drafts(current_steps))
        } else {
            None
        };

        let payload = UpdateWorkflowPayload {
            available_node_ids: selected_available_node_ids,
            name: workflow_name,
            slug: workflow_slug,
            description: description.get().trim().to_string().into_nonempty(),
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            save_intent.set(Some(intent));
            message.set(None);

            let workflow_body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Update request could not be prepared.".into()));
                    is_saving.set(false);
                    save_intent.set(None);
                    return;
                }
            };

            let workflow_url = format!("/api/workflows/{workflow_id}");
            match send_json_id_request(
                gloo_net::http::Request::put(&workflow_url),
                Some(workflow_body),
                "Update workflow",
            )
            .await
            {
                Ok(_) => {
                    let mut version_to_publish =
                        if intent == WorkflowSaveIntent::Publish && version_is_draft {
                            version_id.clone()
                        } else {
                            None
                        };

                    let had_step_update = step_payload.is_some();
                    if let Some(step_payload) = step_payload {
                        let step_result = if version_is_draft {
                            if let Some(version_id) = version_id.clone() {
                                let update_payload = UpdateWorkflowRevisionStepsPayload {
                                    steps: step_payload,
                                };
                                let step_body = match serde_json::to_string(&update_payload) {
                                    Ok(body) => body,
                                    Err(_) => {
                                        message.set(Some(
                                            "Workflow step update request could not be prepared."
                                                .into(),
                                        ));
                                        is_saving.set(false);
                                        save_intent.set(None);
                                        return;
                                    }
                                };
                                let steps_url =
                                    format!("/api/workflow-versions/{version_id}/steps");
                                send_json_id_request(
                                    gloo_net::http::Request::put(&steps_url),
                                    Some(step_body),
                                    "Update workflow steps",
                                )
                                .await
                            } else {
                                let version_payload = CreateWorkflowRevisionPayload {
                                    steps: step_payload,
                                };
                                let version_body = match serde_json::to_string(&version_payload) {
                                    Ok(body) => body,
                                    Err(_) => {
                                        message.set(Some(
                                            "Workflow revision request could not be prepared."
                                                .into(),
                                        ));
                                        is_saving.set(false);
                                        save_intent.set(None);
                                        return;
                                    }
                                };
                                let version_url = format!("/api/workflows/{workflow_id}/versions");
                                send_json_id_request(
                                    gloo_net::http::Request::post(&version_url),
                                    Some(version_body),
                                    "Create workflow revision",
                                )
                                .await
                            }
                        } else {
                            let version_payload = CreateWorkflowRevisionPayload {
                                steps: step_payload,
                            };
                            let version_body = match serde_json::to_string(&version_payload) {
                                Ok(body) => body,
                                Err(_) => {
                                    message.set(Some(
                                        "Workflow revision request could not be prepared.".into(),
                                    ));
                                    is_saving.set(false);
                                    save_intent.set(None);
                                    return;
                                }
                            };
                            let version_url = format!("/api/workflows/{workflow_id}/versions");
                            send_json_id_request(
                                gloo_net::http::Request::post(&version_url),
                                Some(version_body),
                                "Create workflow revision",
                            )
                            .await
                        };

                        let saved_version = match step_result {
                            Ok(body) => body,
                            Err(error) => {
                                if error != "Authentication is required." {
                                    message.set(Some(error));
                                }
                                is_saving.set(false);
                                save_intent.set(None);
                                return;
                            }
                        };

                        if intent == WorkflowSaveIntent::Publish {
                            version_to_publish = Some(saved_version.id);
                        }
                    }

                    if intent == WorkflowSaveIntent::Publish {
                        if let Some(version_id) = version_to_publish {
                            let publish_url =
                                format!("/api/workflow-versions/{version_id}/publish");
                            match send_json_id_request(
                                gloo_net::http::Request::post(&publish_url),
                                None,
                                "Publish workflow revision",
                            )
                            .await
                            {
                                Ok(_) => {
                                    if let Some(window) = web_sys::window() {
                                        let _ = window
                                            .location()
                                            .set_href(&format!("/workflows/{workflow_id}"));
                                    }
                                }
                                Err(error) => {
                                    if error != "Authentication is required." {
                                        message.set(Some(error));
                                    }
                                    is_saving.set(false);
                                    save_intent.set(None);
                                }
                            }
                            return;
                        }

                        message.set(Some(
                            "No draft workflow revision is available to publish.".into(),
                        ));
                        is_saving.set(false);
                        save_intent.set(None);
                        return;
                    }

                    if had_step_update {
                        if let Some(window) = web_sys::window() {
                            let _ = window
                                .location()
                                .set_href(&format!("/workflows/{workflow_id}"));
                        }
                    } else if let Some(window) = web_sys::window() {
                        let _ = window
                            .location()
                            .set_href(&format!("/workflows/{workflow_id}"));
                    }
                }
                Err(error) => {
                    if error != "Authentication is required." {
                        message.set(Some(error));
                    }
                    is_saving.set(false);
                    save_intent.set(None);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            workflow_id,
            version_id,
            version_is_draft,
            name,
            slug,
            available_node_ids,
            steps,
            original_steps,
            description,
            is_saving,
            save_intent,
            message,
            intent,
        );
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
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
