//! Native HTML document assembly.
//!
//! This module owns the server-rendered document wrapper around the Leptos app
//! root, overlay root, head tags, request bootstrap, and hydration script tags.

use leptos::context::Provider;
use leptos::prelude::*;
use leptos_router::location::RequestUrl;
use tessara_web_dashboards::DashboardRouteBootstrap;

use crate::{app, pipeline};

pub(crate) fn render_native_app_document(title: &str, description: &str, path: &str) -> String {
    render_native_app_document_with_optional_dashboard_bootstrap(title, description, path, None)
}

pub(crate) fn render_native_app_document_with_dashboard_bootstrap(
    title: &str,
    description: &str,
    path: &str,
    bootstrap: &DashboardRouteBootstrap,
) -> String {
    render_native_app_document_with_optional_dashboard_bootstrap(
        title,
        description,
        path,
        Some(bootstrap),
    )
}

fn render_native_app_document_with_optional_dashboard_bootstrap(
    title: &str,
    description: &str,
    path: &str,
    dashboard_bootstrap: Option<&DashboardRouteBootstrap>,
) -> String {
    // Workspace-wide `--all-features` builds intentionally unify `ssr` and
    // `hydrate`. Browser-only components can therefore construct Effects while
    // this native renderer is exercised in tests; give those Effects a local
    // executor without changing the normal SSR-only production feature set.
    #[cfg(all(feature = "hydrate", not(target_arch = "wasm32")))]
    let _ = any_spawner::Executor::init_futures_executor();

    let shell_bootstrap = dashboard_bootstrap.cloned();
    let shell_path = path.to_string();
    let shell = Owner::new().with(move || {
        if let Some(bootstrap) = shell_bootstrap {
            view! {
                <Provider value=RequestUrl::new(&shell_path)>
                    <Provider value=bootstrap>
                        <app::App/>
                    </Provider>
                </Provider>
            }
            .to_html()
        } else {
            view! {
                <Provider value=RequestUrl::new(&shell_path)>
                    <app::App/>
                </Provider>
            }
            .to_html()
        }
    });
    let brand = crate::document::document_head_tags(title, description);
    let theme_bootstrap = crate::document::bootstrap_script();
    let stylesheets = crate::document::stylesheet_links();
    let route_bootstrap = dashboard_bootstrap.map_or_else(String::new, |bootstrap| {
        let json = escaped_dashboard_bootstrap_json(bootstrap);
        format!(
            "    <script id=\"{}\" type=\"application/json\">{json}</script>\n",
            crate::DASHBOARD_BOOTSTRAP_SCRIPT_ID
        )
    });
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
    use tessara_web_dashboards::{DashboardRouteBootstrap, DashboardSummary, SessionAccount};

    use super::escaped_dashboard_bootstrap_json;

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
}
