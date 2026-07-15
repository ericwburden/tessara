#![cfg(feature = "ssr")]

#[allow(dead_code)]
mod support;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use tower::ServiceExt;

use support::{TEST_DATABASE_LOCK, authorized_request, login_token, test_app};

#[tokio::test]
async fn dataset_preview_and_revision_edit_direct_loads_keep_native_route_ownership() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
    let admin_token = login_token(app.clone()).await;

    for (path, document_title, native_marker) in [
        (
            "/datasets/dataset-characterization/preview",
            "Dataset Preview",
            "dataset-preview-page",
        ),
        (
            "/datasets/dataset-characterization/revisions/revision-characterization/edit",
            "Edit Dataset Revision",
            "Edit Revision",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(authorized_request("GET", path, &admin_token, None))
            .await
            .expect("native Dataset route should produce a response");
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "direct load failed for {path}"
        );
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("text/html; charset=utf-8"),
            "{path} should remain a native HTML document"
        );
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("native Dataset document should be readable");
        let html =
            String::from_utf8(body.to_vec()).expect("native Dataset document should be UTF-8");

        assert!(
            html.contains(&format!("<title>{document_title}</title>")),
            "{path} should retain its native document metadata"
        );
        assert!(
            html.contains(native_marker),
            "{path} should render its Leptos-owned route surface"
        );
        assert!(html.contains(r#"id="app-root""#));
        assert!(!html.contains("Route not found"));
        assert!(!html.contains("/bridge/"));

        let unauthenticated = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .expect("valid unauthenticated route request"),
            )
            .await
            .expect("protected Dataset route should produce a response");
        assert_eq!(
            unauthenticated.status(),
            StatusCode::SEE_OTHER,
            "{path} should remain authentication protected"
        );
        assert_eq!(
            unauthenticated
                .headers()
                .get(header::LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some("/login")
        );
    }
}
