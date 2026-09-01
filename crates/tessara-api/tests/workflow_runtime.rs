use std::{collections::BTreeSet, sync::LazyLock};

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
async fn demo_seed_backfills_workflows_and_form_links() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
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
    let workflow_items = workflows
        .as_array()
        .expect("workflow list should be an array");
    let generated_source_form_ids = workflow_items
        .iter()
        .filter(|workflow| workflow["source"] == "generated_form")
        .map(|workflow| {
            workflow["source_form_id"]
                .as_str()
                .expect("generated workflow should expose its source form id")
        })
        .collect::<BTreeSet<_>>();
    let expected_generated_source_form_ids = [
        "program_form_id",
        "activity_form_id",
        "intake_activity_form_id",
        "workshop_activity_form_id",
    ]
    .into_iter()
    .map(|key| {
        seed[key]
            .as_str()
            .expect("seed should expose every assigned form id")
    })
    .collect::<BTreeSet<_>>();
    assert_eq!(
        generated_source_form_ids,
        expected_generated_source_form_ids
    );
    let linked_workflow = workflow_items
        .iter()
        .find(|workflow| {
            workflow["source"] == "generated_form"
                && workflow["source_form_id"] == seed["program_form_id"]
        })
        .cloned()
        .expect("seeded Program form should expose its generated workflow");
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
                seed["program_form_id"]
                    .as_str()
                    .expect("seed should include Program form id")
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
    assert_eq!(workflow_detail["workflow_node_type_name"], "Program");
    assert!(
        workflow_detail["assignments"]
            .as_array()
            .expect("workflow detail should include assignments")
            .iter()
            .any(|assignment| {
                assignment["form_id"] == seed["program_form_id"]
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

    let scoped_workflow = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/workflows/{}",
                seed["program_workflow_id"]
                    .as_str()
                    .expect("seed should expose scoped workflow id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(scoped_workflow["id"], seed["program_workflow_id"]);
    assert_eq!(scoped_workflow["workflow_node_type_name"], "Program");
    assert_eq!(
        scoped_workflow["versions"][0]["id"],
        seed["program_workflow_version_id"]
    );
    assert_eq!(
        scoped_workflow["versions"][0]["workflow_revision_label"],
        "1"
    );
    assert_eq!(scoped_workflow["versions"][0]["step_count"], 3);
    let scoped_steps = scoped_workflow["versions"][0]["steps"]
        .as_array()
        .expect("scoped workflow revision should include steps");
    assert_eq!(scoped_steps[0]["form_name"], "Demo Program Snapshot");
    assert_eq!(
        scoped_steps[1]["form_name"],
        "Demo Intake Activity Checkpoint"
    );
    assert_eq!(
        scoped_steps[2]["form_name"],
        "Demo Workshop Activity Checkpoint"
    );
    assert!(
        scoped_workflow["assignments"]
            .as_array()
            .expect("scoped workflow should include assignments")
            .iter()
            .any(|assignment| {
                assignment["id"] == seed["program_workflow_assignment_id"]
                    && assignment["node_name"] == "Demo Program Family Outreach"
                    && assignment["account_email"] == "respondent@tessara.local"
            })
    );
}

#[tokio::test]
async fn form_versions_can_be_reused_across_workflows() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
    let admin_token = login_token(app.clone()).await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let form_version_id = seed["form_version_id"]
        .as_str()
        .expect("seed should expose form version id");
    let session_node_id = seed["session_node_id"]
        .as_str()
        .expect("seed should expose session node id");

    let first_workflow = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflows",
            &admin_token,
            Some(json!({
                "available_node_ids": [session_node_id],
                "name": "Reusable Intake Workflow A",
                "slug": "reusable-intake-workflow-a",
                "description": "Uses the same form as another workflow."
            })),
        ),
    )
    .await;
    let second_workflow = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflows",
            &admin_token,
            Some(json!({
                "available_node_ids": [session_node_id],
                "name": "Reusable Intake Workflow B",
                "slug": "reusable-intake-workflow-b",
                "description": "Also uses the same form."
            })),
        ),
    )
    .await;

    let first_workflow_id = first_workflow["id"]
        .as_str()
        .expect("first workflow should expose id");
    let second_workflow_id = second_workflow["id"]
        .as_str()
        .expect("second workflow should expose id");
    assert_ne!(first_workflow_id, second_workflow_id);

    let first_version = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflows/{first_workflow_id}/versions"),
            &admin_token,
            Some(json!({
                "steps": [{
                    "title": "Shared intake",
                    "form_version_id": form_version_id
                }]
            })),
        ),
    )
    .await;
    let second_version = request_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/workflows/{second_workflow_id}/versions"),
            &admin_token,
            Some(json!({
                "steps": [{
                    "title": "Shared intake",
                    "form_version_id": form_version_id
                }]
            })),
        ),
    )
    .await;

    assert_ne!(first_version["id"], second_version["id"]);
}

