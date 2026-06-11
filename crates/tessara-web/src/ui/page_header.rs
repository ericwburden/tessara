//! Shared page header components.
//!
//! This module owns consistent title, subtitle, action, and toolbar layout for top-level feature pages.

use leptos::prelude::*;

#[component]
/// Renders the page header view.
pub fn PageHeader(
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <header class="page-header">
            <div>
                <h2>{title}</h2>
                {description
                    .map(|description| view! { <p>{description}</p> })}
            </div>
            <div class="page-header__actions">
                {children.map(|children| children())}
            </div>
        </header>
    }
}
