use leptos::prelude::*;

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};
use crate::infra::routing::{SubmissionRouteParams, require_route_params};

#[component]
pub fn ResponsesListPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Responses",
        description: "Tessara responses list screen.",
        body_html: extract_app_root(crate::responses_application_shell_html()),
        page_key: "responses-list",
        active_route: "responses",
        record_id: None,
    })
}

#[component]
pub fn SubmissionsPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Responses",
        description: "Tessara response workspace.",
        body_html: extract_app_root(crate::submission_application_shell_html()),
        page_key: "responses-list",
        active_route: "responses",
        record_id: None,
    })
}

#[component]
pub fn ResponseCreatePage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Create Response",
        description: "Start a Tessara response.",
        body_html: extract_app_root(crate::response_create_application_html()),
        page_key: "response-create",
        active_route: "responses",
        record_id: None,
    })
}

#[component]
pub fn ResponseDetailPage() -> impl IntoView {
    let SubmissionRouteParams { submission_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Response Detail",
        description: "Inspect a Tessara response.",
        body_html: extract_app_root(crate::response_detail_application_html(&submission_id)),
        page_key: "response-detail",
        active_route: "responses",
        record_id: Some(submission_id),
    })
}

#[component]
pub fn ResponseEditPage() -> impl IntoView {
    let SubmissionRouteParams { submission_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Edit Response",
        description: "Edit a Tessara response.",
        body_html: extract_app_root(crate::response_edit_application_html(&submission_id)),
        page_key: "response-edit",
        active_route: "responses",
        record_id: Some(submission_id),
    })
}
