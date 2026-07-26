//! Route definitions for the Dashboards feature.
//!
//! Keep URL nesting, route parameters, shell/session policy, and route-to-content
//! wiring here; Dashboard UI behavior belongs in `tessara-web-dashboards`.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::routes::PRIMARY_SSR_MODE;
use crate::types::route_params::{DashboardRouteParams, require_route_params};
use crate::ui::AppShell;
use tessara_web_dashboards::{
    DashboardCreateContent, DashboardDetailContent, DashboardEditorContent,
    DashboardRouteBootstrap, DashboardViewerContent, DashboardsIndexContent,
    dashboard_route_bootstrap,
};

pub fn dashboard_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/dashboards")
                view=DashboardsPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/dashboards/new")
                view=DashboardCreatePage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/dashboards/:dashboard_id/edit")
                view=DashboardEditorPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/dashboards/:dashboard_id/view")
                view=DashboardViewerPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/dashboards/:dashboard_id")
                view=DashboardDetailPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}

#[component]
fn DashboardsPage() -> impl IntoView {
    view! {
        <AppShell active_route="dashboards" title="Dashboards">
            {dashboard_route_content(view! { <DashboardsIndexContent/> })}
        </AppShell>
    }
}

#[component]
fn DashboardCreatePage() -> impl IntoView {
    view! {
        <AppShell active_route="dashboards" title="Create Dashboard">
            {dashboard_route_content(view! { <DashboardCreateContent/> })}
        </AppShell>
    }
}

#[component]
fn DashboardDetailPage() -> impl IntoView {
    let dashboard_id = require_route_params::<DashboardRouteParams>().dashboard_id;
    view! {
        <AppShell active_route="dashboards" title="Dashboard Detail">
            {dashboard_route_content(view! { <DashboardDetailContent dashboard_id/> })}
        </AppShell>
    }
}

#[component]
fn DashboardEditorPage() -> impl IntoView {
    let dashboard_id = require_route_params::<DashboardRouteParams>().dashboard_id;
    view! {
        <AppShell active_route="dashboards" title="Edit Dashboard">
            {dashboard_route_content(view! { <DashboardEditorContent dashboard_id/> })}
        </AppShell>
    }
}

#[component]
fn DashboardViewerPage() -> impl IntoView {
    let dashboard_id = require_route_params::<DashboardRouteParams>().dashboard_id;
    view! {
        <AppShell active_route="dashboards" title="Dashboard Viewer">
            {dashboard_route_content(view! { <DashboardViewerContent dashboard_id/> })}
        </AppShell>
    }
}

fn dashboard_route_content(content: impl IntoView) -> AnyView {
    match dashboard_route_bootstrap() {
        Some(DashboardRouteBootstrap::Unavailable { retry_href, .. }) => view! {
            <section class="route-panel dashboards-page dashboard-module-unavailable">
                <p class="eyebrow">"Dashboard module"</p>
                <h1>"Dashboards are temporarily unavailable"</h1>
                <p>
                    "The Dashboard Module Instance cannot currently be reached. Dashboard data remains in its isolated Module Instance database; Core credentials, browser cookies, configuration, and saved Component references have not been forwarded or replaced."
                </p>
                <div class="button-row">
                    <a class="button" href=retry_href>"Try Dashboards again"</a>
                    <a class="button button--secondary" href="/administration/modules/tessara.dashboards#diagnostics">
                        "Open Module diagnostics"
                    </a>
                </div>
            </section>
        }
        .into_any(),
        _ => content.into_any(),
    }
}
