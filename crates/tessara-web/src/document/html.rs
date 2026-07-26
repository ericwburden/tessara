//! Native HTML document assembly.
//!
//! This module owns the server-rendered document wrapper around the Leptos app
//! root, overlay root, head tags, request bootstrap, and hydration script tags.

use leptos::context::Provider;
use leptos::prelude::*;
use leptos_router::location::RequestUrl;
use tessara_module_contract::ShellContentV1;
use tessara_web_dashboards::DashboardRouteBootstrap;

use crate::{
    app,
    features::modules::{
        MODULE_MANAGEMENT_BOOTSTRAP_SCRIPT_ID, ModuleManagementRouteBootstrapV1,
        escaped_module_management_bootstrap_json,
    },
    pipeline,
    state::shell_navigation::ShellNavigationResponseV1,
};

pub(crate) fn render_native_app_document(title: &str, description: &str, path: &str) -> String {
    render_native_app_document_with_optional_bootstraps(
        title,
        description,
        path,
        None,
        None,
        None,
        None,
    )
}

pub(crate) fn render_native_app_document_with_dashboard_bootstrap(
    title: &str,
    description: &str,
    path: &str,
    bootstrap: &DashboardRouteBootstrap,
) -> String {
    render_native_app_document_with_optional_bootstraps(
        title,
        description,
        path,
        Some(bootstrap),
        None,
        None,
        None,
    )
}

pub(crate) fn render_native_app_document_with_module_management_bootstrap(
    title: &str,
    description: &str,
    path: &str,
    bootstrap: &ModuleManagementRouteBootstrapV1,
) -> String {
    render_native_app_document_with_optional_bootstraps(
        title,
        description,
        path,
        None,
        Some(bootstrap),
        None,
        None,
    )
}

pub(crate) fn render_native_app_document_with_module_management_and_shell_navigation_bootstrap(
    title: &str,
    description: &str,
    path: &str,
    bootstrap: &ModuleManagementRouteBootstrapV1,
    shell_navigation: &ShellNavigationResponseV1,
) -> String {
    render_native_app_document_with_optional_bootstraps(
        title,
        description,
        path,
        None,
        Some(bootstrap),
        Some(shell_navigation),
        None,
    )
}

pub(crate) fn render_native_app_document_with_scoped_records_bootstrap(
    title: &str,
    description: &str,
    path: &str,
    bootstrap: &ShellContentV1,
) -> String {
    render_native_app_document_with_optional_bootstraps(
        title,
        description,
        path,
        None,
        None,
        None,
        Some(bootstrap),
    )
}

fn render_native_app_document_with_optional_bootstraps(
    title: &str,
    description: &str,
    path: &str,
    dashboard_bootstrap: Option<&DashboardRouteBootstrap>,
    module_management_bootstrap: Option<&ModuleManagementRouteBootstrapV1>,
    shell_navigation_bootstrap: Option<&ShellNavigationResponseV1>,
    scoped_records_bootstrap: Option<&ShellContentV1>,
) -> String {
    // Workspace-wide `--all-features` builds intentionally unify `ssr` and
    // `hydrate`. Browser-only components can therefore construct Effects while
    // this native renderer is exercised in tests; give those Effects a local
    // executor without changing the normal SSR-only production feature set.
    #[cfg(all(feature = "hydrate", not(target_arch = "wasm32")))]
    let _ = any_spawner::Executor::init_futures_executor();

    let shell_bootstrap = dashboard_bootstrap.cloned();
    let module_shell_bootstrap = module_management_bootstrap.cloned();
    let navigation_shell_bootstrap = shell_navigation_bootstrap.cloned();
    let scoped_shell_bootstrap = scoped_records_bootstrap.cloned();
    let shell_path = path.to_string();
    let shell = Owner::new().with(move || {
        if let Some(scoped_records) = scoped_shell_bootstrap {
            return view! {
                <Provider value=RequestUrl::new(&shell_path)>
                    <Provider value=scoped_records>
                        <app::App initial_shell_navigation=navigation_shell_bootstrap/>
                    </Provider>
                </Provider>
            }
            .to_html();
        }
        match (shell_bootstrap, module_shell_bootstrap) {
            (Some(dashboard), Some(module_management)) => view! {
                <Provider value=RequestUrl::new(&shell_path)>
                    <Provider value=dashboard>
                        <Provider value=module_management>
                            <app::App initial_shell_navigation=navigation_shell_bootstrap/>
                        </Provider>
                    </Provider>
                </Provider>
            }
            .to_html(),
            (Some(dashboard), None) => view! {
                <Provider value=RequestUrl::new(&shell_path)>
                    <Provider value=dashboard>
                        <app::App initial_shell_navigation=navigation_shell_bootstrap/>
                    </Provider>
                </Provider>
            }
            .to_html(),
            (None, Some(module_management)) => view! {
                <Provider value=RequestUrl::new(&shell_path)>
                    <Provider value=module_management>
                        <app::App initial_shell_navigation=navigation_shell_bootstrap/>
                    </Provider>
                </Provider>
            }
            .to_html(),
            (None, None) => view! {
                <Provider value=RequestUrl::new(&shell_path)>
                    <app::App initial_shell_navigation=navigation_shell_bootstrap/>
                </Provider>
            }
            .to_html(),
        }
    });
    let brand = crate::document::document_head_tags(title, description);
    let theme_bootstrap = crate::document::bootstrap_script();
    let stylesheets = crate::document::stylesheet_links();
    let mut route_bootstrap = dashboard_bootstrap.map_or_else(String::new, |bootstrap| {
        let json = escaped_dashboard_bootstrap_json(bootstrap);
        format!(
            "    <script id=\"{}\" type=\"application/json\">{json}</script>\n",
            crate::DASHBOARD_BOOTSTRAP_SCRIPT_ID
        )
    });
    if let Some(bootstrap) = module_management_bootstrap {
        let json = escaped_module_management_bootstrap_json(bootstrap);
        route_bootstrap.push_str(&format!(
            "    <script id=\"{MODULE_MANAGEMENT_BOOTSTRAP_SCRIPT_ID}\" type=\"application/json\">{json}</script>\n"
        ));
    }
    if let Some(bootstrap) = shell_navigation_bootstrap {
        let json = escaped_shell_navigation_bootstrap_json(bootstrap);
        route_bootstrap.push_str(&format!(
            "    <script id=\"{}\" type=\"application/json\">{json}</script>\n",
            crate::SHELL_NAVIGATION_BOOTSTRAP_SCRIPT_ID
        ));
    }
    if let Some(bootstrap) = scoped_records_bootstrap {
        let json = escaped_scoped_records_bootstrap_json(bootstrap);
        route_bootstrap.push_str(&format!(
            "    <script id=\"{}\" type=\"application/json\">{json}</script>\n",
            crate::SCOPED_RECORDS_BOOTSTRAP_SCRIPT_ID
        ));
    }
    let hydration = pipeline::hydration_module_tag();

    format!(
        "<!doctype html>\n\
<html lang=\"en\" data-theme=\"light\" data-theme-preference=\"system\">\n\
  <head>\n\
    <meta charset=\"utf-8\">\n\
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
    <title>{title}</title>\n\
    {brand}\n\
    <script>{theme_bootstrap}</script>\n\
    {stylesheets}\n\
  </head>\n\
  <body class=\"tessara-app\">\n\
    <div id=\"app-overlays\"></div>\n\
    <div id=\"{app_root_id}\">{shell}</div>\n\
{route_bootstrap}\
    {hydration}\n\
  </body>\n\
</html>",
        app_root_id = pipeline::APP_ROOT_ID,
    )
}

