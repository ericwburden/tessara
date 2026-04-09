//! Local frontend shell for the API-first Tessara vertical slice.
//!
//! The current implementation uses Leptos SSR components for the shell
//! structure and a browser-side JavaScript controller for workflow actions.
//! That gives us a real Rust frontend layer while preserving the existing
//! user-testable API workflows during the migration.

mod app_script;
mod application;
mod brand;
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
/// The application shell now acts as the real application home: a persistent
/// navigation frame, overview screen, and entry point into the focused routes.
pub fn application_shell_html() -> String {
    application::application_shell_html(shell_style::STYLE, app_script::APPLICATION_SCRIPT)
}

/// Returns the HTML used for the organization application shell.
pub fn organization_application_shell_html() -> String {
    application::organization_application_shell_html(shell_style::STYLE, shell_script::SCRIPT)
}

/// Returns the HTML used for the forms application shell.
pub fn forms_application_shell_html() -> String {
    application::forms_application_shell_html(shell_style::STYLE, shell_script::SCRIPT)
}

/// Returns the HTML used for the responses application shell.
pub fn responses_application_shell_html() -> String {
    application::responses_application_shell_html(
        shell_style::STYLE,
        app_script::APPLICATION_SCRIPT,
    )
}

/// Returns the HTML used for the submission-focused application shell.
pub fn submission_application_shell_html() -> String {
    application::submission_application_shell_html(
        shell_style::STYLE,
        app_script::APPLICATION_SCRIPT,
    )
}

/// Returns the HTML used for the dashboards application shell.
pub fn dashboards_application_shell_html() -> String {
    application::dashboards_application_shell_html(
        shell_style::STYLE,
        app_script::APPLICATION_SCRIPT,
    )
}

/// Returns the HTML used for focused administration application screens.
pub fn administration_application_shell_html() -> String {
    application::administration_application_shell_html(shell_style::STYLE, shell_script::SCRIPT)
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

/// Returns an embedded Tessara SVG asset by public asset filename.
pub fn svg_asset(name: &str) -> Option<&'static str> {
    brand::svg_asset(name)
}

#[cfg(test)]
mod tests {
    use super::{
        admin_shell_html, administration_application_shell_html, application_shell_html,
        dashboards_application_shell_html, forms_application_shell_html,
        migration_application_shell_html, organization_application_shell_html,
        reporting_application_shell_html, submission_application_shell_html,
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
        assert!(html.contains("Remove Relationship"));
        assert!(
            html.contains(
                "/api/admin/node-type-relationships/${parentNodeTypeId}/${childNodeTypeId}"
            )
        );
        assert!(html.contains("Create Metadata Field"));
        assert!(html.contains("Update Metadata Field"));
        assert!(html.contains("metadata-field-id"));
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
        assert!(html.contains("/api/admin/reports/${reportId}"));
        assert!(html.contains("/api/admin/datasets"));
        assert!(html.contains("/api/admin/datasets/${datasetId}"));
        assert!(html.contains("Add Dataset Source"));
        assert!(html.contains("Remove Dataset Source"));
        assert!(html.contains("Add Dataset Field"));
        assert!(html.contains("Remove Dataset Field"));
        assert!(html.contains("Review Dataset Draft"));
        assert!(html.contains("Update Dataset"));
        assert!(html.contains("Remove Dataset"));
        assert!(html.contains("/api/admin/charts/${chartId}"));
        assert!(html.contains("/api/admin/dashboards/${dashboardId}"));
        assert!(html.contains("/api/admin/dashboard-components/${componentId}"));
        assert!(html.contains("/api/admin/charts"));
        assert!(html.contains("/api/charts"));
        assert!(html.contains("Load Charts"));
        assert!(html.contains("/api/admin/dashboards"));
        assert!(html.contains("Remove Report"));
        assert!(html.contains("Remove Chart"));
        assert!(html.contains("Remove Dashboard"));
        assert!(html.contains("/api/form-versions/"));
        assert!(html.contains("/api/forms/published"));
        assert!(html.contains("/api/submissions"));
        assert!(html.contains("submission-search"));
        assert!(html.contains("submission-status-filter"));
        assert!(html.contains("/api/submissions/drafts"));
        assert!(html.contains("/api/submissions/${submissionId}"));
        assert!(html.contains("Save Rendered Values"));
        assert!(html.contains("saveRenderedFormValues"));
        assert!(html.contains("Required fields missing"));
        assert!(html.contains("prefillRenderedValues"));
        assert!(html.contains("Open Response Form"));
        assert!(html.contains("renderResponseFormActions"));
        assert!(html.contains("This submitted response is read-only"));
        assert!(html.contains("Use Section"));
        assert!(html.contains("Use Field Settings"));
        assert!(html.contains("Use Report Source"));
        assert!(html.contains("Update Section"));
        assert!(html.contains("Update Field"));
        assert!(html.contains("Remove Section"));
        assert!(html.contains("Remove Field"));
        assert!(html.contains("section-position"));
        assert!(html.contains("field-position"));
        assert!(html.contains("/api/admin/form-sections/${sectionId}"));
        assert!(html.contains("/api/admin/form-fields/${fieldId}"));
        assert!(html.contains("Add Binding"));
        assert!(html.contains("report-missing-policy"));
        assert!(html.contains("Metadata required"));
        assert!(html.contains("Field required"));
        assert!(html.contains("/api/admin/node-metadata-fields/${fieldId}"));
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
        assert!(html.contains("Use Binding"));
        assert!(html.contains("Remove Selected Binding"));
        assert!(html.contains("Inspect Report By ID"));
        assert!(html.contains("Dashboard ID from seed or import output"));
        assert!(html.contains("dashboard-component-title"));
        assert!(html.contains("Hierarchy Screen"));
        assert!(html.contains("Forms Screen"));
        assert!(html.contains("Published Forms"));
        assert!(html.contains("User Testing Guide"));
        assert!(html.contains("Recommended path for the current Docker Compose test deployment."));
        assert!(html.contains("Open Application Shell"));
        assert!(html.contains("/app"));
        assert!(html.contains("Selected Context"));
        assert!(html.contains("selection-state"));
        assert!(html.contains("tessara-icon-1024.svg"));
        assert!(html.contains("tessara-favicon-32.svg"));
        assert!(html.contains("theme-color"));
        assert!(html.contains("brand-lockup"));
    }

