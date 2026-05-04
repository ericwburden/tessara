use std::sync::LazyLock;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
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

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
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
    let assignment_id = pending[0]["workflow_assignment_id"]
        .as_str()
        .expect("pending work should include assignment id");
    assert!(
        pending[0]["workflow_description"]
            .as_str()
            .expect("pending work should include workflow description")
            .contains("Sprint 2A runtime compatibility")
    );

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
    let draft = drafts
        .as_array()
        .expect("draft list should be an array")
        .iter()
        .find(|draft| draft["id"] == started["id"])
        .expect("started draft should be included in the draft list");
    assert!(
        draft["workflow_description"]
            .as_str()
            .expect("draft summary should include workflow description")
            .contains("Sprint 2A runtime compatibility")
    );
}

#[tokio::test]
async fn response_lifecycle_saves_resumes_submits_and_locks_submitted_records() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
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
    let submission_id = started["id"]
        .as_str()
        .expect("start response should include submission id");

    let missing_required_submit = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/submissions/{submission_id}/submit"),
            &respondent_token,
            None,
        ),
    )
    .await;
    assert_eq!(missing_required_submit.0, StatusCode::BAD_REQUEST);
    assert_eq!(missing_required_submit.1["code"], "bad_request");
    assert!(
        missing_required_submit.1["error"]
            .as_str()
            .unwrap_or_default()
            .contains("required field")
    );

    let draft_after_rejection = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/submissions/{submission_id}"),
            &respondent_token,
            None,
        ),
    )
    .await;
    assert_eq!(draft_after_rejection["status"], "draft");
    assert_eq!(draft_after_rejection["submitted_at"], Value::Null);

    let mut values = serde_json::Map::new();
    let required_fields = draft_after_rejection["values"]
        .as_array()
        .expect("submission detail should include fields")
        .iter()
        .filter(|field| field["required"] == true)
        .collect::<Vec<_>>();
    assert!(
        !required_fields.is_empty(),
        "demo pending response should include required fields"
    );
    for field in required_fields {
        values.insert(
            field["key"]
                .as_str()
                .expect("field should include stable key")
                .to_string(),
            value_for_field_type(
                field["field_type"]
                    .as_str()
                    .expect("field should include field type"),
            ),
        );
    }

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/submissions/{submission_id}/values"),
            &respondent_token,
            Some(json!({ "values": values })),
        ),
    )
    .await;

    let resumed = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/submissions/{submission_id}"),
            &respondent_token,
            None,
        ),
    )
    .await;
    assert_eq!(resumed["status"], "draft");
    assert!(
        resumed["audit_events"]
            .as_array()
            .expect("audit events should be present")
            .iter()
            .any(|event| event["event_type"] == "save_draft")
    );
    assert!(
        resumed["values"]
            .as_array()
            .expect("values should be present")
            .iter()
            .any(|value| value["required"] == true && value["value"] != Value::Null)
    );

    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/submissions/{submission_id}/submit"),
            &respondent_token,
            None,
        ),
    )
    .await;
    let submitted = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/submissions/{submission_id}"),
            &respondent_token,
            None,
        ),
    )
    .await;
    assert_eq!(submitted["status"], "submitted");
    assert_ne!(submitted["submitted_at"], Value::Null);
    assert!(
        submitted["audit_events"]
            .as_array()
            .expect("audit events should be present")
            .iter()
            .any(|event| event["event_type"] == "submit")
    );

    for request in [
        authorized_request(
            "PUT",
            &format!("/api/submissions/{submission_id}/values"),
            &respondent_token,
            Some(json!({ "values": {} })),
        ),
        authorized_request(
            "POST",
            &format!("/api/submissions/{submission_id}/submit"),
            &respondent_token,
            None,
        ),
        authorized_request(
            "DELETE",
            &format!("/api/submissions/{submission_id}"),
            &respondent_token,
            None,
        ),
    ] {
        let rejected = request_status_and_json(app.clone(), request).await;
        assert_eq!(rejected.0, StatusCode::BAD_REQUEST);
        assert_eq!(rejected.1["code"], "bad_request");
    }
}

