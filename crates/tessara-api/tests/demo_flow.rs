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
    assert!(
        dashboard["components"]
            .as_array()
            .expect("dashboard components should be an array")
            .iter()
            .any(|component| component["component_version_id"] == seed["component_version_id"])
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
        !operator_me["scope_nodes"]
            .as_array()
            .expect("operator should have scoped nodes")
            .is_empty()
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
            .all(|submission| submission["assigned_to_display_name"]
                == respondent_me["display_name"])
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
    assert!(keys.contains(&"operations:view"));
}

#[tokio::test]
async fn admin_dataset_query_designer_materializes_generated_sql() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let form_id = seed["form_id"].as_str().expect("seed form id");
    let form_version_id = seed["form_version_id"]
        .as_str()
        .expect("seed form version id");
    let visibility_node_id = seed["partner_node_id"]
        .as_str()
        .expect("seed partner node id");
    let rendered_form = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/form-versions/{form_version_id}/render"),
            &admin_token,
            None,
        ),
    )
    .await;
    let field = &rendered_form["sections"]
        .as_array()
        .expect("rendered sections")[0]["fields"]
        .as_array()
        .expect("rendered fields")[0];
    let field_key = field["key"].as_str().expect("field key");
    let field_label = field["label"].as_str().expect("field label");
    let payload = json!({
        "name": "Query Designer Test Dataset",
        "slug": "query-designer-test-dataset",
        "grain": "submission",
        "composition_mode": "union_all",
        "visibility_node_ids": [visibility_node_id],
        "definition_ast": {
            "kind": "form",
            "alias": "form_a",
            "form_id": form_id,
            "form_version_major": null,
            "selection_rule": "latest"
        },
        "fields": [{
            "key": field_key,
            "label": field_label,
            "source_alias": "form_a",
            "source_field_key": field_key,
            "position": 0
        }]
    });
    let mut legacy_payload = payload.clone();
    legacy_payload["sources"] = json!([{
        "source_alias": "form_a",
        "form_id": form_id,
        "form_version_major": null,
        "selection_rule": "latest"
    }]);
    let legacy_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(legacy_payload),
        ),
    )
    .await;
    assert!(
        !legacy_status.is_success(),
        "legacy flat source payloads should be rejected"
    );

    let created = request_json(
        app.clone(),
        authorized_request("POST", "/api/admin/datasets", &admin_token, Some(payload)),
    )
    .await;
    let dataset_id = created["id"].as_str().expect("created dataset id");
    let detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(detail["definition_ast"]["kind"], "form");
    assert!(detail["generated_sql"].as_str().is_some_and(|sql| {
        sql.contains("analytics.submission_fact") && sql.contains(field_key)
    }));
    assert!(
        detail["materialized_table"]
            .as_str()
            .is_some_and(|table| table.starts_with("dataset_"))
    );
    assert!(
        detail["materialized_row_count"]
            .as_i64()
            .is_some_and(|count| count > 0)
    );

    let table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        table["rows"]
            .as_array()
            .expect("preview rows should be an array")
            .iter()
            .any(|row| row["values"].get(field_key).is_some())
    );

    let aggregation_payload = json!({
        "name": "Query Designer Aggregated Dataset",
        "slug": "query-designer-aggregated-dataset",
        "grain": "submission",
        "composition_mode": "union_all",
        "visibility_node_ids": [visibility_node_id],
        "definition_ast": {
            "kind": "form",
            "alias": "form_a",
            "form_id": form_id,
            "form_version_major": null,
            "selection_rule": "latest"
        },
        "fields": [
            {
                "key": "node_id",
                "label": "Attached Node ID",
                "source_alias": "form_a",
                "source_field_key": "__node_id",
                "position": 0
            },
            {
                "key": field_key,
                "label": field_label,
                "source_alias": "form_a",
                "source_field_key": field_key,
                "position": 1
            }
        ],
        "aggregation": {
            "group_fields": ["node_id"],
            "metrics": [{
                "key": "response_count",
                "label": "Response Count",
                "function": "count_rows",
                "source_field_key": null,
                "position": 0
            }],
            "row_picker": {
                "sort_fields": [{
                    "field_key": field_key,
                    "direction": "lowest",
                    "position": 0
                }]
            }
        }
    });
    let preview = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(aggregation_payload.clone()),
        ),
    )
    .await;
    assert!(
        preview["generated_sql"]
            .as_str()
            .is_some_and(|sql| sql.contains("GROUP BY __node_id, __node_name"))
    );
    let mut invalid_average_payload = aggregation_payload.clone();
    let created_aggregation = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(aggregation_payload),
        ),
    )
    .await;
    let aggregated_dataset_id = created_aggregation["id"]
        .as_str()
        .expect("created aggregated dataset id");
    let aggregated_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{aggregated_dataset_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        aggregated_detail["aggregation"]["scope_mode"].as_str(),
        Some("row_scoped")
    );
    assert!(
        aggregated_detail["output_fields"]
            .as_array()
            .expect("output fields")
            .iter()
            .any(|field| field["key"] == "response_count")
    );
    let aggregated_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{aggregated_dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        aggregated_table["rows"]
            .as_array()
            .expect("aggregated preview rows")
            .iter()
            .any(|row| row["values"].get("response_count").is_some())
    );

    invalid_average_payload["name"] = json!("Invalid Average Dataset");
    invalid_average_payload["slug"] = json!("invalid-average-dataset");
    invalid_average_payload["aggregation"]["metrics"] = json!([{
        "key": "average_text",
        "label": "Average Text",
        "function": "average",
        "source_field_key": field_key,
        "position": 0
    }]);
    let invalid_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(invalid_average_payload),
        ),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::BAD_REQUEST);
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

async fn request_status(app: axum::Router, request: Request<Body>) -> StatusCode {
    app.oneshot(request)
        .await
        .expect("request should succeed")
        .status()
}
