use leptos::prelude::*;
use leptos_router::{path, MatchNestedRoutes, SsrMode};
use leptos_router::components::Route;

use crate::features::operations::OperationsPage;

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

pub fn operation_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/operations") view=OperationsPage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
