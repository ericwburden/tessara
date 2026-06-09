use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::core::LoginPage;

use crate::routes::PRIMARY_SSR_MODE;

pub fn login_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/login") view=LoginPage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
