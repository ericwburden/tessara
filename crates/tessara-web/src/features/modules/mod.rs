//! Core-owned Module Management feature boundary.
//!
//! Root integration owns route registration, document/bootstrap embedding, and
//! Axum authorization. This module owns the web projection, native pages, and
//! navigation-policy interaction once an authorized projection is supplied.

mod api;
mod bootstrap;
mod deployment;
mod detail;
mod directory;
mod models;
mod pages;
mod policy;

pub use bootstrap::{
    MODULE_MANAGEMENT_BOOTSTRAP_SCRIPT_ID, ModuleManagementRouteBootstrapV1,
    ModuleManagementSurfaceV1, NavigationPolicyBootstrapV1,
    clear_module_management_route_bootstrap, escaped_module_management_bootstrap_json,
    module_management_route_bootstrap, parse_module_management_bootstrap_json,
};
pub use models::*;
pub use pages::{ModuleManagementDetailPage, ModuleManagementDirectoryPage};
pub use policy::{
    PolicyMove, destinations_for_group, move_destination, move_destination_to_group,
    ordered_groups, set_destination_visibility,
};
