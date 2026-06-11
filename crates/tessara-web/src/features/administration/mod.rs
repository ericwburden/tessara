//! Owns the features::administration module behavior.

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
