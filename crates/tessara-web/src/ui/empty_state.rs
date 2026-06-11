//! Owns the ui::empty_state module behavior.

use leptos::prelude::*;

#[component]
/// Renders the empty state view.
pub fn EmptyState(title: &'static str, message: &'static str) -> impl IntoView {
    view! {
        <section class="empty-state">
            <h3>{title}</h3>
            <p>{message}</p>
        </section>
    }
}
