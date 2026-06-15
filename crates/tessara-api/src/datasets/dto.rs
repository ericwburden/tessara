use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payload for creating or replacing a dataset definition and revision.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateDatasetRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    #[serde(default = "super::default_dataset_composition_mode")]
    pub(crate) composition_mode: String,
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<Uuid>,
    pub(crate) definition_ast: DatasetExpressionRequest,
    #[serde(default)]
    pub(crate) aggregation: Option<DatasetAggregationRequest>,
    pub(crate) fields: Vec<CreateDatasetFieldRequest>,
}

/// Optional final aggregation applied after the source expression and field projection.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetAggregationRequest {
    #[serde(default)]
    pub(crate) group_fields: Vec<String>,
    #[serde(default)]
    pub(crate) metrics: Vec<DatasetAggregationMetricRequest>,
    #[serde(default)]
    pub(crate) row_picker: Option<DatasetRowPickerRequest>,
}

/// One aggregate metric emitted by the final dataset query.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetAggregationMetricRequest {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) function: String,
    pub(crate) source_field_key: Option<String>,
    pub(crate) position: i32,
}

/// Selects one representative projected row per aggregation group.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetRowPickerRequest {
    #[serde(default)]
    pub(crate) sort_fields: Vec<DatasetRowPickerSortRequest>,
    #[serde(default = "default_row_picker_direction")]
    pub(crate) direction: String,
}

/// One ordered sort criterion for representative row selection.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetRowPickerSortRequest {
    pub(crate) field_key: String,
    pub(crate) position: i32,
}

fn default_row_picker_direction() -> String {
    "lowest".into()
}

/// Recursive visual dataset query expression.
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DatasetExpressionRequest {
    Form {
        alias: String,
        form_id: Uuid,
        form_version_major: Option<i32>,
        selection_rule: String,
    },
    Dataset {
        alias: String,
        dataset_id: Uuid,
        dataset_revision_id: Uuid,
    },
    Operation {
        alias: String,
        operation: String,
        left: Box<DatasetExpressionRequest>,
        right: Box<DatasetExpressionRequest>,
        #[serde(default)]
        join_keys: Vec<DatasetJoinKeyRequest>,
    },
}

/// One explicit join key pair.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetJoinKeyRequest {
    pub(crate) left_field: String,
    pub(crate) right_field: String,
}

/// One exposed field mapped from a dataset source field.
#[derive(Clone, Deserialize)]
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
    pub(crate) materialized_row_count: Option<i64>,
    pub(crate) materialized_at: Option<chrono::DateTime<chrono::Utc>>,
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
    pub(crate) definition_ast: Option<DatasetExpressionRequest>,
    pub(crate) aggregation: Option<DatasetAggregationResponse>,
    pub(crate) generated_sql: Option<String>,
    pub(crate) materialized_schema: Option<String>,
    pub(crate) materialized_table: Option<String>,
    pub(crate) materialized_row_count: Option<i64>,
    pub(crate) materialized_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) visibility_nodes: Vec<DatasetVisibilityNodeSummary>,
    pub(crate) sources: Vec<DatasetSourceDefinition>,
    pub(crate) fields: Vec<DatasetFieldDefinition>,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
}

/// Stored aggregation definition.
#[derive(Serialize)]
pub struct DatasetAggregationResponse {
    pub(crate) group_fields: Vec<String>,
    pub(crate) metrics: Vec<DatasetAggregationMetricRequest>,
    pub(crate) row_picker: Option<DatasetRowPickerRequest>,
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
    pub(crate) dataset_revision_id: Option<Uuid>,
    pub(crate) selection_rule: String,
    pub(crate) position: i32,
}

/// Exposed field definition included in a dataset revision.
#[derive(Clone, Serialize)]
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

/// Generated SQL preview for an unsaved dataset definition draft.
#[derive(Serialize)]
pub struct DatasetSqlPreview {
    pub(crate) generated_sql: String,
}

/// One executed dataset row at submission/source grain.
#[derive(Serialize)]
pub struct DatasetTableRow {
    pub(crate) submission_id: String,
    pub(crate) node_name: String,
    pub(crate) source_alias: String,
    pub(crate) values: BTreeMap<String, Option<String>>,
}
