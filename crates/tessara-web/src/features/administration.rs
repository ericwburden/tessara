use leptos::prelude::*;

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};

#[component]
pub fn AdministrationPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Administration",
        description: "Tessara internal administration landing page.",
        body_html: extract_app_root(crate::administration_application_shell_html()),
        page_key: "administration",
        active_route: "administration",
        record_id: None,
    })
}

#[component]
pub fn LegacyAdminPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara",
        description: "Tessara local admin workbench for migration setup and workflow testing.",
        body_html: extract_app_root(crate::admin_application_shell_html()),
        page_key: "admin-shell",
        active_route: "administration",
        record_id: None,
    })
}