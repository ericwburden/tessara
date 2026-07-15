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
    pub account: Option<SessionAccountResponse>,
}

#[derive(Clone, Serialize)]
pub struct SessionAccountResponse {
    #[serde(flatten)]
    pub account: AccountContext,
    /// Effective capability keys backed by an installation-global assignment.
    ///
    /// The ordinary `capabilities` projection remains the flat effective set
    /// used by existing clients. This companion set supports the few boundaries
    /// where assignment scope itself is material without serializing each
    /// capability-to-scope binding.
    pub global_capabilities: Vec<String>,
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

    /// Whether the account has the requested capability at installation-global scope.
    ///
    /// This deliberately evaluates every direct and implied grant instead of
    /// reusing `capability_scope`: an account may hold a scoped direct grant
    /// and a separate global grant that implies the same read capability.
    pub fn has_global_capability(&self, required: &str) -> bool {
        let implied = implied_manage_capability(required);
        self.capability_scopes.iter().any(|scope| {
            scope.global
                && (scope.capability == "admin:all"
                    || scope.capability == required
                    || implied
                        .as_deref()
                        .is_some_and(|capability| scope.capability == capability))
        })
    }

    pub fn global_capability_keys(&self) -> Vec<String> {
        let mut capabilities = self
            .capabilities
            .iter()
            .filter(|capability| self.has_global_capability(capability))
            .cloned()
            .collect::<Vec<_>>();
        capabilities.sort();
        capabilities.dedup();
        capabilities
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

impl From<AccountContext> for SessionAccountResponse {
    fn from(account: AccountContext) -> Self {
        let global_capabilities = account.global_capability_keys();
        Self {
            account,
            global_capabilities,
        }
    }
}

pub fn implied_manage_capability(required: &str) -> Option<String> {
    if required == "modules:read" {
        return Some("modules:manage_navigation".to_string());
    }

    let domain = required.strip_suffix(":read")?;
    // Dashboard composition intentionally supports internal managers who do
    // not also receive the product-facing reader directory/viewer surface.
    // Other established feature areas retain the historical manage=>read
    // implication.
    (domain != "dashboards").then(|| format!("{domain}:manage"))
}

#[cfg(test)]
mod tests {
    use super::{AccountContext, CapabilityScope, implied_manage_capability};

    #[test]
    fn dashboard_manage_does_not_imply_product_reader_access() {
        assert_eq!(implied_manage_capability("dashboards:read"), None);
        assert_eq!(
            implied_manage_capability("components:read").as_deref(),
            Some("components:manage")
        );
    }

    #[test]
    fn module_navigation_management_implies_module_read() {
        assert_eq!(
            implied_manage_capability("modules:read").as_deref(),
            Some("modules:manage_navigation")
        );
        assert_eq!(implied_manage_capability("modules:manage_navigation"), None);
    }

    #[test]
    fn global_capability_evaluation_ignores_scoped_only_module_grants() {
        let mut account = account_with_scopes(vec![CapabilityScope {
            capability: "modules:manage_navigation".to_string(),
            global: false,
            node_ids: Vec::new(),
        }]);

        assert!(!account.has_global_capability("modules:read"));
        assert!(!account.has_global_capability("modules:manage_navigation"));

        account.capability_scopes.push(CapabilityScope {
            capability: "modules:manage_navigation".to_string(),
            global: true,
            node_ids: Vec::new(),
        });
        assert!(account.has_global_capability("modules:read"));
        assert!(account.has_global_capability("modules:manage_navigation"));
    }

    #[test]
    fn global_admin_implies_both_module_capabilities() {
        let account = account_with_scopes(vec![CapabilityScope {
            capability: "admin:all".to_string(),
            global: true,
            node_ids: Vec::new(),
        }]);

        assert!(account.has_global_capability("modules:read"));
        assert!(account.has_global_capability("modules:manage_navigation"));
    }

    #[test]
    fn serialized_global_capabilities_exclude_scoped_keys_and_retain_global_implications() {
        let account = AccountContext {
            capabilities: vec![
                "forms:read".to_string(),
                "modules:manage_navigation".to_string(),
                "modules:read".to_string(),
            ],
            ..account_with_scopes(vec![
                CapabilityScope {
                    capability: "forms:read".to_string(),
                    global: false,
                    node_ids: vec![uuid::Uuid::new_v4()],
                },
                CapabilityScope {
                    capability: "modules:manage_navigation".to_string(),
                    global: true,
                    node_ids: Vec::new(),
                },
            ])
        };

        assert_eq!(
            account.global_capability_keys(),
            vec![
                "modules:manage_navigation".to_string(),
                "modules:read".to_string()
            ]
        );
    }

    fn account_with_scopes(capability_scopes: Vec<CapabilityScope>) -> AccountContext {
        AccountContext {
            account_id: uuid::Uuid::nil(),
            email: "module-auth@example.test".to_string(),
            display_name: "Module Auth".to_string(),
            is_active: true,
            roles: Vec::new(),
            capabilities: Vec::new(),
            capability_scopes,
            scope_nodes: Vec::new(),
            delegations: Vec::new(),
        }
    }
}
