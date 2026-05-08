//! Tessara API service crate.
//!
//! This crate owns the HTTP routing layer and the current API-first vertical
//! slice implementation. Most endpoint modules are deliberately private so the
//! public Rust API stays focused on service startup, shared configuration, and
//! deterministic demo seeding.

mod analytics;
mod app_summary;
mod auth;
pub mod config;
mod dashboards;
mod datasets;
pub mod db;
pub mod demo;
pub mod error;
mod forms;
mod hierarchy;
pub mod legacy_import;
mod reporting;
mod submissions;
mod users;
mod workflows;

use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
};
use db::AppState;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

fn native_app(path: impl AsRef<str>, title: &str, description: &str) -> Html<String> {
    Html(tessara_web::application_html(
        path.as_ref(),
        title,
        description,
    ))
}

/// Builds the complete Tessara HTTP router for the supplied application state.
///
/// The router includes the API endpoints for the current vertical slice plus a
/// minimal local admin shell at `/`. It is kept as a public function so tests,
/// binaries, and future deployment adapters can construct the same service
/// surface without duplicating route registration.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { native_app("/", "Tessara Home", "Tessara native Leptos home.") }),
        )
        .route(
            "/login",
            get(|| async { native_app("/login", "Tessara Sign In", "Sign in to Tessara.") }),
        )
        .route("/assets/{asset_name}", get(svg_asset))
        .nest_service("/pkg", ServeDir::new(tessara_web::pkg_dir()))
        .route(
            "/organization",
            get(|| async {
                native_app(
                    "/organization",
                    "Tessara Organization",
                    "Browse the Tessara organization hierarchy.",
                )
            }),
        )
        .route(
            "/organization/new",
            get(|| async {
                native_app(
                    "/organization/new",
                    "Create Organization Node",
                    "Create an organization node.",
                )
            }),
        )
        .route(
            "/organization/{node_id}/edit",
            get(|Path(node_id): Path<String>| async move {
                native_app(
                    format!("/organization/{node_id}/edit"),
                    "Edit Organization Node",
                    "Edit an organization node.",
                )
            }),
        )
        .route(
            "/organization/{node_id}",
            get(|Path(node_id): Path<String>| async move {
                native_app(
                    format!("/organization/{node_id}"),
                    "Organization Detail",
                    "Inspect an organization node.",
                )
            }),
        )
        .route(
            "/forms",
            get(|| async { native_app("/forms", "Tessara Forms", "Browse Tessara forms.") }),
        )
        .route(
            "/forms/new",
            get(|| async { native_app("/forms/new", "Create Form", "Create a Tessara form.") }),
        )
        .route(
            "/forms/{form_id}/edit",
            get(|Path(form_id): Path<String>| async move {
                native_app(
                    format!("/forms/{form_id}/edit"),
                    "Edit Form",
                    "Edit a Tessara form.",
                )
            }),
        )
        .route(
            "/forms/{form_id}",
            get(|Path(form_id): Path<String>| async move {
                native_app(
                    format!("/forms/{form_id}"),
                    "Form Detail",
                    "Inspect a Tessara form.",
                )
            }),
        )
        .route(
            "/workflows",
            get(|| async {
                native_app(
                    "/workflows",
                    "Tessara Workflows",
                    "Browse Tessara workflows.",
                )
            }),
        )
        .route(
            "/workflows/new",
            get(|| async {
                native_app(
                    "/workflows/new",
                    "Create Workflow",
                    "Create a Tessara workflow.",
                )
            }),
        )
        .route(
            "/workflows/assignments",
            get(|| async {
                native_app(
                    "/workflows/assignments",
                    "Workflow Assignments",
                    "Manage workflow assignments.",
                )
            }),
        )
        .route(
            "/workflows/{workflow_id}/edit",
            get(|Path(workflow_id): Path<String>| async move {
                native_app(
                    format!("/workflows/{workflow_id}/edit"),
                    "Edit Workflow",
                    "Edit a Tessara workflow.",
                )
            }),
        )
        .route(
            "/workflows/{workflow_id}",
            get(|Path(workflow_id): Path<String>| async move {
                native_app(
                    format!("/workflows/{workflow_id}"),
                    "Workflow Detail",
                    "Inspect a Tessara workflow.",
                )
            }),
        )
        .route(
            "/responses",
            get(|| async {
                native_app(
                    "/responses",
                    "Tessara Responses",
                    "Browse Tessara responses.",
                )
            }),
        )
        .route(
            "/responses/new",
            get(|| async {
                native_app(
                    "/responses/new",
                    "Start Response",
                    "Start a Tessara response.",
                )
            }),
        )
        .route(
            "/responses/{submission_id}/edit",
            get(|Path(submission_id): Path<String>| async move {
                native_app(
                    format!("/responses/{submission_id}/edit"),
                    "Edit Response",
                    "Edit a Tessara response.",
                )
            }),
        )
        .route(
            "/responses/{submission_id}",
            get(|Path(submission_id): Path<String>| async move {
                native_app(
                    format!("/responses/{submission_id}"),
                    "Response Detail",
                    "Inspect a Tessara response.",
                )
            }),
        )
        .route(
            "/components",
            get(|| async {
                native_app(
                    "/components",
                    "Tessara Components",
                    "Browse Tessara components.",
                )
            }),
        )
        .route(
            "/components/{component_ref}",
            get(|Path(component_ref): Path<String>| async move {
                native_app(
                    format!("/components/{component_ref}"),
                    "Component Detail",
                    "Inspect a Tessara component.",
                )
            }),
        )
        .route(
            "/dashboards",
            get(|| async {
                native_app(
                    "/dashboards",
                    "Tessara Dashboards",
                    "Browse Tessara dashboards.",
                )
            }),
        )
        .route(
            "/dashboards/new",
            get(|| async {
                native_app(
                    "/dashboards/new",
                    "Create Dashboard",
                    "Create a Tessara dashboard.",
                )
            }),
        )
        .route(
            "/dashboards/{dashboard_id}/edit",
            get(|Path(dashboard_id): Path<String>| async move {
                native_app(
                    format!("/dashboards/{dashboard_id}/edit"),
                    "Edit Dashboard",
                    "Edit a Tessara dashboard.",
                )
            }),
        )
        .route(
            "/dashboards/{dashboard_id}",
            get(|Path(dashboard_id): Path<String>| async move {
                native_app(
                    format!("/dashboards/{dashboard_id}"),
                    "Dashboard Detail",
                    "Inspect a Tessara dashboard.",
                )
            }),
        )
        .route(
            "/datasets",
            get(|| async {
                native_app("/datasets", "Tessara Datasets", "Browse Tessara datasets.")
            }),
        )
        .route(
            "/datasets/{dataset_id}",
            get(|Path(dataset_id): Path<String>| async move {
                native_app(
                    format!("/datasets/{dataset_id}"),
                    "Dataset Detail",
                    "Inspect a Tessara dataset.",
                )
            }),
        )
        .route(
            "/administration",
            get(|| async {
                native_app(
                    "/administration",
                    "Tessara Administration",
                    "Manage Tessara administration.",
                )
            }),
        )
        .route(
            "/administration/users",
            get(|| async {
                native_app(
                    "/administration/users",
                    "Tessara Users",
                    "Manage Tessara users.",
                )
            }),
        )
        .route(
            "/administration/node-types",
            get(|| async {
                native_app(
                    "/administration/node-types",
                    "Tessara Node Types",
                    "Manage organization node types.",
                )
            }),
        )
        .route(
            "/administration/roles",
            get(|| async {
                native_app(
                    "/administration/roles",
                    "Tessara Roles",
                    "Manage Tessara roles.",
                )
            }),
        )
        .route(
            "/migration",
            get(|| async {
                native_app(
                    "/migration",
                    "Tessara Migration",
                    "Run Tessara migration workflows.",
                )
            }),
        )
        .route("/health", get(|| async { "ok" }))
        .route("/api/summary", get(app_summary::get_summary))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/session", get(auth::session))
        .route("/api/auth/logout", delete(auth::logout))
        .route("/api/me", get(auth::me))
        .route("/api/admin/capabilities", get(users::list_capabilities))
        .route(
            "/api/admin/roles",
            get(users::list_roles).post(users::create_role),
        )
        .route(
            "/api/admin/roles/{role_id}",
            get(users::get_role).put(users::update_role),
        )
        .route(
            "/api/admin/users",
            get(users::list_users).post(users::create_user),
        )
        .route(
            "/api/admin/users/{account_id}",
            get(users::get_user).put(users::update_user),
        )
        .route(
            "/api/admin/users/{account_id}/access",
            get(users::get_user_access).put(users::update_user_access),
        )
        .route(
            "/api/admin/node-types",
            get(hierarchy::list_node_types).post(hierarchy::create_node_type),
        )
        .route(
            "/api/admin/node-types/{node_type_id}",
            get(hierarchy::get_node_type).put(hierarchy::update_node_type),
        )
        .route("/api/node-types", get(hierarchy::list_readable_node_types))
        .route(
            "/api/admin/node-type-relationships",
            get(hierarchy::list_node_type_relationships)
                .post(hierarchy::create_node_type_relationship),
        )
        .route(
            "/api/admin/node-type-relationships/{parent_node_type_id}/{child_node_type_id}",
            delete(hierarchy::delete_node_type_relationship),
        )
        .route(
            "/api/admin/node-metadata-fields",
            get(hierarchy::list_node_metadata_fields).post(hierarchy::create_node_metadata_field),
        )
        .route(
            "/api/admin/node-metadata-fields/{field_id}",
            put(hierarchy::update_node_metadata_field)
                .delete(hierarchy::delete_node_metadata_field),
        )
        .route("/api/admin/nodes", post(hierarchy::create_node))
        .route("/api/admin/nodes/{node_id}", put(hierarchy::update_node))
        .route("/api/nodes", get(hierarchy::list_nodes))
        .route("/api/nodes/{node_id}", get(hierarchy::get_node))
        .route(
            "/api/admin/forms",
            get(forms::list_forms).post(forms::create_form),
        )
        .route("/api/forms", get(forms::list_readable_forms))
        .route("/api/forms/{form_id}", get(forms::get_readable_form))
        .route(
            "/api/admin/forms/{form_id}",
            get(forms::get_form).put(forms::update_form),
        )
        .route(
            "/api/admin/forms/{form_id}/versions",
            post(forms::create_form_version),
        )
        .route(
            "/api/admin/form-versions/{form_version_id}/sections",
            post(forms::create_form_section),
        )
        .route(
            "/api/admin/form-versions/{form_version_id}/fields",
            post(forms::create_form_field),
        )
        .route(
            "/api/admin/form-sections/{section_id}",
            put(forms::update_form_section).delete(forms::delete_form_section),
        )
        .route(
            "/api/admin/form-fields/{field_id}",
            put(forms::update_form_field).delete(forms::delete_form_field),
        )
        .route(
            "/api/admin/form-versions/{form_version_id}/publish",
            post(forms::publish_form_version),
        )
        .route(
            "/api/form-versions/{form_version_id}/render",
            get(forms::render_form_version),
        )
        .route(
            "/api/forms/published",
            get(forms::list_published_form_versions),
        )
        .route(
            "/api/workflows",
            get(workflows::list_workflows).post(workflows::create_workflow),
        )
        .route(
            "/api/workflows/{workflow_id}",
            get(workflows::get_workflow).put(workflows::update_workflow),
        )
        .route(
            "/api/workflows/{workflow_id}/versions",
            post(workflows::create_workflow_version),
        )
        .route(
            "/api/workflow-versions/{workflow_version_id}/publish",
            post(workflows::publish_workflow_version),
        )
        .route(
            "/api/workflow-versions/{workflow_version_id}/steps",
            put(workflows::replace_workflow_version_steps),
        )
        .route(
            "/api/workflow-versions/{workflow_version_id}",
            delete(workflows::delete_workflow_version),
        )
        .route(
            "/api/workflow-assignment-candidates",
            get(workflows::list_assignment_candidates),
        )
        .route(
            "/api/workflow-assignment-candidates/assignees",
            get(workflows::list_assignment_candidate_assignees),
        )
        .route(
            "/api/workflow-assignments",
            get(workflows::list_workflow_assignments).post(workflows::create_workflow_assignment),
        )
        .route(
            "/api/workflow-assignments/bulk",
            post(workflows::bulk_create_workflow_assignments),
        )
        .route(
            "/api/workflow-assignments/pending",
            get(workflows::list_pending_work),
        )
        .route(
            "/api/workflow-assignments/{workflow_assignment_id}",
            put(workflows::update_workflow_assignment),
        )
        .route(
            "/api/workflow-assignments/{workflow_assignment_id}/start",
            post(workflows::start_assignment),
        )
        .route(
            "/api/responses/options",
            get(submissions::list_response_start_options),
        )
        .route("/api/submissions/drafts", post(submissions::create_draft))
        .route("/api/submissions", get(submissions::list_submissions))
        .route(
            "/api/submissions/{submission_id}",
            get(submissions::get_submission).delete(submissions::delete_draft_submission),
        )
        .route(
            "/api/submissions/{submission_id}/values",
            put(submissions::save_submission_values),
        )
        .route(
            "/api/submissions/{submission_id}/submit",
            post(submissions::submit_submission),
        )
        .route(
            "/api/admin/analytics/refresh",
            post(analytics::refresh_analytics),
        )
        .route(
            "/api/admin/analytics/status",
            get(analytics::analytics_status),
        )
        .route("/api/admin/datasets", post(datasets::create_dataset))
        .route(
            "/api/admin/datasets/{dataset_id}",
            put(datasets::update_dataset).delete(datasets::delete_dataset),
        )
        .route("/api/datasets", get(datasets::list_datasets))
        .route("/api/datasets/{dataset_id}", get(datasets::get_dataset))
        .route(
            "/api/datasets/{dataset_id}/table",
            get(datasets::run_dataset_table),
        )
        .route(
            "/api/admin/legacy-fixtures/validate",
            post(legacy_import::validate_legacy_fixture_endpoint),
        )
        .route(
            "/api/admin/legacy-fixtures/dry-run",
            post(legacy_import::dry_run_legacy_fixture_endpoint),
        )
        .route(
            "/api/admin/legacy-fixtures/import",
            post(legacy_import::import_legacy_fixture_endpoint),
        )
        .route(
            "/api/admin/legacy-fixtures/examples",
            get(legacy_import::list_legacy_fixture_examples),
        )
        .route("/api/admin/reports", post(reporting::create_report))
        .route(
            "/api/admin/reports/{report_id}",
            put(reporting::update_report).delete(reporting::delete_report),
        )
        .route("/api/reports", get(reporting::list_reports))
        .route("/api/reports/{report_id}", get(reporting::get_report))
        .route("/api/reports/{report_id}/table", get(reporting::run_report))
        .route(
            "/api/admin/aggregations",
            post(reporting::create_aggregation),
        )
        .route("/api/aggregations", get(reporting::list_aggregations))
        .route(
            "/api/admin/aggregations/{aggregation_id}",
            put(reporting::update_aggregation).delete(reporting::delete_aggregation),
        )
        .route(
            "/api/aggregations/{aggregation_id}",
            get(reporting::get_aggregation),
        )
        .route(
            "/api/aggregations/{aggregation_id}/table",
            get(reporting::run_aggregation),
        )
        .route("/api/admin/charts", post(dashboards::create_chart))
        .route(
            "/api/admin/charts/{chart_id}",
            put(dashboards::update_chart).delete(dashboards::delete_chart),
        )
        .route("/api/charts", get(dashboards::list_charts))
        .route("/api/charts/{chart_id}", get(dashboards::get_chart))
        .route("/api/admin/dashboards", post(dashboards::create_dashboard))
        .route(
            "/api/admin/dashboards/{dashboard_id}",
            put(dashboards::update_dashboard).delete(dashboards::delete_dashboard),
        )
        .route(
            "/api/admin/dashboards/{dashboard_id}/components",
            post(dashboards::add_dashboard_component),
        )
        .route(
            "/api/admin/dashboard-components/{component_id}",
            put(dashboards::update_dashboard_component)
                .delete(dashboards::delete_dashboard_component),
        )
        .route(
            "/api/dashboards/{dashboard_id}",
            get(dashboards::get_dashboard),
        )
        .route("/api/dashboards", get(dashboards::list_dashboards))
        .route("/api/demo/seed", post(demo::seed_demo_endpoint))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn svg_asset(Path(asset_name): Path<String>) -> impl IntoResponse {
    match tessara_web::svg_asset(&asset_name) {
        Some(svg) => (
            [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
            svg,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "asset not found").into_response(),
    }
}
