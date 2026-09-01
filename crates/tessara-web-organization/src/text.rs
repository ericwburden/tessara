//! Feature-local text helpers.

pub(crate) fn text_matches(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    values
        .iter()
        .any(|value| value.to_lowercase().contains(&query))
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) trait IntoNonemptyString {
    fn into_nonempty(self) -> Option<String>;
}

impl IntoNonemptyString for String {
    fn into_nonempty(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self) }
    }
}
