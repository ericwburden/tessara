use leptos::prelude::*;
use leptos_router::{path, MatchNestedRoutes, SsrMode};
use leptos_router::components::Route;

use crate::features::core::LoginPage;

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

pub fn login_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/login") view=LoginPage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
