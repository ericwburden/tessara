//! Client-side API orchestration for the Operations feature.
//!
//! Keep endpoint calls, request assembly, and response handling for Operations screens here; pure DTOs and display formatting belong in sibling modules.

#[cfg(feature = "hydrate")]
use super::types::OperationsStatus;

#[cfg(feature = "hydrate")]
/// Fetches the fetch operations status data.
pub(super) async fn fetch_operations_status() -> Result<OperationsStatus, String> {
    tessara_web_http::fetch_json("/api/operations/status", "Operations status")
        .await
        .map_err(tessara_web_http::RequestError::into_message)
}