#[tokio::test]
async fn scoped_operator_cannot_review_out_of_scope_submission_by_uuid() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;

    let operator_nodes = request_json(
        app.clone(),
        authorized_request("GET", "/api/nodes?q=Demo", &operator_token, None),
    )
    .await;
    let operator_node_ids = operator_nodes
        .as_array()
        .expect("operator nodes should be an array")
        .iter()
        .filter_map(|node| node["id"].as_str())
        .collect::<Vec<_>>();
    assert!(!operator_node_ids.is_empty());

    let all_submissions = request_json(
        app.clone(),
        authorized_request("GET", "/api/submissions", &admin_token, None),
    )
    .await;
    let out_of_scope_submission = all_submissions
        .as_array()
        .expect("submission list should be an array")
        .iter()
        .find(|submission| {
            submission["node_id"]
                .as_str()
                .is_some_and(|node_id| !operator_node_ids.contains(&node_id))
        })
        .expect("demo seed should expose out-of-scope response work");
    let out_of_scope_submission_id = out_of_scope_submission["id"]
        .as_str()
        .expect("submission should expose id");

    let rejected = request_status_and_json(
        app,
        authorized_request(
            "GET",
            &format!("/api/submissions/{out_of_scope_submission_id}"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(rejected.0, StatusCode::FORBIDDEN);
    assert_eq!(rejected.1["code"], "forbidden");
}

#[tokio::test]
async fn response_lifecycle_endpoints_accept_configured_cookie_name() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(state) = test_state_with_cookie_name("custom_tessara_session").await else {
        return;
    };
    let app = router(state);
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let respondent_cookie = login_cookie_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;
    assert!(respondent_cookie.starts_with("custom_tessara_session="));

    let pending = request_json(
        app.clone(),
        cookie_authenticated_request(
            "GET",
            "/api/workflow-assignments/pending",
            &respondent_cookie,
            None,
        ),
    )
    .await;
    let assignment_id = pending[0]["workflow_assignment_id"]
        .as_str()
        .expect("pending work should include assignment id");

    let started = request_json(
        app.clone(),
        cookie_authenticated_request(
            "POST",
            &format!("/api/workflow-assignments/{assignment_id}/start"),
            &respondent_cookie,
            Some(json!({})),
        ),
    )
    .await;
    let submission_id = started["id"]
        .as_str()
        .expect("start should include submission id");

    let submission = request_json(
        app,
        cookie_authenticated_request(
            "GET",
            &format!("/api/submissions/{submission_id}"),
            &respondent_cookie,
            None,
        ),
    )
    .await;
    assert_eq!(submission["status"], "draft");
}

#[tokio::test]
async fn pending_work_excludes_assignments_with_existing_drafts() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
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
            item["has_submitted"] == true
                && matches!(
                    item["account_email"].as_str(),
                    Some("respondent@tessara.local")
                        | Some("delegate@tessara.local")
                        | Some("delegator@tessara.local")
                )
        })
        .cloned()
        .expect("seed should create a submitted workflow assignment");
    let assignment_id = submitted_assignment["id"]
        .as_str()
        .expect("submitted assignment should include id");
    let account_email = submitted_assignment["account_email"]
        .as_str()
        .expect("submitted assignment should include account email");
    let account_password = match account_email {
        "respondent@tessara.local" => "tessara-dev-respondent",
        "delegate@tessara.local" => "tessara-dev-delegate",
        "delegator@tessara.local" => "tessara-dev-delegator",
        other => panic!("unexpected submitted assignment account: {other}"),
    };

    let respondent_token = login_token_for(app.clone(), account_email, account_password).await;

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
        "submitted workflow assignments cannot start new response work"
    );
}

