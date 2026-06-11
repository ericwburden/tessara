//! Owns the routes::login module behavior.

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
