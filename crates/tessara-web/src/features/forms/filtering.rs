//! Form-owned filtering and slug helpers.

use crate::features::forms::FormSummary;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone)]
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

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the existing form slugs behavior.
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
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
