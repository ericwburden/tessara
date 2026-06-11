#![recursion_limit = "512"]

//! Native Leptos SSR frontend for Tessara.

pub mod api;
pub mod app;
mod document;
pub mod features;
mod pipeline;
pub mod routes;
pub mod state;
pub mod types;
pub mod ui;
pub mod utils;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "hydrate")]
#[wasm_bindgen(start)]
/// Handles the start behavior.
pub fn start() {
    console_error_panic_hook::set_once();
    app::hydrate_app(pipeline::APP_ROOT_ID);
}

/// Handles the application html behavior.
pub fn application_html(path: &str, title: &str, description: &str) -> String {
    document::render_native_app_document(title, description, path)
}

/// Handles the css path behavior.
pub fn css_path() -> String {
    pipeline::css_path()
}

/// Handles the js path behavior.
pub fn js_path() -> String {
    pipeline::js_path()
}

/// Handles the pkg dir behavior.
pub fn pkg_dir() -> std::path::PathBuf {
    pipeline::pkg_dir()
}

/// Handles the svg asset behavior.
pub fn svg_asset(name: &str) -> Option<&'static str> {
    document::svg_asset(name)
}

#[cfg(test)]
mod tests {
    use super::application_html;

    #[test]
    /// Handles the native document has overlay root and no app prefix behavior.
    fn native_document_has_overlay_root_and_no_app_prefix() {
        let html = application_html("/", "Tessara Home", "Native Tessara shell.");

        assert!(html.contains(r#"<div id="app-overlays"></div>"#));
        assert!(html.contains(r#"<div id="app-root">"#));
        assert!(!html.contains("/app/"));
    }

    #[test]
    /// Verifies the login is registered as root level route behavior.
    fn login_is_registered_as_root_level_route() {
        let html = application_html("/login", "Tessara Sign In", "Sign in.");

        assert!(html.contains("Sign In"));
        assert!(html.contains(r#"<form class="login-form""#));
        assert!(html.contains(r#"href="/""#));
    }
}
