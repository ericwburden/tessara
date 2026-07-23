use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, postgres::PgPoolOptions};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Debug, FromRow, Serialize)]
struct ScopedRecord {
    id: Uuid,
    label: String,
    scope: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateRecord {
    label: String,
    scope: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await?;
    if env::args().nth(1).as_deref() == Some("migrate") {
        sqlx::migrate!().run(&pool).await?;
        return Ok(());
    }
    let address: SocketAddr = env::var("SCOPED_RECORDS_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8090".into())
        .parse()?;
    let app = Router::new()
        .route("/", get(index))
        .route("/admin", get(admin))
        .route("/api/records", get(list_records).post(create_record))
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/diagnostics", get(diagnostics))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { pool });
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "scoped records module listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let records = fetch_records(&state.pool).await.unwrap_or_default();
    let rows = records
        .into_iter()
        .map(|record| {
            format!(
                "<tr><th scope=\"row\"><code>{}</code></th><td>{}</td><td>{}</td></tr>",
                record.id,
                escape(&record.label),
                escape(&record.scope)
            )
        })
        .collect::<String>();
    Html(page(
        "Scoped Records",
        &format!(
            "<main><h1>Scoped Records</h1><p>Records owned by the independently deployed reference module.</p><table><thead><tr><th>ID</th><th>Label</th><th>Scope</th></tr></thead><tbody>{rows}</tbody></table></main>"
        ),
    ))
}

async fn admin() -> Html<String> {
    Html(page(
        "Scoped Records administration",
        "<main><h1>Scoped Records administration</h1><p>Use the versioned API to create records. Deployment lifecycle remains host-owned.</p></main>",
    ))
}

async fn list_records(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(axum::Json(fetch_records(&state.pool).await?))
}

async fn create_record(
    State(state): State<AppState>,
    axum::Json(input): axum::Json<CreateRecord>,
) -> Result<impl IntoResponse, ApiError> {
    if input.label.trim().is_empty() || input.scope.trim().is_empty() {
        return Err(ApiError::bad_request("label and scope are required"));
    }
    let record = sqlx::query_as::<_, ScopedRecord>("INSERT INTO scoped_records (id, label, scope) VALUES ($1, $2, $3) RETURNING id, label, scope, created_at")
        .bind(Uuid::new_v4()).bind(input.label.trim()).bind(input.scope.trim()).fetch_one(&state.pool).await?;
    Ok((StatusCode::CREATED, axum::Json(record)))
}

async fn live() -> StatusCode {
    StatusCode::NO_CONTENT
}
async fn ready(State(state): State<AppState>) -> StatusCode {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(1) => StatusCode::NO_CONTENT,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    }
}
async fn diagnostics(State(state): State<AppState>) -> impl IntoResponse {
    let database = if sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok()
    {
        "available"
    } else {
        "unavailable"
    };
    axum::Json(
        serde_json::json!({"schema_version": 1, "module": "tessara.reference.scoped-records", "database": database}),
    )
}

async fn fetch_records(pool: &PgPool) -> Result<Vec<ScopedRecord>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, label, scope, created_at FROM scoped_records ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

fn page(title: &str, body: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title} · Tessara</title><style>body{{font-family:system-ui;margin:0;background:#0f172a;color:#e2e8f0}}main{{max-width:70rem;margin:auto;padding:3rem}}table{{width:100%;border-collapse:collapse}}th,td{{padding:.75rem;text-align:left;border-bottom:1px solid #475569}}</style></head><body>{body}</body></html>"
    )
}
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

struct ApiError {
    status: StatusCode,
    message: String,
}
impl ApiError {
    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "database request failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "request failed".into(),
        }
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::json!({"error": self.message}).to_string(),
        )
            .into_response()
    }
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}
