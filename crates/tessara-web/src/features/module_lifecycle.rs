//! Framework-neutral browser lifecycle host for independently deployed modules.

use leptos::prelude::*;

use crate::ui::AppShell;

pub const MODULE_OUTLET_ID: &str = "tessara-module-outlet";

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub fn deactivate_current() {
    leptos::task::spawn_local(browser::deactivate());
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    not(all(feature = "hydrate", target_arch = "wasm32")),
    allow(dead_code)
)]
enum HostState {
    Loading,
    Active,
    Failed(String),
}

#[component]
pub fn ModuleLifecyclePage() -> impl IntoView {
    let state = RwSignal::new(HostState::Loading);
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    let activation_revision = RwSignal::new(0_u64);
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let location = leptos_router::hooks::use_location();
        Effect::new(move |_| {
            let path = location.pathname.get();
            activation_revision.update(|revision| *revision = revision.wrapping_add(1));
            let requested_revision = activation_revision.get_untracked();
            state.set(HostState::Loading);
            leptos::task::spawn_local(async move {
                let result = browser::activate(&path).await;
                if activation_revision.get_untracked() != requested_revision {
                    return;
                }
                match result {
                    Ok(()) => state.set(HostState::Active),
                    Err(error) => state.set(HostState::Failed(error)),
                }
            });
        });
    }

    view! {
        <AppShell active_route="dashboards" title="Dashboards">
            <section class="module-lifecycle-host">
                {move || match state.get() {
                    HostState::Loading => view! {
                        <section class="route-panel module-lifecycle-state" role="status">
                            <p class="eyebrow">"Dashboards"</p>
                            <h1>"Loading module"</h1>
                            <p>"Preparing the active Dashboard release."</p>
                        </section>
                    }.into_any(),
                    HostState::Active => ().into_any(),
                    HostState::Failed(message) => view! {
                        <section class="route-panel module-lifecycle-state" role="alert">
                            <p class="eyebrow">"Module unavailable"</p>
                            <h1>"Dashboards could not be opened"</h1>
                            <p>{message}</p>
                            <p><a class="button" rel="external" href=move || browser::document_fallback_href()>"Reload Dashboards"</a></p>
                        </section>
                    }.into_any(),
                }}
                <section
                    id=MODULE_OUTLET_ID
                    class="module-lifecycle-outlet"
                    data-module-definition="tessara.dashboards"
                    aria-busy=move || (state.get() == HostState::Loading).to_string()
                ></section>
            </section>
        </AppShell>
    }
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
mod browser {
    pub(super) fn document_fallback_href() -> String {
        "/dashboards".into()
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
mod browser {
    use std::cell::{Cell, RefCell};

    use gloo_net::http::Request;
    use js_sys::{Function, JSON, Object, Promise, Reflect};
    use tessara_module_contract::BrowserLifecycleBootstrapV1;
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use wasm_bindgen_futures::JsFuture;

    use super::MODULE_OUTLET_ID;

    struct ActiveModule {
        definition_id: String,
        release_version: String,
        instance: JsValue,
        stylesheet_ids: Vec<String>,
        _navigate: Closure<dyn FnMut(String)>,
        _click_guard: Closure<dyn FnMut(web_sys::MouseEvent)>,
    }

    thread_local! {
        static ACTIVE: RefCell<Option<ActiveModule>> = const { RefCell::new(None) };
        static GENERATION: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn document_fallback_href() -> String {
        web_sys::window()
            .and_then(|window| window.location().pathname().ok())
            .unwrap_or_else(|| "/dashboards".into())
    }

    pub(super) async fn activate(path: &str) -> Result<(), String> {
        let generation = next_generation();
        let response = Request::get(path)
            .header(
                "Accept",
                "application/vnd.tessara.module-view+json; version=1",
            )
            .send()
            .await
            .map_err(|error| format!("Module bootstrap request failed: {error}"))?;
        if !response.ok() {
            return Err(format!(
                "Module bootstrap returned HTTP {}. The complete-document fallback remains available.",
                response.status()
            ));
        }
        let bootstrap = response
            .json::<BrowserLifecycleBootstrapV1>()
            .await
            .map_err(|error| format!("Module bootstrap was invalid: {error}"))?;
        ensure_current(generation)?;
        if !bootstrap.is_supported() || bootstrap.path != path {
            return Err("Module bootstrap is incompatible with lifecycle ABI v1.".into());
        }

        let existing = ACTIVE.with(|slot| {
            slot.borrow().as_ref().and_then(|active| {
                (active.definition_id == bootstrap.definition_id.as_str()
                    && active.release_version == bootstrap.release_version.to_string())
                .then(|| active.instance.clone())
            })
        });
        if let Some(instance) = existing {
            if let Err(error) =
                invoke(&instance, "navigate", Some(&module_input(&bootstrap)?)).await
            {
                deactivate().await;
                return Err(error);
            }
            ensure_current(generation)?;
            update_document(&bootstrap);
            return Ok(());
        }

        deactivate().await;
        // `deactivate` invalidates outstanding work; this activation now owns
        // the next generation before loading its release assets.
        let generation = next_generation();
        let stylesheet_ids = install_stylesheets(&bootstrap)?;
        let imported = match dynamic_import(&bootstrap.entry_asset.url).await {
            Ok(imported) => imported,
            Err(error) => {
                remove_stylesheets(&stylesheet_ids);
                return Err(error);
            }
        };
        if let Err(error) = ensure_current(generation) {
            remove_stylesheets(&stylesheet_ids);
            return Err(error);
        }
        let create = match Reflect::get(&imported, &JsValue::from_str("createModule"))
            .map_err(js_error)
            .and_then(|value| {
                value
                    .dyn_into::<Function>()
                    .map_err(|_| "Module entry does not export createModule(host).".to_string())
            }) {
            Ok(create) => create,
            Err(error) => {
                remove_stylesheets(&stylesheet_ids);
                return Err(error);
            }
        };
        let (host, navigate) = match host_api() {
            Ok(host) => host,
            Err(error) => {
                remove_stylesheets(&stylesheet_ids);
                return Err(error);
            }
        };
        let instance = match create.call1(&JsValue::UNDEFINED, &host).map_err(js_error) {
            Ok(value) => match await_value(value).await {
                Ok(instance) => instance,
                Err(error) => {
                    remove_stylesheets(&stylesheet_ids);
                    return Err(error);
                }
            },
            Err(error) => {
                remove_stylesheets(&stylesheet_ids);
                return Err(error);
            }
        };
        if let Err(error) = ensure_current(generation) {
            let _ = invoke(&instance, "dispose", None).await;
            remove_stylesheets(&stylesheet_ids);
            return Err(error);
        }
        let click_guard = match install_navigation_guard(&instance) {
            Ok(guard) => guard,
            Err(error) => {
                let _ = invoke(&instance, "dispose", None).await;
                remove_stylesheets(&stylesheet_ids);
                return Err(error);
            }
        };
        if let Err(error) = invoke(&instance, "mount", Some(&module_input(&bootstrap)?)).await {
            remove_navigation_guard(&click_guard);
            let _ = invoke(&instance, "dispose", None).await;
            remove_stylesheets(&stylesheet_ids);
            return Err(error);
        }
        ACTIVE.with(|slot| {
            *slot.borrow_mut() = Some(ActiveModule {
                definition_id: bootstrap.definition_id.as_str().to_string(),
                release_version: bootstrap.release_version.to_string(),
                instance,
                stylesheet_ids,
                _navigate: navigate,
                _click_guard: click_guard,
            });
        });
        update_document(&bootstrap);
        Ok(())
    }

    pub(crate) async fn deactivate() {
        next_generation();
        let active = ACTIVE.with(|slot| slot.borrow_mut().take());
        let Some(active) = active else { return };
        let _ = invoke(&active.instance, "unmount", None).await;
        let _ = invoke(&active.instance, "dispose", None).await;
        remove_navigation_guard(&active._click_guard);
        remove_stylesheets(&active.stylesheet_ids);
    }

    fn next_generation() -> u64 {
        GENERATION.with(|generation| {
            let next = generation.get().wrapping_add(1);
            generation.set(next);
            next
        })
    }

    fn ensure_current(expected: u64) -> Result<(), String> {
        GENERATION.with(|generation| {
            (generation.get() == expected)
                .then_some(())
                .ok_or_else(|| "Module activation was superseded.".into())
        })
    }

    fn remove_navigation_guard(guard: &Closure<dyn FnMut(web_sys::MouseEvent)>) {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            let _ = document.remove_event_listener_with_callback_and_bool(
                "click",
                guard.as_ref().unchecked_ref(),
                true,
            );
        }
    }

    fn remove_stylesheets(ids: &[String]) {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            for id in ids {
                if let Some(element) = document.get_element_by_id(id) {
                    element.remove();
                }
            }
        }
    }

    fn module_input(bootstrap: &BrowserLifecycleBootstrapV1) -> Result<JsValue, String> {
        let input = Object::new();
        Reflect::set(
            &input,
            &JsValue::from_str("outletId"),
            &JsValue::from_str(MODULE_OUTLET_ID),
        )
        .map_err(js_error)?;
        let json = serde_json::to_string(bootstrap).map_err(|error| error.to_string())?;
        let value = JSON::parse(&json).map_err(js_error)?;
        Reflect::set(&input, &JsValue::from_str("bootstrap"), &value).map_err(js_error)?;
        Ok(input.into())
    }

    fn host_api() -> Result<(JsValue, Closure<dyn FnMut(String)>), String> {
        let host = Object::new();
        Reflect::set(
            &host,
            &JsValue::from_str("lifecycleAbi"),
            &JsValue::from_str("1.0.0"),
        )
        .map_err(js_error)?;
        let navigate = Closure::wrap(Box::new(move |href: String| {
            leptos::task::spawn_local(navigate_after_guard(href));
        }) as Box<dyn FnMut(String)>);
        Reflect::set(
            &host,
            &JsValue::from_str("navigate"),
            navigate.as_ref().unchecked_ref(),
        )
        .map_err(js_error)?;
        Ok((host.into(), navigate))
    }

    fn install_navigation_guard(
        instance: &JsValue,
    ) -> Result<Closure<dyn FnMut(web_sys::MouseEvent)>, String> {
        let instance = instance.clone();
        let guard = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if event.default_prevented()
                || event.button() != 0
                || event.ctrl_key()
                || event.meta_key()
                || event.shift_key()
                || event.alt_key()
            {
                return;
            }
            let Some(anchor) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .and_then(|target| target.closest("a[href]").ok().flatten())
                .and_then(|anchor| anchor.dyn_into::<web_sys::HtmlAnchorElement>().ok())
            else {
                return;
            };
            if anchor.has_attribute("download") || !matches!(anchor.target().as_str(), "" | "_self")
            {
                return;
            }
            let Some(window) = web_sys::window() else {
                return;
            };
            if window.location().origin().ok().as_deref() != Some(anchor.origin().as_str()) {
                return;
            }
            let href = format!("{}{}{}", anchor.pathname(), anchor.search(), anchor.hash());
            if window.location().pathname().ok().as_deref() == Some(anchor.pathname().as_str())
                && window.location().search().ok().as_deref() == Some(anchor.search().as_str())
                && window.location().hash().ok().as_deref() == Some(anchor.hash().as_str())
            {
                return;
            }
            event.prevent_default();
            event.stop_immediate_propagation();
            let instance = instance.clone();
            leptos::task::spawn_local(async move {
                if deactivation_allowed(&instance).await {
                    navigate_soft(&href);
                }
            });
        }) as Box<dyn FnMut(_)>);
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| "document is unavailable".to_string())?;
        document
            .add_event_listener_with_callback_and_bool(
                "click",
                guard.as_ref().unchecked_ref(),
                true,
            )
            .map_err(js_error)?;
        Ok(guard)
    }

    async fn navigate_after_guard(href: String) {
        let instance =
            ACTIVE.with(|slot| slot.borrow().as_ref().map(|active| active.instance.clone()));
        match instance {
            None => navigate_soft(&href),
            Some(instance) if deactivation_allowed(&instance).await => navigate_soft(&href),
            Some(_) => {}
        }
    }

    async fn deactivation_allowed(instance: &JsValue) -> bool {
        let Ok(decision) = invoke(instance, "canDeactivate", None).await else {
            return false;
        };
        if Reflect::get(&decision, &JsValue::from_str("allowed"))
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            return true;
        }
        let prompt = Reflect::get(&decision, &JsValue::from_str("prompt"))
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_else(|| "Discard unsaved module changes?".into());
        web_sys::window()
            .and_then(|window| window.confirm_with_message(&prompt).ok())
            .unwrap_or(false)
    }

    fn navigate_soft(href: &str) {
        let Some(window) = web_sys::window() else {
            return;
        };
        if window
            .history()
            .and_then(|history| history.push_state_with_url(&JsValue::NULL, "", Some(href)))
            .is_ok()
        {
            if let Ok(event) = web_sys::Event::new("popstate") {
                let _ = window.dispatch_event(&event);
            }
        }
    }

    fn install_stylesheets(bootstrap: &BrowserLifecycleBootstrapV1) -> Result<Vec<String>, String> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| "document is unavailable".to_string())?;
        let head = document
            .head()
            .ok_or_else(|| "document head is unavailable".to_string())?;
        let mut ids = Vec::new();
        for asset in &bootstrap.stylesheet_assets {
            let id = format!(
                "tessara-module-style-{}",
                asset.digest.as_str().replace(':', "-")
            );
            if document.get_element_by_id(&id).is_none() {
                let link = document.create_element("link").map_err(js_error)?;
                link.set_attribute("id", &id).map_err(js_error)?;
                link.set_attribute("rel", "stylesheet").map_err(js_error)?;
                link.set_attribute("href", &asset.url).map_err(js_error)?;
                link.set_attribute(
                    "data-tessara-module-style",
                    bootstrap.definition_id.as_str(),
                )
                .map_err(js_error)?;
                head.append_child(&link).map_err(js_error)?;
            }
            ids.push(id);
        }
        Ok(ids)
    }

    async fn dynamic_import(url: &str) -> Result<JsValue, String> {
        let import = Function::new_with_args("url", "return import(url)");
        await_value(
            import
                .call1(&JsValue::UNDEFINED, &JsValue::from_str(url))
                .map_err(js_error)?,
        )
        .await
    }

    async fn invoke(
        instance: &JsValue,
        method: &str,
        input: Option<&JsValue>,
    ) -> Result<JsValue, String> {
        let function = Reflect::get(instance, &JsValue::from_str(method))
            .map_err(js_error)?
            .dyn_into::<Function>()
            .map_err(|_| format!("Module lifecycle method '{method}' is missing."))?;
        let value = match input {
            Some(input) => function.call1(instance, input),
            None => function.call0(instance),
        }
        .map_err(js_error)?;
        await_value(value).await
    }

    async fn await_value(value: JsValue) -> Result<JsValue, String> {
        const LIFECYCLE_TIMEOUT_MS: f64 = 10_000.0;
        let timeout = Function::new_with_args(
            "value, milliseconds",
            "return Promise.race([Promise.resolve(value), new Promise((_, reject) => setTimeout(() => reject(new Error('Module lifecycle operation timed out')), milliseconds))]);",
        );
        let raced = timeout
            .call2(
                &JsValue::UNDEFINED,
                &value,
                &JsValue::from_f64(LIFECYCLE_TIMEOUT_MS),
            )
            .map_err(js_error)?
            .dyn_into::<Promise>()
            .map_err(|_| "Lifecycle timeout wrapper did not return a Promise.".to_string())?;
        JsFuture::from(raced).await.map_err(js_error)
    }

    fn update_document(bootstrap: &BrowserLifecycleBootstrapV1) {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            document.set_title(&format!("{} · Tessara", bootstrap.title));
            if let Some(outlet) = document.get_element_by_id(MODULE_OUTLET_ID) {
                let _ = outlet.set_attribute("aria-busy", "false");
                let _ = outlet.set_attribute(
                    "data-module-release",
                    &bootstrap.release_version.to_string(),
                );
                if let Ok(Some(heading)) = outlet.query_selector("h1")
                    && let Ok(heading) = heading.dyn_into::<web_sys::HtmlElement>()
                {
                    let _ = heading.set_attribute("tabindex", "-1");
                    let _ = heading.focus();
                }
            }
        }
    }

    fn js_error(error: JsValue) -> String {
        error.as_string().unwrap_or_else(|| format!("{error:?}"))
    }
}
