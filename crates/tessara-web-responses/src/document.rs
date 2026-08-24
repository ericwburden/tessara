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
    "c2e598461f162e4dd73b091788f09890e234287fc0e54e71dcc2b38d7d10a87b";
pub const RESPONSE_JS_SHA256: &str =
    "eaecb5e8c2eb344d5b6e1011447fb50973246266c79247bb53af2bd1bd000d67";
pub const RESPONSE_BINDINGS_JS_SHA256: &str =
    "c966ec796b5ef1370fd101a9094d5392490a0eb698b1837e334479351c782090";
pub const RESPONSE_WASM_SHA256: &str =
    "1916393e4738588ad29e0f07f19ce8a052a8caf7089ef210bcaf27b5178f0f83";

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
