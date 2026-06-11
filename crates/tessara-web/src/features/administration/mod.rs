//! Public boundary for the Administration feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Administration-specific implementation details in child modules.

mod api;
mod models;
mod node_types;
mod nodes;
mod pages;
mod roles;
mod users;
#[cfg(feature = "hydrate")]
pub(crate) use models::{CreateNodePayload, UpdateNodePayload};
pub(crate) use node_types::AdministrationNodeTypesPage;
pub(crate) use pages::AdministrationPage;
pub(crate) use roles::AdministrationRolesPage;
pub(crate) use users::{
    AdministrationUserAccessPage, AdministrationUserDetailPage, AdministrationUserEditPage,
    AdministrationUsersPage,
};
