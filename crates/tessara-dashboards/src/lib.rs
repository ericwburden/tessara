//! Pure dashboard composition contracts.
//!
//! HTTP DTOs and persistence remain outside this crate. This module owns the
//! bounded dashboard policy layered over Tessara's framework-free grid types.

mod composition;
mod placement_config;
mod transition_component;

pub use composition::{
    CompositionError, DASHBOARD_GRID_CONSTRAINTS, DASHBOARD_HARD_MINIMUM,
    DashboardPlacementOperation, DashboardPlacementSizePolicy, DashboardPlacementSizeRule,
    reflow_dashboard_movement, validate_dashboard_layout, validate_dashboard_layout_with,
    validate_dashboard_resize,
};
pub use placement_config::{
    DASHBOARD_PLACEMENT_SCHEMA_VERSION, DashboardPlacementConfigInput,
    DashboardPlacementConfigState, DashboardPlacementConfigV1, LegacyPlacementKey,
    ParsedDashboardPlacement, ParsedDashboardPlacementConfig, classify_dashboard_placement_config,
    encode_dashboard_placement_config, legacy_fallback_layout, parse_dashboard_placement_config,
    parse_dashboard_placement_configs,
};
pub use transition_component::{
    DASHBOARD_COMPONENT_BINDING_KEY, DASHBOARD_COMPONENT_CONTRACT_ID,
    DASHBOARD_COMPONENT_RESOURCE_TYPE, DashboardComponentCatalogResponseV1,
    DashboardComponentMetadataV1, DashboardComponentResolutionRequestV1,
    DashboardComponentResolutionResponseV1, DashboardComponentResolutionValidationError,
    DashboardComponentTransitionAction, DashboardComponentVersionReferenceV1,
    DashboardComponentVersionReferenceValidationError,
};

pub use tessara_core::grid_layout::{GridPlacement, GridRect, GridSize};
