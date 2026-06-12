//! Route definitions for the Operations feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in features::operations.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::operations::OperationsPage;

use crate::routes::PRIMARY_SSR_MODE;

pub fn operation_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/operations") view=OperationsPage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
