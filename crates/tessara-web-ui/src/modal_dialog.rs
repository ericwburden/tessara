//! Controlled modal and fullscreen dialog primitives.

use icons::X;
use leptos::{ev, portal::Portal, prelude::*};

/// Size treatment for [`ModalDialog`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ModalDialogSize {
    #[default]
    Default,
    Fullscreen,
}

impl ModalDialogSize {
    fn modifier(self) -> &'static str {
        match self {
            Self::Default => "modal-dialog--default",
            Self::Fullscreen => "modal-dialog--fullscreen",
        }
    }
}

/// A controlled modal surface with shared labeling, dismissal, focus trapping,
/// background inertness, scroll locking, and focus restoration behavior.
#[component]
pub fn ModalDialog(
    #[prop(into)] id: String,
    #[prop(into)] title: String,
    open: Signal<bool>,
    on_close: Callback<()>,
    children: ChildrenFn,
    #[prop(optional, into)] description: String,
    #[prop(optional)] size: ModalDialogSize,
    #[prop(default = "Close dialog")] close_label: &'static str,
    #[prop(default = Callback::new(|_| {}))] on_after_close: Callback<()>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    view! {
        <Portal>
            <ModalDialogSurface
                id=id.clone()
                title=title.clone()
                open
                on_close
                description=description.clone()
                size
                close_label
                on_after_close
                class=class.clone()
                children=children.clone()
            />
        </Portal>
    }
}

#[component]
fn ModalDialogSurface(
    id: String,
    title: String,
    open: Signal<bool>,
    on_close: Callback<()>,
    children: ChildrenFn,
    description: String,
    size: ModalDialogSize,
    close_label: &'static str,
    on_after_close: Callback<()>,
    class: String,
) -> impl IntoView {
    let title_id = format!("{id}-title");
    let description_id = (!description.trim().is_empty()).then(|| format!("{id}-description"));
    let title_id_for_dialog = title_id.clone();
    let description_id_for_dialog = description_id.clone();
    let description_id_for_content = description_id.clone();
    let dialog_class = if class.trim().is_empty() {
        format!("modal-dialog blurred-surface {}", size.modifier())
    } else {
        format!(
            "modal-dialog blurred-surface {} {}",
            size.modifier(),
            class.trim()
        )
    };
    let dismiss = move || {
        on_close.run(());
        on_after_close.run(());
    };
    let dismiss_from_scrim = dismiss;
    let dismiss_from_button = dismiss;
    let dismiss_from_keyboard = dismiss;
    let dismiss_from_document = dismiss;
    let close_button = NodeRef::<leptos::html::Button>::new();
    manage_dialog(open, id.clone(), close_button, dismiss_from_document);

    view! {
        <section
            id=id
            class="modal-overlay"
            hidden=move || !open.get()
            inert=move || !open.get()
            aria-hidden=move || (!open.get()).to_string()
        >
                <button
                    class="modal-overlay__scrim"
                    type="button"
                    aria-label=close_label
                    tabindex="-1"
                    on:click=move |_| dismiss_from_scrim()
                ></button>
                <div
                    class=dialog_class.clone()
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby=title_id_for_dialog.clone()
                    aria-describedby=description_id_for_dialog.clone()
                    tabindex="-1"
                    on:keydown=move |event| {
                        handle_dialog_keydown(event, dismiss_from_keyboard);
                    }
                >
                    <header class="modal-dialog__header">
                        <div>
                            <h2 id=title_id.clone()>{title.clone()}</h2>
                            {(!description.trim().is_empty()).then(|| {
                                view! { <p id=description_id_for_content.clone()>{description.clone()}</p> }
                            })}
                        </div>
                        <button
                            node_ref=close_button
                            class="icon-button modal-dialog__close"
                            type="button"
                            aria-label=close_label
                            title=close_label
                            on:click=move |_| dismiss_from_button()
                        >
                            <X class="icon-button__icon"/>
                        </button>
                    </header>
                    <div class="modal-dialog__content">{children()}</div>
                </div>
        </section>
    }
}

