//! Shared filtering and option helpers.
//!
//! Keep only reusable filtering and slug logic here; product-owned filtering lives in the owning feature.

use std::collections::HashSet;

pub(crate) fn unique_filter_options(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut options = values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

/// Handles the slug from label behavior.
pub(crate) fn slug_from_label(label: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for character in label
        .trim()
        .chars()
        .flat_map(|character| character.to_lowercase())
    {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    slug
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the unique slug from label behavior.
pub(crate) fn unique_slug_from_label(label: &str, existing_slugs: &[String]) -> String {
    let base = slug_from_label(label);
    if base.is_empty() {
        return String::new();
    }

    let existing = existing_slugs.iter().cloned().collect::<HashSet<_>>();
    if !existing.contains(&base) {
        return base;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !existing.contains(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}
