//! Local frontend shell for the API-first Tessara vertical slice.
//!
//! The current implementation uses Leptos SSR components for the shell
//! structure and a browser-side JavaScript controller for workflow actions.
//! That gives us a real Rust frontend layer while preserving the existing
//! user-testable API workflows during the migration.

mod app_script;
mod application;
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

/// Returns the HTML used for the first replacement-oriented application shell.
///
/// The application shell keeps the existing JavaScript controller while the
/// Leptos screen structure stabilizes. It is intentionally separate from the
/// admin workbench so user testing can exercise application workflows without
/// navigating the full builder surface.
pub fn application_shell_html() -> String {
    application::application_shell_html(shell_style::STYLE, app_script::APPLICATION_SCRIPT)
}

/// Returns the HTML used for focused admin application screens.
pub fn admin_application_shell_html() -> String {
    application::admin_application_shell_html(shell_style::STYLE, shell_script::SCRIPT)
}

/// Returns the HTML used for focused migration application screens.
pub fn migration_application_shell_html() -> String {
    application::migration_application_shell_html(
        shell_style::STYLE,
        app_script::APPLICATION_SCRIPT,
    )
}

/// Returns the HTML used for focused reporting application screens.
pub fn reporting_application_shell_html() -> String {
    application::reporting_application_shell_html(
        shell_style::STYLE,
        app_script::APPLICATION_SCRIPT,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        admin_application_shell_html, admin_shell_html, application_shell_html,
        migration_application_shell_html, reporting_application_shell_html,
    };

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
        assert!(html.contains("Open Application Shell"));
        assert!(html.contains("/app"));
        assert!(html.contains("Selected Context"));
        assert!(html.contains("selection-state"));
    }

    #[test]
    fn application_shell_exposes_submission_workflow_screen() {
        let html = application_shell_html();

        assert!(html.contains("Submission Workspace"));
        assert!(html.contains("Submit Data"));
        assert!(html.contains("Choose Published Form"));
        assert!(html.contains("Choose Target Node"));
        assert!(html.contains("Create Draft"));
        assert!(html.contains("Save Values"));
        assert!(html.contains("Submit"));
        assert!(html.contains("Start Demo Submission"));
        assert!(html.contains("startDemoSubmissionFlow"));
        assert!(html.contains("Review Submissions"));
        assert!(html.contains("View Reports"));
        assert!(html.contains("Open Admin Workbench"));
        assert!(html.contains("Open Admin Setup"));
        assert!(html.contains("Open Migration Workbench"));
        assert!(html.contains("Open Reporting Workspace"));
        assert!(html.contains("Load App Summary"));
        assert!(html.contains("/api/app/summary"));
        assert!(html.contains("/api/forms/published"));
        assert!(html.contains("/api/submissions/drafts"));
        assert!(html.contains("/api/reports"));
        assert!(html.contains("sessionStorage"));
        assert!(html.contains("tessara.devToken"));
        assert!(html.contains("selection-state"));
    }

    #[test]
    fn admin_application_shell_exposes_setup_screens() {
        let html = admin_application_shell_html();

        assert!(html.contains("Setup Workspace"));
        assert!(html.contains("Hierarchy Setup"));
        assert!(html.contains("Form Builder"));
        assert!(html.contains("Report Builder"));
        assert!(html.contains("Create Node Type"));
        assert!(html.contains("Update Node"));
        assert!(html.contains("Create Form"));
        assert!(html.contains("Update Field"));
        assert!(html.contains("Publish Version"));
        assert!(html.contains("Add Binding"));
        assert!(html.contains("Open Submission Workspace"));
        assert!(html.contains("Open Migration Workbench"));
        assert!(html.contains("Open Reporting Workspace"));
        assert!(html.contains("Load App Summary"));
        assert!(html.contains("selection-state"));
    }

    #[test]
    fn migration_application_shell_exposes_fixture_workflow() {
        let html = migration_application_shell_html();

        assert!(html.contains("Migration Workbench"));
        assert!(html.contains("Legacy Fixture Validation"));
        assert!(html.contains("Load Fixture Examples"));
        assert!(html.contains("Validate Fixture"));
        assert!(html.contains("Dry-Run Fixture"));
        assert!(html.contains("Import Fixture"));
        assert!(html.contains("/api/admin/legacy-fixtures/examples"));
        assert!(html.contains("/api/admin/legacy-fixtures/validate"));
        assert!(html.contains("/api/admin/legacy-fixtures/dry-run"));
        assert!(html.contains("/api/admin/legacy-fixtures/import"));
        assert!(html.contains("Open Submission Workspace"));
        assert!(html.contains("Open Admin Setup"));
        assert!(html.contains("Open Reporting Workspace"));
        assert!(html.contains("Load App Summary"));
    }

    #[test]
    fn reporting_application_shell_exposes_report_dashboard_workflow() {
        let html = reporting_application_shell_html();

        assert!(html.contains("Reporting Workspace"));
        assert!(html.contains("Report Runner"));
        assert!(html.contains("Dashboard Preview"));
        assert!(html.contains("Open Demo Dashboard"));
        assert!(html.contains("openDemoDashboard"));
        assert!(html.contains("Refresh Analytics"));
        assert!(html.contains("Choose Report"));
        assert!(html.contains("Inspect Report"));
        assert!(html.contains("Run Report"));
        assert!(html.contains("Choose Dashboard"));
        assert!(html.contains("Open Dashboard"));
        assert!(html.contains("Choose Chart"));
        assert!(html.contains("Load App Summary"));
        assert!(html.contains("/api/admin/analytics/refresh"));
        assert!(html.contains("/api/app/summary"));
        assert!(html.contains("/api/reports"));
        assert!(html.contains("/api/dashboards"));
        assert!(html.contains("/api/charts"));
        assert!(html.contains("Open Submission Workspace"));
        assert!(html.contains("Open Admin Setup"));
        assert!(html.contains("Open Migration Workbench"));
    }
}
