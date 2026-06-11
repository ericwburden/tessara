//! Owns the ui::shell::mobile_nav module behavior.

use super::nav::SidebarContent;
use icons::Menu;
use leptos::prelude::*;

#[component]
/// Renders the mobile nav view.
pub fn MobileNav(active_route: &'static str) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let nav_class = move || {
        if is_open.get() {
            "mobile-nav is-open"
        } else {
            "mobile-nav"
        }
    };

    view! {
        <div class=nav_class>
            <button
                class="icon-button mobile-nav__toggle"
                type="button"
                aria-label="Open navigation"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.set(true)
            >
                <Menu class="icon-button__icon"/>
            </button>
            <button
                class="mobile-nav__scrim"
                type="button"
                aria-label="Close navigation"
                on:click=move |_| is_open.set(false)
            ></button>
            <aside class="mobile-nav__panel blurred-surface" aria-label="Primary navigation">
                <SidebarContent active_route/>
            </aside>
        </div>
    }
}
