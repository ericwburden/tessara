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
mod composition;
pub mod config;
mod core_security;
mod dashboard_components_adapter;
mod dashboard_dependencies;
mod datasets;
pub mod db;
pub mod demo;
pub mod error;
mod forms;
mod hierarchy;
mod module_gateway;
mod modules;
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
        .route("/enrollment", get(core_security::enrollment_page))
        .route("/api/module-unavailable", get(module_unavailable_fallback))
        .route(
            "/_tessara/modules/{*asset_path}",
            get(module_gateway::asset),
        )
        .route("/assets/{asset_name}", get(static_asset))
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
            "/components/new",
            get(|| async {
                native_app(
                    "/components/new",
                    "Create Component",
                    "Create a Tessara component.",
                )
            }),
        )
        .route(
            "/components/{component_ref}/edit",
            get(|Path(component_ref): Path<String>| async move {
                native_app(
                    format!("/components/{component_ref}/edit"),
                    "Edit Component",
                    "Edit a Tessara component.",
                )
            }),
        )
        .route(
            "/components/{component_ref}/view",
            get(|Path(component_ref): Path<String>| async move {
                native_app(
                    format!("/components/{component_ref}/view"),
                    "Tessara Component",
                    "View a Tessara component.",
                )
            }),
        )
        .route(
            "/components/{component_ref}/versions",
            get(|Path(component_ref): Path<String>| async move {
                native_app(
                    format!("/components/{component_ref}/versions"),
                    "Component Versions",
                    "Review Tessara component version history.",
                )
            }),
        )
        .route(
            "/components/{component_ref}",
            get(|Path(component_ref): Path<String>| async move {
                native_app(
                    format!("/components/{component_ref}"),
                    "Tessara Component",
                    "View a Tessara component.",
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
            "/datasets/new",
            get(|| async {
                native_app(
                    "/datasets/new",
                    "Create Dataset",
                    "Create a Tessara dataset.",
                )
            }),
        )
        .route(
            "/datasets/{dataset_id}/edit",
            get(|Path(dataset_id): Path<String>| async move {
                native_app(
                    format!("/datasets/{dataset_id}/edit"),
                    "Edit Dataset",
                    "Edit a Tessara dataset.",
                )
            }),
        )
        .route(
            "/datasets/{dataset_id}/revisions",
            get(|Path(dataset_id): Path<String>| async move {
                native_app(
                    format!("/datasets/{dataset_id}/revisions"),
                    "Dataset Revisions",
                    "Review dataset revision history.",
                )
            }),
        )
        .route(
            "/datasets/{dataset_id}/preview",
            get(|Path(dataset_id): Path<String>| async move {
                native_app(
                    format!("/datasets/{dataset_id}/preview"),
                    "Dataset Preview",
                    "Preview a Tessara dataset.",
                )
            }),
        )
        .route(
            "/datasets/{dataset_id}/revisions/{revision_id}/edit",
            get(
                |Path((dataset_id, revision_id)): Path<(String, String)>| async move {
                    native_app(
                        format!("/datasets/{dataset_id}/revisions/{revision_id}/edit"),
                        "Edit Dataset Revision",
                        "Edit a Tessara dataset revision.",
                    )
                },
            ),
        )
        .route(
            "/datasets/{dataset_id}/revisions/{revision_id}",
            get(
                |Path((dataset_id, revision_id)): Path<(String, String)>| async move {
                    native_app(
                        format!("/datasets/{dataset_id}/revisions/{revision_id}"),
                        "Dataset Revision",
                        "Inspect a dataset revision.",
                    )
                },
            ),
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
            "/administration/users",
            get(|| async {
                native_app(
                    "/administration/users",
                    "Tessara Users",
                    "Manage Tessara users.",
                )
            }),
        )
        .route("/administration/modules", get(modules::native_directory))
        .route("/administration/composition", get(composition::native_page))
        .route(
            "/administration/modules/{definition_id}",
            get(modules::native_detail),
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
        .fallback(module_gateway::dispatch)
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
        .merge(core_security::routes())
        .merge(hierarchy::routes())
        .merge(operations::routes())
        .merge(forms::routes())
        .merge(workflows::routes())
        .merge(submissions::routes())
        .merge(analytics::routes())
        .merge(datasets::routes())
        .merge(components::routes())
        .merge(composition::routes())
        .merge(dashboard_components_adapter::routes())
        .merge(modules::routes())
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

async fn module_unavailable_fallback() -> Response {
    module_unavailable_fallback_response()
}

pub(crate) fn module_unavailable_fallback_response() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Html(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Module unavailable · Tessara</title><link rel="stylesheet" href="/pkg/tessara-web.css"></head><body class="tessara-app"><main class="login-shell"><section class="login-panel blurred-surface" aria-labelledby="module-unavailable-title"><a class="login-brand" href="/" aria-label="Tessara home"><img src="/assets/tessara-icon-256.svg" alt=""><span>Tessara</span></a><div class="login-panel__header"><h1 id="module-unavailable-title">Module temporarily unavailable</h1><p>The requested module is not ready. Tessara Core and its administration surfaces remain available.</p></div><a class="button" href="/administration/modules">Open Module Management</a></section></main></body></html>"#,
        ),
    )
        .into_response()
}

fn is_protected_ui_request(request: &Request) -> bool {
    if !matches!(request.method(), &Method::GET | &Method::HEAD) {
        return false;
    }

    let path = request.uri().path();
    !(path == "/"
        || path == "/login"
        || path == "/enrollment"
        || path == "/health"
        || path == "/api"
        || path.starts_with("/api/")
        || path == "/assets"
        || path.starts_with("/assets/")
        || path == "/pkg"
        || path.starts_with("/pkg/")
        || path == "/_tessara"
        || path.starts_with("/_tessara/"))
}

async fn static_asset(Path(asset_name): Path<String>) -> impl IntoResponse {
    #[cfg(feature = "ssr")]
    match tessara_web::static_asset(&asset_name) {
        Some((content, content_type)) => {
            ([(header::CONTENT_TYPE, content_type)], content).into_response()
        }
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
