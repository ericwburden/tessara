//! Owns the routes::operations module behavior.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::operations::OperationsPage;

use crate::routes::PRIMARY_SSR_MODE;

/// Handles the operation routes behavior.
pub fn operation_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/operations") view=OperationsPage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
