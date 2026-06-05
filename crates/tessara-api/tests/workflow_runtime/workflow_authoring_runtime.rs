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
        .find(|workflow| workflow["slug"] == "demo-session-log-workflow")
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
    assert_eq!(workflow_detail["workflow_node_type_name"], "Session");
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
    assert_eq!(scoped_workflow["workflow_node_type_name"], "Program");
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
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let form_version_id = seed["form_version_id"]
        .as_str()
        .expect("seed should expose form version id");
    let workflow_node_type_id =
        workflow_node_type_id_for_slug(app.clone(), &admin_token, "demo-session-log-workflow")
            .await;

    let first_workflow = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflows",
            &admin_token,
            Some(json!({
                "workflow_node_type_id": workflow_node_type_id,
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
                "workflow_node_type_id": workflow_node_type_id,
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
    let Some(app) = test_app().await else { return };
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
async fn multi_step_workflow_advances_to_next_form_for_same_assignee() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(state) = test_state().await else {
        return;
    };
    let app = router(state.clone());
    let admin_token = login_token(app.clone()).await;

    let seed = request_json(
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
    let seed_form_id: uuid::Uuid = seed["form_id"]
        .as_str()
        .expect("seed should expose form id")
        .parse()
        .expect("form id should be uuid");
    let seed_form_version_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        SELECT id
        FROM form_versions
        WHERE form_id = $1
          AND status = 'published'::form_version_status
        ORDER BY published_at DESC NULLS LAST, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(seed_form_id)
    .fetch_one(&state.pool)
    .await
    .expect("seed should expose a published form version");
    let follow_up_form_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO forms (name, slug, scope_node_type_id)
        SELECT 'Sprint 2E Follow-up', 'sprint-2e-follow-up', scope_node_type_id
        FROM forms
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(seed_form_id)
    .fetch_one(&state.pool)
    .await
    .expect("follow-up form should be created");
    let follow_up_form_version_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO form_versions (form_id, version_label, status, published_at)
        VALUES ($1, '2E follow-up', 'published'::form_version_status, now())
        RETURNING id
        "#,
    )
    .bind(follow_up_form_id)
    .fetch_one(&state.pool)
    .await
    .expect("follow-up form version should be published");

    let workflows = request_json(
        app.clone(),
        authorized_request("GET", "/api/workflows", &admin_token, None),
    )
    .await;
    let workflow_id = workflows
        .as_array()
        .expect("workflow list should be an array")
        .iter()
        .find(|workflow| workflow["slug"] == "demo-session-log-workflow")
        .and_then(|workflow| workflow["id"].as_str())
        .expect("seed form should expose workflow")
        .to_string();
    let created_version = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflows/{workflow_id}/versions"),
            &admin_token,
            Some(json!({
                "steps": [
                    {
                        "title": "Initial intake",
                        "form_version_id": seed_form_version_id
                    },
                    {
                        "title": "Follow-up collection",
                        "form_version_id": follow_up_form_version_id
                    }
                ]
            })),
        ),
    )
    .await;
    let workflow_version_id = created_version["id"]
        .as_str()
        .expect("created version should expose id");
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-versions/{workflow_version_id}/publish"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;
    let follow_up_form_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/forms/{follow_up_form_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        follow_up_form_detail["workflows"]
            .as_array()
            .expect("follow-up form detail should include workflows")
            .iter()
            .any(|workflow| workflow["id"] == workflow_id),
        "form detail should include workflows that use the form through any step"
    );

    let existing_assignments = request_json(
        app.clone(),
        authorized_request("GET", "/api/workflow-assignments", &admin_token, None),
    )
    .await;
    let respondent_seed_assignment = existing_assignments
        .as_array()
        .expect("assignment list should be an array")
        .iter()
        .find(|assignment| assignment["account_email"] == "respondent@tessara.local")
        .expect("seed should expose a respondent assignment");
    let node_id = seed["session_node_id"]
        .as_str()
        .expect("seed should expose session node id");
    let account_id = respondent_seed_assignment["account_id"]
        .as_str()
        .expect("assignment should expose account id");

    let bulk = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflow-assignments/bulk",
            &admin_token,
            Some(json!({
                "workflow_version_id": workflow_version_id,
                "node_id": node_id,
                "account_ids": [account_id]
            })),
        ),
    )
    .await;
    assert_eq!(bulk["results"][0]["status"], "created");
    let repeated_bulk = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflow-assignments/bulk",
            &admin_token,
            Some(json!({
                "workflow_version_id": workflow_version_id,
                "node_id": node_id,
                "account_ids": [account_id]
            })),
        ),
    )
    .await;
    assert_eq!(repeated_bulk["results"][0]["status"], "skipped");

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
    let first_assignment_id = pending
        .as_array()
        .expect("pending work should be an array")
        .iter()
        .find(|item| {
            item["workflow_version_id"] == workflow_version_id
                && item["workflow_step_title"] == "Initial intake"
        })
        .and_then(|item| item["workflow_assignment_id"].as_str())
        .expect("first step should be pending");
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
    let first_submission_id = first_started["id"]
        .as_str()
        .expect("first step start should return submission id");
    save_required_values(app.clone(), &respondent_token, first_submission_id).await;
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/submissions/{first_submission_id}/submit"),
            &respondent_token,
            None,
        ),
    )
    .await;

    let pending_after_first = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/workflow-assignments/pending",
            &respondent_token,
            None,
        ),
    )
    .await;
    let second_assignment = pending_after_first
        .as_array()
        .expect("pending work should be an array")
        .iter()
        .find(|item| {
            item["workflow_version_id"] == workflow_version_id
                && item["workflow_step_title"] == "Follow-up collection"
        })
        .cloned()
        .expect("second step should become pending");
    assert_eq!(second_assignment["form_name"], "Sprint 2E Follow-up");
    assert_eq!(second_assignment["workflow_step_position"], 1);
    assert_eq!(second_assignment["workflow_step_count"], 2);

    let second_assignment_id = second_assignment["workflow_assignment_id"]
        .as_str()
        .expect("second step should expose assignment id");
    let second_started = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{second_assignment_id}/start"),
            &respondent_token,
            Some(json!({})),
        ),
    )
    .await;
    let second_submission_id = second_started["id"]
        .as_str()
        .expect("second step start should return submission id");
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/submissions/{second_submission_id}/submit"),
            &respondent_token,
            None,
        ),
    )
    .await;
    let second_detail = request_json(
        app,
        authorized_request(
            "GET",
            &format!("/api/submissions/{second_submission_id}"),
            &respondent_token,
            None,
        ),
    )
    .await;
    assert_eq!(second_detail["runtime"]["step_count"], 2);
    assert_eq!(
        second_detail["runtime"]["history"]
            .as_array()
            .expect("runtime history should be an array")
            .len(),
        2
    );
    let instance_status: String = sqlx::query_scalar(
        "SELECT status FROM workflow_instances WHERE id = (SELECT workflow_instance_id FROM submissions WHERE id = $1)",
    )
    .bind(
        second_submission_id
            .parse::<uuid::Uuid>()
            .expect("submission id should be uuid"),
    )
    .fetch_one(&state.pool)
    .await
    .expect("workflow instance should be present");
    assert_eq!(instance_status, "completed");
}

