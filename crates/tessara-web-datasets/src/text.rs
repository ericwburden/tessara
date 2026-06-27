//! Text helpers local to the datasets feature crate.

pub(crate) fn text_matches(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    values
        .iter()
        .any(|value| value.to_lowercase().contains(&query))
}

pub(crate) fn sentence_label(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
