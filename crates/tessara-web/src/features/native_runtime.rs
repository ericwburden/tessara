#![cfg_attr(not(feature = "hydrate"), allow(dead_code, unused_imports))]

use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use gloo_net::http::Request;

#[cfg(feature = "hydrate")]
use serde::de::DeserializeOwned;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[cfg(feature = "hydrate")]
use web_sys::{
    Document, HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement,
    UrlSearchParams, window,
};

pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(feature = "hydrate")]
fn document() -> Option<Document> {
    window().and_then(|window| window.document())
}

#[cfg(feature = "hydrate")]
pub fn set_page_context(
    page_key: &'static str,
    active_route: &'static str,
    record_id: Option<String>,
) {
    Effect::new(move |_| {
        let Some(document) = document() else {
            return;
        };
        let Some(body) = document.body() else {
            return;
        };
        body.set_class_name("tessara-app");
        body.set_attribute("data-page-key", page_key).ok();
        body.set_attribute("data-active-route", active_route).ok();
        if let Some(record_id) = record_id.as_ref() {
            body.set_attribute("data-record-id", record_id).ok();
        } else {
            body.remove_attribute("data-record-id").ok();
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub fn set_page_context(
    _page_key: &'static str,
    _active_route: &'static str,
    _record_id: Option<String>,
) {
}

#[cfg(feature = "hydrate")]
pub fn by_id(id: &str) -> Option<web_sys::Element> {
    document()?.get_element_by_id(id)
}

#[cfg(feature = "hydrate")]
pub fn set_html(id: &str, html: &str) {
    if let Some(element) = by_id(id) {
        element.set_inner_html(html);
    }
}

#[cfg(feature = "hydrate")]
pub fn set_text(id: &str, text: &str) {
    if let Some(element) = by_id(id) {
        element.set_text_content(Some(text));
    }
}

#[cfg(feature = "hydrate")]
pub fn input_value(id: &str) -> Option<String> {
    by_id(id)
        .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
        .map(|input| input.value())
}

#[cfg(feature = "hydrate")]
pub fn select_value(id: &str) -> Option<String> {
    by_id(id)
        .and_then(|element| element.dyn_into::<HtmlSelectElement>().ok())
        .map(|input| input.value())
}

#[cfg(feature = "hydrate")]
pub fn textarea_value(id: &str) -> Option<String> {
    by_id(id)
        .and_then(|element| element.dyn_into::<HtmlTextAreaElement>().ok())
        .map(|input| input.value())
}

#[cfg(feature = "hydrate")]
pub fn set_input_value(id: &str, value: &str) {
    if let Some(input) = by_id(id).and_then(|element| element.dyn_into::<HtmlInputElement>().ok()) {
        input.set_value(value);
    }
}

#[cfg(feature = "hydrate")]
pub fn set_select_value(id: &str, value: &str) {
    if let Some(input) = by_id(id).and_then(|element| element.dyn_into::<HtmlSelectElement>().ok())
    {
        input.set_value(value);
    }
}

#[cfg(feature = "hydrate")]
pub fn set_textarea_value(id: &str, value: &str) {
    if let Some(input) =
        by_id(id).and_then(|element| element.dyn_into::<HtmlTextAreaElement>().ok())
    {
        input.set_value(value);
    }
}

#[cfg(feature = "hydrate")]
pub fn current_search_param(key: &str) -> Option<String> {
    let location = window()?.location();
    let search = location.search().ok()?;
    let params = UrlSearchParams::new_with_str(&search).ok()?;
    params.get(key)
}

#[cfg(feature = "hydrate")]
pub fn redirect(path: &str) {
    let _ = window().and_then(|window| window.location().set_href(path).ok().map(|_| window));
}

#[cfg(feature = "hydrate")]
pub async fn get_json<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    Request::get(path)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json::<T>()
        .await
        .map_err(|error| error.to_string())
}

#[cfg(feature = "hydrate")]
pub async fn get_text(path: &str) -> Result<String, String> {
    Request::get(path)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .text()
        .await
        .map_err(|error| error.to_string())
}

#[cfg(feature = "hydrate")]
pub async fn post_json<T: DeserializeOwned>(
    path: &str,
    body: &serde_json::Value,
) -> Result<T, String> {
    Request::post(path)
        .json(body)
        .map_err(|error| error.to_string())?
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json::<T>()
        .await
        .map_err(|error| error.to_string())
}

#[cfg(feature = "hydrate")]
pub async fn put_json<T: DeserializeOwned>(
    path: &str,
    body: &serde_json::Value,
) -> Result<T, String> {
    Request::put(path)
        .json(body)
        .map_err(|error| error.to_string())?
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json::<T>()
        .await
        .map_err(|error| error.to_string())
}

#[cfg(feature = "hydrate")]
pub async fn delete_json<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    Request::delete(path)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json::<T>()
        .await
        .map_err(|error| error.to_string())
}

#[cfg(feature = "hydrate")]
pub fn html_element(id: &str) -> Option<HtmlElement> {
    by_id(id)?.dyn_into::<HtmlElement>().ok()
}