/// Convenience wrapper for a fullscreen [`ModalDialog`].
#[component]
pub fn FullscreenDialog(
    #[prop(into)] id: String,
    #[prop(into)] title: String,
    open: Signal<bool>,
    on_close: Callback<()>,
    children: ChildrenFn,
    #[prop(optional, into)] description: String,
    #[prop(default = "Close fullscreen view")] close_label: &'static str,
    #[prop(default = Callback::new(|_| {}))] on_after_close: Callback<()>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    view! {
        <ModalDialog
            id=id
            title=title
            open=open
            on_close=on_close
            description=description
            size=ModalDialogSize::Fullscreen
            close_label=close_label
            on_after_close=on_after_close
            class=class
            children=children
        />
    }
}

/// Applies the shared Escape and Tab-loop behavior to a dialog container.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub(crate) fn handle_dialog_keydown(event: ev::KeyboardEvent, dismiss: impl FnOnce()) {
    use wasm_bindgen::JsCast;

    if event.key() == "Escape" {
        event.prevent_default();
        event.stop_propagation();
        dismiss();
        return;
    }
    if event.key() != "Tab" {
        return;
    }
    let Some(dialog) = event
        .current_target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Ok(nodes) = dialog.query_selector_all(
        "a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])",
    ) else {
        return;
    };
    let focusable = (0..nodes.length())
        .filter_map(|index| nodes.item(index))
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .collect::<Vec<_>>();
    let (Some(first), Some(last)) = (focusable.first(), focusable.last()) else {
        event.prevent_default();
        if let Ok(dialog) = dialog.dyn_into::<web_sys::HtmlElement>() {
            let _ = dialog.focus();
        }
        return;
    };
    let active = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element());
    if event.shift_key() && active.as_ref() == Some(first.as_ref()) {
        event.prevent_default();
        let _ = last.focus();
    } else if !event.shift_key() && active.as_ref() == Some(last.as_ref()) {
        event.prevent_default();
        let _ = first.focus();
    }
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
pub(crate) fn handle_dialog_keydown(_: ev::KeyboardEvent, _: impl FnOnce()) {}

