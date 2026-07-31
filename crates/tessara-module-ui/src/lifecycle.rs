//! Reusable Leptos/WASM adapter for browser lifecycle ABI v1.

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use leptos::prelude::IntoView;

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
