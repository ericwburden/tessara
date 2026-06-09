//! API endpoint helpers for the frontend API client layer.
//!
//! This module intentionally keeps endpoint path construction colocated so feature
//! modules can share a stable boundary for URL construction.

/// Common API namespace for all HTTP calls made from the frontend.
pub(crate) const API_PREFIX: &str = "/api";

/// Construct a normalized endpoint path under the configured API prefix.
pub(crate) fn endpoint(path: &str) -> String {
    match path {
        "" => API_PREFIX.to_string(),
        _ => format!("{API_PREFIX}/{}", trim_slash(path)),
    }
}

fn trim_slash(path: &str) -> &str {
    path.trim_start_matches('/')
}

/// Construct an API endpoint from an identifier and optional nested segments.
pub(crate) fn endpoint_with_id(base: &str, id: &str, action: Option<&str>) -> String {
    match action {
        Some(action) if !action.trim().is_empty() => {
            endpoint(&format!("{base}/{id}/{action}", base = trim_slash(base)))
        }
        _ => endpoint(&format!("{base}/{id}", base = trim_slash(base))),
    }
}
