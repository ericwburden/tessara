use leptos::prelude::*;

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static NEXT_LISTENER_ID: Cell<u64> = const { Cell::new(1) };
    static LISTENERS: RefCell<BTreeMap<u64, DirtyNavigationListeners>> =
        const { RefCell::new(BTreeMap::new()) };
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
struct DirtyNavigationListeners {
    beforeunload: Closure<dyn FnMut(web_sys::Event)>,
    click: Closure<dyn FnMut(web_sys::MouseEvent)>,
    popstate: Closure<dyn FnMut(web_sys::Event)>,
}

/// Tracks the browser history entry protected by the editor.
///
/// The Navigation API supplies stable entry indices in Chromium. Comparing the
/// protected and target indices lets the guard restore either a Back or Forward
/// traversal without guessing its direction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
struct HistorySentinel {
    protected_index: i32,
    restoring: bool,
}

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
impl HistorySentinel {
    const fn new(protected_index: i32) -> Self {
        Self {
            protected_index,
            restoring: false,
        }
    }

    fn complete_restoration(&mut self, target_index: i32) -> bool {
        if self.restoring && target_index == self.protected_index {
            self.restoring = false;
            true
        } else {
            false
        }
    }

    fn protect(&mut self, target_index: i32) {
        self.protected_index = target_index;
        self.restoring = false;
    }

    fn begin_restoration(&mut self, target_index: i32) -> Option<i32> {
        let delta = self.protected_index.checked_sub(target_index)?;
        if delta == 0 {
            return None;
        }
        self.restoring = true;
        Some(delta)
    }

    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    fn cancel_restoration(&mut self) {
        self.restoring = false;
    }
}

#[cfg(feature = "hydrate")]
pub(super) fn navigate(href: &str) {
    crate::navigate_dashboard_href(href);
}

#[cfg(not(feature = "hydrate"))]
pub(super) fn navigate(_: &str) {}

#[cfg(feature = "hydrate")]
pub(super) fn navigate_with_dirty_confirmation(
    href: &str,
    dirty: RwSignal<bool>,
    confirmed_navigation: RwSignal<bool>,
) {
    if dirty.get_untracked() && !confirmed_navigation.get_untracked() {
        let confirmed = web_sys::window()
            .and_then(|window| {
                window
                    .confirm_with_message("Discard unsaved Dashboard layout changes?")
                    .ok()
            })
            .unwrap_or(false);
        if !confirmed {
            return;
        }
    }
    confirmed_navigation.set(true);
    navigate(href);
}

