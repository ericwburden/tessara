use leptos::prelude::*;
use leptos_meta::{Meta, Title};

#[cfg(feature = "hydrate")]
use web_sys::window;

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct TransitionalPage {
    pub title: &'static str,
    pub description: &'static str,
    pub body_html: String,
    pub page_key: &'static str,
    pub active_route: &'static str,
    pub record_id: Option<String>,
}

pub(crate) fn extract_app_root(document: String) -> String {
    let start = document
        .find(crate::pipeline::APP_ROOT_START)
        .map(|idx| idx + crate::pipeline::APP_ROOT_START.len())
        .expect("document should include app root start marker");
    let end = document[start..]
        .find(crate::pipeline::APP_ROOT_END)
        .map(|idx| start + idx)
        .expect("document should include app root end marker");

    document[start..end].to_string()
}

pub(crate) fn render_transitional_route(page: TransitionalPage) -> impl IntoView {
    #[cfg(feature = "hydrate")]
    {
        let page_key = page.page_key.to_string();
        let active_route = page.active_route.to_string();
        let record_id = page.record_id.clone();

        Effect::new(move |_| {
            let Some(document) = window().and_then(|window| window.document()) else {
                return;
            };
            let Some(body) = document.body() else {
                return;
            };

            body.set_class_name("tessara-app");
            let _ = body.set_attribute("data-page-key", &page_key);
            let _ = body.set_attribute("data-active-route", &active_route);
            if let Some(record_id) = &record_id {
                let _ = body.set_attribute("data-record-id", record_id);
            } else {
                let _ = body.remove_attribute("data-record-id");
            }
        });
    }

    let html = page.body_html;

    view! {
        <Title text=page.title/>
        <Meta name="description" content=page.description/>
        <div inner_html=html></div>
    }
}