#[tokio::test]
async fn multi_step_workflow_can_target_descendant_step_form_nodes() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(state) = test_state().await else {
        return;
    };
    let app = router(state.clone());
    let admin_token = login_token(app.clone()).await;

    let seed = request_json(
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
    let program_form_version_id = seed["program_form_version_id"]
        .as_str()
        .expect("seed should expose program form version id");
    let activity_form_version_id = seed["activity_form_version_id"]
        .as_str()
        .expect("seed should expose activity form version id");
    let workflow_node_type_id = workflow_node_type_id_for_slug(
        app.clone(),
        &admin_token,
        "demo-program-checkpoint-workflow",
    )
    .await;
    let program_node_id = seed["program_node_id"]
        .as_str()
        .expect("seed should expose program node id");
    let partner_node_id = seed["partner_node_id"]
        .as_str()
        .expect("seed should expose partner node id");
    let respondent_account_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM accounts WHERE email = 'respondent@tessara.local'")
            .fetch_one(&state.pool)
            .await
            .expect("respondent account should exist");

    let workflow = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflows",
            &admin_token,
            Some(json!({
                "workflow_node_type_id": workflow_node_type_id,
                "name": "Program With Activity Follow-up",
                "slug": "program-with-activity-follow-up",
                "description": "Program assignment that collects an activity-scoped second step."
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
                        "title": "Program snapshot",
                        "form_version_id": program_form_version_id
                    },
                    {
                        "title": "Activity plan",
                        "form_version_id": activity_form_version_id
                    }
                ]
            })),
        ),
    )
    .await;
    let workflow_version_id = created_version["id"]
        .as_str()
        .expect("created version should expose id");
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-versions/{workflow_version_id}/publish"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;

    let candidates = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/workflow-assignment-candidates?node_id={program_node_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        candidates
            .as_array()
            .expect("candidate list should be an array")
            .iter()
            .any(|candidate| candidate["workflow_version_id"] == workflow_version_id),
        "program node should be eligible when activity-scoped step can resolve to a descendant"
    );
    let partner_candidates = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/workflow-assignment-candidates?node_id={partner_node_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        !partner_candidates
            .as_array()
            .expect("candidate list should be an array")
            .iter()
            .any(|candidate| candidate["workflow_version_id"] == workflow_version_id),
        "workflow should be pinned to the highest component form scope, not an ancestor above it"
    );

    let bulk = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflow-assignments/bulk",
            &admin_token,
            Some(json!({
                "workflow_version_id": workflow_version_id,
                "node_id": program_node_id,
                "account_ids": [respondent_account_id]
            })),
        ),
    )
    .await;
    assert_eq!(bulk["results"][0]["status"], "created");

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
    let first_assignment_id = pending
        .as_array()
        .expect("pending work should be an array")
        .iter()
        .find(|item| {
            item["workflow_version_id"] == workflow_version_id
                && item["workflow_step_title"] == "Program snapshot"
        })
        .and_then(|item| item["workflow_assignment_id"].as_str())
        .expect("program step should be pending");
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
    let first_submission_id = first_started["id"]
        .as_str()
        .expect("first start should return submission id");
    let first_submission_node_id: uuid::Uuid =
        sqlx::query_scalar("SELECT node_id FROM submissions WHERE id = $1")
            .bind(
                first_submission_id
                    .parse::<uuid::Uuid>()
                    .expect("submission id should be uuid"),
            )
            .fetch_one(&state.pool)
            .await
            .expect("first submission should exist");
    assert_eq!(
        first_submission_node_id.to_string(),
        program_node_id,
        "program-scoped step should use the workflow assignment node"
    );

    save_required_values(app.clone(), &respondent_token, first_submission_id).await;
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/submissions/{first_submission_id}/submit"),
            &respondent_token,
            None,
        ),
    )
    .await;

    let pending_after_first = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/workflow-assignments/pending",
            &respondent_token,
            None,
        ),
    )
    .await;
    let second_assignment = pending_after_first
        .as_array()
        .expect("pending work should be an array")
        .iter()
        .find(|item| {
            item["workflow_version_id"] == workflow_version_id
                && item["workflow_step_title"] == "Activity plan"
        })
        .cloned()
        .expect("activity step should become pending");
    assert_eq!(second_assignment["form_name"], "Demo Activity Plan");

    let second_assignment_id = second_assignment["workflow_assignment_id"]
        .as_str()
        .expect("second step should expose assignment id");
    let second_started = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/workflow-assignments/{second_assignment_id}/start"),
            &respondent_token,
            Some(json!({})),
        ),
    )
    .await;
    let second_submission_id = second_started["id"]
        .as_str()
        .expect("second start should return submission id");
    let second_submission_node: (uuid::Uuid, Option<uuid::Uuid>, String) = sqlx::query_as(
        r#"
        SELECT nodes.id, nodes.parent_node_id, node_types.name
        FROM submissions
        JOIN nodes ON nodes.id = submissions.node_id
        JOIN node_types ON node_types.id = nodes.node_type_id
        WHERE submissions.id = $1
        "#,
    )
    .bind(
        second_submission_id
            .parse::<uuid::Uuid>()
            .expect("submission id should be uuid"),
    )
    .fetch_one(&state.pool)
    .await
    .expect("second submission should exist");
    assert_eq!(
        second_submission_node
            .1
            .expect("activity should have parent")
            .to_string(),
        program_node_id
    );
    assert_eq!(second_submission_node.2, "Activity");
    assert_ne!(second_submission_node.0.to_string(), program_node_id);

    save_required_values(app.clone(), &respondent_token, second_submission_id).await;
    request_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/submissions/{second_submission_id}/submit"),
            &respondent_token,
            None,
        ),
    )
    .await;
    let instance_status: String = sqlx::query_scalar(
        "SELECT status FROM workflow_instances WHERE id = (SELECT workflow_instance_id FROM submissions WHERE id = $1)",
    )
    .bind(
        second_submission_id
            .parse::<uuid::Uuid>()
            .expect("submission id should be uuid"),
    )
    .fetch_one(&state.pool)
    .await
    .expect("workflow instance should be present");
    assert_eq!(instance_status, "completed");
}

