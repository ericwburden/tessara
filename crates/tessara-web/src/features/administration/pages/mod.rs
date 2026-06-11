//! Route-level page composition for the Administration feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

mod landing;
mod node_types;
mod roles;
mod users;

pub(crate) use landing::AdministrationPage;
pub(crate) use node_types::AdministrationNodeTypesPage;
pub(crate) use roles::AdministrationRolesPage;
pub(crate) use users::{
    AdministrationUserAccessPage, AdministrationUserDetailPage, AdministrationUserEditPage,
    AdministrationUsersPage,
};
