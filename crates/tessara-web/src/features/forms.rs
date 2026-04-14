use crate::app::transitional::extract_app_root;
use crate::infra::routing::{FormRouteParams, require_route_params};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};

#[cfg(feature = "hydrate")]
fn set_forms_page_context(page_key: &'static str, record_id: Option<String>) {
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
        body.set_attribute("data-active-route", "forms").ok();
        if let Some(record_id) = record_id.as_ref() {
            body.set_attribute("data-record-id", record_id).ok();
        } else {
            body.remove_attribute("data-record-id").ok();
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn set_forms_page_context(_page_key: &'static str, _record_id: Option<String>) {}

#[component]
pub fn FormsListPage() -> impl IntoView {
    set_forms_page_context("form-list", None);
    let html = extract_app_root(crate::forms_application_shell_html());

    view! {
        <Title text="Tessara Forms" />
        <Meta name="description" content="Tessara forms list screen." />
        <div inner_html=html></div>
    }
}

#[component]
pub fn FormCreatePage() -> impl IntoView {
    set_forms_page_context("form-create", None);
    let html = extract_app_root(crate::form_create_application_html());

    view! {
        <Title text="Create Form" />
        <Meta name="description" content="Create a Tessara form." />
        <div inner_html=html></div>
    }
}

#[component]
pub fn FormDetailPage() -> impl IntoView {
    let FormRouteParams { form_id } = require_route_params();
    set_forms_page_context("form-detail", Some(form_id.clone()));
    let html = extract_app_root(crate::form_detail_application_html(&form_id));

    view! {
        <Title text="Form Detail" />
        <Meta name="description" content="Inspect a Tessara form." />
        <div inner_html=html></div>
    }
}

#[component]
pub fn FormEditPage() -> impl IntoView {
    let FormRouteParams { form_id } = require_route_params();
    set_forms_page_context("form-edit", Some(form_id.clone()));
    let html = extract_app_root(crate::form_edit_application_html(&form_id));

    view! {
        <Title text="Edit Form" />
        <Meta name="description" content="Edit a Tessara form." />
        <div inner_html=html></div>
    }
}