#[tokio::test]
async fn workflow_publish_rejects_branching_step_form_scopes() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(state) = test_state().await else {
        return;
    };
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
    let workflow_node_type_id = workflow_node_type_id_for_slug(
        app.clone(),
        &admin_token,
        "demo-program-checkpoint-workflow",
    )
    .await;

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
                "workflow_node_type_id": workflow_node_type_id,
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
    let rejected = request_status_and_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/workflow-versions/{workflow_version_id}/publish"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(rejected.0, StatusCode::BAD_REQUEST);
    assert_eq!(rejected.1["code"], "bad_request");
    assert!(
        rejected.1["error"]
            .as_str()
            .unwrap_or_default()
            .contains("one hierarchy lineage")
    );
}

#[tokio::test]
async fn workflow_publish_rejects_sibling_step_assignment_nodes() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
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
    let workflow_node_type_id = workflow_node_type_id_for_slug(
        app.clone(),
        &admin_token,
        "demo-intake-activity-checkpoint-workflow",
    )
    .await;

    let workflow = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/workflows",
            &admin_token,
            Some(json!({
                "workflow_node_type_id": workflow_node_type_id,
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

    let rejected = request_status_and_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/workflow-versions/{workflow_version_id}/publish"),
            &admin_token,
            Some(json!({})),
        ),
    )
    .await;
    assert_eq!(rejected.0, StatusCode::BAD_REQUEST);
    assert_eq!(rejected.1["code"], "bad_request");
    assert!(
        rejected.1["error"]
            .as_str()
            .unwrap_or_default()
            .contains("one hierarchy lineage")
    );
}


