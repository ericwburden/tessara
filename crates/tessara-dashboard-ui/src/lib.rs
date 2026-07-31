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
    DASHBOARD_CSS, DASHBOARD_CSS_SHA256, DASHBOARD_JS, DASHBOARD_JS_SHA256,
    DASHBOARD_LIFECYCLE_CSS, DASHBOARD_LIFECYCLE_CSS_SHA256, DASHBOARD_WASM, DASHBOARD_WASM_SHA256,
    dashboard_asset_path, render_dashboard_document,
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
thread_local! {
    static LIFECYCLE_ROOT: std::cell::RefCell<tessara_module_ui::LeptosLifecycleRoot> =
        const { std::cell::RefCell::new(tessara_module_ui::LeptosLifecycleRoot::new()) };
    static LIFECYCLE_BOOTSTRAP: std::cell::RefCell<Option<leptos::prelude::RwSignal<DashboardRouteBootstrap>>> =
        const { std::cell::RefCell::new(None) };
    static LIFECYCLE_ROOT_ID: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
    static LIFECYCLE_DIRTY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

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

/// Mounts Dashboard UI into a Core-owned outlet for lifecycle ABI v1.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount_dashboard(root_id: &str, bootstrap_json: &str) -> Result<(), wasm_bindgen::JsValue> {
    use leptos::{context::Provider, prelude::*};
    use wasm_bindgen::JsCast;

    unmount_dashboard();
    let bootstrap = serde_json::from_str::<DashboardRouteBootstrap>(bootstrap_json)
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?;
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("document is unavailable"))?;
    let root = document
        .get_element_by_id(root_id)
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("module outlet is unavailable"))?;
    root.set_inner_html("");
    let state = RwSignal::new(bootstrap);
    LIFECYCLE_ROOT.with(|lifecycle| {
        lifecycle.borrow_mut().mount(root.clone(), move || {
            view! {
                {move || {
                    let current = state.get();
                    let provided = current.clone();
                    view! {
                        <Provider value=provided>
                            {document::dashboard_content(&current)}
                        </Provider>
                    }
                }}
            }
        });
    });
    LIFECYCLE_BOOTSTRAP.with(|slot| *slot.borrow_mut() = Some(state));
    LIFECYCLE_ROOT_ID.with(|slot| *slot.borrow_mut() = Some(root_id.to_string()));
    LIFECYCLE_DIRTY.with(|dirty| dirty.set(false));
    let _ = root.set_attribute("data-module-lifecycle", "active");
    Ok(())
}

/// Applies a same-module route transition without replacing the runtime.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn navigate_dashboard(bootstrap_json: &str) -> Result<(), wasm_bindgen::JsValue> {
    use leptos::prelude::Set;

    let bootstrap = serde_json::from_str::<DashboardRouteBootstrap>(bootstrap_json)
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?;
    LIFECYCLE_BOOTSTRAP.with(|slot| {
        let signal = *slot.borrow();
        signal
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("Dashboard is not mounted"))?
            .set(bootstrap);
        Ok(())
    })
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn can_deactivate_dashboard() -> bool {
    LIFECYCLE_DIRTY.with(|dirty| !dirty.get())
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn suspend_dashboard() {
    set_lifecycle_visibility(true);
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn resume_dashboard() {
    set_lifecycle_visibility(false);
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn set_lifecycle_visibility(hidden: bool) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    LIFECYCLE_ROOT_ID.with(|slot| {
        if let Some(root) = slot
            .borrow()
            .as_deref()
            .and_then(|id| document.get_element_by_id(id))
        {
            let _ = root.set_attribute(
                "data-module-lifecycle",
                if hidden { "suspended" } else { "active" },
            );
            let _ = root.toggle_attribute_with_force("hidden", hidden);
            let _ = root.toggle_attribute_with_force("inert", hidden);
        }
    });
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn unmount_dashboard() {
    LIFECYCLE_ROOT.with(|lifecycle| lifecycle.borrow_mut().unmount());
    LIFECYCLE_BOOTSTRAP.with(|slot| *slot.borrow_mut() = None);
    LIFECYCLE_ROOT_ID.with(|slot| *slot.borrow_mut() = None);
    LIFECYCLE_DIRTY.with(|dirty| dirty.set(false));
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub(crate) fn set_lifecycle_dirty(dirty: bool) {
    LIFECYCLE_DIRTY.with(|state| state.set(dirty));
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
pub(crate) fn set_lifecycle_dirty(_: bool) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub(crate) fn navigate_dashboard_href(href: &str) {
    use wasm_bindgen::{JsCast, JsValue};

    let global = js_sys::global();
    let host = js_sys::Reflect::get(&global, &JsValue::from_str("__tessaraModuleHostV1")).ok();
    let navigate = host.as_ref().and_then(|host| {
        js_sys::Reflect::get(host, &JsValue::from_str("navigate"))
            .ok()?
            .dyn_into::<js_sys::Function>()
            .ok()
    });
    if let (Some(host), Some(navigate)) = (host, navigate) {
        let _ = navigate.call1(&host, &JsValue::from_str(href));
    } else if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(href);
    }
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
#[allow(dead_code)]
pub(crate) fn navigate_dashboard_href(_: &str) {}