    #[test]
    fn application_shell_exposes_home_navigation() {
        let html = application_shell_html();

        assert!(html.contains("Application Overview"));
        assert!(html.contains("Welcome to Tessara"));
        assert!(html.contains("Home"));
        assert!(html.contains("/app/organization"));
        assert!(html.contains("/app/forms"));
        assert!(html.contains("/app/responses"));
        assert!(html.contains("/app/administration"));
        assert!(html.contains("/app/reports"));
        assert!(html.contains("/app/dashboards"));
        assert!(html.contains("/app/migration"));
        assert!(html.contains("Product Areas"));
        assert!(html.contains("Internal Areas"));
        assert!(html.contains("Refresh Overview"));
        assert!(html.contains("Start Demo Response"));
        assert!(html.contains("Open Demo Dashboard"));
        assert!(html.contains("Selection Context"));
        assert!(html.contains("selection-state"));
        assert!(html.contains("tessara-icon-1024.svg"));
        assert!(html.contains("tessara-favicon-32.svg"));
        assert!(!html.contains("Create Shortcuts"));
    }

    #[test]
    fn submission_application_shell_exposes_submission_workflow_screen() {
        let html = submission_application_shell_html();

        assert!(html.contains("Responses"));
        assert!(html.contains("/app"));
        assert!(html.contains("/app/responses"));
        assert!(html.contains("Response Console"));
        assert!(html.contains("Response Queues"));
        assert!(html.contains("Guided Path"));
        assert!(html.contains("Response Stages"));
        assert!(html.contains("Response Directory"));
        assert!(html.contains("Open Response Entry"));
        assert!(html.contains("Open Target Selection"));
        assert!(html.contains("Open Response Review"));
        assert!(html.contains("Open Related Reports"));
        assert!(html.contains("Load Published Forms"));
        assert!(html.contains("Load Target Nodes"));
        assert!(html.contains("Published Forms"));
        assert!(html.contains("Target Nodes"));
        assert!(html.contains("Draft Responses"));
        assert!(html.contains("Submitted Responses"));
        assert!(html.contains("All Responses"));
        assert!(html.contains("Response Entry"));
        assert!(html.contains("Choose Published Form"));
        assert!(html.contains("Choose Target Node"));
        assert!(html.contains("Open Selected Form"));
        assert!(html.contains("Use Selected Target"));
        assert!(html.contains("Open This Form"));
        assert!(html.contains("Use Target and Continue"));
        assert!(html.contains("Create Draft"));
        assert!(html.contains("Save Values"));
        assert!(html.contains("Submit"));
        assert!(html.contains("Discard Draft"));
        assert!(html.contains("Clear Response Context"));
        assert!(html.contains("Start Demo Response"));
        assert!(html.contains("startDemoSubmissionFlow"));
        assert!(html.contains("Response Review"));
        assert!(html.contains("Show Drafts"));
        assert!(html.contains("Show Submitted"));
        assert!(html.contains("Clear Review Filters"));
        assert!(html.contains("Response Reports"));
        assert!(html.contains("Load App Summary"));
        assert!(html.contains("Current User"));
        assert!(html.contains("Log Out"));
        assert!(html.contains("/api/me"));
        assert!(html.contains("/api/app/summary"));
        assert!(html.contains("/api/forms/published"));
        assert!(html.contains("/api/submissions/drafts"));
        assert!(html.contains("DELETE"));
        assert!(html.contains("submission-status-filter"));
        assert!(html.contains("Submission search"));
        assert!(html.contains("Use Form Version"));
        assert!(html.contains("Use Node"));
        assert!(html.contains("Open Response Form"));
        assert!(html.contains("Use Report Context"));
        assert!(html.contains("Use Chart Context"));
        assert!(html.contains("Use Binding"));
        assert!(html.contains("Report Results"));
        assert!(html.contains("Report Definition"));
        assert!(html.contains("Run This Report"));
        assert!(html.contains("Refresh and Run Report"));
        assert!(html.contains("Refresh and Reopen Dashboard"));
        assert!(html.contains("table-wrap"));
        assert!(html.contains("/api/reports"));
        assert!(html.contains("sessionStorage"));
        assert!(html.contains("tessara.devToken"));
        assert!(html.contains("Selection Context"));
        assert!(html.contains("selection-state"));
        assert!(html.contains("tessara-icon-1024.svg"));
        assert!(html.contains("tessara-favicon-32.svg"));
        assert!(!html.contains("Create Shortcuts"));
    }

