//! Workflow update and publish save orchestration.

#[cfg(feature = "hydrate")]
use super::api::{
    create_workflow_revision, publish_workflow_revision, update_workflow,
    update_workflow_revision_steps,
};
#[cfg(feature = "hydrate")]
use super::{workflow_step_payloads_from_drafts, workflow_step_signature};
#[cfg(feature = "hydrate")]
use crate::features::workflows::{
    CreateWorkflowRevisionPayload, UpdateWorkflowPayload, UpdateWorkflowRevisionStepsPayload,
};
use crate::features::workflows::{WorkflowSaveIntent, WorkflowStepDraft};
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

            match update_workflow(&workflow_id, payload).await {
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
                                update_workflow_revision_steps(&version_id, update_payload).await
                            } else {
                                let version_payload = CreateWorkflowRevisionPayload {
                                    steps: step_payload,
                                };
                                create_workflow_revision(&workflow_id, version_payload).await
                            }
                        } else {
                            let version_payload = CreateWorkflowRevisionPayload {
                                steps: step_payload,
                            };
                            create_workflow_revision(&workflow_id, version_payload).await
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
                            match publish_workflow_revision(&version_id).await {
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
