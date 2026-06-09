use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::core::HomePage;

use crate::routes::PRIMARY_SSR_MODE;

pub fn home_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/") view=HomePage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
