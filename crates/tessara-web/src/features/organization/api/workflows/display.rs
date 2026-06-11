//! Workflow display helpers used through the Organization API boundary.

use crate::features::workflows::types::WorkflowSummary;

/// Builds a summary label for workflow assigned users.
pub(crate) fn workflow_assigned_users_label(workflow: &WorkflowSummary) -> String {
    if workflow.assigned_users.is_empty() {
        "No active assignments".to_string()
    } else {
        workflow
            .assigned_users
            .iter()
            .map(|user| format!("{} {}", user.display_name, user.email))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
