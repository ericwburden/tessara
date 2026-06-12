//! Workflow assignment status table for Operations.

mod filtering;
mod mobile_card;
mod row;

use crate::features::operations::types::WorkflowAssignmentStatus;
use crate::features::shared::unique_filter_options;
use crate::ui::{DataTable, EmptyState, TableFilterHeader, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use icons::Search;
use leptos::prelude::*;

use filtering::filtered_workflow_assignments;
use mobile_card::WorkflowAssignmentMobileCard;
use row::WorkflowAssignmentRow;

#[component]
pub(crate) fn WorkflowAssignmentsTable(
    assignments: Vec<WorkflowAssignmentStatus>,
) -> impl IntoView {
    let all_assignments = assignments.clone();
    let search = RwSignal::new(String::new());
    let node_filter = RwSignal::new("all".to_string());
    let assignee_filter = RwSignal::new("all".to_string());
    let status_filter = RwSignal::new("all".to_string());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let node_options = unique_filter_options(
        all_assignments
            .iter()
            .map(|assignment| assignment.node_name.clone()),
    );
    let assignee_options = unique_filter_options(
        assignments
            .iter()
            .map(|assignment| assignment.assignee_display_name.clone()),
    );
    let status_options = unique_filter_options(
        assignments
            .iter()
            .map(|assignment| assignment.assignment_status.clone()),
    );
    let filtered_assignments = Memo::new(move |_| {
        filtered_workflow_assignments(
            &all_assignments,
            &search.get(),
            &node_filter.get(),
            &assignee_filter.get(),
            &status_filter.get(),
        )
    });
    let total_count = Memo::new(move |_| filtered_assignments.get().len());

    view! {
        <section class="route-panel__section operations-table-section" aria-label="Workflow assignments">
            <h3>"Workflow Assignments"</h3>
            {if assignments.is_empty() {
                view! {
                    <EmptyState
                        title="No workflow assignments to display"
                        message="No workflow assignments are visible for the current account."
                    />
                }
                .into_any()
            } else {
                view! {
                    <div class="searchable-data-table operations-status-table operations-responsive-table">
                        <div class="searchable-data-table__toolbar forms-list__toolbar">
                            <label class="searchable-data-table__search searchable-data-table__control">
                                <Search class="searchable-data-table__control-icon"/>
                                <span class="sr-only">"Search workflow assignments"</span>
                                <input
                                    type="search"
                                    placeholder="Search workflow assignments"
                                    prop:value=move || search.get()
                                    on:input=move |event| {
                                        search.set(event_target_value(&event));
                                        page_index.set(0);
                                    }
                                />
                            </label>
                        </div>
                        <DataTable>
                            <thead>
                                <tr>
                                    <th scope="col">"Workflow"</th>
                                    <th scope="col">
                                        <TableFilterHeader
                                            label="Node"
                                            all_label="All Nodes"
                                            filter=node_filter
                                            options=node_options.clone()
                                            always_searchable=true
                                        />
                                    </th>
                                    <th scope="col">
                                        <TableFilterHeader
                                            label="Assignee"
                                            all_label="All Assignees"
                                            filter=assignee_filter
                                            options=assignee_options.clone()
                                            always_searchable=true
                                        />
                                    </th>
                                    <th class="data-table__cell--center" scope="col">
                                        <TableFilterHeader
                                            label="Status"
                                            all_label="All Statuses"
                                            filter=status_filter
                                            options=status_options.clone()
                                        />
                                    </th>
                                    <th scope="col">"Current step"</th>
                                    <th class="data-table__cell--center" scope="col">"Responses"</th>
                                    <th scope="col">"Started"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || filtered_assignments.get()
                                    .into_iter()
                                    .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                                    .take(page_size.get())
                                    .map(|instance| {
                                        view! { <WorkflowAssignmentRow instance/> }
                                    })
                                    .collect_view()}
                            </tbody>
                        </DataTable>
                        <TablePaginationFooter
                            aria_label="Workflow assignments table pagination"
                            item_label="workflow assignments"
                            total_count=total_count
                            page_size=page_size
                            page_index=page_index
                        />
                        <div class="operations-mobile-cards">
                            {move || {
                                let visible_assignments = filtered_assignments.get();
                                if visible_assignments.is_empty() {
                                    view! { <p class="related-work-mobile-empty">"No Workflow Assignments to Display"</p> }.into_any()
                                } else {
                                    visible_assignments
                                        .into_iter()
                                        .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                                        .take(page_size.get())
                                        .map(|instance| view! { <WorkflowAssignmentMobileCard instance/> })
                                        .collect_view()
                                        .into_any()
                                }
                            }}
                        </div>
                    </div>
                }
                .into_any()
            }}
        </section>
    }
}
