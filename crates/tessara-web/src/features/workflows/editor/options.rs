//! Workflow editor option and slug helpers.

use crate::features::forms::FormSummary;
use crate::features::forms::{form_version_label, form_version_sort_label};
use crate::features::organization::NodeTypeCatalogEntry;
use crate::features::workflows::types::WorkflowSummary;

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the existing workflow slugs behavior.
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn existing_workflow_slugs(workflows: &[WorkflowSummary]) -> Vec<String> {
    workflows
        .iter()
        .map(|workflow| workflow.slug.clone())
        .collect()
}

/// Handles the workflow form is in scope behavior.
pub(crate) fn workflow_form_is_in_scope(
    form: &FormSummary,
    node_types: &[NodeTypeCatalogEntry],
    workflow_node_type_id: &str,
) -> bool {
    let _ = (form, node_types, workflow_node_type_id);
    true
}

/// Handles the workflow form version options behavior.
pub(crate) fn workflow_form_version_options(
    forms: &[FormSummary],
    node_types: &[NodeTypeCatalogEntry],
    workflow_node_type_id: &str,
) -> Vec<(String, String, String)> {
    let mut options = Vec::new();

    for form in forms {
        if !workflow_form_is_in_scope(form, node_types, workflow_node_type_id) {
            continue;
        }
        let mut versions = form
            .versions
            .iter()
            .filter(|version| version.status == "published")
            .collect::<Vec<_>>();
        versions.sort_by(|left, right| {
            form_version_sort_label(left).cmp(&form_version_sort_label(right))
        });

        for version in versions {
            let version_label = form_version_label(Some(version));
            options.push((
                version.id.clone(),
                format!("{} ({version_label})", form.name),
                form.name.clone(),
            ));
        }
    }

    options.sort_by(|left, right| left.1.cmp(&right.1));
    options
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the workflow step form label behavior.
pub(crate) fn workflow_step_form_label(forms: &[FormSummary], form_version_id: &str) -> String {
    forms
        .iter()
        .flat_map(|form| {
            form.versions.iter().map(move |version| {
                (
                    version.id.as_str(),
                    format!("{} ({})", form.name, form_version_label(Some(version))),
                )
            })
        })
        .find(|(id, _)| *id == form_version_id)
        .map(|(_, label)| label)
        .unwrap_or_else(|| "Select form version".to_string())
}
