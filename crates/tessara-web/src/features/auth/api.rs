//! Client-side API orchestration for the Auth feature.
//!
//! Keep endpoint calls, request assembly, and response handling for Auth screens here; pure DTOs and display formatting belong in sibling modules.

use crate::features::auth::types::SessionStateResponse;

#[cfg(feature = "hydrate")]
/// Fetches the fetch session data.
pub async fn fetch_session() -> Option<SessionStateResponse> {
    tessara_web_http::fetch_json("/api/auth/session", "Session")
        .await
        .ok()
}

#[cfg(not(feature = "hydrate"))]
/// Fetches the fetch session data.
pub async fn fetch_session() -> Option<SessionStateResponse> {
    None
}

#[cfg(feature = "hydrate")]
/// Submits the submit logout request.
pub async fn submit_logout() {
    let _ = tessara_web_http::send_without_response(
        gloo_net::http::Request::delete("/api/auth/logout"),
        "Sign out",
    )
    .await;
}

#[cfg(not(feature = "hydrate"))]
/// Submits the submit logout request.
pub async fn submit_logout() {}
