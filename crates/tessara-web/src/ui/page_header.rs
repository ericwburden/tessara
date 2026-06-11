//! Owns the ui::page_header module behavior.

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
