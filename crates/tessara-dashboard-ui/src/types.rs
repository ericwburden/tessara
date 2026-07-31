//! Dashboard-owned HTTP and editor state contracts.

use serde::{Deserialize, Serialize};
pub use tessara_dashboards::{DashboardPlacementConfigState, DashboardPlacementOperation};

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionAccount {
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl SessionAccount {
    pub fn can_read_dashboards(&self) -> bool {
        self.capabilities
            .iter()
            .any(|capability| capability == "dashboards:read" || capability == "admin:all")
    }

    pub fn can_manage_dashboards(&self) -> bool {
        self.capabilities
            .iter()
            .any(|capability| capability == "dashboards:manage" || capability == "admin:all")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DashboardVisibilityNode {
    pub node_id: String,
    pub node_name: String,
    pub node_type_name: String,
    pub parent_node_id: Option<String>,
    pub node_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DashboardSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub visibility_nodes: Vec<DashboardVisibilityNode>,
    pub placement_count: i64,
    /// Whether the current account may manage this specific Dashboard scope.
    #[serde(default)]
    pub can_manage: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub visibility_nodes: Vec<DashboardVisibilityNode>,
    pub placement_count: i64,
    /// Whether the current account may manage this specific Dashboard scope.
    #[serde(default)]
    pub can_manage: bool,
    #[serde(default)]
    pub placements: Vec<DashboardPlacement>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DashboardPlacementAvailability {
    Available,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DashboardPlacementResolutionState {
    Available,
    Restricted,
    ProviderUnavailable,
    Inactive,
    Superseded,
    Tombstoned,
    OwnerTombstoned,
    OwnerDataDestroyed,
    Missing,
    Incompatible,
    NotEvaluated,
}

impl DashboardPlacementResolutionState {
    pub const fn title(self) -> &'static str {
        match self {
            Self::Available => "Placement available",
            Self::Restricted => "Placement unavailable",
            Self::ProviderUnavailable => "Component provider unavailable",
            Self::Inactive => "ComponentVersion is inactive",
            Self::Superseded => "ComponentVersion is superseded",
            Self::Tombstoned => "ComponentVersion was removed",
            Self::OwnerTombstoned => "Owner Module Instance was removed",
            Self::OwnerDataDestroyed => "Owner data was destroyed",
            Self::Missing => "ComponentVersion is missing",
            Self::Incompatible => "Component provider is incompatible",
            Self::NotEvaluated => "Placement was not evaluated",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Available => "Available",
            Self::Restricted => "Placement unavailable",
            Self::ProviderUnavailable => "Provider unavailable",
            Self::Inactive => "Component inactive",
            Self::Superseded => "Component superseded",
            Self::Tombstoned => "Component removed",
            Self::OwnerTombstoned => "Owner Module Instance removed",
            Self::OwnerDataDestroyed => "Owner data destroyed",
            Self::Missing => "Component missing",
            Self::Incompatible => "Component incompatible",
            Self::NotEvaluated => "Not evaluated",
        }
    }

    pub const fn message(self) -> &'static str {
        match self {
            Self::Available => "This placement is available.",
            Self::Restricted => {
                "This placement cannot be displayed. No Component identity or lifecycle detail is available."
            }
            Self::ProviderUnavailable => {
                "This placement cannot be rendered right now. The Dashboard remains available while the Components provider recovers."
            }
            Self::Inactive => {
                "This exact ComponentVersion is inactive. Retain it or explicitly replace it in Placement details."
            }
            Self::Superseded => {
                "This exact ComponentVersion has been superseded. Tessara will not automatically rebind the placement."
            }
            Self::Tombstoned => {
                "The provider reports that this ComponentVersion was removed. Replace or remove the saved reference."
            }
            Self::OwnerTombstoned => {
                "The Module Instance that owned this resource has been removed."
            }
            Self::OwnerDataDestroyed => {
                "The owning Module Instance reports that its retained resource data was destroyed."
            }
            Self::Missing => {
                "The authorized Components provider returned no matching ComponentVersion."
            }
            Self::Incompatible => {
                "The Components provider is reachable, but it does not support the required Dashboard compatibility contract."
            }
            Self::NotEvaluated => "No current resolution decision is available for this placement.",
        }
    }

    pub const fn retryable(self) -> bool {
        matches!(self, Self::ProviderUnavailable | Self::NotEvaluated)
    }

    pub const fn css_class(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Restricted => "restricted",
            Self::ProviderUnavailable => "provider-unavailable",
            Self::Inactive => "inactive",
            Self::Superseded => "superseded",
            Self::Tombstoned => "tombstoned",
            Self::OwnerTombstoned => "owner-tombstoned",
            Self::OwnerDataDestroyed => "owner-data-destroyed",
            Self::Missing => "missing",
            Self::Incompatible => "incompatible",
            Self::NotEvaluated => "not-evaluated",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DashboardPlacement {
    pub placement_id: String,
    pub position: i32,
    pub grid_row: i32,
    pub grid_column: i32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub availability: DashboardPlacementAvailability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_state: Option<DashboardPlacementResolutionState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_state: Option<DashboardPlacementConfigState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<DashboardComponentVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_operations: Option<Vec<DashboardPlacementOperation>>,
}

impl DashboardPlacement {
    pub fn effective_resolution_state(&self) -> DashboardPlacementResolutionState {
        self.resolution_state.unwrap_or_else(|| {
            if self.availability == DashboardPlacementAvailability::Available {
                DashboardPlacementResolutionState::Available
            } else {
                DashboardPlacementResolutionState::Restricted
            }
        })
    }

    pub fn display_title(&self) -> String {
        self.title
            .clone()
            .or_else(|| {
                self.component
                    .as_ref()
                    .map(|component| component.component_name.clone())
            })
            .unwrap_or_else(|| "Unavailable placement".to_string())
    }

    pub fn kind_label(&self) -> &str {
        self.component
            .as_ref()
            .map(|component| component.component_type.as_str())
            .unwrap_or("redacted")
    }

    pub fn allows(&self, operation: DashboardPlacementOperation) -> bool {
        self.allowed_operations
            .as_ref()
            .is_some_and(|operations| operations.contains(&operation))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DashboardComponentVersion {
    pub component_version_id: String,
    pub component_id: String,
    pub component_name: String,
    pub component_slug: String,
    pub component_type: String,
    pub version_number: i32,
    pub version_label: String,
    pub version_status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DashboardComponentVersionOption {
    pub component_version_id: String,
    pub component_id: String,
    pub component_name: String,
    pub component_slug: String,
    pub component_type: String,
    pub version_number: i32,
    pub version_label: String,
    pub version_status: String,
    pub default_grid_width: i32,
    pub default_grid_height: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DashboardPlacementIdMapping {
    pub client_key: String,
    pub placement_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DashboardComposition {
    pub dashboard: Dashboard,
    #[serde(default)]
    pub available_component_versions: Vec<DashboardComponentVersionOption>,
    #[serde(default)]
    pub new_placement_ids: Vec<DashboardPlacementIdMapping>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct DashboardPlacementGeometry {
    pub grid_row: i32,
    pub grid_column: i32,
    pub grid_width: i32,
    pub grid_height: i32,
}

impl From<&DashboardPlacement> for DashboardPlacementGeometry {
    fn from(placement: &DashboardPlacement) -> Self {
        Self {
            grid_row: placement.grid_row,
            grid_column: placement.grid_column,
            grid_width: placement.grid_width,
            grid_height: placement.grid_height,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum DashboardCompositionCommand {
    Retain {
        placement_id: String,
        geometry: DashboardPlacementGeometry,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "is_false")]
        repair: bool,
    },
    Bind {
        #[serde(skip_serializing_if = "Option::is_none")]
        placement_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_key: Option<String>,
        component_version_id: String,
        geometry: DashboardPlacementGeometry,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    Remove {
        placement_id: String,
    },
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReconcileDashboardCompositionRequest {
    pub commands: Vec<DashboardCompositionCommand>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DashboardMetadataRequest {
    pub name: String,
    pub description: Option<String>,
    pub visibility_node_ids: Vec<String>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct IdResponse {
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct VisibilityNodeOption {
    pub id: String,
    pub node_type_name: String,
    pub parent_node_name: Option<String>,
    pub name: String,
}

impl VisibilityNodeOption {
    pub fn label(&self) -> String {
        match &self.parent_node_name {
            Some(parent) => format!("{parent} / {} ({})", self.name, self.node_type_name),
            None => format!("{} ({})", self.name, self.node_type_name),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPlacement {
    pub placement: DashboardPlacement,
    pub client_key: Option<String>,
    pub removed: bool,
    pub requested_title: Option<Option<String>>,
    pub replace_with: Option<String>,
    pub repair: bool,
}

impl EditorPlacement {
    pub fn existing(placement: DashboardPlacement) -> Self {
        Self {
            placement,
            client_key: None,
            removed: false,
            requested_title: None,
            replace_with: None,
            repair: false,
        }
    }

    pub fn key(&self) -> &str {
        self.client_key
            .as_deref()
            .unwrap_or(&self.placement.placement_id)
    }

    pub fn geometry(&self) -> DashboardPlacementGeometry {
        DashboardPlacementGeometry::from(&self.placement)
    }
}
