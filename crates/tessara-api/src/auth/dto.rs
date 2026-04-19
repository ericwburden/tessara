use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub signed_out: bool,
}

#[derive(Clone, Serialize)]
pub struct SessionStateResponse {
    pub authenticated: bool,
    pub account: Option<AccountContext>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UiAccessProfile {
    Admin,
    Operator,
    ResponseUser,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionTransport {
    Bearer,
    Cookie,
}

#[derive(Clone, Debug)]
pub struct SessionContext {
    pub token: Uuid,
    pub account_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub transport: SessionTransport,
}

#[derive(Clone, Serialize)]
pub struct ScopeNodeSummary {
    pub node_id: Uuid,
    pub node_name: String,
    pub node_type_name: String,
    pub parent_node_id: Option<Uuid>,
    pub parent_node_name: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct DelegationSummary {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
}

#[derive(Clone, Serialize)]
pub struct AccountContext {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub ui_access_profile: UiAccessProfile,
    pub roles: Vec<String>,
    pub capabilities: Vec<String>,
    pub scope_nodes: Vec<ScopeNodeSummary>,
    pub delegations: Vec<DelegationSummary>,
}

impl AccountContext {
    pub fn is_admin(&self) -> bool {
        self.ui_access_profile == UiAccessProfile::Admin
    }

    pub fn is_operator(&self) -> bool {
        self.ui_access_profile == UiAccessProfile::Operator
    }
}
