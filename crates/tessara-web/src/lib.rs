//! Local frontend shell for the API-first Tessara vertical slice.
//!
//! The current implementation is intentionally dependency-light: it serves a
//! structured HTML shell with colocated CSS and JavaScript modules. That keeps
//! the browser workflows maintainable while the team decides when to introduce
//! the full Leptos application layer.

mod shell;
mod shell_script;
mod shell_style;

/// Returns the HTML used for the current local admin shell.
///
/// The shell exercises the same API endpoints as the smoke test: development
/// login, demo seeding, hierarchy/form builder screens, submission workflow,
/// report execution, and dashboard inspection.
pub fn admin_shell_html() -> String {
    shell::admin_shell_html(shell_style::STYLE, shell_script::SCRIPT)
}

#[cfg(test)]
mod tests {
    use super::admin_shell_html;

    #[test]
    fn shell_links_to_current_demo_api_contract() {
        let html = admin_shell_html();

        assert!(html.contains("/api/auth/login"));
        assert!(html.contains("/api/demo/seed"));
        assert!(html.contains("/api/nodes"));
        assert!(html.contains("/api/admin/node-types"));
        assert!(html.contains("/api/admin/node-type-relationships"));
        assert!(html.contains("/api/admin/node-metadata-fields"));
        assert!(html.contains("/api/admin/forms"));
        assert!(html.contains("Create Node Type"));
        assert!(html.contains("Create Relationship"));
        assert!(html.contains("Create Metadata Field"));
        assert!(html.contains("Create Form"));
        assert!(html.contains("Create Version"));
        assert!(html.contains("Publish Version"));
        assert!(html.contains("Create Report"));
        assert!(html.contains("Create Chart"));
        assert!(html.contains("Add Component"));
        assert!(html.contains("/api/admin/form-versions/"));
        assert!(html.contains("/api/admin/reports"));
        assert!(html.contains("/api/admin/charts"));
        assert!(html.contains("/api/admin/dashboards"));
        assert!(html.contains("/api/form-versions/"));
        assert!(html.contains("/api/submissions"));
        assert!(html.contains("/api/submissions/drafts"));
        assert!(html.contains("/api/admin/analytics/refresh"));
        assert!(html.contains("/api/dashboards/"));
        assert!(html.contains("/api/dashboards"));
        assert!(html.contains("/api/reports/"));
        assert!(html.contains("/api/reports"));
        assert!(html.contains("Dashboard ID from seed or import output"));
        assert!(html.contains("Hierarchy Screen"));
        assert!(html.contains("Forms Screen"));
    }
}
