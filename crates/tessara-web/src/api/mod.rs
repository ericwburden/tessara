//! Compatibility API surface for generic HTTP and endpoint wiring.
//!
//! This module preserves existing behavior by re-exporting the current infra
//! transport helpers while the longer-term frontend layout work proceeds.

// TODO: The api module is currently a thin wrapper around the api/client.rs file. 
// Consider flattening this structure if no additional API-related code is needed.

pub(crate) mod client;

#[cfg(feature = "hydrate")]
pub(crate) use client::{redirect_to_login, send_json_request};
