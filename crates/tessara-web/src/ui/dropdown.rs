//! Shared dropdown menu primitives.
//!
//! Keep generic trigger, menu, and item composition here; feature-specific choices and side effects belong in caller modules.

use icons::Ellipsis;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[cfg(feature = "hydrate")]
/// Handles the scroll app main by behavior.
fn scroll_app_main_by(delta_y: f64) {
    let Some(scroller) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.query_selector(".app-main").ok().flatten())
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };

    let next_scroll_top = (scroller.scroll_top() as f64 + delta_y).round().max(0.0) as i32;
    scroller.set_scroll_top(next_scroll_top);
}

#[component]
/// Renders the dropdown menu view.
pub fn DropdownMenu(#[prop(into)] label: String, children: Children) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let menu_class = move || {
        if is_open.get() {
            "dropdown-menu is-open"
        } else {
            "dropdown-menu"
        }
    };

    view! {
        <div class=menu_class>
            <button
                class="icon-button"
                type="button"
                aria-label=label.clone()
                title=label
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <span aria-hidden="true">
                    <Ellipsis class="icon-button__icon"/>
                </span>
            </button>
            <button
                class="dropdown-menu__scrim"
                type="button"
                aria-label="Close menu"
                on:wheel=move |event| {
                    let _ = &event;
                    #[cfg(feature = "hydrate")]
                    {
                        event.prevent_default();
                        scroll_app_main_by(event.delta_y());
                    }
                }
                on:click=move |_| is_open.set(false)
            ></button>
            <div
                class="dropdown-menu__content blurred-surface"
                role="menu"
                on:click=move |_| is_open.set(false)
            >
                {children()}
            </div>
        </div>
    }
}
