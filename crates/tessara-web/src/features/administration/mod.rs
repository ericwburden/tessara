//! Public boundary for the Administration feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Administration-specific implementation details in child modules.

mod api;
mod components;
mod display;
mod graph;
mod models;
mod nodes;
mod pages;
mod state;
#[cfg(feature = "hydrate")]
pub(crate) use models::{CreateNodePayload, UpdateNodePayload};
pub(crate) use pages::{
    AdministrationNodeTypesPage, AdministrationPage, AdministrationRolesPage,
    AdministrationUserAccessPage, AdministrationUserDetailPage, AdministrationUserEditPage,
    AdministrationUsersPage,
};
