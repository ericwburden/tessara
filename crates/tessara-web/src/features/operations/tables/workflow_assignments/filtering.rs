//! Filtering helpers for the operations workflow assignments table.

use crate::features::operations::types::WorkflowAssignmentStatus;
use crate::utils::text::text_matches;

pub(super) fn filtered_workflow_assignments(
    assignments: &[WorkflowAssignmentStatus],
    query: &str,
    selected_node: &str,
    selected_assignee: &str,
    selected_status: &str,
) -> Vec<WorkflowAssignmentStatus> {
    assignments
        .iter()
        .filter(|assignment| {
            let matches_node = selected_node == "all" || assignment.node_name == selected_node;
            let matches_assignee =
                selected_assignee == "all" || assignment.assignee_display_name == selected_assignee;
            let matches_status =
                selected_status == "all" || assignment.assignment_status == selected_status;
            matches_node
                && matches_assignee
                && matches_status
                && text_matches(
                    query,
                    &[
                        assignment.workflow_name.as_str(),
                        assignment.node_name.as_str(),
                        assignment.assignee_display_name.as_str(),
                        assignment.assignee_email.as_str(),
                        assignment.assignment_status.as_str(),
                        assignment
                            .current_step_title
                            .as_deref()
                            .unwrap_or("No active step"),
                    ],
                )
        })
        .cloned()
        .collect()
}
