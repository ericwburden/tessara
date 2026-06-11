//! Organization form API compatibility boundary.
//!
//! Re-export focused form version helpers and Forms-owned save flows while keeping callers on the stable `organization::api::forms` boundary.

mod version;

pub(crate) use crate::features::forms::{submit_create_form, submit_update_form};
#[cfg(feature = "hydrate")]
pub(crate) use version::editable_form_definition_version;
pub(crate) use version::{
    active_form_definition_version, active_form_version, form_version_label,
    form_version_sort_label,
};
