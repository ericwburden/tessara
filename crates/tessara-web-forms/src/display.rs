//! Display formatting helpers for the Forms feature.
//!
//! Keep label, class, and summary formatting here when it depends on Forms domain values but not on route state.

use crate::support::text::{nonempty_text, sentence_label};
use crate::types::{FormAttachmentLink, FormDefinition, FormVersionSummary, RenderedField};
use icons::FileText;
use leptos::prelude::*;
use tessara_module_ui::empty_view;

pub(crate) fn status_badge_class(status: &str) -> &'static str {
    match status {
        "published" | "done" | "active" | "submitted" => "status-badge is-success",
        "draft" | "in_progress" => "status-badge is-warning",
        "error" | "archived" => "status-badge is-danger",
        _ => "status-badge is-info",
    }
}

pub(crate) fn node_count_label(count: usize) -> String {
    if count == 1 {
        "1 Node".to_string()
    } else {
        format!("{count} Nodes")
    }
}

pub(crate) fn form_workflow_revision_label_from_option(label: Option<String>) -> String {
    label
        .as_deref()
        .map(form_workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
}

fn form_workflow_revision_label_from_raw(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }

    if let Ok(revision) = trimmed.parse::<u64>() {
        return revision.to_string();
    }

    trimmed
        .split('.')
        .next()
        .and_then(|part| part.trim().parse::<u64>().ok())
        .map(|revision| revision.to_string())
        .unwrap_or_else(|| trimmed.to_string())
}

#[component]
pub(crate) fn FormWorkflowSourceMarker(source: String) -> impl IntoView {
    if source == "generated_form" {
        view! {
            <span
                class="workflow-source-marker"
                title="Single-Form, Generated Workflow"
                aria-label="Single-Form, Generated Workflow"
            >
                <FileText class="workflow-source-marker__icon"/>
                <span>"Single-form"</span>
            </span>
        }
        .into_any()
    } else {
        empty_view()
    }
}

pub(crate) fn form_version_desc_sort_key(version: &FormVersionSummary) -> (i32, i32, i32, String) {
    (
        version.version_major.unwrap_or(-1),
        version.version_minor.unwrap_or(-1),
        version.version_patch.unwrap_or(-1),
        version.published_at.clone().unwrap_or_default(),
    )
}

pub(crate) fn form_field_count_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| version.field_count.to_string())
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn form_status_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| sentence_label(&version.status))
        .unwrap_or_else(|| "No versions".to_string())
}

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

pub(crate) fn form_definition_scope_label(form: &FormDefinition) -> String {
    nonempty_text(form.scope_node_type_name.as_deref(), "All node types")
}

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

pub(crate) fn rendered_field_type_label(field_type: &str) -> String {
    match field_type {
        "static_text" => "Static text".to_string(),
        "single_choice" => "Single choice".to_string(),
        "multi_choice" => "Multi choice".to_string(),
        "boolean" => "Checkbox".to_string(),
        _ => sentence_label(field_type),
    }
}

pub(crate) fn rendered_field_layout_label(field: &RenderedField) -> String {
    format!(
        "Row {}, Column {} · {} wide x {} tall",
        field.grid_row, field.grid_column, field.grid_width, field.grid_height
    )
}
