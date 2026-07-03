//! Timestamp display component.
//!
//! This module owns browser-local timestamp formatting and the reusable component that presents API timestamps in feature views.

use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use js_sys::Date;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsValue;

#[component]
pub fn Timestamp(value: String) -> impl IntoView {
    let datetime = value.clone();
    let display_value = RwSignal::new(value.clone());

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        display_value.set(format_local_timestamp(&value));
    });

    view! {
        <time datetime=datetime>{move || display_value.get()}</time>
    }
}

#[cfg(feature = "hydrate")]
/// Formats the format local timestamp value.
fn format_local_timestamp(value: &str) -> String {
    let date = Date::new(&JsValue::from_str(value));
    if date.get_time().is_nan() {
        return value.to_string();
    }

    let month = date.get_month() + 1;
    let day = date.get_date();
    let year = date.get_full_year() % 100;
    let mut hour = date.get_hours();
    let minute = date.get_minutes();
    let second = date.get_seconds();
    let meridiem = if hour >= 12 { "PM" } else { "AM" };

    hour %= 12;
    if hour == 0 {
        hour = 12;
    }

    format!("{month:02}/{day:02}/{year:02} {hour:02}:{minute:02}:{second:02} {meridiem}")
}
