//! Display formatting helpers for the Forms feature.
//!
//! Keep label, class, and summary formatting here when it depends on Forms domain values but not on route state.

use crate::features::forms::types::{FormDefinition, FormVersionSummary, RenderedField};
use crate::features::shared::FormAttachmentLink;
use crate::utils::text::{nonempty_text, sentence_label};

/// Handles the form version desc sort key behavior.
pub(crate) fn form_version_desc_sort_key(version: &FormVersionSummary) -> (i32, i32, i32, String) {
    (
        version.version_major.unwrap_or(-1),
        version.version_minor.unwrap_or(-1),
        version.version_patch.unwrap_or(-1),
        version.published_at.clone().unwrap_or_default(),
    )
}

/// Handles the form field count label behavior.
pub(crate) fn form_field_count_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| version.field_count.to_string())
        .unwrap_or_else(|| "-".to_string())
}

/// Handles the form status label behavior.
pub(crate) fn form_status_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| sentence_label(&version.status))
        .unwrap_or_else(|| "No versions".to_string())
}

/// Handles the form attached to label behavior.
pub(crate) fn form_attached_to_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| {
            version
                .assignment_nodes
                .iter()
                .map(|node| node.node_name.as_str())
                .filter(|name| !name.trim().is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Not attached".to_string())
}

/// Handles the form definition scope label behavior.
pub(crate) fn form_definition_scope_label(form: &FormDefinition) -> String {
    nonempty_text(form.scope_node_type_name.as_deref(), "All node types")
}

/// Handles the form attached nodes behavior.
pub(crate) fn form_attached_nodes(version: Option<&FormVersionSummary>) -> Vec<FormAttachmentLink> {
    version
        .map(|version| {
            version
                .assignment_nodes
                .iter()
                .filter(|node| !node.node_name.trim().is_empty())
                .map(|node| FormAttachmentLink {
                    href: format!("/organization/{}", node.node_id),
                    label: node.node_name.clone(),
                    title: if node.node_path.trim().is_empty() {
                        node.node_name.clone()
                    } else {
                        node.node_path.replace(" / ", " > ")
                    },
                })
                .collect::<Vec<_>>()
        })
        .filter(|nodes| !nodes.is_empty())
        .unwrap_or_default()
}

/// Handles the rendered field type label behavior.
pub(crate) fn rendered_field_type_label(field_type: &str) -> String {
    match field_type {
        "static_text" => "Static text".to_string(),
        "single_choice" => "Single choice".to_string(),
        "multi_choice" => "Multi choice".to_string(),
        "boolean" => "Checkbox".to_string(),
        _ => sentence_label(field_type),
    }
}

/// Handles the rendered field layout label behavior.
pub(crate) fn rendered_field_layout_label(field: &RenderedField) -> String {
    format!(
        "Row {}, Column {} · {} wide x {} tall",
        field.grid_row, field.grid_column, field.grid_width, field.grid_height
    )
}
