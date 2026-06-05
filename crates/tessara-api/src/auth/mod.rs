mod dto;
mod extractor;
mod handlers;
mod repo;
mod service;

pub use dto::{
    AccountContext, CapabilityBoundary, DelegationSummary, LoginRequest, LoginResponse,
    LogoutResponse, ScopeNodeSummary, SessionContext, SessionStateResponse,
};
pub use extractor::AuthenticatedRequest;
pub use handlers::{login, logout, me, session};
pub use repo::{load_delegations, load_effective_capabilities, load_scope_nodes};
pub use service::{
    authenticate_request, capability_allows_node, capability_boundary, ensure_capability,
    hash_password_for_storage, password_scheme, require_capability,
    require_capability_contains_nodes, resolve_accessible_delegate_account_id, store_password_hash,
};
