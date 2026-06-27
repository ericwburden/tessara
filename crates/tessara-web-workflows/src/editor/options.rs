//! Workflow editor option and slug helpers.

use crate::types::NodeTypeCatalogEntry;
use crate::types::{WorkflowFormSummary, WorkflowFormVersionSummary, WorkflowSummary};

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn existing_workflow_slugs(workflows: &[WorkflowSummary]) -> Vec<String> {
    workflows
        .iter()
        .map(|workflow| workflow.slug.clone())
        .collect()
}

pub(crate) fn workflow_form_is_in_scope(
    form: &WorkflowFormSummary,
    node_types: &[NodeTypeCatalogEntry],
    workflow_node_type_id: &str,
) -> bool {
    let _ = (form, node_types, workflow_node_type_id);
    true
}

pub(crate) fn workflow_form_version_options(
    forms: &[WorkflowFormSummary],
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
            workflow_form_version_sort_label(left).cmp(&workflow_form_version_sort_label(right))
        });

        for version in versions {
            let version_label = workflow_form_version_label(Some(version));
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
pub(crate) fn workflow_step_form_label(
    forms: &[WorkflowFormSummary],
    form_version_id: &str,
) -> String {
    forms
        .iter()
        .flat_map(|form| {
            form.versions.iter().map(move |version| {
                (
                    version.id.as_str(),
                    format!(
                        "{} ({})",
                        form.name,
                        workflow_form_version_label(Some(version))
                    ),
                )
            })
        })
        .find(|(id, _)| *id == form_version_id)
        .map(|(_, label)| label)
        .unwrap_or_else(|| "Select form version".to_string())
}

fn workflow_form_version_label(version: Option<&WorkflowFormVersionSummary>) -> String {
    version
        .and_then(|version| version.version_label.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| "-".to_string())
}

fn workflow_form_version_sort_label(version: &WorkflowFormVersionSummary) -> String {
    format!(
        "{:04}.{:04}.{:04}.{}",
        version.version_major.unwrap_or(-1),
        version.version_minor.unwrap_or(-1),
        version.version_patch.unwrap_or(-1),
        version.version_label.clone().unwrap_or_default()
    )
}
