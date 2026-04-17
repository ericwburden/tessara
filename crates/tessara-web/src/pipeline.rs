use std::path::PathBuf;

pub const OUTPUT_NAME: &str = "tessara-web";
pub const APP_ROOT_ID: &str = "app-root";
pub const APP_ROOT_START: &str = "<!--tessara-app-root-start-->";
pub const APP_ROOT_END: &str = "<!--tessara-app-root-end-->";
pub const HYDRATE_EXPORT: &str = "hydrate_app";

pub fn site_root() -> PathBuf {
    std::env::var("LEPTOS_SITE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/site"))
}

pub fn site_pkg_dir() -> String {
    std::env::var("LEPTOS_SITE_PKG_DIR").unwrap_or_else(|_| "pkg".into())
}

pub fn pkg_dir() -> PathBuf {
    site_root().join(site_pkg_dir())
}

pub fn pkg_asset_path(name: &str) -> String {
    format!("/{}/{}", site_pkg_dir(), name)
}

pub fn css_path() -> String {
    pkg_asset_path(&format!("{OUTPUT_NAME}.css"))
}

pub fn js_path() -> String {
    pkg_asset_path(&format!("{OUTPUT_NAME}.js"))
}

pub fn wasm_path() -> String {
    pkg_asset_path(&format!("{OUTPUT_NAME}.wasm"))
}

pub fn bridge_asset_path(name: &str) -> String {
    format!("/bridge/{name}")
}

pub fn hydration_module_tag() -> String {
    let js_path = js_path();
    let wasm_path = wasm_path();
    format!(
        r#"<script type="module">
import init from "{js_path}";
await init("{wasm_path}");
</script>"#
    )
}
