use crate::ui::components::AppShell;
use crate::ui::components::EmptyState;
use leptos::prelude::*;

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
