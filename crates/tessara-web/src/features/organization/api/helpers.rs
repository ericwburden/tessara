//! Small helpers shared by organization API submodules and adjacent feature pages.
//!
//! Keep query-string parsing and narrow string conversion helpers here when they support organization workflows but do not belong to a specific form, workflow, or node endpoint group.

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) trait IntoNonemptyString {
    /// Handles the into nonempty behavior.
    fn into_nonempty(self) -> Option<String>;
}

impl IntoNonemptyString for String {
    /// Handles the into nonempty behavior.
    fn into_nonempty(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self) }
    }
}

#[cfg(feature = "hydrate")]
/// Handles the current search param behavior.
pub(crate) fn current_search_param(name: &str) -> Option<String> {
    let search = web_sys::window().and_then(|window| window.location().search().ok())?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get(name).filter(|value| !value.is_empty())
}
