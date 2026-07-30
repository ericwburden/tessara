use icons::{Eye, Pencil};
use leptos::prelude::*;
use tessara_module_ui::{DataTable, EmptyState, PageHeader, TablePaginationFooter, TableSearch};

use crate::types::{DashboardSummary, SessionAccount};
use crate::{DashboardRouteBootstrap, dashboard_route_bootstrap};

use super::visibility_scope::DashboardVisibilityScopeSheet;

const DASHBOARD_VISIBILITY_SCOPE_DIALOG_ID: &str = "dashboard-visibility-scope";

#[component]
pub fn DashboardsIndexContent() -> impl IntoView {
    let (initial_account, initial_dashboards, bootstrapped) = match dashboard_route_bootstrap() {
        Some(DashboardRouteBootstrap::Directory {
            account,
            dashboards,
        }) => (Some(account), dashboards, true),
        _ => (None, Vec::new(), false),
    };
    let dashboards = RwSignal::new(initial_dashboards);
    let account = RwSignal::new(initial_account);
    let loading = RwSignal::new(cfg!(feature = "hydrate") && !bootstrapped);
    let error = RwSignal::new(None::<String>);
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10_usize);
    let page_index = RwSignal::new(0_usize);
    let selected_scope = RwSignal::new(None::<DashboardSummary>);

    Effect::new(move |_| {
        if !bootstrapped {
            load_directory(dashboards, account, loading, error);
        }
    });

    let filtered = Memo::new(move |_| {
        let query = search.get().trim().to_lowercase();
        dashboards
            .get()
            .into_iter()
            .filter(|dashboard| {
                query.is_empty()
                    || dashboard.name.to_lowercase().contains(&query)
                    || dashboard
                        .description
                        .as_deref()
                        .is_some_and(|description| description.to_lowercase().contains(&query))
                    || dashboard.visibility_nodes.iter().any(|node| {
                        node.node_path.to_lowercase().contains(&query)
                            || node.node_type_name.to_lowercase().contains(&query)
                    })
            })
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered.get().len());
    let paged = Memo::new(move |_| {
        let dashboards = filtered.get();
        let page_size = page_size.get().max(1);
        let page_count = dashboards.len().max(1).div_ceil(page_size);
        let page = page_index.get().min(page_count.saturating_sub(1));
        let start = page * page_size;
        let end = (start + page_size).min(dashboards.len());
        dashboards[start..end].to_vec()
    });

    view! {
        <section class="route-panel dashboards-page dashboards-directory">
            <PageHeader
                title="Dashboards"
                description="Compose and view operational reporting from pinned Component versions."
            >
                {move || account.get().filter(SessionAccount::can_manage_dashboards).map(|_| {
                    view! { <a class="button" href="/dashboards/new">"Create Dashboard"</a> }
                })}
            </PageHeader>

            {move || if loading.get() {
                view! { <EmptyState title="Loading Dashboards" message="Fetching visible Dashboard metadata."/> }.into_any()
            } else if let Some(message) = error.get() {
                view! { <EmptyState title="Dashboards unavailable" message=message/> }.into_any()
            } else if dashboards.get().is_empty() {
                view! { <EmptyState title="No visible Dashboards" message="No Dashboards are visible for the current account."/> }.into_any()
            } else {
                view! {
                    <div class="searchable-data-table dashboard-directory__surface">
                    <div class="searchable-data-table__toolbar dashboard-directory__toolbar">
                        <TableSearch
                            value=Signal::derive(move || search.get())
                            on_input=Callback::new(move |value| {
                                search.set(value);
                                page_index.set(0);
                            })
                            label="Search Dashboards"
                            placeholder="Search Dashboards"
                        />
                        <span class="dashboard-directory__count" aria-live="polite">
                            {move || format!("{} shown", filtered.get().len())}
                        </span>
                    </div>
                    {move || if filtered.get().is_empty() {
                        view! { <EmptyState title="No matching Dashboards" message="Try a different search."/> }.into_any()
                    } else {
                        view! {
                            <DataTable>
                                <thead>
                                    <tr>
                                        <th scope="col">"Dashboard"</th>
                                        <th scope="col">"Visibility"</th>
                                        <th scope="col">"Placements"</th>
                                        <th scope="col" class="data-table__actions data-table__cell--center">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || paged.get().into_iter().map(|dashboard| {
                                        let detail_href = format!("/dashboards/{}", dashboard.id);
                                        let view_href = format!("{detail_href}/view");
                                        let edit_href = format!("{detail_href}/edit");
                                        let description = dashboard.description.clone().unwrap_or_else(|| "No description".into());
                                        let visibility_count = dashboard.visibility_nodes.len();
                                        let visibility_label = if visibility_count == 1 {
                                            "1 node".to_string()
                                        } else {
                                            format!("{visibility_count} nodes")
                                        };
                                        let scope_dashboard = dashboard.clone();
                                        let scope_dashboard_id = dashboard.id.clone();
                                        let scope_button_id = format!("dashboard-scope-{}", dashboard.id);
                                        let can_manage = dashboard.can_manage;
                                        let view_label = format!("View {}", dashboard.name);
                                        let edit_label = format!("Edit {}", dashboard.name);
                                        view! {
                                            <tr data-dashboard-id=dashboard.id>
                                                <th scope="row" class="data-table__stacked-label">
                                                    <a href=detail_href><strong>{dashboard.name}</strong></a>
                                                    <span class="data-table__secondary-text">{description}</span>
                                                </th>
                                                <td>
                                                    <span class="dashboard-directory__mobile-label">"Visibility"</span>
                                                    {if visibility_count == 0 {
                                                        view! { <span>"No visibility scope"</span> }.into_any()
                                                    } else {
                                                        view! {
                                                            <button
                                                                id=scope_button_id
                                                                class="link-button dashboard-directory__scope-trigger"
                                                                type="button"
                                                                aria-label=format!("View {visibility_label} in {} visibility scope", scope_dashboard.name)
                                                                aria-haspopup="dialog"
                                                                aria-controls=DASHBOARD_VISIBILITY_SCOPE_DIALOG_ID
                                                                aria-expanded=move || selected_scope
                                                                    .get()
                                                                    .as_ref()
                                                                    .is_some_and(|selected| selected.id == scope_dashboard_id)
                                                                    .to_string()
                                                                on:click=move |_| selected_scope.set(Some(scope_dashboard.clone()))
                                                            >{visibility_label.clone()}</button>
                                                        }.into_any()
                                                    }}
                                                </td>
                                                <td>
                                                    <span class="dashboard-directory__mobile-label">"Placements"</span>
                                                    {dashboard.placement_count}
                                                </td>
                                                <td class="data-table__actions data-table__cell--center">
                                                    <span class="dashboard-directory__mobile-label">"Actions"</span>
                                                    <div class="data-table__action-group dashboard-directory__row-actions">
                                                        <a class="icon-button" href=view_href aria-label=view_label.clone() title=view_label>
                                                            <Eye class="icon-button__icon"/>
                                                        </a>
                                                        {can_manage.then(|| view! {
                                                            <a class="icon-button" href=edit_href aria-label=edit_label.clone() title=edit_label>
                                                                <Pencil class="icon-button__icon"/>
                                                            </a>
                                                        })}
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </DataTable>
                            <TablePaginationFooter
                                aria_label="Dashboards table pagination"
                                item_label="Dashboards"
                                empty_item_label="Dashboards"
                                total_count
                                page_size
                                page_index
                            />
                        }.into_any()
                    }}
                    </div>
                }.into_any()
            }}
            {move || selected_scope.get().map(|dashboard| {
                let nodes = dashboard.visibility_nodes;
                let dashboard_name = dashboard.name;
                view! {
                    <DashboardVisibilityScopeSheet
                        id=DASHBOARD_VISIBILITY_SCOPE_DIALOG_ID
                        dashboard_name
                        nodes
                        open=Signal::derive(|| true)
                        on_close=Callback::new(move |_| selected_scope.set(None))
                    />
                }
            })}
        </section>
    }
}

#[cfg(feature = "hydrate")]
fn load_directory(
    dashboards: RwSignal<Vec<DashboardSummary>>,
    account: RwSignal<Option<SessionAccount>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        loading.set(true);
        error.set(None);
        match crate::api::fetch_account().await {
            Ok(payload) => account.set(Some(payload)),
            Err(message) => error.set(Some(message)),
        }
        match crate::api::fetch_dashboards().await {
            Ok(payload) => dashboards.set(payload),
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_directory(
    _: RwSignal<Vec<DashboardSummary>>,
    _: RwSignal<Option<SessionAccount>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}
