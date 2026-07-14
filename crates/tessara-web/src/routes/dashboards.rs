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
    DashboardCreateContent, DashboardDetailContent, DashboardEditorContent, DashboardViewerContent,
    DashboardsIndexContent,
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
            <DashboardsIndexContent/>
        </AppShell>
    }
}

#[component]
fn DashboardCreatePage() -> impl IntoView {
    view! {
        <AppShell active_route="dashboards" title="Create Dashboard">
            <DashboardCreateContent/>
        </AppShell>
    }
}

#[component]
fn DashboardDetailPage() -> impl IntoView {
    let dashboard_id = require_route_params::<DashboardRouteParams>().dashboard_id;
    view! {
        <AppShell active_route="dashboards" title="Dashboard Detail">
            <DashboardDetailContent dashboard_id/>
        </AppShell>
    }
}

#[component]
fn DashboardEditorPage() -> impl IntoView {
    let dashboard_id = require_route_params::<DashboardRouteParams>().dashboard_id;
    view! {
        <AppShell active_route="dashboards" title="Edit Dashboard">
            <DashboardEditorContent dashboard_id/>
        </AppShell>
    }
}

#[component]
fn DashboardViewerPage() -> impl IntoView {
    let dashboard_id = require_route_params::<DashboardRouteParams>().dashboard_id;
    view! {
        <AppShell active_route="dashboards" title="Dashboard Viewer">
            <DashboardViewerContent dashboard_id/>
        </AppShell>
    }
}
