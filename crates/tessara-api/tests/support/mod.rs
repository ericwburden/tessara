use std::sync::LazyLock;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tessara_api::{config::Config, db, router};
use tower::ServiceExt;
use tracing_subscriber::EnvFilter;

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

pub async fn test_app() -> Option<axum::Router> {
    LazyLock::force(&TEST_TRACING);
    Some(router(test_state().await?))
}

pub async fn workflow_node_type_id_for_slug(app: axum::Router, token: &str, slug: &str) -> String {
    let workflows = request_json(
        app,
        authorized_request("GET", "/api/workflows", token, None),
    )
    .await;
    workflows
        .as_array()
        .expect("workflow list should be an array")
        .iter()
        .find(|workflow| workflow["slug"] == slug)
        .and_then(|workflow| workflow["workflow_node_type_id"].as_str())
        .unwrap_or_else(|| panic!("workflow {slug} should expose a node type id"))
        .to_string()
}

pub async fn node_type_id_for_slug(app: axum::Router, token: &str, slug: &str) -> String {
    let node_types = request_json(
        app,
        authorized_request("GET", "/api/admin/node-types", token, None),
    )
    .await;
    node_types
        .as_array()
        .expect("node type list should be an array")
        .iter()
        .find(|node_type| node_type["slug"] == slug)
        .and_then(|node_type| node_type["id"].as_str())
        .unwrap_or_else(|| panic!("node type {slug} should be present"))
        .to_string()
}

pub async fn create_publishable_form(
    app: axum::Router,
    token: &str,
    name: &str,
    slug: &str,
    scope_node_type_id: &str,
    key_suffix: &str,
) -> (String, String) {
    let form = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/forms",
            token,
            Some(json!({
                "name": name,
                "slug": slug,
                "scope_node_type_id": scope_node_type_id
            })),
        ),
    )
    .await;
    let form_id = form["id"]
        .as_str()
        .expect("created form should expose id")
        .to_string();
    let version = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/forms/{form_id}/versions"),
            token,
            Some(json!({})),
        ),
    )
    .await;
    let version_id = version["id"]
        .as_str()
        .expect("created form revision should expose id")
        .to_string();
    add_publishable_form_contents(app, token, &version_id, key_suffix).await;

    (form_id, version_id)
}

pub async fn add_publishable_form_contents(
    app: axum::Router,
    token: &str,
    form_version_id: &str,
    key_suffix: &str,
) {
    let section = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{form_version_id}/sections"),
            token,
            Some(json!({
                "title": "Main",
                "description": "",
                "position": 0
            })),
        ),
    )
    .await;
    let section_id = section["id"]
        .as_str()
        .expect("created form section should expose id");
    request_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{form_version_id}/fields"),
            token,
            Some(json!({
                "section_id": section_id,
                "key": format!("uat_field_{key_suffix}"),
                "label": "UAT Field",
                "field_type": "text",
                "required": true,
                "position": 0,
                "grid_row": 1,
                "grid_column": 1,
                "grid_width": 12,
                "grid_height": 2
            })),
        ),
    )
    .await;
}

pub async fn current_generated_workflow_for_form(
    app: axum::Router,
    token: &str,
    form_id: &str,
) -> Value {
    let form = request_json(
        app,
        authorized_request("GET", &format!("/api/forms/{form_id}"), token, None),
    )
    .await;
    form["workflows"]
        .as_array()
        .expect("form detail should include workflows")
        .iter()
        .find(|workflow| {
            workflow["source"] == "generated_form"
                && workflow["current_status"] == "published"
                && workflow["current_version_id"].as_str().is_some()
        })
        .cloned()
        .expect("form should expose a current generated workflow")
}

pub async fn test_state() -> Option<db::AppState> {
    test_state_with_cookie_name("tessara_session").await
}

