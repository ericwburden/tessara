//! Pure payload preparation for workflow editor actions.

use super::existing_workflow_slugs;
use super::validation::{
    validate_workflow_steps, validated_available_node_ids, validated_workflow_name,
};
use super::workflow_step_payloads_from_drafts;
use crate::features::workflows::types::WorkflowSummary;
use crate::features::workflows::{
    CreateWorkflowPayload, CreateWorkflowRevisionPayload, WorkflowStepDraft,
};
use crate::utils::slug::unique_slug_from_label;
use crate::utils::text::IntoNonemptyString;
use std::collections::HashSet;

pub(super) struct PreparedWorkflowCreate {
    pub(super) payload: CreateWorkflowPayload,
    pub(super) version_payload: CreateWorkflowRevisionPayload,
}

pub(super) fn prepare_workflow_create(
    name: String,
    available_node_ids: HashSet<String>,
    steps: Vec<WorkflowStepDraft>,
    description: String,
    existing_workflows: &[WorkflowSummary],
) -> Result<PreparedWorkflowCreate, String> {
    let workflow_name = validated_workflow_name(name)?;
    let selected_available_node_ids = validated_available_node_ids(available_node_ids)?;
    validate_workflow_steps(&steps)?;

    let workflow_slug =
        unique_slug_from_label(&workflow_name, &existing_workflow_slugs(existing_workflows));
    if workflow_slug.is_empty() {
        return Err("Workflow name must contain letters or numbers.".into());
    }

    Ok(PreparedWorkflowCreate {
        payload: CreateWorkflowPayload {
            available_node_ids: selected_available_node_ids,
            name: workflow_name,
            slug: workflow_slug,
            description: description.trim().to_string().into_nonempty(),
        },
        version_payload: CreateWorkflowRevisionPayload {
            steps: workflow_step_payloads_from_drafts(steps),
        },
    })
}
