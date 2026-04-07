//! Local frontend shell for the API-first Tessara vertical slice.
//!
//! The current implementation uses Leptos SSR components for the shell
//! structure and a browser-side JavaScript controller for workflow actions.
//! That gives us a real Rust frontend layer while preserving the existing
//! user-testable API workflows during the migration.

mod shell;
mod shell_model;
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
        assert!(html.contains("node-search"));
        assert!(html.contains("/api/admin/node-types"));
        assert!(html.contains("/api/admin/node-type-relationships"));
        assert!(html.contains("/api/admin/node-metadata-fields"));
        assert!(html.contains("/api/admin/nodes"));
        assert!(html.contains("/api/admin/nodes/${nodeId}"));
        assert!(html.contains("Update Node"));
        assert!(html.contains("/api/admin/forms"));
        assert!(html.contains("Create Node Type"));
        assert!(html.contains("Create Relationship"));
        assert!(html.contains("Create Metadata Field"));
        assert!(html.contains("Create Node"));
        assert!(html.contains("node-metadata-json"));
        assert!(html.contains("Create Form"));
        assert!(html.contains("Create Version"));
        assert!(html.contains("Publish Version"));
        assert!(html.contains("Create Report"));
        assert!(html.contains("Create Chart"));
        assert!(html.contains("Add Component"));
        assert!(html.contains("/api/admin/form-versions/"));
        assert!(html.contains("/api/admin/reports"));
        assert!(html.contains("/api/admin/charts"));
        assert!(html.contains("/api/charts"));
        assert!(html.contains("Load Charts"));
        assert!(html.contains("/api/admin/dashboards"));
        assert!(html.contains("/api/form-versions/"));
        assert!(html.contains("/api/forms/published"));
        assert!(html.contains("/api/submissions"));
        assert!(html.contains("/api/submissions/drafts"));
        assert!(html.contains("/api/submissions/${submissionId}"));
        assert!(html.contains("Save Rendered Values"));
        assert!(html.contains("saveRenderedFormValues"));
        assert!(html.contains("Required fields missing"));
        assert!(html.contains("Use Section"));
        assert!(html.contains("Use Field Settings"));
        assert!(html.contains("Use Report Source"));
        assert!(html.contains("Update Section"));
        assert!(html.contains("Update Field"));
        assert!(html.contains("/api/admin/form-fields/${fieldId}"));
        assert!(html.contains("Add Binding"));
        assert!(html.contains("report-missing-policy"));
        assert!(html.contains("Metadata required"));
        assert!(html.contains("Field required"));
        assert!(html.contains("Load Submission By ID"));
        assert!(html.contains("/api/admin/analytics/refresh"));
        assert!(html.contains("/api/admin/legacy-fixtures/validate"));
        assert!(html.contains("/api/admin/legacy-fixtures/dry-run"));
        assert!(html.contains("/api/admin/legacy-fixtures/examples"));
        assert!(html.contains("Load Fixture Examples"));
        assert!(html.contains("Validate Legacy Fixture"));
        assert!(html.contains("Dry-Run Legacy Fixture"));
        assert!(html.contains("/api/dashboards/"));
        assert!(html.contains("/api/dashboards"));
        assert!(html.contains("/api/reports/"));
        assert!(html.contains("/api/reports/${component.chart.report_id}/table"));
        assert!(html.contains("/api/reports"));
        assert!(html.contains("report-fields-json"));
        assert!(html.contains("Inspect Report By ID"));
        assert!(html.contains("Dashboard ID from seed or import output"));
        assert!(html.contains("Hierarchy Screen"));
        assert!(html.contains("Forms Screen"));
        assert!(html.contains("Published Forms"));
        assert!(html.contains("User Testing Guide"));
        assert!(html.contains("Recommended path for the current Docker Compose test deployment."));
        assert!(html.contains("Selected Context"));
        assert!(html.contains("selection-state"));
    }
}
