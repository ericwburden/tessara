use serde::{Deserialize, Serialize};
use tessara_module_contract::ShellContextV2;
use tessara_module_ui::{
    MODULE_UI_CSS_SHA256, ModuleBootstrapData, ModuleDocumentAssets, ModuleReleaseMetadata,
    ShellPresentation, render_module_view_document,
};

use crate::response_content;

pub const RESPONSE_BOOTSTRAP_SCRIPT_ID: &str = "tessara-response-bootstrap";
pub const RESPONSE_CSS: &str = include_str!("../assets/response.css");
pub const RESPONSE_JS: &str = include_str!("../assets/response.js");
pub const RESPONSE_BINDINGS_JS: &str = include_str!("../assets/response-bindings.js");
pub const RESPONSE_CSS_SHA256: &str =
    "e1580f8e8d7e3a58443734e37ff630014a57d79f6c125c220e7de9b852c712c7";
pub const RESPONSE_JS_SHA256: &str =
    "fee5dcb70cd6987e673b557c404c29fd8fb872b83601341e67e9a1207e974462";
pub const RESPONSE_BINDINGS_JS_SHA256: &str =
    "67df799cbf64eac17e122b7aa2e68f1f9bbc885e68a408d208b6dc3285bbaddf";
pub const RESPONSE_WASM_SHA256: &str =
    "245e9dc803bc24790f9e84af94d592a1ef4563b269b2b36529bcc8081a8c6273";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "route", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResponseRouteBootstrap {
    Directory,
    Start,
    Detail { response_id: String },
    Edit { response_id: String },
}

pub fn response_asset_path(release: &str, digest: &str, name: &str) -> String {
    format!("/_tessara/modules/tessara.responses/{release}/sha256:{digest}/{name}")
}

pub fn render_response_document(
    context: &ShellContextV2,
    path: &str,
    title: &str,
    bootstrap: &ResponseRouteBootstrap,
    release: &str,
) -> String {
    let presentation = ShellPresentation::from_verified_context(context, path, title);
    let view_bootstrap = bootstrap.clone();
    render_module_view_document(
        &presentation,
        &ModuleDocumentAssets {
            stylesheets: vec![
                response_asset_path(release, MODULE_UI_CSS_SHA256, "module-ui.css"),
                response_asset_path(release, RESPONSE_CSS_SHA256, "response.css"),
            ],
            deferred_scripts: Vec::new(),
            hydration_script: Some(response_asset_path(
                release,
                RESPONSE_JS_SHA256,
                "response.js",
            )),
        },
        &ModuleReleaseMetadata {
            definition_id: "tessara.responses".into(),
            release_version: release.into(),
            asset_digest: format!("sha256:{RESPONSE_JS_SHA256}"),
        },
        Some(&ModuleBootstrapData {
            script_id: RESPONSE_BOOTSTRAP_SCRIPT_ID.into(),
            json: escaped_bootstrap_json(bootstrap),
        }),
        move || response_content(&view_bootstrap),
    )
}

fn escaped_bootstrap_json(bootstrap: &ResponseRouteBootstrap) -> String {
    serde_json::to_string(bootstrap)
        .expect("Response route bootstrap should serialize")
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_entry_asset_supports_complete_documents_and_lifecycle_hosts() {
        for required in [
            "hydrate_response",
            "export async function createModule(host)",
            "mount_response",
            "navigate_response",
            "can_deactivate_response",
            "suspend_response",
            "resume_response",
            "unmount_response",
            "Discard unsaved Response changes?",
        ] {
            assert!(
                RESPONSE_JS.contains(required),
                "Response entry asset omitted {required}"
            );
        }
        assert!(!RESPONSE_JS.contains("https://"));
        assert!(!RESPONSE_JS.contains("http://"));
    }

    #[test]
    fn response_route_bootstrap_preserves_all_public_document_shapes() {
        for bootstrap in [
            ResponseRouteBootstrap::Directory,
            ResponseRouteBootstrap::Start,
            ResponseRouteBootstrap::Detail {
                response_id: "draft-response".into(),
            },
            ResponseRouteBootstrap::Edit {
                response_id: "draft-response".into(),
            },
        ] {
            let encoded = escaped_bootstrap_json(&bootstrap);
            let decoded = serde_json::from_str::<ResponseRouteBootstrap>(&encoded)
                .expect("Response route bootstrap should round trip");
            assert_eq!(decoded, bootstrap);
        }
    }
}
