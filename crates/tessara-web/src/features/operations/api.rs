//! Owns the features::operations::api module behavior.

#[cfg(feature = "hydrate")]
use super::types::OperationsStatus;

#[cfg(feature = "hydrate")]
/// Fetches the fetch operations status data.
pub(super) async fn fetch_operations_status() -> Result<OperationsStatus, String> {
    match gloo_net::http::Request::get("/api/operations/status")
        .send()
        .await
    {
        Ok(response) if response.ok() => response
            .json::<OperationsStatus>()
            .await
            .map_err(|error| format!("Unable to parse operations status: {error}")),
        Ok(response) => Err(format!(
            "Unable to load operations status. Server returned {}.",
            response.status()
        )),
        Err(error) => Err(format!("Unable to load operations status: {error}")),
    }
}
