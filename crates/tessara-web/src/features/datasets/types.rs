//! Data contracts for the Datasets feature.
//!
//! Keep API response shapes, request payloads, and feature-local value objects here when they are owned by Datasets.

mod editor;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub(super) use editor::{DatasetDesignerSelection, DatasetFieldDraft, DatasetSourceDraft};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct SessionAccount {
    pub(super) capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetSummary {
    pub(super) id: String,
    pub(super) current_revision_id: Option<String>,
    pub(super) name: String,
    pub(super) slug: String,
    pub(super) grain: String,
    pub(super) composition_mode: String,
    pub(super) materialized_row_count: Option<i64>,
    pub(super) materialized_at: Option<String>,
    pub(super) visibility_nodes: Vec<DatasetVisibilityNode>,
    pub(super) source_count: i64,
    pub(super) field_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetDefinition {
    pub(super) id: String,
    pub(super) current_revision_id: Option<String>,
    pub(super) name: String,
    pub(super) slug: String,
    pub(super) grain: String,
    pub(super) composition_mode: String,
    pub(super) definition_ast: Option<DatasetExpressionPayload>,
    pub(super) generated_sql: Option<String>,
    pub(super) materialized_schema: Option<String>,
    pub(super) materialized_table: Option<String>,
    pub(super) materialized_row_count: Option<i64>,
    pub(super) materialized_at: Option<String>,
    pub(super) visibility_nodes: Vec<DatasetVisibilityNode>,
    pub(super) sources: Vec<DatasetSourceDefinition>,
    pub(super) fields: Vec<DatasetFieldDefinition>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetVisibilityNode {
    pub(super) node_id: String,
    pub(super) node_name: String,
    pub(super) node_type_name: String,
    pub(super) parent_node_id: Option<String>,
    pub(super) node_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetSourceDefinition {
    pub(super) source_alias: String,
    pub(super) form_id: Option<String>,
    pub(super) form_name: Option<String>,
    pub(super) form_version_major: Option<i32>,
    pub(super) dataset_revision_id: Option<String>,
    pub(super) selection_rule: String,
    pub(super) position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetFieldDefinition {
    pub(super) key: String,
    pub(super) label: String,
    pub(super) source_alias: String,
    pub(super) source_field_key: String,
    pub(super) field_type: String,
    pub(super) position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetTable {
    pub(super) rows: Vec<DatasetTableRow>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetTableRow {
    pub(super) submission_id: String,
    pub(super) node_name: String,
    pub(super) source_alias: String,
    pub(super) values: BTreeMap<String, Option<String>>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetSqlPreviewResponse {
    pub(super) generated_sql: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetFormOption {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) versions: Vec<DatasetFormVersionOption>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetFormVersionOption {
    pub(super) id: String,
    pub(super) version_label: Option<String>,
    pub(super) status: String,
    pub(super) version_major: Option<i32>,
    pub(super) field_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetRenderedForm {
    pub(super) form_version_id: String,
    pub(super) form_id: String,
    pub(super) form_name: String,
    pub(super) sections: Vec<DatasetRenderedSection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetRenderedSection {
    pub(super) fields: Vec<DatasetRenderedField>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct DatasetRenderedField {
    pub(super) key: String,
    pub(super) label: String,
    pub(super) field_type: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(super) struct NodeResponse {
    pub(super) id: String,
    pub(super) node_type_name: String,
    pub(super) parent_node_name: Option<String>,
    pub(super) name: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
pub(super) struct DatasetPayload {
    pub(super) name: String,
    pub(super) slug: String,
    pub(super) grain: String,
    pub(super) composition_mode: String,
    pub(super) visibility_node_ids: Vec<String>,
    pub(super) definition_ast: DatasetExpressionPayload,
    pub(super) fields: Vec<DatasetFieldPayload>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum DatasetExpressionPayload {
    Form {
        alias: String,
        form_id: String,
        form_version_major: Option<i32>,
        selection_rule: String,
    },
    Dataset {
        alias: String,
        dataset_id: String,
        dataset_revision_id: String,
    },
    Operation {
        alias: String,
        operation: String,
        left: Box<DatasetExpressionPayload>,
        right: Box<DatasetExpressionPayload>,
        join_keys: Vec<DatasetJoinKeyPayload>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(super) struct DatasetJoinKeyPayload {
    pub(super) left_field: String,
    pub(super) right_field: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
pub(super) struct DatasetFieldPayload {
    pub(super) key: String,
    pub(super) label: String,
    pub(super) source_alias: String,
    pub(super) source_field_key: String,
    pub(super) position: i32,
}
