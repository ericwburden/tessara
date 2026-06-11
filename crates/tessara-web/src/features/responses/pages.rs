//! Response route components.

use leptos::prelude::*;

use super::detail::ResponsesDetailPageContent;
use super::edit::ResponsesEditPageContent;
use super::list::ResponsesPageContent;
use super::start::ResponsesNewPageContent;

#[component]
/// Renders the responses page view.
pub fn ResponsesPage() -> impl IntoView {
    view! { <ResponsesPageContent/> }
}

#[component]
/// Renders the responses new page view.
pub fn ResponsesNewPage() -> impl IntoView {
    view! { <ResponsesNewPageContent/> }
}

#[component]
/// Renders the responses detail page view.
pub fn ResponsesDetailPage() -> impl IntoView {
    view! { <ResponsesDetailPageContent/> }
}

#[component]
/// Renders the responses edit page view.
pub fn ResponsesEditPage() -> impl IntoView {
    view! { <ResponsesEditPageContent/> }
}
