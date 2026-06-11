//! Owns the routes::home module behavior.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::home::HomePage;

use crate::routes::PRIMARY_SSR_MODE;

/// Handles the home routes behavior.
pub fn home_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/") view=HomePage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
