use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct OperationsStatus {
    pub(super) summary: OperationsSummary,
    pub(super) workflow_assignments: Vec<WorkflowAssignmentStatus>,
    pub(super) dataset_readiness: DatasetReadiness,
    pub(super) reporting_data: ReportingDataStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct OperationsSummary {
    pub(super) open_workflow_assignment_count: i64,
    pub(super) draft_response_count: i64,
    pub(super) dataset_attention_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct WorkflowAssignmentStatus {
    pub(super) workflow_instance_id: String,
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
    pub(super) started_at: String,
    pub(super) completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetReadiness {
    pub(super) datasets: Vec<DatasetStatus>,
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
pub(super) struct ReportingDataStatus {
    pub(super) status: String,
    pub(super) reporting_node_count: i64,
    pub(super) submitted_response_count: i64,
    pub(super) response_value_count: i64,
    pub(super) message: String,
}
