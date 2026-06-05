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
pub enum SessionTransport {
    Bearer,
    Cookie,
}

#[derive(Clone, Debug)]
pub struct SessionContext {
    pub token: Uuid,
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

#[derive(Clone, Debug)]
pub struct CapabilityScope {
    pub capability: String,
    pub global: bool,
    pub node_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityBoundary {
    None,
    Global,
    Scoped(Vec<Uuid>),
}

#[derive(Clone, Serialize)]
pub struct AccountContext {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub roles: Vec<String>,
    pub capabilities: Vec<String>,
    #[serde(skip)]
    pub capability_scopes: Vec<CapabilityScope>,
    pub scope_nodes: Vec<ScopeNodeSummary>,
    pub delegations: Vec<DelegationSummary>,
}

impl AccountContext {
    pub fn has_capability(&self, required: &str) -> bool {
        self.capability_scope(required).is_some()
    }

    pub fn capability_scope(&self, required: &str) -> Option<&CapabilityScope> {
        self.matching_capability_scope(required)
            .map(|(scope, _)| scope)
    }

    pub fn matching_capability_scope(&self, required: &str) -> Option<(&CapabilityScope, String)> {
        if let Some(scope) = self
            .capability_scopes
            .iter()
            .find(|scope| scope.capability == "admin:all" || scope.capability == required)
        {
            return Some((scope, scope.capability.clone()));
        }

        let implied = implied_manage_capability(required)?;
        self.capability_scopes
            .iter()
            .find(|scope| scope.capability == implied)
            .map(|scope| (scope, implied))
    }
}

pub fn implied_manage_capability(required: &str) -> Option<String> {
    required
        .strip_suffix(":read")
        .map(|domain| format!("{domain}:manage"))
}
