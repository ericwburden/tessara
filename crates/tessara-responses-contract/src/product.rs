use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const RESPONSE_PRODUCT_SCHEMA_VERSION: u16 = 1;
pub const RESPONSE_IDEMPOTENCY_HEADER: &str = "idempotency-key";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseFormSnapshot {
    pub form_version_id: Uuid,
    pub form_id: Uuid,
    pub form_name: String,
    pub version_label: Option<String>,
    pub status: String,
    pub sections: Vec<ResponseFormSection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseFormSection {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub position: i32,
    pub fields: Vec<ResponseFormField>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseFormField {
    pub field_id: Uuid,
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub options: Vec<Value>,
    pub position: i32,
    pub grid_row: i32,
    pub grid_column: i32,
    pub grid_width: i32,
    pub grid_height: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseStartOption {
    pub workflow_assignment_id: Uuid,
    pub workflow_name: String,
    pub workflow_version_label: Option<String>,
    pub workflow_step_title: String,
    pub workflow_step_position: i32,
    pub workflow_step_count: i64,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_version_id: Uuid,
    pub form_version_label: Option<String>,
    pub node_id: Uuid,
    pub node_name: String,
    pub account_id: Uuid,
    pub account_display_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseStartOptions {
    pub assignments: Vec<ResponseStartOption>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseSummary {
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
    pub created_at: String,
    pub last_modified_at: String,
    pub submitted_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseRuntimeStepHistory {
    pub title: String,
    pub form_name: String,
    pub status: String,
    pub position: i32,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseRuntimeDetail {
    pub workflow_name: String,
    pub current_step_title: String,
    pub current_step_position: i32,
    pub step_count: i64,
    pub next_step_title: Option<String>,
    pub history: Vec<ResponseRuntimeStepHistory>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseValueDetail {
    pub field_id: Uuid,
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub value: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseAuditEventSummary {
    pub event_type: String,
    pub actor_display_name: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseDetail {
    pub id: Uuid,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub form_name: String,
    pub version_label: String,
    pub node_id: Uuid,
    pub node_name: String,
    pub status: String,
    pub revision: u64,
    pub created_at: String,
    pub submitted_at: Option<String>,
    pub values: Vec<ResponseValueDetail>,
    pub audit_events: Vec<ResponseAuditEventSummary>,
    pub runtime: Option<ResponseRuntimeDetail>,
    pub form: ResponseFormSnapshot,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveResponseValuesRequest {
    pub expected_revision: u64,
    pub values: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseMutationResult {
    pub id: Uuid,
    pub revision: u64,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartResponseRequest {
    pub workflow_assignment_id: Uuid,
}