fn escaped_scoped_records_bootstrap_json(bootstrap: &ShellContentV1) -> String {
    serde_json::to_string(bootstrap)
        .expect("Scoped Records bootstrap should serialize")
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

fn escaped_shell_navigation_bootstrap_json(bootstrap: &ShellNavigationResponseV1) -> String {
    serde_json::to_string(bootstrap)
        .expect("Shell navigation bootstrap should serialize")
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

fn escaped_dashboard_bootstrap_json(bootstrap: &DashboardRouteBootstrap) -> String {
    serde_json::to_string(bootstrap)
        .expect("Dashboard route bootstrap should serialize")
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[cfg(test)]
mod tests {
    use crate::state::shell_navigation::{
        ShellNavigationGroupV1, ShellNavigationItemOwnerV1, ShellNavigationItemV1,
        ShellNavigationResponseV1, ShellNavigationStateV1, ShellNavigationUnavailableV1,
    };
    use tessara_web_dashboards::{DashboardRouteBootstrap, DashboardSummary, SessionAccount};

    use super::{escaped_dashboard_bootstrap_json, escaped_shell_navigation_bootstrap_json};

    #[test]
    fn bootstrap_json_cannot_terminate_its_script_tag() {
        let bootstrap = DashboardRouteBootstrap::directory(
            SessionAccount {
                capabilities: vec!["dashboards:read".into()],
            },
            vec![DashboardSummary {
                id: "dashboard-1".into(),
                name: "</script><script>alert('x')</script>".into(),
                description: Some("A&B".into()),
                visibility_nodes: Vec::new(),
                placement_count: 0,
                can_manage: false,
            }],
        );

        let json = escaped_dashboard_bootstrap_json(&bootstrap);
        assert!(!json.contains('<'));
        assert!(!json.contains('&'));
        assert!(json.contains("\\u003c/script\\u003e"));
        assert_eq!(
            serde_json::from_str::<DashboardRouteBootstrap>(&json).expect("parse escaped JSON"),
            bootstrap
        );
    }

    #[test]
    fn shell_navigation_json_cannot_terminate_its_script_tag() {
        let bootstrap = ShellNavigationResponseV1 {
            schema_version: 2,
            policy_revision: None,
            state: ShellNavigationStateV1::Unavailable,
            groups: vec![ShellNavigationGroupV1 {
                id: "core.main".into(),
                name: "Main".into(),
                items: vec![ShellNavigationItemV1 {
                    key: "home".into(),
                    label: "Home".into(),
                    href: "/".into(),
                    owner: ShellNavigationItemOwnerV1::Core,
                    contribution_id: None,
                }],
            }],
            unavailable: Some(ShellNavigationUnavailableV1 {
                code: "shell_navigation_unavailable".into(),
                message: "</script><script>alert('x')</script>&".into(),
            }),
        };

        let json = escaped_shell_navigation_bootstrap_json(&bootstrap);
        assert!(!json.contains('<'));
        assert!(!json.contains('&'));
        assert!(json.contains("\\u003c/script\\u003e"));
        assert_eq!(
            serde_json::from_str::<ShellNavigationResponseV1>(&json)
                .expect("parse escaped shell JSON"),
            bootstrap
        );
    }
}
