//! Table and summary components for the Operations feature.

use super::display::{
    workflow_assignment_href, workflow_response_summary, workflow_revision_label,
    workflow_step_summary,
};
use super::types::*;
use crate::features::shared::unique_filter_options;
use crate::ui::{DataTable, EmptyState, FilterHeader, StatusBadge, Timestamp};
use crate::utils::{
    pagination::{
        pagination_current_page, pagination_page_count, pagination_page_end, pagination_page_start,
    },
    text::text_matches,
};
use icons::Search;
use leptos::prelude::*;

#[component]
/// Renders the operations summary panel view.
pub(crate) fn OperationsSummaryPanel(
    summary: OperationsSummary,
    reporting_data: ReportingDataStatus,
) -> impl IntoView {
    view! {
        <section class="route-panel__section operations-summary" aria-label="Operations overview">
            <div class="metric-grid operations-action-metrics">
                <OperationsMetric label="Open workflow assignments" value=summary.open_workflow_assignment_count.to_string()/>
                <OperationsMetric label="Draft form responses" value=summary.draft_response_count.to_string()/>
                <OperationsMetric label="Datasets needing attention" value=summary.dataset_attention_count.to_string()/>
                <OperationsMetric label="Reporting data status" value=reporting_data.status.clone()/>
            </div>
        </section>
    }
}

#[component]
/// Renders the operations metric view.
fn OperationsMetric(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="metric-card">
            <span>{label}</span>
            <strong>{value}</strong>
        </div>
    }
}

