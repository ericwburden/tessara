//! Mobile workflow assignment card for Operations.

use crate::features::operations::display::{
    workflow_assignment_href, workflow_response_summary, workflow_revision_label,
    workflow_step_summary,
};
use crate::features::operations::types::WorkflowAssignmentStatus;
use crate::ui::{StatusBadge, Timestamp};
use leptos::prelude::*;

#[component]
pub(super) fn WorkflowAssignmentMobileCard(instance: WorkflowAssignmentStatus) -> impl IntoView {
    let assignment_href = workflow_assignment_href(&instance);
    view! {
        <article class="related-work-mobile-card operations-mobile-card">
            <header class="related-work-mobile-card__header">
                <a href=assignment_href>{instance.workflow_name.clone()}</a>
                <small class="workflow-assignment-step-meta">{workflow_revision_label(&instance)}</small>
            </header>
            <dl>
                <div>
                    <dt>"Node"</dt>
                    <dd>{instance.node_name.clone()}</dd>
                </div>
                <div>
                    <dt>"Assignee"</dt>
                    <dd>
                        <strong>{instance.assignee_display_name.clone()}</strong>
                        <small class="workflow-assignment-step-meta">{instance.assignee_email.clone()}</small>
                    </dd>
                </div>
                <div>
                    <dt>"Status"</dt>
                    <dd><StatusBadge label=instance.assignment_status.clone()/></dd>
                </div>
                <div>
                    <dt>"Current step"</dt>
                    <dd>
                        <strong>{instance.current_step_title.clone().unwrap_or_else(|| "No active step".to_string())}</strong>
                        <small class="workflow-assignment-step-meta">{workflow_step_summary(&instance)}</small>
                    </dd>
                </div>
                <div>
                    <dt>"Responses"</dt>
                    <dd>{workflow_response_summary(&instance)}</dd>
                </div>
                <div>
                    <dt>"Started"</dt>
                    <dd><Timestamp value=instance.started_at.clone()/></dd>
                </div>
            </dl>
        </article>
    }
}
