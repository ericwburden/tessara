use leptos::prelude::*;
use tessara_module_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, EmptyState,
    PageHeader,
};

use crate::types::{Dashboard, DashboardPlacementAvailability};
use crate::{DashboardRouteBootstrap, dashboard_route_bootstrap};

use super::visibility_scope::DashboardVisibilityScopeSheet;

const DASHBOARD_DETAIL_VISIBILITY_SHEET_ID: &str = "dashboard-detail-visibility-scope";

#[component]
pub fn DashboardDetailContent(dashboard_id: String) -> impl IntoView {
    let (initial_dashboard, bootstrapped) = match dashboard_route_bootstrap() {
        Some(DashboardRouteBootstrap::Detail { dashboard, .. }) if dashboard.id == dashboard_id => {
            (Some(dashboard), true)
        }
        _ => (None, false),
    };
    let dashboard = RwSignal::new(initial_dashboard);
    let loading = RwSignal::new(cfg!(feature = "hydrate") && !bootstrapped);
    let error = RwSignal::new(None::<String>);
    let delete_error = RwSignal::new(None::<String>);
    let visibility_sheet_open = RwSignal::new(false);

    Effect::new({
        let dashboard_id = dashboard_id.clone();
        move |_| {
            if !bootstrapped {
                load_detail(dashboard_id.clone(), dashboard, loading, error);
            }
        }
    });

    view! {
        <section class="route-panel dashboards-page dashboard-detail">
            <Breadcrumb>
                <BreadcrumbItem><BreadcrumbLink href="/dashboards">"Dashboards"</BreadcrumbLink></BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem><BreadcrumbPage>"Dashboard Detail"</BreadcrumbPage></BreadcrumbItem>
            </Breadcrumb>

            {move || if loading.get() {
                view! { <EmptyState title="Loading Dashboard" message="Fetching Dashboard metadata and placements."/> }.into_any()
            } else if let Some(message) = error.get() {
                view! { <EmptyState title="Dashboard unavailable" message=message/> }.into_any()
            } else if let Some(loaded) = dashboard.get() {
                let view_href = format!("/dashboards/{}/view", loaded.id);
                let edit_href = format!("/dashboards/{}/edit", loaded.id);
                let delete_id = loaded.id.clone();
                let title = loaded.name.clone();
                let description = loaded.description.clone().unwrap_or_else(|| "No description".into());
                let visibility_nodes = loaded.visibility_nodes.clone();
                let visibility_count = visibility_nodes.len();
                let visibility_label = visibility_count_label(visibility_count);
                let can_manage = loaded.can_manage;
                view! {
                    <PageHeader title description=description>
                        <a class="button button--secondary" href=view_href>"View Dashboard"</a>
                        {can_manage.then(|| view! { <a class="button" href=edit_href>"Edit composition"</a> })}
                        {can_manage.then(|| view! {
                            <button class="button button--danger" type="button" on:click=move |_| {
                                delete_dashboard(delete_id.clone(), delete_error);
                            }>"Delete"</button>
                        })}
                    </PageHeader>
                    {move || delete_error.get().map(|message| view! { <p class="form-error" role="alert">{message}</p> })}
                    <section class="route-panel__section" aria-label="Dashboard summary">
                        <div class="metric-grid">
                            <button
                                id="dashboard-detail-visibility-trigger"
                                class="metric-card metric-card--button"
                                type="button"
                                aria-haspopup="dialog"
                                aria-controls=DASHBOARD_DETAIL_VISIBILITY_SHEET_ID
                                aria-expanded=move || visibility_sheet_open.get().to_string()
                                on:click=move |_| visibility_sheet_open.set(true)
                            >
                                <span>"Visibility"</span>
                                <strong>{visibility_label}</strong>
                            </button>
                            <div class="metric-card">
                                <span>"Placements"</span>
                                <strong>{loaded.placement_count}</strong>
                            </div>
                        </div>
                    </section>
                    <section class="dashboard-detail__layout-section" aria-labelledby="dashboard-layout-heading">
                        <div class="dashboard-section-heading">
                            <h2 id="dashboard-layout-heading">"Saved layout"</h2>
                            <span>{format!("{} total placements", loaded.placement_count)}</span>
                        </div>
                        {if loaded.placements.is_empty() {
                            view! { <EmptyState title="Empty Dashboard" message="This Dashboard has no Component placements yet."/> }.into_any()
                        } else {
                            view! {
                                <div class="dashboard-saved-grid" style="--dashboard-track-size: 52px">
                                    {loaded.placements.into_iter().map(|placement| {
                                        let style = placement_style(
                                            placement.grid_row,
                                            placement.grid_column,
                                            placement.grid_width,
                                            placement.grid_height,
                                        );
                                        let available = placement.availability == DashboardPlacementAvailability::Available;
                                        let title = placement.display_title();
                                        let subtitle = placement.component.as_ref().map(|component| {
                                            format!("{} · {}", kind_label(&component.component_type), super::version_label(component.version_number, &component.version_label))
                                        }).unwrap_or_else(|| "Redacted Component placement".into());
                                        view! {
                                            <article
                                                class:dashboard-placement-card=true
                                                class:is-unavailable=move || !available
                                                style=style
                                                data-placement-id=placement.placement_id
                                            >
                                                <span class="dashboard-placement-card__order">{placement.position + 1}</span>
                                                <div>
                                                    <h3>{title}</h3>
                                                    <p>{subtitle}</p>
                                                </div>
                                                <span class="dashboard-placement-card__size">
                                                    {format!("{} × {}", placement.grid_width, placement.grid_height)}
                                                </span>
                                            </article>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }}
                    </section>
                    <DashboardVisibilityScopeSheet
                        id=DASHBOARD_DETAIL_VISIBILITY_SHEET_ID
                        dashboard_name=loaded.name.clone()
                        nodes=visibility_nodes
                        open=Signal::derive(move || visibility_sheet_open.get())
                        on_close=Callback::new(move |_| visibility_sheet_open.set(false))
                    />
                }.into_any()
            } else {
                view! { <EmptyState title="Dashboard unavailable" message="No Dashboard payload was returned."/> }.into_any()
            }}
        </section>
    }
}

fn visibility_count_label(count: usize) -> String {
    if count == 1 {
        "1 Node".to_string()
    } else {
        format!("{count} Nodes")
    }
}

pub(super) fn placement_style(row: i32, column: i32, width: i32, height: i32) -> String {
    format!("grid-row: {row} / span {height}; grid-column: {column} / span {width};")
}

pub(super) fn kind_label(kind: &str) -> String {
    match kind {
        "stat_card" => "Stat Card".into(),
        other => {
            let mut characters = other.chars();
            characters
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
                .unwrap_or_else(|| "Component".into())
        }
    }
}

#[cfg(feature = "hydrate")]
fn load_detail(
    dashboard_id: String,
    dashboard: RwSignal<Option<Dashboard>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        loading.set(true);
        error.set(None);
        match crate::api::fetch_dashboard(&dashboard_id).await {
            Ok(payload) => dashboard.set(Some(payload)),
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_detail(
    _: String,
    _: RwSignal<Option<Dashboard>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
fn delete_dashboard(dashboard_id: String, error: RwSignal<Option<String>>) {
    let confirmed = web_sys::window()
        .and_then(|window| {
            window
                .confirm_with_message("Delete this Dashboard and all of its placements?")
                .ok()
        })
        .unwrap_or(false);
    if !confirmed {
        return;
    }
    leptos::task::spawn_local(async move {
        match crate::api::delete_dashboard(&dashboard_id).await {
            Ok(()) => {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/dashboards");
                }
            }
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn delete_dashboard(_: String, _: RwSignal<Option<String>>) {}

#[cfg(test)]
mod tests {
    use super::{placement_style, visibility_count_label};

    #[test]
    fn saved_geometry_maps_directly_to_css_grid() {
        assert_eq!(
            placement_style(3, 7, 6, 2),
            "grid-row: 3 / span 2; grid-column: 7 / span 6;"
        );
    }

    #[test]
    fn visibility_disclosure_uses_concise_node_counts() {
        assert_eq!(visibility_count_label(0), "0 Nodes");
        assert_eq!(visibility_count_label(1), "1 Node");
        assert_eq!(visibility_count_label(42), "42 Nodes");
    }
}
