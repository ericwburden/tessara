//! Display formatting helpers for the Workflows feature.
//!
//! Keep label, class, and summary formatting here when it depends on Workflows domain values but not on route state.

use crate::features::workflows::types::{
    WorkflowAvailableNodeSummary, WorkflowDefinition, WorkflowSummary, WorkflowVersionSummary,
};
use crate::ui::empty_view;
use crate::utils::text::{nonempty_text, sentence_label};
use icons::FileText;
use leptos::prelude::*;

/// Handles the workflow revision label from raw behavior.
pub(crate) fn workflow_revision_label_from_raw(label: &str) -> String {
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

/// Builds a summary label for workflow assigned users.
pub(crate) fn workflow_assigned_users_label(workflow: &WorkflowSummary) -> String {
    if workflow.assigned_users.is_empty() {
        "No active assignments".to_string()
    } else {
        workflow
            .assigned_users
            .iter()
            .map(|user| format!("{} {}", user.display_name, user.email))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Handles the workflow revision label from option behavior.
pub(crate) fn workflow_revision_label_from_option(label: Option<String>) -> String {
    label
        .as_deref()
        .map(workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
}

/// Handles the workflow version label behavior.
pub(crate) fn workflow_version_label(workflow: &WorkflowSummary) -> String {
    workflow
        .current_version_label
        .as_deref()
        .map(workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
}

/// Handles the workflow status key behavior.
pub(crate) fn workflow_status_key(workflow: &WorkflowSummary) -> &str {
    workflow.current_status.as_deref().unwrap_or("none")
}

/// Handles the workflow status label behavior.
pub(crate) fn workflow_status_label(workflow: &WorkflowSummary) -> String {
    workflow
        .current_status
        .as_deref()
        .map(sentence_label)
        .unwrap_or_else(|| "No revisions".to_string())
}

/// Handles the workflow description label behavior.
pub(crate) fn workflow_description_label(workflow: &WorkflowSummary) -> String {
    nonempty_text(Some(workflow.description.as_str()), "No description")
}

/// Handles the workflow available nodes label behavior.
pub(crate) fn workflow_available_nodes_label(nodes: &[WorkflowAvailableNodeSummary]) -> String {
    match nodes.len() {
        1 => nodes[0].name.clone(),
        2 => nodes
            .iter()
            .map(|node| node.name.clone())
            .collect::<Vec<_>>()
            .join(", "),
        count => format!("{count} nodes"),
    }
}

/// Handles the workflow source label behavior.
pub(crate) fn workflow_source_label(source: &str) -> Option<&'static str> {
    if source == "generated_form" {
        Some("Generated single-form")
    } else {
        None
    }
}

#[component]
/// Renders the workflow source marker view.
pub(crate) fn WorkflowSourceMarker(source: String) -> impl IntoView {
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

/// Handles the active workflow definition version behavior.
pub(crate) fn active_workflow_definition_version(
    workflow: &WorkflowDefinition,
) -> Option<&WorkflowVersionSummary> {
    workflow
        .versions
        .iter()
        .find(|version| version.status.eq_ignore_ascii_case("published"))
        .or_else(|| workflow.versions.first())
}

/// Handles the workflow definition version label behavior.
pub(crate) fn workflow_definition_version_label(
    version: Option<&WorkflowVersionSummary>,
) -> String {
    version
        .and_then(|version| version.workflow_revision_label.as_deref())
        .map(workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
}

/// Handles the workflow definition status label behavior.
pub(crate) fn workflow_definition_status_label(version: Option<&WorkflowVersionSummary>) -> String {
    version
        .map(|version| sentence_label(&version.status))
        .unwrap_or_else(|| "No revisions".to_string())
}
