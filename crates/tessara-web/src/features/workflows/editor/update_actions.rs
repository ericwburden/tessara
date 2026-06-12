//! Workflow update and publish save orchestration.

#[cfg(feature = "hydrate")]
use super::action_helpers::{handle_workflow_editor_error, navigate_to_workflow};
#[cfg(feature = "hydrate")]
use super::api::{
    create_workflow_revision, publish_workflow_revision, update_workflow,
    update_workflow_revision_steps,
};
#[cfg(feature = "hydrate")]
use super::update_payloads::prepare_workflow_update;
#[cfg(feature = "hydrate")]
use crate::features::workflows::{
    CreateWorkflowRevisionPayload, UpdateWorkflowRevisionStepsPayload,
};
use crate::features::workflows::{WorkflowSaveIntent, WorkflowStepDraft};
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

        let prepared_update = match prepare_workflow_update(
            name.get(),
            slug.get(),
            available_node_ids.get(),
            steps.get(),
            &original_steps.get_untracked(),
            description.get(),
            version_is_draft,
            intent,
        ) {
            Ok(prepared_update) => prepared_update,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            save_intent.set(Some(intent));
            message.set(None);

            match update_workflow(&workflow_id, prepared_update.payload).await {
                Ok(_) => {
                    let mut version_to_publish =
                        if intent == WorkflowSaveIntent::Publish && version_is_draft {
                            version_id.clone()
                        } else {
                            None
                        };

                    if let Some(step_payload) = prepared_update.step_payload {
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
                                handle_workflow_editor_error(
                                    error,
                                    is_saving,
                                    message,
                                    Some(save_intent),
                                );
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
                                    navigate_to_workflow(&workflow_id);
                                }
                                Err(error) => {
                                    handle_workflow_editor_error(
                                        error,
                                        is_saving,
                                        message,
                                        Some(save_intent),
                                    );
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

                    navigate_to_workflow(&workflow_id);
                }
                Err(error) => {
                    handle_workflow_editor_error(error, is_saving, message, Some(save_intent));
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
