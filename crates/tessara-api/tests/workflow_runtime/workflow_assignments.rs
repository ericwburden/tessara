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


