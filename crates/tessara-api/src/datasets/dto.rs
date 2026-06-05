use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payload for creating or replacing a dataset definition and revision.
#[derive(Deserialize)]
pub struct CreateDatasetRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    #[serde(default = "super::default_dataset_composition_mode")]
    pub(crate) composition_mode: String,
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<Uuid>,
    pub(crate) sources: Vec<CreateDatasetSourceRequest>,
    pub(crate) fields: Vec<CreateDatasetFieldRequest>,
}

/// One source form selection inside a dataset revision.
#[derive(Deserialize)]
pub struct CreateDatasetSourceRequest {
    pub(crate) source_alias: String,
    pub(crate) form_id: Option<Uuid>,
    pub(crate) form_version_major: Option<i32>,
    pub(crate) selection_rule: String,
}

/// One exposed field mapped from a dataset source field.
#[derive(Deserialize)]
pub struct CreateDatasetFieldRequest {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) source_alias: String,
    pub(crate) source_field_key: String,
    pub(crate) position: i32,
}

/// Compact dataset row used by list surfaces.
#[derive(Serialize)]
pub struct DatasetSummary {
    pub(crate) id: Uuid,
    pub(crate) current_revision_id: Option<Uuid>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    pub(crate) composition_mode: String,
    pub(crate) visibility_nodes: Vec<DatasetVisibilityNodeSummary>,
    pub(crate) source_count: i64,
    pub(crate) field_count: i64,
}

/// Dataset detail with the current revision's sources and fields.
#[derive(Serialize)]
pub struct DatasetDefinition {
    pub(crate) id: Uuid,
    pub(crate) current_revision_id: Option<Uuid>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    pub(crate) composition_mode: String,
    pub(crate) visibility_nodes: Vec<DatasetVisibilityNodeSummary>,
    pub(crate) sources: Vec<DatasetSourceDefinition>,
    pub(crate) fields: Vec<DatasetFieldDefinition>,
}

/// Organization node that makes a dataset visible.
#[derive(Clone, Serialize)]
pub struct DatasetVisibilityNodeSummary {
    pub(crate) node_id: Uuid,
    pub(crate) node_name: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<Uuid>,
    pub(crate) node_path: String,
}

/// Source definition included in a dataset revision.
#[derive(Serialize)]
pub struct DatasetSourceDefinition {
    pub(crate) id: Uuid,
    pub(crate) source_alias: String,
    pub(crate) form_id: Option<Uuid>,
    pub(crate) form_name: Option<String>,
    pub(crate) form_version_major: Option<i32>,
    pub(crate) selection_rule: String,
    pub(crate) position: i32,
}

/// Exposed field definition included in a dataset revision.
#[derive(Serialize)]
pub struct DatasetFieldDefinition {
    pub(crate) id: Uuid,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) source_alias: String,
    pub(crate) source_field_key: String,
    pub(crate) field_type: String,
    pub(crate) position: i32,
}

/// Executed dataset preview table.
#[derive(Serialize)]
pub struct DatasetTable {
    pub(crate) dataset_id: Uuid,
    pub(crate) rows: Vec<DatasetTableRow>,
}

/// One executed dataset row at submission/source grain.
#[derive(Serialize)]
pub struct DatasetTableRow {
    pub(crate) submission_id: String,
    pub(crate) node_name: String,
    pub(crate) source_alias: String,
    pub(crate) values: BTreeMap<String, Option<String>>,
}
