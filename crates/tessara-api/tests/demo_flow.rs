use std::sync::LazyLock;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use tessara_api::{config::Config, db, router};
use tower::ServiceExt;
use tracing_subscriber::EnvFilter;

#[path = "support/database_safety.rs"]
mod database_safety;

use database_safety::{DISPOSABLE_DATABASE_NAME_TOKENS, is_disposable_database_name};

static TEST_DATABASE_LOCK: LazyLock<tokio::sync::Mutex<()>> =
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

#[tokio::test]
async fn demo_seed_requires_an_empty_domain_database() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    assert_eq!(seed["seed_version"], "uat-demo-v2");

    let (status, body) = request_status_and_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "bad_request");
    assert!(
        body["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("requires an empty database")
    );
}

#[tokio::test]
async fn core_seed_catalog_excludes_independent_module_capabilities() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", &admin_token, None),
    )
    .await;
    let keys = capabilities
        .as_array()
        .expect("capabilities should be an array")
        .iter()
        .map(|capability| capability["key"].as_str().expect("capability key"))
        .collect::<Vec<_>>();
    assert!(!keys.contains(&"datasets:read"));
    assert!(!keys.contains(&"datasets:manage"));
    assert!(keys.contains(&"operations:view"));
    assert!(!keys.contains(&"components:read"));
    assert!(!keys.contains(&"components:manage"));
    assert!(!keys.contains(&"dashboards:read"));
    assert!(!keys.contains(&"dashboards:manage"));
}

#[tokio::test]
async fn capability_catalog_and_role_detail_expose_scope_and_durable_provenance() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", &admin_token, None),
    )
    .await;
    let capability_items = capabilities.as_array().expect("capability catalog");
    let forms_read = capability_items
        .iter()
        .find(|capability| capability["key"] == "forms:read")
        .expect("forms:read capability");
    assert_eq!(forms_read["scope_mode"], "scope_aware");
    let forms_provenance = forms_read["provenance"]
        .as_array()
        .expect("forms:read provenance");
    assert_eq!(forms_provenance.len(), 2);
    assert_eq!(forms_provenance[0]["source_kind"], "core");
    assert_eq!(forms_provenance[0]["source_key"], "core");
    assert_eq!(forms_provenance[0]["provider_state"], "core_authoritative");
    assert!(forms_provenance[0]["source_digest"].is_null());
    assert_eq!(
        forms_provenance[1]["source_kind"],
        "transition_contribution"
    );
    assert_eq!(forms_provenance[1]["definition_id"], "tessara.forms");
    assert_eq!(forms_provenance[1]["definition_display_name"], "Forms");
    assert_eq!(
        forms_provenance[1]["provider_state"],
        "transitional_in_process"
    );
    assert_eq!(
        forms_provenance[1]["source_digest"],
        "sha256:71bebdd07ff0028cc0da8bbd9707c393bade9951e5cedb265a4b8465d54b493e"
    );

    let modules_read = capability_items
        .iter()
        .find(|capability| capability["key"] == "modules:read")
        .expect("modules:read capability");
    assert_eq!(modules_read["scope_mode"], "installation_global");
    let module_provenance = modules_read["provenance"]
        .as_array()
        .expect("modules:read provenance");
    assert_eq!(module_provenance.len(), 1);
    assert_eq!(module_provenance[0]["source_kind"], "core");
    assert_eq!(module_provenance[0]["provider_state"], "core_authoritative");

    let roles = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/roles", &admin_token, None),
    )
    .await;
    let operator_role_id = roles
        .as_array()
        .expect("role catalog")
        .iter()
        .find(|role| role["name"] == "operator")
        .and_then(|role| role["id"].as_str())
        .expect("operator role id");
    let operator_role = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/roles/{operator_role_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let role_forms_read = operator_role["capabilities"]
        .as_array()
        .expect("role capabilities")
        .iter()
        .find(|capability| capability["key"] == "forms:read")
        .expect("role forms:read capability");
    assert_eq!(role_forms_read["scope_mode"], "scope_aware");
    assert_eq!(role_forms_read["provenance"], forms_read["provenance"]);
}

async fn test_app() -> axum::Router {
    LazyLock::force(&TEST_TRACING);
    let database_url = std::env::var("TEST_API_DATABASE_URL")
        .expect("TEST_API_DATABASE_URL is required; database integration tests must never skip");
    assert!(
        !database_url.trim().is_empty(),
        "TEST_API_DATABASE_URL is required and must not be empty"
    );
    let reset_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect test database");
    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&reset_pool)
        .await
        .expect("current database should be readable");
    assert!(
        is_disposable_database_name(&database_name),
        "TEST_API_DATABASE_URL must point at a database with a token-bounded disposable name marker ({}); got '{database_name}'",
        DISPOSABLE_DATABASE_NAME_TOKENS.join(", ")
    );
    sqlx::query("DROP SCHEMA public CASCADE")
        .execute(&reset_pool)
        .await
        .expect("drop test database schema");
    sqlx::query("DROP SCHEMA IF EXISTS analytics CASCADE")
        .execute(&reset_pool)
        .await
        .expect("drop analytics schema");
    sqlx::query("CREATE SCHEMA public")
        .execute(&reset_pool)
        .await
        .expect("create test database schema");
    reset_pool.close().await;
    let config = Config {
        database_url,
        installation_id: None,
        bind_addr: "127.0.0.1:0".into(),
        dev_admin_email: "admin@tessara.local".into(),
        dev_admin_password: "tessara-dev-admin".into(),
        auth_cookie_name: "tessara_session".into(),
        auth_cookie_secure: false,
        auth_session_ttl_hours: 12,
    };
    let pool = db::connect_and_prepare(&config)
        .await
        .expect("prepare database");
    router(db::AppState { pool, config })
}

async fn login_token_for(app: axum::Router, email: &str, password: &str) -> String {
    let response = request_json(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "email": email, "password": password }).to_string(),
            ))
            .expect("valid login request"),
    )
    .await;
    response["token"]
        .as_str()
        .expect("login response should include token")
        .to_string()
}

async fn request_json(app: axum::Router, request: Request<Body>) -> Value {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let (status, body) = request_status_and_json(app, request).await;
    assert!(
        status.is_success(),
        "expected success status for {method} {uri}, got {status}: {body}"
    );
    body
}

async fn request_status_and_json(app: axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.oneshot(request).await.expect("request should succeed");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1_000_000)
        .await
        .expect("read response body");
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            panic!(
                "response should be json, status {status}, body {}",
                String::from_utf8_lossy(&bytes)
            )
        })
    };
    (status, body)
}

fn authorized_request(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(match body {
            Some(body) => Body::from(body.to_string()),
            None => Body::empty(),
        })
        .expect("valid authorized request")
}
