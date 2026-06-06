//! Tessara API service crate.
//!
//! This crate owns the HTTP routing layer and the current API-first vertical
//! slice implementation. Most endpoint modules are deliberately private so the
//! public Rust API stays focused on service startup, shared configuration, and
//! deterministic demo seeding.

mod analytics;
mod app_summary;
mod auth;
mod components;
pub mod config;
mod dashboards;
mod datasets;
pub mod db;
pub mod demo;
pub mod error;
mod forms;
mod hierarchy;
mod operations;
mod submissions;
mod users;
mod workflows;

#[cfg(feature = "ssr")]
use axum::http::header;
use axum::{
    Router,
    extract::{Path, Request, State},
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use db::AppState;
use error::ApiError;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

fn native_app(path: impl AsRef<str>, title: &str, description: &str) -> Html<String> {
    #[cfg(feature = "ssr")]
    {
        Html(tessara_web::application_html(
            path.as_ref(),
            title,
            description,
        ))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let path = path.as_ref();
        Html(format!(
            r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>{title}</title><meta name="description" content="{description}"></head><body><main id="app-root" data-path="{path}"></main></body></html>"#
        ))
    }
}

fn shell_pkg_dir() -> std::path::PathBuf {
    #[cfg(feature = "ssr")]
    {
        tessara_web::pkg_dir()
    }

    #[cfg(not(feature = "ssr"))]
    {
        std::path::PathBuf::from("target/site/pkg")
    }
}

/// Builds the complete Tessara HTTP router for the supplied application state.
///
/// The router includes the API endpoints for the current vertical slice plus a
/// minimal local admin shell at `/`. It is kept as a public function so tests,
/// binaries, and future deployment adapters can construct the same service
/// surface without duplicating route registration.
pub fn router(state: AppState) -> Router {
    let auth_state = state.clone();

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
        .nest_service("/pkg", ServeDir::new(shell_pkg_dir()))
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
            "/operations",
            get(|| async {
                native_app(
                    "/operations",
                    "Tessara Operations",
                    "Inspect workflow assignment and dataset readiness status.",
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
            "/administration/users/{account_id}",
            get(|Path(account_id): Path<String>| async move {
                native_app(
                    format!("/administration/users/{account_id}"),
                    "Tessara User Detail",
                    "Inspect a Tessara user.",
                )
            }),
        )
        .route(
            "/administration/users/{account_id}/access",
            get(|Path(account_id): Path<String>| async move {
                native_app(
                    format!("/administration/users/{account_id}/access"),
                    "Tessara User Permissions",
                    "Manage Tessara user permissions.",
                )
            }),
        )
        .route(
            "/administration/users/{account_id}/edit",
            get(|Path(account_id): Path<String>| async move {
                native_app(
                    format!("/administration/users/{account_id}/edit"),
                    "Edit Tessara User",
                    "Edit a Tessara user account.",
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
        .merge(api_routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(
            auth_state,
            require_authenticated_ui_route,
        ))
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/summary", get(app_summary::get_summary))
        .merge(auth::routes())
        .merge(users::routes())
        .merge(hierarchy::routes())
        .merge(operations::routes())
        .merge(forms::routes())
        .merge(workflows::routes())
        .merge(submissions::routes())
        .merge(analytics::routes())
        .merge(datasets::routes())
        .merge(components::routes())
        .merge(dashboards::routes())
        .merge(demo::routes())
}

async fn require_authenticated_ui_route(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if !is_protected_ui_request(&request) {
        return next.run(request).await;
    }

    match auth::authenticate_request(&state.pool, &state.config, request.headers()).await {
        Ok(_) => next.run(request).await,
        Err(ApiError::Unauthorized | ApiError::SessionExpired | ApiError::SessionRevoked) => {
            Redirect::to("/login").into_response()
        }
        Err(error) => error.into_response(),
    }
}

fn is_protected_ui_request(request: &Request) -> bool {
    if !matches!(request.method(), &Method::GET | &Method::HEAD) {
        return false;
    }

    let path = request.uri().path();
    !(path == "/"
        || path == "/login"
        || path == "/health"
        || path == "/api"
        || path.starts_with("/api/")
        || path == "/assets"
        || path.starts_with("/assets/")
        || path == "/pkg"
        || path.starts_with("/pkg/"))
}

async fn svg_asset(Path(asset_name): Path<String>) -> impl IntoResponse {
    #[cfg(feature = "ssr")]
    match tessara_web::svg_asset(&asset_name) {
        Some(svg) => (
            [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
            svg,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "asset not found").into_response(),
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = asset_name;
        (
            StatusCode::NOT_FOUND,
            "asset not available in API test build",
        )
            .into_response()
    }
}
