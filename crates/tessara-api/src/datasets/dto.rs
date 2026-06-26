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
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<Uuid>,
    pub(crate) initial_source: DatasetSourceRequest,
    pub(crate) operations: Vec<DatasetOperationRequest>,
    #[serde(default)]
    pub(crate) restriction_policy: Option<DatasetRestrictionPolicyRequest>,
}

/// One source stream that can initialize or extend a dataset query pipeline.
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DatasetSourceRequest {
    Form {
        alias: String,
        form_id: Uuid,
        form_version_id: Uuid,
    },
    Dataset {
        alias: String,
        dataset_id: Uuid,
        dataset_revision_id: Uuid,
    },
}

/// One ordered dataset operation applied after source composition.
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DatasetOperationRequest {
    JoinSource {
        source: DatasetSourceRequest,
        operation: String,
        #[serde(default)]
        join_keys: Vec<DatasetJoinKeyRequest>,
        #[serde(default)]
        position: i32,
    },
    UnionSource {
        source: DatasetSourceRequest,
        #[serde(default)]
        position: i32,
    },
    UnionAllSource {
        source: DatasetSourceRequest,
        #[serde(default)]
        position: i32,
    },
    Projection {
        #[serde(default)]
        fields: Vec<DatasetProjectionFieldRequest>,
        #[serde(default)]
        position: i32,
    },
    Aggregation {
        #[serde(default)]
        group_fields: Vec<String>,
        #[serde(default)]
        metrics: Vec<DatasetAggregationMetricRequest>,
        #[serde(default)]
        row_picker: Option<DatasetRowPickerRequest>,
        #[serde(default)]
        position: i32,
    },
    CalculatedFields {
        #[serde(default)]
        fields: Vec<DatasetCalculatedFieldRequest>,
        #[serde(default)]
        position: i32,
    },
    Filter {
        #[serde(default)]
        filters: Vec<DatasetRowFilterRequest>,
        #[serde(default)]
        position: i32,
    },
}

/// One projected field emitted by a projection operation.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetProjectionFieldRequest {
    pub(crate) key: String,
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) input_field_key: Option<String>,
    pub(crate) position: i32,
}

/// Aggregation applied by an ordered operation.
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

/// One output-field row filter applied after projection and before aggregation.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetRowFilterRequest {
    pub(crate) field_key: String,
    pub(crate) operator: String,
    pub(crate) value_mode: String,
    #[serde(default)]
    pub(crate) value: Option<String>,
    pub(crate) value_field_key: Option<String>,
    pub(crate) position: i32,
}

/// One calculated output field produced from a base output field and function chain.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetCalculatedFieldRequest {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) base_field_key: String,
    #[serde(default)]
    pub(crate) functions: Vec<DatasetCalculationFunctionRequest>,
    pub(crate) position: i32,
}

/// One ordered function application in a calculated-field pipeline.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetCalculationFunctionRequest {
    pub(crate) function: String,
    #[serde(default)]
    pub(crate) argument: Option<String>,
    #[serde(default = "default_argument_mode")]
    pub(crate) argument_mode: String,
    #[serde(default)]
    pub(crate) argument_field_key: Option<String>,
    pub(crate) position: i32,
}

fn default_argument_mode() -> String {
    "value".into()
}

/// Row tier policy used to enforce dataset restrictions after materialization.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetRestrictionPolicyRequest {
    #[serde(default)]
    pub(crate) internal_field_key: Option<String>,
    #[serde(default)]
    pub(crate) restricted_field_key: Option<String>,
    #[serde(default)]
    pub(crate) confidential_field_key: Option<String>,
}

fn default_row_picker_direction() -> String {
    "lowest".into()
}

/// One explicit join key pair.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetJoinKeyRequest {
    pub(crate) left_field: String,
    pub(crate) right_field: String,
}

/// Compact dataset row used by list surfaces.
#[derive(Serialize)]
pub struct DatasetSummary {
    pub(crate) id: Uuid,
    pub(crate) current_revision_id: Option<Uuid>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
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
    pub(crate) initial_source: Option<DatasetSourceRequest>,
    pub(crate) operations: Vec<DatasetOperationRequest>,
    pub(crate) restriction_policy: Option<DatasetRestrictionPolicyRequest>,
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
    pub(crate) form_version_id: Option<Uuid>,
    pub(crate) dataset_revision_id: Option<Uuid>,
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
