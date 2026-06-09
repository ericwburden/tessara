use leptos::prelude::*;
use leptos_router::{path, MatchNestedRoutes, SsrMode};
use leptos_router::components::Route;

use crate::features::responses::{ResponsesDetailPage, ResponsesEditPage, ResponsesNewPage, ResponsesPage};

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

pub fn response_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/responses") view=ResponsesPage ssr=PRIMARY_SSR_MODE/>
            <Route
                path=path!("/responses/new")
                view=ResponsesNewPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/responses/:submission_id")
                view=ResponsesDetailPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/responses/:submission_id/edit")
                view=ResponsesEditPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}
