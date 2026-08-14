#![recursion_limit = "512"]

//! Crate root for the Tessara Leptos frontend.
//!
//! Keep public crate exports, hydration entry points, and native document helpers here; route, feature, UI, and utility behavior should stay in their dedicated modules.

pub mod app;
mod document;
pub mod features;
pub mod http;
mod pipeline;
pub mod routes;
pub mod state;
pub mod types;
pub mod ui;
pub mod utils;

pub use features::modules::{
    MODULE_MANAGEMENT_BOOTSTRAP_SCRIPT_ID, ModuleDetailResponseV1, ModuleInventoryResponseV1,
    ModuleManagementAccessV1, ModuleManagementRouteBootstrapV1, ModuleManagementSurfaceV1,
    NavigationPolicyBootstrapV1, NavigationPolicyResponseV2,
};
pub use state::shell_navigation::{
    ShellNavigationGroupV1, ShellNavigationItemOwnerV1, ShellNavigationItemV1,
    ShellNavigationModeV1, ShellNavigationResponseV1, ShellNavigationStateV1,
    ShellNavigationUnavailableV1,
};
/// DOM id for the request-scoped, actor-filtered shell navigation payload.
pub const SHELL_NAVIGATION_BOOTSTRAP_SCRIPT_ID: &str = "tessara-shell-navigation-bootstrap";

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

/// Renders one authenticated Module Management route with its authorization-
/// filtered request bootstrap shared by SSR content and hydration.
pub fn application_html_with_module_management_bootstrap(
    path: &str,
    title: &str,
    description: &str,
    bootstrap: &ModuleManagementRouteBootstrapV1,
) -> String {
    document::render_native_app_document_with_module_management_bootstrap(
        title,
        description,
        path,
        bootstrap,
    )
}

/// Renders one authenticated Module Management route with its route state and
/// actor-filtered shell navigation sourced from the same server request.
pub fn application_html_with_module_management_and_shell_navigation_bootstrap(
    path: &str,
    title: &str,
    description: &str,
    bootstrap: &ModuleManagementRouteBootstrapV1,
    shell_navigation: &ShellNavigationResponseV1,
) -> String {
    document::render_native_app_document_with_module_management_and_shell_navigation_bootstrap(
        title,
        description,
        path,
        bootstrap,
        shell_navigation,
    )
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
    document::svg_asset(name)
}

#[cfg(feature = "ssr")]
pub fn static_asset(name: &str) -> Option<(&'static str, &'static str)> {
    document::static_asset(name).map(|asset| (asset.content, asset.content_type))
}

#[cfg(test)]
mod tests {
    use super::application_html;

    fn initialize_test_executor() {
        let _ = any_spawner::Executor::init_futures_executor();
    }

    #[test]
    fn native_document_has_overlay_root_and_no_app_prefix() {
        initialize_test_executor();
        let html = application_html("/", "Tessara Home", "Native Tessara shell.");

        assert!(html.contains(r#"<div id="app-overlays"></div>"#));
        assert!(html.contains(r#"<div id="app-root">"#));
        assert!(!html.contains("/app/"));
    }

    #[test]
    /// Verifies the login is registered as root level route behavior.
    fn login_is_registered_as_root_level_route() {
        initialize_test_executor();
        let html = application_html("/login", "Tessara Sign In", "Sign in.");

        assert!(html.contains("Sign In"));
        assert!(html.contains(r#"<form class="login-form""#));
        assert!(html.contains(r#"href="/""#));
    }

    #[test]
    fn lifecycle_host_route_is_policy_neutral_and_covers_unowned_paths() {
        initialize_test_executor();
        for path in [
            "/example",
            "/example/new",
            "/example/item/edit",
            "/datasets",
            "/datasets/new",
        ] {
            let html = application_html(path, "Module", "Lifecycle module host.");
            assert!(html.contains(r#"id="tessara-module-outlet""#), "{path}");
            assert!(!html.contains("data-module-definition"), "{path}");
        }
    }

    #[test]
    fn lifecycle_host_source_contains_no_product_specific_policy() {
        for source in [
            include_str!("routes/module_lifecycle.rs"),
            include_str!("features/module_lifecycle.rs"),
            include_str!("app.rs"),
        ] {
            for forbidden in [
                "tessara.components",
                "tessara.dashboards",
                "\"/components",
                "\"/dashboards",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "lifecycle host source must not contain product policy {forbidden}"
                );
            }
        }
    }
}
