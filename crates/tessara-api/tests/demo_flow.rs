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
async fn demo_seed_uses_capability_scope_ownership_and_components() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    assert_eq!(seed["seed_version"], "uat-demo-v1");
    assert_eq!(seed["dataset_count"], 4);
    assert_eq!(seed["dataset_revision_count"], 4);
    assert_eq!(seed["component_count"], 4);
    assert_eq!(seed["dashboard_count"], 1);

    let summary = request_json(
        app.clone(),
        authorized_request("GET", "/api/summary", &admin_token, None),
    )
    .await;
    assert_eq!(summary["datasets"], 4);
    assert_eq!(summary["dataset_revisions"], 4);
    assert_eq!(summary["components"], 4);
    assert_eq!(summary["component_versions"], 4);
    assert_eq!(summary["dashboards"], 1);
    assert!(summary.get("reports").is_none());
    assert!(summary.get("charts").is_none());

    let components = request_json(
        app.clone(),
        authorized_request("GET", "/api/components", &admin_token, None),
    )
    .await;
    assert!(
        components
            .as_array()
            .expect("components should be an array")
            .iter()
            .any(|component| component["current_version_id"] == seed["component_version_id"])
    );

    let dashboard = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/dashboards/{}",
                seed["dashboard_id"].as_str().expect("dashboard id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        dashboard["components"]
            .as_array()
            .expect("dashboard components should be an array")
            .len(),
        4
    );
    assert_eq!(
        dashboard["components"][0]["component_version_id"],
        seed["component_version_id"]
    );

    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    let operator_me = request_json(
        app.clone(),
        authorized_request("GET", "/api/me", &operator_token, None),
    )
    .await;
    assert!(
        operator_me["capabilities"]
            .as_array()
            .expect("capabilities should be an array")
            .iter()
            .any(|capability| capability == "forms:read")
    );
    assert!(
        operator_me["capability_scopes"]
            .as_array()
            .expect("capability scopes should be an array")
            .iter()
            .any(|scope| scope["capability"] == "forms:read"
                && scope["scope"]["scope_type"] == "scoped")
    );
    assert!(
        operator_me["scope_nodes"]
            .as_array()
            .expect("operator should have scoped nodes")
            .len()
            >= 1
    );

    let respondent_token = login_token_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;
    let respondent_me = request_json(
        app.clone(),
        authorized_request("GET", "/api/me", &respondent_token, None),
    )
    .await;
    assert!(
        respondent_me["capabilities"]
            .as_array()
            .expect("capabilities should be an array")
            .iter()
            .any(|capability| capability == "submissions:read_own")
    );

    let respondent_submissions = request_json(
        app.clone(),
        authorized_request("GET", "/api/submissions", &respondent_token, None),
    )
    .await;
    assert!(
        respondent_submissions
            .as_array()
            .expect("respondent submissions should be an array")
            .iter()
            .all(|submission| submission["assignment_account_id"] == respondent_me["account_id"])
    );
}

#[tokio::test]
async fn seeded_capability_catalog_uses_components_and_dashboards() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
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
    assert!(keys.contains(&"datasets:read"));
    assert!(keys.contains(&"components:read"));
    assert!(keys.contains(&"dashboards:read"));
}

async fn test_app() -> Option<axum::Router> {
    LazyLock::force(&TEST_TRACING);
    let database_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("skipping database integration test because TEST_DATABASE_URL is not set");
            return None;
        }
    };
    let reset_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect test database");
    sqlx::query("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
        .execute(&reset_pool)
        .await
        .expect("reset test database");
    reset_pool.close().await;
    let config = Config {
        database_url,
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
    Some(router(db::AppState { pool, config }))
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
    let (status, body) = request_status_and_json(app, request).await;
    assert!(
        status.is_success(),
        "expected success status, got {status}: {body}"
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
