//! Workflow update and publish save orchestration.

#[cfg(feature = "hydrate")]
use super::{workflow_step_payloads_from_drafts, workflow_step_signature};
#[cfg(feature = "hydrate")]
use crate::features::workflows::{
    CreateWorkflowRevisionPayload, UpdateWorkflowPayload, UpdateWorkflowRevisionStepsPayload,
};
use crate::features::workflows::{WorkflowSaveIntent, WorkflowStepDraft};
#[cfg(feature = "hydrate")]
use crate::http::send_json_id_request;
#[cfg(feature = "hydrate")]
use crate::utils::text::IntoNonemptyString;
use leptos::prelude::*;
use std::collections::HashSet;

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
