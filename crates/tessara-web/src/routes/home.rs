use leptos::prelude::*;
use leptos_router::{path, MatchNestedRoutes, SsrMode};
use leptos_router::components::Route;

use crate::features::core::HomePage;

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

pub fn home_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/") view=HomePage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
