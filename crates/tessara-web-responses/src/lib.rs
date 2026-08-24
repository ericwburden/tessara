//! Public boundary for the Responses feature.
//!
//! Re-export only route content components; keep Responses-specific
//! implementation details in child modules.

mod actions;
mod api;
mod components;
mod detail;
pub(crate) mod display;
mod document;
mod edit;
mod filtering;
mod http;
mod list;
mod loaders;
mod metadata;
mod pagination;
mod start;
mod status;
mod text;
pub(crate) mod types;
mod url;
pub(crate) mod value_collection;

pub(crate) use display::workflow_revision_label_from_option;

pub use detail::ResponseDetailContent;
pub use document::{
    RESPONSE_BINDINGS_JS, RESPONSE_BINDINGS_JS_SHA256, RESPONSE_BOOTSTRAP_SCRIPT_ID, RESPONSE_CSS,
    RESPONSE_CSS_SHA256, RESPONSE_JS, RESPONSE_JS_SHA256, RESPONSE_WASM_SHA256,
    ResponseRouteBootstrap, render_response_document, response_asset_path,
};
pub use edit::ResponseEditContent;
pub use list::ResponsesIndexContent;
pub use start::ResponseStartContent;

pub(crate) fn response_content(bootstrap: &ResponseRouteBootstrap) -> leptos::prelude::AnyView {
    use leptos::prelude::*;
    let content = match bootstrap {
        ResponseRouteBootstrap::Directory => view! { <ResponsesIndexContent/> }.into_any(),
        ResponseRouteBootstrap::Start => view! { <ResponseStartContent/> }.into_any(),
        ResponseRouteBootstrap::Detail { response_id } => {
            view! { <ResponseDetailContent submission_id=response_id.clone()/> }.into_any()
        }
        ResponseRouteBootstrap::Edit { response_id } => {
            view! { <ResponseEditContent submission_id=response_id.clone()/> }.into_any()
        }
    };
    view! { <div class="tessara-response">{content}</div> }.into_any()
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static LIFECYCLE: std::cell::RefCell<tessara_module_ui::LeptosLifecycleAdapter<ResponseRouteBootstrap>> =
        const { std::cell::RefCell::new(tessara_module_ui::LeptosLifecycleAdapter::new()) };
    static DIRECT_ROOT: std::cell::RefCell<tessara_module_ui::LeptosLifecycleRoot> =
        const { std::cell::RefCell::new(tessara_module_ui::LeptosLifecycleRoot::new()) };
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate_response() {
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
        .get_element_by_id(RESPONSE_BOOTSTRAP_SCRIPT_ID)
        .and_then(|script| script.text_content())
        .and_then(|json| serde_json::from_str::<ResponseRouteBootstrap>(&json).ok())
    else {
        return;
    };
    let view_bootstrap = bootstrap.clone();
    DIRECT_ROOT.with(|direct| {
        direct
            .borrow_mut()
            .hydrate(root.clone(), move || response_content(&view_bootstrap))
    });
    let _ = root.set_attribute("data-hydration", "ready");
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount_response(root_id: &str, bootstrap_json: &str) -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    unmount_response();
    let bootstrap = serde_json::from_str::<ResponseRouteBootstrap>(bootstrap_json)
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?;
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("document is unavailable"))?;
    let root = document
        .get_element_by_id(root_id)
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("module outlet is unavailable"))?;
    LIFECYCLE.with(|lifecycle| {
        lifecycle
            .borrow_mut()
            .mount(root_id, root, bootstrap, |current| {
                response_content(&current)
            })
    });
    Ok(())
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn navigate_response(bootstrap_json: &str) -> Result<(), wasm_bindgen::JsValue> {
    let bootstrap = serde_json::from_str::<ResponseRouteBootstrap>(bootstrap_json)
        .map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))?;
    LIFECYCLE.with(|lifecycle| {
        lifecycle
            .borrow()
            .navigate(bootstrap)
            .map_err(wasm_bindgen::JsValue::from_str)
    })
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn can_deactivate_response() -> bool {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow().can_deactivate())
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn suspend_response() {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow().set_suspended(true));
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn resume_response() {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow().set_suspended(false));
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn unmount_response() {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow_mut().unmount());
}
