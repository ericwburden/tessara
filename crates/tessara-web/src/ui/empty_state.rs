//! Empty-state presentation component.
//!
//! This module owns the reusable message pattern for views with no records or no matching results.

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
