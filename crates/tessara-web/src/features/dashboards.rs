use leptos::prelude::*;

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};
use crate::infra::routing::{DashboardRouteParams, require_route_params};

#[component]
pub fn DashboardsPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Dashboards",
        description: "Tessara dashboards list screen.",
        body_html: extract_app_root(crate::dashboards_application_shell_html()),
        page_key: "dashboard-list",
        active_route: "dashboards",
        record_id: None,
    })
}

#[component]
pub fn DashboardCreatePage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Create Dashboard",
        description: "Create a Tessara dashboard.",
        body_html: extract_app_root(crate::dashboard_create_application_html()),
        page_key: "dashboard-create",
        active_route: "dashboards",
        record_id: None,
    })
}

#[component]
pub fn DashboardDetailPage() -> impl IntoView {
    let DashboardRouteParams { dashboard_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Dashboard Detail",
        description: "Inspect a Tessara dashboard.",
        body_html: extract_app_root(crate::dashboard_detail_application_html(&dashboard_id)),
        page_key: "dashboard-detail",
        active_route: "dashboards",
        record_id: Some(dashboard_id),
    })
}

#[component]
pub fn DashboardEditPage() -> impl IntoView {
    let DashboardRouteParams { dashboard_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Edit Dashboard",
        description: "Edit a Tessara dashboard.",
        body_html: extract_app_root(crate::dashboard_edit_application_html(&dashboard_id)),
        page_key: "dashboard-edit",
        active_route: "dashboards",
        record_id: Some(dashboard_id),
    })
}