#[tokio::test]
async fn generated_form_workflow_is_replaced_after_shortcut_is_promoted() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
    let admin_token = login_token(app.clone()).await;

    let _seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let activity_node_type_id = node_type_id_for_slug(app.clone(), &admin_token, "activity").await;
    let (form_id, first_version_id) = create_publishable_form(
        app.clone(),
        &admin_token,
        "Workflow Shortcut Regression",
        "workflow-shortcut-regression",
        &activity_node_type_id,
        "first",
    )
    .await;

    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{first_version_id}/publish"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;

    let initial_workflow =
        current_generated_workflow_for_form(app.clone(), &admin_token, &form_id).await;
    let initial_workflow_id = initial_workflow["id"]
        .as_str()
        .expect("generated workflow should expose id")
        .to_string();
    let initial_revision_id = initial_workflow["current_version_id"]
        .as_str()
        .expect("generated workflow should expose current revision")
        .to_string();
    let initial_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/workflows/{initial_workflow_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(initial_detail["source"], "generated_form");
    assert_eq!(initial_detail["source_form_id"], form_id);
    assert_eq!(
        initial_detail["versions"]
            .as_array()
            .expect("workflow should include revisions")
            .iter()
            .find(|version| version["id"] == initial_revision_id)
            .expect("current generated revision should be present")["step_count"],
        1
    );

    let multi_step_revision = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflows/{initial_workflow_id}/versions"),
            &admin_token,
            Some(json!({
                "steps": [
                    {
                        "title": "Initial response",
                        "form_version_id": first_version_id
                    },
                    {
                        "title": "Follow-up response",
                        "form_version_id": first_version_id
                    }
                ]
            })),
        ),
    )
    .await;
    let multi_step_revision_id = multi_step_revision["id"]
        .as_str()
        .expect("created workflow revision should expose id");
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-versions/{multi_step_revision_id}/publish"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;

    let promoted_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/workflows/{initial_workflow_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(promoted_detail["source"], "authored");
    assert!(promoted_detail["source_form_id"].is_null());

    let second_version_id = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/forms/{form_id}/versions"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await["id"]
        .as_str()
        .expect("new form revision should expose id")
        .to_string();
    add_publishable_form_contents(app.clone(), &admin_token, &second_version_id, "second").await;
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{second_version_id}/publish"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;

    let regenerated =
        current_generated_workflow_for_form(app.clone(), &admin_token, &form_id).await;
    let regenerated_workflow_id = regenerated["id"]
        .as_str()
        .expect("regenerated workflow should expose id");
    assert_ne!(regenerated_workflow_id, initial_workflow_id);

    let regenerated_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/workflows/{regenerated_workflow_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(regenerated_detail["source"], "generated_form");
    assert_eq!(regenerated_detail["source_form_id"], form_id);
    let regenerated_revision_id = regenerated["current_version_id"]
        .as_str()
        .expect("regenerated workflow should expose current revision");
    let regenerated_revision = regenerated_detail["versions"]
        .as_array()
        .expect("regenerated workflow should include revisions")
        .iter()
        .find(|version| version["id"] == regenerated_revision_id)
        .expect("regenerated current revision should be present");
    assert_eq!(regenerated_revision["step_count"], 1);
    assert_eq!(
        regenerated_revision["steps"][0]["form_version_id"],
        second_version_id
    );
}

#[tokio::test]
async fn workflow_publish_allows_branching_step_form_scopes() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let app = router(state.clone());
    let admin_token = login_token(app.clone()).await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let activity_form_version_id = seed["activity_form_version_id"]
        .as_str()
        .expect("seed should expose activity form version id");
    let program_node_id = seed["program_node_id"]
        .as_str()
        .expect("seed should expose program node id");

    let program_type_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM node_types WHERE name = 'Program'")
            .fetch_one(&state.pool)
            .await
            .expect("program type should exist");
    let sibling_type_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO node_types (name, slug)
        VALUES ('Branch Sibling', 'branch-sibling')
        RETURNING id
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .expect("sibling node type should be created");
    sqlx::query(
        "INSERT INTO node_type_relationships (parent_node_type_id, child_node_type_id) VALUES ($1, $2)",
    )
    .bind(program_type_id)
    .bind(sibling_type_id)
    .execute(&state.pool)
    .await
    .expect("sibling relationship should be created");
    let sibling_form_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO forms (name, slug, scope_node_type_id)
        VALUES ('Branch Sibling Form', 'branch-sibling-form', $1)
        RETURNING id
        "#,
    )
    .bind(sibling_type_id)
    .fetch_one(&state.pool)
    .await
    .expect("sibling form should be created");
    let sibling_form_version_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO form_versions (form_id, version_label, status, published_at)
        VALUES ($1, '1.0.0', 'published'::form_version_status, now())
        RETURNING id
        "#,
    )
    .bind(sibling_form_id)
    .fetch_one(&state.pool)
    .await
    .expect("sibling form version should be created");

    let workflow = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflows",
            &admin_token,
            Some(json!({
                "available_node_ids": [program_node_id],
                "name": "Branching Workflow",
                "slug": "branching-workflow",
                "description": "Should not publish because child scopes branch."
            })),
        ),
    )
    .await;
    let workflow_id = workflow["id"].as_str().expect("workflow should expose id");
    let created_version = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflows/{workflow_id}/versions"),
            &admin_token,
            Some(json!({
                "steps": [
                    {
                        "title": "Activity branch",
                        "form_version_id": activity_form_version_id
                    },
                    {
                        "title": "Sibling branch",
                        "form_version_id": sibling_form_version_id
                    }
                ]
            })),
        ),
    )
    .await;
    let workflow_version_id = created_version["id"]
        .as_str()
        .expect("created version should expose id");
    let published = request_status_and_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/workflow-versions/{workflow_version_id}/publish"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(published.0, StatusCode::OK);
    assert_eq!(published.1["id"], workflow_version_id);
}

