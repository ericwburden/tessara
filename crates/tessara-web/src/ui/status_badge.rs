//! Owns the ui::status_badge module behavior.

use leptos::prelude::*;

#[component]
/// Renders the status badge view.
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
