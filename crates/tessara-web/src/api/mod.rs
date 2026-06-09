//! Compatibility API surface for generic HTTP and endpoint wiring.
//!
//! This module preserves existing behavior by re-exporting the current infra
//! transport helpers while the longer-term frontend layout work proceeds.

pub(crate) mod client;
pub(crate) mod endpoints;
pub(crate) mod error;

#[cfg(feature = "hydrate")]
pub(crate) use client::{redirect_to_login, send_json_request};
