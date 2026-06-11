//! Shared filtering and option helpers.
//!
//! Keep reusable node, form, workflow, slug, and metadata filter logic here when several feature tables need the same semantics.

use crate::features::forms::FormSummary;
use crate::features::organization::{
    NodeTypeCatalogEntry, form_version_label, form_version_sort_label,
};
use crate::features::workflows::types::WorkflowSummary;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct FormNodeFilterOption {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) path: String,
    pub(crate) depth: usize,
}

/// Handles the form node filter options behavior.
pub(crate) fn form_node_filter_options(forms: &[FormSummary]) -> Vec<FormNodeFilterOption> {
    let mut options_by_id = BTreeMap::<String, FormNodeFilterOption>::new();

    for form in forms {
        for version in &form.versions {
            for node in &version.assignment_nodes {
                if node.node_id.trim().is_empty() || node.node_name.trim().is_empty() {
                    continue;
                }

                let path = if node.node_path.trim().is_empty() {
                    node.node_name.clone()
                } else {
                    node.node_path.clone()
                };

                options_by_id
                    .entry(node.node_id.clone())
                    .or_insert_with(|| FormNodeFilterOption {
                        id: node.node_id.clone(),
                        name: node.node_name.clone(),
                        parent_node_id: node.parent_node_id.clone(),
                        path,
                        depth: 0,
                    });
            }
        }
    }

    let options_map = options_by_id.clone();
    let mut options = options_by_id
        .into_values()
        .map(|mut option| {
            option.depth = form_node_filter_depth(&option.id, &options_map, &mut HashSet::new());
            option.path = form_node_filter_path(&option.id, &options_map, &mut HashSet::new());
            option
        })
        .collect::<Vec<_>>();
    options.sort_by(|left, right| left.path.cmp(&right.path).then(left.name.cmp(&right.name)));
    options
}

/// Handles the form node filter depth behavior.
pub(crate) fn form_node_filter_depth(
    node_id: &str,
    options_by_id: &BTreeMap<String, FormNodeFilterOption>,
    visited: &mut HashSet<String>,
) -> usize {
    if !visited.insert(node_id.to_string()) {
        return 0;
    }

    options_by_id
        .get(node_id)
        .and_then(|option| option.parent_node_id.as_deref())
        .and_then(|parent_id| {
            options_by_id
                .contains_key(parent_id)
                .then(|| 1 + form_node_filter_depth(parent_id, options_by_id, visited))
        })
        .unwrap_or(0)
}

/// Handles the form node filter path behavior.
pub(crate) fn form_node_filter_path(
    node_id: &str,
    options_by_id: &BTreeMap<String, FormNodeFilterOption>,
    visited: &mut HashSet<String>,
) -> String {
    if !visited.insert(node_id.to_string()) {
        return options_by_id
            .get(node_id)
            .map(|option| option.name.clone())
            .unwrap_or_else(|| node_id.to_string());
    }

    let Some(option) = options_by_id.get(node_id) else {
        return node_id.to_string();
    };

    option
        .parent_node_id
        .as_deref()
        .filter(|parent_id| options_by_id.contains_key(*parent_id))
        .map(|parent_id| {
            format!(
                "{} / {}",
                form_node_filter_path(parent_id, options_by_id, visited),
                option.name
            )
        })
        .unwrap_or_else(|| option.name.clone())
}

/// Handles the form matches node filter behavior.
pub(crate) fn form_matches_node_filter(
    form: &FormSummary,
    selected_node_id: Option<&str>,
    options: &[FormNodeFilterOption],
) -> bool {
    let Some(selected_node_id) = selected_node_id else {
        return true;
    };

    form.versions.iter().any(|version| {
        version.assignment_nodes.iter().any(|node| {
            node.node_id == selected_node_id
                || form_node_is_descendant_of_selected(&node.node_id, selected_node_id, options)
        })
    })
}

/// Handles the form node is descendant of selected behavior.
pub(crate) fn form_node_is_descendant_of_selected(
    node_id: &str,
    selected_node_id: &str,
    options: &[FormNodeFilterOption],
) -> bool {
    let by_id = options
        .iter()
        .map(|option| (option.id.as_str(), option))
        .collect::<HashMap<_, _>>();
    let mut current_parent = by_id
        .get(node_id)
        .and_then(|option| option.parent_node_id.as_deref());
    let mut visited = HashSet::<String>::new();

    while let Some(parent_id) = current_parent {
        if parent_id == selected_node_id {
            return true;
        }
        if !visited.insert(parent_id.to_string()) {
            return false;
        }
        current_parent = by_id
            .get(parent_id)
            .and_then(|option| option.parent_node_id.as_deref());
    }

    false
}

/// Handles the visible form node filter options behavior.
pub(crate) fn visible_form_node_filter_options(
    options: &[FormNodeFilterOption],
    selected_node_id: Option<&str>,
    query: &str,
) -> Vec<FormNodeFilterOption> {
    let query = query.trim().to_lowercase();

    options
        .iter()
        .filter(|option| {
            if selected_node_id == Some(option.id.as_str()) {
                return false;
            }

            let Some(selected_node_id) = selected_node_id else {
                return true;
            };

            form_node_is_descendant_of_selected(&option.id, selected_node_id, options)
        })
        .filter(|option| {
            query.is_empty()
                || option.name.to_lowercase().contains(&query)
                || option.path.to_lowercase().contains(&query)
        })
        .cloned()
        .collect()
}

/// Handles the indented node label behavior.
pub(crate) fn indented_node_label(option: &FormNodeFilterOption) -> String {
    format!("{}{}", " ".repeat(option.depth), option.name)
}

/// Handles the unique filter options behavior.
pub(crate) fn unique_filter_options(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut options = values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

/// Handles the slug from label behavior.
pub(crate) fn slug_from_label(label: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for character in label
        .trim()
        .chars()
        .flat_map(|character| character.to_lowercase())
    {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    slug
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the unique slug from label behavior.
pub(crate) fn unique_slug_from_label(label: &str, existing_slugs: &[String]) -> String {
    let base = slug_from_label(label);
    if base.is_empty() {
        return String::new();
    }

    let existing = existing_slugs.iter().cloned().collect::<HashSet<_>>();
    if !existing.contains(&base) {
        return base;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !existing.contains(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the existing form slugs behavior.
pub(crate) fn existing_form_slugs(forms: &[FormSummary]) -> Vec<String> {
    forms.iter().map(|form| form.slug.clone()).collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the existing form slugs for update behavior.
pub(crate) fn existing_form_slugs_for_update(
    forms: &[FormSummary],
    current_form_id: &str,
) -> Vec<String> {
    forms
        .iter()
        .filter(|form| form.id != current_form_id)
        .map(|form| form.slug.clone())
        .collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the existing workflow slugs behavior.
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
