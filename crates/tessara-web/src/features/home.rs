use leptos::prelude::*;

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};

#[component]
pub fn AdminWorkbenchPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara",
        description: "Tessara local admin workbench for migration setup and workflow testing.",
        body_html: extract_app_root(crate::admin_shell_html()),
        page_key: "admin-shell",
        active_route: "administration",
        record_id: None,
    })
}

#[component]
pub fn HomePage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Home",
        description: "Tessara application home for local replacement workflow testing.",
        body_html: extract_app_root(crate::application_shell_html()),
        page_key: "home",
        active_route: "home",
        record_id: None,
    })
}

#[component]
pub fn LoginPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Sign In",
        description: "Sign in to the Tessara application shell.",
        body_html: extract_app_root(crate::login_application_html()),
        page_key: "login",
        active_route: "login",
        record_id: None,
    })
}
