pub(crate) fn text_matches(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    values
        .iter()
        .any(|value| value.to_lowercase().contains(&query))
}

pub(crate) fn nonempty_text(value: Option<&str>, fallback: &'static str) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}
