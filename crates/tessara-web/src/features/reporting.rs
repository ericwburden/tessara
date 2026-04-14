use leptos::prelude::*;

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};
use crate::infra::routing::{ReportRouteParams, require_route_params};

#[component]
pub fn ReportsPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Reports",
        description: "Tessara reports list screen.",
        body_html: extract_app_root(crate::reporting_application_shell_html()),
        page_key: "report-list",
        active_route: "reports",
        record_id: None,
    })
}

#[component]
pub fn ReportCreatePage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Create Report",
        description: "Create a Tessara report.",
        body_html: extract_app_root(crate::report_create_application_html()),
        page_key: "report-create",
        active_route: "reports",
        record_id: None,
    })
}

#[component]
pub fn ReportDetailPage() -> impl IntoView {
    let ReportRouteParams { report_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Report Detail",
        description: "Inspect a Tessara report.",
        body_html: extract_app_root(crate::report_detail_application_html(&report_id)),
        page_key: "report-detail",
        active_route: "reports",
        record_id: Some(report_id),
    })
}

#[component]
pub fn ReportEditPage() -> impl IntoView {
    let ReportRouteParams { report_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Edit Report",
        description: "Edit a Tessara report.",
        body_html: extract_app_root(crate::report_edit_application_html(&report_id)),
        page_key: "report-edit",
        active_route: "reports",
        record_id: Some(report_id),
    })
}
