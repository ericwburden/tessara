//! Response route components.
//!
//! The response implementation still shares DTOs and helpers with the broader
//! native module. Keeping the public route components here gives the next
//! cleanup pass a stable module boundary for moving the remaining response
//! internals without changing app routing.

use super::*;

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
