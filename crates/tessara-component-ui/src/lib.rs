#![recursion_limit = "512"]

//! Public boundary for the Components feature.

mod api;
mod bootstrap;
mod document;
mod http;
mod pages;
mod types;

pub use bootstrap::{
    ComponentDefinitionBootstrap, ComponentDirectoryItem, ComponentRouteBootstrap,
    ComponentVersionBootstrap,
};
pub use document::{
    COMPONENT_BINDINGS_JS, COMPONENT_BINDINGS_JS_SHA256, COMPONENT_BOOTSTRAP_SCRIPT_ID,
    COMPONENT_CSS, COMPONENT_CSS_SHA256, COMPONENT_JS, COMPONENT_JS_SHA256,
    COMPONENT_LIFECYCLE_CSS, COMPONENT_LIFECYCLE_CSS_SHA256, COMPONENT_WASM, COMPONENT_WASM_SHA256,
    component_asset_path, render_component_document,
};
pub use pages::{
    ComponentEditorContent, ComponentVersionsContent, ComponentViewerContent,
    ComponentsIndexContent,
};
pub use tessara_component_viewer_ui::{
    ComponentRequestActivity, ComponentRequestActivityCallback, ComponentVersionExecutionContent,
    ComponentVersionKind, ComponentVersionTarget, ComponentViewerMode,
};

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static LIFECYCLE: std::cell::RefCell<tessara_module_ui::LeptosLifecycleAdapter<ComponentRouteBootstrap>> =
        const { std::cell::RefCell::new(tessara_module_ui::LeptosLifecycleAdapter::new()) };
    static DIRECT_ROOT: std::cell::RefCell<tessara_module_ui::LeptosLifecycleRoot> =
        const { std::cell::RefCell::new(tessara_module_ui::LeptosLifecycleRoot::new()) };
}

pub(crate) fn component_content(bootstrap: &ComponentRouteBootstrap) -> leptos::prelude::AnyView {
    use leptos::prelude::*;
    match bootstrap {
        ComponentRouteBootstrap::Directory { .. } => view! { <ComponentsIndexContent/> }.into_any(),
        ComponentRouteBootstrap::Create { .. } => {
            view! { <ComponentEditorContent component_ref=None/> }.into_any()
        }
        ComponentRouteBootstrap::Edit { component, .. } => {
            view! { <ComponentEditorContent component_ref=Some(component.slug.clone())/> }
                .into_any()
        }
        ComponentRouteBootstrap::Versions { component, .. } => {
            view! { <ComponentVersionsContent component_ref=component.slug.clone()/> }.into_any()
        }
        ComponentRouteBootstrap::Detail { component, .. }
        | ComponentRouteBootstrap::View { component, .. } => {
            view! { <ComponentViewerContent component_ref=component.slug.clone()/> }.into_any()
        }
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate_component() {
    use leptos::{context::Provider, prelude::*};
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
        .get_element_by_id(COMPONENT_BOOTSTRAP_SCRIPT_ID)
        .and_then(|script| script.text_content())
        .and_then(|json| serde_json::from_str::<ComponentRouteBootstrap>(&json).ok())
    else {
        return;
    };
    let bootstrap_for_view = bootstrap.clone();
    DIRECT_ROOT.with(|direct| {
        direct.borrow_mut().hydrate(root.clone(), move || {
            view! {
                <Provider value=bootstrap_for_view.clone()>
                    {component_content(&bootstrap_for_view)}
                </Provider>
            }
        })
    });
    let _ = root.set_attribute("data-hydration", "ready");
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount_component(root_id: &str, bootstrap_json: &str) -> Result<(), wasm_bindgen::JsValue> {
    use leptos::{context::Provider, prelude::*};
    use wasm_bindgen::JsCast;
    unmount_component();
    let bootstrap = serde_json::from_str::<ComponentRouteBootstrap>(bootstrap_json)
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
                let provided = current.clone();
                view! {
                    <Provider value=provided>
                        {component_content(&current)}
                    </Provider>
                }
                .into_any()
            })
    });
    Ok(())
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn navigate_component(bootstrap_json: &str) -> Result<(), wasm_bindgen::JsValue> {
    let bootstrap = serde_json::from_str::<ComponentRouteBootstrap>(bootstrap_json)
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
pub fn can_deactivate_component() -> bool {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow().can_deactivate())
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn suspend_component() {
    set_lifecycle_visibility(true);
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn resume_component() {
    set_lifecycle_visibility(false);
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn set_lifecycle_visibility(hidden: bool) {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow().set_suspended(hidden));
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub(crate) fn set_lifecycle_dirty(dirty: bool) {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow_mut().set_dirty(dirty));
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
pub(crate) fn set_lifecycle_dirty(_: bool) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn unmount_component() {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow_mut().unmount());
}
