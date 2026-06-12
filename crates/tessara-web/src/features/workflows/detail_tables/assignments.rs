//! Workflow detail assignments table.

use crate::features::shared::status_badge_class;
use crate::features::workflows::assignments::{
    WorkflowAssignmentDetailSheet, WorkflowAssignmentSummary, toggle_workflow_assignment,
};
use crate::features::workflows::{
    workflow_assignment_state, workflow_assignment_state_label, workflow_assignment_status_key,
    workflow_assignment_status_label,
};
use crate::ui::{DataTable, DropdownMenu, Timestamp};
use icons::{PanelRight, X};
use leptos::prelude::*;

#[component]
pub(in crate::features::workflows) fn WorkflowDetailAssignmentsTable(
    assignments: Vec<WorkflowAssignmentSummary>,
) -> impl IntoView {
    let assignments_signal = RwSignal::new(assignments);
    let selected_detail = RwSignal::new(None::<WorkflowAssignmentSummary>);
    let assignments_loading = RwSignal::new(false);
    let assignments_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);
    let close_detail = move |_| selected_detail.set(None);

    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Assignee"</th>
                    <th class="data-table__cell--center" scope="col">"Work State"</th>
                    <th class="data-table__cell--center" scope="col">"Status"</th>
                    <th scope="col">"Assigned"</th>
                    <th class="data-table__cell--center" scope="col">"Actions"</th>
                </tr>
            </thead>
            <tbody>
                {move || {
                    let assignments = assignments_signal.get();
                    if assignments.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="5">"No Assignments to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        assignments
                            .into_iter()
                            .map(|assignment| {
                                let state_key = workflow_assignment_state(&assignment);
                                let state_label = workflow_assignment_state_label(&assignment);
                                let status_key = workflow_assignment_status_key(&assignment);
                                let status_label = workflow_assignment_status_label(&assignment);
                                let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                                let assignment_for_detail = assignment.clone();
                                let assignment_for_toggle = assignment.clone();
                                view! {
                                    <tr>
                                        <th scope="row">
                                            <span>{assignment.account_display_name.clone()}</span>
                                            <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                        </th>
                                        <td class="data-table__cell--center">
                                            <span class=status_badge_class(state_key)>{state_label}</span>
                                        </td>
                                        <td class="data-table__cell--center">
                                            <span class=status_badge_class(status_key)>{status_label}</span>
                                        </td>
                                        <td><Timestamp value=assignment.created_at/></td>
                                        <td class="data-table__cell--center">
                                            <DropdownMenu label=format!("Open actions for {}", assignment.account_display_name)>
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
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </tbody>
        </DataTable>
        {move || selected_detail.get().map(|assignment| {
            view! { <WorkflowAssignmentDetailSheet assignment on_close=close_detail/> }
        })}
    }
}
