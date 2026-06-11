//! Owns the ui::button module behavior.

use leptos::prelude::*;

#[component]
/// Renders the button view.
pub fn Button(label: &'static str, #[prop(optional)] href: Option<&'static str>) -> impl IntoView {
    match href {
        Some(href) => view! { <a class="button" href=href>{label}</a> }.into_any(),
        None => view! { <button class="button" type="button">{label}</button> }.into_any(),
    }
}
