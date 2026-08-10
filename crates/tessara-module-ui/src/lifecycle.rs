//! Reusable Leptos/WASM adapter for browser lifecycle ABI v1.

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use leptos::prelude::{AnyView, Get, IntoView, RwSignal, Set};

/// Owns one Leptos reactive root and guarantees that replacing or dropping the
/// adapter unmounts the view and disposes its reactive owner.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[derive(Default)]
pub struct LeptosLifecycleRoot {
    handle: Option<Box<dyn std::any::Any>>,
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
impl LeptosLifecycleRoot {
    pub const fn new() -> Self {
        Self { handle: None }
    }

    pub fn mount<F, N>(&mut self, outlet: web_sys::HtmlElement, view: F)
    where
        F: FnOnce() -> N + 'static,
        N: IntoView,
        N::State: 'static,
    {
        self.unmount();
        outlet.set_inner_html("");
        self.handle = Some(Box::new(leptos::mount::mount_to(outlet, view)));
    }

    pub fn unmount(&mut self) {
        drop(self.handle.take());
    }

    pub fn is_mounted(&self) -> bool {
        self.handle.is_some()
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
impl Drop for LeptosLifecycleRoot {
    fn drop(&mut self) {
        self.unmount();
    }
}

/// Canonical lifecycle state machine for a hydrated module mounted inside
/// Core. Product crates provide only typed bootstrap state and a view function.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub struct LeptosLifecycleAdapter<T>
where
    T: Clone + Send + Sync + 'static,
{
    root: LeptosLifecycleRoot,
    state: Option<RwSignal<T>>,
    outlet_id: Option<String>,
    dirty: bool,
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
impl<T> LeptosLifecycleAdapter<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub const fn new() -> Self {
        Self {
            root: LeptosLifecycleRoot::new(),
            state: None,
            outlet_id: None,
            dirty: false,
        }
    }

    pub fn mount<F>(
        &mut self,
        outlet_id: &str,
        outlet: web_sys::HtmlElement,
        bootstrap: T,
        render: F,
    ) where
        F: Fn(T) -> AnyView + Clone + Send + Sync + 'static,
    {
        self.unmount();
        let state = RwSignal::new(bootstrap);
        self.root.mount(outlet.clone(), move || {
            let render = render.clone();
            leptos::view! { {move || render(state.get())} }
        });
        self.state = Some(state);
        self.outlet_id = Some(outlet_id.to_string());
        self.dirty = false;
        let _ = outlet.set_attribute("data-module-lifecycle", "active");
    }

    pub fn navigate(&self, bootstrap: T) -> Result<(), &'static str> {
        self.state.ok_or("module is not mounted")?.set(bootstrap);
        Ok(())
    }

    pub fn can_deactivate(&self) -> bool {
        !self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn set_suspended(&self, suspended: bool) {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let Some(outlet) = self
            .outlet_id
            .as_deref()
            .and_then(|id| document.get_element_by_id(id))
        else {
            return;
        };
        let _ = outlet.set_attribute(
            "data-module-lifecycle",
            if suspended { "suspended" } else { "active" },
        );
        let _ = outlet.toggle_attribute_with_force("hidden", suspended);
        let _ = outlet.toggle_attribute_with_force("inert", suspended);
    }

    pub fn unmount(&mut self) {
        self.root.unmount();
        self.state = None;
        self.outlet_id = None;
        self.dirty = false;
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
impl<T> Default for LeptosLifecycleAdapter<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
