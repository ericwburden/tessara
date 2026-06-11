//! HTTP infrastructure boundary for frontend feature modules.
//!
//! Re-export shared request helpers from here so features do not depend on lower-level transport module paths.

pub(crate) mod client;

#[cfg(feature = "hydrate")]
pub(crate) use client::{redirect_to_login, send_json_id_request};
