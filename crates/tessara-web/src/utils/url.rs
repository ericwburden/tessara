//! Browser URL helpers.

#[cfg(feature = "hydrate")]
pub(crate) fn current_search_param(name: &str) -> Option<String> {
    let search = web_sys::window().and_then(|window| window.location().search().ok())?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get(name).filter(|value| !value.is_empty())
}

#[cfg(not(feature = "hydrate"))]
#[allow(dead_code)]
pub(crate) fn current_search_param(_: &str) -> Option<String> {
    None
}
