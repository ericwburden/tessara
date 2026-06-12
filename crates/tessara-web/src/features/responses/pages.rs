//! Route-level page composition for the Responses feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use leptos::prelude::*;

use super::detail::ResponsesDetailPageContent;
use super::edit::ResponsesEditPageContent;
use super::list::ResponsesPageContent;
use super::start::ResponsesNewPageContent;

#[component]
pub fn ResponsesPage() -> impl IntoView {
    view! { <ResponsesPageContent/> }
}

#[component]
pub fn ResponsesNewPage() -> impl IntoView {
    view! { <ResponsesNewPageContent/> }
}

#[component]
pub fn ResponsesDetailPage() -> impl IntoView {
    view! { <ResponsesDetailPageContent/> }
}

#[component]
pub fn ResponsesEditPage() -> impl IntoView {
    view! { <ResponsesEditPageContent/> }
}
