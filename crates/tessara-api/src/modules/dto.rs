//! Versioned HTTP wire types for Core module discovery and platform adapters.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tessara_module_contract::{
    DeploymentReceiptV1, ModuleManifest, ResourceOwner, ResourceTypeId, SemanticDestination,
    TypedResourceReference,
};
pub(crate) use tessara_module_contract::{
    IndependentConfigurationV1, IndependentDefinitionV1, IndependentDiagnosticsV1,
    IndependentInstanceV1, IndependentReleaseV1,
};
use uuid::Uuid;

pub(crate) const MODULE_HTTP_SCHEMA_VERSION_V1: u16 = 1;
pub(crate) const NAVIGATION_POLICY_SCHEMA_VERSION_V2: u16 = 2;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ModuleInventoryResponseV1 {
    pub(crate) schema_version: u16,
    pub(crate) installation: ApplicationInstallationV1,
    pub(crate) core_runtime: CoreRuntimeObservationV1,
    pub(crate) entries: Vec<Value>,
    pub(crate) deployment: Option<DeploymentReceiptV1>,
    pub(crate) deployment_history: Vec<DeploymentReceiptV1>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ApplicationInstallationV1 {
    pub(crate) id: Uuid,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CoreRuntimeObservationV1 {
    pub(crate) provenance: String,
    pub(crate) observed_version: String,
    pub(crate) finding_code: String,
    pub(crate) observed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum IndependentModuleEntryV1 {
    IndependentlyDeployed {
        definition: IndependentDefinitionV1,
        release: IndependentReleaseV1,
        instance: IndependentInstanceV1,
        configuration: IndependentConfigurationV1,
        diagnostics: IndependentDiagnosticsV1,
        manifest: Option<ModuleManifest>,
        findings: Vec<Value>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ModuleDetailResponseV1 {
    pub(crate) schema_version: u16,
    pub(crate) installation_id: Uuid,
    pub(crate) entry: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg(test)]
pub(crate) struct NavigationPolicyResponseV1 {
    pub(crate) schema_version: u16,
    pub(crate) installation_id: Uuid,
    pub(crate) revision: i64,
    /// Authoritative global permission for enabling policy mutation controls.
    pub(crate) can_manage_navigation: bool,
    pub(crate) immutable_core_items: Vec<ImmutableCoreNavigationItemV1>,
    pub(crate) contributions: Vec<NavigationPolicyContributionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg(test)]
pub(crate) struct ImmutableCoreNavigationItemV1 {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) group: String,
    pub(crate) route: String,
    pub(crate) policy_mutable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg(test)]
pub(crate) struct NavigationPolicyContributionV1 {
    pub(crate) id: String,
    pub(crate) definition_id: String,
    pub(crate) label: String,
    pub(crate) destination: String,
    pub(crate) group: String,
    pub(crate) reorder_band: String,
    pub(crate) before_core_anchor: String,
    pub(crate) after_core_anchor: String,
    pub(crate) visible: bool,
    pub(crate) order: i32,
    pub(crate) required_capabilities_any_of: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg(test)]
pub(crate) struct UpdateNavigationPolicyRequestV1 {
    pub(crate) schema_version: u16,
    pub(crate) expected_revision: i64,
    pub(crate) contributions: Vec<NavigationPolicyMutationV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg(test)]
pub(crate) struct NavigationPolicyMutationV1 {
    pub(crate) id: String,
    pub(crate) group: String,
    pub(crate) reorder_band: String,
    pub(crate) visible: bool,
    pub(crate) order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NavigationPolicyResponseV2 {
    pub(crate) schema_version: u16,
    pub(crate) installation_id: Uuid,
    pub(crate) revision: i64,
    pub(crate) can_manage_navigation: bool,
    pub(crate) groups: Vec<NavigationGroupV2>,
    pub(crate) destinations: Vec<NavigationDestinationV2>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NavigationGroupOwnerV2 {
    Core,
    Custom,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NavigationGroupV2 {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) order: i32,
    pub(crate) owner: NavigationGroupOwnerV2,
    pub(crate) can_rename: bool,
    pub(crate) can_move: bool,
    pub(crate) can_delete: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NavigationDestinationOwnerV2 {
    Core,
    Contribution,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NavigationDestinationV2 {
    pub(crate) id: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) route: String,
    pub(crate) semantic_destination: Option<String>,
    pub(crate) definition_id: Option<String>,
    pub(crate) owner: NavigationDestinationOwnerV2,
    pub(crate) required_capabilities_any_of: Vec<String>,
    pub(crate) group_id: String,
    pub(crate) visible: bool,
    pub(crate) order: i32,
    pub(crate) available: bool,
    pub(crate) can_hide: bool,
    pub(crate) can_move_between_groups: bool,
    pub(crate) can_reorder: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdateNavigationPolicyRequestV2 {
    pub(crate) schema_version: u16,
    pub(crate) expected_revision: i64,
    pub(crate) groups: Vec<NavigationGroupMutationV2>,
    pub(crate) destinations: Vec<NavigationDestinationMutationV2>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NavigationGroupMutationV2 {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NavigationDestinationMutationV2 {
    pub(crate) id: String,
    pub(crate) group_id: String,
    pub(crate) visible: bool,
    pub(crate) order: i32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResolveDestinationRequestV1 {
    pub(crate) schema_version: u16,
    pub(crate) destination: SemanticDestination,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DestinationResolutionStatusV1 {
    Resolved,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DestinationResolutionResponseV1 {
    pub(crate) schema_version: u16,
    pub(crate) status: DestinationResolutionStatusV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) finding: Option<PlatformFindingV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PlatformFindingV1 {
    pub(crate) code: &'static str,
    pub(crate) path: &'static str,
    pub(crate) message: &'static str,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateResourceReferenceRequestV1 {
    pub(crate) schema_version: u16,
    pub(crate) installation_id: Uuid,
    pub(crate) owner: ResourceOwner,
    pub(crate) resource_type: ResourceTypeId,
    pub(crate) resource_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ResourceReferenceResponseV1 {
    pub(crate) schema_version: u16,
    pub(crate) reference: TypedResourceReference,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResolveResourceReferenceRequestV1 {
    pub(crate) schema_version: u16,
    pub(crate) reference: TypedResourceReference,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ResourceObservationResponseV1 {
    pub(crate) schema_version: u16,
    pub(crate) resolution: tessara_module_contract::ResourceResolutionV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) observation: Option<tessara_module_contract::ResourceObservationV1>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{NavigationPolicyMutationV1, UpdateNavigationPolicyRequestV1};

    #[test]
    fn policy_mutation_rejects_unknown_fields() {
        let wire = json!({
            "schema_version": 1,
            "expected_revision": 4,
            "contributions": [{
                "id": "tessara.forms.navigation",
                "group": "Main",
                "reorder_band": "main_between_organization_and_operations",
                "visible": true,
                "order": 0,
                "route": "/caller-controlled"
            }]
        });

        assert!(serde_json::from_value::<UpdateNavigationPolicyRequestV1>(wire).is_err());
    }

    #[test]
    fn policy_mutation_has_no_route_or_capability_fields() {
        let wire = json!({
            "id": "tessara.forms.navigation",
            "group": "Main",
            "reorder_band": "main_between_organization_and_operations",
            "visible": false,
            "order": 0
        });

        let mutation = serde_json::from_value::<NavigationPolicyMutationV1>(wire)
            .expect("canonical mutation parses");
        assert_eq!(mutation.id, "tessara.forms.navigation");
    }
}
