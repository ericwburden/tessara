//! Workflow step helper functions used by workflow editor flows.

use super::super::helpers::IntoNonemptyString;
use crate::features::workflows::{CreateWorkflowStepPayload, WorkflowStepDraft};

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
