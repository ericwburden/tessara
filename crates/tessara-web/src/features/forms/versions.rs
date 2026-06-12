//! Version-focused helpers for the Forms feature.
//!
//! Keep version selection and labels here rather than mixing them into broad page modules.

use crate::features::forms::{FormDefinition, FormSummary, FormVersionSummary};

/// Selects the active version for a form summary.
pub(crate) fn active_form_version(form: &FormSummary) -> Option<&FormVersionSummary> {
    form.versions
        .iter()
        .rev()
        .find(|version| version.status == "published")
        .or_else(|| form.versions.last())
}

/// Selects the active version for a form definition.
pub(crate) fn active_form_definition_version(form: &FormDefinition) -> Option<&FormVersionSummary> {
    form.versions
        .iter()
        .rev()
        .find(|version| version.status == "published")
        .or_else(|| form.versions.last())
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Selects the editable version for a form definition.
pub(crate) fn editable_form_definition_version(
    form: &FormDefinition,
) -> Option<&FormVersionSummary> {
    form.versions
        .iter()
        .rev()
        .find(|version| version.status == "draft")
        .or_else(|| active_form_definition_version(form))
}

/// Formats a form version label.
pub(crate) fn form_version_label(version: Option<&FormVersionSummary>) -> String {
    version
        .and_then(|version| version.version_label.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| "-".to_string())
}

/// Formats a sortable form version label.
pub(crate) fn form_version_sort_label(version: &FormVersionSummary) -> String {
    version.version_label.clone().unwrap_or_else(|| {
        match (
            version.version_major,
            version.version_minor,
            version.version_patch,
        ) {
            (Some(major), Some(minor), Some(patch)) => format!("{major}.{minor}.{patch}"),
            _ => "-".to_string(),
        }
    })
}