#[component]
/// Renders the workflow assignments table view.
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
        let query = search.get();
        let selected_node = node_filter.get();
        let selected_assignee = assignee_filter.get();
        let selected_status = status_filter.get();
        all_assignments
            .iter()
            .filter(|assignment| {
                let matches_node = selected_node == "all" || assignment.node_name == selected_node;
                let matches_assignee = selected_assignee == "all"
                    || assignment.assignee_display_name == selected_assignee;
                let matches_status =
                    selected_status == "all" || assignment.assignment_status == selected_status;
                matches_node
                    && matches_assignee
                    && matches_status
                    && text_matches(
                        &query,
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
            .collect::<Vec<_>>()
    });
    let page_count =
        move || pagination_page_count(filtered_assignments.get().len(), page_size.get());
    let current_page = move || {
        pagination_current_page(
            filtered_assignments.get().len(),
            page_size.get(),
            page_index.get(),
        )
    };
    let page_summary = move || {
        let total_count = filtered_assignments.get().len();
        if total_count == 0 {
            "No workflow assignments to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} workflow assignments",
                pagination_page_start(total_count, page_size.get(), page_index.get()) + 1,
                pagination_page_end(total_count, page_size.get(), page_index.get()),
                total_count
            )
        }
    };

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
                                        <FilterHeader
                                            label="Node"
                                            all_label="All Nodes"
                                            filter=node_filter
                                            options=node_options.clone()
                                            always_searchable=true
                                        />
                                    </th>
                                    <th scope="col">
                                        <FilterHeader
                                            label="Assignee"
                                            all_label="All Assignees"
                                            filter=assignee_filter
                                            options=assignee_options.clone()
                                            always_searchable=true
                                        />
                                    </th>
                                    <th class="data-table__cell--center" scope="col">
                                        <FilterHeader
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
                                    .skip(pagination_page_start(filtered_assignments.get().len(), page_size.get(), page_index.get()))
                                    .take(page_size.get())
                                    .map(|instance| {
                                        view! { <WorkflowAssignmentRow instance/> }
                                    })
                                    .collect_view()}
                            </tbody>
                        </DataTable>
                        <div class="directory-table-pagination" aria-label="Workflow assignments table pagination">
                            <p>{move || page_summary()}</p>
                            <div class="directory-table-pagination__actions">
                                <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                                    <span>"Rows"</span>
                                    <select
                                        prop:value=move || page_size.get().to_string()
                                        on:change=move |event| {
                                            if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                                page_size.set(size);
                                                page_index.set(0);
                                            }
                                        }
                                    >
                                        <option value="10">"10"</option>
                                        <option value="25">"25"</option>
                                        <option value="50">"50"</option>
                                    </select>
                                </label>
                                <button
                                    class="button button--compact button--secondary"
                                    type="button"
                                    disabled=move || current_page() == 0
                                    on:click=move |_| {
                                        page_index.update(|page| *page = page.saturating_sub(1));
                                    }
                                >
                                    "Previous"
                                </button>
                                <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                                <button
                                    class="button button--compact button--secondary"
                                    type="button"
                                    disabled=move || { current_page() + 1 >= page_count() }
                                    on:click=move |_| {
                                        let last_page = page_count().saturating_sub(1);
                                        page_index.update(|page| *page = (*page + 1).min(last_page));
                                    }
                                >
                                    "Next"
                                </button>
                            </div>
                        </div>
                        <div class="operations-mobile-cards">
                            {move || {
                                let visible_assignments = filtered_assignments.get();
                                if visible_assignments.is_empty() {
                                    view! { <p class="related-work-mobile-empty">"No Workflow Assignments to Display"</p> }.into_any()
                                } else {
                                    visible_assignments
                                        .into_iter()
                                        .skip(pagination_page_start(filtered_assignments.get().len(), page_size.get(), page_index.get()))
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

#[component]
/// Renders the dataset readiness table view.
pub(crate) fn DatasetReadinessTable(datasets: Vec<DatasetStatus>) -> impl IntoView {
    let all_datasets = datasets.clone();
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let status_options =
        unique_filter_options(datasets.iter().map(|dataset| dataset.readiness.clone()));
    let filtered_datasets = Memo::new(move |_| {
        let query = search.get();
        let selected_status = status_filter.get();
        all_datasets
            .iter()
            .filter(|dataset| {
                let matches_status =
                    selected_status == "all" || dataset.readiness == selected_status;
                matches_status
                    && text_matches(
                        &query,
                        &[
                            dataset.dataset_name.as_str(),
                            dataset.readiness.as_str(),
                            dataset.revision_status.as_str(),
                        ],
                    )
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let page_count = move || pagination_page_count(filtered_datasets.get().len(), page_size.get());
    let current_page = move || {
        pagination_current_page(
            filtered_datasets.get().len(),
            page_size.get(),
            page_index.get(),
        )
    };
    let page_summary = move || {
        let total_count = filtered_datasets.get().len();
        if total_count == 0 {
            "No datasets to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} datasets",
                pagination_page_start(total_count, page_size.get(), page_index.get()) + 1,
                pagination_page_end(total_count, page_size.get(), page_index.get()),
                total_count
            )
        }
    };

    view! {
        <section class="route-panel__section operations-table-section" aria-label="Dataset readiness">
            <h3>"Dataset Readiness"</h3>
            {if datasets.is_empty() {
                view! {
                    <EmptyState
                        title="No visible datasets"
                        message="No dataset readiness information is visible for the current account."
                    />
                }
                .into_any()
            } else {
                view! {
                    <div class="searchable-data-table operations-status-table operations-responsive-table">
                        <div class="searchable-data-table__toolbar forms-list__toolbar">
                            <label class="searchable-data-table__search searchable-data-table__control">
                                <Search class="searchable-data-table__control-icon"/>
                                <span class="sr-only">"Search datasets"</span>
                                <input
                                    type="search"
                                    placeholder="Search datasets"
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
                                    <th scope="col">"Dataset"</th>
                                    <th class="data-table__cell--center" scope="col">
                                        <FilterHeader
                                            label="Status"
                                            all_label="All Statuses"
                                            filter=status_filter
                                            options=status_options.clone()
                                        />
                                    </th>
                                    <th scope="col">"Published version"</th>
                                    <th class="data-table__cell--center" scope="col">"Linked forms"</th>
                                    <th class="data-table__cell--center" scope="col">"Columns"</th>
                                    <th class="data-table__cell--center" scope="col">"Ready responses"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || filtered_datasets.get()
                                    .into_iter()
                                    .skip(pagination_page_start(filtered_datasets.get().len(), page_size.get(), page_index.get()))
                                    .take(page_size.get())
                                    .map(|dataset| {
                                        view! { <DatasetReadinessRow dataset/> }
                                    })
                                    .collect_view()}
                            </tbody>
                        </DataTable>
                        <div class="directory-table-pagination" aria-label="Dataset readiness table pagination">
                            <p>{move || page_summary()}</p>
                            <div class="directory-table-pagination__actions">
                                <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                                    <span>"Rows"</span>
                                    <select
                                        prop:value=move || page_size.get().to_string()
                                        on:change=move |event| {
                                            if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                                page_size.set(size);
                                                page_index.set(0);
                                            }
                                        }
                                    >
                                        <option value="10">"10"</option>
                                        <option value="25">"25"</option>
                                        <option value="50">"50"</option>
                                    </select>
                                </label>
                                <button
                                    class="button button--compact button--secondary"
                                    type="button"
                                    disabled=move || current_page() == 0
                                    on:click=move |_| {
                                        page_index.update(|page| *page = page.saturating_sub(1));
                                    }
                                >
                                    "Previous"
                                </button>
                                <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                                <button
                                    class="button button--compact button--secondary"
                                    type="button"
                                    disabled=move || { current_page() + 1 >= page_count() }
                                    on:click=move |_| {
                                        let last_page = page_count().saturating_sub(1);
                                        page_index.update(|page| *page = (*page + 1).min(last_page));
                                    }
                                >
                                    "Next"
                                </button>
                            </div>
                        </div>
                        <div class="operations-mobile-cards">
                            {move || {
                                let visible_datasets = filtered_datasets.get();
                                if visible_datasets.is_empty() {
                                    view! { <p class="related-work-mobile-empty">"No Datasets to Display"</p> }.into_any()
                                } else {
                                    visible_datasets
                                        .into_iter()
                                        .skip(pagination_page_start(filtered_datasets.get().len(), page_size.get(), page_index.get()))
                                        .take(page_size.get())
                                        .map(|dataset| view! { <DatasetReadinessMobileCard dataset/> })
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

#[component]
/// Renders the workflow assignment row view.
fn WorkflowAssignmentRow(instance: WorkflowAssignmentStatus) -> impl IntoView {
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

#[component]
/// Renders the workflow assignment mobile card view.
fn WorkflowAssignmentMobileCard(instance: WorkflowAssignmentStatus) -> impl IntoView {
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

#[component]
/// Renders the dataset readiness row view.
fn DatasetReadinessRow(dataset: DatasetStatus) -> impl IntoView {
    let dataset_href = format!("/datasets/{}", dataset.dataset_id);
    view! {
        <tr>
            <th scope="row">
                <a class="data-table__primary-link" href=dataset_href>{dataset.dataset_name}</a>
            </th>
            <td class="data-table__cell--center"><StatusBadge label=dataset.readiness.clone()/></td>
            <td>{dataset.revision_status.clone()}</td>
            <td class="data-table__cell--center">{dataset.source_count}</td>
            <td class="data-table__cell--center">{dataset.field_count}</td>
            <td class="data-table__cell--center">{dataset.ready_response_count}</td>
        </tr>
    }
}

#[component]
/// Renders the dataset readiness mobile card view.
fn DatasetReadinessMobileCard(dataset: DatasetStatus) -> impl IntoView {
    let dataset_href = format!("/datasets/{}", dataset.dataset_id);
    view! {
        <article class="related-work-mobile-card operations-mobile-card">
            <header class="related-work-mobile-card__header">
                <a href=dataset_href>{dataset.dataset_name.clone()}</a>
            </header>
            <dl>
                <div>
                    <dt>"Status"</dt>
                    <dd><StatusBadge label=dataset.readiness.clone()/></dd>
                </div>
                <div>
                    <dt>"Published version"</dt>
                    <dd>{dataset.revision_status.clone()}</dd>
                </div>
                <div>
                    <dt>"Linked forms"</dt>
                    <dd>{dataset.source_count}</dd>
                </div>
                <div>
                    <dt>"Columns"</dt>
                    <dd>{dataset.field_count}</dd>
                </div>
                <div>
                    <dt>"Ready responses"</dt>
                    <dd>{dataset.ready_response_count}</dd>
                </div>
            </dl>
        </article>
    }
}
