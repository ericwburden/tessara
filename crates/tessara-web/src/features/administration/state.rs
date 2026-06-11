//! State helpers for Administration feature interactions.

use leptos::prelude::{RwSignal, Update};

/// Toggles a string value in a selected values signal.
pub(crate) fn toggle_string_selection(
    selection: RwSignal<Vec<String>>,
    value: String,
    selected: bool,
) {
    selection.update(|values| {
        if selected {
            if !values.iter().any(|existing| existing == &value) {
                values.push(value);
            }
        } else {
            values.retain(|existing| existing != &value);
        }
    });
}
