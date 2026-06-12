//! Role-management API loaders and save orchestration.

mod actions;
mod loaders;

pub(crate) use actions::save_admin_role;
pub(crate) use loaders::{load_admin_role_detail, load_admin_roles_context};