pub async fn test_state_with_cookie_name(auth_cookie_name: &str) -> Option<db::AppState> {
    LazyLock::force(&TEST_TRACING);
    let Some(database_url) = std::env::var("TEST_DATABASE_URL").ok() else {
        eprintln!("skipping database integration test; TEST_DATABASE_URL is not set");
        return None;
    };

    reset_database(&database_url).await;
    let config = Config {
        database_url,
        bind_addr: "127.0.0.1:0".to_string(),
        dev_admin_email: "admin@tessara.local".to_string(),
        dev_admin_password: "tessara-dev-admin".to_string(),
        auth_cookie_name: auth_cookie_name.to_string(),
        auth_cookie_secure: false,
        auth_session_ttl_hours: 12,
    };
    let pool = db::connect_and_prepare(&config)
        .await
        .expect("database should migrate and seed");

    Some(db::AppState { pool, config })
}

pub fn value_for_field_type(field_type: &str) -> Value {
    match field_type {
        "number" => json!(7),
        "boolean" => json!(false),
        "date" => json!("2026-05-04"),
        "multi_choice" => json!(["Sprint 2D"]),
        "single_choice" => json!("Sprint 2D"),
        _ => json!("Sprint 2D response value"),
    }
}

pub async fn save_required_values(app: axum::Router, token: &str, submission_id: &str) {
    let detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/submissions/{submission_id}"),
            token,
            None,
        ),
    )
    .await;
    let mut values = serde_json::Map::new();
    for field in detail["values"]
        .as_array()
        .expect("submission detail should include values")
        .iter()
        .filter(|field| field["required"] == true)
    {
        values.insert(
            field["key"]
                .as_str()
                .expect("field should include key")
                .to_string(),
            value_for_field_type(
                field["field_type"]
                    .as_str()
                    .expect("field should include field type"),
            ),
        );
    }
    if !values.is_empty() {
        request_json(
            app,
            authorized_request(
                "PUT",
                &format!("/api/submissions/{submission_id}/values"),
                token,
                Some(json!({ "values": values })),
            ),
        )
        .await;
    }
}

pub async fn login_token(app: axum::Router) -> String {
    login_token_for(app, "admin@tessara.local", "tessara-dev-admin").await
}

pub async fn login_token_for(app: axum::Router, email: &str, password: &str) -> String {
    let login = request_json(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "email": email,
                    "password": password
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

pub async fn login_cookie_for(app: axum::Router, email: &str, password: &str) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "email": email,
                        "password": password
                    })
                    .to_string(),
                ))
                .expect("valid login request"),
        )
        .await
        .expect("router should produce response");
    assert_eq!(response.status(), StatusCode::OK);

    response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .expect("login should set a browser session cookie")
        .to_string()
}

pub async fn request_json(app: axum::Router, request: Request<Body>) -> Value {
    let (status, body) = request_status_and_json(app, request).await;
    assert_eq!(status, StatusCode::OK, "unexpected response: {body}");
    body
}

pub async fn request_status_and_json(
    app: axum::Router,
    request: Request<Body>,
) -> (StatusCode, Value) {
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

pub fn cookie_authenticated_request(
    method: &str,
    uri: &str,
    cookie: &str,
    body: Option<Value>,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::COOKIE, cookie);

    let body = if let Some(body) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };

    builder
        .body(body)
        .expect("valid cookie-authenticated request")
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
        database_name.contains("test"),
        "TEST_DATABASE_URL must point at a disposable database; got '{database_name}'"
    );

    drop_all_public_tables(&pool).await;
    sqlx::query("DROP SCHEMA IF EXISTS analytics CASCADE")
        .execute(&pool)
        .await
        .expect("analytics schema should be droppable");
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
        .execute(&pool)
        .await
        .expect("migration table should be droppable");
    for type_name in [
        "field_type",
        "form_version_status",
        "submission_status",
        "missing_data_policy",
    ] {
        sqlx::query(&format!("DROP TYPE IF EXISTS {type_name} CASCADE"))
            .execute(&pool)
            .await
            .expect("enum type should be droppable");
    }
}

async fn drop_all_public_tables(pool: &PgPool) {
    let tables = sqlx::query_scalar::<_, String>(
        r#"
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public'
        "#,
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
