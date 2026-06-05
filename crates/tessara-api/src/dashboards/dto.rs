use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Payload for creating or updating dashboard metadata and visibility.
#[derive(Deserialize)]
pub struct CreateDashboardRequest {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<Uuid>,
}

/// Payload for placing a component version on a dashboard.
#[derive(Deserialize)]
pub struct AddDashboardComponentRequest {
    pub(crate) component_version_id: Uuid,
    pub(crate) position: i32,
    #[serde(default)]
    pub(crate) config: Value,
}

/// Compact dashboard row used by list surfaces.
#[derive(Serialize)]
pub struct DashboardSummary {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) visibility_nodes: Vec<DashboardVisibilityNodeSummary>,
    pub(crate) component_count: i64,
}

/// Dashboard detail with visible scope and component placements.
#[derive(Serialize)]
pub struct DashboardResponse {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) visibility_nodes: Vec<DashboardVisibilityNodeSummary>,
    pub(crate) components: Vec<DashboardComponentResponse>,
}

/// Organization node that makes a dashboard visible.
#[derive(Clone, Serialize)]
pub struct DashboardVisibilityNodeSummary {
    pub(crate) node_id: Uuid,
    pub(crate) node_name: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<Uuid>,
    pub(crate) node_path: String,
}

/// Component placement rendered inside a dashboard.
#[derive(Serialize)]
pub struct DashboardComponentResponse {
    pub(crate) id: Uuid,
    pub(crate) position: i32,
    pub(crate) config: Value,
    pub(crate) component_version_id: Uuid,
    pub(crate) component_id: Uuid,
    pub(crate) component_name: String,
    pub(crate) component_slug: String,
    pub(crate) component_type: String,
    pub(crate) dataset_revision_id: Uuid,
}
