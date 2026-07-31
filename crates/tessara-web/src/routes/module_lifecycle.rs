//! Core-owned routes for lifecycle-v1 module surfaces.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::module_lifecycle::ModuleLifecyclePage;
use crate::routes::PRIMARY_SSR_MODE;

pub fn module_lifecycle_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/dashboards/*path")
                view=ModuleLifecyclePage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}
