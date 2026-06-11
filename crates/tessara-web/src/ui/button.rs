//! Shared button primitives.
//!
//! Keep general-purpose button variants here when they are not coupled to a feature workflow or form state.

use leptos::prelude::*;

#[component]
/// Renders the button view.
pub fn Button(label: &'static str, #[prop(optional)] href: Option<&'static str>) -> impl IntoView {
    match href {
        Some(href) => view! { <a class="button" href=href>{label}</a> }.into_any(),
        None => view! { <button class="button" type="button">{label}</button> }.into_any(),
    }
}
