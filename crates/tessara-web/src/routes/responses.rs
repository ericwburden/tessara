//! Route definitions for the Responses feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in features::responses.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::responses::{
    ResponsesDetailPage, ResponsesEditPage, ResponsesNewPage, ResponsesPage,
};

use crate::routes::PRIMARY_SSR_MODE;

/// Handles the response routes behavior.
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
