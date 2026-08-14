use std::sync::LazyLock;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tessara_api::{config::Config, db};
use tracing_subscriber::EnvFilter;

#[path = "../support/database_safety.rs"]
mod database_safety;

use database_safety::{DISPOSABLE_DATABASE_NAME_TOKENS, is_disposable_database_name};

pub static TEST_DATABASE_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));
static TEST_TRACING: LazyLock<()> = LazyLock::new(|| {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("tessara_api=debug,sqlx=warn")),
        )
        .with_test_writer()
        .try_init();
});

pub async fn test_state() -> db::AppState {
    LazyLock::force(&TEST_TRACING);
    let database_url = std::env::var("TEST_API_DATABASE_URL")
        .expect("TEST_API_DATABASE_URL is required; database integration tests must never skip");
    assert!(
        !database_url.trim().is_empty(),
        "TEST_API_DATABASE_URL is required and must not be empty"
    );
    reset_database(&database_url).await;
    let config = Config {
        database_url,
        installation_id: None,
        bind_addr: "127.0.0.1:0".to_string(),
        dev_admin_email: "admin@tessara.local".to_string(),
        dev_admin_password: "tessara-dev-admin".to_string(),
        auth_cookie_name: "tessara_session".to_string(),
        auth_cookie_secure: false,
        auth_session_ttl_hours: 12,
    };
    let pool = db::connect_and_prepare(&config)
        .await
        .expect("database should migrate and seed");
    db::AppState { pool, config }
}

pub async fn login_token(app: axum::Router) -> String {
    let login = request_json(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "email": "admin@tessara.local",
                    "password": "tessara-dev-admin"
                })
                .to_string(),
            ))
            .expect("valid login request"),
    )
    .await;
    login["token"]
        .as_str()
        .expect("login response should contain token")
        .to_string()
}

async fn request_json(app: axum::Router, request: Request<Body>) -> Value {
    let (status, body) = request_status_and_json(app, request).await;
    assert_eq!(status, StatusCode::OK, "unexpected response: {body}");
    body
}

pub async fn request_status_and_json(
    app: axum::Router,
    request: Request<Body>,
) -> (StatusCode, Value) {
    use tower::ServiceExt as _;

    let response = app
        .oneshot(request)
        .await
        .expect("router should produce response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    (
        status,
        serde_json::from_slice(&body).unwrap_or_else(|_| {
            panic!(
                "response should be JSON: {}",
                String::from_utf8_lossy(&body)
            )
        }),
    )
}

pub fn authorized_request(
    method: &str,
    uri: &str,
    token: &str,
    body: Option<Value>,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"));
    let body = if let Some(body) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    builder.body(body).expect("valid authorized request")
}

async fn reset_database(database_url: &str) {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await
        .expect("test database should be reachable");
    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("current database should be readable");
    assert!(
        is_disposable_database_name(&database_name),
        "TEST_API_DATABASE_URL must point at a database with a token-bounded disposable name marker ({}); got '{database_name}'",
        DISPOSABLE_DATABASE_NAME_TOKENS.join(", ")
    );
    drop_all_public_tables(&pool).await;
    drop_all_public_functions(&pool).await;
    sqlx::query("DROP SCHEMA IF EXISTS analytics CASCADE")
        .execute(&pool)
        .await
        .expect("analytics schema should be droppable");
    sqlx::query("DROP SCHEMA IF EXISTS dataset_materialized CASCADE")
        .execute(&pool)
        .await
        .expect("dataset materialized schema should be droppable");
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
        .execute(&pool)
        .await
        .expect("migration table should be droppable");
    for type_name in [
        "field_type",
        "form_version_status",
        "submission_status",
        "dataset_revision_status",
        "component_type",
        "component_version_status",
        "component_lifecycle_state",
        "component_change_category",
        "missing_data_policy",
    ] {
        sqlx::query(&format!("DROP TYPE IF EXISTS {type_name} CASCADE"))
            .execute(&pool)
            .await
            .expect("enum type should be droppable");
    }
}

async fn drop_all_public_functions(pool: &PgPool) {
    sqlx::query(
        r#"
        DO $reset$
        DECLARE candidate record;
        BEGIN
            FOR candidate IN
                SELECT function.oid::regprocedure::text AS identity
                  FROM pg_proc function
                  JOIN pg_namespace namespace ON namespace.oid=function.pronamespace
                 WHERE namespace.nspname='public'
                   AND NOT EXISTS (
                       SELECT 1 FROM pg_depend dependency
                        WHERE dependency.classid='pg_proc'::regclass
                          AND dependency.objid=function.oid
                          AND dependency.deptype='e'
                   )
            LOOP
                EXECUTE format('DROP FUNCTION IF EXISTS %s CASCADE', candidate.identity);
            END LOOP;
        END
        $reset$;
        "#,
    )
    .execute(pool)
    .await
    .expect("non-extension public functions should be droppable");
}

async fn drop_all_public_tables(pool: &PgPool) {
    let tables = sqlx::query_scalar::<_, String>(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public'",
    )
    .fetch_all(pool)
    .await
    .expect("public tables should be listable");
    for table in tables {
        sqlx::query(&format!("DROP TABLE IF EXISTS public.{table} CASCADE"))
            .execute(pool)
            .await
            .expect("public table should be droppable");
    }
}
