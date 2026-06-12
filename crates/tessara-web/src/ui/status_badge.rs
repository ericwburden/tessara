//! Generic status badge component.
//!
//! Keep domain-neutral badge rendering here; status-specific class selection belongs with the feature or shared display helper that understands the value.

use leptos::prelude::*;

#[component]
pub fn StatusBadge(#[prop(into)] label: String) -> impl IntoView {
    let class = match label.as_str() {
        "Available" | "Done" | "Ready" | "Steps Complete" => "status-badge is-success",
        "Error" => "status-badge is-danger",
        "In Progress" => "status-badge is-warning",
        "Pending" => "status-badge is-info",
        _ => "status-badge is-info",
    };

    view! { <span class=class>{label}</span> }
}
