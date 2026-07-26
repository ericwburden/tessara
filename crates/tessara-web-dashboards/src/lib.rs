#![recursion_limit = "512"]

//! Dashboard-specific web content and request-bootstrap contracts.
//!
//! Route registration, parameter parsing, shell composition, and session policy
//! remain in root `tessara-web`.

mod api;
mod bootstrap;
mod http;
mod pages;
mod types;

#[cfg(feature = "hydrate")]
pub use bootstrap::clear_dashboard_route_bootstrap;
pub use bootstrap::{DashboardRouteBootstrap, dashboard_route_bootstrap};
pub use pages::{
    DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS, DashboardCreateContent, DashboardDetailContent,
    DashboardEditorContent, DashboardViewerContent, DashboardsIndexContent,
};
pub use types::{
    Dashboard, DashboardComponentVersion, DashboardComponentVersionOption, DashboardComposition,
    DashboardPlacement, DashboardPlacementAvailability, DashboardPlacementConfigState,
    DashboardPlacementIdMapping, DashboardPlacementOperation, DashboardPlacementResolutionState,
    DashboardSummary, DashboardVisibilityNode, SessionAccount, VisibilityNodeOption,
};
