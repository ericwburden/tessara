//! Bounded private projection of Dashboard-owned reverse dependencies.
//!
//! Core uses this document only where its own Component, Dataset, or
//! Organization operations must account for saved Dashboard references. The
//! projection contains no Dashboard credentials and creates no cross-database
//! connection or relational constraint.

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DashboardDependencyProjectionV1 {
    pub schema_version: u16,
    pub dashboards: Vec<DashboardDependencyV1>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DashboardDependencyV1 {
    pub dashboard_id: Uuid,
    pub dashboard_name: String,
    pub description: Option<String>,
    pub scope_node_ids: Vec<Uuid>,
    pub placements: Vec<DashboardPlacementDependencyV1>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DashboardPlacementDependencyV1 {
    pub placement_id: Uuid,
    pub component_version_id: Uuid,
    pub position: i32,
    pub config: Value,
}

pub(crate) async fn load() -> ApiResult<DashboardDependencyProjectionV1> {
    let response = reqwest::Client::new()
        .get(format!(
            "{}/api/private/dependency-projection",
            dashboard_url()
        ))
        .header(
            "x-tessara-module-control-key",
            std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
                .unwrap_or_else(|_| "development-module-control-only".into()),
        )
        .send()
        .await
        .map_err(|_| unavailable())?
        .error_for_status()
        .map_err(|_| unavailable())?
        .json::<DashboardDependencyProjectionV1>()
        .await
        .map_err(|_| unavailable())?;
    if response.schema_version != 1 {
        return Err(unavailable());
    }
    Ok(response)
}

fn dashboard_url() -> String {
    std::env::var("TESSARA_DASHBOARD_MODULE_URL")
        .unwrap_or_else(|_| "http://dashboards:8091".into())
        .trim_end_matches('/')
        .to_string()
}

fn unavailable() -> ApiError {
    ApiError::ServiceUnavailable("Dashboard dependency projection unavailable".into())
}
