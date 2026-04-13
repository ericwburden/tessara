//! Local frontend shell for the API-first Tessara vertical slice.

pub mod app;
mod app_script;
mod application;
mod brand;
pub mod features;
pub mod infra;
mod pipeline;
mod shell;
mod shell_model;
mod shell_script;
mod theme;

pub fn admin_shell_html() -> String {
    shell::admin_shell_html(shell_script::SCRIPT)
}

pub fn bridge_asset(name: &str) -> Option<&'static str> {
    match name {
        "app-legacy.js" => Some(app_script::APPLICATION_SCRIPT),
        "admin-legacy.js" => Some(shell_script::SCRIPT),
        _ => None,
    }
}

pub fn css_path() -> String {
    pipeline::css_path()
}

pub fn js_path() -> String {
    pipeline::js_path()
}

pub fn pkg_dir() -> std::path::PathBuf {
    pipeline::pkg_dir()
}

pub fn application_shell_html() -> String {
    application::application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn login_application_html() -> String {
    application::login_application_html(app_script::APPLICATION_SCRIPT)
}

pub fn organization_application_shell_html() -> String {
    application::organization_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn organization_create_application_html() -> String {
    application::organization_create_application_html(app_script::APPLICATION_SCRIPT)
}

pub fn organization_detail_application_html(node_id: &str) -> String {
    application::organization_detail_application_html(app_script::APPLICATION_SCRIPT, node_id)
}

pub fn organization_edit_application_html(node_id: &str) -> String {
    application::organization_edit_application_html(app_script::APPLICATION_SCRIPT, node_id)
}

pub fn forms_application_shell_html() -> String {
    application::forms_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn form_create_application_html() -> String {
    application::form_create_application_html(app_script::APPLICATION_SCRIPT)
}

pub fn form_detail_application_html(form_id: &str) -> String {
    application::form_detail_application_html(app_script::APPLICATION_SCRIPT, form_id)
}

pub fn form_edit_application_html(form_id: &str) -> String {
    application::form_edit_application_html(app_script::APPLICATION_SCRIPT, form_id)
}

pub fn responses_application_shell_html() -> String {
    application::responses_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn submission_application_shell_html() -> String {
    application::submission_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn response_create_application_html() -> String {
    application::response_create_application_html(app_script::APPLICATION_SCRIPT)
}

pub fn response_detail_application_html(submission_id: &str) -> String {
    application::response_detail_application_html(app_script::APPLICATION_SCRIPT, submission_id)
}

pub fn response_edit_application_html(submission_id: &str) -> String {
    application::response_edit_application_html(app_script::APPLICATION_SCRIPT, submission_id)
}

pub fn dashboards_application_shell_html() -> String {
    application::dashboards_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn dashboard_create_application_html() -> String {
    application::dashboard_create_application_html(app_script::APPLICATION_SCRIPT)
}

pub fn dashboard_detail_application_html(dashboard_id: &str) -> String {
    application::dashboard_detail_application_html(app_script::APPLICATION_SCRIPT, dashboard_id)
}

pub fn dashboard_edit_application_html(dashboard_id: &str) -> String {
    application::dashboard_edit_application_html(app_script::APPLICATION_SCRIPT, dashboard_id)
}

pub fn administration_application_shell_html() -> String {
    application::administration_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn users_application_shell_html() -> String {
    application::users_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn user_create_application_html() -> String {
    application::user_create_application_html(app_script::APPLICATION_SCRIPT)
}

pub fn user_detail_application_html(account_id: &str) -> String {
    application::user_detail_application_html(app_script::APPLICATION_SCRIPT, account_id)
}

pub fn user_edit_application_html(account_id: &str) -> String {
    application::user_edit_application_html(app_script::APPLICATION_SCRIPT, account_id)
}

pub fn user_access_application_html(account_id: &str) -> String {
    application::user_access_application_html(app_script::APPLICATION_SCRIPT, account_id)
}

pub fn roles_application_shell_html() -> String {
    application::roles_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn role_detail_application_html(role_id: &str) -> String {
    application::role_detail_application_html(app_script::APPLICATION_SCRIPT, role_id)
}

pub fn role_edit_application_html(role_id: &str) -> String {
    application::role_edit_application_html(app_script::APPLICATION_SCRIPT, role_id)
}

pub fn admin_application_shell_html() -> String {
    shell::admin_shell_html(shell_script::SCRIPT)
}

pub fn migration_application_shell_html() -> String {
    application::migration_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn reporting_application_shell_html() -> String {
    application::reporting_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn report_create_application_html() -> String {
    application::report_create_application_html(app_script::APPLICATION_SCRIPT)
}

pub fn report_detail_application_html(report_id: &str) -> String {
    application::report_detail_application_html(app_script::APPLICATION_SCRIPT, report_id)
}

pub fn report_edit_application_html(report_id: &str) -> String {
    application::report_edit_application_html(app_script::APPLICATION_SCRIPT, report_id)
}

pub fn svg_asset(name: &str) -> Option<&'static str> {
    brand::svg_asset(name)
}

#[cfg(test)]
mod tests {
    use super::{
        admin_shell_html, administration_application_shell_html, application_shell_html,
        bridge_asset, dashboard_create_application_html, dashboard_detail_application_html,
        dashboard_edit_application_html, dashboards_application_shell_html,
        form_create_application_html, form_detail_application_html, form_edit_application_html,
        forms_application_shell_html, login_application_html, migration_application_shell_html,
        organization_application_shell_html, organization_create_application_html,
        organization_detail_application_html, organization_edit_application_html,
        report_create_application_html, report_detail_application_html,
        report_edit_application_html, reporting_application_shell_html,
        response_create_application_html, response_detail_application_html,
        response_edit_application_html, role_detail_application_html, role_edit_application_html,
        roles_application_shell_html, submission_application_shell_html,
        user_access_application_html, user_create_application_html, user_detail_application_html,
        user_edit_application_html, users_application_shell_html,
    };

    #[test]
    fn admin_shell_still_exposes_legacy_builder_contract() {
        let html = admin_shell_html();
        let bridge = bridge_asset("admin-legacy.js").expect("admin bridge should exist");

        assert!(html.contains("/pkg/tessara-web.css"));
        assert!(html.contains("data-theme=\"light\""));
        assert!(html.contains("theme-toggle"));
        assert!(html.contains("/bridge/admin-legacy.js"));
        assert!(html.contains("/pkg/tessara-web.js"));
        assert!(bridge.contains("/api/auth/login"));
        assert!(bridge.contains("/api/demo/seed"));
        assert!(bridge.contains("/api/admin/node-types"));
        assert!(bridge.contains("/api/admin/forms"));
        assert!(bridge.contains("/api/admin/reports"));
        assert!(bridge.contains("/api/admin/dashboards"));
        assert!(html.contains("Open Application Shell"));
    }

    #[test]
    fn home_shell_exposes_shared_navigation() {
        let html = application_shell_html();

        assert!(html.contains("Application Overview"));
        assert!(html.contains("Welcome to Tessara"));
        assert!(html.contains("Role-Ready Home Modules"));
        assert!(html.contains("Product Areas"));
        assert!(html.contains("Current Deployment Readiness"));
        assert!(html.contains("Current Workflow Context"));
        assert!(html.contains("Internal Areas"));
        assert!(html.contains("/app/organization"));
        assert!(html.contains("/app/forms"));
        assert!(html.contains("/app/responses"));
        assert!(html.contains("/app/reports"));
        assert!(html.contains("/app/dashboards"));
        assert!(html.contains("/app/administration"));
        assert!(html.contains("/app/migration"));
        assert!(!html.contains("Create Shortcuts"));
    }

    #[test]
    fn login_shell_exposes_credentials_form() {
        let html = login_application_html();

        assert!(html.contains("Sign In"));
        assert!(html.contains("/pkg/tessara-web.css"));
        assert!(html.contains("/bridge/app-legacy.js"));
        assert!(html.contains("login-form"));
        assert!(html.contains("login-feedback"));
        assert!(html.contains("login-email"));
        assert!(html.contains("login-password"));
        assert!(html.contains("operator@tessara.local"));
        assert!(html.contains("data-theme-choice=\"system\""));
    }

    #[test]
    fn product_list_pages_expose_dedicated_list_screens() {
        let organization = organization_application_shell_html();
        let forms = forms_application_shell_html();
        let responses = submission_application_shell_html();
        let reports = reporting_application_shell_html();
        let dashboards = dashboards_application_shell_html();

        assert!(organization.contains("Organizations"));
        assert!(organization.contains("Create Organization"));
        assert!(organization.contains("organization-list"));
        assert!(!organization.contains("Node ID"));

        assert!(forms.contains("Forms"));
        assert!(forms.contains("Create Form"));
        assert!(forms.contains("form-list"));
        assert!(!forms.contains("Form ID"));

        assert!(responses.contains("Responses"));
        assert!(responses.contains("Start Response"));
        assert!(responses.contains("Start New Response"));
        assert!(responses.contains("Draft Responses"));
        assert!(responses.contains("Submitted Responses"));
        assert!(!responses.contains("Draft submission ID"));

        assert!(reports.contains("Reports"));
        assert!(reports.contains("Create Report"));
        assert!(reports.contains("report-list"));

        assert!(dashboards.contains("Dashboards"));
        assert!(dashboards.contains("Create Dashboard"));
        assert!(dashboards.contains("dashboard-list"));
    }

    #[test]
    fn create_edit_and_detail_pages_are_dedicated() {
        let organization_new = organization_create_application_html();
        let organization_detail =
            organization_detail_application_html("00000000-0000-0000-0000-000000000001");
        let organization_edit =
            organization_edit_application_html("00000000-0000-0000-0000-000000000001");
        let form_new = form_create_application_html();
        let form_detail = form_detail_application_html("00000000-0000-0000-0000-000000000002");
        let form_edit = form_edit_application_html("00000000-0000-0000-0000-000000000002");
        let response_new = response_create_application_html();
        let response_detail =
            response_detail_application_html("00000000-0000-0000-0000-000000000003");
        let response_edit = response_edit_application_html("00000000-0000-0000-0000-000000000003");
        let report_new = report_create_application_html();
        let report_detail = report_detail_application_html("00000000-0000-0000-0000-000000000004");
        let report_edit = report_edit_application_html("00000000-0000-0000-0000-000000000004");
        let dashboard_new = dashboard_create_application_html();
        let dashboard_detail =
            dashboard_detail_application_html("00000000-0000-0000-0000-000000000005");
        let dashboard_edit =
            dashboard_edit_application_html("00000000-0000-0000-0000-000000000005");

        for html in [
            organization_new.as_str(),
            organization_edit.as_str(),
            form_new.as_str(),
            form_edit.as_str(),
            response_new.as_str(),
            response_edit.as_str(),
            report_new.as_str(),
            report_edit.as_str(),
            dashboard_new.as_str(),
            dashboard_edit.as_str(),
        ] {
            assert!(html.contains("Submit"));
            assert!(html.contains("Cancel"));
            assert!(!html.contains(" ID"));
        }

        assert!(organization_detail.contains("Organization Detail"));
        assert!(organization_detail.contains("Back to List"));
        assert!(form_detail.contains("Form Detail"));
        assert!(response_detail.contains("Response Detail"));
        assert!(report_detail.contains("Report Detail"));
        assert!(report_detail.contains("Run"));
        assert!(dashboard_detail.contains("Dashboard Detail"));
        assert!(dashboard_detail.contains("View"));
    }

    #[test]
    fn administration_and_migration_stay_internal() {
        let administration = administration_application_shell_html();
        let users = users_application_shell_html();
        let roles = roles_application_shell_html();
        let migration = migration_application_shell_html();

        assert!(administration.contains("Administration"));
        assert!(administration.contains("Advanced Configuration"));
        assert!(administration.contains("User Accounts"));
        assert!(administration.contains("/app/administration/users"));
        assert!(administration.contains("Role Catalog"));
        assert!(administration.contains("/app/administration/roles"));
        assert!(administration.contains("Open Legacy Builder"));
        assert!(administration.contains("/app/admin"));

        assert!(users.contains("User Management"));
        assert!(users.contains("Create User"));
        assert!(users.contains("user-list"));

        assert!(roles.contains("Roles"));
        assert!(roles.contains("role-list"));

        assert!(migration.contains("Migration Workbench"));
        assert!(migration.contains("Load Fixture Examples"));
        assert!(migration.contains("Validate Fixture"));
        assert!(migration.contains("Dry-Run Fixture"));
        assert!(migration.contains("Import Fixture"));
    }

    #[test]
    fn user_management_pages_are_dedicated() {
        let create = user_create_application_html();
        let detail = user_detail_application_html("00000000-0000-0000-0000-000000000006");
        let edit = user_edit_application_html("00000000-0000-0000-0000-000000000006");
        let access = user_access_application_html("00000000-0000-0000-0000-000000000006");
        let role_detail = role_detail_application_html("00000000-0000-0000-0000-000000000007");
        let role_edit = role_edit_application_html("00000000-0000-0000-0000-000000000007");

        assert!(create.contains("Create User"));
        assert!(create.contains("user-form"));
        assert!(create.contains("user-role-options"));
        assert!(create.contains("Account is active"));

        assert!(detail.contains("User Detail"));
        assert!(detail.contains("Back to List"));

        assert!(edit.contains("Edit User"));
        assert!(edit.contains("Leave blank to keep the current password."));
        assert!(edit.contains("Submit"));
        assert!(edit.contains("Cancel"));

        assert!(access.contains("User Access"));
        assert!(access.contains("user-access-form"));
        assert!(access.contains("user-scope-options"));

        assert!(role_detail.contains("Role Detail"));
        assert!(role_detail.contains("Back to List"));

        assert!(role_edit.contains("Edit Role"));
        assert!(role_edit.contains("role-capability-options"));
        assert!(role_edit.contains("Submit"));
    }
}
