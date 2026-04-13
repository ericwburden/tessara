use leptos::prelude::*;

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};
use crate::infra::routing::{NodeRouteParams, require_route_params};

#[component]
pub fn OrganizationListPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Organizations",
        description: "Tessara organization list screen.",
        body_html: extract_app_root(crate::organization_application_shell_html()),
        page_key: "organization-list",
        active_route: "organization",
        record_id: None,
    })
}

#[component]
pub fn OrganizationCreatePage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Create Organization",
        description: "Create a runtime organization record.",
        body_html: extract_app_root(crate::organization_create_application_html()),
        page_key: "organization-create",
        active_route: "organization",
        record_id: None,
    })
}

#[component]
pub fn OrganizationDetailPage() -> impl IntoView {
    let NodeRouteParams { node_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Organization Detail",
        description: "Organization detail screen.",
        body_html: extract_app_root(crate::organization_detail_application_html(&node_id)),
        page_key: "organization-detail",
        active_route: "organization",
        record_id: Some(node_id),
    })
}

#[component]
pub fn OrganizationEditPage() -> impl IntoView {
    let NodeRouteParams { node_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Edit Organization",
        description: "Edit a runtime organization record.",
        body_html: extract_app_root(crate::organization_edit_application_html(&node_id)),
        page_key: "organization-edit",
        active_route: "organization",
        record_id: Some(node_id),
    })
}
