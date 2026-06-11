//! Full application shell layout.
//!
//! This module owns the page frame that combines top navigation, side navigation, mobile navigation, overlays, and feature page content.

use leptos::prelude::*;

use super::{sidebar::Sidebar, top_app_bar::TopAppBar};
use crate::features::auth;

#[component]
/// Renders the app shell view.
pub fn AppShell(
    active_route: &'static str,
    title: &'static str,
    children: Children,
) -> impl IntoView {
    auth::guards::require_authenticated_route(active_route);

    view! {
        <main class="app-shell">
            <Sidebar active_route/>
            <section class="app-main" aria-label="Application content">
                <TopAppBar active_route title/>
                <div class="app-page">
                    {children()}
                </div>
            </section>
        </main>
    }
}
