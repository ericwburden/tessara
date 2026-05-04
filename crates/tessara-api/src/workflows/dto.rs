use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateWorkflowRequest {
    pub form_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateWorkflowRequest {
    pub form_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateWorkflowVersionRequest {
    pub form_version_id: Uuid,
    pub title: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateWorkflowAssignmentRequest {
    pub workflow_version_id: Uuid,
    pub node_id: Uuid,
    pub account_id: Uuid,
}

#[derive(Deserialize)]
pub struct UpdateWorkflowAssignmentRequest {
    pub node_id: Uuid,
    pub account_id: Uuid,
    pub is_active: bool,
}

#[derive(Deserialize, Default)]
pub struct WorkflowAssignmentQuery {
    pub workflow_id: Option<Uuid>,
    pub workflow_version_id: Option<Uuid>,
    pub form_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub node_id: Option<Uuid>,
    pub active: Option<bool>,
    pub delegate_account_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct WorkflowSummary {
    pub id: Uuid,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_slug: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub current_version_id: Option<Uuid>,
    pub current_version_label: Option<String>,
    pub current_form_version_id: Option<Uuid>,
    pub current_status: Option<String>,
    pub assignment_count: i64,
}

#[derive(Serialize)]
pub struct WorkflowVersionSummary {
    pub id: Uuid,
    pub form_version_id: Uuid,
    pub form_version_label: Option<String>,
    pub title: String,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct WorkflowAssignmentSummary {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub workflow_version_id: Uuid,
    pub workflow_version_label: Option<String>,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_version_id: Uuid,
    pub form_version_label: Option<String>,
    pub workflow_step_id: Uuid,
    pub workflow_step_title: String,
    pub node_id: Uuid,
    pub node_name: String,
    pub account_id: Uuid,
    pub account_display_name: String,
    pub account_email: String,
    pub is_active: bool,
    pub has_draft: bool,
    pub has_submitted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct WorkflowDefinition {
    pub id: Uuid,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_slug: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub versions: Vec<WorkflowVersionSummary>,
    pub assignments: Vec<WorkflowAssignmentSummary>,
}

#[derive(Serialize)]
pub struct PendingWorkflowWork {
    pub workflow_assignment_id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub workflow_version_id: Uuid,
    pub workflow_version_label: Option<String>,
    pub workflow_step_title: String,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_version_id: Uuid,
    pub form_version_label: Option<String>,
    pub node_id: Uuid,
    pub node_name: String,
    pub account_id: Uuid,
    pub account_display_name: String,
}