    #[test]
    fn organization_and_forms_shells_expose_product_area_routes() {
        let organization = organization_application_shell_html();
        let forms = forms_application_shell_html();

        assert!(organization.contains("Organization"));
        assert!(organization.contains("/app/organization"));
        assert!(organization.contains("Organization Areas"));
        assert!(organization.contains("Organization Console"));
        assert!(organization.contains("Load Nodes"));
        assert!(organization.contains("Load Node Types"));
        assert!(organization.contains("/app/forms"));
        assert!(organization.contains("/app/dashboards"));
        assert!(!organization.contains("Create Shortcuts"));

        assert!(forms.contains("Forms"));
        assert!(forms.contains("/app/forms"));
        assert!(forms.contains("Forms Areas"));
        assert!(forms.contains("Forms Console"));
        assert!(forms.contains("Load Forms"));
        assert!(forms.contains("/app/responses"));
        assert!(forms.contains("/app/organization"));
        assert!(!forms.contains("Create Shortcuts"));
    }

    #[test]
    fn admin_application_shell_exposes_setup_screens() {
        let html = administration_application_shell_html();

        assert!(html.contains("Administration"));
        assert!(html.contains("/app"));
        assert!(html.contains("/app/administration"));
        assert!(html.contains("Configuration Console"));
        assert!(html.contains("Management Queues"));
        assert!(html.contains("Admin Path"));
        assert!(html.contains("Management Areas"));
        assert!(html.contains("Entity Directory"));
        assert!(html.contains("Open Organization Setup"));
        assert!(html.contains("Open Forms Configuration"));
        assert!(html.contains("Open Reporting Configuration"));
        assert!(html.contains("Open Dashboard Configuration"));
        assert!(html.contains("Load Node Types"));
        assert!(html.contains("Load Forms"));
        assert!(html.contains("Load Datasets"));
        assert!(html.contains("Load Dashboards"));
        assert!(html.contains("Node Types"));
        assert!(html.contains("Datasets"));
        assert!(html.contains("Aggregations"));
        assert!(html.contains("Organization Setup"));
        assert!(html.contains("Forms Configuration"));
        assert!(html.contains("Reporting Configuration"));
        assert!(html.contains("Dataset name"));
        assert!(html.contains("Dataset grain"));
        assert!(html.contains("Create Dataset"));
        assert!(html.contains("Load Datasets"));
        assert!(html.contains("Inspect Dataset"));
        assert!(html.contains("Run Dataset"));
        assert!(html.contains("/api/admin/datasets"));
        assert!(html.contains("/api/admin/datasets/${datasetId}"));
        assert!(html.contains("/api/datasets"));
        assert!(html.contains("Add Dataset Source"));
        assert!(html.contains("Remove Dataset Source"));
        assert!(html.contains("Add Dataset Field"));
        assert!(html.contains("Remove Dataset Field"));
        assert!(html.contains("Review Dataset Draft"));
        assert!(html.contains("Update Dataset"));
        assert!(html.contains("Remove Dataset"));
        assert!(html.contains("Aggregation name"));
        assert!(html.contains("Create Aggregation"));
        assert!(html.contains("Load Aggregations"));
        assert!(html.contains("Inspect Aggregation"));
        assert!(html.contains("Update Aggregation"));
        assert!(html.contains("Remove Aggregation"));
        assert!(html.contains("/api/admin/aggregations"));
        assert!(html.contains("/api/admin/aggregations/${aggregationId}"));
        assert!(html.contains("/api/aggregations/${aggregationId}"));
        assert!(html.contains("/api/aggregations/${aggregationId}/table"));
        assert!(html.contains("Create Node Type"));
        assert!(html.contains("Inspect Node Type"));
        assert!(html.contains("Update Node Type"));
        assert!(html.contains("Edit Node Type"));
        assert!(html.contains("/api/admin/node-types/${nodeTypeId}"));
        assert!(html.contains("Use Form Scope"));
        assert!(html.contains("Use Compatibility Group"));
        assert!(html.contains("Use Metadata Target"));
        assert!(html.contains("Use Node Type As Form Scope"));
        assert!(html.contains("Use Node Type As Metadata Target"));
        assert!(html.contains("Remove Relationship"));
        assert!(html.contains("Update Metadata Field"));
        assert!(html.contains("Update Node"));
        assert!(html.contains("Choose Node To Edit"));
        assert!(html.contains("Edit Node"));
        assert!(html.contains("Create Form"));
        assert!(html.contains("Update Form"));
        assert!(html.contains("Inspect Form"));
        assert!(html.contains("Edit Form"));
        assert!(html.contains("/api/admin/forms/${formId}"));
        assert!(html.contains("Create Basic Version"));
        assert!(html.contains("Remove Section"));
        assert!(html.contains("Update Field"));
        assert!(html.contains("Remove Field"));
        assert!(html.contains("Section position"));
        assert!(html.contains("Field position"));
        assert!(html.contains("Publish Version"));
        assert!(html.contains("Publish and Preview Version"));
        assert!(html.contains("Add Binding"));
        assert!(html.contains("Report computed expression"));
        assert!(html.contains("Update Report"));
        assert!(html.contains("Remove Report"));
        assert!(html.contains("Update Chart"));
        assert!(html.contains("Remove Chart"));
        assert!(html.contains("Update Dashboard"));
        assert!(html.contains("Remove Dashboard"));
        assert!(html.contains("Update Component"));
        assert!(html.contains("Remove Component"));
        assert!(html.contains("Dashboard component title"));
        assert!(html.contains("Use Report Context"));
        assert!(html.contains("Use Chart Context"));
        assert!(html.contains("Use Component Context"));
        assert!(html.contains("Use Binding"));
        assert!(html.contains("Remove Binding"));
        assert!(html.contains("Report Results"));
        assert!(html.contains("Report Definition"));
        assert!(html.contains("Run This Report"));
        assert!(html.contains("Refresh and Run Report"));
        assert!(html.contains("Refresh and Reopen Dashboard"));
        assert!(html.contains("Refresh and Open Dashboard"));
        assert!(html.contains("Product Areas"));
        assert!(html.contains("Create Shortcuts"));
        assert!(html.contains("Load App Summary"));
        assert!(html.contains("selection-state"));
    }