#[tokio::test]
async fn workflow_publish_allows_sibling_step_assignment_nodes() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
    let admin_token = login_token(app.clone()).await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let intake_activity_form_version_id = seed["intake_activity_form_version_id"]
        .as_str()
        .expect("seed should expose intake activity form version id");
    let workshop_activity_form_version_id = seed["workshop_activity_form_version_id"]
        .as_str()
        .expect("seed should expose workshop activity form version id");
    let activity_node_id = seed["activity_node_id"]
        .as_str()
        .expect("seed should expose activity node id");

    let workflow = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflows",
            &admin_token,
            Some(json!({
                "available_node_ids": [activity_node_id],
                "name": "Sibling Activity Workflow",
                "slug": "sibling-activity-workflow",
                "description": "Should not publish because concrete activity nodes are siblings."
            })),
        ),
    )
    .await;
    let workflow_id = workflow["id"].as_str().expect("workflow should expose id");
    let created_version = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflows/{workflow_id}/versions"),
            &admin_token,
            Some(json!({
                "steps": [
                    {
                        "title": "Intake checkpoint",
                        "form_version_id": intake_activity_form_version_id
                    },
                    {
                        "title": "Workshop checkpoint",
                        "form_version_id": workshop_activity_form_version_id
                    }
                ]
            })),
        ),
    )
    .await;
    let workflow_version_id = created_version["id"]
        .as_str()
        .expect("created version should expose id");

    let published = request_status_and_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/workflow-versions/{workflow_version_id}/publish"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(published.0, StatusCode::OK);
    assert_eq!(published.1["id"], workflow_version_id);
}

