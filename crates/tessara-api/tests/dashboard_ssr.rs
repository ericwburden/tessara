#![cfg(feature = "ssr")]

#[allow(dead_code)]
mod support;

use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, Request, StatusCode, header},
};
use serde_json::json;
use tower::ServiceExt;

use support::{
    TEST_DATABASE_LOCK, authorized_request, login_token, login_token_for, request_json, test_app,
};

#[tokio::test]
async fn authenticated_dashboard_direct_loads_embed_authorized_ssr_state() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let dashboard_id = seed["dashboard_id"].as_str().expect("Dashboard id");

    let (admin_status, admin_html, admin_headers) = response_text(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/dashboards/{dashboard_id}/view"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(admin_status, StatusCode::OK);
    assert_eq!(
        admin_headers
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("private, no-store")
    );
    assert_eq!(
        admin_headers
            .get(header::VARY)
            .and_then(|value| value.to_str().ok()),
        Some("Cookie, Authorization")
    );
    assert!(admin_html.contains("tessara-dashboard-bootstrap"));
    assert!(admin_html.contains(r#""route":"viewer""#));
    assert!(admin_html.contains("Demo Operations Dashboard"));
    assert!(admin_html.contains(r#""placement_count":9"#));
    assert!(admin_html.contains(r#""grid_width":6"#));
    assert!(admin_html.contains(r#""grid_height":4"#));
    assert!(admin_html.contains("Partner Profile"));
    assert!(!admin_html.contains("dataset_id"));

    let reader_token = create_user_with_capabilities(
        app.clone(),
        &admin_token,
        "dashboard-ssr-reader@tessara.local",
        "Dashboard SSR Reader",
        &["dashboards:read"],
    )
    .await;
    let (reader_status, reader_html, _) = response_text(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/dashboards/{dashboard_id}/view"),
            &reader_token,
            None,
        ),
    )
    .await;
    assert_eq!(reader_status, StatusCode::OK);
    assert!(reader_html.contains("Demo Operations Dashboard"));
    assert!(reader_html.contains(r#""placement_count":9"#));
    assert!(reader_html.contains(r#""availability":"unavailable""#));
    assert!(reader_html.contains(r#""grid_row":1"#));
    assert!(reader_html.contains(r#""grid_column":1"#));
    assert!(!reader_html.contains("component_version_id"));
    assert!(!reader_html.contains("component_name"));
    assert!(!reader_html.contains("version_label"));
    assert!(!reader_html.contains("dataset_id"));
    assert!(!reader_html.contains("Partner Profile"));

    let (unauthenticated_status, _, unauthenticated_headers) = response_text(
        app.clone(),
        Request::builder()
            .uri("/dashboards")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(unauthenticated_status, StatusCode::SEE_OTHER);
    assert_eq!(
        unauthenticated_headers
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/login")
    );

    let respondent_token = login_token_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;
    let (forbidden_status, _, _) = response_text(
        app,
        authorized_request("GET", "/dashboards", &respondent_token, None),
    )
    .await;
    assert_eq!(forbidden_status, StatusCode::FORBIDDEN);
}

async fn create_user_with_capabilities(
    app: axum::Router,
    admin_token: &str,
    email: &str,
    display_name: &str,
    capability_keys: &[&str],
) -> String {
    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", admin_token, None),
    )
    .await;
    let capability_ids = capability_keys
        .iter()
        .map(|key| {
            capabilities
                .as_array()
                .expect("capability list")
                .iter()
                .find(|capability| capability["key"] == *key)
                .and_then(|capability| capability["id"].as_str())
                .unwrap_or_else(|| panic!("missing capability {key}"))
        })
        .collect::<Vec<_>>();
    let role = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/roles",
            admin_token,
            Some(json!({
                "name": format!("{display_name} Role"),
                "capability_ids": capability_ids
            })),
        ),
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/users",
            admin_token,
            Some(json!({
                "email": email,
                "display_name": display_name,
                "password": "tessara-test-password-123",
                "is_active": true,
                "role_ids": [role["id"]]
            })),
        ),
    )
    .await;
    login_token_for(app, email, "tessara-test-password-123").await
}

async fn response_text(
    app: axum::Router,
    request: Request<Body>,
) -> (StatusCode, String, HeaderMap) {
    let response = app
        .oneshot(request)
        .await
        .expect("router should produce response");
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    (
        status,
        String::from_utf8(body.to_vec()).expect("UTF-8 response"),
        headers,
    )
}
