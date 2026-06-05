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


