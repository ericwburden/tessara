#![recursion_limit = "512"]
#![allow(
    clippy::collapsible_if,
    clippy::manual_div_ceil,
    clippy::needless_return,
    clippy::redundant_closure,
    clippy::redundant_iter_cloned,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unit_arg
)]

//! Native Leptos SSR frontend for Tessara.

pub mod app;
mod brand;
mod document;
pub mod features;
pub mod infra;
mod pipeline;
mod theme;
pub mod ui;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "hydrate")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    app::hydrate_app(pipeline::APP_ROOT_ID);
}

pub fn application_html(path: &str, title: &str, description: &str) -> String {
    document::render_native_app_document(title, description, path)
}

pub fn css_path() -> String {
    pipeline::css_path()
}

pub fn js_path() -> String {
    pipeline::js_path()
}

pub fn pkg_dir() -> std::path::PathBuf {
    pipeline::pkg_dir()
}

pub fn svg_asset(name: &str) -> Option<&'static str> {
    brand::svg_asset(name)
}

#[cfg(test)]
mod tests {
    use super::application_html;

    #[test]
    fn native_document_has_overlay_root_and_no_app_prefix() {
        let html = application_html("/", "Tessara Home", "Native Tessara shell.");

        assert!(html.contains(r#"<div id="app-overlays"></div>"#));
        assert!(html.contains(r#"<div id="app-root">"#));
        assert!(!html.contains("/app/"));
    }

    #[test]
    fn login_is_registered_as_root_level_route() {
        let html = application_html("/login", "Tessara Sign In", "Sign in.");

        assert!(html.contains("Sign In"));
        assert!(html.contains(r#"<form class="login-form""#));
        assert!(html.contains(r#"href="/""#));
    }
}
