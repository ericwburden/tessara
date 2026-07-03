//! Empty-state presentation component.
//!
//! This module owns the reusable message pattern for views with no records or no matching results.

use leptos::prelude::*;

#[component]
pub fn EmptyState(#[prop(into)] title: String, #[prop(into)] message: String) -> impl IntoView {
    view! {
        <section class="empty-state">
            <h3>{title}</h3>
            <p>{message}</p>
        </section>
    }
}
