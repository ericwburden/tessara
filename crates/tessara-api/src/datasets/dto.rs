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
    pub(crate) version_label: Option<String>,
    #[serde(default)]
    pub(crate) force_new_major_version: bool,
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<Uuid>,
    pub(crate) initial_source: DatasetSourceRequest,
    pub(crate) operations: Vec<DatasetOperationRequest>,
    #[serde(default)]
    pub(crate) restriction_policy: Option<DatasetRestrictionPolicyRequest>,
}

/// Dataset metadata captured with one immutable or draft revision snapshot.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetRevisionMetadata {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    #[serde(default)]
    pub(crate) force_new_major_version: bool,
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<Uuid>,
}

impl From<&CreateDatasetRequest> for DatasetRevisionMetadata {
    fn from(value: &CreateDatasetRequest) -> Self {
        Self {
            name: value.name.clone(),
            slug: value.slug.clone(),
            grain: value.grain.clone(),
            force_new_major_version: value.force_new_major_version,
            visibility_node_ids: value.visibility_node_ids.clone(),
        }
    }
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
    DatasetMajor {
        alias: String,
        dataset_id: Uuid,
        version_major: i32,
    },
}

/// One ordered dataset operation applied after source composition.
///
/// The operations array order is authoritative. Each operation's `position`
/// must match its zero-based index in that array; the backend validates it as a
/// client-side consistency guard and does not sort by `position`.
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DatasetOperationRequest {
    AddSource {
        source: DatasetSourceRequest,
        add_type: String,
        #[serde(default)]
        join_keys: Vec<DatasetJoinKeyRequest>,
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
    pub(crate) current_version_major: Option<i32>,
    pub(crate) current_version_minor: Option<i32>,
    pub(crate) current_version_patch: Option<i32>,
    pub(crate) major_versions: Vec<i32>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    pub(crate) materialized_row_count: Option<i64>,
    pub(crate) materialized_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) visibility_nodes: Vec<DatasetVisibilityNodeSummary>,
    pub(crate) source_count: i64,
    pub(crate) field_count: i64,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
    pub(crate) revisions: Vec<DatasetRevisionFieldSummary>,
}

/// Dataset detail with the current revision's sources and fields.
#[derive(Serialize)]
pub struct DatasetDefinition {
    pub(crate) id: Uuid,
    pub(crate) current_revision_id: Option<Uuid>,
    pub(crate) current_revision_number: Option<i32>,
    pub(crate) current_revision_label: Option<String>,
    pub(crate) current_version_major: Option<i32>,
    pub(crate) current_version_minor: Option<i32>,
    pub(crate) current_version_patch: Option<i32>,
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

/// Stable revision lifecycle state exposed to web clients.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetRevisionStatus {
    Draft,
    Published,
    Superseded,
}

/// Semantic version impact of a changelog finding between revisions.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DatasetVersionImpact {
    Patch,
    Minor,
    Major,
}

/// Roll-up compatibility state for one candidate revision.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetCompatibilityState {
    Compatible,
    Review,
    Breaking,
}

/// Type of downstream asset pinned to a dataset revision.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetDependencyKind {
    Dataset,
    ComponentVersion,
    Dashboard,
}

/// Binding mode used by a downstream dependency.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetDependencyBindingMode {
    ExactRevision,
    MajorLine,
}

/// Sprint 3C carry-forward guidance; no downstream asset is repointed by publish.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetCarryForwardState {
    Safe,
    ManualReview,
    Blocked,
}

/// One changelog finding produced for a draft or historical revision.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatasetCompatibilityFinding {
    pub(crate) version_impact: DatasetVersionImpact,
    pub(crate) state: DatasetCompatibilityState,
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) field_key: Option<String>,
}

/// Summary of changelog impact for a revision.
#[derive(Clone, Serialize)]
pub struct DatasetCompatibilitySummary {
    pub(crate) state: DatasetCompatibilityState,
    pub(crate) major_count: usize,
    pub(crate) minor_count: usize,
    pub(crate) patch_count: usize,
}

/// Downstream dependency pinned to the current published revision.
#[derive(Clone, Serialize)]
pub struct DatasetDependencyImpact {
    pub(crate) kind: DatasetDependencyKind,
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) pinned_revision_id: Uuid,
    pub(crate) pinned_version_major: Option<i32>,
    pub(crate) binding_mode: DatasetDependencyBindingMode,
    pub(crate) carry_forward_state: DatasetCarryForwardState,
    pub(crate) message: String,
}

/// Semantic version bump assigned when a dataset revision is published.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatasetSemanticBump {
    Initial,
    Patch,
    Minor,
    Major,
}

/// Aggregate dependency counts and carry-forward state.
#[derive(Clone, Serialize)]
pub struct DatasetDependencySummary {
    pub(crate) dependency_count: usize,
    pub(crate) dataset_count: usize,
    pub(crate) component_version_count: usize,
    pub(crate) dashboard_count: usize,
    pub(crate) carry_forward_state: DatasetCarryForwardState,
}

