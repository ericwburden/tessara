//! Route definitions for the Responses feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in tessara-web-responses.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::routes::PRIMARY_SSR_MODE;
use crate::types::route_params::{SubmissionRouteParams, require_route_params};
use crate::ui::AppShell;
use tessara_web_responses::{
    ResponseDetailContent, ResponseEditContent, ResponseStartContent, ResponsesIndexContent,
};

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

#[component]
fn ResponsesPage() -> impl IntoView {
    view! {
        <AppShell active_route="responses" title="Responses">
            <ResponsesIndexContent/>
        </AppShell>
    }
}

#[component]
fn ResponsesNewPage() -> impl IntoView {
    view! {
        <AppShell active_route="responses" title="Start Response">
            <ResponseStartContent/>
        </AppShell>
    }
}

#[component]
fn ResponsesDetailPage() -> impl IntoView {
    let params = require_route_params::<SubmissionRouteParams>();

    view! {
        <AppShell active_route="responses" title="Response Detail">
            <ResponseDetailContent submission_id=params.submission_id/>
        </AppShell>
    }
}

#[component]
fn ResponsesEditPage() -> impl IntoView {
    let params = require_route_params::<SubmissionRouteParams>();

    view! {
        <AppShell active_route="responses" title="Edit Response">
            <ResponseEditContent submission_id=params.submission_id/>
        </AppShell>
    }
}
