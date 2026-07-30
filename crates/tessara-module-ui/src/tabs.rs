//! Shared tab components.
//!
//! Keep accessible tab list, trigger, and content primitives here while tab labels and feature state remain with callers.

use leptos::prelude::*;

#[component]
pub fn Tabs(active: RwSignal<String>, children: Children) -> impl IntoView {
    view! {
        <div class="tabs" data-active=move || active.get()>
            {children()}
        </div>
    }
}

#[component]
pub fn TabsList(children: Children) -> impl IntoView {
    view! {
        <div class="tabs-list" role="tablist">
            {children()}
        </div>
    }
}

#[component]
pub fn TabsTrigger(
    active: RwSignal<String>,
    value: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                if active.get() == value {
                    "tabs-trigger is-active"
                } else {
                    "tabs-trigger"
                }
            }
            type="button"
            role="tab"
            aria-selected=move || (active.get() == value).to_string()
            on:click=move |_| active.set(value.to_string())
        >
            {children()}
        </button>
    }
}

#[component]
pub fn TabsContent(
    active: RwSignal<String>,
    value: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class="tabs-content"
            role="tabpanel"
            hidden=move || active.get() != value
        >
            {children()}
        </section>
    }
}
