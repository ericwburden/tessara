#![recursion_limit = "512"]

//! Public boundary for the Datasets feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Datasets-specific implementation details in child modules.

mod actions;
mod api;
mod bootstrap;
mod components;
mod display;
mod document;
mod editor;
mod expressions;
mod http;
mod loaders;
mod pages;
mod pagination;
#[cfg(feature = "hydrate")]
mod payloads;
mod permissions;
mod text;
mod types;
mod validation;
pub use bootstrap::{DatasetEditorBootstrap, DatasetRouteBootstrap};
pub use document::{
    DATASET_BINDINGS_JS_SHA256, DATASET_BOOTSTRAP_SCRIPT_ID, DATASET_CSS, DATASET_CSS_SHA256,
    DATASET_JS, DATASET_JS_SHA256, DATASET_LIFECYCLE_CSS, DATASET_LIFECYCLE_CSS_SHA256,
    DATASET_WASM_SHA256, dataset_asset_path, render_dataset_document,
};
pub use editor::DatasetAggregationEditor;
pub use pages::{
    DatasetDetailContent, DatasetEditorContent, DatasetPreviewContent,
    DatasetRevisionDetailContent, DatasetRevisionEditorContent, DatasetRevisionHistoryContent,
    DatasetsIndexContent,
};
pub use types::{
    DatasetAggregationDraft, DatasetAggregationMetricDraft, DatasetFieldDraft,
    DatasetRowPickerDraft, DatasetRowPickerSortDraft,
};

pub(crate) fn dataset_content(bootstrap: &DatasetRouteBootstrap) -> leptos::prelude::AnyView {
    use leptos::prelude::*;
    match bootstrap {
        DatasetRouteBootstrap::Directory { .. } => view! { <DatasetsIndexContent/> }.into_any(),
        DatasetRouteBootstrap::Create { .. } => {
            view! { <DatasetEditorContent dataset_id=None/> }.into_any()
        }
        DatasetRouteBootstrap::Detail { dataset_id, .. } => {
            view! { <DatasetDetailContent dataset_id=dataset_id.clone()/> }.into_any()
        }
        DatasetRouteBootstrap::Preview { dataset_id, .. } => {
            view! { <DatasetPreviewContent dataset_id=dataset_id.clone()/> }.into_any()
        }
        DatasetRouteBootstrap::Edit { dataset_id, .. } => {
            view! { <DatasetEditorContent dataset_id=Some(dataset_id.clone())/> }.into_any()
        }
        DatasetRouteBootstrap::Revisions { dataset_id, .. } => {
            view! { <DatasetRevisionHistoryContent dataset_id=dataset_id.clone()/> }.into_any()
        }
        DatasetRouteBootstrap::RevisionDetail {
            dataset_id,
            revision_id,
            ..
        } => view! {
            <DatasetRevisionDetailContent
                dataset_id=dataset_id.clone()
                revision_id=revision_id.clone()
            />
        }
        .into_any(),
        DatasetRouteBootstrap::RevisionEdit {
            dataset_id,
            revision_id,
            ..
        } => view! {
            <DatasetRevisionEditorContent
                dataset_id=dataset_id.clone()
                revision_id=revision_id.clone()
            />
        }
        .into_any(),
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static LIFECYCLE: std::cell::RefCell<tessara_module_ui::LeptosLifecycleAdapter<DatasetRouteBootstrap>> =
        const { std::cell::RefCell::new(tessara_module_ui::LeptosLifecycleAdapter::new()) };
    static DIRECT_ROOT: std::cell::RefCell<tessara_module_ui::LeptosLifecycleRoot> =
        const { std::cell::RefCell::new(tessara_module_ui::LeptosLifecycleRoot::new()) };
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate_dataset() {
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
        .get_element_by_id("tessara-dataset-bootstrap")
        .and_then(|script| script.text_content())
        .and_then(|json| serde_json::from_str::<DatasetRouteBootstrap>(&json).ok())
    else {
        return;
    };
    let bootstrap_for_view = bootstrap.clone();
    DIRECT_ROOT.with(|direct| {
        direct.borrow_mut().hydrate(root.clone(), move || {
            view! {
                <Provider value=bootstrap_for_view.clone()>
                    {dataset_content(&bootstrap_for_view)}
                </Provider>
            }
        })
    });
    let _ = root.set_attribute("data-hydration", "ready");
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount_dataset(root_id: &str, bootstrap_json: &str) -> Result<(), wasm_bindgen::JsValue> {
    use leptos::{context::Provider, prelude::*};
    use wasm_bindgen::JsCast;
    unmount_dataset();
    let bootstrap = serde_json::from_str::<DatasetRouteBootstrap>(bootstrap_json)
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
                        {dataset_content(&current)}
                    </Provider>
                }
                .into_any()
            })
    });
    Ok(())
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn navigate_dataset(bootstrap_json: &str) -> Result<(), wasm_bindgen::JsValue> {
    let bootstrap = serde_json::from_str::<DatasetRouteBootstrap>(bootstrap_json)
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
pub fn can_deactivate_dataset() -> bool {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow().can_deactivate())
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn suspend_dataset() {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow().set_suspended(true));
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn resume_dataset() {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow().set_suspended(false));
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub(crate) fn set_lifecycle_dirty(dirty: bool) {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow_mut().set_dirty(dirty));
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
pub(crate) fn set_lifecycle_dirty(_: bool) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn unmount_dataset() {
    LIFECYCLE.with(|lifecycle| lifecycle.borrow_mut().unmount());
}
