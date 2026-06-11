//! Cross-feature display formatting helpers.
//!
//! Keep status class and label helpers here when the same visual mapping is shared by several feature domains.

pub(crate) fn status_badge_class(status: &str) -> &'static str {
    match status {
        "published" | "done" | "active" | "submitted" => "status-badge is-success",
        "draft" | "in_progress" => "status-badge is-warning",
        "error" | "archived" => "status-badge is-danger",
        _ => "status-badge is-info",
    }
}
