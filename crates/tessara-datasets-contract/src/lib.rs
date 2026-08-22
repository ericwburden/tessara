//! Exact Dataset Module v2 contract.
//!
//! Dataset, DatasetRevision, and DatasetMajorLine references are owned by one
//! enrolled Dataset Module Instance. References grant no authority; provider
//! actions still require a fresh action- and resource-bound grant.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tessara_module_contract::{
    ContractCompatibilityState, ModuleInstanceOwnerState, OwnerDataState,
    ProviderAvailabilityState, ResourceAccessState, ResourceIdentityState, ResourceLifecycleState,
    ResourceObservationV1, ResourceOwner, ResourceOwnerState, ResourceResolutionV1,
    TypedResourceReference,
};
use uuid::Uuid;

pub const DATASET_CONTRACT_SCHEMA_VERSION: u16 = 2;
pub const DATASET_CONTRACT_VERSION: &str = "2.0.0";
pub const DATASET_CONTRACT_ID: &str = "tessara.datasets.dataset-major-line";
pub const DATASET_AUTHORING_CONTRACT_ID: &str = "tessara.datasets.authoring";
pub const DATASET_BINDING_KEY: &str = "tessara.components.dataset-major-line";
pub const DATASET_CORE_BINDING_KEY: &str = "tessara.core.datasets";
pub const DATASET_MODULE_DEFINITION_ID: &str = "tessara.datasets";
pub const DATASET_IDEMPOTENCY_HEADER: &str = "x-idempotency-key";
pub const DATASET_RESOURCE_CONTRACT_ID: &str = "tessara.datasets.dataset";
pub const DATASET_REVISION_CONTRACT_ID: &str = "tessara.datasets.dataset-revision";
pub const DATASET_RESOURCE_TYPE: &str = "tessara.datasets.dataset";
pub const DATASET_REVISION_RESOURCE_TYPE: &str = "tessara.datasets.dataset_revision";
pub const DATASET_MAJOR_LINE_RESOURCE_TYPE: &str = "tessara.datasets.dataset_major_line";
pub const DATASET_BOOTSTRAP_VALIDATION_ACTION: &str = "datasets.bootstrap_validate";
pub const DATASET_BOOTSTRAP_VALIDATION_PATH: &str = "/api/private/datasets/bootstrap-validation";
pub const DATASET_COMPATIBILITY_MATERIALIZATION_NOT_READY: &str = "materialization_not_ready";
pub const DATASET_SOURCE_USAGE_CONTRACT_ID: &str = "tessara.datasets.source-usage";
pub const DATASET_OPERATIONAL_STATUS_CONTRACT_ID: &str = "tessara.datasets.operational-status";
pub const DATASET_RESOURCE_OBSERVATION_CONTRACT_ID: &str = "tessara.datasets.resource-observation";
pub const DATASET_REVERSE_CONTRACT_VERSION: &str = "1.0.0";
pub const DATASET_SOURCE_USAGE_PATH: &str = "/api/private/datasets/source-usage";
pub const DATASET_OPERATIONS_STATUS_PATH: &str = "/api/private/datasets/operations-status";
pub const DATASET_SUMMARY_PATH: &str = "/api/private/datasets/summary";
pub const DATASET_RESOLVE_PATH: &str = "/api/private/datasets/resolve";
pub const DATASET_SOURCE_USAGE_ACTION: &str = "datasets.source_usage";
pub const DATASET_OPERATIONS_STATUS_ACTION: &str = "datasets.operations_status";
pub const DATASET_SUMMARY_ACTION: &str = "datasets.summary";
pub const DATASET_RESOLVE_ACTION: &str = "datasets.resolve";

