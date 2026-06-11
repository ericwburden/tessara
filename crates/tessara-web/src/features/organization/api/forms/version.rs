//! Form version selection and label helpers for Organization-owned form flows.

use crate::features::forms::{FormDefinition, FormSummary, FormVersionSummary};

/// Handles the active form version behavior.
pub(crate) fn active_form_version(form: &FormSummary) -> Option<&FormVersionSummary> {
    form.versions
        .iter()
        .rev()
        .find(|version| version.status == "published")
        .or_else(|| form.versions.last())
}

/// Handles the active form definition version behavior.
pub(crate) fn active_form_definition_version(form: &FormDefinition) -> Option<&FormVersionSummary> {
    form.versions
        .iter()
        .rev()
        .find(|version| version.status == "published")
        .or_else(|| form.versions.last())
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the editable form definition version behavior.
pub(crate) fn editable_form_definition_version(
    form: &FormDefinition,
) -> Option<&FormVersionSummary> {
    form.versions
        .iter()
        .rev()
        .find(|version| version.status == "draft")
        .or_else(|| active_form_definition_version(form))
}

/// Handles the form version label behavior.
pub(crate) fn form_version_label(version: Option<&FormVersionSummary>) -> String {
    version
        .and_then(|version| version.version_label.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| "-".to_string())
}

/// Handles the form version sort label behavior.
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
