use leptos::{context::Provider, prelude::*};
use tessara_module_contract::ShellContextV1;
use tessara_module_ui::{ShellPresentation, escape_attribute, render_module_document};

use crate::{
    DashboardCreateContent, DashboardDetailContent, DashboardEditorContent,
    DashboardRouteBootstrap, DashboardViewerContent, DashboardsIndexContent,
};

pub const DASHBOARD_BOOTSTRAP_SCRIPT_ID: &str = "tessara-dashboard-bootstrap";
pub const DASHBOARD_CSS: &str = concat!(
    include_str!("../../tessara-module-ui/assets/module-shell.css"),
    "\n",
    include_str!("../assets/dashboard.css")
);
pub const DASHBOARD_JS: &str = include_str!("../assets/dashboard.js");
pub const DASHBOARD_BINDINGS_JS: &str = include_str!("../assets/dashboard-bindings.js");
pub const DASHBOARD_WASM: &[u8] = include_bytes!("../assets/dashboard.wasm");
pub const DASHBOARD_CSS_SHA256: &str =
    "a88186f05236add56720b1580c3e8ba872eb3ea35f9dbf37b64fb4e38ffc817d";
pub const DASHBOARD_JS_SHA256: &str =
    "8795628ec3677c50e09645bd83a88ef61684e2b299db818105632979ed40e828";
pub const DASHBOARD_BINDINGS_JS_SHA256: &str =
    "855e7c29bf8ed443779055c20148134aaa9ee10be91725cd89accf3ae93fde54";
pub const DASHBOARD_WASM_SHA256: &str =
    "12c1a5f9437a0ec2520156f0a90365d914665517f5703f825a65a14fcc812a07";

pub fn dashboard_asset_path(release: &str, digest: &str, name: &str) -> String {
    format!("/_tessara/modules/tessara.dashboards/{release}/sha256:{digest}/{name}")
}

pub fn render_dashboard_document(
    context: &ShellContextV1,
    path: &str,
    title: &str,
    bootstrap: &DashboardRouteBootstrap,
    release: &str,
) -> String {
    let bootstrap_for_view = bootstrap.clone();
    let content = Owner::new().with(move || {
        view! {
            <Provider value=bootstrap_for_view.clone()>
                {dashboard_content(&bootstrap_for_view)}
            </Provider>
        }
        .to_html()
    });
    let presentation = ShellPresentation::from_verified_context(context, path, title);
    let stylesheet = dashboard_asset_path(release, DASHBOARD_CSS_SHA256, "dashboard.css");
    let hydration = dashboard_asset_path(release, DASHBOARD_JS_SHA256, "dashboard.js");
    let mut document =
        render_module_document(&presentation, &stylesheet, Some(&hydration), &content);
    let bootstrap_json = escaped_bootstrap_json(bootstrap);
    let metadata = format!(
        r#"<meta name="tessara-module-definition" content="tessara.dashboards"><meta name="tessara-module-release" content="{}"><meta name="tessara-module-asset-digest" content="sha256:{}"><script id="{}" type="application/json">{}</script>"#,
        escape_attribute(release),
        DASHBOARD_JS_SHA256,
        DASHBOARD_BOOTSTRAP_SCRIPT_ID,
        bootstrap_json,
    );
    document = document.replacen("</head>", &format!("{metadata}</head>"), 1);
    document
}

pub(crate) fn dashboard_content(bootstrap: &DashboardRouteBootstrap) -> AnyView {
    match bootstrap {
        DashboardRouteBootstrap::Unavailable { retry_href, .. } => view! {
            <section class="route-panel dashboards-page dashboard-module-unavailable">
                <p class="eyebrow">"Dashboard module unavailable"</p>
                <h1>"Dashboards cannot be reached right now"</h1>
                <p>
                    "Core and the rest of Tessara remain available. Dashboard data and configuration are preserved."
                </p>
                <p><a class="button" href=retry_href.clone()>"Try Dashboards again"</a></p>
            </section>
        }
        .into_any(),
        DashboardRouteBootstrap::Directory { .. } => {
            view! { <DashboardsIndexContent/> }.into_any()
        }
        DashboardRouteBootstrap::Create { .. } => {
            view! { <DashboardCreateContent/> }.into_any()
        }
        DashboardRouteBootstrap::Detail { dashboard, .. } => {
            view! { <DashboardDetailContent dashboard_id=dashboard.id.clone()/> }.into_any()
        }
        DashboardRouteBootstrap::Editor { composition, .. } => view! {
            <DashboardEditorContent dashboard_id=composition.dashboard.id.clone()/>
        }
        .into_any(),
        DashboardRouteBootstrap::Viewer { dashboard, .. } => {
            view! { <DashboardViewerContent dashboard_id=dashboard.id.clone()/> }.into_any()
        }
    }
}

fn escaped_bootstrap_json(bootstrap: &DashboardRouteBootstrap) -> String {
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
    use chrono::{Duration, Utc};
    use tessara_module_contract::{
        ModuleDefinitionId, OriginalActorProjectionV1, ShellDocumentStateV1, ShellThemeV1,
    };
    use uuid::Uuid;

    use super::*;
    use crate::{DashboardSummary, SessionAccount};

    #[test]
    fn complete_document_is_dashboard_owned_and_release_observable() {
        let now = Utc::now();
        let context = ShellContextV1 {
            schema_version: 1,
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.dashboards").unwrap(),
            module_instance_id: Uuid::from_u128(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: Uuid::from_u128(3),
                display_name: "Operator".into(),
                email: None,
            },
            theme: ShellThemeV1::System,
            navigation: vec![],
            return_destination: "/".into(),
            locale: "en-US".into(),
            time_zone: "UTC".into(),
            correlation_id: Uuid::from_u128(4),
            document_state: ShellDocumentStateV1::Active,
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        };
        let html = render_dashboard_document(
            &context,
            "/dashboards",
            "Dashboards",
            &DashboardRouteBootstrap::directory(
                SessionAccount {
                    capabilities: vec!["dashboards:read".into()],
                },
                vec![DashboardSummary {
                    id: "dashboard-1".into(),
                    name: "Delivery".into(),
                    description: None,
                    visibility_nodes: vec![],
                    placement_count: 0,
                    can_manage: false,
                }],
            ),
            "2.0.0",
        );
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("Delivery"));
        assert!(html.contains(r#"name="tessara-module-release" content="2.0.0""#));
        assert!(html.contains(DASHBOARD_BOOTSTRAP_SCRIPT_ID));
        assert!(!html.contains("tessara-web"));
    }
}
