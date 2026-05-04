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
    pub version_label: String,
    pub node_id: Uuid,
    pub node_name: String,
    pub status: String,
    pub value_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
