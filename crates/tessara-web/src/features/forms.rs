use leptos::prelude::*;

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};
use crate::infra::routing::{FormRouteParams, require_route_params};

#[component]
pub fn FormsListPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Forms",
        description: "Tessara forms list screen.",
        body_html: extract_app_root(crate::forms_application_shell_html()),
        page_key: "form-list",
        active_route: "forms",
        record_id: None,
    })
}

#[component]
pub fn FormCreatePage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Create Form",
        description: "Create a Tessara form.",
        body_html: extract_app_root(crate::form_create_application_html()),
        page_key: "form-create",
        active_route: "forms",
        record_id: None,
    })
}

#[component]
pub fn FormDetailPage() -> impl IntoView {
    let FormRouteParams { form_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Form Detail",
        description: "Inspect a Tessara form.",
        body_html: extract_app_root(crate::form_detail_application_html(&form_id)),
        page_key: "form-detail",
        active_route: "forms",
        record_id: Some(form_id),
    })
}

#[component]
pub fn FormEditPage() -> impl IntoView {
    let FormRouteParams { form_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Edit Form",
        description: "Edit a Tessara form.",
        body_html: extract_app_root(crate::form_edit_application_html(&form_id)),
        page_key: "form-edit",
        active_route: "forms",
        record_id: Some(form_id),
    })
}
