//! Summary helpers for dataset detail panels.

use leptos::prelude::*;

#[component]
pub(super) fn MetricCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="metric-card">
            <span>{label}</span>
            <strong>{value}</strong>
        </div>
    }
}

pub(super) fn tab_class(
    active_tab: RwSignal<String>,
    value: &'static str,
) -> impl Fn() -> &'static str {
    move || {
        if active_tab.get() == value {
            "tabs-trigger is-active"
        } else {
            "tabs-trigger"
        }
    }
}
