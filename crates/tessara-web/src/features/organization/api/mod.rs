//! Organization API module boundary.
//!
//! Re-export focused form, workflow, and helper APIs from here so callers keep a stable `organization::api` boundary without a single mixed-concern implementation file.

mod helpers;

pub(crate) use helpers::IntoNonemptyString;
#[cfg(feature = "hydrate")]
pub(crate) use helpers::current_search_param;
