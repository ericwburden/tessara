//! Workflow update and publish save orchestration.

#[cfg(feature = "hydrate")]
use super::action_helpers::{handle_workflow_editor_error, navigate_to_workflow};
#[cfg(feature = "hydrate")]
use super::api::{
    create_workflow_revision, publish_workflow_revision, update_workflow,
    update_workflow_revision_steps,
};
#[cfg(feature = "hydrate")]
use super::errors::WorkflowEditorMutationError;
#[cfg(feature = "hydrate")]
use super::update_payloads::{WorkflowUpdateDraft, prepare_workflow_update};
#[cfg(feature = "hydrate")]
use crate::features::workflows::{
    CreateWorkflowRevisionPayload, CreateWorkflowStepPayload, UpdateWorkflowRevisionStepsPayload,
};
use crate::features::workflows::{WorkflowSaveIntent, WorkflowStepDraft};
use leptos::prelude::*;
use std::collections::HashSet;

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct SubmitUpdateWorkflowInput {
    pub(crate) workflow_id: String,
    pub(crate) version_id: Option<String>,
    pub(crate) version_is_draft: bool,
    pub(crate) name: RwSignal<String>,
    pub(crate) slug: RwSignal<String>,
    pub(crate) available_node_ids: RwSignal<HashSet<String>>,
    pub(crate) steps: RwSignal<Vec<WorkflowStepDraft>>,
    pub(crate) original_steps: RwSignal<Vec<WorkflowStepDraft>>,
    pub(crate) description: RwSignal<String>,
    pub(crate) is_saving: RwSignal<bool>,
    pub(crate) save_intent: RwSignal<Option<WorkflowSaveIntent>>,
    pub(crate) message: RwSignal<Option<String>>,
    pub(crate) intent: WorkflowSaveIntent,
}

#[cfg(feature = "hydrate")]
async fn save_workflow_step_revision(
    workflow_id: &str,
    version_id: Option<String>,
    version_is_draft: bool,
    step_payload: Option<Vec<CreateWorkflowStepPayload>>,
    intent: WorkflowSaveIntent,
) -> Result<Option<String>, WorkflowEditorMutationError> {
    let mut version_to_publish = if intent == WorkflowSaveIntent::Publish && version_is_draft {
        version_id.clone()
    } else {
        None
    };

    if let Some(step_payload) = step_payload {
        let saved_version = if version_is_draft {
            if let Some(version_id) = version_id {
                let update_payload = UpdateWorkflowRevisionStepsPayload {
                    steps: step_payload,
                };
                update_workflow_revision_steps(&version_id, update_payload).await?
            } else {
                let version_payload = CreateWorkflowRevisionPayload {
                    steps: step_payload,
                };
                create_workflow_revision(workflow_id, version_payload).await?
            }
        } else {
            let version_payload = CreateWorkflowRevisionPayload {
                steps: step_payload,
            };
            create_workflow_revision(workflow_id, version_payload).await?
        };

        if intent == WorkflowSaveIntent::Publish {
            version_to_publish = Some(saved_version.id);
        }
    }

    Ok(version_to_publish)
}

pub(crate) fn submit_update_workflow(input: SubmitUpdateWorkflowInput) {
    #[cfg(feature = "hydrate")]
    {
        let SubmitUpdateWorkflowInput {
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
        } = input;

        if is_saving.get() {
            return;
        }

        let prepared_update = match prepare_workflow_update(WorkflowUpdateDraft {
            name: name.get(),
            slug: slug.get(),
            available_node_ids: available_node_ids.get(),
            current_steps: steps.get(),
            original_steps: original_steps.get_untracked(),
            description: description.get(),
            version_is_draft,
            intent,
        }) {
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
                    let version_to_publish = match save_workflow_step_revision(
                        &workflow_id,
                        version_id.clone(),
                        version_is_draft,
                        prepared_update.step_payload,
                        intent,
                    )
                    .await
                    {
                        Ok(version_to_publish) => version_to_publish,
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
        let _ = input;
    }
}
