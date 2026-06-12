//! Mobile card rendering for the workflows list.

use crate::features::shared::{
    WorkflowAssignedUsersSheetData, WorkflowAvailableNodesSheetData, status_badge_class,
};
use crate::features::workflows::types::WorkflowSummary;
use crate::features::workflows::{
    WorkflowAssignedUsersList, WorkflowAvailableNodesList, WorkflowSourceMarker,
    workflow_assigned_user_links, workflow_available_node_links, workflow_status_key,
    workflow_status_label, workflow_version_label,
};
use crate::utils::pagination::pagination_page_start;
use leptos::prelude::*;

#[component]
pub(super) fn WorkflowsMobileCards(
    workflows: Vec<WorkflowSummary>,
    total_count: Memo<usize>,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
    available_nodes_sheet: RwSignal<Option<WorkflowAvailableNodesSheetData>>,
    assigned_users_sheet: RwSignal<Option<WorkflowAssignedUsersSheetData>>,
) -> impl IntoView {
    view! {
        <div class="forms-list-mobile-cards">
            {move || if workflows.is_empty() {
                view! { <p class="forms-list-mobile-empty">"No Workflows to Display"</p> }.into_any()
            } else {
                workflows
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
    }
}
