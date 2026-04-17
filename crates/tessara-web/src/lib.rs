#![recursion_limit = "512"]

//! Local frontend shell for the API-first Tessara vertical slice.

pub mod app;
mod app_script;
mod application;
mod brand;
mod document;
pub mod features;
pub mod infra;
mod pipeline;
mod shell;
mod shell_model;
mod shell_script;
mod theme;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "hydrate")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    app::hydrate_app(pipeline::APP_ROOT_ID);
}

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
    document::render_native_app_document(
        "Tessara Home",
        "Tessara application home for local replacement workflow testing.",
        "/app",
    )
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
    document::render_native_app_document(
        "Tessara Forms",
        "Tessara forms list screen.",
        "/app/forms",
    )
}

pub fn form_create_application_html() -> String {
    document::render_native_app_document("Create Form", "Create a Tessara form.", "/app/forms/new")
}

pub fn form_detail_application_html(form_id: &str) -> String {
    document::render_native_app_document(
        "Form Detail",
        "Inspect a Tessara form.",
        &format!("/app/forms/{form_id}"),
    )
}

pub fn form_edit_application_html(form_id: &str) -> String {
    document::render_native_app_document(
        "Edit Form",
        "Edit a Tessara form.",
        &format!("/app/forms/{form_id}/edit"),
    )
}

pub fn workflows_application_shell_html() -> String {
    document::render_native_app_document(
        "Tessara Workflows",
        "Tessara workflows list screen.",
        "/app/workflows",
    )
}

pub fn workflow_create_application_html() -> String {
    document::render_native_app_document(
        "Create Workflow",
        "Create a Tessara workflow.",
        "/app/workflows/new",
    )
}

pub fn workflow_detail_application_html(workflow_id: &str) -> String {
    document::render_native_app_document(
        "Workflow Detail",
        "Inspect a Tessara workflow.",
        &format!("/app/workflows/{workflow_id}"),
    )
}

pub fn workflow_edit_application_html(workflow_id: &str) -> String {
    document::render_native_app_document(
        "Edit Workflow",
        "Edit a Tessara workflow.",
        &format!("/app/workflows/{workflow_id}/edit"),
    )
}

pub fn workflow_assignments_application_html() -> String {
    document::render_native_app_document(
        "Workflow Assignments",
        "Workflow assignment console.",
        "/app/workflows/assignments",
    )
}

pub fn responses_application_shell_html() -> String {
    document::render_native_app_document(
        "Tessara Responses",
        "Tessara responses list screen.",
        "/app/responses",
    )
}

pub fn submission_application_shell_html() -> String {
    document::render_native_app_document(
        "Tessara Responses",
        "Tessara responses list screen.",
        "/app/submissions",
    )
}

pub fn response_create_application_html() -> String {
    document::render_native_app_document(
        "Start Response",
        "Start a Tessara response.",
        "/app/responses/new",
    )
}

pub fn response_detail_application_html(submission_id: &str) -> String {
    document::render_native_app_document(
        "Response Detail",
        "Inspect a Tessara response.",
        &format!("/app/responses/{submission_id}"),
    )
}

pub fn response_edit_application_html(submission_id: &str) -> String {
    document::render_native_app_document(
        "Edit Response",
        "Edit a Tessara response.",
        &format!("/app/responses/{submission_id}/edit"),
    )
}

