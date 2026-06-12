//! Payload preparation for workflow update actions.

use super::validation::{
    validate_workflow_steps, validated_available_node_ids, validated_workflow_name,
    validated_workflow_slug,
};
use super::{workflow_step_payloads_from_drafts, workflow_step_signature};
use crate::features::workflows::{
    CreateWorkflowStepPayload, UpdateWorkflowPayload, WorkflowSaveIntent, WorkflowStepDraft,
};
use crate::utils::text::IntoNonemptyString;
use std::collections::HashSet;

pub(super) struct PreparedWorkflowUpdate {
    pub(super) payload: UpdateWorkflowPayload,
    pub(super) step_payload: Option<Vec<CreateWorkflowStepPayload>>,
}

pub(super) fn prepare_workflow_update(
    name: String,
    slug: String,
    available_node_ids: HashSet<String>,
    current_steps: Vec<WorkflowStepDraft>,
    original_steps: &[WorkflowStepDraft],
    description: String,
    version_is_draft: bool,
    intent: WorkflowSaveIntent,
) -> Result<PreparedWorkflowUpdate, String> {
    let workflow_name = validated_workflow_name(name)?;
    let workflow_slug = validated_workflow_slug(slug)?;
    let selected_available_node_ids = validated_available_node_ids(available_node_ids)?;

    let steps_changed =
        workflow_step_signature(&current_steps) != workflow_step_signature(original_steps);
    if intent == WorkflowSaveIntent::Publish && !version_is_draft && !steps_changed {
        return Err("Make a workflow step change before publishing a new revision.".into());
    }

    let step_payload = if steps_changed {
        validate_workflow_steps(&current_steps)?;
        Some(workflow_step_payloads_from_drafts(current_steps))
    } else {
        None
    };

    Ok(PreparedWorkflowUpdate {
        payload: UpdateWorkflowPayload {
            available_node_ids: selected_available_node_ids,
            name: workflow_name,
            slug: workflow_slug,
            description: description.trim().to_string().into_nonempty(),
        },
        step_payload,
    })
}
