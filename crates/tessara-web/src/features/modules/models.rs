//! Web-owned Module Management projections.
//!
//! These types mirror the versioned Core API without importing persistence or
//! contract-crate implementation types into the UI. Keeping the bootstrap and
//! browser response shapes identical prevents SSR and hydration from rendering
//! different interpretations of module inventory.

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use serde_json::Value;

pub const MODULE_HTTP_SCHEMA_VERSION_V1: u16 = 1;

fn deserialize_schema_version_v1<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version == MODULE_HTTP_SCHEMA_VERSION_V1 {
        Ok(version)
    } else {
        Err(D::Error::custom(format!(
            "unsupported Module Management schema version {version}"
        )))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleInventoryResponseV1 {
    #[serde(deserialize_with = "deserialize_schema_version_v1")]
    pub schema_version: u16,
    pub installation: ApplicationInstallationV1,
    pub core_runtime: CoreRuntimeObservationV1,
    pub entries: Vec<ModuleInventoryEntryV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationInstallationV1 {
    pub id: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreRuntimeObservationV1 {
    pub provenance: String,
    pub observed_version: String,
    pub finding_code: String,
    pub observed_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleDetailResponseV1 {
    #[serde(deserialize_with = "deserialize_schema_version_v1")]
    pub schema_version: u16,
    pub installation_id: String,
    pub entry: ModuleInventoryEntryV1,
}

/// Sprint 6A inventory contains only truthful transition projections.
///
/// A real Module Instance remains a public platform-contract type, but Core
/// does not persist or serve one in this sprint. Intentionally rejecting a
/// different `kind` keeps this UI from silently presenting future state using
/// transition labels.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModuleInventoryEntryV1 {
    TransitionalInProcess {
        descriptor: TransitionalContributionDescriptorV1,
        source_digest: String,
        resource_owner: ResourceOwnerV1,
        provider_eligible: bool,
        supervisor_materializable: bool,
        findings: Vec<ModuleFindingV1>,
    },
}

impl ModuleInventoryEntryV1 {
    pub fn descriptor(&self) -> &TransitionalContributionDescriptorV1 {
        match self {
            Self::TransitionalInProcess { descriptor, .. } => descriptor,
        }
    }

    pub fn definition_id(&self) -> &str {
        self.descriptor().reserved_definition_id.as_str()
    }

    pub fn source_digest(&self) -> &str {
        match self {
            Self::TransitionalInProcess { source_digest, .. } => source_digest,
        }
    }

    pub fn findings(&self) -> &[ModuleFindingV1] {
        match self {
            Self::TransitionalInProcess { findings, .. } => findings,
        }
    }

    pub const fn is_transition(&self) -> bool {
        matches!(self, Self::TransitionalInProcess { .. })
    }

    /// Projects lifecycle dimensions independently so transition metadata can
    /// never be mistaken for a Module Release or Module Instance observation.
    pub fn detail_dimensions(&self) -> ModuleDetailDimensionsV1 {
        let descriptor = self.descriptor();
        let dependency = if descriptor.dependencies.is_empty() {
            ModuleDetailDimensionV1 {
                state: ModuleDetailDimensionStateV1::NoDeclaration,
                evidence: "No functional dependencies are declared.".into(),
            }
        } else {
            let count = descriptor.dependencies.len();
            ModuleDetailDimensionV1 {
                state: ModuleDetailDimensionStateV1::TransitionInternalOnly,
                evidence: format!(
                    "{count} declared {} describe current in-process coupling and cannot be satisfied by a transition contribution provider.",
                    if count == 1 {
                        "relationship"
                    } else {
                        "relationships"
                    }
                ),
            }
        };
        let not_applicable = ModuleDetailDimensionStateV1::NotApplicableNoReleaseInstance;

        ModuleDetailDimensionsV1 {
            dependency,
            compatibility: ModuleDetailDimensionV1 {
                state: not_applicable,
                evidence:
                    "No Module Release exists, so Core has no release compatibility decision."
                        .into(),
            },
            configuration: ModuleDetailDimensionV1 {
                state: not_applicable,
                evidence: if descriptor.configuration_schema.is_some() {
                    "A transition configuration schema is declared as discovery metadata; no Module Instance configuration exists."
                        .into()
                } else {
                    "No transition configuration schema is declared, and no Module Instance configuration exists."
                        .into()
                },
            },
            readiness: ModuleDetailDimensionV1 {
                state: not_applicable,
                evidence: "No Module Instance exists, so Core has no readiness observation.".into(),
            },
            health: ModuleDetailDimensionV1 {
                state: not_applicable,
                evidence: "No Module Instance exists, so Core does not evaluate or infer health."
                    .into(),
            },
        }
    }
}

/// Exact lifecycle wording for transition dimensions that require a real
/// Module Release or Module Instance.
pub const NOT_APPLICABLE_NO_MODULE_RELEASE_INSTANCE_LABEL: &str =
    "Not applicable — no Module Release/Instance";

/// Independent Module detail dimensions. These are presentation projections,
/// not persisted lifecycle facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDetailDimensionsV1 {
    pub dependency: ModuleDetailDimensionV1,
    pub compatibility: ModuleDetailDimensionV1,
    pub configuration: ModuleDetailDimensionV1,
    pub readiness: ModuleDetailDimensionV1,
    pub health: ModuleDetailDimensionV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDetailDimensionV1 {
    pub state: ModuleDetailDimensionStateV1,
    pub evidence: String,
}

/// Closed presentation states prevent independent dimensions from being
/// flattened into a generic finding or inferred runtime status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleDetailDimensionStateV1 {
    TransitionInternalOnly,
    NoDeclaration,
    NotApplicableNoReleaseInstance,
}

impl ModuleDetailDimensionStateV1 {
    pub const fn label(self) -> &'static str {
        match self {
            Self::TransitionInternalOnly => "Transition-internal only",
            Self::NoDeclaration => "No functional dependencies declared",
            Self::NotApplicableNoReleaseInstance => NOT_APPLICABLE_NO_MODULE_RELEASE_INSTANCE_LABEL,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResourceOwnerV1 {
    CoreInstallation { installation_id: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionalContributionDescriptorV1 {
    #[serde(deserialize_with = "deserialize_schema_version_v1")]
    pub schema_version: u16,
    pub reserved_definition_id: String,
    pub display_name: String,
    pub description: String,
    pub availability: TransitionAvailabilityV1,
    #[serde(default)]
    pub features: Vec<FeatureDeclarationV1>,
    #[serde(default)]
    pub provided_contracts: Vec<FunctionalContractDeclarationV1>,
    #[serde(default)]
    pub dependencies: Vec<FunctionalDependencyV1>,
    #[serde(default)]
    pub resource_types: Vec<ResourceTypeDeclarationV1>,
    #[serde(default)]
    pub routes: Vec<RouteDeclarationV1>,
    #[serde(default)]
    pub navigation: Vec<NavigationContributionDeclarationV1>,
    #[serde(default)]
    pub security_capabilities: Vec<SecurityCapabilityDeclarationV1>,
    pub configuration_schema: Option<Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionAvailabilityV1 {
    ActiveInProcess,
    Unavailable,
    Retired,
}

impl TransitionAvailabilityV1 {
    pub const fn label(self) -> &'static str {
        match self {
            Self::ActiveInProcess => "Active in Core process",
            Self::Unavailable => "Unavailable",
            Self::Retired => "Retired",
        }
    }

    pub const fn explanation(self) -> &'static str {
        match self {
            Self::ActiveInProcess => {
                "The declared current surface executes in the shared Core process."
            }
            Self::Unavailable => {
                "The intended current transition surface cannot respond at this time."
            }
            Self::Retired => {
                "The former surface was deliberately withdrawn and has no live destination."
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureDeclarationV1 {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub use_cases: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outcomes: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub contracts: Vec<String>,
    #[serde(default)]
    pub resource_types: Vec<String>,
    #[serde(default)]
    pub destinations: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub configuration_pointers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionalContractDeclarationV1 {
    pub id: String,
    pub version: String,
    pub kind: FunctionalContractKindV1,
    pub description: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionalContractKindV1 {
    Api,
    Event,
    Resource,
    Behavior,
}

impl FunctionalContractKindV1 {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Api => "API",
            Self::Event => "Event",
            Self::Resource => "Resource",
            Self::Behavior => "Behavior",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionalDependencyV1 {
    pub contract_id: String,
    pub version_requirement: String,
    pub binding_key: String,
    pub optional: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceTypeDeclarationV1 {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteDeclarationV1 {
    pub name: String,
    pub kind: RouteKindV1,
    #[serde(default)]
    pub parameters: Vec<RouteParameterDeclarationV1>,
    /// Optional Core-resolved same-origin path. The current normalized catalog
    /// may omit this and the UI then presents the semantic name as metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteKindV1 {
    Product,
    Administration,
    Configuration,
    Diagnostics,
}

impl RouteKindV1 {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Product => "Product",
            Self::Administration => "Administration",
            Self::Configuration => "Configuration",
            Self::Diagnostics => "Diagnostics",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteParameterDeclarationV1 {
    pub name: String,
    pub value_type: RouteParameterTypeV1,
    pub required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteParameterTypeV1 {
    String,
    Integer,
    Boolean,
    Uuid,
}

impl RouteParameterTypeV1 {
    pub const fn label(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Boolean => "boolean",
            Self::Uuid => "UUID",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationContributionDeclarationV1 {
    pub id: String,
    pub destination: String,
    pub label: String,
    pub group: String,
    pub order_hint: i32,
    pub required_capabilities_any_of: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityCapabilityDeclarationV1 {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleFindingV1 {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationPolicyResponseV1 {
    #[serde(deserialize_with = "deserialize_schema_version_v1")]
    pub schema_version: u16,
    pub installation_id: String,
    pub revision: i64,
    /// Authoritative installation-global authority for mutation controls.
    pub can_manage_navigation: bool,
    pub immutable_core_items: Vec<ImmutableCoreNavigationItemV1>,
    pub contributions: Vec<NavigationPolicyContributionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableCoreNavigationItemV1 {
    pub id: String,
    pub label: String,
    pub group: String,
    pub route: String,
    pub policy_mutable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationPolicyContributionV1 {
    pub id: String,
    pub definition_id: String,
    pub label: String,
    pub destination: String,
    pub group: String,
    pub reorder_band: String,
    pub before_core_anchor: String,
    pub after_core_anchor: String,
    pub visible: bool,
    pub order: i32,
    pub required_capabilities_any_of: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateNavigationPolicyRequestV1 {
    #[serde(deserialize_with = "deserialize_schema_version_v1")]
    pub schema_version: u16,
    pub expected_revision: i64,
    pub contributions: Vec<NavigationPolicyMutationV1>,
}

impl From<&NavigationPolicyResponseV1> for UpdateNavigationPolicyRequestV1 {
    fn from(policy: &NavigationPolicyResponseV1) -> Self {
        Self {
            schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
            expected_revision: policy.revision,
            contributions: policy
                .contributions
                .iter()
                .map(|contribution| NavigationPolicyMutationV1 {
                    id: contribution.id.clone(),
                    group: contribution.group.clone(),
                    reorder_band: contribution.reorder_band.clone(),
                    visible: contribution.visible,
                    order: contribution.order,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationPolicyMutationV1 {
    pub id: String,
    pub group: String,
    pub reorder_band: String,
    pub visible: bool,
    pub order: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleManagementAccessV1 {
    can_read: bool,
    can_manage_navigation: bool,
}

impl ModuleManagementAccessV1 {
    pub const fn restricted() -> Self {
        Self {
            can_read: false,
            can_manage_navigation: false,
        }
    }

    pub const fn read_only() -> Self {
        Self {
            can_read: true,
            can_manage_navigation: false,
        }
    }

    pub const fn manager() -> Self {
        Self {
            can_read: true,
            can_manage_navigation: true,
        }
    }

    /// Builds access from keys that the caller has already proven are
    /// installation-global. Pass the session account's `global_capabilities`
    /// companion set, never its flattened `capabilities` list.
    pub fn from_global_capabilities(capabilities: &[String]) -> Self {
        let admin = capabilities.iter().any(|value| value == "admin:all");
        let manage = admin
            || capabilities
                .iter()
                .any(|value| value == "modules:manage_navigation");
        let read = manage || capabilities.iter().any(|value| value == "modules:read");
        Self {
            can_read: read,
            can_manage_navigation: manage,
        }
    }

    /// Controls fail closed if a malformed bootstrap claims manage without
    /// the implied read authority.
    pub const fn may_read(self) -> bool {
        self.can_read
    }

    pub const fn may_manage_navigation(self) -> bool {
        self.can_read && self.can_manage_navigation
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{
        ModuleDetailDimensionStateV1, ModuleInventoryEntryV1, ModuleManagementAccessV1,
        NOT_APPLICABLE_NO_MODULE_RELEASE_INSTANCE_LABEL, TransitionalContributionDescriptorV1,
        UpdateNavigationPolicyRequestV1,
    };

    #[test]
    fn canonical_transition_fixture_maps_to_owned_web_projection() {
        let descriptor: TransitionalContributionDescriptorV1 = serde_json::from_str(include_str!(
            "../../../../tessara-module-contract/tests/fixtures/transition-forms-v1.json"
        ))
        .expect("canonical Forms descriptor parses");
        let installation_id = "00000000-0000-0000-0000-000000000001";
        let entry: ModuleInventoryEntryV1 = serde_json::from_value(json!({
            "kind": "transitional_in_process",
            "descriptor": descriptor,
            "source_digest": "sha256:71bebdd07ff0028cc0da8bbd9707c393bade9951e5cedb265a4b8465d54b493e",
            "resource_owner": {
                "kind": "core_installation",
                "installation_id": installation_id
            },
            "provider_eligible": false,
            "supervisor_materializable": false,
            "findings": []
        }))
        .expect("normalized transition projection parses");

        assert_eq!(entry.definition_id(), "tessara.forms");
        assert!(entry.is_transition());
        assert_eq!(entry.descriptor().features.len(), 3);

        let dimensions = entry.detail_dimensions();
        assert_eq!(
            dimensions.dependency.state,
            ModuleDetailDimensionStateV1::NoDeclaration
        );
        for dimension in [
            &dimensions.compatibility,
            &dimensions.configuration,
            &dimensions.readiness,
            &dimensions.health,
        ] {
            assert_eq!(
                dimension.state.label(),
                NOT_APPLICABLE_NO_MODULE_RELEASE_INSTANCE_LABEL
            );
        }
        assert_eq!(
            dimensions.health.evidence,
            "No Module Instance exists, so Core does not evaluate or infer health."
        );
    }

    #[test]
    fn transition_dependency_and_configuration_metadata_do_not_become_runtime_state() {
        let mut descriptor: TransitionalContributionDescriptorV1 =
            serde_json::from_str(include_str!(
                "../../../../tessara-module-contract/tests/fixtures/transition-responses-v1.json"
            ))
            .expect("canonical Responses descriptor parses");
        descriptor.configuration_schema = Some(json!({"type": "object"}));
        let entry: ModuleInventoryEntryV1 = serde_json::from_value(json!({
            "kind": "transitional_in_process",
            "descriptor": descriptor,
            "source_digest": "sha256:synthetic",
            "resource_owner": {
                "kind": "core_installation",
                "installation_id": "installation-1"
            },
            "provider_eligible": false,
            "supervisor_materializable": false,
            "findings": []
        }))
        .expect("synthetic transition projection parses");

        let dimensions = entry.detail_dimensions();
        assert_eq!(
            dimensions.dependency.state,
            ModuleDetailDimensionStateV1::TransitionInternalOnly
        );
        assert_eq!(
            dimensions.dependency.evidence,
            "2 declared relationships describe current in-process coupling and cannot be satisfied by a transition contribution provider."
        );
        assert_eq!(
            dimensions.configuration.state,
            ModuleDetailDimensionStateV1::NotApplicableNoReleaseInstance
        );
        assert_eq!(
            dimensions.configuration.evidence,
            "A transition configuration schema is declared as discovery metadata; no Module Instance configuration exists."
        );
    }

    #[test]
    fn proven_global_access_implication_is_explicit_and_unrecognized_keys_do_not_match() {
        for capabilities in [
            vec!["modules:manage_navigation".to_string()],
            vec!["admin:all".to_string()],
        ] {
            assert_eq!(
                ModuleManagementAccessV1::from_global_capabilities(&capabilities),
                ModuleManagementAccessV1::manager()
            );
        }
        assert_eq!(
            ModuleManagementAccessV1::from_global_capabilities(&["modules:read".to_string()]),
            ModuleManagementAccessV1::read_only()
        );
        assert_eq!(
            ModuleManagementAccessV1::from_global_capabilities(&["modules:read@node".to_string()]),
            ModuleManagementAccessV1::restricted()
        );
    }

    #[test]
    fn policy_write_projection_has_no_route_or_capability_mutation_fields() {
        let policy = serde_json::from_value(json!({
            "schema_version": 1,
            "installation_id": "00000000-0000-0000-0000-000000000001",
            "revision": 7,
            "can_manage_navigation": false,
            "immutable_core_items": [],
            "contributions": [{
                "id": "tessara.forms.navigation",
                "definition_id": "tessara.forms",
                "label": "Forms",
                "destination": "forms.directory",
                "group": "Main",
                "reorder_band": "main_between_organization_and_operations",
                "before_core_anchor": "operations",
                "after_core_anchor": "organization",
                "visible": true,
                "order": 0,
                "required_capabilities_any_of": ["forms:read"]
            }]
        }))
        .expect("policy parses");
        let wire = serde_json::to_value(UpdateNavigationPolicyRequestV1::from(&policy))
            .expect("request serializes");
        let contribution = &wire["contributions"][0];

        assert_eq!(contribution["id"], "tessara.forms.navigation");
        assert_eq!(wire["expected_revision"], 7);
        assert_eq!(contribution.get("route"), None);
        assert_eq!(contribution.get("destination"), None);
        assert_eq!(contribution.get("required_capabilities_any_of"), None);
        assert_eq!(wire.get("schema_version"), Some(&Value::from(1)));
    }

    #[test]
    fn unsupported_http_schema_fails_closed_before_rendering() {
        let error = serde_json::from_value::<super::ModuleInventoryResponseV1>(json!({
            "schema_version": 2,
            "installation": {
                "id": "installation-1",
                "created_at": "2026-07-14T12:00:00Z"
            },
            "core_runtime": {
                "provenance": "unresolved",
                "observed_version": "0.1.0",
                "finding_code": "core_release_provenance_unresolved",
                "observed_at": "2026-07-14T12:00:00Z"
            },
            "entries": []
        }))
        .expect_err("v2 must not render through the v1 projection");

        assert!(
            error
                .to_string()
                .contains("unsupported Module Management schema version 2")
        );
    }
}