#[tokio::test]
async fn workflow_assignments_can_be_deactivated() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
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
    let app = test_app().await;
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
async fn logout_revokes_the_current_session_token() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
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
    let app = test_app().await;

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
    let app = test_app().await;
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
async fn operations_status_keeps_assignments_usable_when_dataset_provider_is_unavailable() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let app = router(state.clone());
    let admin_token = login_token(app.clone()).await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let status = request_json(
        app.clone(),
        authorized_request("GET", "/api/operations/status", &admin_token, None),
    )
    .await;
    let datasets = status["dataset_readiness"]["datasets"]
        .as_array()
        .expect("operations status should include dataset readiness");
    assert!(datasets.is_empty());
    assert_eq!(status["dataset_readiness"]["state"], "unavailable");
    assert!(status["summary"]["dataset_attention_count"].is_null());
    let unstarted_assignment = status["workflow_assignments"]
        .as_array()
        .expect("operations status should include workflow assignments")
        .iter()
        .find(|assignment| {
            assignment["workflow_assignment_id"] == seed["program_workflow_assignment_id"]
        })
        .expect("Operations should retain the unstarted Program workflow assignment");
    assert_eq!(
        unstarted_assignment["workflow_id"],
        seed["program_workflow_id"]
    );
    assert!(unstarted_assignment["workflow_instance_id"].is_null());
    assert_eq!(unstarted_assignment["assignment_status"], "Not Started");
    assert!(unstarted_assignment["current_step_title"].is_null());
    assert_eq!(unstarted_assignment["completed_step_count"], 0);
    assert_eq!(unstarted_assignment["total_step_count"], 3);
    assert_eq!(unstarted_assignment["draft_response_count"], 0);
    assert_eq!(unstarted_assignment["submitted_response_count"], 0);
    assert!(unstarted_assignment["started_at"].is_null());
    assert!(unstarted_assignment["completed_at"].is_null());

    let workflow_assignment_id = seed["program_workflow_assignment_id"]
        .as_str()
        .expect("seed should expose Program workflow assignment id")
        .parse::<uuid::Uuid>()
        .expect("Program workflow assignment id should be a UUID");
    let workflow_instance_id = sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO workflow_instances (
            workflow_assignment_id,
            workflow_version_id,
            node_id,
            assignee_account_id,
            started_by_account_id
        )
        SELECT id, workflow_version_id, node_id, account_id, account_id
        FROM workflow_assignments
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(workflow_assignment_id)
    .fetch_one(&state.pool)
    .await
    .expect("test setup should start the Program workflow assignment");
    sqlx::query(
        r#"
        INSERT INTO workflow_step_instances (workflow_instance_id, workflow_step_id)
        SELECT $1, workflow_step_id
        FROM workflow_assignments
        WHERE id = $2
        "#,
    )
    .bind(workflow_instance_id)
    .bind(workflow_assignment_id)
    .execute(&state.pool)
    .await
    .expect("test setup should start the assigned Workflow step");

    let started_status = request_json(
        app.clone(),
        authorized_request("GET", "/api/operations/status", &admin_token, None),
    )
    .await;
    let started_assignment = started_status["workflow_assignments"]
        .as_array()
        .expect("operations status should retain started workflow assignments")
        .iter()
        .find(|assignment| {
            assignment["workflow_assignment_id"] == seed["program_workflow_assignment_id"]
        })
        .expect("Operations should project the started Program workflow assignment");
    assert_eq!(
        started_assignment["workflow_instance_id"],
        workflow_instance_id.to_string()
    );
    assert_eq!(started_assignment["assignment_status"], "In Progress");
    assert_eq!(started_assignment["current_step_title"], "Program Snapshot");
    assert!(started_assignment["started_at"].as_str().is_some());
    let app_summary = request_json(
        app.clone(),
        authorized_request("GET", "/api/summary", &admin_token, None),
    )
    .await;
    assert_eq!(app_summary["dataset_state"], "unavailable");
    assert!(app_summary["datasets"].is_null());
    assert!(app_summary["dataset_revisions"].is_null());
    assert!(app_summary["published_form_versions"].as_i64().is_some());

    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    let operator_status = request_json(
        app.clone(),
        authorized_request("GET", "/api/operations/status", &operator_token, None),
    )
    .await;
    assert!(
        operator_status["summary"]["open_workflow_assignment_count"]
            .as_i64()
            .expect("operator operations summary should expose scoped open assignment count")
            >= 0
    );
    assert_eq!(operator_status["dataset_readiness"]["state"], "unavailable");
    assert!(operator_status["summary"]["dataset_attention_count"].is_null());

    let anonymous_analytics = request_status_and_json(
        app,
        Request::builder()
            .method("GET")
            .uri("/api/admin/analytics/status")
            .body(Body::empty())
            .expect("valid anonymous analytics status request"),
    )
    .await;
    assert_eq!(anonymous_analytics.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn invalid_login_uses_stable_error_payload() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;

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
    let state = test_state().await;
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
    let state = test_state().await;
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

async fn test_app() -> axum::Router {
    LazyLock::force(&TEST_TRACING);
    router(test_state().await)
}

async fn node_type_id_for_slug(app: axum::Router, token: &str, slug: &str) -> String {
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

async fn create_publishable_form(
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

async fn add_publishable_form_contents(
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

async fn current_generated_workflow_for_form(
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

async fn test_state() -> db::AppState {
    test_state_with_cookie_name("tessara_session").await
}

async fn test_state_with_cookie_name(auth_cookie_name: &str) -> db::AppState {
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
        auth_cookie_name: auth_cookie_name.to_string(),
        auth_cookie_secure: false,
        auth_session_ttl_hours: 12,
    };
    let pool = db::connect_and_prepare(&config)
        .await
        .expect("database should migrate and seed");

    db::AppState { pool, config }
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
        is_disposable_database_name(&database_name),
        "TEST_API_DATABASE_URL must point at a database with a token-bounded disposable name marker ({}); got '{database_name}'",
        DISPOSABLE_DATABASE_NAME_TOKENS.join(", ")
    );

    drop_all_public_tables(&pool).await;
    drop_all_public_routines(&pool).await;
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

async fn drop_all_public_routines(pool: &PgPool) {
    let routines = sqlx::query_scalar::<_, String>(
        r#"
        SELECT procedure.oid::regprocedure::text
        FROM pg_proc procedure
        JOIN pg_namespace namespace ON namespace.oid = procedure.pronamespace
        LEFT JOIN pg_depend extension_dependency
          ON extension_dependency.classid = 'pg_proc'::regclass
         AND extension_dependency.objid = procedure.oid
         AND extension_dependency.deptype = 'e'
        WHERE namespace.nspname = 'public'
          AND procedure.prokind IN ('f', 'p')
          AND extension_dependency.objid IS NULL
        ORDER BY procedure.oid::regprocedure::text
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("public test routines should be enumerable");

    for routine in routines {
        sqlx::query(&format!("DROP ROUTINE IF EXISTS {routine} CASCADE"))
            .execute(pool)
            .await
            .expect("public test routine should be droppable");
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
