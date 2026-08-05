//! Minimal non-product module used to prove the canonical SDK boundary.

use serde::{Deserialize, Serialize};
use tessara_module_contract::ModuleManifest;
use tessara_module_ui::{ShellPresentation, escape_text, render_module_document};
use uuid::Uuid;

pub const DEFINITION_ID: &str = "tessara.reference.module-sdk";
pub const RELEASE_VERSION: &str = "1.0.0";
pub const READ_CAPABILITY: &str = "tessara.reference.module-sdk:read";
pub const ROOT_PATH: &str = "/reference/module-sdk";
pub const MODULE_SHELL_CSS_DIGEST: &str =
    "sha256:ca238aca616f242bfa144764a09ae4a76d0b6f075a288604cbb333d90859af46";
pub const MODULE_SHELL_CSS_PATH: &str = "/_tessara/modules/tessara.reference.module-sdk/1.0.0/sha256:ca238aca616f242bfa144764a09ae4a76d0b6f075a288604cbb333d90859af46/module-shell.css";
pub const MODULE_SHELL_JS_DIGEST: &str =
    "sha256:8265b868960d45fc50fa3fc8173968b94b6d36f1d9ce12e027ab6599942682ff";
pub const MODULE_SHELL_JS_PATH: &str = "/_tessara/modules/tessara.reference.module-sdk/1.0.0/sha256:8265b868960d45fc50fa3fc8173968b94b6d36f1d9ce12e027ab6599942682ff/module-shell.js";

pub fn manifest() -> ModuleManifest {
    serde_json::from_str(include_str!("../manifest.json"))
        .expect("reference module manifest is valid current JSON")
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceConfiguration {
    pub display_label: String,
}

impl ReferenceConfiguration {
    pub fn normalize(display_label: &str) -> Result<Self, &'static str> {
        let display_label = display_label.trim();
        if display_label.is_empty() || display_label.chars().count() > 80 {
            return Err("display_label must contain between 1 and 80 characters");
        }
        Ok(Self {
            display_label: display_label.to_owned(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSecurityState {
    pub installation_id: Uuid,
    pub module_instance_id: Uuid,
    pub authorization_revision: u64,
    pub organization_revision: u64,
    pub enabled: bool,
    pub document_state: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceState {
    pub schema_version: u16,
    pub configuration: ReferenceConfiguration,
    pub security: Option<ReferenceSecurityState>,
}

impl Default for ReferenceState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            configuration: ReferenceConfiguration {
                display_label: "Module SDK Reference".into(),
            },
            security: None,
        }
    }
}

pub fn render_reference_document(
    presentation: &ShellPresentation,
    configuration: &ReferenceConfiguration,
) -> String {
    render_module_document(
        presentation,
        MODULE_SHELL_CSS_PATH,
        Some(MODULE_SHELL_JS_PATH),
        &format!(
            "<section aria-labelledby=\"reference-title\"><h1 id=\"reference-title\">{}</h1><p>This non-product module proves independent manifest, runtime, UI, configuration, health, diagnostics, asset, outage, and shutdown behavior.</p><p><a href=\"/reference/module-sdk/diagnostics\">Open sanitized diagnostics</a></p></section>",
            escape_text(&configuration.display_label)
        ),
    )
}

#[cfg(feature = "ssr")]
pub mod native;

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn hydrate() {
    if let Some(document) = web_sys::window().and_then(|window| window.document()) {
        if let Some(root) = document.get_element_by_id("module-content") {
            let _ = root.set_attribute("data-hydrated", "true");
        }
    }
}

#[cfg(test)]
mod tests {
    use tessara_module_testkit::signed_shell_fixture;
    use tessara_module_ui::ShellPresentation;

    use super::*;

    #[test]
    fn configuration_is_trimmed_and_bounded() {
        assert_eq!(
            ReferenceConfiguration::normalize("  Fixture  ").unwrap(),
            ReferenceConfiguration {
                display_label: "Fixture".into()
            }
        );
        assert!(ReferenceConfiguration::normalize(" ").is_err());
    }

    #[test]
    fn document_is_non_product_and_no_javascript_useful() {
        let fixture = signed_shell_fixture(DEFINITION_ID);
        let presentation = ShellPresentation::from_verified_context(
            &fixture.envelope.payload,
            ROOT_PATH,
            "Reference",
        );
        let html =
            render_reference_document(&presentation, &ReferenceState::default().configuration);
        assert!(html.contains("non-product module"));
        assert!(html.contains("<div id=\"module-content\">"));
        assert!(html.contains(MODULE_SHELL_JS_PATH));
    }

    #[test]
    fn current_manifest_is_exact_and_valid() {
        let manifest = manifest();
        manifest
            .validate(
                &tessara_module_contract::ManifestNamespaceAuthority::new(
                    manifest.definition_id.clone(),
                    manifest.publisher.clone(),
                    ["tessara.reference.module-sdk"],
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(manifest.browser_routes.len(), 3);
        assert_eq!(
            MODULE_SHELL_CSS_DIGEST,
            format!("sha256:{}", tessara_module_ui::MODULE_SHELL_CSS_SHA256)
        );
        assert_eq!(
            MODULE_SHELL_JS_DIGEST,
            format!("sha256:{}", tessara_module_ui::MODULE_SHELL_JS_SHA256)
        );
    }
}
