use std::sync::LazyLock;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tessara_api::{config::Config, db, router};
use tower::ServiceExt;

static TEST_DATABASE_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

#[tokio::test]
async fn demo_seed_backfills_workflows_and_form_links() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let workflows = request_json(
        app.clone(),
        authorized_request("GET", "/api/workflows", &admin_token, None),
    )
    .await;
    let linked_workflow = workflows
        .as_array()
        .expect("workflow list should be an array")
        .iter()
        .find(|workflow| workflow["form_id"] == seed["form_id"])
        .cloned()
        .expect("seeded form should expose a linked workflow");
    assert!(
        workflows
            .as_array()
            .expect("workflow list should be an array")
            .len()
            >= seed["form_count"]
                .as_u64()
                .expect("seed should report form_count") as usize
    );
    assert_eq!(linked_workflow["current_status"], "published");
    assert!(
        linked_workflow["assignment_count"]
            .as_i64()
            .expect("workflow summary should expose assignment count")
            > 0
    );

    let form_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/forms/{}",
                seed["form_id"]
                    .as_str()
                    .expect("seed should include form id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        form_detail["workflows"]
            .as_array()
            .expect("form detail should include workflows")
            .iter()
            .any(|workflow| {
                workflow["id"] == linked_workflow["id"] && workflow["current_status"] == "published"
            })
    );

    let workflow_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/workflows/{}",
                linked_workflow["id"]
                    .as_str()
                    .expect("linked workflow should expose an id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(workflow_detail["form_id"], seed["form_id"]);
    assert!(
        workflow_detail["assignments"]
            .as_array()
            .expect("workflow detail should include assignments")
            .iter()
            .any(|assignment| {
                assignment["form_id"] == seed["form_id"]
                    && assignment["is_active"] == true
                    && assignment["workflow_step_title"]
                        .as_str()
                        .is_some_and(|title| !title.trim().is_empty())
            })
    );

    let assignments = request_json(
        app.clone(),
        authorized_request("GET", "/api/workflow-assignments", &admin_token, None),
    )
    .await;
    assert!(
        assignments
            .as_array()
            .expect("assignment list should be an array")
            .iter()
            .any(|assignment| assignment["is_active"] == true)
    );
}

#[tokio::test]
async fn assignee_pending_work_can_start_workflow_response() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let respondent_token = login_token_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let pending = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/workflow-assignments/pending",
            &respondent_token,
            None,
        ),
    )
    .await;
    let assignment_id = pending[0]["workflow_assignment_id"]
        .as_str()
        .expect("pending work should include assignment id");

    let started = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{assignment_id}/start"),
            &respondent_token,
            Some(json!({})),
        ),
    )
    .await;

    let submission = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/submissions/{}",
                started["id"]
                    .as_str()
                    .expect("start response should include id")
            ),
            &respondent_token,
            None,
        ),
    )
    .await;
    assert_eq!(submission["status"], "draft");

    let drafts = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/submissions?status=draft",
            &respondent_token,
            None,
        ),
    )
    .await;
    assert!(
        drafts
            .as_array()
            .expect("draft list should be an array")
            .iter()
            .any(|draft| draft["id"] == started["id"])
    );
}

#[tokio::test]
async fn pending_work_excludes_assignments_with_existing_drafts() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let respondent_token = login_token_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let pending = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/workflow-assignments/pending",
            &respondent_token,
            None,
        ),
    )
    .await;
    let assignment_id = pending[0]["workflow_assignment_id"]
        .as_str()
        .expect("pending work should include assignment id");

    let started = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{assignment_id}/start"),
            &respondent_token,
            Some(json!({})),
        ),
    )
    .await;

    let restarted = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{assignment_id}/start"),
            &respondent_token,
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(restarted["id"], started["id"]);

    let pending_after_start = request_json(
        app,
        authorized_request(
            "GET",
            "/api/workflow-assignments/pending",
            &respondent_token,
            None,
        ),
    )
    .await;
    assert!(
        !pending_after_start
            .as_array()
            .expect("pending work should be an array")
            .iter()
            .any(|item| item["workflow_assignment_id"] == assignment_id)
    );
}

