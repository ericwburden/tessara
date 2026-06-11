//! Data contracts for the Workflows feature.
//!
//! Keep API response shapes, request payloads, and feature-local value objects here when they are owned by Workflows.

use serde::Deserialize;

use crate::features::workflows::assignments::types::WorkflowAssignmentSummary;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowSummary {
    pub(crate) id: String,
    pub(crate) workflow_node_type_id: String,
    pub(crate) workflow_node_type_name: String,
    #[serde(default)]
    pub(crate) available_nodes: Vec<WorkflowAvailableNodeSummary>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) source: String,
    pub(crate) source_form_id: Option<String>,
    pub(crate) current_version_id: Option<String>,
    pub(crate) current_version_label: Option<String>,
    pub(crate) current_status: Option<String>,
    #[serde(default)]
    pub(crate) assigned_users: Vec<WorkflowAssignedUserSummary>,
    pub(crate) version_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAvailableNodeSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) node_type_name: String,
    pub(crate) path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAssignedUserSummary {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) email: String,
    pub(crate) assignment_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowDefinition {
    pub(crate) id: String,
    pub(crate) workflow_node_type_id: String,
    pub(crate) workflow_node_type_name: String,
    #[serde(default)]
    pub(crate) available_nodes: Vec<WorkflowAvailableNodeSummary>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) source: String,
    pub(crate) source_form_id: Option<String>,
    #[serde(default)]
    pub(crate) versions: Vec<WorkflowVersionSummary>,
    #[serde(default)]
    pub(crate) assignments: Vec<WorkflowAssignmentSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowVersionSummary {
    pub(crate) id: String,
    pub(crate) workflow_revision_label: Option<String>,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) published_at: Option<String>,
    pub(crate) created_at: String,
    pub(crate) step_count: i64,
    #[serde(default)]
    pub(crate) steps: Vec<WorkflowStepSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowStepSummary {
    pub(crate) id: String,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_version_id: String,
    pub(crate) form_version_label: Option<String>,
    pub(crate) title: String,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct WorkflowStepDraft {
    pub(crate) id: usize,
    pub(crate) title: String,
    pub(crate) form_version_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) enum WorkflowSaveIntent {
    Draft,
    Publish,
}
