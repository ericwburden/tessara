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
pub const DASHBOARD_LIFECYCLE_CSS: &str = include_str!("../assets/dashboard-lifecycle.css");
pub const DASHBOARD_JS: &str = include_str!("../assets/dashboard.js");
pub const DASHBOARD_BINDINGS_JS: &str = include_str!("../assets/dashboard-bindings.js");
pub const DASHBOARD_WASM: &[u8] = include_bytes!("../assets/dashboard.wasm");
pub const DASHBOARD_CSS_SHA256: &str =
    "38d4d592914df1654658c7e3072485ef4b11d8db0d1ab109f29ebe1518f8040b";
pub const DASHBOARD_LIFECYCLE_CSS_SHA256: &str =
    "ee0e3730df679d40e0987f003564063e00d97ae24d4bdf236535bfce691fbe99";
pub const DASHBOARD_JS_SHA256: &str =
    "4539231667bec9c9e4ca11280e76a8c5151474b405418706513178f89727c629";
pub const DASHBOARD_BINDINGS_JS_SHA256: &str =
    "14d1cba2346ea25118a4057a785b155957925ea658ef8d723dd9c0cf2cdcb7fa";
pub const DASHBOARD_WASM_SHA256: &str =
    "d53ff1e5dc08752d95403f911e768e899fba04709aeeb86814edf1d23142d70c";

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
    // Workspace-wide `--all-features` builds intentionally unify `ssr` and
    // `hydrate`. Browser-only components can therefore construct Effects while
    // this native renderer is exercised in tests; give those Effects a local
    // executor without changing the normal SSR-only production feature set.
    #[cfg(all(feature = "hydrate", not(target_arch = "wasm32")))]
    let _ = any_spawner::Executor::init_futures_executor();

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
    use sha2::{Digest, Sha256};
    use tessara_module_contract::{
        ModuleDefinitionId, OriginalActorProjectionV1, ShellDocumentStateV1, ShellThemeV1,
    };
    use uuid::Uuid;

    use super::*;
    use crate::{DashboardSummary, SessionAccount};

    #[test]
    fn complete_document_stylesheet_is_self_contained_responsive_and_digest_pinned() {
        assert!(!DASHBOARD_CSS.contains("@import"));
        assert!(DASHBOARD_CSS.contains(".dashboard-saved-grid"));
        assert!(DASHBOARD_CSS.contains("@media (max-width: 780px)"));
        assert!(DASHBOARD_CSS.contains(".dashboard-saved-grid > *"));
        assert!(DASHBOARD_CSS.contains(".dashboard-viewer-placement"));
        assert!(DASHBOARD_CSS.contains(".app-shell"));
        assert!(DASHBOARD_CSS.contains(".brand-lockup"));
        assert!(DASHBOARD_CSS.contains(".mobile-nav__panel"));
        assert!(DASHBOARD_CSS.contains(".dashboard-composition-tile__symbol svg"));
        assert!(DASHBOARD_CSS.contains("width: 1.9375rem"));

        let digest = format!("{:x}", Sha256::digest(DASHBOARD_CSS.as_bytes()));
        assert_eq!(digest, DASHBOARD_CSS_SHA256);
    }

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
            "2.1.0",
        );
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("Delivery"));
        assert!(html.contains(r#"name="tessara-module-release" content="2.1.0""#));
        assert!(html.contains(DASHBOARD_BOOTSTRAP_SCRIPT_ID));
        assert!(html.contains(r#"class="app-shell""#));
        assert!(html.contains(r#"class="brand-lockup""#));
        assert!(html.contains(r#"class="top-app-bar""#));
        assert!(html.contains(r#"placeholder="Search Tessara""#));
        assert!(!html.contains("PROTOTYPE CONTROL"));
        assert!(!html.contains("tessara-web"));
    }
}
