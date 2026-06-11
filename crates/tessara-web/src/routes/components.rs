//! Route definitions for the Components feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in features::components.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::components::{ComponentsDetailPage, ComponentsPage};

use crate::routes::PRIMARY_SSR_MODE;

/// Handles the component routes behavior.
pub fn component_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/components")
                view=ComponentsPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/components/:component_ref")
                view=ComponentsDetailPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}
