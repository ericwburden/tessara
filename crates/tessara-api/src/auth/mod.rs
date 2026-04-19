mod dto;
mod extractor;
mod handlers;
mod repo;
mod service;

pub use dto::{
    AccountContext, DelegationSummary, LoginRequest, LoginResponse, LogoutResponse,
    ScopeNodeSummary, SessionContext, UiAccessProfile,
};
pub use extractor::AuthenticatedRequest;
pub use handlers::{login, logout, me};
pub use repo::{
    effective_scope_node_ids, load_delegations, load_effective_capabilities, load_scope_nodes,
};
pub use service::{
    backfill_legacy_password_hashes, derive_ui_access_profile, hash_password_for_storage,
    password_scheme, require_authenticated, require_capability,
    resolve_accessible_delegate_account_id, scope_assignments_are_meaningful, store_password_hash,
};