/// Canonical browser-facing Dataset directory contract. This shape is owned by
/// the Dataset module rather than mirrored by Core or the Dataset UI crate.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductProvenanceSummaryV1 {
    #[serde(default)]
    pub forms: Vec<DatasetProductProvenanceItemV1>,
    #[serde(default)]
    pub datasets: Vec<DatasetProductProvenanceItemV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductProvenanceItemV1 {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductVisibilityNodeV1 {
    pub node_id: String,
    pub node_name: String,
    pub node_type_name: String,
    pub parent_node_id: Option<String>,
    pub node_path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductFieldV1 {
    pub key: String,
    pub label: String,
    pub source_alias: String,
    pub source_field_key: String,
    pub field_type: String,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductRevisionFieldSummaryV1 {
    pub id: String,
    pub version_number: i32,
    pub version_major: Option<i32>,
    pub version_minor: Option<i32>,
    pub version_patch: Option<i32>,
    #[serde(default)]
    pub output_fields: Vec<DatasetProductFieldV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductSummaryV1 {
    pub id: String,
    pub current_revision_id: Option<String>,
    pub current_version_major: Option<i32>,
    pub current_version_minor: Option<i32>,
    pub current_version_patch: Option<i32>,
    #[serde(default)]
    pub major_versions: Vec<i32>,
    pub name: String,
    pub slug: String,
    pub grain: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub provenance: DatasetProductProvenanceSummaryV1,
    pub materialized_row_count: Option<i64>,
    pub materialized_at: Option<String>,
    pub freshness: DatasetProductFreshnessV1,
    pub visibility_nodes: Vec<DatasetProductVisibilityNodeV1>,
    pub source_count: i64,
    pub field_count: i64,
    #[serde(default)]
    pub output_fields: Vec<DatasetProductFieldV1>,
    #[serde(default)]
    pub revisions: Vec<DatasetProductRevisionFieldSummaryV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductLineageNodeV1 {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    pub source_type: String,
    pub version_label: Option<String>,
    pub children: Vec<DatasetProductLineageNodeV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductSourceDefinitionV1 {
    pub source_alias: String,
    pub form_id: Option<String>,
    pub form_name: Option<String>,
    pub form_version_id: Option<String>,
    pub form_version_label: Option<String>,
    pub source_dataset_id: Option<String>,
    pub source_dataset_name: Option<String>,
    pub source_dataset_slug: Option<String>,
    pub dataset_revision_id: Option<String>,
    pub dataset_revision_label: Option<String>,
    pub dataset_version_major: Option<i32>,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetProductSourceV1 {
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetProductOperationV1 {
    AddSource {
        source: DatasetProductSourceV1,
        add_type: String,
        #[serde(default)]
        join_keys: Vec<DatasetProductJoinKeyV1>,
        position: i32,
    },
    Projection {
        fields: Vec<DatasetProductProjectionFieldV1>,
        position: i32,
    },
    Aggregation {
        group_fields: Vec<String>,
        metrics: Vec<DatasetProductAggregationMetricV1>,
        row_picker: Option<DatasetProductRowPickerV1>,
        position: i32,
    },
    CalculatedFields {
        fields: Vec<DatasetProductCalculatedFieldV1>,
        position: i32,
    },
    Filter {
        filters: Vec<DatasetProductRowFilterV1>,
        position: i32,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductProjectionFieldV1 {
    pub key: String,
    pub label: String,
    pub input_field_key: Option<String>,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductAggregationMetricV1 {
    pub key: String,
    pub label: String,
    pub function: String,
    pub source_field_key: Option<String>,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductRowPickerV1 {
    pub sort_fields: Vec<DatasetProductRowPickerSortV1>,
    #[serde(default = "default_product_row_picker_direction")]
    pub direction: String,
}

fn default_product_row_picker_direction() -> String {
    "lowest".into()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductRowPickerSortV1 {
    pub field_key: String,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductRowFilterV1 {
    pub field_key: String,
    pub operator: String,
    pub value_mode: String,
    pub value: Option<String>,
    pub value_field_key: Option<String>,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductCalculatedFieldV1 {
    pub key: String,
    pub label: String,
    pub base_field_key: String,
    pub functions: Vec<DatasetProductCalculationFunctionV1>,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductCalculationFunctionV1 {
    pub function: String,
    pub argument: Option<String>,
    #[serde(default = "default_product_calculation_argument_mode")]
    pub argument_mode: String,
    #[serde(default)]
    pub argument_field_key: Option<String>,
    pub position: i32,
}

fn default_product_calculation_argument_mode() -> String {
    "value".into()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductRestrictionPolicyV1 {
    #[serde(default)]
    pub internal_field_key: Option<String>,
    #[serde(default)]
    pub restricted_field_key: Option<String>,
    #[serde(default)]
    pub confidential_field_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductJoinKeyV1 {
    pub left_field: String,
    pub right_field: String,
}

/// Canonical Dataset authoring payload used by create, draft save, and SQL
/// preview. Source identifiers remain canonical string UUIDs at the public
/// boundary so browser and server consumers share this exact wire type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetAuthoringRequestV1 {
    pub name: String,
    pub slug: String,
    pub grain: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_label: Option<String>,
    #[serde(default)]
    pub force_new_major_version: bool,
    #[serde(default)]
    pub visibility_node_ids: Vec<String>,
    pub initial_source: DatasetProductSourceV1,
    pub operations: Vec<DatasetProductOperationV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restriction_policy: Option<DatasetProductRestrictionPolicyV1>,
}

/// Canonical result of compiling a Dataset authoring snapshot. Preview and
/// persistence use the same compiler; this response exposes only its SQL
/// artifact and cannot become a second authoring model in the browser.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSqlPreviewResponseV1 {
    pub generated_sql: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductDefinitionV1 {
    pub id: String,
    pub current_revision_id: Option<String>,
    pub current_revision_number: Option<i32>,
    pub current_revision_label: Option<String>,
    pub current_version_major: Option<i32>,
    pub current_version_minor: Option<i32>,
    pub current_version_patch: Option<i32>,
    pub name: String,
    pub slug: String,
    pub grain: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub provenance: DatasetProductProvenanceSummaryV1,
    pub lineage: DatasetProductLineageNodeV1,
    pub initial_source: Option<DatasetProductSourceV1>,
    pub operations: Vec<DatasetProductOperationV1>,
    pub restriction_policy: Option<DatasetProductRestrictionPolicyV1>,
    pub generated_sql: Option<String>,
    pub materialized_schema: Option<String>,
    pub materialized_table: Option<String>,
    pub materialized_row_count: Option<i64>,
    pub materialized_at: Option<String>,
    pub freshness: DatasetProductFreshnessV1,
    pub visibility_nodes: Vec<DatasetProductVisibilityNodeV1>,
    pub sources: Vec<DatasetProductSourceDefinitionV1>,
    pub fields: Vec<DatasetProductFieldV1>,
    pub output_fields: Vec<DatasetProductFieldV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetUpdateTagsRequestV1 {
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetMutationIdResponseV1 {
    pub id: String,
}

/// Canonical browser-facing preview table returned by the Dataset owner.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductTableV1 {
    pub rows: Vec<DatasetProductTableRowV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductTableRowV1 {
    pub submission_id: String,
    pub node_name: String,
    pub source_alias: String,
    pub values: BTreeMap<String, Option<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductDistinctValuesQueryV1 {
    pub version_major: i32,
    pub field: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductDistinctValuesV1 {
    pub dataset_id: String,
    pub version_major: i32,
    pub field: String,
    pub values: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductFreshnessV1 {
    pub state: DatasetFreshnessState,
    pub last_checked_at: Option<String>,
    pub last_succeeded_at: Option<String>,
    pub sanitized_failure_code: Option<String>,
}

/// Canonical terminal result of an explicit synchronous Dataset refresh.
/// Provider cursors and authenticated heads remain owner-private; callers can
/// distinguish a semantic no-op from an atomic promotion and retain only the
/// non-sensitive receipt identities needed for replay diagnostics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetRefreshResponseV1 {
    pub dataset_id: String,
    pub changed: bool,
    pub freshness: DatasetProductFreshnessV1,
    pub materialization_receipt_ids: Vec<String>,
}

/// Response returned after an authoring snapshot is saved as the Dataset's
/// single open draft revision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetDraftRevisionResponseV1 {
    pub dataset_id: String,
    pub revision_id: String,
    pub status: DatasetProductRevisionStatusV1,
    pub compatibility: DatasetProductCompatibilitySummaryV1,
    pub dependencies: DatasetProductDependencySummaryV1,
}

/// Canonical result of publishing a saved Dataset revision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetPublishRevisionResponseV1 {
    pub dataset_id: String,
    pub revision_id: String,
    pub superseded_revision_id: Option<String>,
    pub semantic_version: String,
    pub version_label: String,
    pub version_major: i32,
    pub version_minor: i32,
    pub version_patch: i32,
    pub semantic_bump: DatasetProductSemanticBumpV1,
    pub started_new_major_line: bool,
    pub status: DatasetProductRevisionStatusV1,
    pub dependencies: DatasetProductDependencySummaryV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetEditorFormOptionV1 {
    pub id: String,
    pub name: String,
    pub versions: Vec<DatasetEditorFormVersionOptionV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetEditorFormVersionOptionV1 {
    pub id: String,
    pub version_label: Option<String>,
    pub status: String,
    pub version_major: Option<i32>,
    pub field_count: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetEditorRenderedFormV1 {
    pub form_version_id: String,
    pub form_id: String,
    pub form_name: String,
    pub sections: Vec<DatasetEditorRenderedSectionV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetEditorRenderedSectionV1 {
    pub fields: Vec<DatasetEditorRenderedFieldV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetEditorRenderedFieldV1 {
    pub key: String,
    pub label: String,
    pub field_type: String,
    #[serde(default)]
    pub value_options: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetEditorScopeOptionV1 {
    pub id: String,
    pub node_type_name: String,
    pub parent_node_id: Option<String>,
    pub parent_node_name: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetEditorPrincipalOptionV1 {
    pub display_name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetRevisionLabelRequestV1 {
    pub version_label: Option<String>,
    pub revision_notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetRevisionLabelResponseV1 {
    pub dataset_id: String,
    pub revision_id: String,
    pub version_label: String,
    pub revision_notes: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetRevisionOptionsRequestV1 {
    pub force_new_major_version: bool,
}

/// Dataset revision lifecycle exposed by the Dataset owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetProductRevisionStatusV1 {
    Draft,
    Published,
    Superseded,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetProductVersionImpactV1 {
    Patch,
    Minor,
    Major,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetProductCompatibilityStateV1 {
    Compatible,
    Review,
    Breaking,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetProductDependencyKindV1 {
    Dataset,
    ComponentVersion,
    Dashboard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetProductDependencyBindingModeV1 {
    ExactRevision,
    MajorLine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetProductCarryForwardStateV1 {
    Safe,
    ManualReview,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatasetProductSemanticBumpV1 {
    Initial,
    Patch,
    Minor,
    Major,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductRevisionMetadataV1 {
    pub name: String,
    pub slug: String,
    pub grain: String,
    #[serde(default)]
    pub visibility_node_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductCompatibilityFindingV1 {
    pub version_impact: DatasetProductVersionImpactV1,
    pub state: DatasetProductCompatibilityStateV1,
    pub code: String,
    pub message: String,
    pub field_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductCompatibilitySummaryV1 {
    pub state: DatasetProductCompatibilityStateV1,
    pub major_count: usize,
    pub minor_count: usize,
    pub patch_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductDependencyImpactV1 {
    pub kind: DatasetProductDependencyKindV1,
    pub id: String,
    pub name: String,
    pub pinned_revision_id: Option<String>,
    pub pinned_version_major: Option<i32>,
    pub binding_mode: DatasetProductDependencyBindingModeV1,
    pub carry_forward_state: DatasetProductCarryForwardStateV1,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductDependencySummaryV1 {
    pub dependency_count: usize,
    pub dataset_count: usize,
    pub carry_forward_state: DatasetProductCarryForwardStateV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductRevisionSummaryV1 {
    pub id: String,
    pub dataset_id: String,
    pub version_number: i32,
    pub version_label: String,
    pub version_major: Option<i32>,
    pub version_minor: Option<i32>,
    pub version_patch: Option<i32>,
    pub semantic_bump: Option<DatasetProductSemanticBumpV1>,
    pub started_new_major_line: Option<bool>,
    pub force_new_major_version: bool,
    pub status: DatasetProductRevisionStatusV1,
    pub is_current: bool,
    pub created_at: String,
    pub published_at: Option<String>,
    pub materialized_at: Option<String>,
    pub materialized_row_count: Option<i64>,
    pub output_field_count: usize,
    pub compatibility: DatasetProductCompatibilitySummaryV1,
    pub dependencies: DatasetProductDependencySummaryV1,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProductRevisionDetailV1 {
    pub id: String,
    pub dataset_id: String,
    pub version_number: i32,
    pub version_label: String,
    #[serde(default)]
    pub revision_notes: String,
    pub version_major: Option<i32>,
    pub version_minor: Option<i32>,
    pub version_patch: Option<i32>,
    pub semantic_bump: Option<DatasetProductSemanticBumpV1>,
    pub started_new_major_line: Option<bool>,
    pub force_new_major_version: bool,
    pub status: DatasetProductRevisionStatusV1,
    pub is_current: bool,
    pub created_at: String,
    pub published_at: Option<String>,
    pub materialized_schema: Option<String>,
    pub materialized_table: Option<String>,
    pub materialized_row_count: Option<i64>,
    pub materialized_at: Option<String>,
    pub metadata: DatasetProductRevisionMetadataV1,
    pub initial_source: DatasetProductSourceV1,
    pub operations: Vec<DatasetProductOperationV1>,
    pub restriction_policy: Option<DatasetProductRestrictionPolicyV1>,
    pub generated_sql: Option<String>,
    pub output_fields: Vec<DatasetProductFieldV1>,
    pub compatibility: DatasetProductCompatibilitySummaryV1,
    pub compatibility_findings: Vec<DatasetProductCompatibilityFindingV1>,
    pub dependencies: DatasetProductDependencySummaryV1,
    pub dependency_impacts: Vec<DatasetProductDependencyImpactV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DatasetMajorLineReference {
    reference: TypedResourceReference,
}

impl DatasetMajorLineReference {
    pub fn new(
        reference: TypedResourceReference,
    ) -> Result<Self, DatasetMajorLineReferenceValidationError> {
        reference.validate()?;
        if !matches!(reference.owner(), ResourceOwner::ModuleInstance { .. }) {
            return Err(DatasetMajorLineReferenceValidationError::ExpectedModuleInstanceOwner);
        }
        if reference.resource_type().as_str() != DATASET_MAJOR_LINE_RESOURCE_TYPE {
            return Err(
                DatasetMajorLineReferenceValidationError::UnexpectedResourceType {
                    actual: reference.resource_type().as_str().to_string(),
                },
            );
        }
        parse_resource_id(reference.resource_id())?;
        Ok(Self { reference })
    }

    pub fn from_parts(
        installation_id: Uuid,
        module_instance_id: Uuid,
        dataset_id: Uuid,
        major: i32,
    ) -> Result<Self, DatasetMajorLineReferenceValidationError> {
        if major <= 0 {
            return Err(DatasetMajorLineReferenceValidationError::InvalidMajor);
        }
        Self::new(TypedResourceReference::new(
            installation_id,
            ResourceOwner::ModuleInstance {
                installation_id,
                module_instance_id,
            },
            DATASET_MAJOR_LINE_RESOURCE_TYPE
                .parse()
                .expect("Dataset resource type constant is valid"),
            format!("{dataset_id}@{major}"),
        )?)
    }

    pub const fn reference(&self) -> &TypedResourceReference {
        &self.reference
    }

    pub fn dataset_id(&self) -> Uuid {
        parse_resource_id(self.reference.resource_id())
            .expect("validated Dataset major-line reference")
            .0
    }

    pub fn major(&self) -> i32 {
        parse_resource_id(self.reference.resource_id())
            .expect("validated Dataset major-line reference")
            .1
    }

    pub fn module_instance_id(&self) -> Uuid {
        let ResourceOwner::ModuleInstance {
            module_instance_id, ..
        } = self.reference.owner()
        else {
            unreachable!("validated Dataset major-line reference")
        };
        *module_instance_id
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DatasetMajorLineReferenceWire {
    reference: TypedResourceReference,
}

impl<'de> Deserialize<'de> for DatasetMajorLineReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = DatasetMajorLineReferenceWire::deserialize(deserializer)?;
        Self::new(wire.reference).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DatasetMajorLineReferenceValidationError {
    #[error(transparent)]
    InvalidReference(#[from] tessara_module_contract::ReferenceValidationError),
    #[error("Dataset major-line references must be Module Instance owned")]
    ExpectedModuleInstanceOwner,
    #[error(
        "Dataset major-line reference has resource type '{actual}', expected \
         'tessara.datasets.dataset_major_line'"
    )]
    UnexpectedResourceType { actual: String },
    #[error("Dataset major-line resource identity must be '<canonical-uuid>@<positive-major>'")]
    InvalidResourceId,
    #[error("Dataset major-line major version must be positive")]
    InvalidMajor,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DatasetReference {
    reference: TypedResourceReference,
}

impl DatasetReference {
    pub fn new(reference: TypedResourceReference) -> Result<Self, DatasetReferenceValidationError> {
        validate_module_resource_reference(&reference, DATASET_RESOURCE_TYPE)?;
        parse_canonical_uuid(reference.resource_id())?;
        Ok(Self { reference })
    }

    pub fn from_parts(
        installation_id: Uuid,
        module_instance_id: Uuid,
        dataset_id: Uuid,
    ) -> Result<Self, DatasetReferenceValidationError> {
        Self::new(TypedResourceReference::new(
            installation_id,
            ResourceOwner::ModuleInstance {
                installation_id,
                module_instance_id,
            },
            DATASET_RESOURCE_TYPE
                .parse()
                .expect("Dataset resource type constant is valid"),
            dataset_id.to_string(),
        )?)
    }

    pub const fn reference(&self) -> &TypedResourceReference {
        &self.reference
    }
    pub fn dataset_id(&self) -> Uuid {
        parse_canonical_uuid(self.reference.resource_id()).expect("validated Dataset reference")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DatasetReferenceWire {
    reference: TypedResourceReference,
}

impl<'de> Deserialize<'de> for DatasetReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(DatasetReferenceWire::deserialize(deserializer)?.reference)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DatasetRevisionReference {
    reference: TypedResourceReference,
}

impl DatasetRevisionReference {
    pub fn new(reference: TypedResourceReference) -> Result<Self, DatasetReferenceValidationError> {
        validate_module_resource_reference(&reference, DATASET_REVISION_RESOURCE_TYPE)?;
        parse_canonical_uuid(reference.resource_id())?;
        Ok(Self { reference })
    }

    pub fn from_parts(
        installation_id: Uuid,
        module_instance_id: Uuid,
        revision_id: Uuid,
    ) -> Result<Self, DatasetReferenceValidationError> {
        Self::new(TypedResourceReference::new(
            installation_id,
            ResourceOwner::ModuleInstance {
                installation_id,
                module_instance_id,
            },
            DATASET_REVISION_RESOURCE_TYPE
                .parse()
                .expect("Dataset revision resource type constant is valid"),
            revision_id.to_string(),
        )?)
    }

    pub const fn reference(&self) -> &TypedResourceReference {
        &self.reference
    }
    pub fn revision_id(&self) -> Uuid {
        parse_canonical_uuid(self.reference.resource_id())
            .expect("validated DatasetRevision reference")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DatasetRevisionReferenceWire {
    reference: TypedResourceReference,
}

impl<'de> Deserialize<'de> for DatasetRevisionReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(DatasetRevisionReferenceWire::deserialize(deserializer)?.reference)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DatasetReferenceValidationError {
    #[error(transparent)]
    InvalidReference(#[from] tessara_module_contract::ReferenceValidationError),
    #[error("Dataset references must be Module Instance owned")]
    ExpectedModuleInstanceOwner,
    #[error("Dataset reference has resource type '{actual}', expected '{expected}'")]
    UnexpectedResourceType {
        actual: String,
        expected: &'static str,
    },
    #[error("Dataset resource identity must be one canonical UUID")]
    InvalidResourceId,
}

fn validate_module_resource_reference(
    reference: &TypedResourceReference,
    expected_type: &'static str,
) -> Result<(), DatasetReferenceValidationError> {
    reference.validate()?;
    if !matches!(reference.owner(), ResourceOwner::ModuleInstance { .. }) {
        return Err(DatasetReferenceValidationError::ExpectedModuleInstanceOwner);
    }
    if reference.resource_type().as_str() != expected_type {
        return Err(DatasetReferenceValidationError::UnexpectedResourceType {
            actual: reference.resource_type().as_str().to_string(),
            expected: expected_type,
        });
    }
    Ok(())
}

fn parse_canonical_uuid(value: &str) -> Result<Uuid, DatasetReferenceValidationError> {
    let id =
        Uuid::parse_str(value).map_err(|_| DatasetReferenceValidationError::InvalidResourceId)?;
    if id.to_string() != value {
        return Err(DatasetReferenceValidationError::InvalidResourceId);
    }
    Ok(id)
}

fn parse_resource_id(value: &str) -> Result<(Uuid, i32), DatasetMajorLineReferenceValidationError> {
    let Some((dataset, major_text)) = value.split_once('@') else {
        return Err(DatasetMajorLineReferenceValidationError::InvalidResourceId);
    };
    if major_text.contains('@') {
        return Err(DatasetMajorLineReferenceValidationError::InvalidResourceId);
    }
    let dataset_id = Uuid::parse_str(dataset)
        .map_err(|_| DatasetMajorLineReferenceValidationError::InvalidResourceId)?;
    if dataset_id.to_string() != dataset {
        return Err(DatasetMajorLineReferenceValidationError::InvalidResourceId);
    }
    let major = major_text
        .parse::<i32>()
        .map_err(|_| DatasetMajorLineReferenceValidationError::InvalidResourceId)?;
    if major <= 0 || major.to_string() != major_text {
        return Err(DatasetMajorLineReferenceValidationError::InvalidMajor);
    }
    Ok((dataset_id, major))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetAction {
    Catalog,
    ResolveSchema,
    DistinctValues,
    CheckCompatibility,
    Execute,
    ValidateBootstrap,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetFieldContract {
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub restriction_tier: String,
}

/// Compact source metadata carried with Dataset catalog and major-line schema
/// results. These are discoverability fields only; they never participate in
/// authorization or compatibility decisions.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProvenanceSummary {
    pub forms: Vec<DatasetProvenanceItem>,
    pub datasets: Vec<DatasetProvenanceItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProvenanceItem {
    pub id: Uuid,
    pub name: String,
    pub slug: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetMajorLineMetadata {
    pub reference: DatasetMajorLineReference,
    pub dataset_name: String,
    pub dataset_slug: String,
    pub grain: String,
    pub tags: Vec<String>,
    pub provenance: DatasetProvenanceSummary,
    pub materialization_state: String,
    pub fields: Vec<DatasetFieldContract>,
    pub scope_node_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCatalogRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCatalogResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub datasets: Vec<DatasetMajorLineMetadata>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSchemaRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
    pub reference: DatasetMajorLineReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetDistinctValuesRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
    pub reference: DatasetMajorLineReference,
    pub field_key: String,
    pub limit: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetDistinctValuesResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetFieldRequirement {
    pub field_key: String,
    pub accepted_types: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCompatibilityRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
    pub reference: DatasetMajorLineReference,
    pub required_fields: Vec<DatasetFieldRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCompatibilityFinding {
    pub code: String,
    pub field_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCompatibilityResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub compatible: bool,
    pub findings: Vec<DatasetCompatibilityFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetBootstrapValidationItem {
    pub validation_key: String,
    pub reference: DatasetMajorLineReference,
    pub required_fields: Vec<DatasetFieldRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetBootstrapValidationBatch {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub items: Vec<DatasetBootstrapValidationItem>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetBootstrapValidationResult {
    pub validation_key: String,
    pub metadata: DatasetMajorLineMetadata,
    pub compatibility: DatasetCompatibilityResponse,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetBootstrapValidationResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub results: Vec<DatasetBootstrapValidationResult>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetFilterOperator {
    Eq,
    NotEq,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Between,
    NotBetween,
    IsEmpty,
    IsNotEmpty,
    IsNull,
    IsNotNull,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetFilter {
    pub field_key: String,
    pub operator: DatasetFilterOperator,
    #[serde(default)]
    pub value: Option<Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetAggregateFunction {
    Count,
    UniqueCount,
    Sum,
    Average,
    Median,
    Minimum,
    Maximum,
    SingleValue,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetMissingPolicy {
    #[default]
    Omit,
    Zero,
    ExplicitMissing,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSearch {
    pub field_keys: Vec<String>,
    pub query: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetAggregate {
    pub field_key: Option<String>,
    pub function: DatasetAggregateFunction,
    pub output_key: String,
    #[serde(default)]
    pub missing_policy: DatasetMissingPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetSortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSort {
    pub field_key: String,
    pub direction: DatasetSortDirection,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetExecutionRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
    pub reference: DatasetMajorLineReference,
    #[serde(default)]
    pub projection: Vec<String>,
    #[serde(default)]
    pub filters: Vec<DatasetFilter>,
    #[serde(default)]
    pub search: Option<DatasetSearch>,
    #[serde(default)]
    pub group_by: Vec<String>,
    #[serde(default)]
    pub group_missing_policies: BTreeMap<String, DatasetMissingPolicy>,
    #[serde(default)]
    pub aggregates: Vec<DatasetAggregate>,
    #[serde(default)]
    pub order_by: Vec<DatasetSort>,
    pub limit: u32,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetExecutionRow {
    pub row_id: String,
    pub values: BTreeMap<String, Option<Value>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetExecutionResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub materialization_state: String,
    pub fields: Vec<DatasetFieldContract>,
    pub rows: Vec<DatasetExecutionRow>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetProviderResultState {
    Available,
    Empty,
    Unavailable,
    Undisclosed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSourceUsageRequest {
    pub schema_version: u16,
    pub form_id: Uuid,
    pub form_version_id: Option<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSourceUsageItem {
    pub dataset: DatasetReference,
    pub dataset_name: String,
    pub source_alias: String,
    pub pinned_form_version_id: Uuid,
    pub lifecycle_state: String,
    pub semantic_destination: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSourceUsageResponse {
    pub schema_version: u16,
    pub state: DatasetProviderResultState,
    pub items: Vec<DatasetSourceUsageItem>,
}

impl DatasetSourceUsageResponse {
    pub fn validate(&self) -> Result<(), DatasetReverseContractValidationError> {
        validate_reverse_schema(self.schema_version)?;
        match self.state {
            DatasetProviderResultState::Available if self.items.is_empty() => {
                Err(DatasetReverseContractValidationError::AvailableWithoutItems)
            }
            DatasetProviderResultState::Empty
            | DatasetProviderResultState::Unavailable
            | DatasetProviderResultState::Undisclosed
                if !self.items.is_empty() =>
            {
                Err(DatasetReverseContractValidationError::NonAvailableCarriesData)
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetReadinessLabel {
    Ready,
    NoReadyResponses,
    Draft,
    Superseded,
    Unavailable,
    NoPublishedRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetFreshnessState {
    Current,
    Stale,
    Degraded,
    Refreshing,
    Failed,
    NeverMaterialized,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetOperationsStatusRequest {
    pub schema_version: u16,
    pub requested_scope_node_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetOperationsStatusItem {
    pub dataset: DatasetReference,
    pub dataset_name: String,
    pub readiness: DatasetReadinessLabel,
    pub revision_status: Option<String>,
    pub source_count: u64,
    pub field_count: u64,
    pub ready_response_count: u64,
    pub freshness: DatasetFreshnessState,
    pub sanitized_failure_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetOperationsStatusResponse {
    pub schema_version: u16,
    pub state: DatasetProviderResultState,
    pub items: Vec<DatasetOperationsStatusItem>,
}

impl DatasetOperationsStatusResponse {
    pub fn validate(&self) -> Result<(), DatasetReverseContractValidationError> {
        validate_reverse_schema(self.schema_version)?;
        match self.state {
            DatasetProviderResultState::Available if self.items.is_empty() => {
                Err(DatasetReverseContractValidationError::AvailableWithoutItems)
            }
            DatasetProviderResultState::Empty
            | DatasetProviderResultState::Unavailable
            | DatasetProviderResultState::Undisclosed
                if !self.items.is_empty() =>
            {
                Err(DatasetReverseContractValidationError::NonAvailableCarriesData)
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSummaryRequest {
    pub schema_version: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSummaryResponse {
    pub schema_version: u16,
    pub state: DatasetProviderResultState,
    pub dataset_count: Option<u64>,
    pub published_revision_count: Option<u64>,
}

impl DatasetSummaryResponse {
    pub fn validate(&self) -> Result<(), DatasetReverseContractValidationError> {
        validate_reverse_schema(self.schema_version)?;
        let has_counts = self.dataset_count.is_some() && self.published_revision_count.is_some();
        match self.state {
            DatasetProviderResultState::Available | DatasetProviderResultState::Empty
                if !has_counts =>
            {
                Err(DatasetReverseContractValidationError::AvailableWithoutCounts)
            }
            DatasetProviderResultState::Unavailable | DatasetProviderResultState::Undisclosed
                if self.dataset_count.is_some() || self.published_revision_count.is_some() =>
            {
                Err(DatasetReverseContractValidationError::NonAvailableCarriesData)
            }
            DatasetProviderResultState::Available
                if self.dataset_count == Some(0)
                    || self
                        .published_revision_count
                        .zip(self.dataset_count)
                        .is_some_and(|(revisions, datasets)| revisions > datasets) =>
            {
                Err(DatasetReverseContractValidationError::InvalidSummaryCounts)
            }
            DatasetProviderResultState::Empty
                if self.dataset_count != Some(0) || self.published_revision_count != Some(0) =>
            {
                Err(DatasetReverseContractValidationError::InvalidSummaryCounts)
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetResourceObservationRequest {
    pub schema_version: u16,
    pub reference: TypedResourceReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetResourceObservationResponse {
    pub schema_version: u16,
    pub resolution: ResourceResolutionV1,
    pub observation: Option<ResourceObservationV1>,
}

impl DatasetResourceObservationResponse {
    /// Validates the canonical provider response against the exact requested
    /// typed reference. Restricted and unresolved projections must carry no
    /// observation; a disclosed resolution must carry one exact Dataset-owned
    /// reference and its resource contract identity.
    pub fn validate_for(
        &self,
        requested_reference: &TypedResourceReference,
    ) -> Result<(), DatasetReverseContractValidationError> {
        validate_reverse_schema(self.schema_version)?;
        let disclosed_resolution = self.resolution.access_state()
            == ResourceAccessState::Authorized
            && self.resolution.resource_identity_state() == ResourceIdentityState::Resolved
            && self.resolution.compatibility_state() == ContractCompatibilityState::Compatible
            && self.resolution.availability_state() == ProviderAvailabilityState::Available;

        if !disclosed_resolution {
            return if self.observation.is_none() {
                Ok(())
            } else {
                Err(DatasetReverseContractValidationError::RestrictedOrUnresolvedObservation)
            };
        }

        if self.resolution.owner_state()
            != (ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::Live,
                data_state: OwnerDataState::Retained,
            })
            || !matches!(
                self.resolution.resource_lifecycle_state(),
                ResourceLifecycleState::ProviderDefined { .. }
            )
        {
            return Err(DatasetReverseContractValidationError::ObservationOwnerMismatch);
        }

        let observation = self
            .observation
            .as_ref()
            .ok_or(DatasetReverseContractValidationError::MissingResourceObservation)?;
        if observation.reference() != requested_reference {
            return Err(DatasetReverseContractValidationError::ObservationReferenceMismatch);
        }
        let (expected_contract, expected_version) =
            expected_observation_contract(requested_reference)?;
        if observation.provider_contract().contract_id().as_str() != expected_contract
            || observation
                .provider_contract()
                .contract_version()
                .to_string()
                != expected_version
        {
            return Err(DatasetReverseContractValidationError::ObservationContractMismatch);
        }
        Ok(())
    }
}

fn expected_observation_contract(
    reference: &TypedResourceReference,
) -> Result<(&'static str, &'static str), DatasetReverseContractValidationError> {
    if DatasetReference::new(reference.clone()).is_ok() {
        return Ok((
            DATASET_RESOURCE_CONTRACT_ID,
            DATASET_REVERSE_CONTRACT_VERSION,
        ));
    }
    if DatasetRevisionReference::new(reference.clone()).is_ok() {
        return Ok((
            DATASET_REVISION_CONTRACT_ID,
            DATASET_REVERSE_CONTRACT_VERSION,
        ));
    }
    if DatasetMajorLineReference::new(reference.clone()).is_ok() {
        return Ok((DATASET_CONTRACT_ID, DATASET_CONTRACT_VERSION));
    }
    Err(DatasetReverseContractValidationError::ObservationReferenceUnsupported)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetErrorCode {
    NotFoundOrForbidden,
    DependencyUnavailable,
    DependencyIncompatible,
    ValidationFailed,
    Conflict,
    IdempotencyMismatch,
    MalformedRequest,
}

impl DatasetErrorCode {
    pub const fn as_code(self) -> &'static str {
        match self {
            Self::NotFoundOrForbidden => "dataset.not_found_or_forbidden",
            Self::DependencyUnavailable => "dataset.dependency_unavailable",
            Self::DependencyIncompatible => "dataset.dependency_incompatible",
            Self::ValidationFailed => "dataset.validation_failed",
            Self::Conflict => "dataset.conflict",
            Self::IdempotencyMismatch => "dataset.idempotency_mismatch",
            Self::MalformedRequest => "dataset.malformed_request",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetErrorEnvelope {
    pub schema_version: u16,
    pub error: DatasetErrorDetail,
    pub correlation_id: Uuid,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetErrorDetail {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DatasetReverseContractValidationError {
    #[error("Dataset reverse-contract schema version is unsupported")]
    UnsupportedSchemaVersion,
    #[error("available reverse-provider result requires at least one item")]
    AvailableWithoutItems,
    #[error("unavailable, undisclosed, or empty state cannot carry product data")]
    NonAvailableCarriesData,
    #[error("available or empty summary must carry explicit counts")]
    AvailableWithoutCounts,
    #[error("summary counts do not match the declared available or empty state")]
    InvalidSummaryCounts,
    #[error("a restricted or unresolved resource response cannot carry an observation")]
    RestrictedOrUnresolvedObservation,
    #[error("an authorized resolved resource response requires an observation")]
    MissingResourceObservation,
    #[error("the resource observation owner state is not the live retained Module Instance")]
    ObservationOwnerMismatch,
    #[error("the resource observation does not echo the requested reference")]
    ObservationReferenceMismatch,
    #[error("the resource observation reference is not a canonical Dataset resource reference")]
    ObservationReferenceUnsupported,
    #[error("the resource observation contract identity does not match its resource type")]
    ObservationContractMismatch,
}

fn validate_reverse_schema(version: u16) -> Result<(), DatasetReverseContractValidationError> {
    if version != 1 {
        return Err(DatasetReverseContractValidationError::UnsupportedSchemaVersion);
    }
    Ok(())
}

fn deserialize_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != DATASET_CONTRACT_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "Dataset contract schema version {version} is unsupported; expected {DATASET_CONTRACT_SCHEMA_VERSION}"
        )));
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tessara_module_contract::{
        FunctionalContractId, ProviderContractIdentity, ResourceObservationStrategy, ResourceOwner,
        ResourceRevision, TypedResourceReference,
    };

    use super::*;

    const INSTALLATION_ID: Uuid = Uuid::from_u128(1);
    const DATASET_ID: Uuid = Uuid::from_u128(2);
    const MODULE_INSTANCE_ID: Uuid = Uuid::from_u128(3);

    #[test]
    fn canonical_dataset_major_line_round_trips() {
        let reference = DatasetMajorLineReference::from_parts(
            INSTALLATION_ID,
            MODULE_INSTANCE_ID,
            DATASET_ID,
            3,
        )
        .unwrap();
        assert_eq!(reference.dataset_id(), DATASET_ID);
        assert_eq!(reference.major(), 3);
        assert_eq!(
            reference.reference().resource_id(),
            format!("{DATASET_ID}@3")
        );
        let wire = serde_json::to_value(&reference).unwrap();
        assert_eq!(
            serde_json::from_value::<DatasetMajorLineReference>(wire).unwrap(),
            reference
        );

        let maximum = DatasetMajorLineReference::from_parts(
            INSTALLATION_ID,
            MODULE_INSTANCE_ID,
            DATASET_ID,
            i32::MAX,
        )
        .unwrap();
        assert_eq!(maximum.major(), i32::MAX);
    }

    #[test]
    fn wrong_owner_type_and_noncanonical_identity_fail() {
        let core_owned = TypedResourceReference::new(
            INSTALLATION_ID,
            ResourceOwner::CoreInstallation {
                installation_id: INSTALLATION_ID,
            },
            DATASET_MAJOR_LINE_RESOURCE_TYPE.parse().unwrap(),
            format!("{DATASET_ID}@1"),
        )
        .unwrap();
        assert!(DatasetMajorLineReference::new(core_owned).is_err());

        let wrong_type = TypedResourceReference::new(
            INSTALLATION_ID,
            ResourceOwner::CoreInstallation {
                installation_id: INSTALLATION_ID,
            },
            DATASET_REVISION_RESOURCE_TYPE.parse().unwrap(),
            format!("{DATASET_ID}@1"),
        )
        .unwrap();
        assert!(DatasetMajorLineReference::new(wrong_type).is_err());

        let invalid = json!({
            "reference": {
                "installation_id": INSTALLATION_ID,
                "owner": {"kind": "module_instance", "installation_id": INSTALLATION_ID, "module_instance_id": MODULE_INSTANCE_ID},
                "resource_type": DATASET_MAJOR_LINE_RESOURCE_TYPE,
                "resource_id": format!("{}@0", DATASET_ID.to_string().to_uppercase())
            }
        });
        assert!(serde_json::from_value::<DatasetMajorLineReference>(invalid).is_err());

        let out_of_storage_range = json!({
            "reference": {
                "installation_id": INSTALLATION_ID,
                "owner": {"kind": "module_instance", "installation_id": INSTALLATION_ID, "module_instance_id": MODULE_INSTANCE_ID},
                "resource_type": DATASET_MAJOR_LINE_RESOURCE_TYPE,
                "resource_id": format!("{DATASET_ID}@2147483648")
            }
        });
        assert!(serde_json::from_value::<DatasetMajorLineReference>(out_of_storage_range).is_err());
    }

    #[test]
    fn dataset_and_revision_references_are_exact_module_owned_types() {
        let dataset =
            DatasetReference::from_parts(INSTALLATION_ID, MODULE_INSTANCE_ID, DATASET_ID).unwrap();
        assert_eq!(dataset.dataset_id(), DATASET_ID);
        let revision_id = Uuid::from_u128(4);
        let revision =
            DatasetRevisionReference::from_parts(INSTALLATION_ID, MODULE_INSTANCE_ID, revision_id)
                .unwrap();
        assert_eq!(revision.revision_id(), revision_id);

        let legacy = json!({
            "reference": {
                "installation_id": INSTALLATION_ID,
                "owner": {"kind": "core_installation", "installation_id": INSTALLATION_ID},
                "resource_type": "tessara.transition.dataset_revision",
                "resource_id": revision_id
            }
        });
        assert!(serde_json::from_value::<DatasetRevisionReference>(legacy).is_err());
    }

    #[test]
    fn old_schema_and_unknown_fields_fail_closed() {
        let reference = DatasetMajorLineReference::from_parts(
            INSTALLATION_ID,
            MODULE_INSTANCE_ID,
            DATASET_ID,
            1,
        )
        .unwrap();
        let request = DatasetSchemaRequest {
            schema_version: 2,
            action: DatasetAction::ResolveSchema,
            reference,
        };
        let mut wire = serde_json::to_value(request).unwrap();
        wire["schema_version"] = json!(0);
        assert!(serde_json::from_value::<DatasetSchemaRequest>(wire.clone()).is_err());
        wire["schema_version"] = json!(2);
        wire.as_object_mut()
            .unwrap()
            .insert("fallback".into(), json!(true));
        assert!(serde_json::from_value::<DatasetSchemaRequest>(wire).is_err());
    }

    #[test]
    fn major_line_metadata_preserves_picker_discovery_context_exactly() {
        let form_id = Uuid::from_u128(3);
        let upstream_dataset_id = Uuid::from_u128(4);
        let metadata = DatasetMajorLineMetadata {
            reference: DatasetMajorLineReference::from_parts(
                INSTALLATION_ID,
                MODULE_INSTANCE_ID,
                DATASET_ID,
                3,
            )
            .unwrap(),
            dataset_name: "Enrollment outcomes".into(),
            dataset_slug: "enrollment-outcomes".into(),
            grain: "submission".into(),
            tags: vec!["outcomes".into(), "enrollment".into()],
            provenance: DatasetProvenanceSummary {
                forms: vec![DatasetProvenanceItem {
                    id: form_id,
                    name: "Enrollment intake".into(),
                    slug: None,
                }],
                datasets: vec![DatasetProvenanceItem {
                    id: upstream_dataset_id,
                    name: "Enrollment activity".into(),
                    slug: Some("enrollment-activity".into()),
                }],
            },
            materialization_state: "ready".into(),
            fields: vec![DatasetFieldContract {
                key: "program".into(),
                label: "Program".into(),
                field_type: "text".into(),
                restriction_tier: "provider_enforced".into(),
            }],
            scope_node_ids: vec![Uuid::from_u128(5)],
        };

        let mut wire = serde_json::to_value(&metadata).unwrap();
        assert_eq!(wire["dataset_name"], "Enrollment outcomes");
        assert_eq!(
            wire["reference"]["reference"]["resource_id"],
            format!("{DATASET_ID}@3")
        );
        assert_eq!(wire["grain"], "submission");
        assert_eq!(wire["tags"], json!(["outcomes", "enrollment"]));
        assert_eq!(wire["provenance"]["forms"][0]["name"], "Enrollment intake");
        assert_eq!(
            wire["provenance"]["datasets"][0]["slug"],
            "enrollment-activity"
        );
        assert_eq!(wire["fields"][0]["label"], "Program");
        assert_eq!(
            serde_json::from_value::<DatasetMajorLineMetadata>(wire.clone()).unwrap(),
            metadata
        );

        wire.as_object_mut()
            .unwrap()
            .insert("legacy_picker_label".into(), json!("unsupported"));
        assert!(serde_json::from_value::<DatasetMajorLineMetadata>(wire).is_err());
    }

    #[test]
    fn reverse_consumer_states_never_turn_outage_into_empty_data() {
        let unavailable = DatasetSummaryResponse {
            schema_version: 1,
            state: DatasetProviderResultState::Unavailable,
            dataset_count: None,
            published_revision_count: None,
        };
        assert!(unavailable.validate().is_ok());
        let false_zero = DatasetSummaryResponse {
            dataset_count: Some(0),
            published_revision_count: Some(0),
            ..unavailable
        };
        assert_eq!(
            false_zero.validate(),
            Err(DatasetReverseContractValidationError::NonAvailableCarriesData)
        );

        for inconsistent in [
            DatasetSummaryResponse {
                schema_version: 1,
                state: DatasetProviderResultState::Available,
                dataset_count: Some(0),
                published_revision_count: Some(0),
            },
            DatasetSummaryResponse {
                schema_version: 1,
                state: DatasetProviderResultState::Empty,
                dataset_count: Some(1),
                published_revision_count: Some(0),
            },
        ] {
            assert_eq!(
                inconsistent.validate(),
                Err(DatasetReverseContractValidationError::InvalidSummaryCounts)
            );
        }
    }

    #[test]
    fn resource_observation_response_requires_exact_reference_contract_and_disclosure_shape() {
        let reference =
            DatasetReference::from_parts(INSTALLATION_ID, MODULE_INSTANCE_ID, DATASET_ID)
                .expect("Dataset reference");
        let observation = ResourceObservationV1::new(
            reference.reference().clone(),
            ProviderContractIdentity::new(
                FunctionalContractId::new(DATASET_RESOURCE_CONTRACT_ID).expect("contract"),
                DATASET_REVERSE_CONTRACT_VERSION.parse().expect("version"),
            ),
            ResourceObservationStrategy::LiveResolutionWithRevision,
            ResourceRevision::new(7).expect("revision"),
        );
        let resolution = ResourceResolutionV1::authorized(
            ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::Live,
                data_state: OwnerDataState::Retained,
            },
            ResourceIdentityState::Resolved,
            ResourceLifecycleState::ProviderDefined {
                state: "active".into(),
            },
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        )
        .expect("resolution");
        let valid = DatasetResourceObservationResponse {
            schema_version: 1,
            resolution: resolution.clone(),
            observation: Some(observation.clone()),
        };
        assert!(valid.validate_for(reference.reference()).is_ok());

        let other =
            DatasetReference::from_parts(INSTALLATION_ID, MODULE_INSTANCE_ID, Uuid::from_u128(99))
                .expect("other reference");
        assert_eq!(
            valid.validate_for(other.reference()),
            Err(DatasetReverseContractValidationError::ObservationReferenceMismatch)
        );
        assert_eq!(
            DatasetResourceObservationResponse {
                schema_version: 1,
                resolution,
                observation: None,
            }
            .validate_for(reference.reference()),
            Err(DatasetReverseContractValidationError::MissingResourceObservation)
        );

        let restricted = ResourceResolutionV1::restricted(ResourceAccessState::Unauthorized)
            .expect("restricted resolution");
        assert!(
            DatasetResourceObservationResponse {
                schema_version: 1,
                resolution: restricted.clone(),
                observation: None,
            }
            .validate_for(reference.reference())
            .is_ok()
        );
        assert_eq!(
            DatasetResourceObservationResponse {
                schema_version: 1,
                resolution: restricted,
                observation: Some(observation),
            }
            .validate_for(reference.reference()),
            Err(DatasetReverseContractValidationError::RestrictedOrUnresolvedObservation)
        );
    }

    #[test]
    fn every_dataset_resource_observation_has_its_exact_provider_contract() {
        for (reference, contract_id, contract_version) in [
            (
                DatasetReference::from_parts(INSTALLATION_ID, MODULE_INSTANCE_ID, DATASET_ID)
                    .expect("Dataset reference")
                    .reference()
                    .clone(),
                DATASET_RESOURCE_CONTRACT_ID,
                DATASET_REVERSE_CONTRACT_VERSION,
            ),
            (
                DatasetRevisionReference::from_parts(
                    INSTALLATION_ID,
                    MODULE_INSTANCE_ID,
                    Uuid::from_u128(21),
                )
                .expect("revision reference")
                .reference()
                .clone(),
                DATASET_REVISION_CONTRACT_ID,
                DATASET_REVERSE_CONTRACT_VERSION,
            ),
            (
                DatasetMajorLineReference::from_parts(
                    INSTALLATION_ID,
                    MODULE_INSTANCE_ID,
                    DATASET_ID,
                    2,
                )
                .expect("major-line reference")
                .reference()
                .clone(),
                DATASET_CONTRACT_ID,
                DATASET_CONTRACT_VERSION,
            ),
        ] {
            assert_eq!(
                expected_observation_contract(&reference),
                Ok((contract_id, contract_version))
            );
        }
    }

    #[test]
    fn dataset_error_codes_are_module_owned_and_distinct() {
        assert_eq!(
            DatasetErrorCode::NotFoundOrForbidden.as_code(),
            "dataset.not_found_or_forbidden"
        );
        assert_ne!(
            DatasetErrorCode::DependencyUnavailable.as_code(),
            DatasetErrorCode::DependencyIncompatible.as_code()
        );
        assert_ne!(
            DatasetErrorCode::Conflict.as_code(),
            DatasetErrorCode::IdempotencyMismatch.as_code()
        );
    }

    #[test]
    fn dataset_mutation_idempotency_header_is_exact() {
        assert_eq!(DATASET_IDEMPOTENCY_HEADER, "x-idempotency-key");
        assert_eq!(DATASET_AUTHORING_CONTRACT_ID, "tessara.datasets.authoring");
    }

    #[test]
    fn product_mutation_contracts_reject_legacy_or_unknown_fields() {
        let valid = json!({
            "version_label": "Reviewed",
            "revision_notes": "Owner approved"
        });
        let request: DatasetRevisionLabelRequestV1 = serde_json::from_value(valid.clone()).unwrap();
        assert_eq!(request.version_label.as_deref(), Some("Reviewed"));

        let mut unknown = valid;
        unknown
            .as_object_mut()
            .unwrap()
            .insert("legacy_label".into(), json!("unsupported"));
        assert!(serde_json::from_value::<DatasetRevisionLabelRequestV1>(unknown).is_err());
        assert!(
            serde_json::from_value::<DatasetUpdateTagsRequestV1>(json!({
                "tags": [],
                "dataset_id": DATASET_ID
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<DatasetRevisionOptionsRequestV1>(json!({
                "force_new_major_version": true,
                "legacy_semver_override": "2.0.0"
            }))
            .is_err()
        );

        let authoring = DatasetAuthoringRequestV1 {
            name: "Enrollment outcomes".into(),
            slug: "enrollment-outcomes".into(),
            grain: "submission".into(),
            version_label: None,
            force_new_major_version: false,
            visibility_node_ids: vec![Uuid::from_u128(7).to_string()],
            initial_source: DatasetProductSourceV1::Form {
                alias: "enrollment".into(),
                form_id: Uuid::from_u128(8).to_string(),
                form_version_id: Uuid::from_u128(9).to_string(),
            },
            operations: Vec::new(),
            restriction_policy: None,
        };
        let mut authoring_wire = serde_json::to_value(&authoring).unwrap();
        assert!(authoring_wire.get("version_label").is_none());
        assert!(authoring_wire.get("restriction_policy").is_none());
        assert_eq!(
            serde_json::from_value::<DatasetAuthoringRequestV1>(authoring_wire.clone()).unwrap(),
            authoring
        );
        authoring_wire
            .as_object_mut()
            .unwrap()
            .insert("legacy_source".into(), json!({}));
        assert!(serde_json::from_value::<DatasetAuthoringRequestV1>(authoring_wire).is_err());

        let refresh = DatasetRefreshResponseV1 {
            dataset_id: DATASET_ID.to_string(),
            changed: false,
            freshness: DatasetProductFreshnessV1 {
                state: DatasetFreshnessState::Current,
                last_checked_at: Some("2026-08-13T12:00:00Z".into()),
                last_succeeded_at: Some("2026-08-13T11:00:00Z".into()),
                sanitized_failure_code: None,
            },
            materialization_receipt_ids: Vec::new(),
        };
        let mut refresh_wire = serde_json::to_value(&refresh).unwrap();
        assert_eq!(
            serde_json::from_value::<DatasetRefreshResponseV1>(refresh_wire.clone()).unwrap(),
            refresh
        );
        refresh_wire
            .as_object_mut()
            .unwrap()
            .insert("authenticated_head".into(), json!("must-not-leak"));
        assert!(serde_json::from_value::<DatasetRefreshResponseV1>(refresh_wire).is_err());
    }
}
