//! Related workflow table for form detail pages.

use crate::features::forms::FormWorkflowLink;
use crate::features::shared::status_badge_class;
use crate::features::workflows::{WorkflowSourceMarker, workflow_revision_label_from_option};
use crate::ui::{SearchableDataTable, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::{sentence_label, text_matches};
use leptos::prelude::*;

#[component]
pub(crate) fn FormRelatedWorkflowsTable(workflows: Vec<FormWorkflowLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let workflows_for_filter = workflows;
    let filtered_workflows = Memo::new(move |_| {
        let query = search.get();
        workflows_for_filter
            .iter()
            .filter(|workflow| {
                text_matches(
                    &query,
                    &[
                        &workflow.name,
                        &workflow.slug,
                        workflow
                            .current_version_label
                            .as_deref()
                            .unwrap_or_default(),
                        workflow.current_status.as_deref().unwrap_or_default(),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_workflows.get().len());

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search workflows" placeholder="Search related workflows" search>
                <thead>
                    <tr>
                        <th scope="col">"Workflow"</th>
                        <th scope="col">"Revision"</th>
                        <th scope="col">"Status"</th>
                        <th class="data-table__cell--center" scope="col">"Assignments"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_workflows.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="4">"No Related Workflows to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            let total_count = rows.len();
                            let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                            rows
                                .iter()
                                .skip(start)
                                .take(page_size.get())
                                .cloned()
                                .map(|workflow| {
                                    let href = format!("/workflows/{}", workflow.id);
                                    let status = workflow.current_status.clone().unwrap_or_else(|| "none".to_string());
                                    let workflow_source = workflow.source.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=href>{workflow.name}</a>
                                                <WorkflowSourceMarker source=workflow_source/>
                                                <small class="workflow-assignment-step-meta">{workflow.slug}</small>
                                            </th>
                                            <td>{workflow_revision_label_from_option(workflow.current_version_label)}</td>
                                            <td><span class=status_badge_class(&status)>{sentence_label(&status)}</span></td>
                                            <td class="data-table__cell--center">{workflow.assignment_count.to_string()}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </SearchableDataTable>
            <TablePaginationFooter
                aria_label="Related workflows table pagination"
                item_label="related workflows"
                total_count=total_count
                page_size=page_size
                page_index=page_index
            />
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_workflows.get();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Workflows to Display"</p> }.into_any()
                    } else {
                        let total_count = rows.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        rows
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|workflow| {
                                let href = format!("/workflows/{}", workflow.id);
                                let status = workflow.current_status.clone().unwrap_or_else(|| "none".to_string());
                                let workflow_source = workflow.source.clone();
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4>
                                                <a href=href>{workflow.name}</a>
                                                <WorkflowSourceMarker source=workflow_source/>
                                            </h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Slug"</dt>
                                                <dd>{workflow.slug}</dd>
                                            </div>
                                            <div>
                                                <dt>"Revision"</dt>
                                                <dd>{workflow_revision_label_from_option(workflow.current_version_label)}</dd>
                                            </div>
                                            <div>
                                                <dt>"Status"</dt>
                                                <dd><span class=status_badge_class(&status)>{sentence_label(&status)}</span></dd>
                                            </div>
                                            <div>
                                                <dt>"Assignments"</dt>
                                                <dd>{workflow.assignment_count.to_string()}</dd>
                                            </div>
                                        </dl>
                                    </article>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}
