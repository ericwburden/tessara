//! Workflow assignment table row.

use crate::features::shared::status_badge_class;
use crate::features::workflows::assignments::WorkflowAssignmentSummary;
use crate::features::workflows::{
    workflow_assignment_state, workflow_assignment_state_label, workflow_assignment_status_key,
    workflow_assignment_status_label,
};
use crate::ui::{DropdownMenu, Timestamp};
use icons::{PanelRight, X};
use leptos::prelude::*;

use super::mutations::toggle_workflow_assignment;

#[allow(clippy::too_many_arguments)]
#[component]
pub(in crate::features::workflows) fn WorkflowAssignmentTableRow(
    assignment: WorkflowAssignmentSummary,
    selected_detail: RwSignal<Option<WorkflowAssignmentSummary>>,
    assignments_signal: RwSignal<Vec<WorkflowAssignmentSummary>>,
    assignments_loading: RwSignal<bool>,
    assignments_error: RwSignal<Option<String>>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let workflow_href = format!("/workflows/{}", assignment.workflow_id);
    let state_label = workflow_assignment_state_label(&assignment);
    let state_key = workflow_assignment_state(&assignment);
    let status_key = workflow_assignment_status_key(&assignment);
    let status_label = workflow_assignment_status_label(&assignment);
    let action_label = if assignment.is_active {
        "Deactivate"
    } else {
        "Activate"
    };
    let assignment_for_toggle = assignment.clone();
    let assignment_for_detail = assignment.clone();

    view! {
        <tr>
            <th scope="row">
                <a class="data-table__primary-link" href=workflow_href>{assignment.workflow_name.clone()}</a>
            </th>
            <td>
                <span>{assignment.account_display_name}</span>
                <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
            </td>
            <td class="data-table__cell--center">
                <span class=status_badge_class(state_key)>{state_label}</span>
            </td>
            <td class="data-table__cell--center">
                <span class=status_badge_class(status_key)>{status_label}</span>
            </td>
            <td><Timestamp value=assignment.created_at/></td>
            <td class="data-table__cell--center">
                <DropdownMenu label=format!("Open actions for {}", assignment.workflow_name)>
                    <button
                        class="dropdown-menu__item"
                        type="button"
                        role="menuitem"
                        on:click=move |_| selected_detail.set(Some(assignment_for_detail.clone()))
                    >
                        <PanelRight class="dropdown-menu__item-icon"/>
                        <span>"View Details"</span>
                    </button>
                    <button
                        class="dropdown-menu__item"
                        type="button"
                        role="menuitem"
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
                        <X class="dropdown-menu__item-icon"/>
                        <span>{action_label}</span>
                    </button>
                </DropdownMenu>
            </td>
        </tr>
    }
}
