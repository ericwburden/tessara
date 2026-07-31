#![recursion_limit = "512"]

//! Dashboard-owned documents, assets, hydration bootstrap, and product UI.

mod api;
mod bootstrap;
mod document;
mod http;
mod pages;
mod types;

#[cfg(feature = "hydrate")]
pub use bootstrap::clear_dashboard_route_bootstrap;
pub use bootstrap::{DashboardRouteBootstrap, dashboard_route_bootstrap};
pub use document::{
    DASHBOARD_BINDINGS_JS, DASHBOARD_BINDINGS_JS_SHA256, DASHBOARD_BOOTSTRAP_SCRIPT_ID,
    DASHBOARD_CSS, DASHBOARD_CSS_SHA256, DASHBOARD_JS, DASHBOARD_JS_SHA256, DASHBOARD_WASM,
    DASHBOARD_WASM_SHA256, dashboard_asset_path, render_dashboard_document,
};
pub use pages::{
    DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS, DashboardCreateContent, DashboardDetailContent,
    DashboardEditorContent, DashboardViewerContent, DashboardsIndexContent,
};
pub use types::{
    Dashboard, DashboardComponentVersion, DashboardComponentVersionOption, DashboardComposition,
    DashboardPlacement, DashboardPlacementAvailability, DashboardPlacementConfigState,
    DashboardPlacementIdMapping, DashboardPlacementOperation, DashboardPlacementResolutionState,
    DashboardSummary, DashboardVisibilityNode, SessionAccount, VisibilityNodeOption,
};

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate_dashboard() {
    use leptos::{context::Provider, mount::hydrate_from, prelude::*};
    use wasm_bindgen::JsCast;

    let _ = any_spawner::Executor::init_wasm_bindgen();
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(root) = document
        .get_element_by_id("module-content")
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };
    let Some(bootstrap) = document
        .get_element_by_id(DASHBOARD_BOOTSTRAP_SCRIPT_ID)
        .and_then(|script| script.text_content())
        .and_then(|json| serde_json::from_str::<DashboardRouteBootstrap>(&json).ok())
    else {
        return;
    };
    let bootstrap_for_view = bootstrap.clone();
    let handle = hydrate_from(root.clone(), move || {
        view! {
            <Provider value=bootstrap_for_view.clone()>
                {document::dashboard_content(&bootstrap_for_view)}
            </Provider>
        }
    });
    handle.forget();
    let _ = root.set_attribute("data-hydration", "ready");
}
