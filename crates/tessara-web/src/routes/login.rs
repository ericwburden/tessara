//! Route definitions for the Login feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in features::login.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::login::LoginPage;

use crate::routes::PRIMARY_SSR_MODE;

/// Handles the login routes behavior.
pub fn login_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/login") view=LoginPage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
