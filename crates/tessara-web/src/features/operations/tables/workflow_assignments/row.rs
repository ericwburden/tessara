//! Desktop workflow assignment row for Operations.

use crate::features::operations::display::{
    workflow_assignment_href, workflow_response_summary, workflow_revision_label,
    workflow_step_summary,
};
use crate::features::operations::types::WorkflowAssignmentStatus;
use crate::ui::{StatusBadge, Timestamp};
use leptos::prelude::*;

#[component]
pub(super) fn WorkflowAssignmentRow(instance: WorkflowAssignmentStatus) -> impl IntoView {
    let assignment_href = workflow_assignment_href(&instance);
    let step_summary = workflow_step_summary(&instance);
    view! {
        <tr>
            <th scope="row">
                <a class="data-table__primary-link" href=assignment_href>{instance.workflow_name.clone()}</a>
                <small class="workflow-assignment-step-meta">{workflow_revision_label(&instance)}</small>
            </th>
            <td>{instance.node_name.clone()}</td>
            <td>
                <strong>{instance.assignee_display_name.clone()}</strong>
                <small class="workflow-assignment-step-meta">{instance.assignee_email.clone()}</small>
            </td>
            <td class="data-table__cell--center"><StatusBadge label=instance.assignment_status.clone()/></td>
            <td>
                <strong>{instance.current_step_title.clone().unwrap_or_else(|| "No active step".to_string())}</strong>
                <small class="workflow-assignment-step-meta">{step_summary}</small>
            </td>
            <td class="data-table__cell--center">{workflow_response_summary(&instance)}</td>
            <td><Timestamp value=instance.started_at.clone()/></td>
        </tr>
    }
}
