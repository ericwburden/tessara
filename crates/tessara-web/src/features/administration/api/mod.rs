//! Client-side API orchestration for the Administration feature.
//!
//! Keep endpoint calls, request assembly, and response handling grouped by Administration subdomain.

mod node_types;
mod roles;
mod users;

pub(crate) use node_types::{
    load_admin_node_type_catalog, load_admin_node_type_detail, save_node_type_metadata_field,
};
pub(crate) use roles::{load_admin_role_detail, load_admin_roles_context, save_admin_role};
pub(crate) use users::{
    load_admin_capability_catalog, load_admin_user_access, load_admin_user_edit_context,
    load_admin_users, submit_update_admin_user, submit_update_admin_user_access,
};
