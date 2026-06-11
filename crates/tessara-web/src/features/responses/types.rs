//! Owns the features::responses::types module behavior.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionSummary {
    pub(crate) id: String,
    pub(crate) form_id: String,
    pub(crate) form_version_id: String,
    pub(crate) form_name: String,
    pub(crate) workflow_name: Option<String>,
    pub(crate) workflow_description: Option<String>,
    pub(crate) workflow_step_position: Option<i32>,
    pub(crate) workflow_step_count: Option<i64>,
    pub(crate) workflow_steps_completed: Option<i64>,
    pub(crate) current_workflow_step_title: Option<String>,
    pub(crate) next_workflow_step_title: Option<String>,
    pub(crate) next_workflow_step_form_name: Option<String>,
    pub(crate) assigned_to_display_name: Option<String>,
    pub(crate) version_label: String,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) status: String,
    pub(crate) value_count: i64,
    pub(crate) created_at: String,
    pub(crate) last_modified_at: String,
    pub(crate) submitted_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionDetail {
    pub(crate) id: String,
    pub(crate) form_id: String,
    pub(crate) form_version_id: String,
    pub(crate) form_name: String,
    pub(crate) version_label: String,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) status: String,
    pub(crate) created_at: String,
    pub(crate) submitted_at: Option<String>,
    #[serde(default)]
    pub(crate) values: Vec<SubmissionValueDetail>,
    #[serde(default)]
    pub(crate) audit_events: Vec<SubmissionAuditEventSummary>,
    pub(crate) runtime: Option<SubmissionRuntimeDetail>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionRuntimeDetail {
    pub(crate) workflow_name: String,
    pub(crate) current_step_title: String,
    pub(crate) current_step_position: i32,
    pub(crate) step_count: i64,
    pub(crate) next_step_title: Option<String>,
    #[serde(default)]
    pub(crate) history: Vec<SubmissionRuntimeStepHistory>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionRuntimeStepHistory {
    pub(crate) title: String,
    pub(crate) form_name: String,
    pub(crate) status: String,
    pub(crate) position: i32,
    pub(crate) completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionValueDetail {
    pub(crate) field_id: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
    pub(crate) value: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionAuditEventSummary {
    pub(crate) event_type: String,
    pub(crate) account_email: Option<String>,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AssignmentResponseStartOption {
    pub(crate) workflow_assignment_id: String,
    pub(crate) workflow_name: String,
    pub(crate) workflow_version_label: Option<String>,
    pub(crate) workflow_step_title: String,
    pub(crate) workflow_step_position: i32,
    pub(crate) workflow_step_count: i64,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_version_id: String,
    pub(crate) form_version_label: Option<String>,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) account_id: String,
    pub(crate) account_display_name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AssignmentResponseStartOptions {
    #[serde(default)]
    pub(crate) assignments: Vec<AssignmentResponseStartOption>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct SaveSubmissionValuesPayload {
    pub(crate) values: HashMap<String, Value>,
}
