//! Route-level page composition for the Home feature.

use leptos::prelude::*;

use crate::ui::{AppShell, PageHeader};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <AppShell active_route="home" title="Home">
            <section class="route-panel home-page">
                <PageHeader title="Home"/>
            </section>
        </AppShell>
    }
}
