//! Mobile cards for workflow assignment summaries.

use crate::features::shared::status_badge_class;
use crate::features::workflows::assignments::WorkflowAssignmentSummary;
use crate::features::workflows::{
    workflow_assignment_state, workflow_assignment_state_label, workflow_assignment_status_key,
    workflow_assignment_status_label,
};
use crate::ui::Timestamp;
use crate::utils::pagination::pagination_page_start;
use leptos::prelude::*;

use super::mutations::toggle_workflow_assignment;

#[allow(clippy::too_many_arguments)]
#[component]
pub(crate) fn WorkflowAssignmentMobileCards(
    assignments: Vec<WorkflowAssignmentSummary>,
    total_count: Memo<usize>,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
    selected_detail: RwSignal<Option<WorkflowAssignmentSummary>>,
    assignments_signal: RwSignal<Vec<WorkflowAssignmentSummary>>,
    assignments_loading: RwSignal<bool>,
    assignments_error: RwSignal<Option<String>>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="forms-list-mobile-cards workflow-assignment-mobile-cards">
            {move || if assignments.is_empty() {
                view! { <p class="forms-list-mobile-empty">"No Workflow Assignments to Display"</p> }.into_any()
            } else {
                assignments
                    .iter()
                    .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                    .take(page_size.get())
                    .cloned()
                    .map(|assignment| {
                        let workflow_href = format!("/workflows/{}", assignment.workflow_id);
                        let node_href = format!("/organization/{}", assignment.node_id);
                        let state_label = workflow_assignment_state_label(&assignment);
                        let state_key = workflow_assignment_state(&assignment);
                        let status_key = workflow_assignment_status_key(&assignment);
                        let status_label = workflow_assignment_status_label(&assignment);
                        let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                        let assignment_for_toggle = assignment.clone();
                        let assignment_for_detail = assignment.clone();
                        view! {
                            <article class="forms-list-mobile-card workflow-assignment-mobile-card">
                                <div class="forms-list-mobile-card__header">
                                    <div class="forms-list-mobile-card__title-row">
                                        <h3><a href=workflow_href>{assignment.workflow_name}</a></h3>
                                    </div>
                                </div>
                                <dl>
                                    <div>
                                        <dt>"Assignee"</dt>
                                        <dd>
                                            <span>{assignment.account_display_name}</span>
                                            <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                        </dd>
                                    </div>
                                    <div>
                                        <dt>"Form"</dt>
                                        <dd>{assignment.form_name}</dd>
                                    </div>
                                    <div>
                                        <dt>"Node"</dt>
                                        <dd><a href=node_href>{assignment.node_name}</a></dd>
                                    </div>
                                    <div>
                                        <dt>"Step"</dt>
                                        <dd>{assignment.workflow_step_title}</dd>
                                    </div>
                                    <div>
                                        <dt>"Work State"</dt>
                                        <dd><span class=status_badge_class(state_key)>{state_label}</span></dd>
                                    </div>
                                    <div>
                                        <dt>"Status"</dt>
                                        <dd><span class=status_badge_class(status_key)>{status_label}</span></dd>
                                    </div>
                                    <div>
                                        <dt>"Assigned"</dt>
                                        <dd><Timestamp value=assignment.created_at/></dd>
                                    </div>
                                </dl>
                                <div class="workflow-assignment-mobile-card__actions">
                                    <button
                                        class="button button--compact"
                                        type="button"
                                        on:click=move |_| selected_detail.set(Some(assignment_for_detail.clone()))
                                    >
                                        "View Details"
                                    </button>
                                    <button
                                        class="button button--compact"
                                        type="button"
                                        on:click=move |_| {
                                            toggle_workflow_assignment(
                                                assignment_for_toggle.clone(),
                                                assignments_signal,
                                                assignments_loading,
                                                assignments_error,
                                                message,
                                            );
                                        }
                                    >
                                        {action_label}
                                    </button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}
        </div>
    }
}
