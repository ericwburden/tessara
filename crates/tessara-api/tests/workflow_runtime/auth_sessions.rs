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


