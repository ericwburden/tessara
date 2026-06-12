//! User access Administration components.

mod capabilities;
mod delegations;
mod scope_nodes;

pub(crate) use capabilities::AdminCapabilityList;
pub(crate) use delegations::{AdminDelegationChecklist, AdminDelegationList};
pub(crate) use scope_nodes::{AdminScopeNodeChecklist, AdminScopeNodeList};
