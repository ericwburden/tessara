//! Workflow assignment UI components.

use crate::features::workflows::assignments::{
    WorkflowAssignmentDetailSheet, WorkflowAssignmentMobileCards, WorkflowAssignmentSummary,
    WorkflowAssignmentTableRow,
};
use crate::ui::{DataTable, TableFilterHeader, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use icons::Search;
use leptos::prelude::*;

#[component]
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
                                    view! {
                                        <WorkflowAssignmentTableRow
                                            assignment
                                            selected_detail
                                            assignments_signal
                                            assignments_loading
                                            assignments_error
                                            message
                                        />
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
            <WorkflowAssignmentMobileCards
                assignments=card_assignments
                total_count
                page_size
                page_index
                selected_detail
                assignments_signal
                assignments_loading
                assignments_error
                message
            />
        </div>
        {move || selected_detail.get().map(|assignment| view! {
            <WorkflowAssignmentDetailSheet assignment on_close=close_detail/>
        })}
    }
}
