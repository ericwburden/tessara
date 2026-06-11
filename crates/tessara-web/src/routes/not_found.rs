//! Not-found route definition.
//!
//! Keep the fallback page route boundary here and delegate any reusable missing-state UI to shared UI modules.

use crate::ui::AppShell;
use crate::ui::EmptyState;
use leptos::prelude::*;

#[component]
/// Renders the not found page view.
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <AppShell active_route="home" title="Not Found">
            <section class="route-panel">
                <EmptyState title="Route not found" message="This route is not part of the Tessara interface."/>
            </section>
        </AppShell>
    }
}
