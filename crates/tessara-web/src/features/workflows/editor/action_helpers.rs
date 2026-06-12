//! Shared helpers for workflow editor actions.

#[cfg(feature = "hydrate")]
use super::errors::WorkflowEditorMutationError;
#[cfg(feature = "hydrate")]
use crate::features::workflows::WorkflowSaveIntent;
#[cfg(feature = "hydrate")]
use crate::http::{navigate_to_href, redirect_to_login};
#[cfg(feature = "hydrate")]
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
pub(super) fn navigate_to_workflow(workflow_id: &str) {
    navigate_to_href(&format!("/workflows/{workflow_id}"));
}

#[cfg(feature = "hydrate")]
pub(super) fn handle_workflow_editor_error(
    error: WorkflowEditorMutationError,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    save_intent: Option<RwSignal<Option<WorkflowSaveIntent>>>,
) {
    match error {
        WorkflowEditorMutationError::Unauthorized => {
            redirect_to_login();
            is_saving.set(false);
        }
        WorkflowEditorMutationError::Message(error) => {
            message.set(Some(error));
            is_saving.set(false);
        }
    }

    if let Some(save_intent) = save_intent {
        save_intent.set(None);
    }
}
