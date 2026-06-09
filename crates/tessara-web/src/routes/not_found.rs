use leptos::prelude::*;
use crate::ui::components::EmptyState;
use crate::ui::components::AppShell;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <AppShell active_route="home" title="Not Found">
            <section class="route-panel">
                <EmptyState title="Route not found" message="This route is not part of the Tessara interface."/>
            </section>
        </AppShell>
    }
}
