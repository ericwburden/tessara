//! Display helpers for the Operations feature.

use super::types::WorkflowAssignmentStatus;

/// Handles the workflow revision label behavior.
pub(crate) fn workflow_revision_label(instance: &WorkflowAssignmentStatus) -> String {
    format!(
        "Revision {}",
        instance
            .workflow_version_label
            .clone()
            .unwrap_or_else(|| "-".to_string())
    )
}

/// Handles the workflow assignment href behavior.
pub(crate) fn workflow_assignment_href(instance: &WorkflowAssignmentStatus) -> String {
    format!(
        "/workflows/assignments?assignment_id={}",
        instance.workflow_assignment_id
    )
}

/// Handles the workflow step summary behavior.
pub(crate) fn workflow_step_summary(instance: &WorkflowAssignmentStatus) -> String {
    format!(
        "{} of {} steps complete",
        instance.completed_step_count, instance.total_step_count
    )
}

/// Handles the workflow response summary behavior.
pub(crate) fn workflow_response_summary(instance: &WorkflowAssignmentStatus) -> String {
    format!(
        "{} draft / {} submitted",
        instance.draft_response_count, instance.submitted_response_count
    )
}
