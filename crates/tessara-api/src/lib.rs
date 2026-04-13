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

use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
};
use db::AppState;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

/// Builds the complete Tessara HTTP router for the supplied application state.
///
/// The router includes the API endpoints for the current vertical slice plus a
/// minimal local admin shell at `/`. It is kept as a public function so tests,
/// binaries, and future deployment adapters can construct the same service
/// surface without duplicating route registration.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Html(tessara_web::admin_shell_html()) }))
        .route("/assets/{asset_name}", get(svg_asset))
        .route("/bridge/{asset_name}", get(bridge_asset))
        .nest_service("/pkg", ServeDir::new(tessara_web::pkg_dir()))
        .route(
            "/app",
            get(|| async { Html(tessara_web::application_shell_html()) }),
        )
        .route(
            "/app/login",
            get(|| async { Html(tessara_web::login_application_html()) }),
        )
        .route(
            "/app/organization",
            get(|| async { Html(tessara_web::organization_application_shell_html()) }),
        )
        .route(
            "/app/organization/new",
            get(|| async { Html(tessara_web::organization_create_application_html()) }),
        )
        .route(
            "/app/organization/{node_id}/edit",
            get(|Path(node_id): Path<String>| async move {
                Html(tessara_web::organization_edit_application_html(&node_id))
            }),
        )
        .route(
            "/app/organization/{node_id}",
            get(|Path(node_id): Path<String>| async move {
                Html(tessara_web::organization_detail_application_html(&node_id))
            }),
        )
        .route(
            "/app/forms",
            get(|| async { Html(tessara_web::forms_application_shell_html()) }),
        )
        .route(
            "/app/forms/new",
            get(|| async { Html(tessara_web::form_create_application_html()) }),
        )
        .route(
            "/app/forms/{form_id}/edit",
            get(|Path(form_id): Path<String>| async move {
                Html(tessara_web::form_edit_application_html(&form_id))
            }),
        )
        .route(
            "/app/forms/{form_id}",
            get(|Path(form_id): Path<String>| async move {
                Html(tessara_web::form_detail_application_html(&form_id))
            }),
        )
        .route(
            "/app/responses",
            get(|| async { Html(tessara_web::responses_application_shell_html()) }),
        )
        .route(
            "/app/responses/new",
            get(|| async { Html(tessara_web::response_create_application_html()) }),
        )
        .route(
            "/app/responses/{submission_id}/edit",
            get(|Path(submission_id): Path<String>| async move {
                Html(tessara_web::response_edit_application_html(&submission_id))
            }),
        )
        .route(
            "/app/responses/{submission_id}",
            get(|Path(submission_id): Path<String>| async move {
                Html(tessara_web::response_detail_application_html(
                    &submission_id,
                ))
            }),
        )
        .route(
            "/app/submissions",
            get(|| async { Html(tessara_web::submission_application_shell_html()) }),
        )
        .route(
            "/app/administration",
            get(|| async { Html(tessara_web::administration_application_shell_html()) }),
        )

        .route(
            "/app/administration/users",
            get(|| async { Html(tessara_web::users_application_shell_html()) }),
        )
        .route(
            "/app/administration/users/new",
            get(|| async { Html(tessara_web::user_create_application_html()) }),
        )
        .route(
            "/app/administration/users/{account_id}/edit",
            get(|Path(account_id): Path<String>| async move {
                Html(tessara_web::user_edit_application_html(&account_id))
            }),
        )
        .route(
            "/app/administration/users/{account_id}",
            get(|Path(account_id): Path<String>| async move {
                Html(tessara_web::user_detail_application_html(&account_id))
            }),
        )
        .route(
            "/app/administration/users/{account_id}/access",
            get(|Path(account_id): Path<String>| async move {
                Html(tessara_web::user_access_application_html(&account_id))
            }),
        )
        .route(
            "/app/administration/roles",
            get(|| async { Html(tessara_web::roles_application_shell_html()) }),
        )
        .route(
            "/app/administration/roles/{role_id}/edit",
            get(|Path(role_id): Path<String>| async move {
                Html(tessara_web::role_edit_application_html(&role_id))
            }),
        )
        .route(
            "/app/administration/roles/{role_id}",
            get(|Path(role_id): Path<String>| async move {
                Html(tessara_web::role_detail_application_html(&role_id))
            }),
        )
        .route(
            "/app/admin",
            get(|| async { Html(tessara_web::admin_application_shell_html()) }),
        )
        .route(
            "/app/reports",
            get(|| async { Html(tessara_web::reporting_application_shell_html()) }),
        )
        .route(
            "/app/reports/new",
            get(|| async { Html(tessara_web::report_create_application_html()) }),
        )
        .route(
            "/app/reports/{report_id}/edit",
            get(|Path(report_id): Path<String>| async move {
                Html(tessara_web::report_edit_application_html(&report_id))
            }),
        )
        .route(
            "/app/reports/{report_id}",
            get(|Path(report_id): Path<String>| async move {
                Html(tessara_web::report_detail_application_html(&report_id))
            }),
        )
        .route(
            "/app/dashboards",
            get(|| async { Html(tessara_web::dashboards_application_shell_html()) }),
        )
        .route(
            "/app/dashboards/new",
            get(|| async { Html(tessara_web::dashboard_create_application_html()) }),
        )
        .route(
            "/app/dashboards/{dashboard_id}/edit",
            get(|Path(dashboard_id): Path<String>| async move {
                Html(tessara_web::dashboard_edit_application_html(&dashboard_id))
            }),
        )
        .route(
            "/app/dashboards/{dashboard_id}",
            get(|Path(dashboard_id): Path<String>| async move {
                Html(tessara_web::dashboard_detail_application_html(
                    &dashboard_id,
                ))
            }),
        )
        .route(
            "/app/migration",
            get(|| async { Html(tessara_web::migration_application_shell_html()) }),
        )
        .route("/health", get(|| async { "ok" }))
        .route("/api/app/summary", get(app_summary::get_summary))
        .route("/api/auth/login", post(auth::login))
        .route("/api/me", get(auth::me))
        .route("/api/admin/capabilities", get(users::list_capabilities))
        .route("/api/admin/roles", get(users::list_roles))
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
            put(users::update_user_access),
        )
        .route(
            "/api/admin/node-types",
            get(hierarchy::list_node_types).post(hierarchy::create_node_type),
        )
        .route(
            "/api/admin/node-types/{node_type_id}",
            get(hierarchy::get_node_type).put(hierarchy::update_node_type),
        )
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
            put(hierarchy::update_node_metadata_field),
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

async fn bridge_asset(Path(asset_name): Path<String>) -> impl IntoResponse {
    match tessara_web::bridge_asset(&asset_name) {
        Some(asset) => (
            [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
            asset,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "asset not found").into_response(),
    }
}
