//! Owns the features::workflows::assignments::types module behavior.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAssignmentSummary {
    pub(crate) id: String,
    pub(crate) workflow_id: String,
    pub(crate) workflow_name: String,
    pub(crate) workflow_version_id: String,
    pub(crate) workflow_version_label: Option<String>,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_version_id: String,
    pub(crate) form_version_label: Option<String>,
    pub(crate) workflow_step_id: String,
    pub(crate) workflow_step_title: String,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) account_id: String,
    pub(crate) account_display_name: String,
    pub(crate) account_email: String,
    pub(crate) is_active: bool,
    pub(crate) has_draft: bool,
    pub(crate) has_submitted: bool,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct PendingWorkflowWork {
    pub(crate) workflow_assignment_id: String,
    pub(crate) workflow_id: String,
    pub(crate) workflow_name: String,
    pub(crate) workflow_description: String,
    pub(crate) workflow_version_id: String,
    pub(crate) workflow_version_label: Option<String>,
    pub(crate) workflow_step_title: String,
    pub(crate) workflow_step_position: i32,
    pub(crate) workflow_step_count: i64,
    pub(crate) next_workflow_step_title: Option<String>,
    pub(crate) next_workflow_step_form_name: Option<String>,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_version_id: String,
    pub(crate) form_version_label: Option<String>,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) account_id: String,
    pub(crate) account_display_name: String,
    pub(crate) assigned_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAssignmentCandidate {
    pub(crate) workflow_version_id: String,
    pub(crate) workflow_id: String,
    pub(crate) workflow_name: String,
    pub(crate) workflow_version_label: Option<String>,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) node_path: String,
    pub(crate) label: String,
    pub(crate) step_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAssigneeOption {
    pub(crate) account_id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct BulkWorkflowAssignmentPayload {
    pub(crate) workflow_version_id: String,
    pub(crate) node_id: String,
    pub(crate) account_ids: Vec<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateWorkflowAssignmentPayload {
    pub(crate) node_id: String,
    pub(crate) account_id: String,
    pub(crate) is_active: bool,
}
