//! Workflow creation save orchestration.

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
