use leptos::prelude::*;
use leptos_router::{path, MatchNestedRoutes, SsrMode};
use leptos_router::components::Route;

use crate::features::components::{ComponentsDetailPage, ComponentsPage};

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

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
