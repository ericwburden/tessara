//! Components feature DTOs.
#![cfg_attr(not(feature = "hydrate"), allow(dead_code))]

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
    pub(crate) current_version_id: Option<String>,
    pub(crate) current_version_label: Option<String>,
    pub(crate) current_component_type: Option<String>,
    #[serde(default)]
    pub(crate) draft_version_id: Option<String>,
    #[serde(default)]
    pub(crate) draft_version_label: Option<String>,
}

#[derive(Clone, Deserialize, PartialEq)]
pub(crate) struct ComponentDefinition {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
    pub(crate) versions: Vec<ComponentVersionSummary>,
}

#[derive(Clone, Deserialize, PartialEq)]
pub(crate) struct ComponentVersionSummary {
    pub(crate) id: String,
    pub(crate) component_id: String,
    pub(crate) dataset_id: String,
    pub(crate) dataset_version_major: i32,
    pub(crate) binding_mode: String,
    pub(crate) component_type: String,
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) lifecycle_state: Option<String>,
    #[serde(default)]
    pub(crate) resource_revision: i64,
    #[serde(default)]
    pub(crate) successor_version_id: Option<String>,
    pub(crate) version_label: String,
    #[serde(default)]
    pub(crate) version_note: String,
    pub(crate) config: Value,
}

#[derive(Clone, Serialize)]
pub(crate) struct UpdateComponentRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
}

#[derive(Clone, Serialize)]
pub(crate) struct SaveComponentEditRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) component_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) draft_version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) published_version_id: Option<String>,
    pub(crate) action: String,
    pub(crate) component: UpdateComponentRequest,
    pub(crate) version: CreateComponentVersionRequest,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentValidationResponse {
    pub(crate) valid: bool,
    pub(crate) findings: Vec<ComponentValidationFinding>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentValidationFinding {
    pub(crate) code: String,
    pub(crate) severity: String,
    pub(crate) field_path: Option<String>,
    pub(crate) message: String,
}

#[derive(Clone, Serialize)]
pub(crate) struct CreateComponentVersionRequest {
    pub(crate) dataset_id: Option<String>,
    pub(crate) dataset_version_major: Option<i32>,
    pub(crate) component_type: String,
    pub(crate) config: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) version_note: Option<String>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct IdResponse {
    pub(crate) id: String,
}

#[derive(Clone, Serialize)]
pub(crate) struct ComponentLifecycleRequest {
    pub(crate) action: String,
    pub(crate) expected_resource_revision: i64,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentLifecycleResponse {
    pub(crate) id: String,
    pub(crate) lifecycle_state: String,
    pub(crate) resource_revision: i64,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetSummary {
    pub(crate) id: String,
    pub(crate) current_version_major: Option<i32>,
    pub(crate) major_versions: Vec<i32>,
    pub(crate) name: String,
    pub(crate) slug: String,
    #[serde(default)]
    pub(crate) grain: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) provenance: DatasetProvenanceSummary,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
    #[serde(default)]
    pub(crate) revisions: Vec<DatasetRevisionFieldSummary>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetRevisionFieldSummary {
    pub(crate) version_number: i32,
    pub(crate) version_major: Option<i32>,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetDistinctValues {
    pub(crate) dataset_id: String,
    pub(crate) version_major: i32,
    pub(crate) field: String,
    pub(crate) values: Vec<String>,
}

#[derive(Clone, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetProvenanceSummary {
    #[serde(default)]
    pub(crate) forms: Vec<DatasetProvenanceItem>,
    #[serde(default)]
    pub(crate) datasets: Vec<DatasetProvenanceItem>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetProvenanceItem {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: Option<String>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetFieldDefinition {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
}