    #[test]
    fn migration_application_shell_exposes_fixture_workflow() {
        let html = migration_application_shell_html();

        assert!(html.contains("Migration Workbench"));
        assert!(html.contains("/app"));
        assert!(html.contains("/app/responses"));
        assert!(html.contains("Migration Stages"));
        assert!(html.contains("Migration Directory"));
        assert!(html.contains("Open Fixture Intake"));
        assert!(html.contains("Open Validation"));
        assert!(html.contains("Open Dry Run"));
        assert!(html.contains("Open Import Results"));
        assert!(html.contains("Fixture Examples"));
        assert!(html.contains("Validation Results"));
        assert!(html.contains("Dry Runs"));
        assert!(html.contains("Imports"));
        assert!(html.contains("Fixture Intake and Validation"));
        assert!(html.contains("Load Fixture Examples"));
        assert!(html.contains("Validate Fixture"));
        assert!(html.contains("Dry-Run Fixture"));
        assert!(html.contains("Import Fixture"));
        assert!(html.contains("/api/admin/legacy-fixtures/examples"));
        assert!(html.contains("/api/admin/legacy-fixtures/validate"));
        assert!(html.contains("/api/admin/legacy-fixtures/dry-run"));
        assert!(html.contains("/api/admin/legacy-fixtures/import"));
        assert!(html.contains("Product Areas"));
        assert!(html.contains("Load App Summary"));
        assert!(html.contains("Current User"));
        assert!(html.contains("Log Out"));
        assert!(!html.contains("Create Shortcuts"));
    }