#[tokio::test]
async fn pending_work_excludes_assignments_with_submitted_responses_and_start_rejects_them() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let assignments = request_json(
        app.clone(),
        authorized_request("GET", "/api/workflow-assignments", &admin_token, None),
    )
    .await;
    let submitted_assignment = assignments
        .as_array()
        .expect("assignment list should be an array")
        .iter()
        .find(|item| {
            item["has_submitted"] == true && item["account_email"] == "respondent@tessara.local"
        })
        .cloned()
        .expect("seed should create a respondent assignment with a submitted response");
    let assignment_id = submitted_assignment["id"]
        .as_str()
        .expect("submitted assignment should include id");

    let respondent_token = login_token_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;

    let pending = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/workflow-assignments/pending",
            &respondent_token,
            None,
        ),
    )
    .await;
    assert!(
        !pending
            .as_array()
            .expect("pending work should be an array")
            .iter()
            .any(|item| item["workflow_assignment_id"] == assignment_id)
    );

    let rejected = request_status_and_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{assignment_id}/start"),
            &respondent_token,
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(rejected.0, StatusCode::BAD_REQUEST);
    assert_eq!(
        rejected.1["error"],
        "bad request: submitted workflow assignments cannot start new response work"
    );
}

#[tokio::test]
async fn starting_distinct_assignments_returns_distinct_submission_ids() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let respondent_token = login_token_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let pending = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/workflow-assignments/pending",
            &respondent_token,
            None,
        ),
    )
    .await;
    let pending_items = pending.as_array().expect("pending work should be an array");
    assert!(
        pending_items.len() >= 2,
        "seed should expose at least two pending assignments"
    );

    let first_assignment_id = pending_items[0]["workflow_assignment_id"]
        .as_str()
        .expect("pending work should include assignment id");
    let second_assignment_id = pending_items[1]["workflow_assignment_id"]
        .as_str()
        .expect("pending work should include assignment id");

    let first_started = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{first_assignment_id}/start"),
            &respondent_token,
            Some(json!({})),
        ),
    )
    .await;
    let second_started = request_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{second_assignment_id}/start"),
            &respondent_token,
            Some(json!({})),
        ),
    )
    .await;

    assert_ne!(first_started["id"], second_started["id"]);
}

#[tokio::test]
async fn workflow_assignments_can_be_deactivated() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let assignments = request_json(
        app.clone(),
        authorized_request("GET", "/api/workflow-assignments", &admin_token, None),
    )
    .await;
    let assignment = assignments[0].clone();

    let updated = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!(
                "/api/workflow-assignments/{}",
                assignment["id"]
                    .as_str()
                    .expect("assignment should include id")
            ),
            &admin_token,
            Some(json!({
                "node_id": assignment["node_id"],
                "account_id": assignment["account_id"],
                "is_active": false
            })),
        ),
    )
    .await;
    assert_eq!(updated["id"], assignment["id"]);

    let inactive = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/workflow-assignments?active=false",
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        inactive
            .as_array()
            .expect("inactive assignment list should be an array")
            .iter()
            .any(|item| item["id"] == assignment["id"])
    );
}

#[tokio::test]
async fn workflow_assignment_filters_support_reactivation_and_context_queries() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let assignments = request_json(
        app.clone(),
        authorized_request("GET", "/api/workflow-assignments", &admin_token, None),
    )
    .await;
    let assignment = assignments[0].clone();
    let assignment_id = assignment["id"]
        .as_str()
        .expect("assignment should include id");
    let workflow_id = assignment["workflow_id"]
        .as_str()
        .expect("assignment should include workflow id");
    let workflow_version_id = assignment["workflow_version_id"]
        .as_str()
        .expect("assignment should include workflow version id");
    let form_id = assignment["form_id"]
        .as_str()
        .expect("assignment should include form id");
    let node_id = assignment["node_id"]
        .as_str()
        .expect("assignment should include node id");
    let account_id = assignment["account_id"]
        .as_str()
        .expect("assignment should include account id");

    for uri in [
        format!("/api/workflow-assignments?workflow_id={workflow_id}"),
        format!("/api/workflow-assignments?form_id={form_id}"),
        format!("/api/workflow-assignments?node_id={node_id}"),
        format!("/api/workflow-assignments?account_id={account_id}"),
        "/api/workflow-assignments?active=true".to_string(),
    ] {
        let filtered = request_json(
            app.clone(),
            authorized_request("GET", &uri, &admin_token, None),
        )
        .await;
        assert!(
            filtered
                .as_array()
                .expect("filtered assignment list should be an array")
                .iter()
                .any(|item| item["id"] == assignment_id)
        );
    }

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/workflow-assignments/{assignment_id}"),
            &admin_token,
            Some(json!({
                "node_id": node_id,
                "account_id": account_id,
                "is_active": false
            })),
        ),
    )
    .await;

    let active_after_deactivate = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/workflow-assignments?active=true",
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        active_after_deactivate
            .as_array()
            .expect("active assignment list should be an array")
            .iter()
            .all(|item| item["id"] != assignment_id)
    );

    let reactivated = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflow-assignments",
            &admin_token,
            Some(json!({
                "workflow_version_id": workflow_version_id,
                "node_id": node_id,
                "account_id": account_id
            })),
        ),
    )
    .await;
    assert_eq!(reactivated["id"], assignment_id);

    let active_after_reactivate = request_json(
        app,
        authorized_request(
            "GET",
            "/api/workflow-assignments?active=true",
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        active_after_reactivate
            .as_array()
            .expect("active assignment list should be an array")
            .iter()
            .any(|item| item["id"] == assignment_id && item["workflow_id"] == workflow_id)
    );
}

