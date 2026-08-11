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
    "2bc73b45249710846f70f6dd6856f224f1b305d9a4190712c0212a28a61afb81";
pub const COMPONENT_LIFECYCLE_CSS_SHA256: &str =
    "4c3394597a44d378a1452cbf30469ba1f48dca0e1c0d659d6504c392c2f2619c";
pub const COMPONENT_JS_SHA256: &str =
    "79ce8b02dceb6c27ef6d75a37a09c51e505dd7b6120601212887c84f6d6cd312";
pub const COMPONENT_BINDINGS_JS_SHA256: &str =
    "8ccf28f0eecb2c5861661de11be8efe1909bee1c393d91c9cad785b6c09725a6";
pub const COMPONENT_WASM_SHA256: &str =
    "745fd0b24f4a4b8f1cfeae6b0d5b5c8f292867c6f5f78c2942508188beed4ffb";

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
        assert!(!html.contains("data-component-search"));
    }
}
