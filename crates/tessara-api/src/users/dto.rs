use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{DelegationSummary, ScopeNodeSummary};

/// Compact role row used by admin list and account assignment screens.
#[derive(Serialize)]
pub struct RoleSummary {
    pub id: Uuid,
    pub name: String,
    pub capability_count: i64,
    pub account_count: i64,
}

/// Capability catalog entry that can be assigned to administrator-managed roles.
#[derive(Serialize)]
pub struct CapabilitySummary {
    pub id: Uuid,
    pub key: String,
    pub description: String,
}

/// Account row shown from a role detail page.
#[derive(Serialize)]
pub struct AccountAssignmentSummary {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
}

/// Role detail including its behavioral capabilities and assigned accounts.
#[derive(Serialize)]
pub struct RoleDetail {
    pub id: Uuid,
    pub name: String,
    pub capabilities: Vec<CapabilitySummary>,
    pub assigned_accounts: Vec<AccountAssignmentSummary>,
}

/// Compact account row used by the administration users list.
#[derive(Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub roles: Vec<RoleSummary>,
}

/// Account detail with effective access metadata for review screens.
#[derive(Serialize)]
pub struct UserDetail {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub capabilities: Vec<String>,
    pub roles: Vec<RoleSummary>,
    pub scope_nodes: Vec<ScopeNodeSummary>,
    pub delegations: Vec<DelegationSummary>,
    pub delegated_by: Vec<DelegationSummary>,
}

/// Editable scope/delegation state for account access management.
#[derive(Serialize)]
pub struct UserAccessDetail {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub capabilities: Vec<String>,
    pub scope_nodes: Vec<ScopeNodeSummary>,
    pub available_scope_nodes: Vec<ScopeNodeSummary>,
    pub delegations: Vec<DelegationSummary>,
    pub available_delegate_accounts: Vec<DelegationSummary>,
    pub scope_assignments_editable: bool,
    pub delegation_assignments_editable: bool,
}

/// Standard identifier response for create and update endpoints.
#[derive(Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}

/// Payload for creating a local account and initial credential.
#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub display_name: String,
    pub password: String,
    pub is_active: bool,
    pub role_ids: Vec<Uuid>,
}

/// Payload for editing a local account and optionally replacing its password.
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub email: String,
    pub display_name: String,
    pub password: Option<String>,
    pub is_active: bool,
    pub role_ids: Vec<Uuid>,
}

/// Payload for creating a reusable role capability bundle.
#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub capability_ids: Vec<Uuid>,
}

/// Payload for replacing the capabilities assigned to a role.
#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub capability_ids: Vec<Uuid>,
}

/// Payload for replacing account scope roots and delegation grants.
#[derive(Deserialize)]
pub struct UpdateUserAccessRequest {
    pub scope_node_ids: Vec<Uuid>,
    pub delegate_account_ids: Vec<Uuid>,
}