/// Coordinates one dialog's environment and focus lifecycle.
///
/// Keeping these operations in one effect is intentional: the background must
/// be made interactive again before focus can be restored to the opener. It
/// also gives conditionally rendered dialogs the same close path during owner
/// cleanup as dialogs whose controlled `open` signal becomes false.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub(crate) fn manage_dialog(
    open: Signal<bool>,
    id: String,
    close_button: NodeRef<leptos::html::Button>,
    dismiss: impl Fn() + 'static,
) {
    let was_open = StoredValue::new(false);
    let cleanup_id = id.clone();
    let effect_id = id.clone();
    Effect::new(move |_| {
        let is_open = open.get();
        let was_open_value = was_open.get_value();
        if is_open && !was_open_value {
            update_dialog_environment(&effect_id, true);
            if let Some(button) = close_button.get() {
                let _ = button.focus();
            }
        } else if !is_open && was_open_value {
            if let Some(element) = update_dialog_environment(&effect_id, false) {
                restore_dialog_focus(element);
            }
        }
        was_open.set_value(is_open);
    });
    on_cleanup(move || {
        if was_open.get_value()
            && let Some(element) = update_dialog_environment(&cleanup_id, false)
        {
            restore_dialog_focus(element);
        }
    });
    register_document_escape(open, id, dismiss);
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn restore_dialog_focus(element: web_sys::HtmlElement) {
    use wasm_bindgen::{JsCast, closure::Closure};

    if focus_element(&element) {
        return;
    }

    let Some(window) = web_sys::window() else {
        return;
    };
    let callback = Closure::once(move |_: f64| {
        let _ = focus_element(&element);
    });
    if window
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .is_ok()
    {
        callback.forget();
    }
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
pub(crate) fn manage_dialog(
    _: Signal<bool>,
    _: String,
    _: NodeRef<leptos::html::Button>,
    _: impl Fn() + 'static,
) {
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn focus_element(element: &web_sys::HtmlElement) -> bool {
    let _ = element.focus();
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element())
        .as_ref()
        == Some(element.as_ref())
}

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
struct DialogStack<T> {
    open_ids: Vec<String>,
    focus_origins: std::collections::HashMap<String, T>,
}

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
impl<T> Default for DialogStack<T> {
    fn default() -> Self {
        Self {
            open_ids: Vec::new(),
            focus_origins: std::collections::HashMap::new(),
        }
    }
}

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
impl<T> DialogStack<T> {
    fn opened(&mut self, id: &str, focus_origin: Option<T>) {
        self.open_ids.retain(|open_id| open_id != id);
        self.focus_origins.remove(id);
        self.open_ids.push(id.to_string());
        if let Some(focus_origin) = focus_origin {
            self.focus_origins.insert(id.to_string(), focus_origin);
        }
    }

    fn closed(&mut self, id: &str) -> Option<T> {
        self.open_ids.retain(|open_id| open_id != id);
        self.focus_origins.remove(id)
    }

    fn active(&self) -> Option<&str> {
        self.open_ids.last().map(String::as_str)
    }

    fn is_empty(&self) -> bool {
        self.open_ids.is_empty()
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
struct DialogEnvironment {
    stack: DialogStack<web_sys::HtmlElement>,
    original_inert: Vec<(web_sys::Element, bool)>,
    body_overflow: Option<String>,
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
impl DialogEnvironment {
    fn new() -> Self {
        Self {
            stack: DialogStack::default(),
            original_inert: Vec::new(),
            body_overflow: None,
        }
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static DIALOG_ENVIRONMENT: std::cell::RefCell<DialogEnvironment> =
        std::cell::RefCell::new(DialogEnvironment::new());
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
struct DocumentEscapeRegistration {
    document: web_sys::Document,
    callback: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::KeyboardEvent)>,
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static NEXT_DOCUMENT_ESCAPE_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
    static DOCUMENT_ESCAPE_REGISTRATIONS:
        std::cell::RefCell<std::collections::HashMap<u64, DocumentEscapeRegistration>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Keeps Escape reliable while a reactive child replacement temporarily
/// leaves focus on the document. Dialog-local keyboard handling remains the
/// normal path and stops propagation before this fallback runs.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn register_document_escape(open: Signal<bool>, id: String, dismiss: impl Fn() + 'static) {
    use wasm_bindgen::JsCast;

    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let callback = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
        move |event: web_sys::KeyboardEvent| {
            if event.key() != "Escape" || !open.get_untracked() || !dialog_is_active(&id) {
                return;
            }
            event.prevent_default();
            event.stop_immediate_propagation();
            dismiss();
        },
    );
    if document
        .add_event_listener_with_callback("keydown", callback.as_ref().unchecked_ref())
        .is_err()
    {
        return;
    }
    let registration_id = NEXT_DOCUMENT_ESCAPE_ID.with(|next_id| {
        let registration_id = next_id.get();
        next_id.set(registration_id.wrapping_add(1).max(1));
        registration_id
    });
    DOCUMENT_ESCAPE_REGISTRATIONS.with(|registrations| {
        registrations.borrow_mut().insert(
            registration_id,
            DocumentEscapeRegistration { document, callback },
        );
    });
    on_cleanup(move || remove_document_escape(registration_id));
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn dialog_is_active(id: &str) -> bool {
    DIALOG_ENVIRONMENT.with_borrow(|environment| environment.stack.active() == Some(id))
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn remove_document_escape(registration_id: u64) {
    use wasm_bindgen::JsCast;

    DOCUMENT_ESCAPE_REGISTRATIONS.with(|registrations| {
        let Some(registration) = registrations.borrow_mut().remove(&registration_id) else {
            return;
        };
        let _ = registration.document.remove_event_listener_with_callback(
            "keydown",
            registration.callback.as_ref().unchecked_ref(),
        );
    });
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn update_dialog_environment(id: &str, opening: bool) -> Option<web_sys::HtmlElement> {
    use wasm_bindgen::JsCast;

    DIALOG_ENVIRONMENT.with_borrow_mut(|environment| {
        let focus_origin = if opening {
            let focus_origin = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.active_element())
                .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
            environment.stack.opened(id, focus_origin);
            None
        } else {
            environment.stack.closed(id)
        };
        if environment.stack.is_empty() {
            restore_dialog_environment(environment);
        } else {
            apply_dialog_environment(environment);
        }
        focus_origin
    })
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn apply_dialog_environment(environment: &mut DialogEnvironment) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    if environment.body_overflow.is_none() {
        environment.body_overflow = Some(
            body.style()
                .get_property_value("overflow")
                .unwrap_or_default(),
        );
    }
    let _ = body.style().set_property("overflow", "hidden");

    let active_container = environment
        .stack
        .active()
        .and_then(|id| document.get_element_by_id(id))
        .and_then(body_child_for_dialog);
    let children = body.children();
    for index in 0..children.length() {
        let Some(element) = children.item(index) else {
            continue;
        };
        if !environment
            .original_inert
            .iter()
            .any(|(known, _)| known == &element)
        {
            environment
                .original_inert
                .push((element.clone(), element.has_attribute("inert")));
        }
        if active_container.as_ref() == Some(&element) {
            let _ = element.remove_attribute("inert");
        } else {
            let _ = element.set_attribute("inert", "");
        }
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn body_child_for_dialog(mut element: web_sys::Element) -> Option<web_sys::Element> {
    loop {
        let parent = element.parent_element()?;
        if parent.tag_name() == "BODY" {
            return Some(element);
        }
        element = parent;
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn restore_dialog_environment(environment: &mut DialogEnvironment) {
    for (element, was_inert) in environment.original_inert.drain(..) {
        if was_inert {
            let _ = element.set_attribute("inert", "");
        } else {
            let _ = element.remove_attribute("inert");
        }
    }
    if let Some(body) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.body())
        && let Some(overflow) = environment.body_overflow.take()
    {
        if overflow.is_empty() {
            let _ = body.style().remove_property("overflow");
        } else {
            let _ = body.style().set_property("overflow", &overflow);
        }
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn fullscreen_dialog_renders_shared_modal_semantics() {
        let html = Owner::new().with(|| {
            view! {
                <ModalDialogSurface
                    id="table-fullscreen".to_string()
                    title="Orders".to_string()
                    description="Full table view".to_string()
                    open=Signal::derive(|| true)
                    on_close=Callback::new(|_| {})
                    size=ModalDialogSize::Fullscreen
                    close_label="Close fullscreen view"
                    on_after_close=Callback::new(|_| {})
                    class=String::new()
                >
                    <table><tbody><tr><td>"Example"</td></tr></tbody></table>
                </ModalDialogSurface>
            }
            .to_html()
        });

        assert!(html.contains("role=\"dialog\""));
        assert!(html.contains("aria-modal=\"true\""));
        assert!(html.contains("aria-labelledby=\"table-fullscreen-title\""));
        assert!(html.contains("aria-describedby=\"table-fullscreen-description\""));
        assert!(html.contains("modal-dialog--fullscreen"));
        assert!(html.contains("modal-dialog__content"));
        assert!(html.contains("<table>"));
    }

    #[test]
    fn dialog_stack_keeps_the_most_recent_dialog_active() {
        let mut stack = DialogStack::default();
        stack.opened("placement-details", Some("open-placement"));
        stack.opened("table-fullscreen", Some("open-fullscreen"));
        assert_eq!(stack.active(), Some("table-fullscreen"));

        assert_eq!(stack.closed("table-fullscreen"), Some("open-fullscreen"));
        assert_eq!(stack.active(), Some("placement-details"));
        assert!(!stack.is_empty());

        assert_eq!(stack.closed("placement-details"), Some("open-placement"));
        assert!(stack.is_empty());
    }

    #[test]
    fn dialog_stack_returns_focus_origin_when_open_dialog_is_unmounted() {
        let mut stack = DialogStack::default();
        stack.opened("visibility-scope", Some("show-scope"));

        assert_eq!(stack.closed("visibility-scope"), Some("show-scope"));
        assert!(stack.is_empty());
        assert_eq!(stack.closed("visibility-scope"), None);
    }
}
