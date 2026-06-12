//! Workflow list attachment panel boundary.

mod assigned_users;
mod available_nodes;

pub(in crate::features::workflows) use assigned_users::{
    WorkflowAssignedUsersList, WorkflowAssignedUsersSheet,
};
pub(in crate::features::workflows) use available_nodes::{
    WorkflowAvailableNodesList, WorkflowAvailableNodesSheet,
};
