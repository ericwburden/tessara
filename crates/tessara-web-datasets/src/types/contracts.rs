//! API and feature-local data contracts for the Datasets feature.

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SessionAccount {
    pub(crate) capabilities: Vec<String>,
}

pub(crate) type DatasetSummary = tessara_datasets_contract::DatasetProductSummaryV1;

pub(crate) type DatasetDefinition = tessara_datasets_contract::DatasetProductDefinitionV1;

pub(crate) type DatasetProvenanceSummary =
    tessara_datasets_contract::DatasetProductProvenanceSummaryV1;

pub(crate) type DatasetLineageNode = tessara_datasets_contract::DatasetProductLineageNodeV1;

#[cfg(feature = "hydrate")]
pub(crate) type UpdateDatasetTagsRequest = tessara_datasets_contract::DatasetUpdateTagsRequestV1;

pub(crate) type DatasetRevisionStatus = tessara_datasets_contract::DatasetProductRevisionStatusV1;
pub(crate) type DatasetVersionImpact = tessara_datasets_contract::DatasetProductVersionImpactV1;
pub(crate) type DatasetCompatibilityState =
    tessara_datasets_contract::DatasetProductCompatibilityStateV1;
pub(crate) type DatasetDependencyKind = tessara_datasets_contract::DatasetProductDependencyKindV1;
pub(crate) type DatasetDependencyBindingMode =
    tessara_datasets_contract::DatasetProductDependencyBindingModeV1;
pub(crate) type DatasetCarryForwardState =
    tessara_datasets_contract::DatasetProductCarryForwardStateV1;
pub(crate) type DatasetCompatibilityFinding =
    tessara_datasets_contract::DatasetProductCompatibilityFindingV1;
pub(crate) type DatasetCompatibilitySummary =
    tessara_datasets_contract::DatasetProductCompatibilitySummaryV1;
pub(crate) type DatasetDependencyImpact =
    tessara_datasets_contract::DatasetProductDependencyImpactV1;
pub(crate) type DatasetDependencySummary =
    tessara_datasets_contract::DatasetProductDependencySummaryV1;
pub(crate) type DatasetRevisionSummary = tessara_datasets_contract::DatasetProductRevisionSummaryV1;
pub(crate) type DatasetRevisionDetail = tessara_datasets_contract::DatasetProductRevisionDetailV1;

#[allow(dead_code)]
pub(crate) type DatasetDraftRevisionResponse =
    tessara_datasets_contract::DatasetDraftRevisionResponseV1;

#[allow(dead_code)]
pub(crate) type DatasetPublishRevisionResponse =
    tessara_datasets_contract::DatasetPublishRevisionResponseV1;

#[cfg(feature = "hydrate")]
pub(crate) type DatasetRevisionLabelRequest =
    tessara_datasets_contract::DatasetRevisionLabelRequestV1;

#[cfg(feature = "hydrate")]
pub(crate) type DatasetRevisionOptionsRequest =
    tessara_datasets_contract::DatasetRevisionOptionsRequestV1;

#[cfg(feature = "hydrate")]
pub(crate) type DatasetRevisionLabelResponse =
    tessara_datasets_contract::DatasetRevisionLabelResponseV1;

pub(crate) type DatasetVisibilityNode = tessara_datasets_contract::DatasetProductVisibilityNodeV1;
#[cfg(test)]
pub(crate) type DatasetRevisionFieldSummary =
    tessara_datasets_contract::DatasetProductRevisionFieldSummaryV1;

pub(crate) type DatasetSourceDefinition =
    tessara_datasets_contract::DatasetProductSourceDefinitionV1;

pub(crate) type DatasetFieldDefinition = tessara_datasets_contract::DatasetProductFieldV1;

pub(crate) type DatasetTable = tessara_datasets_contract::DatasetProductTableV1;

#[cfg(feature = "hydrate")]
pub(crate) type DatasetRefreshResponse = tessara_datasets_contract::DatasetRefreshResponseV1;

#[cfg(feature = "hydrate")]
pub(crate) type DatasetSqlPreviewResponse = tessara_datasets_contract::DatasetSqlPreviewResponseV1;

pub(crate) type DatasetFormOption = tessara_datasets_contract::DatasetEditorFormOptionV1;
pub(crate) type DatasetFormVersionOption =
    tessara_datasets_contract::DatasetEditorFormVersionOptionV1;
pub(crate) type DatasetRenderedForm = tessara_datasets_contract::DatasetEditorRenderedFormV1;
#[cfg(test)]
pub(crate) type DatasetRenderedSection = tessara_datasets_contract::DatasetEditorRenderedSectionV1;
pub(crate) type DatasetRenderedField = tessara_datasets_contract::DatasetEditorRenderedFieldV1;

pub(crate) type NodeResponse = tessara_datasets_contract::DatasetEditorScopeOptionV1;
pub(crate) type DatasetUserOption = tessara_datasets_contract::DatasetEditorPrincipalOptionV1;

#[allow(dead_code)]
pub(crate) type DatasetPayload = tessara_datasets_contract::DatasetAuthoringRequestV1;

pub(crate) type DatasetSourcePayload = tessara_datasets_contract::DatasetProductSourceV1;
pub(crate) type DatasetOperationPayload = tessara_datasets_contract::DatasetProductOperationV1;
pub(crate) type DatasetProjectionFieldPayload =
    tessara_datasets_contract::DatasetProductProjectionFieldV1;
pub(crate) type DatasetAggregationMetricPayload =
    tessara_datasets_contract::DatasetProductAggregationMetricV1;
pub(crate) type DatasetRowPickerPayload = tessara_datasets_contract::DatasetProductRowPickerV1;
#[cfg(feature = "hydrate")]
pub(crate) type DatasetRowPickerSortPayload =
    tessara_datasets_contract::DatasetProductRowPickerSortV1;
pub(crate) type DatasetRowFilterPayload = tessara_datasets_contract::DatasetProductRowFilterV1;
pub(crate) type DatasetCalculatedFieldPayload =
    tessara_datasets_contract::DatasetProductCalculatedFieldV1;
#[cfg(feature = "hydrate")]
pub(crate) type DatasetCalculationFunctionPayload =
    tessara_datasets_contract::DatasetProductCalculationFunctionV1;
pub(crate) type DatasetRestrictionPolicyPayload =
    tessara_datasets_contract::DatasetProductRestrictionPolicyV1;
#[cfg(feature = "hydrate")]
pub(crate) type DatasetJoinKeyPayload = tessara_datasets_contract::DatasetProductJoinKeyV1;
