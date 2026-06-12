//! Public boundary for the Auth feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Auth-specific implementation details in child modules.

pub mod api;
pub mod guards;
pub mod types;

pub use api::{fetch_session, submit_logout};
pub use guards::require_authenticated_route;
pub use types::{SessionStateResponse, ShellAccountContext, ShellAccountSummary};
