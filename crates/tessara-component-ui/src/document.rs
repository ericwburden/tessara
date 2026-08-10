use tessara_module_contract::ShellContextV1;
use tessara_module_ui::{
    MODULE_UI_CSS_SHA256, ModuleBootstrapData, ModuleDocumentAssets, ModuleReleaseMetadata,
    ShellPresentation, render_module_view_document,
};

use crate::{ComponentRouteBootstrap, component_bootstrap_view};

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
    "f223b447fe5ac606e68ff72de4b9759fc878dbd19fbcedf2db60f4252b984924";
pub const COMPONENT_LIFECYCLE_CSS_SHA256: &str =
    "a7e0a747a2615a786efb5bdc413fcf4ecd46dce0dc85aeff6a1d2ab1129ebf94";
pub const COMPONENT_JS_SHA256: &str =
    "716275d70eef41ce0e97d56ddd0fa31e54446096f01f32db439d94c23ebfbf18";
pub const COMPONENT_BINDINGS_JS_SHA256: &str =
    "89adb1a1d3fe348cc2a2f00741b0e078493dbdcb48e284754535b845d33a0b80";
pub const COMPONENT_WASM_SHA256: &str =
    "6a72a0fc1d697ff902393a7052901795d0c5883c9686bf5fe20cc298d21686f1";

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
        || component_bootstrap_view(bootstrap),
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