#[tokio::test]
async fn starting_distinct_assignments_returns_distinct_submission_ids() {
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
    let pending_items = assignments
        .as_array()
        .expect("assignment list should be an array")
        .iter()
        .filter(|item| item["has_draft"] == false && item["has_submitted"] == false)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        pending_items.len() >= 2,
        "seed should expose at least two startable assignments"
    );

    let first_assignment_id = pending_items[0]["id"]
        .as_str()
        .expect("assignment list should include assignment id");
    let second_assignment_id = pending_items[1]["id"]
        .as_str()
        .expect("assignment list should include assignment id");

    let first_started = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{first_assignment_id}/start"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;
    let second_started = request_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{second_assignment_id}/start"),
            &admin_token,
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
async fn scoped_operator_cannot_start_out_of_scope_workflow_assignment_by_uuid() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;

    let operator_nodes = request_json(
        app.clone(),
        authorized_request("GET", "/api/nodes?q=Demo", &operator_token, None),
    )
    .await;
    let operator_node_ids = operator_nodes
        .as_array()
        .expect("operator node list should be an array")
        .iter()
        .filter_map(|node| node["id"].as_str())
        .collect::<Vec<_>>();
    assert!(
        !operator_node_ids.is_empty(),
        "seeded operator should have effective scope"
    );

    let assignments = request_json(
        app.clone(),
        authorized_request("GET", "/api/workflow-assignments", &admin_token, None),
    )
    .await;
    let assignment_items = assignments
        .as_array()
        .expect("assignment list should be an array");
    let is_startable = |item: &&Value| item["has_draft"] == false && item["has_submitted"] == false;
    let in_scope_assignment = assignment_items
        .iter()
        .find(|item| {
            is_startable(item)
                && item["node_id"]
                    .as_str()
                    .is_some_and(|node_id| operator_node_ids.contains(&node_id))
        })
        .cloned()
        .expect("seed should expose an in-scope startable assignment");
    let out_of_scope_assignment = assignment_items
        .iter()
        .find(|item| {
            is_startable(item)
                && item["node_id"]
                    .as_str()
                    .is_some_and(|node_id| !operator_node_ids.contains(&node_id))
        })
        .cloned()
        .expect("seed should expose an out-of-scope startable assignment");

    let out_of_scope_assignment_id = out_of_scope_assignment["id"]
        .as_str()
        .expect("assignment should expose id");
    let rejected = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{out_of_scope_assignment_id}/start"),
            &operator_token,
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(rejected.0, StatusCode::FORBIDDEN);
    assert_eq!(rejected.1["code"], "forbidden");

    let admin_started = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{out_of_scope_assignment_id}/start"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(admin_started.0, StatusCode::OK);
    assert!(
        admin_started.1["id"].as_str().is_some(),
        "admin start should create or return a draft submission"
    );

    let in_scope_assignment_id = in_scope_assignment["id"]
        .as_str()
        .expect("assignment should expose id");
    let operator_started = request_status_and_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{in_scope_assignment_id}/start"),
            &operator_token,
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(operator_started.0, StatusCode::OK);
    assert!(
        operator_started.1["id"].as_str().is_some(),
        "in-scope operator start should create or return a draft submission"
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

#[tokio::test]
async fn login_sets_cookie_session_for_browser_requests() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };

    let response = app
        .clone()
        .oneshot(
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
        .await
        .expect("router should produce response");
    assert_eq!(response.status(), StatusCode::OK);

    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .expect("login should set a browser session cookie")
        .to_string();
    assert!(set_cookie.contains("tessara_session="));
    assert!(set_cookie.contains("HttpOnly"));

    let cookie = set_cookie
        .split(';')
        .next()
        .expect("cookie pair should be present")
        .to_string();

    let me = request_json(
        app,
        Request::builder()
            .method("GET")
            .uri("/api/me")
            .header(header::COOKIE, cookie)
            .body(Body::empty())
            .expect("valid cookie-authenticated request"),
    )
    .await;
    assert_eq!(me["email"], "admin@tessara.local");
}

#[tokio::test]
async fn forms_and_hierarchy_endpoints_accept_cookie_sessions_without_authorization_headers() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let operator_cookie = login_cookie_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    let readable_forms = request_json(
        app.clone(),
        cookie_authenticated_request("GET", "/api/forms", &operator_cookie, None),
    )
    .await;
    assert!(
        !readable_forms
            .as_array()
            .expect("forms response should be an array")
            .is_empty()
    );
    let readable_nodes = request_json(
        app.clone(),
        cookie_authenticated_request("GET", "/api/nodes", &operator_cookie, None),
    )
    .await;
    assert!(
        !readable_nodes
            .as_array()
            .expect("nodes response should be an array")
            .is_empty()
    );

    let admin_cookie =
        login_cookie_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;
    let admin_forms = request_json(
        app.clone(),
        cookie_authenticated_request("GET", "/api/admin/forms", &admin_cookie, None),
    )
    .await;
    assert!(
        !admin_forms
            .as_array()
            .expect("admin forms response should be an array")
            .is_empty()
    );
    let admin_node_types = request_json(
        app,
        cookie_authenticated_request("GET", "/api/admin/node-types", &admin_cookie, None),
    )
    .await;
    assert!(
        !admin_node_types
            .as_array()
            .expect("admin node types response should be an array")
            .is_empty()
    );
}

