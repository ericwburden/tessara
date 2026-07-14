//! Dashboard feature content. Root `tessara-web` continues to own route and shell policy.

mod create;
mod detail;
mod directory;
mod editor;
mod viewer;
mod visibility_scope;

pub use create::DashboardCreateContent;
pub use detail::DashboardDetailContent;
pub use directory::DashboardsIndexContent;
pub use editor::DashboardEditorContent;
pub use viewer::{DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS, DashboardViewerContent};

fn version_label(version_number: i32, version_label: &str) -> String {
    if version_label.trim().is_empty() {
        format!("v{version_number}")
    } else {
        format!("v{version_number} · {version_label}")
    }
}
