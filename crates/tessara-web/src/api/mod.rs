//! HTTP transport helpers for Tessara web feature modules.

pub(crate) mod client;

#[cfg(feature = "hydrate")]
pub(crate) use client::{redirect_to_login, send_json_id_request};
