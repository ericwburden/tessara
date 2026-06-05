mod dto;
mod extractor;
mod handlers;
mod repo;
mod service;

use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::db::AppState;

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

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/session", get(session))
        .route("/api/auth/logout", delete(logout))
        .route("/api/me", get(me))
}
