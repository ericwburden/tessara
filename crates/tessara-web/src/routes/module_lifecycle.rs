//! Core-owned routes for lifecycle-v1 module surfaces.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::module_lifecycle::ModuleLifecyclePage;
use crate::routes::PRIMARY_SSR_MODE;

#[component]
fn DashboardLifecyclePage() -> impl IntoView {
    view! {
        <ModuleLifecyclePage
            active_route="dashboards"
            title="Dashboards"
            product_name="Dashboards"
            definition_id="tessara.dashboards"
        />
    }
}

#[component]
fn ComponentLifecyclePage() -> impl IntoView {
    view! {
        <ModuleLifecyclePage
            active_route="components"
            title="Components"
            product_name="Components"
            definition_id="tessara.components"
        />
    }
}

pub fn module_lifecycle_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/dashboards/*path")
                view=DashboardLifecyclePage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/components/*path")
                view=ComponentLifecyclePage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}
