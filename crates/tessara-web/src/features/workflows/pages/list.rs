//! List view components for the Workflows feature.
//!
//! Keep collection tables, list filters, and list-page presentation here; detail/editor flows should stay in their dedicated modules.

use crate::features::organization::OrganizationNode;
use crate::features::shared::{
    FormAttachmentLink, WorkflowAssignedUsersSheetData, WorkflowAvailableNodesSheetData,
    node_count_label, status_badge_class, unique_filter_options, user_count_label,
};
use crate::features::workflows::types::WorkflowSummary;
use crate::features::workflows::{
    WorkflowSourceMarker, workflow_assigned_user_links, workflow_available_node_links,
    workflow_available_nodes_label, workflow_description_label, workflow_status_key,
    workflow_status_label, workflow_version_label,
};
use crate::features::workflows::{
    load_workflow_assignment_nodes, load_workflows, workflow_assigned_users_label,
};
use crate::ui::{AppShell, Button, DataTable, PageHeader, TablePaginationFooter};
use crate::ui::{FilterHeader as SharedFilterHeader, empty_view};
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::text_matches;
use icons::{ExternalLink, PanelRight, Search, X};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
/// Renders the workflows list view.
fn WorkflowsList(
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

#[component]
/// Renders the workflow available nodes list view.
fn WorkflowAvailableNodesList(
    nodes: Vec<FormAttachmentLink>,
    workflow_name: String,
    workflow_href: String,
    sheet: RwSignal<Option<WorkflowAvailableNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let nodes_for_sheet = nodes.clone();
    let workflow_name_for_sheet = workflow_name.clone();
    let workflow_href_for_sheet = workflow_href.clone();

    view! {
        <div class="forms-attached-list">
            {if total_nodes == 0 {
                view! { <p>"Not available"</p> }.into_any()
            } else {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View available organization nodes for {workflow_name_for_sheet}")
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(WorkflowAvailableNodesSheetData {
                                workflow_name: workflow_name_for_sheet.clone(),
                                workflow_href: workflow_href_for_sheet.clone(),
                                nodes: nodes_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{node_count_label(total_nodes)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            }}
        </div>
    }
}

#[component]
/// Renders the workflow available nodes sheet view.
fn WorkflowAvailableNodesSheet(
    detail: RwSignal<Option<WorkflowAvailableNodesSheetData>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = move |_| {
        detail.set(None);
        search.set(String::new());
    };
    let filtered_nodes = move || {
        let query = search.get().trim().to_lowercase();
        detail
            .get()
            .map(|data| {
                data.nodes
                    .into_iter()
                    .filter(|node| {
                        query.is_empty()
                            || node.label.to_lowercase().contains(&query)
                            || node.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Available organization nodes">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close available nodes" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Available organization nodes">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.workflow_href aria-label="Open workflow detail" title="Open workflow detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(empty_view)
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close available nodes" title="Close available nodes" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.nodes.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Available Nodes"</p>
                                            <h2>{data.workflow_name}</h2>
                                            <span class="forms-attached-sheet__count">{format!("{total} nodes")}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search class="searchable-data-table__control-icon"/>
                                                <span class="sr-only">"Search available nodes"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search available nodes"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let nodes = filtered_nodes();
                                                    if nodes.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Available Nodes to Display"</p> }.into_any()
                                                    } else {
                                                        nodes
                                                            .into_iter()
                                                            .map(|node| {
                                                                let node_title = node.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=node.href title=node_title>
                                                                        <span>{node.label}</span>
                                                                        <small>{node.title}</small>
                                                                    </a>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </section>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(empty_view)
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
/// Renders the workflow assigned users list view.
fn WorkflowAssignedUsersList(
    users: Vec<FormAttachmentLink>,
    workflow_name: String,
    workflow_href: String,
    sheet: RwSignal<Option<WorkflowAssignedUsersSheetData>>,
) -> impl IntoView {
    let total_users = users.len();
    let users_for_sheet = users.clone();
    let workflow_name_for_sheet = workflow_name.clone();
    let workflow_href_for_sheet = workflow_href.clone();

    view! {
        <div class="forms-attached-list">
            {if total_users == 0 {
                view! { <p>"No active assignments"</p> }.into_any()
            } else {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View assigned users for {workflow_name_for_sheet}")
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(WorkflowAssignedUsersSheetData {
                                workflow_name: workflow_name_for_sheet.clone(),
                                workflow_href: workflow_href_for_sheet.clone(),
                                users: users_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{user_count_label(total_users)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            }}
        </div>
    }
}

#[component]
/// Renders the workflow assigned users sheet view.
fn WorkflowAssignedUsersSheet(
    detail: RwSignal<Option<WorkflowAssignedUsersSheetData>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = move |_| {
        detail.set(None);
        search.set(String::new());
    };
    let filtered_nodes = move || {
        let query = search.get().trim().to_lowercase();
        detail
            .get()
            .map(|data| {
                data.users
                    .into_iter()
                    .filter(|user| {
                        query.is_empty()
                            || user.label.to_lowercase().contains(&query)
                            || user.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Assigned users">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close assigned users" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Assigned users">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.workflow_href aria-label="Open workflow detail" title="Open workflow detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(empty_view)
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close assigned users" title="Close assigned users" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.users.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Assigned Users"</p>
                                            <h2>{data.workflow_name}</h2>
                                            <span class="forms-attached-sheet__count">{user_count_label(total)}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search class="searchable-data-table__control-icon"/>
                                                <span class="sr-only">"Search assigned users"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search assigned users"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let users = filtered_nodes();
                                                    if users.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Assigned Users to Display"</p> }.into_any()
                                                    } else {
                                                        users
                                                            .into_iter()
                                                            .map(|user| {
                                                                let user_title = user.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=user.href title=user_title>
                                                                        <span>{user.label}</span>
                                                                        <small>{user.title}</small>
                                                                    </a>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </section>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(empty_view)
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
/// Renders the workflows page view.
pub fn WorkflowsPage() -> impl IntoView {
    let workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflows(workflows, is_loading, load_error);
        load_workflow_assignment_nodes(organization_nodes);
    });

    let filtered_workflows = move || {
        let query = search.get();
        let selected_status = status_filter.get();
        workflows
            .get()
            .into_iter()
            .filter(|workflow| {
                let version_label = workflow_version_label(workflow);
                let status_label = workflow_status_label(workflow);
                let assigned_to = workflow_assigned_users_label(workflow);
                let description = workflow_description_label(workflow);
                let available_at = workflow_available_nodes_label(&workflow.available_nodes);
                text_matches(
                    &query,
                    &[
                        workflow.name.as_str(),
                        workflow.slug.as_str(),
                        description.as_str(),
                        version_label.as_str(),
                        status_label.as_str(),
                        assigned_to.as_str(),
                        available_at.as_str(),
                    ],
                ) && (selected_status == "all" || selected_status == status_label)
            })
            .collect::<Vec<_>>()
    };

    let status_options = move || {
        unique_filter_options(
            workflows
                .get()
                .iter()
                .map(workflow_status_label)
                .collect::<Vec<_>>(),
        )
    };

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <section class="route-panel workflows-page">
                <PageHeader title="Workflows">
                    <Button label="Create Workflow" href="/workflows/new"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading workflows"</h3>
                                <p>"Fetching workflow definitions."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Workflows unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <WorkflowsList
                                workflows=filtered_workflows()
                                search=search
                                status_filter=status_filter
                                status_options=status_options()
                                organization_nodes=organization_nodes.get()
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
