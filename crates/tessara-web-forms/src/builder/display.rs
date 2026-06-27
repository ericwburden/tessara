//! Display formatting helpers for the Forms feature.
//!
//! Keep label, class, and summary formatting here when it depends on Forms domain values but not on route state.

use icons::{
    CalendarDays, CircleDot, Hash, ListChecks, SquareCheckBig, TextCursorInput, TextQuote,
};
use leptos::prelude::*;

pub(crate) fn form_builder_field_default_label(field_type: &str, id: usize) -> String {
    if field_type == "static_text" {
        "Static text".into()
    } else {
        format!("Field {id}")
    }
}

pub(crate) fn form_builder_field_type_icon(field_type: &str) -> AnyView {
    match field_type {
        "static_text" => view! { <TextQuote /> }.into_any(),
        "number" => view! { <Hash /> }.into_any(),
        "date" => view! { <CalendarDays /> }.into_any(),
        "boolean" => view! { <SquareCheckBig /> }.into_any(),
        "single_choice" => view! { <CircleDot /> }.into_any(),
        "multi_choice" => view! { <ListChecks /> }.into_any(),
        _ => view! { <TextCursorInput /> }.into_any(),
    }
}
