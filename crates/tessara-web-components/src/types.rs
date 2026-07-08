//! Components feature DTOs.
#![cfg_attr(not(feature = "hydrate"), allow(dead_code))]

use std::collections::BTreeMap;

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
    pub(crate) draft_version_id: Option<String>,
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
    pub(crate) version_label: String,
    #[serde(default)]
    pub(crate) version_note: String,
    pub(crate) config: Value,
}

#[derive(Clone, Deserialize, PartialEq)]
pub(crate) struct ComponentTable {
    pub(crate) component_id: String,
    pub(crate) component_version_id: String,
    pub(crate) dataset_id: String,
    pub(crate) dataset_version_major: i32,
    pub(crate) component_type: String,
    pub(crate) materialization_state: String,
    pub(crate) columns: Vec<ComponentTableColumn>,
    pub(crate) rows: Vec<ComponentTableRow>,
    pub(crate) pagination: ComponentTablePagination,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentTablePagination {
    pub(crate) page_size: usize,
    pub(crate) next_cursor: Option<String>,
    pub(crate) has_more: bool,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentTableColumn {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentTableRow {
    pub(crate) row_id: String,
    pub(crate) values: BTreeMap<String, Option<String>>,
}

#[derive(Clone, Serialize)]
pub(crate) struct CreateComponentRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
    pub(crate) version: Option<CreateComponentVersionRequest>,
}

#[derive(Clone, Serialize)]
pub(crate) struct UpdateComponentRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
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
