//! Shared filtering and option helpers.
//!
//! Keep only reusable filtering logic here; product-owned filtering lives in the owning feature.

pub(crate) fn unique_filter_options(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut options = values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}
