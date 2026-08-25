//! Data contracts for the Operations feature.
//!
//! Keep API response shapes, request payloads, and feature-local value objects here when they are owned by Operations.

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct OperationsStatus {
    pub(super) summary: OperationsSummary,
    pub(super) workflow_assignments: Vec<WorkflowAssignmentStatus>,
    pub(super) dataset_readiness: DatasetReadiness,
    pub(super) response_owner: ResponseOwnerStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct OperationsSummary {
    pub(super) open_workflow_assignment_count: i64,
    pub(super) draft_response_count: i64,
    pub(super) dataset_attention_count: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct WorkflowAssignmentStatus {
    pub(super) workflow_instance_id: Option<String>,
    pub(super) workflow_assignment_id: String,
    pub(super) workflow_id: String,
    pub(super) workflow_name: String,
    pub(super) workflow_version_label: Option<String>,
    pub(super) node_name: String,
    pub(super) assignee_display_name: String,
    pub(super) assignee_email: String,
    pub(super) assignment_status: String,
    pub(super) current_step_title: Option<String>,
    pub(super) completed_step_count: i64,
    pub(super) total_step_count: i64,
    pub(super) draft_response_count: i64,
    pub(super) submitted_response_count: i64,
    pub(super) started_at: Option<String>,
    pub(super) completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetReadiness {
    pub(super) state: DatasetProviderResultState,
    pub(super) datasets: Vec<DatasetStatus>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum DatasetProviderResultState {
    Available,
    Empty,
    Unavailable,
    Undisclosed,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetStatus {
    pub(super) dataset_id: String,
    pub(super) dataset_name: String,
    pub(super) revision_status: String,
    pub(super) readiness: String,
    pub(super) source_count: i64,
    pub(super) field_count: i64,
    pub(super) ready_response_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct ResponseOwnerStatus {
    pub(super) state: ResponseProviderResultState,
    pub(super) status: Option<ResponseOperationsStatus>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum ResponseProviderResultState {
    Available,
    Empty,
    Unavailable,
    Undisclosed,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct ResponseOperationsStatus {
    pub(super) ready: bool,
    pub(super) forms_binding_compatible: bool,
    pub(super) workflow_binding_compatible: bool,
    pub(super) pending_workflow_event_count: u64,
    pub(super) export_head_sequence: u64,
    pub(super) sanitized_findings: Vec<String>,
}
