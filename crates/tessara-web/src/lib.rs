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
    ShellNavigationResponseV1, ShellNavigationStateV1, ShellNavigationUnavailableV1,
};
pub use tessara_web_dashboards::{
    Dashboard, DashboardComponentVersion, DashboardComponentVersionOption, DashboardComposition,
    DashboardPlacement, DashboardPlacementAvailability, DashboardPlacementIdMapping,
    DashboardRouteBootstrap, DashboardSummary, DashboardVisibilityNode, SessionAccount,
    VisibilityNodeOption, dashboard_route_bootstrap,
};

/// DOM id for the request-scoped Dashboard hydration payload.
pub const DASHBOARD_BOOTSTRAP_SCRIPT_ID: &str = "tessara-dashboard-bootstrap";

/// DOM id for the request-scoped, actor-filtered shell navigation payload.
pub const SHELL_NAVIGATION_BOOTSTRAP_SCRIPT_ID: &str = "tessara-shell-navigation-bootstrap";

/// DOM id for module-provided content rendered inside the Core shell.
pub const SCOPED_RECORDS_BOOTSTRAP_SCRIPT_ID: &str = "tessara-scoped-records-bootstrap";

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

/// Renders one authenticated Dashboard route with its authorization-filtered
/// request bootstrap available to both SSR content and client hydration.
pub fn application_html_with_dashboard_bootstrap(
    path: &str,
    title: &str,
    description: &str,
    bootstrap: &DashboardRouteBootstrap,
) -> String {
    document::render_native_app_document_with_dashboard_bootstrap(
        title,
        description,
        path,
        bootstrap,
    )
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

/// Renders Scoped Records content inside the Core-owned shell.
pub fn application_html_with_scoped_records_bootstrap(
    path: &str,
    title: &str,
    description: &str,
    bootstrap: &tessara_module_contract::ShellContentV1,
) -> String {
    document::render_native_app_document_with_scoped_records_bootstrap(
        title,
        description,
        path,
        bootstrap,
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
    use super::{
        Dashboard, DashboardPlacement, DashboardPlacementAvailability, DashboardRouteBootstrap,
        SessionAccount, application_html, application_html_with_dashboard_bootstrap,
        application_html_with_scoped_records_bootstrap,
    };
    use tessara_module_contract::ShellContentV1;

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
    fn dashboard_routes_keep_root_shell_and_feature_content_boundaries() {
        initialize_test_executor();
        for (path, heading) in [
            ("/dashboards", "Dashboards"),
            ("/dashboards/new", "Create Dashboard"),
            ("/dashboards/dashboard-42", "Dashboard Detail"),
            ("/dashboards/dashboard-42/edit", "Edit Dashboard"),
            ("/dashboards/dashboard-42/view", "Dashboard Viewer"),
        ] {
            let html = application_html(path, heading, "Dashboard route.");
            assert!(
                html.contains(r#"class="app-shell""#),
                "missing shell for {path}"
            );
            assert!(html.contains(heading), "missing heading for {path}");
            assert!(
                html.contains("dashboards-page"),
                "missing feature content for {path}"
            );
            assert!(!html.contains("Native route placeholder"));
        }
    }

    #[test]
    fn scoped_records_content_is_rendered_inside_the_core_shell() {
        initialize_test_executor();
        let html = application_html_with_scoped_records_bootstrap(
            "/reference/scoped-records",
            "Records Library · Tessara",
            "Scoped Records.",
            &ShellContentV1 {
                schema_version: 1,
                title: "Records Library".into(),
                body_html: "<h1>Records Library</h1><p>Module-owned content.</p>".into(),
            },
        );

        assert!(html.contains(r#"class="app-shell""#));
        assert!(html.contains(r#"class="top-app-bar__title">Records Library"#));
        assert!(html.contains(r#"class="sidebar-nav""#));
        assert!(html.contains(r#"class="top-app-bar__actions""#));
        assert!(html.contains(r#"class="route-panel scoped-records-page""#));
        assert!(html.contains("<p>Module-owned content.</p>"));
        assert!(!html.contains("data-shell-state="));
    }

    #[test]
    fn dashboard_document_provides_and_embeds_the_same_redacted_route_state() {
        initialize_test_executor();
        let bootstrap = DashboardRouteBootstrap::viewer(
            SessionAccount {
                capabilities: vec!["dashboards:read".into()],
            },
            Dashboard {
                id: "dashboard-42".into(),
                name: "Delivery health".into(),
                description: Some("Current delivery status".into()),
                visibility_nodes: Vec::new(),
                placement_count: 1,
                can_manage: false,
                placements: vec![DashboardPlacement {
                    placement_id: "placement-opaque".into(),
                    position: 0,
                    grid_row: 3,
                    grid_column: 7,
                    grid_width: 6,
                    grid_height: 2,
                    availability: DashboardPlacementAvailability::Unavailable,
                    config_state: None,
                    title: None,
                    component: None,
                    allowed_operations: None,
                }],
            },
        );
        let html = application_html_with_dashboard_bootstrap(
            "/dashboards/dashboard-42/view",
            "Dashboard Viewer",
            "View Dashboard.",
            &bootstrap,
        );

        assert!(html.contains(r#"id="tessara-dashboard-bootstrap""#));
        assert!(html.contains("Delivery health"));
        assert!(html.contains(r#""grid_column":7"#));
        assert!(html.contains("Unavailable placement"));
        assert!(!html.contains("component_version_id"));
        assert!(!html.contains("dataset_id"));
    }
}
