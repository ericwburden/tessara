//! API and feature-local data contracts for the Datasets feature.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SessionAccount {
    pub(crate) capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetSummary {
    pub(crate) id: String,
    pub(crate) current_revision_id: Option<String>,
    pub(crate) current_version_major: Option<i32>,
    pub(crate) current_version_minor: Option<i32>,
    pub(crate) current_version_patch: Option<i32>,
    #[serde(default)]
    pub(crate) major_versions: Vec<i32>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    pub(crate) materialized_row_count: Option<i64>,
    pub(crate) materialized_at: Option<String>,
    pub(crate) visibility_nodes: Vec<DatasetVisibilityNode>,
    pub(crate) source_count: i64,
    pub(crate) field_count: i64,
    #[serde(default)]
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
    #[serde(default)]
    pub(crate) revisions: Vec<DatasetRevisionFieldSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetDefinition {
    pub(crate) id: String,
    pub(crate) current_revision_id: Option<String>,
    pub(crate) current_revision_number: Option<i32>,
    pub(crate) current_revision_label: Option<String>,
    pub(crate) current_version_major: Option<i32>,
    pub(crate) current_version_minor: Option<i32>,
    pub(crate) current_version_patch: Option<i32>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    pub(crate) initial_source: Option<DatasetSourcePayload>,
    pub(crate) operations: Vec<DatasetOperationPayload>,
    pub(crate) restriction_policy: Option<DatasetRestrictionPolicyPayload>,
    pub(crate) generated_sql: Option<String>,
    pub(crate) materialized_schema: Option<String>,
    pub(crate) materialized_table: Option<String>,
    pub(crate) materialized_row_count: Option<i64>,
    pub(crate) materialized_at: Option<String>,
    pub(crate) visibility_nodes: Vec<DatasetVisibilityNode>,
    pub(crate) sources: Vec<DatasetSourceDefinition>,
    pub(crate) fields: Vec<DatasetFieldDefinition>,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetRevisionMetadata {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DatasetRevisionStatus {
    Draft,
    Published,
    Superseded,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DatasetVersionImpact {
    Patch,
    Minor,
    Major,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DatasetCompatibilityState {
    Compatible,
    Review,
    Breaking,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DatasetDependencyKind {
    Dataset,
    ComponentVersion,
    Dashboard,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DatasetDependencyBindingMode {
    ExactRevision,
    MajorLine,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DatasetCarryForwardState {
    Safe,
    ManualReview,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetCompatibilityFinding {
    pub(crate) version_impact: DatasetVersionImpact,
    pub(crate) state: DatasetCompatibilityState,
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) field_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetCompatibilitySummary {
    pub(crate) state: DatasetCompatibilityState,
    pub(crate) major_count: usize,
    pub(crate) minor_count: usize,
    pub(crate) patch_count: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetDependencyImpact {
    pub(crate) kind: DatasetDependencyKind,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) pinned_revision_id: String,
    pub(crate) pinned_version_major: Option<i32>,
    pub(crate) binding_mode: DatasetDependencyBindingMode,
    pub(crate) carry_forward_state: DatasetCarryForwardState,
    pub(crate) message: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum DatasetSemanticBump {
    Initial,
    Patch,
    Minor,
    Major,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetDependencySummary {
    pub(crate) dependency_count: usize,
    pub(crate) dataset_count: usize,
    pub(crate) component_version_count: usize,
    pub(crate) dashboard_count: usize,
    pub(crate) carry_forward_state: DatasetCarryForwardState,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetRevisionSummary {
    pub(crate) id: String,
    pub(crate) dataset_id: String,
    pub(crate) version_number: i32,
    pub(crate) version_label: String,
    pub(crate) version_major: Option<i32>,
    pub(crate) version_minor: Option<i32>,
    pub(crate) version_patch: Option<i32>,
    pub(crate) semantic_bump: Option<DatasetSemanticBump>,
    pub(crate) started_new_major_line: Option<bool>,
    pub(crate) status: DatasetRevisionStatus,
    pub(crate) is_current: bool,
    pub(crate) created_at: String,
    pub(crate) published_at: Option<String>,
    pub(crate) materialized_at: Option<String>,
    pub(crate) materialized_row_count: Option<i64>,
    pub(crate) output_field_count: usize,
    pub(crate) compatibility: DatasetCompatibilitySummary,
    pub(crate) dependencies: DatasetDependencySummary,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetRevisionDetail {
    pub(crate) id: String,
    pub(crate) dataset_id: String,
    pub(crate) version_number: i32,
    pub(crate) version_label: String,
    #[serde(default)]
    pub(crate) revision_notes: String,
    pub(crate) version_major: Option<i32>,
    pub(crate) version_minor: Option<i32>,
    pub(crate) version_patch: Option<i32>,
    pub(crate) semantic_bump: Option<DatasetSemanticBump>,
    pub(crate) started_new_major_line: Option<bool>,
    pub(crate) force_new_major_version: bool,
    pub(crate) status: DatasetRevisionStatus,
    pub(crate) is_current: bool,
    pub(crate) created_at: String,
    pub(crate) published_at: Option<String>,
    pub(crate) materialized_schema: Option<String>,
    pub(crate) materialized_table: Option<String>,
    pub(crate) materialized_row_count: Option<i64>,
    pub(crate) materialized_at: Option<String>,
    pub(crate) metadata: DatasetRevisionMetadata,
    pub(crate) initial_source: DatasetSourcePayload,
    pub(crate) operations: Vec<DatasetOperationPayload>,
    pub(crate) restriction_policy: Option<DatasetRestrictionPolicyPayload>,
    pub(crate) generated_sql: Option<String>,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
    pub(crate) compatibility: DatasetCompatibilitySummary,
    pub(crate) compatibility_findings: Vec<DatasetCompatibilityFinding>,
    pub(crate) dependencies: DatasetDependencySummary,
    pub(crate) dependency_impacts: Vec<DatasetDependencyImpact>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[allow(dead_code)]
pub(crate) struct DatasetDraftRevisionResponse {
    pub(crate) dataset_id: String,
    pub(crate) revision_id: String,
    pub(crate) status: DatasetRevisionStatus,
    pub(crate) compatibility: DatasetCompatibilitySummary,
    pub(crate) dependencies: DatasetDependencySummary,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[allow(dead_code)]
pub(crate) struct DatasetPublishRevisionResponse {
    pub(crate) dataset_id: String,
    pub(crate) revision_id: String,
    pub(crate) superseded_revision_id: Option<String>,
    pub(crate) version_label: String,
    pub(crate) version_major: i32,
    pub(crate) version_minor: i32,
    pub(crate) version_patch: i32,
    pub(crate) semantic_bump: DatasetSemanticBump,
    pub(crate) started_new_major_line: bool,
    pub(crate) status: DatasetRevisionStatus,
    pub(crate) dependencies: DatasetDependencySummary,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DatasetRevisionLabelRequest {
    pub(crate) version_label: Option<String>,
    pub(crate) revision_notes: Option<String>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DatasetRevisionOptionsRequest {
    pub(crate) force_new_major_version: bool,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetRevisionLabelResponse {
    pub(crate) dataset_id: String,
    pub(crate) revision_id: String,
    pub(crate) version_label: String,
    pub(crate) revision_notes: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetVisibilityNode {
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) node_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetRevisionFieldSummary {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetSourceDefinition {
    pub(crate) source_alias: String,
    pub(crate) form_id: Option<String>,
    pub(crate) form_name: Option<String>,
    pub(crate) form_version_id: Option<String>,
    pub(crate) source_dataset_id: Option<String>,
    pub(crate) dataset_revision_id: Option<String>,
    pub(crate) dataset_version_major: Option<i32>,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetFieldDefinition {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) source_alias: String,
    pub(crate) source_field_key: String,
    pub(crate) field_type: String,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetTable {
    pub(crate) rows: Vec<DatasetTableRow>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetTableRow {
    pub(crate) submission_id: String,
    pub(crate) node_name: String,
    pub(crate) source_alias: String,
    pub(crate) values: BTreeMap<String, Option<String>>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetSqlPreviewResponse {
    pub(crate) generated_sql: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetFormOption {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) versions: Vec<DatasetFormVersionOption>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetFormVersionOption {
    pub(crate) id: String,
    pub(crate) version_label: Option<String>,
    pub(crate) status: String,
    pub(crate) version_major: Option<i32>,
    pub(crate) field_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetRenderedForm {
    pub(crate) form_version_id: String,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) sections: Vec<DatasetRenderedSection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetRenderedSection {
    pub(crate) fields: Vec<DatasetRenderedField>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetRenderedField {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    #[serde(default)]
    pub(crate) value_options: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct NodeResponse {
    pub(crate) id: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) parent_node_name: Option<String>,
    pub(crate) name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct DatasetUserOption {
    pub(crate) display_name: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DatasetPayload {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    pub(crate) force_new_major_version: bool,
    pub(crate) visibility_node_ids: Vec<String>,
    pub(crate) initial_source: DatasetSourcePayload,
    pub(crate) operations: Vec<DatasetOperationPayload>,
    pub(crate) restriction_policy: Option<DatasetRestrictionPolicyPayload>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum DatasetSourcePayload {
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
    DatasetMajor {
        alias: String,
        dataset_id: String,
        version_major: i32,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum DatasetOperationPayload {
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
pub(crate) struct DatasetProjectionFieldPayload {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) input_field_key: Option<String>,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct DatasetAggregationMetricPayload {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) function: String,
    pub(crate) source_field_key: Option<String>,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct DatasetRowPickerPayload {
    pub(crate) sort_fields: Vec<DatasetRowPickerSortPayload>,
    #[serde(default = "default_row_picker_direction")]
    pub(crate) direction: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct DatasetRowPickerSortPayload {
    pub(crate) field_key: String,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct DatasetRowFilterPayload {
    pub(crate) field_key: String,
    pub(crate) operator: String,
    pub(crate) value_mode: String,
    pub(crate) value: Option<String>,
    pub(crate) value_field_key: Option<String>,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct DatasetCalculatedFieldPayload {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) base_field_key: String,
    pub(crate) functions: Vec<DatasetCalculationFunctionPayload>,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct DatasetCalculationFunctionPayload {
    pub(crate) function: String,
    pub(crate) argument: Option<String>,
    #[serde(default = "default_calculation_argument_mode")]
    pub(crate) argument_mode: String,
    #[serde(default)]
    pub(crate) argument_field_key: Option<String>,
    pub(crate) position: i32,
}

fn default_calculation_argument_mode() -> String {
    "value".into()
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct DatasetRestrictionPolicyPayload {
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct DatasetJoinKeyPayload {
    pub(crate) left_field: String,
    pub(crate) right_field: String,
}