    #[test]
    fn reporting_application_shell_exposes_report_dashboard_workflow() {
        let html = reporting_application_shell_html();

        assert!(html.contains("Reports"));
        assert!(html.contains("/app"));
        assert!(html.contains("/app/reports"));
        assert!(html.contains("/app/dashboards"));
        assert!(html.contains("Insight Console"));
        assert!(html.contains("Reporting Queues"));
        assert!(html.contains("Reporting Path"));
        assert!(html.contains("Report Areas"));
        assert!(html.contains("Reporting Directory"));
        assert!(html.contains("Open Dataset Workflows"));
        assert!(html.contains("Open Report Runner"));
        assert!(html.contains("Open Aggregations"));
        assert!(html.contains("Open Dashboard Preview"));
        assert!(html.contains("Load Datasets"));
        assert!(html.contains("Load Reports"));
        assert!(html.contains("Load Aggregations"));
        assert!(html.contains("Load Dashboards"));
        assert!(html.contains("Datasets"));
        assert!(html.contains("Aggregations"));
        assert!(html.contains("Charts"));
        assert!(html.contains("Report Runner"));
        assert!(html.contains("Dashboard Preview"));
        assert!(html.contains("Open Demo Dashboard"));
        assert!(html.contains("openDemoDashboard"));
        assert!(html.contains("Refresh Analytics"));
        assert!(html.contains("Choose Dataset"));
        assert!(html.contains("Inspect Dataset"));
        assert!(html.contains("Run Dataset"));
        assert!(html.contains("Choose Report"));
        assert!(html.contains("Inspect Report"));
        assert!(html.contains("Run Report"));
        assert!(html.contains("Refresh and Run Report"));
        assert!(html.contains("Choose Aggregation"));
        assert!(html.contains("Inspect Aggregation"));
        assert!(html.contains("Run Aggregation"));
        assert!(html.contains("Choose Dashboard"));
        assert!(html.contains("Open Dashboard"));
        assert!(html.contains("Refresh and Open Dashboard"));
        assert!(html.contains("Choose Chart"));
        assert!(html.contains("Inspect Chart"));
        assert!(html.contains("Load App Summary"));
        assert!(html.contains("Current User"));
        assert!(html.contains("Log Out"));
        assert!(html.contains("/api/admin/analytics/refresh"));
        assert!(html.contains("/api/app/summary"));
        assert!(html.contains("/api/datasets"));
        assert!(html.contains("/api/datasets/${datasetId}"));
        assert!(html.contains("/api/datasets/${datasetId}/table"));
        assert!(html.contains("Dataset Definition"));
        assert!(html.contains("Dataset Results"));
        assert!(html.contains("/api/reports"));
        assert!(html.contains("/api/dashboards"));
        assert!(html.contains("/api/charts"));
        assert!(html.contains("/api/charts/${chartId}"));
        assert!(html.contains("Chart Definition"));
        assert!(html.contains("Open Linked Report"));
        assert!(html.contains("Open Linked Aggregation"));
        assert!(html.contains("Product Areas"));
        assert!(!html.contains("Create Shortcuts"));
    }

    #[test]
    fn dashboard_application_shell_exposes_dashboard_workflow() {
        let html = dashboards_application_shell_html();

        assert!(html.contains("Dashboards"));
        assert!(html.contains("/app/dashboards"));
        assert!(html.contains("Dashboard Areas"));
        assert!(html.contains("Dashboard Console"));
        assert!(html.contains("Dashboard Preview"));
        assert!(html.contains("Open Demo Dashboard"));
        assert!(html.contains("/app/reports"));
        assert!(!html.contains("Create Shortcuts"));
    }
}