/// Compact row for revision history.
#[derive(Serialize)]
pub struct DatasetRevisionSummary {
    pub(crate) id: Uuid,
    pub(crate) dataset_id: Uuid,
    pub(crate) version_number: i32,
    pub(crate) version_label: String,
    pub(crate) version_major: Option<i32>,
    pub(crate) version_minor: Option<i32>,
    pub(crate) version_patch: Option<i32>,
    pub(crate) semantic_bump: Option<DatasetSemanticBump>,
    pub(crate) started_new_major_line: Option<bool>,
    pub(crate) force_new_major_version: bool,
    pub(crate) status: DatasetRevisionStatus,
    pub(crate) is_current: bool,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) materialized_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) materialized_row_count: Option<i64>,
    pub(crate) output_field_count: usize,
    pub(crate) compatibility: DatasetCompatibilitySummary,
    pub(crate) dependencies: DatasetDependencySummary,
}

/// Full revision detail and review payload.
#[derive(Serialize)]
pub struct DatasetRevisionDetail {
    pub(crate) id: Uuid,
    pub(crate) dataset_id: Uuid,
    pub(crate) version_number: i32,
    pub(crate) version_label: String,
    pub(crate) revision_notes: String,
    pub(crate) version_major: Option<i32>,
    pub(crate) version_minor: Option<i32>,
    pub(crate) version_patch: Option<i32>,
    pub(crate) semantic_bump: Option<DatasetSemanticBump>,
    pub(crate) started_new_major_line: Option<bool>,
    pub(crate) force_new_major_version: bool,
    pub(crate) status: DatasetRevisionStatus,
    pub(crate) is_current: bool,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) materialized_schema: Option<String>,
    pub(crate) materialized_table: Option<String>,
    pub(crate) materialized_row_count: Option<i64>,
    pub(crate) materialized_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) metadata: DatasetRevisionMetadata,
    pub(crate) initial_source: DatasetSourceRequest,
    pub(crate) operations: Vec<DatasetOperationRequest>,
    pub(crate) restriction_policy: Option<DatasetRestrictionPolicyRequest>,
    pub(crate) generated_sql: Option<String>,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
    pub(crate) compatibility: DatasetCompatibilitySummary,
    pub(crate) compatibility_findings: Vec<DatasetCompatibilityFinding>,
    pub(crate) dependencies: DatasetDependencySummary,
    pub(crate) dependency_impacts: Vec<DatasetDependencyImpact>,
}

/// Payload for renaming a revision label without changing its definition snapshot.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateDatasetRevisionLabelRequest {
    pub(crate) version_label: Option<String>,
    #[serde(default)]
    pub(crate) revision_notes: Option<String>,
}

/// Payload for changing draft revision review options.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateDatasetRevisionOptionsRequest {
    pub(crate) force_new_major_version: bool,
}

/// Response returned after renaming a revision label.
#[derive(Serialize)]
pub struct DatasetRevisionLabelResponse {
    pub(crate) dataset_id: Uuid,
    pub(crate) revision_id: Uuid,
    pub(crate) version_label: String,
    pub(crate) revision_notes: String,
}

/// Response returned after saving an existing dataset edit as a draft revision.
#[derive(Serialize)]
pub struct DatasetDraftRevisionResponse {
    pub(crate) dataset_id: Uuid,
    pub(crate) revision_id: Uuid,
    pub(crate) status: DatasetRevisionStatus,
    pub(crate) compatibility: DatasetCompatibilitySummary,
    pub(crate) dependencies: DatasetDependencySummary,
}

/// Response returned after publishing a draft revision.
#[derive(Serialize)]
pub struct DatasetPublishRevisionResponse {
    pub(crate) dataset_id: Uuid,
    pub(crate) revision_id: Uuid,
    pub(crate) superseded_revision_id: Option<Uuid>,
    pub(crate) semantic_version: String,
    pub(crate) version_label: String,
    pub(crate) version_major: i32,
    pub(crate) version_minor: i32,
    pub(crate) version_patch: i32,
    pub(crate) semantic_bump: DatasetSemanticBump,
    pub(crate) started_new_major_line: bool,
    pub(crate) status: DatasetRevisionStatus,
    pub(crate) dependencies: DatasetDependencySummary,
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

/// Output-field snapshot for one dataset revision.
#[derive(Clone, Serialize)]
pub struct DatasetRevisionFieldSummary {
    pub(crate) id: Uuid,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
}

/// Source definition included in a dataset revision.
#[derive(Serialize)]
pub struct DatasetSourceDefinition {
    pub(crate) id: Uuid,
    pub(crate) source_alias: String,
    pub(crate) form_id: Option<Uuid>,
    pub(crate) form_name: Option<String>,
    pub(crate) form_version_id: Option<Uuid>,
    pub(crate) source_dataset_id: Option<Uuid>,
    pub(crate) dataset_revision_id: Option<Uuid>,
    pub(crate) dataset_version_major: Option<i32>,
    pub(crate) position: i32,
}

/// Exposed field definition included in a dataset revision.
#[derive(Clone, Deserialize, Serialize)]
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
