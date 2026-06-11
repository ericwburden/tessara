//! View models for the Administration feature.
//!
//! Keep derived frontend models and lightweight state shapes here when they are shared by multiple Administration pages or helpers.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::features::organization::AdminRoleSummary;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminCapabilitySummary {
    pub(crate) id: String,
    pub(crate) key: String,
    pub(crate) description: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminAccountAssignmentSummary {
    pub(crate) account_id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminRoleDetail {
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) capabilities: Vec<AdminCapabilitySummary>,
    #[serde(default)]
    pub(crate) assigned_accounts: Vec<AdminAccountAssignmentSummary>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateAdminRolePayload {
    pub(crate) name: String,
    pub(crate) capability_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateAdminRolePayload {
    pub(crate) capability_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminUserSummary {
    pub(crate) id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
    pub(crate) is_active: bool,
    #[serde(default)]
    pub(crate) roles: Vec<AdminRoleSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminScopeNodeSummary {
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) parent_node_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminDelegationSummary {
    pub(crate) account_id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminUserDetail {
    pub(crate) id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
    pub(crate) is_active: bool,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) roles: Vec<AdminRoleSummary>,
    #[serde(default)]
    pub(crate) scope_nodes: Vec<AdminScopeNodeSummary>,
    #[serde(default)]
    pub(crate) delegations: Vec<AdminDelegationSummary>,
    #[serde(default)]
    pub(crate) delegated_by: Vec<AdminDelegationSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminUserAccessDetail {
    pub(crate) account_id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) scope_nodes: Vec<AdminScopeNodeSummary>,
    #[serde(default)]
    pub(crate) available_scope_nodes: Vec<AdminScopeNodeSummary>,
    #[serde(default)]
    pub(crate) delegations: Vec<AdminDelegationSummary>,
    #[serde(default)]
    pub(crate) available_delegate_accounts: Vec<AdminDelegationSummary>,
    pub(crate) scope_assignments_editable: bool,
    pub(crate) delegation_assignments_editable: bool,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateAdminUserPayload {
    pub(crate) email: String,
    pub(crate) display_name: String,
    pub(crate) password: Option<String>,
    pub(crate) is_active: bool,
    pub(crate) role_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateAdminUserAccessPayload {
    pub(crate) scope_node_ids: Vec<String>,
    pub(crate) delegate_account_ids: Vec<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateNodePayload {
    pub(crate) node_type_id: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) name: String,
    pub(crate) metadata: serde_json::Map<String, Value>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateNodePayload {
    pub(crate) parent_node_id: Option<String>,
    pub(crate) name: String,
    pub(crate) metadata: serde_json::Map<String, Value>,
}
