use crate::app::transitional::extract_app_root;
use crate::infra::routing::{NodeRouteParams, require_route_params};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

#[cfg(feature = "hydrate")]
fn set_organization_page_context(page_key: &'static str, record_id: Option<String>) {
    use web_sys::window;

    Effect::new(move |_| {
        let Some(document) = window().and_then(|window| window.document()) else {
            return;
        };
        let Some(body) = document.body() else {
            return;
        };
        body.set_class_name("tessara-app");
        body.set_attribute("data-page-key", page_key).ok();
        body.set_attribute("data-active-route", "organization").ok();
        if let Some(record_id) = record_id.as_ref() {
            body.set_attribute("data-record-id", record_id).ok();
        } else {
            body.remove_attribute("data-record-id").ok();
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn set_organization_page_context(_page_key: &'static str, _record_id: Option<String>) {}

#[component]
pub fn OrganizationListPage() -> impl IntoView {
    set_organization_page_context("organization-list", None);
    let html = extract_app_root(crate::organization_application_shell_html());

    view! {
        <Title text="Tessara Organizations" />
        <Meta name="description" content="Tessara organization list screen." />
        <div inner_html=html></div>
    }
}

#[component]
pub fn OrganizationCreatePage() -> impl IntoView {
    set_organization_page_context("organization-create", None);
    let html = extract_app_root(crate::organization_create_application_html());

    view! {
        <Title text="Create Organization" />
        <Meta name="description" content="Create a runtime organization record." />
        <div inner_html=html></div>
    }
}

#[component]
pub fn OrganizationDetailPage() -> impl IntoView {
    let NodeRouteParams { node_id } = require_route_params();
    set_organization_page_context("organization-detail", Some(node_id.clone()));
    let html = extract_app_root(crate::organization_detail_application_html(&node_id));

    view! {
        <Title text="Organization Detail" />
        <Meta name="description" content="Organization detail screen." />
        <div inner_html=html></div>
    }
}

#[component]
pub fn OrganizationEditPage() -> impl IntoView {
    let NodeRouteParams { node_id } = require_route_params();
    set_organization_page_context("organization-edit", Some(node_id.clone()));
    let html = extract_app_root(crate::organization_edit_application_html(&node_id));

    view! {
        <Title text="Edit Organization" />
        <Meta name="description" content="Edit a runtime organization record." />
        <div inner_html=html></div>
    }
}
