use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateDraftRequest {
    pub form_version_id: Uuid,
    pub node_id: Uuid,
    pub delegate_account_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct SaveSubmissionValuesRequest {
    pub values: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub struct ListSubmissionsQuery {
    pub status: Option<String>,
    pub form_id: Option<Uuid>,
    pub node_id: Option<Uuid>,
    pub delegate_account_id: Option<Uuid>,
    pub q: Option<String>,
}

#[derive(Serialize)]
pub struct ResponseNodeSummary {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
pub struct ResponseStartAssignment {
    pub form_id: Uuid,
    pub form_name: String,
    pub form_version_id: Uuid,
    pub version_label: String,
    pub node_id: Uuid,
    pub node_name: String,
    pub delegate_account_id: Option<Uuid>,
    pub delegate_display_name: Option<String>,
}

#[derive(Serialize)]
pub struct ResponseStartOptions {
    pub mode: String,
    pub published_forms: Vec<crate::forms::PublishedFormVersionSummary>,
    pub nodes: Vec<ResponseNodeSummary>,
    pub assignments: Vec<ResponseStartAssignment>,
}

#[derive(Serialize)]
pub struct SubmissionSummary {
    pub id: Uuid,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub form_name: String,
    pub workflow_name: Option<String>,
    pub workflow_description: Option<String>,
    pub workflow_step_position: Option<i32>,
    pub workflow_step_count: Option<i64>,
    pub workflow_steps_completed: Option<i64>,
    pub current_workflow_step_title: Option<String>,
    pub next_workflow_step_title: Option<String>,
    pub next_workflow_step_form_name: Option<String>,
    pub assigned_to_display_name: Option<String>,
    pub version_label: String,
    pub node_id: Uuid,
    pub node_name: String,
    pub status: String,
    pub value_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified_at: chrono::DateTime<chrono::Utc>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct SubmissionDetail {
    pub id: Uuid,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub form_name: String,
    pub version_label: String,
    pub node_id: Uuid,
    pub node_name: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub values: Vec<SubmissionValueDetail>,
    pub audit_events: Vec<SubmissionAuditEventSummary>,
    pub runtime: Option<SubmissionRuntimeDetail>,
}

#[derive(Serialize)]
pub struct SubmissionRuntimeDetail {
    pub workflow_name: String,
    pub current_step_title: String,
    pub current_step_position: i32,
    pub step_count: i64,
    pub next_step_title: Option<String>,
    pub history: Vec<SubmissionRuntimeStepHistory>,
}

#[derive(Serialize)]
pub struct SubmissionRuntimeStepHistory {
    pub title: String,
    pub form_name: String,
    pub status: String,
    pub position: i32,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct SubmissionValueDetail {
    pub field_id: Uuid,
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub value: Option<Value>,
}

#[derive(Serialize)]
pub struct SubmissionAuditEventSummary {
    pub event_type: String,
    pub account_email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
