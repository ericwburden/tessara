//! Workflow creation save orchestration.

#[cfg(feature = "hydrate")]
use super::action_helpers::{handle_workflow_editor_error, navigate_to_workflow};
#[cfg(feature = "hydrate")]
use super::api::{create_initial_workflow_revision, create_workflow};
#[cfg(feature = "hydrate")]
use super::payloads::prepare_workflow_create;
use crate::features::workflows::WorkflowStepDraft;
use crate::features::workflows::types::WorkflowSummary;
use leptos::prelude::*;
use std::collections::HashSet;

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

        let prepared_create = match prepare_workflow_create(
            name.get(),
            available_node_ids.get(),
            steps.get(),
            description.get(),
            existing_workflows.get_untracked().as_slice(),
        ) {
            Ok(prepared_create) => prepared_create,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            match create_workflow(prepared_create.payload).await {
                Ok(created) => {
                    match create_initial_workflow_revision(
                        &created.id,
                        prepared_create.version_payload,
                    )
                    .await
                    {
                        Ok(_) => {
                            navigate_to_workflow(&created.id);
                        }
                        Err(error) => {
                            handle_workflow_editor_error(error, is_saving, message, None);
                        }
                    }
                }
                Err(error) => {
                    handle_workflow_editor_error(error, is_saving, message, None);
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
