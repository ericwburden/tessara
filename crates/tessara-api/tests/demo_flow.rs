use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tessara_api::{config::Config, db, router};
use tower::ServiceExt;

#[tokio::test]
async fn demo_seed_report_and_dashboard_flow_works_against_database() {
    let Some(database_url) = std::env::var("TEST_DATABASE_URL").ok() else {
        eprintln!("skipping database integration test; TEST_DATABASE_URL is not set");
        return;
    };

    reset_database(&database_url).await;
    let config = Config {
        database_url,
        bind_addr: "127.0.0.1:0".to_string(),
        dev_admin_email: "admin@tessara.local".to_string(),
        dev_admin_password: "tessara-dev-admin".to_string(),
    };
    let pool = db::connect_and_prepare(&config)
        .await
        .expect("database should migrate and seed");
    let state = db::AppState { pool, config };
    let app = router(state);

    let login = request_json(
        app.clone(),
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
    let token = login["token"]
        .as_str()
        .expect("login response should contain token");

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", token, None),
    )
    .await;
    assert_eq!(seed["analytics_values"], 1);

    let report_id = seed["report_id"]
        .as_str()
        .expect("seed response should contain report id");
    let report = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{report_id}/table"),
            token,
            None,
        ),
    )
    .await;
    assert_eq!(report["rows"][0]["field_value"], "42");

    let dashboard_id = seed["dashboard_id"]
        .as_str()
        .expect("seed response should contain dashboard id");
    let dashboard = request_json(
        app,
        Request::builder()
            .method("GET")
            .uri(format!("/api/dashboards/{dashboard_id}"))
            .body(Body::empty())
            .expect("valid dashboard request"),
    )
    .await;
    assert_eq!(dashboard["components"][0]["chart"]["report_id"], report_id);
}

async fn request_json(app: axum::Router, request: Request<Body>) -> Value {
    let response = app
        .oneshot(request)
        .await
        .expect("router should produce response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    assert_eq!(
        status,
        StatusCode::OK,
        "unexpected response: {}",
        String::from_utf8_lossy(&body)
    );
    serde_json::from_slice(&body).expect("response should be JSON")
}

fn authorized_request(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
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
