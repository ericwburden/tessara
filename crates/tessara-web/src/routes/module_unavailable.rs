//! Core-owned fallback for an independently deployed module that is unavailable.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::routes::PRIMARY_SSR_MODE;
use crate::ui::AppShell;

pub fn module_unavailable_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/module-unavailable")
                view=ModuleUnavailablePage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}

#[component]
fn ModuleUnavailablePage() -> impl IntoView {
    view! {
        <AppShell active_route="module_management" title="Module unavailable">
            <section class="route-panel module-unavailable-page">
                <div class="organization-detail-card empty-state">
                    <h1>"Module temporarily unavailable"</h1>
                    <p>"The requested module is not ready. Tessara Core and its administration surfaces remain available."</p>
                    <a class="button" href="/administration/modules">"Open Module Management"</a>
                </div>
            </section>
        </AppShell>
    }
}
