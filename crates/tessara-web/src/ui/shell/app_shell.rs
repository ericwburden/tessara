//! Full application shell layout.
//!
//! This module owns the page frame that combines top navigation, side navigation, mobile navigation, overlays, and feature page content.

use leptos::prelude::*;

use super::{sidebar::Sidebar, top_app_bar::TopAppBar};
use crate::features::auth;

#[component]
pub fn AppShell(
    active_route: &'static str,
    #[prop(into)] title: String,
    children: Children,
) -> impl IntoView {
    auth::guards::require_authenticated_route(active_route);

    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let document_title = title.clone();
        Effect::new(move |_| {
            if let Some(document) = web_sys::window().and_then(|window| window.document()) {
                document.set_title(&format!("{document_title} · Tessara"));
            }
        });
    }

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
