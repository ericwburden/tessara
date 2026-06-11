//! Owns the features::shared::display module behavior.

pub(crate) fn status_badge_class(status: &str) -> &'static str {
    match status {
        "published" | "done" | "active" | "submitted" => "status-badge is-success",
        "draft" | "in_progress" => "status-badge is-warning",
        "error" | "archived" => "status-badge is-danger",
        _ => "status-badge is-info",
    }
}
