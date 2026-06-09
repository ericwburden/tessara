#[cfg(feature = "hydrate")]
use gloo_net::http::Request;

use crate::features::auth::types::SessionStateResponse;

#[cfg(feature = "hydrate")]
pub async fn fetch_session() -> Option<SessionStateResponse> {
    let response = Request::get("/api/auth/session").send().await;

    match response {
        Ok(response) if response.ok() => response.json::<SessionStateResponse>().await.ok(),
        _ => None,
    }
}

#[cfg(not(feature = "hydrate"))]
pub async fn fetch_session() -> Option<SessionStateResponse> {
    None
}

#[cfg(feature = "hydrate")]
pub async fn submit_logout() {
    let _ = Request::delete("/api/auth/logout").send().await;
}

#[cfg(not(feature = "hydrate"))]
pub async fn submit_logout() {}
