//! API and feature-local data contracts for the Datasets feature.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct SessionAccount {
    pub(in crate::features::datasets) capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetSummary {
    pub(in crate::features::datasets) id: String,
    pub(in crate::features::datasets) current_revision_id: Option<String>,
    pub(in crate::features::datasets) name: String,
    pub(in crate::features::datasets) slug: String,
    pub(in crate::features::datasets) grain: String,
    pub(in crate::features::datasets) materialized_row_count: Option<i64>,
    pub(in crate::features::datasets) materialized_at: Option<String>,
    pub(in crate::features::datasets) visibility_nodes: Vec<DatasetVisibilityNode>,
    pub(in crate::features::datasets) source_count: i64,
    pub(in crate::features::datasets) field_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetDefinition {
    pub(in crate::features::datasets) id: String,
    pub(in crate::features::datasets) current_revision_id: Option<String>,
    pub(in crate::features::datasets) name: String,
    pub(in crate::features::datasets) slug: String,
    pub(in crate::features::datasets) grain: String,
    pub(in crate::features::datasets) initial_source: Option<DatasetSourcePayload>,
    pub(in crate::features::datasets) operations: Vec<DatasetOperationPayload>,
    pub(in crate::features::datasets) restriction_policy: Option<DatasetRestrictionPolicyPayload>,
    pub(in crate::features::datasets) generated_sql: Option<String>,
    pub(in crate::features::datasets) materialized_schema: Option<String>,
    pub(in crate::features::datasets) materialized_table: Option<String>,
    pub(in crate::features::datasets) materialized_row_count: Option<i64>,
    pub(in crate::features::datasets) materialized_at: Option<String>,
    pub(in crate::features::datasets) visibility_nodes: Vec<DatasetVisibilityNode>,
    pub(in crate::features::datasets) sources: Vec<DatasetSourceDefinition>,
    pub(in crate::features::datasets) fields: Vec<DatasetFieldDefinition>,
    pub(in crate::features::datasets) output_fields: Vec<DatasetFieldDefinition>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetVisibilityNode {
    pub(in crate::features::datasets) node_id: String,
    pub(in crate::features::datasets) node_name: String,
    pub(in crate::features::datasets) node_type_name: String,
    pub(in crate::features::datasets) parent_node_id: Option<String>,
    pub(in crate::features::datasets) node_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetSourceDefinition {
    pub(in crate::features::datasets) source_alias: String,
    pub(in crate::features::datasets) form_id: Option<String>,
    pub(in crate::features::datasets) form_name: Option<String>,
    pub(in crate::features::datasets) form_version_id: Option<String>,
    pub(in crate::features::datasets) dataset_revision_id: Option<String>,
    pub(in crate::features::datasets) position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetFieldDefinition {
    pub(in crate::features::datasets) key: String,
    pub(in crate::features::datasets) label: String,
    pub(in crate::features::datasets) source_alias: String,
    pub(in crate::features::datasets) source_field_key: String,
    pub(in crate::features::datasets) field_type: String,
    pub(in crate::features::datasets) position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetTable {
    pub(in crate::features::datasets) rows: Vec<DatasetTableRow>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetTableRow {
    pub(in crate::features::datasets) submission_id: String,
    pub(in crate::features::datasets) node_name: String,
    pub(in crate::features::datasets) source_alias: String,
    pub(in crate::features::datasets) values: BTreeMap<String, Option<String>>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetSqlPreviewResponse {
    pub(in crate::features::datasets) generated_sql: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetFormOption {
    pub(in crate::features::datasets) id: String,
    pub(in crate::features::datasets) name: String,
    pub(in crate::features::datasets) versions: Vec<DatasetFormVersionOption>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetFormVersionOption {
    pub(in crate::features::datasets) id: String,
    pub(in crate::features::datasets) version_label: Option<String>,
    pub(in crate::features::datasets) status: String,
    pub(in crate::features::datasets) version_major: Option<i32>,
    pub(in crate::features::datasets) field_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetRenderedForm {
    pub(in crate::features::datasets) form_version_id: String,
    pub(in crate::features::datasets) form_id: String,
    pub(in crate::features::datasets) form_name: String,
    pub(in crate::features::datasets) sections: Vec<DatasetRenderedSection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetRenderedSection {
    pub(in crate::features::datasets) fields: Vec<DatasetRenderedField>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetRenderedField {
    pub(in crate::features::datasets) key: String,
    pub(in crate::features::datasets) label: String,
    pub(in crate::features::datasets) field_type: String,
    #[serde(default)]
    pub(in crate::features::datasets) value_options: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct NodeResponse {
    pub(in crate::features::datasets) id: String,
    pub(in crate::features::datasets) node_type_name: String,
    pub(in crate::features::datasets) parent_node_id: Option<String>,
    pub(in crate::features::datasets) parent_node_name: Option<String>,
    pub(in crate::features::datasets) name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetUserOption {
    pub(in crate::features::datasets) display_name: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
pub(in crate::features::datasets) struct DatasetPayload {
    pub(in crate::features::datasets) name: String,
    pub(in crate::features::datasets) slug: String,
    pub(in crate::features::datasets) grain: String,
    pub(in crate::features::datasets) visibility_node_ids: Vec<String>,
    pub(in crate::features::datasets) initial_source: DatasetSourcePayload,
    pub(in crate::features::datasets) operations: Vec<DatasetOperationPayload>,
    pub(in crate::features::datasets) restriction_policy: Option<DatasetRestrictionPolicyPayload>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(in crate::features::datasets) enum DatasetSourcePayload {
    Form {
        alias: String,
        form_id: String,
        form_version_id: String,
    },
    Dataset {
        alias: String,
        dataset_id: String,
        dataset_revision_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(in crate::features::datasets) enum DatasetOperationPayload {
    AddSource {
        source: DatasetSourcePayload,
        add_type: String,
        #[serde(default)]
        join_keys: Vec<DatasetJoinKeyPayload>,
        position: i32,
    },
    Projection {
        fields: Vec<DatasetProjectionFieldPayload>,
        position: i32,
    },
    Aggregation {
        group_fields: Vec<String>,
        metrics: Vec<DatasetAggregationMetricPayload>,
        row_picker: Option<DatasetRowPickerPayload>,
        position: i32,
    },
    CalculatedFields {
        fields: Vec<DatasetCalculatedFieldPayload>,
        position: i32,
    },
    Filter {
        filters: Vec<DatasetRowFilterPayload>,
        position: i32,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetProjectionFieldPayload {
    pub(in crate::features::datasets) key: String,
    pub(in crate::features::datasets) label: String,
    pub(in crate::features::datasets) input_field_key: Option<String>,
    pub(in crate::features::datasets) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetAggregationMetricPayload {
    pub(in crate::features::datasets) key: String,
    pub(in crate::features::datasets) label: String,
    pub(in crate::features::datasets) function: String,
    pub(in crate::features::datasets) source_field_key: Option<String>,
    pub(in crate::features::datasets) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetRowPickerPayload {
    pub(in crate::features::datasets) sort_fields: Vec<DatasetRowPickerSortPayload>,
    #[serde(default = "default_row_picker_direction")]
    pub(in crate::features::datasets) direction: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetRowPickerSortPayload {
    pub(in crate::features::datasets) field_key: String,
    pub(in crate::features::datasets) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetRowFilterPayload {
    pub(in crate::features::datasets) field_key: String,
    pub(in crate::features::datasets) operator: String,
    pub(in crate::features::datasets) value_mode: String,
    pub(in crate::features::datasets) value: Option<String>,
    pub(in crate::features::datasets) value_field_key: Option<String>,
    pub(in crate::features::datasets) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetCalculatedFieldPayload {
    pub(in crate::features::datasets) key: String,
    pub(in crate::features::datasets) label: String,
    pub(in crate::features::datasets) base_field_key: String,
    pub(in crate::features::datasets) functions: Vec<DatasetCalculationFunctionPayload>,
    pub(in crate::features::datasets) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetCalculationFunctionPayload {
    pub(in crate::features::datasets) function: String,
    pub(in crate::features::datasets) argument: Option<String>,
    #[serde(default = "default_calculation_argument_mode")]
    pub(in crate::features::datasets) argument_mode: String,
    #[serde(default)]
    pub(in crate::features::datasets) argument_field_key: Option<String>,
    pub(in crate::features::datasets) position: i32,
}

fn default_calculation_argument_mode() -> String {
    "value".into()
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetRestrictionPolicyPayload {
    #[serde(default)]
    pub(in crate::features::datasets) internal_field_key: Option<String>,
    #[serde(default)]
    pub(in crate::features::datasets) restricted_field_key: Option<String>,
    #[serde(default)]
    pub(in crate::features::datasets) confidential_field_key: Option<String>,
}

fn default_row_picker_direction() -> String {
    "lowest".into()
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(in crate::features::datasets) struct DatasetJoinKeyPayload {
    pub(in crate::features::datasets) left_field: String,
    pub(in crate::features::datasets) right_field: String,
}
