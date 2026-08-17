//! Summary helpers for dataset detail panels.

use chrono::{DateTime, Utc};
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

pub(super) fn readable_refresh_timestamp(value: Option<&str>, empty: &'static str) -> String {
    value
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| {
            value
                .with_timezone(&Utc)
                .format("%b %d, %Y at %H:%M UTC")
                .to_string()
        })
        .unwrap_or_else(|| empty.to_string())
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

#[cfg(test)]
mod tests {
    use super::readable_refresh_timestamp;

    #[test]
    fn refresh_timestamp_is_readable_bounded_utc_text() {
        assert_eq!(
            readable_refresh_timestamp(Some("2026-08-17T17:02:43.262473+00:00"), "Not yet"),
            "Aug 17, 2026 at 17:02 UTC"
        );
        assert_eq!(readable_refresh_timestamp(None, "Not yet"), "Not yet");
        assert_eq!(
            readable_refresh_timestamp(Some("not-a-timestamp"), "Not checked yet"),
            "Not checked yet"
        );
    }
}