#[tokio::test]
async fn invalid_login_uses_stable_error_payload() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };

    let login = request_status_and_json(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "email": "admin@tessara.local",
                    "password": "wrong-password"
                })
                .to_string(),
            ))
            .expect("valid invalid-login request"),
    )
    .await;

    assert_eq!(login.0, StatusCode::UNAUTHORIZED);
    assert_eq!(login.1["code"], "auth_invalid_credentials");
    assert_eq!(login.1["message"], "Email or password is incorrect.");
    assert_eq!(login.1["error"], "Email or password is incorrect.");
}

#[tokio::test]
async fn revoked_and_expired_sessions_return_stable_auth_codes() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(state) = test_state().await else {
        return;
    };
    let app = router(state.clone());

    let revoked_token = login_token(app.clone()).await;
    sqlx::query("UPDATE auth_sessions SET revoked_at = now() WHERE token = $1")
        .bind(
            revoked_token
                .parse::<uuid::Uuid>()
                .expect("token should be uuid"),
        )
        .execute(&state.pool)
        .await
        .expect("session should be revocable");

    let revoked = request_status_and_json(
        app.clone(),
        authorized_request("GET", "/api/me", &revoked_token, None),
    )
    .await;
    assert_eq!(revoked.0, StatusCode::UNAUTHORIZED);
    assert_eq!(revoked.1["code"], "auth_session_revoked");
    assert_eq!(
        revoked.1["message"],
        "Your session is no longer active. Sign in again."
    );

    let expired_token = login_token(app.clone()).await;
    sqlx::query("UPDATE auth_sessions SET expires_at = $2, revoked_at = NULL WHERE token = $1")
        .bind(
            expired_token
                .parse::<uuid::Uuid>()
                .expect("token should be uuid"),
        )
        .bind(Utc::now() - Duration::minutes(5))
        .execute(&state.pool)
        .await
        .expect("session should be expirable");

    let expired = request_status_and_json(
        app,
        authorized_request("GET", "/api/me", &expired_token, None),
    )
    .await;
    assert_eq!(expired.0, StatusCode::UNAUTHORIZED);
    assert_eq!(expired.1["code"], "auth_session_expired");
    assert_eq!(
        expired.1["message"],
        "Your session has expired. Sign in again."
    );
}

#[tokio::test]
async fn authenticated_requests_update_last_seen_timestamp() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(state) = test_state().await else {
        return;
    };
    let app = router(state.clone());
    let token = login_token(app.clone()).await;
    let token_uuid = token.parse::<uuid::Uuid>().expect("token should be uuid");

    let initial_last_seen: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT last_seen_at FROM auth_sessions WHERE token = $1")
            .bind(token_uuid)
            .fetch_one(&state.pool)
            .await
            .expect("session should exist");

    tokio::time::sleep(std::time::Duration::from_millis(15)).await;

    let me = request_json(app, authorized_request("GET", "/api/me", &token, None)).await;
    assert_eq!(me["email"], "admin@tessara.local");

    let updated_last_seen: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT last_seen_at FROM auth_sessions WHERE token = $1")
            .bind(token_uuid)
            .fetch_one(&state.pool)
            .await
            .expect("session should still exist");

    assert!(updated_last_seen > initial_last_seen);
}

async fn test_app() -> Option<axum::Router> {
    LazyLock::force(&TEST_TRACING);
    Some(router(test_state().await?))
}

async fn test_state() -> Option<db::AppState> {
    test_state_with_cookie_name("tessara_session").await
}

async fn test_state_with_cookie_name(auth_cookie_name: &str) -> Option<db::AppState> {
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

fn value_for_field_type(field_type: &str) -> Value {
    match field_type {
        "number" => json!(7),
        "boolean" => json!(false),
        "date" => json!("2026-05-04"),
        "multi_choice" => json!(["Sprint 2D"]),
        "single_choice" => json!("Sprint 2D"),
        _ => json!("Sprint 2D response value"),
    }
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

async fn login_cookie_for(app: axum::Router, email: &str, password: &str) -> String {
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

fn cookie_authenticated_request(
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
