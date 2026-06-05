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


