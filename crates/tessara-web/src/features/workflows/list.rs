//! List view components for the Workflows feature.
//!
//! Keep collection tables, list filters, and list-page presentation here; detail/editor flows should stay in their dedicated modules.

use crate::features::organization::OrganizationNode;
use crate::features::shared::{
    WorkflowAssignedUsersSheetData, WorkflowAvailableNodesSheetData, status_badge_class,
};
use crate::features::workflows::types::WorkflowSummary;
use crate::features::workflows::{
    WorkflowAssignedUsersList, WorkflowAssignedUsersSheet, WorkflowAvailableNodesList,
    WorkflowAvailableNodesSheet, WorkflowSourceMarker, workflow_assigned_user_links,
    workflow_available_node_links, workflow_status_key, workflow_status_label,
    workflow_version_label,
};
use crate::ui::FilterHeader as SharedFilterHeader;
use crate::ui::{DataTable, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use icons::Search;
use leptos::prelude::*;

pub(crate) use crate::features::workflows::pages::list::WorkflowsPage;

#[component]
/// Renders the workflows list view.
pub(in crate::features::workflows) fn WorkflowsList(
    workflows: Vec<WorkflowSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    status_options: Vec<String>,
    organization_nodes: Vec<OrganizationNode>,
) -> impl IntoView {
    let mut table_workflows = workflows.clone();
    table_workflows.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then(left.id.cmp(&right.id))
    });
    let card_workflows = table_workflows.clone();
    let _ = organization_nodes;
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_workflows.len();
    let total_count = Memo::new(move |_| total_count);
    let available_nodes_sheet = RwSignal::new(None::<WorkflowAvailableNodesSheetData>);
    let assigned_users_sheet = RwSignal::new(None::<WorkflowAssignedUsersSheetData>);

    view! {
        <div class="forms-list forms-list-responsive-table">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search workflows"</span>
                        <input
                            type="search"
                            placeholder="Search workflows"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Workflow name"</th>
                            <th scope="col">"Available at"</th>
                            <th class="data-table__cell--center" scope="col">"Active revision"</th>
                            <th class="data-table__cell--center" scope="col">
                                <SharedFilterHeader
                                    label="Status"
                                    all_label="All statuses"
                                    filter=status_filter
                                    options=status_options
                                />
                            </th>
                            <th scope="col">"Active assignments"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || if table_workflows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="5">"No Workflows to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_workflows
                                .iter()
                                .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                                .take(page_size.get())
                                .cloned()
                                .map(|workflow| {
                                    let workflow_href = format!("/workflows/{}", workflow.id);
                                    let status_key = workflow_status_key(&workflow).to_string();
                                    let status_label = workflow_status_label(&workflow);
                                    let version_label = workflow_version_label(&workflow);
                                    let available_at = workflow_available_node_links(&workflow.available_nodes);
                                    let assigned_users = workflow_assigned_user_links(&workflow);
                                    let workflow_name = workflow.name.clone();
                                    let workflow_source = workflow.source.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=workflow_href.clone()>{workflow.name}</a>
                                                <WorkflowSourceMarker source=workflow_source/>
                                            </th>
                                            <td>
                                                <WorkflowAvailableNodesList
                                                    nodes=available_at
                                                    workflow_name=workflow_name.clone()
                                                    workflow_href=workflow_href.clone()
                                                    sheet=available_nodes_sheet
                                                />
                                            </td>
                                            <td class="data-table__cell--center">{version_label}</td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(&status_key)>{status_label}</span>
                                            </td>
                                            <td>
                                                <WorkflowAssignedUsersList
                                                    users=assigned_users
                                                    workflow_name=workflow_name
                                                    workflow_href=workflow_href
                                                    sheet=assigned_users_sheet
                                                />
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
                    aria_label="Workflow table pagination"
                    item_label="workflows"
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                />
            </div>
            <div class="forms-list-mobile-cards">
                {move || if card_workflows.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Workflows to Display"</p> }.into_any()
                } else {
                    card_workflows
                        .iter()
                        .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                        .take(page_size.get())
                        .cloned()
                        .map(|workflow| {
                            let workflow_href = format!("/workflows/{}", workflow.id);
                            let status_key = workflow_status_key(&workflow).to_string();
                            let status_label = workflow_status_label(&workflow);
                            let version_label = workflow_version_label(&workflow);
                            let available_at = workflow_available_node_links(&workflow.available_nodes);
                            let assigned_users = workflow_assigned_user_links(&workflow);
                            let workflow_name = workflow.name.clone();
                            let workflow_source = workflow.source.clone();
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div class="forms-list-mobile-card__title-row">
                                            <h3><a href=workflow_href.clone()>{workflow.name}</a></h3>
                                            <WorkflowSourceMarker source=workflow_source/>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Available at"</dt>
                                            <dd>
                                                <WorkflowAvailableNodesList
                                                    nodes=available_at
                                                    workflow_name=workflow_name.clone()
                                                    workflow_href=workflow_href.clone()
                                                    sheet=available_nodes_sheet
                                                />
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Active revision"</dt>
                                            <dd>{version_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(&status_key)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Active assignments"</dt>
                                            <dd>
                                                <WorkflowAssignedUsersList
                                                    users=assigned_users
                                                    workflow_name=workflow_name
                                                    workflow_href=workflow_href
                                                    sheet=assigned_users_sheet
                                                />
                                            </dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
            <WorkflowAvailableNodesSheet detail=available_nodes_sheet/>
            <WorkflowAssignedUsersSheet detail=assigned_users_sheet/>
        </div>
    }
}
