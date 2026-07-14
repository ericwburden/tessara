//! Dashboard transport contracts.

use serde::{Deserialize, Serialize};
use tessara_dashboards::{DashboardPlacementConfigState, DashboardPlacementOperation};
use uuid::Uuid;

/// Payload for creating or updating Dashboard metadata and visibility.
#[derive(Clone, Debug, Deserialize)]
pub struct CreateDashboardRequest {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<Uuid>,
}

/// Geometry submitted by the Dashboard composition editor.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DashboardPlacementGeometry {
    pub(crate) grid_row: i32,
    pub(crate) grid_column: i32,
    pub(crate) grid_width: i32,
    pub(crate) grid_height: i32,
}

/// One command in a transactional full-layout reconciliation request.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum DashboardCompositionCommand {
    /// Preserve an existing binding while changing geometry and, when
    /// authorized, its title. Omitting `title` preserves the stored title;
    /// an empty title clears it.
    Retain {
        placement_id: Uuid,
        geometry: DashboardPlacementGeometry,
        #[serde(default)]
        title: Option<String>,
        /// Explicitly replace a malformed V1 payload with canonical V1 data.
        /// Ordinary full-layout saves preserve malformed and future payloads.
        #[serde(default)]
        repair: bool,
    },
    /// Add a new binding (`client_key`) or explicitly replace an existing
    /// binding (`placement_id`). Exactly one identity field must be present.
    Bind {
        #[serde(default)]
        placement_id: Option<Uuid>,
        #[serde(default)]
        client_key: Option<String>,
        component_version_id: Uuid,
        geometry: DashboardPlacementGeometry,
        #[serde(default)]
        title: Option<String>,
    },
    /// Remove an existing placement by its stable opaque id.
    Remove { placement_id: Uuid },
}

/// Transactional Dashboard composition payload.
#[derive(Clone, Debug, Deserialize)]
pub struct ReconcileDashboardCompositionRequest {
    #[serde(default)]
    pub(crate) commands: Vec<DashboardCompositionCommand>,
}

/// Compact Dashboard row used by list surfaces.
#[derive(Clone, Debug, Serialize)]
pub struct DashboardSummary {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) visibility_nodes: Vec<DashboardVisibilityNodeSummary>,
    pub(crate) placement_count: i64,
    pub(crate) can_manage: bool,
}

/// Dashboard detail with complete placement envelopes.
#[derive(Clone, Debug, Serialize)]
pub struct DashboardResponse {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) visibility_nodes: Vec<DashboardVisibilityNodeSummary>,
    pub(crate) placement_count: i64,
    pub(crate) can_manage: bool,
    pub(crate) placements: Vec<DashboardPlacementResponse>,
}

/// Manage-authorized composition bootstrap and canonical save response.
#[derive(Clone, Debug, Serialize)]
pub struct DashboardCompositionResponse {
    pub(crate) dashboard: DashboardResponse,
    pub(crate) available_component_versions: Vec<DashboardComponentVersionOption>,
    pub(crate) new_placement_ids: Vec<DashboardPlacementIdMapping>,
}

/// Correlates a client-only add key to the stable stored placement id.
#[derive(Clone, Debug, Serialize)]
pub struct DashboardPlacementIdMapping {
    pub(crate) client_key: String,
    pub(crate) placement_id: Uuid,
}

/// Organization node that makes a Dashboard visible.
#[derive(Clone, Debug, Serialize)]
pub struct DashboardVisibilityNodeSummary {
    pub(crate) node_id: Uuid,
    pub(crate) node_name: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<Uuid>,
    pub(crate) node_path: String,
}

/// Organization node available to Dashboard metadata/create settings.
#[derive(Clone, Debug, Serialize)]
pub struct DashboardVisibilityNodeOption {
    pub(crate) id: Uuid,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_name: Option<String>,
    pub(crate) name: String,
}

/// Redaction-safe placement envelope returned for every stored row.
#[derive(Clone, Debug, Serialize)]
pub struct DashboardPlacementResponse {
    pub(crate) placement_id: Uuid,
    pub(crate) position: i32,
    pub(crate) grid_row: i32,
    pub(crate) grid_column: i32,
    pub(crate) grid_width: i32,
    pub(crate) grid_height: i32,
    pub(crate) availability: DashboardPlacementAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) config_state: Option<DashboardPlacementConfigState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) component: Option<DashboardComponentVersionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) allowed_operations: Option<Vec<DashboardPlacementOperation>>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DashboardPlacementAvailability {
    Available,
    Unavailable,
}

/// Component/version metadata included only when the caller can read it.
#[derive(Clone, Debug, Serialize)]
pub struct DashboardComponentVersionSummary {
    pub(crate) component_version_id: Uuid,
    pub(crate) component_id: Uuid,
    pub(crate) component_name: String,
    pub(crate) component_slug: String,
    pub(crate) component_type: String,
    pub(crate) version_number: i32,
    pub(crate) version_label: String,
    pub(crate) version_status: String,
}

/// Placeable exact version returned by the scoped composition picker.
#[derive(Clone, Debug, Serialize)]
pub struct DashboardComponentVersionOption {
    pub(crate) component_version_id: Uuid,
    pub(crate) component_id: Uuid,
    pub(crate) component_name: String,
    pub(crate) component_slug: String,
    pub(crate) component_type: String,
    pub(crate) version_number: i32,
    pub(crate) version_label: String,
    pub(crate) version_status: String,
    pub(crate) default_grid_width: u16,
    pub(crate) default_grid_height: u16,
}