#[cfg(not(feature = "hydrate"))]
pub(super) fn navigate_with_dirty_confirmation(_: &str, _: RwSignal<bool>, _: RwSignal<bool>) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub(super) fn install_dirty_navigation_guard(
    dirty: RwSignal<bool>,
    confirmed_navigation: RwSignal<bool>,
) {
    let beforeunload = Closure::wrap(Box::new(move |event: web_sys::Event| {
        if dirty.get_untracked() && !confirmed_navigation.get_untracked() {
            event.prevent_default();
            let _ = js_sys::Reflect::set(
                event.as_ref(),
                &JsValue::from_str("returnValue"),
                &JsValue::from_str(""),
            );
        }
    }) as Box<dyn FnMut(_)>);
    let click = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if !dirty.get_untracked()
            || confirmed_navigation.get_untracked()
            || event.default_prevented()
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
        let raw_href = anchor.get_attribute("href");
        if !internal_anchor_may_navigate(raw_href.as_deref())
            || anchor.has_attribute("download")
            || !matches!(anchor.target().as_str(), "" | "_self")
        {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let location = window.location();
        if location.origin().ok().as_deref() != Some(anchor.origin().as_str())
            || location.href().ok().as_deref() == Some(anchor.href().as_str())
        {
            return;
        }
        let confirmed = window
            .confirm_with_message("Discard unsaved Dashboard layout changes?")
            .unwrap_or(false);
        if confirmed {
            confirmed_navigation.set(true);
        } else {
            event.prevent_default();
            event.stop_immediate_propagation();
        }
    }) as Box<dyn FnMut(_)>);

    let sentinel = Rc::new(RefCell::new(
        web_sys::window()
            .as_ref()
            .and_then(current_history_index)
            .map(HistorySentinel::new),
    ));
    let sentinel_for_popstate = Rc::clone(&sentinel);
    let popstate = Closure::wrap(Box::new(move |event: web_sys::Event| {
        if confirmed_navigation.get_untracked() {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(target_index) = current_history_index(&window) else {
            // Do not guess a direction when this browser cannot expose one.
            return;
        };
        let mut sentinel = sentinel_for_popstate.borrow_mut();
        let Some(sentinel) = sentinel.as_mut() else {
            return;
        };
        if !dirty.get_untracked() {
            sentinel.protect(target_index);
            return;
        }
        if sentinel.complete_restoration(target_index) {
            return;
        }
        let confirmed = window
            .confirm_with_message("Discard unsaved Dashboard layout changes?")
            .unwrap_or(false);
        if confirmed {
            confirmed_navigation.set(true);
            return;
        }

        event.stop_immediate_propagation();
        let Some(delta) = sentinel.begin_restoration(target_index) else {
            return;
        };
        if window
            .history()
            .and_then(|history| history.go_with_delta(delta))
            .is_err()
        {
            sentinel.cancel_restoration();
        }
    }) as Box<dyn FnMut(_)>);

    let listener_id = NEXT_LISTENER_ID.with(|next_id| {
        let listener_id = next_id.get();
        next_id.set(listener_id.wrapping_add(1).max(1));
        listener_id
    });
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
    {
        let _ = window.add_event_listener_with_callback(
            "beforeunload",
            beforeunload.as_ref().unchecked_ref(),
        );
        let _ = document.add_event_listener_with_callback_and_bool(
            "click",
            click.as_ref().unchecked_ref(),
            true,
        );
        let _ = window.add_event_listener_with_callback_and_bool(
            "popstate",
            popstate.as_ref().unchecked_ref(),
            true,
        );
        LISTENERS.with(|listeners| {
            listeners.borrow_mut().insert(
                listener_id,
                DirtyNavigationListeners {
                    beforeunload,
                    click,
                    popstate,
                },
            );
        });
        on_cleanup(move || remove_dirty_navigation_listeners(listener_id));
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn current_history_index(window: &web_sys::Window) -> Option<i32> {
    let navigation =
        js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("navigation")).ok()?;
    let current_entry =
        js_sys::Reflect::get(&navigation, &JsValue::from_str("currentEntry")).ok()?;
    let index = js_sys::Reflect::get(&current_entry, &JsValue::from_str("index"))
        .ok()?
        .as_f64()?;
    if index.fract() != 0.0 || !(0.0..=f64::from(i32::MAX)).contains(&index) {
        return None;
    }
    Some(index as i32)
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn remove_dirty_navigation_listeners(listener_id: u64) {
    let listeners = LISTENERS.with(|listeners| listeners.borrow_mut().remove(&listener_id));
    let (Some(listeners), Some(window)) = (listeners, web_sys::window()) else {
        return;
    };
    let _ = window.remove_event_listener_with_callback(
        "beforeunload",
        listeners.beforeunload.as_ref().unchecked_ref(),
    );
    if let Some(document) = window.document() {
        let _ = document.remove_event_listener_with_callback_and_bool(
            "click",
            listeners.click.as_ref().unchecked_ref(),
            true,
        );
    }
    let _ = window.remove_event_listener_with_callback_and_bool(
        "popstate",
        listeners.popstate.as_ref().unchecked_ref(),
        true,
    );
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
pub(super) fn install_dirty_navigation_guard(_: RwSignal<bool>, _: RwSignal<bool>) {}

#[cfg_attr(
    not(any(test, all(feature = "hydrate", target_arch = "wasm32"))),
    allow(dead_code)
)]
fn internal_anchor_may_navigate(raw_href: Option<&str>) -> bool {
    let Some(href) = raw_href.map(str::trim).filter(|href| !href.is_empty()) else {
        return false;
    };
    if href.starts_with('#') {
        return false;
    }
    let lowercase = href.to_ascii_lowercase();
    !["javascript:", "mailto:", "tel:", "data:"]
        .iter()
        .any(|scheme| lowercase.starts_with(scheme))
}

#[cfg(test)]
mod tests {
    use super::{HistorySentinel, internal_anchor_may_navigate};

    #[test]
    fn sentinel_restores_a_cancelled_back_traversal_forward() {
        let mut sentinel = HistorySentinel::new(8);
        assert_eq!(sentinel.begin_restoration(7), Some(1));
        assert!(sentinel.complete_restoration(8));
    }

    #[test]
    fn sentinel_restores_a_cancelled_forward_traversal_backward() {
        let mut sentinel = HistorySentinel::new(8);
        assert_eq!(sentinel.begin_restoration(11), Some(-3));
        assert!(sentinel.complete_restoration(8));
    }

    #[test]
    fn sentinel_tracks_clean_same_document_traversals() {
        let mut sentinel = HistorySentinel::new(8);
        sentinel.protect(5);
        assert_eq!(sentinel.begin_restoration(7), Some(-2));
    }

    #[test]
    fn dirty_navigation_guard_targets_real_links_without_blocking_page_controls() {
        assert!(internal_anchor_may_navigate(Some("/dashboards/42")));
        assert!(internal_anchor_may_navigate(Some("components/table")));
        assert!(!internal_anchor_may_navigate(Some("#placement-title")));
        assert!(!internal_anchor_may_navigate(Some(
            "mailto:owner@example.com"
        )));
        assert!(!internal_anchor_may_navigate(Some("javascript:void(0)")));
        assert!(!internal_anchor_may_navigate(None));
    }
}