pub fn dashboards_application_shell_html() -> String {
    application::dashboards_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn datasets_application_shell_html() -> String {
    application::datasets_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn dataset_detail_application_html(dataset_id: &str) -> String {
    application::dataset_detail_application_html(app_script::APPLICATION_SCRIPT, dataset_id)
}

pub fn components_application_shell_html() -> String {
    application::components_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn component_detail_application_html(component_ref: &str) -> String {
    application::component_detail_application_html(app_script::APPLICATION_SCRIPT, component_ref)
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

pub fn node_types_application_shell_html() -> String {
    application::node_types_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn node_type_create_application_html() -> String {
    application::node_type_create_application_html(app_script::APPLICATION_SCRIPT)
}

pub fn node_type_detail_application_html(node_type_id: &str) -> String {
    application::node_type_detail_application_html(app_script::APPLICATION_SCRIPT, node_type_id)
}

pub fn node_type_edit_application_html(node_type_id: &str) -> String {
    application::node_type_edit_application_html(app_script::APPLICATION_SCRIPT, node_type_id)
}

pub fn roles_application_shell_html() -> String {
    application::roles_application_shell_html(app_script::APPLICATION_SCRIPT)
}

pub fn role_create_application_html() -> String {
    application::role_create_application_html(app_script::APPLICATION_SCRIPT)
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
        bridge_asset, component_detail_application_html, components_application_shell_html,
        dashboard_create_application_html, dashboard_detail_application_html,
        dashboard_edit_application_html, dashboards_application_shell_html,
        dataset_detail_application_html, datasets_application_shell_html,
        form_create_application_html, form_detail_application_html, form_edit_application_html,
        forms_application_shell_html, login_application_html, migration_application_shell_html,
        node_type_create_application_html, node_type_detail_application_html,
        node_type_edit_application_html, node_types_application_shell_html,
        organization_application_shell_html, organization_create_application_html,
        organization_detail_application_html, organization_edit_application_html,
        report_create_application_html, report_detail_application_html,
        report_edit_application_html, reporting_application_shell_html,
        response_create_application_html, response_detail_application_html,
        response_edit_application_html, role_create_application_html, role_detail_application_html,
        role_edit_application_html, roles_application_shell_html,
        submission_application_shell_html, user_access_application_html,
        user_create_application_html, user_detail_application_html, user_edit_application_html,
        users_application_shell_html,
    };
    use std::collections::{HashMap, HashSet};

    #[derive(Clone)]
    struct OrganizationScopeFixture {
        node_id: Option<&'static str>,
        id: Option<&'static str>,
        node_type_name: Option<&'static str>,
        scope_node_type: Option<&'static str>,
        scope_node_type_name: Option<&'static str>,
    }

    #[derive(Clone)]
    struct OrganizationNodeFixture {
        id: &'static str,
        node_type_name: Option<&'static str>,
        parent_node_id: Option<&'static str>,
    }

    fn normalize_type_label(raw: &str) -> String {
        let parts = raw
            .trim()
            .split(&['_', '-'][..])
            .filter_map(|part| {
                let part = part.trim();
                if part.is_empty() {
                    None
                } else {
                    let mut chars = part.chars();
                    Some(match chars.next() {
                        Some(first) => {
                            let mut out = String::new();
                            out.push(first.to_ascii_uppercase());
                            out.push_str(&chars.as_str().to_ascii_lowercase());
                            out
                        }
                        None => String::new(),
                    })
                }
            })
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();

        if parts.is_empty() {
            "Organization".to_string()
        } else {
            parts.join(" ")
        }
    }

    fn derive_destination_label_for_test(
        scopes: &[OrganizationScopeFixture],
        nodes: &[OrganizationNodeFixture],
    ) -> String {
        let node_by_id = nodes
            .iter()
            .map(|node| (node.id, node.clone()))
            .collect::<HashMap<_, _>>();

        let mut scored = Vec::new();
        for scope in scopes {
            let raw_id = scope.node_id.or(scope.id);
            let node = raw_id.and_then(|scope_id| node_by_id.get(scope_id).cloned());
            let type_label = node
                .as_ref()
                .and_then(|node| node.node_type_name)
                .or(scope.node_type_name)
                .or(scope.scope_node_type)
                .or(scope.scope_node_type_name)
                .unwrap_or("Organization");

            let mut depth = 0usize;
            let mut cursor = node;
            let mut seen = HashSet::new();
            while let Some(current_node) = cursor {
                if seen.contains(current_node.id) {
                    break;
                }
                seen.insert(current_node.id);
                let next = current_node
                    .parent_node_id
                    .and_then(|parent_id| node_by_id.get(parent_id).cloned());
                if let Some(parent_node) = next {
                    depth += 1;
                    cursor = Some(parent_node);
                } else {
                    break;
                }
            }

            scored.push((depth, type_label.to_string()));
        }

        let mut filtered = scored
            .into_iter()
            .filter(|(_, label)| !label.trim().is_empty())
            .collect::<Vec<_>>();
        if filtered.is_empty() {
            return "Organization List".to_string();
        }

        filtered.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        format!("{} List", normalize_type_label(&filtered[0].1))
    }

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
        assert!(html.contains("Role-Ready Home Modules"));
        assert!(html.contains("Product Areas"));
        assert!(html.contains("Transitional Reporting"));
        assert!(html.contains("Current Deployment Readiness"));
        assert!(html.contains("Current Workflow Context"));
        assert!(html.contains("Internal Workspaces"));
        assert!(html.contains("top-app-bar"));
        assert!(html.contains("app-nav-toggle"));
        assert!(html.contains("global-search"));
        assert!(html.contains("Product navigation"));
        assert!(html.contains("/app"));
        assert!(!html.contains("Create Shortcuts"));
        assert!(!html.contains("breadcrumb-item"));
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
        assert!(html.contains("global-search"));
    }

    #[test]
    fn product_list_pages_expose_dedicated_list_screens() {
        let organization = organization_application_shell_html();
        let forms = forms_application_shell_html();
        let responses = submission_application_shell_html();
        let datasets = datasets_application_shell_html();
        let components = components_application_shell_html();
        let reports = reporting_application_shell_html();
        let dashboards = dashboards_application_shell_html();

        assert!(organization.contains("Organization Directory"));
        assert!(organization.contains("Create Organization"));
        assert!(organization.contains("organization-directory-tree"));
        assert!(organization.contains("organization-skeleton-card"));
        assert!(organization.contains("organization-toggle-button"));
        assert!(organization.contains("organization-create-link"));
        assert!(!organization.contains("organization-list-title"));
        assert!(!organization.contains("organization-list-status"));
        assert!(!organization.contains("organization-expand-all"));
        assert!(!organization.contains("organization-collapse-all"));
        assert!(!organization.contains("record-card"));
        assert!(!organization.contains("Node ID"));

        assert!(forms.contains("Forms"));
        assert!(forms.contains("Create Form"));
        assert!(forms.contains("form-list"));
        assert!(!forms.contains("Form ID"));

        assert!(responses.contains("Responses"));
        assert!(responses.contains("Start New Response"));
        assert!(responses.contains("response-start-actions"));
        assert!(responses.contains("Draft Responses"));
        assert!(responses.contains("Submitted Responses"));
        assert!(!responses.contains("Draft submission ID"));

        assert!(datasets.contains("Datasets"));
        assert!(datasets.contains("Dataset Directory"));
        assert!(datasets.contains("dataset-list"));

        assert!(components.contains("Components"));
        assert!(components.contains("Component Directory"));
        assert!(components.contains("component-list"));

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
        let dataset_detail =
            dataset_detail_application_html("00000000-0000-0000-0000-000000000010");
        let component_detail = component_detail_application_html(
            "00000000-0000-0000-0000-000000000011__00000000-0000-0000-0000-000000000012",
        );
        let report_new = report_create_application_html();
        let report_detail = report_detail_application_html("00000000-0000-0000-0000-000000000004");
        let report_edit = report_edit_application_html("00000000-0000-0000-0000-000000000004");
        let dashboard_new = dashboard_create_application_html();
        let dashboard_detail =
            dashboard_detail_application_html("00000000-0000-0000-0000-000000000005");
        let dashboard_edit =
            dashboard_edit_application_html("00000000-0000-0000-0000-000000000005");

        for html in [
            report_new.as_str(),
            report_edit.as_str(),
            dashboard_new.as_str(),
            dashboard_edit.as_str(),
        ] {
            assert!(html.contains("Submit"));
            assert!(html.contains("Cancel"));
            assert!(!html.contains(" ID"));
        }

        assert!(form_new.contains("Create Form"));
        assert!(form_new.contains("Cancel"));
        assert!(form_new.contains("form-editor-status"));
        assert!(!form_new.contains(" ID"));
        assert!(form_edit.contains("Edit Form"));
        assert!(form_edit.contains("Save Form"));
        assert!(form_edit.contains("Draft Version Workspace"));
        assert!(form_edit.contains("Cancel"));
        assert!(!form_edit.contains(" ID"));

        assert!(response_new.contains("Start Response"));
        assert!(response_new.contains("Start Draft"));
        assert!(response_new.contains("Cancel"));
        assert!(response_edit.contains("Edit Response"));
        assert!(response_edit.contains("Loading response form"));
        assert!(response_edit.contains("response-edit-surface"));

        assert!(organization_new.contains("Submit"));
        assert!(organization_new.contains("Cancel"));
        assert!(organization_detail.contains("Organization Detail"));
        assert!(organization_detail.contains("Back to List"));
        assert!(organization_detail.contains("organization-detail"));
        assert!(organization_detail.contains("No Node ID was provided"));
        assert!(organization_detail.contains("organization-detail-path"));
        assert!(organization_detail.contains("organization-child-actions"));
        assert!(!organization_detail.contains("Related Responses"));
        assert!(organization_edit.contains("Submit"));
        assert!(organization_edit.contains("Cancel"));
        assert!(form_detail.contains("Form Detail"));
        assert!(response_detail.contains("Response Detail"));
        assert!(dataset_detail.contains("Dataset Detail"));
        assert!(dataset_detail.contains("dataset-detail"));
        assert!(component_detail.contains("Component Detail"));
        assert!(component_detail.contains("component-detail"));
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
        assert!(administration.contains("Organization Schema"));
        assert!(administration.contains("/app/administration/node-types"));
        assert!(administration.contains("Open Legacy Builder"));
        assert!(administration.contains("/app/admin"));
        assert!(administration.contains("/app/datasets"));
        assert!(administration.contains("/app/components"));

        assert!(users.contains("User Management"));
        assert!(users.contains("Create User"));
        assert!(users.contains("user-list"));

        assert!(roles.contains("Roles"));
        assert!(roles.contains("role-list"));
        assert!(roles.contains("Create Role"));

        assert!(migration.contains("Migration Workbench"));
        assert!(migration.contains("Load Fixture Examples"));
        assert!(migration.contains("Validate Fixture"));
        assert!(migration.contains("Dry-Run Fixture"));
        assert!(migration.contains("Import Fixture"));
        assert!(administration.contains("Administration Workspace"));
        assert!(migration.contains("Migration Workspace"));
    }

    #[test]
    fn breadcrumbs_only_render_on_deeper_routes() {
        let forms = forms_application_shell_html();
        let organization_detail =
            organization_detail_application_html("00000000-0000-0000-0000-000000000001");
        let report_detail = report_detail_application_html("00000000-0000-0000-0000-000000000004");

        assert!(!forms.contains("breadcrumb-item"));
        assert!(organization_detail.contains("breadcrumb-item"));
        assert!(report_detail.contains("breadcrumb-item"));
    }

    #[test]
    fn org_and_node_type_pages_expose_route_ownership_markers() {
        let organization = organization_application_shell_html();
        let organization_create = organization_create_application_html();
        let organization_detail =
            organization_detail_application_html("00000000-0000-0000-0000-000000000001");
        let organization_edit =
            organization_edit_application_html("00000000-0000-0000-0000-000000000001");
        let node_types = node_types_application_shell_html();
        let node_type_create = node_type_create_application_html();
        let node_type_detail =
            node_type_detail_application_html("00000000-0000-0000-0000-000000000010");
        let node_type_edit =
            node_type_edit_application_html("00000000-0000-0000-0000-000000000010");

        assert!(organization.contains(r#"data-page-key="organization-list""#));
        assert!(organization.contains(r#"data-active-route="organization""#));
        assert!(organization_create.contains(r#"data-page-key="organization-create""#));
        assert!(organization_create.contains(r#"data-active-route="organization""#));
        assert!(organization_detail.contains(r#"data-page-key="organization-detail""#));
        assert!(
            organization_detail
                .contains(r#"data-record-id="00000000-0000-0000-0000-000000000001""#)
        );
        assert!(organization_edit.contains(r#"data-page-key="organization-edit""#));

        assert!(node_types.contains(r#"data-page-key="node-type-list""#));
        assert!(node_types.contains(r#"data-active-route="administration""#));
        assert!(node_type_create.contains(r#"data-page-key="node-type-create""#));
        assert!(node_type_detail.contains(r#"data-page-key="node-type-detail""#));
        assert!(
            node_type_detail.contains(r#"data-record-id="00000000-0000-0000-0000-000000000010""#)
        );
        assert!(node_type_edit.contains(r#"data-page-key="node-type-edit""#));
    }

    #[test]
    fn organization_scope_title_prefers_top_level_scope_over_deeper_scope_nodes() {
        let nodes = vec![
            OrganizationNodeFixture {
                id: "partner-id",
                node_type_name: Some("Partner"),
                parent_node_id: None,
            },
            OrganizationNodeFixture {
                id: "program-id",
                node_type_name: Some("Program"),
                parent_node_id: Some("partner-id"),
            },
            OrganizationNodeFixture {
                id: "activity-id",
                node_type_name: Some("Activity"),
                parent_node_id: Some("program-id"),
            },
        ];
        let scopes = vec![
            OrganizationScopeFixture {
                node_id: Some("activity-id"),
                id: None,
                node_type_name: None,
                scope_node_type: None,
                scope_node_type_name: None,
            },
            OrganizationScopeFixture {
                node_id: Some("partner-id"),
                id: None,
                node_type_name: None,
                scope_node_type: None,
                scope_node_type_name: None,
            },
        ];

        let label = derive_destination_label_for_test(&scopes, &nodes);

        assert_eq!(label, "Partner List");
    }

    #[test]
    fn organization_scope_title_handles_missing_tree_rows_with_scope_fallbacks() {
        let nodes = vec![OrganizationNodeFixture {
            id: "orphan-child-id",
            node_type_name: Some("Session"),
            parent_node_id: Some("missing-parent-id"),
        }];
        let scopes = vec![
            OrganizationScopeFixture {
                node_id: Some("orphan-child-id"),
                id: None,
                node_type_name: None,
                scope_node_type: None,
                scope_node_type_name: None,
            },
            OrganizationScopeFixture {
                node_id: Some("missing-scope-id"),
                id: None,
                node_type_name: None,
                scope_node_type: None,
                scope_node_type_name: Some("Partner"),
            },
        ];

        let label = derive_destination_label_for_test(&scopes, &nodes);

        assert_eq!(label, "Partner List");
    }

    #[test]
    fn user_management_pages_are_dedicated() {
        let create = user_create_application_html();
        let detail = user_detail_application_html("00000000-0000-0000-0000-000000000006");
        let edit = user_edit_application_html("00000000-0000-0000-0000-000000000006");
        let access = user_access_application_html("00000000-0000-0000-0000-000000000006");
        let node_types = node_types_application_shell_html();
        let node_type_create = node_type_create_application_html();
        let node_type_detail =
            node_type_detail_application_html("00000000-0000-0000-0000-000000000008");
        let node_type_edit =
            node_type_edit_application_html("00000000-0000-0000-0000-000000000008");
        let role_create = role_create_application_html();
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
        assert!(access.contains("user-delegation-options"));

        assert!(node_types.contains("Organization Node Types"));
        assert!(node_types.contains("node-type-list"));
        assert!(node_types.contains("Create Organization Node Type"));
        assert!(node_type_create.contains("node-type-form"));
        assert!(!node_type_create.contains("node-type-singular-label"));
        assert!(node_type_create.contains("node-type-parent-tags"));
        assert!(node_type_create.contains("node-type-parent-filter"));
        assert!(node_type_create.contains("node-type-parent-options"));
        assert!(node_type_create.contains("node-type-child-tags"));
        assert!(node_type_create.contains("node-type-child-filter"));
        assert!(node_type_create.contains("node-type-child-options"));
        assert!(node_type_create.contains("node-type-metadata-fields-editor"));
        assert!(node_type_create.contains("node-type-metadata-settings-modal"));
        assert!(node_type_detail.contains("Organization Node Type Detail"));
        assert!(node_type_detail.contains("Back to List"));
        assert!(node_type_edit.contains("Edit Organization Node Type"));
        assert!(node_type_edit.contains("node-type-plural-label"));
        assert!(node_type_edit.contains("Submit"));

        assert!(role_create.contains("Create Role"));
        assert!(role_create.contains("role-name"));
        assert!(role_detail.contains("Role Detail"));
        assert!(role_detail.contains("Back to List"));

        assert!(role_edit.contains("Edit Role"));
        assert!(role_edit.contains("role-capability-options"));
        assert!(role_edit.contains("Submit"));
    }
}
