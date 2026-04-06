mod analytics;
mod auth;
mod config;
mod db;
mod error;
mod forms;
mod hierarchy;
mod reporting;
mod submissions;

use std::net::SocketAddr;

use axum::{
    Router,
    routing::{get, post, put},
};
use config::Config;
use db::AppState;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "tessara_api=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let pool = db::connect_and_prepare(&config).await?;
    let state = AppState { pool, config };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/auth/login", post(auth::login))
        .route("/api/me", get(auth::me))
        .route("/api/admin/node-types", post(hierarchy::create_node_type))
        .route(
            "/api/admin/node-type-relationships",
            post(hierarchy::create_node_type_relationship),
        )
        .route(
            "/api/admin/node-metadata-fields",
            post(hierarchy::create_node_metadata_field),
        )
        .route("/api/admin/nodes", post(hierarchy::create_node))
        .route("/api/nodes", get(hierarchy::list_nodes))
        .route("/api/admin/forms", post(forms::create_form))
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
            "/api/admin/form-versions/{form_version_id}/publish",
            post(forms::publish_form_version),
        )
        .route(
            "/api/form-versions/{form_version_id}/render",
            get(forms::render_form_version),
        )
        .route("/api/submissions/drafts", post(submissions::create_draft))
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
        .route("/api/admin/reports", post(reporting::create_report))
        .route("/api/reports/{report_id}/table", get(reporting::run_report))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    let addr: SocketAddr = state.config.bind_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "starting tessara api");
    axum::serve(listener, app).await?;

    Ok(())
}
