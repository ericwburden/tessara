//! Full application shell layout.
//!
//! This module owns the page frame that combines top navigation, side navigation, mobile navigation, overlays, and feature page content.

use leptos::prelude::*;
use std::sync::Arc;

use super::nav::SidebarContent;
use crate::features::auth;
use tessara_module_ui::ApplicationShell;

#[component]
pub fn AppShell(
    active_route: &'static str,
    #[prop(into)] title: String,
    children: Children,
) -> impl IntoView {
    auth::guards::require_authenticated_route(active_route);

    view! {
        <ApplicationShell
            title
            navigation=Arc::new(move || view! { <SidebarContent active_route/> }.into_any())
        >
            {children()}
        </ApplicationShell>
    }
}