#[tokio::test]
async fn delegator_can_query_pending_work_for_an_accessible_delegate_account() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let delegate_token = login_token_for(
        app.clone(),
        "delegate@tessara.local",
        "tessara-dev-delegate",
    )
    .await;
    let delegate_pending = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/workflow-assignments/pending",
            &delegate_token,
            None,
        ),
    )
    .await;
    let delegate_pending_ids = delegate_pending
        .as_array()
        .expect("delegate pending work should be an array")
        .iter()
        .filter_map(|item| item["workflow_assignment_id"].as_str())
        .collect::<Vec<_>>();
    assert!(
        !delegate_pending_ids.is_empty(),
        "demo seed should expose delegate pending work"
    );

    let delegator_token = login_token_for(
        app.clone(),
        "delegator@tessara.local",
        "tessara-dev-delegator",
    )
    .await;
    let delegator_me = request_json(
        app.clone(),
        authorized_request("GET", "/api/me", &delegator_token, None),
    )
    .await;
    let delegate_account_id = delegator_me["delegations"]
        .as_array()
        .expect("delegator should include delegations")
        .first()
        .and_then(|delegation| delegation["account_id"].as_str())
        .expect("delegator should expose delegate account id");

    let delegated_pending = request_json(
        app,
        authorized_request(
            "GET",
            &format!("/api/workflow-assignments/pending?delegate_account_id={delegate_account_id}"),
            &delegator_token,
            None,
        ),
    )
    .await;
    let delegated_pending_ids = delegated_pending
        .as_array()
        .expect("delegated pending work should be an array")
        .iter()
        .filter_map(|item| item["workflow_assignment_id"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(delegated_pending_ids, delegate_pending_ids);
}

#[tokio::test]
async fn logout_revokes_the_current_session_token() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let logout = request_json(
        app.clone(),
        authorized_request("DELETE", "/api/auth/logout", &admin_token, None),
    )
    .await;
    assert_eq!(logout["signed_out"], true);

    let me = request_status_and_json(
        app,
        authorized_request("GET", "/api/me", &admin_token, None),
    )
    .await;
    assert_eq!(me.0, StatusCode::UNAUTHORIZED);
}

async fn test_app() -> Option<axum::Router> {
    Some(router(test_state().await?))
}

async fn test_state() -> Option<db::AppState> {
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
    };
    let pool = db::connect_and_prepare(&config)
        .await
        .expect("database should migrate and seed");

    Some(db::AppState { pool, config })
}

async fn login_token(app: axum::Router) -> String {
    login_token_for(app, "admin@tessara.local", "tessara-dev-admin").await
}

async fn login_token_for(app: axum::Router, email: &str, password: &str) -> String {
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

async fn request_json(app: axum::Router, request: Request<Body>) -> Value {
    let (status, body) = request_status_and_json(app, request).await;
    assert_eq!(status, StatusCode::OK, "unexpected response: {body}");
    body
}

async fn request_status_and_json(app: axum::Router, request: Request<Body>) -> (StatusCode, Value) {
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
