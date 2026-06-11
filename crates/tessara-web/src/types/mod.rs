//! Shared frontend type registry.
//!
//! Re-export cross-cutting value types here when they are not owned by a single product feature.

pub(crate) mod route_params;

pub(crate) use route_params::AccountRouteParams;
