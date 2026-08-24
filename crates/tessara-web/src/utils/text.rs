//! Generic text matching and label helpers.
//!
//! This module owns search-friendly string comparisons and fallback label formatting used across filters and tables.

pub(crate) fn text_matches(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    values
        .iter()
        .any(|value| value.to_lowercase().contains(&query))
}
