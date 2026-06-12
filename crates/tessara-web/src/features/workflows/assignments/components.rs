//! Workflow assignment UI components.

use crate::features::shared::status_badge_class;
use crate::features::workflows::assignments::{
    WorkflowAssignmentDetailSheet, WorkflowAssignmentSummary,
};
use crate::features::workflows::{
    toggle_workflow_assignment, workflow_assignment_state, workflow_assignment_state_label,
    workflow_assignment_status_key, workflow_assignment_status_label,
};
use crate::ui::{DataTable, DropdownMenu, TableFilterHeader, TablePaginationFooter, Timestamp};
use crate::utils::pagination::pagination_page_start;
use icons::{PanelRight, Search, X};
use leptos::prelude::*;

#[component]
/// Renders the workflow assignments list view.
pub(in crate::features::workflows) fn WorkflowAssignmentsList(
    assignments: Vec<WorkflowAssignmentSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    state_filter: RwSignal<String>,
    assignee_filter: RwSignal<String>,
    assignee_options: Vec<String>,
    assignments_signal: RwSignal<Vec<WorkflowAssignmentSummary>>,
    assignments_loading: RwSignal<bool>,
    assignments_error: RwSignal<Option<String>>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let mut table_assignments = assignments.clone();
    table_assignments.sort_by(|left, right| {
        left.workflow_name
            .to_lowercase()
            .cmp(&right.workflow_name.to_lowercase())
            .then(
                left.account_display_name
                    .to_lowercase()
                    .cmp(&right.account_display_name.to_lowercase()),
            )
            .then(left.id.cmp(&right.id))
    });
    let card_assignments = table_assignments.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count_value = table_assignments.len();
    let total_count = Memo::new(move |_| total_count_value);
    let selected_detail = RwSignal::new(None::<WorkflowAssignmentSummary>);
    let close_detail = move |_| selected_detail.set(None);

    view! {
        <div class="forms-list forms-list-responsive-table workflow-assignments-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search assignments"</span>
                        <input
                            type="search"
                            placeholder="Search assignments"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Workflow"</th>
                            <th scope="col">
                                <TableFilterHeader
                                    label="Assignee"
                                    all_label="All Assignees"
                                    filter=assignee_filter
                                    options=assignee_options
                                    always_searchable=true
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <TableFilterHeader
                                    label="Work State"
                                    all_label="All States"
                                    filter=state_filter
                                    options=vec!["pending".into(), "draft".into(), "submitted".into()]
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <TableFilterHeader
                                    label="Status"
                                    all_label="All Statuses"
                                    filter=status_filter
                                    options=vec!["active".into(), "inactive".into()]
                                />
                            </th>
                            <th scope="col">"Assigned"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || if table_assignments.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="6">"No Workflow Assignments to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_assignments
                                .iter()
                                .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                                .take(page_size.get())
                                .cloned()
                                .map(|assignment| {
                                    let workflow_href = format!("/workflows/{}", assignment.workflow_id);
                                    let state_label = workflow_assignment_state_label(&assignment);
                                    let state_key = workflow_assignment_state(&assignment);
                                    let status_key = workflow_assignment_status_key(&assignment);
                                    let status_label = workflow_assignment_status_label(&assignment);
                                    let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
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
                                })
                                .collect_view()
                                .into_any()
                        }}
                    </tbody>
                </DataTable>
                <TablePaginationFooter
                    aria_label="Workflow assignments table pagination"
                    item_label="workflow assignments"
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                />
            </div>
            <div class="forms-list-mobile-cards workflow-assignment-mobile-cards">
                {move || if card_assignments.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Workflow Assignments to Display"</p> }.into_any()
                } else {
                    card_assignments
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
        </div>
        {move || selected_detail.get().map(|assignment| view! {
            <WorkflowAssignmentDetailSheet assignment on_close=close_detail/>
        })}
    }
}
