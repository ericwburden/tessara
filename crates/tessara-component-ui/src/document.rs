use tessara_module_contract::ShellContextV1;
use tessara_module_ui::{
    MODULE_UI_CSS_SHA256, ModuleBootstrapData, ModuleDocumentAssets, ModuleReleaseMetadata,
    ShellPresentation, render_module_view_document,
};

use leptos::{context::Provider, prelude::*};

use crate::{ComponentRouteBootstrap, component_content};

pub const COMPONENT_BOOTSTRAP_SCRIPT_ID: &str = "tessara-component-bootstrap";
pub const COMPONENT_CSS: &str = include_str!("../assets/component.css");
pub const COMPONENT_LIFECYCLE_CSS: &str = concat!(
    include_str!("../assets/component.css"),
    "\n",
    include_str!("../assets/component-lifecycle.css")
);
pub const COMPONENT_JS: &str = include_str!("../assets/component.js");
pub const COMPONENT_BINDINGS_JS: &str = include_str!("../assets/component-bindings.js");
pub const COMPONENT_WASM: &[u8] = include_bytes!("../assets/component.wasm");
pub const COMPONENT_CSS_SHA256: &str =
    "752387257cd0ddb69780425d88be40128a1a3cc96ed0cdffd725749ba723600e";
pub const COMPONENT_LIFECYCLE_CSS_SHA256: &str =
    "17eac83dcf01b4d39808474e7f58b5d4188d784f077d61f9d0a09021cb612724";
pub const COMPONENT_JS_SHA256: &str =
    "8c8c2f792140302511971ce776d640608730175d173cf9e0998987c99bd3c05d";
pub const COMPONENT_BINDINGS_JS_SHA256: &str =
    "8647b25ef0ac193922ceed4446a43ea18d8bb61e77298e10c05a4d370166b633";
pub const COMPONENT_WASM_SHA256: &str =
    "391b51c31892ca8a99743a08c427a5d01f54e495f46931820b04e9faeab76283";

pub fn component_asset_path(release: &str, digest: &str, name: &str) -> String {
    format!("/_tessara/modules/tessara.components/{release}/sha256:{digest}/{name}")
}

pub fn render_component_document(
    context: &ShellContextV1,
    path: &str,
    title: &str,
    bootstrap: &ComponentRouteBootstrap,
    release: &str,
) -> String {
    let presentation = ShellPresentation::from_verified_context(context, path, title);
    let bootstrap_for_view = bootstrap.clone();
    render_module_view_document(
        &presentation,
        &ModuleDocumentAssets {
            stylesheets: vec![
                component_asset_path(release, MODULE_UI_CSS_SHA256, "module-ui.css"),
                component_asset_path(release, COMPONENT_CSS_SHA256, "component.css"),
            ],
            deferred_scripts: vec![
                "/assets/d3.v7.9.0.min.js".into(),
                "/assets/tessara-d3-charts.js".into(),
            ],
            hydration_script: Some(component_asset_path(
                release,
                COMPONENT_JS_SHA256,
                "component.js",
            )),
        },
        &ModuleReleaseMetadata {
            definition_id: "tessara.components".into(),
            release_version: release.into(),
            asset_digest: format!("sha256:{COMPONENT_JS_SHA256}"),
        },
        Some(&ModuleBootstrapData {
            script_id: COMPONENT_BOOTSTRAP_SCRIPT_ID.into(),
            json: escaped_bootstrap_json(bootstrap),
        }),
        move || {
            view! {
                <Provider value=bootstrap_for_view.clone()>
                    {component_content(&bootstrap_for_view)}
                </Provider>
            }
        },
    )
}

fn escaped_bootstrap_json(bootstrap: &ComponentRouteBootstrap) -> String {
    serde_json::to_string(bootstrap)
        .expect("Component route bootstrap should serialize")
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
        ModuleDefinitionId, NavigationContributionId, NavigationProjectionV1,
        OriginalActorProjectionV1, ShellDocumentStateV1, ShellThemeV1,
    };
    use uuid::Uuid;

    use super::*;
    use crate::ComponentDirectoryItem;

    #[test]
    fn direct_document_renders_the_same_interactive_directory_view() {
        let now = Utc::now();
        let context = ShellContextV1 {
            schema_version: 1,
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.components").unwrap(),
            module_instance_id: Uuid::from_u128(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: Uuid::from_u128(3),
                display_name: "Admin".into(),
                email: None,
            },
            theme: ShellThemeV1::Dark,
            navigation: vec![NavigationProjectionV1 {
                contribution_id: NavigationContributionId::new("tessara.components.navigation")
                    .unwrap(),
                label: "Components".into(),
                href: "/components".into(),
            }],
            return_destination: "/".into(),
            locale: "en-US".into(),
            time_zone: "America/New_York".into(),
            correlation_id: Uuid::from_u128(4),
            document_state: ShellDocumentStateV1::Active,
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        };
        let bootstrap = ComponentRouteBootstrap::Directory {
            components: vec![ComponentDirectoryItem {
                component_id: "component-1".into(),
                name: "Delivery health".into(),
                slug: "delivery-health".into(),
                description: Some("Current delivery health".into()),
                component_type: "table".into(),
                publication_state: "published".into(),
                current_version_id: Some("version-1".into()),
                current_version_label: Some("1.0".into()),
                draft_version_id: None,
                draft_version_label: None,
                manageable: true,
            }],
            can_manage: true,
        };

        let html =
            render_component_document(&context, "/components", "Components", &bootstrap, "1.0.1");

        assert!(html.contains(r#"class="top-app-bar__title">Components</span>"#));
        assert!(html.contains("data-component-directory-item"));
        assert!(html.contains("searchable-data-table__search"));
        assert!(html.contains("Delivery health"));
        assert!(html.contains(r#"src="/assets/d3.v7.9.0.min.js" defer"#));
        assert!(html.contains(r#"src="/assets/tessara-d3-charts.js" defer"#));
        assert!(!html.contains("data-component-search"));
    }
}
