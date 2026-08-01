//! Deterministic application composition contracts and resolution.
//!
//! This crate is deliberately policy-neutral and side-effect free. Core and
//! the Supervisor CLI use the same functions so a Blueprint cannot resolve to
//! different artifacts depending on its entrypoint.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use chrono::{DateTime, Utc};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tessara_module_contract::ArtifactDigest;
use uuid::Uuid;

pub const BLUEPRINT_API_V1: &str = "tessara.io/application-blueprint/v1";
pub const CATALOG_API_V1: &str = "tessara.io/release-catalog/v1";
pub const LOCKFILE_API_V1: &str = "tessara.io/application-lockfile/v1";
pub const PLAN_API_V1: &str = "tessara.io/materialization-plan/v1";
pub const AUTHORIZATION_API_V1: &str = "tessara.io/apply-authorization/v1";
pub const OPERATION_API_V1: &str = "tessara.io/composition-operation/v1";
pub const RECEIPT_API_V1: &str = "tessara.io/installation-receipt/v1";
pub const ENGINE_VERSION_V1: &str = "1.0.0";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationBlueprintV1 {
    pub api_version: String,
    pub installation_id: Uuid,
    pub revision: u64,
    pub core: CoreSelectionV1,
    #[serde(default)]
    pub modules: Vec<ModuleSelectionV1>,
    #[serde(default)]
    pub navigation: Vec<NavigationPolicyEntryV1>,
    #[serde(default)]
    pub roles: Vec<RoleDefinitionV1>,
    pub administrator_enrollment_role: String,
    #[serde(default)]
    pub secret_references: BTreeMap<String, SecretReferenceV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreSelectionV1 {
    pub version_requirement: VersionReq,
    #[serde(default)]
    pub configuration: Value,
    #[serde(default)]
    pub bootstrap: Option<BootstrapInputV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleSelectionV1 {
    pub definition_id: String,
    pub version_requirement: VersionReq,
    pub enabled: bool,
    #[serde(default)]
    pub dependency_bindings: BTreeMap<String, String>,
    #[serde(default)]
    pub configuration: Value,
    #[serde(default)]
    pub bootstrap: Option<BootstrapInputV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
pub enum BootstrapInputV1 {
    Inline {
        schema_version: String,
        value: Value,
    },
    LocalCas {
        schema_version: String,
        digest: ArtifactDigest,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretReferenceV1 {
    pub provider: SecretProviderV1,
    pub name: String,
    pub revision: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretProviderV1 {
    Environment,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationPolicyEntryV1 {
    pub destination_id: String,
    pub group_id: String,
    pub order: u32,
    pub visible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleDefinitionV1 {
    pub name: String,
    pub capabilities: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseCatalogV1 {
    pub api_version: String,
    pub catalog_id: String,
    pub revision: u64,
    pub issued_at: DateTime<Utc>,
    pub core_releases: Vec<CoreCatalogReleaseV1>,
    #[serde(default)]
    pub module_releases: Vec<ModuleCatalogReleaseV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreCatalogReleaseV1 {
    pub version: Version,
    pub core_image: ArtifactDigest,
    pub gateway_image: ArtifactDigest,
    pub database_image: ArtifactDigest,
    pub deployment_profile: String,
    pub capability_floor_version: String,
    pub capability_floor: BTreeSet<String>,
    pub configuration_schema_version: String,
    #[serde(default)]
    pub provided_contracts: BTreeMap<String, Version>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleCatalogReleaseV1 {
    pub definition_id: String,
    pub version: Version,
    pub manifest_digest: ArtifactDigest,
    pub runtime_image: ArtifactDigest,
    pub deployment_profile: String,
    pub configuration_schema_version: String,
    #[serde(default)]
    pub bootstrap_schema_version: Option<String>,
    #[serde(default)]
    pub provided_contracts: BTreeMap<String, Version>,
    #[serde(default)]
    pub dependencies: Vec<ContractDependencyV1>,
    #[serde(default)]
    pub feature_declarations: Vec<Value>,
    #[serde(default)]
    pub contribution_schemas: BTreeMap<String, Value>,
    #[serde(default)]
    pub configuration_schema: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractDependencyV1 {
    pub binding_key: String,
    pub contract_id: String,
    pub version_requirement: VersionReq,
    pub optional: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationLockfileV1 {
    pub api_version: String,
    pub installation_id: Uuid,
    pub blueprint_revision: u64,
    pub blueprint_digest: ArtifactDigest,
    pub catalog_digest: ArtifactDigest,
    pub composition_engine_version: Version,
    pub composition_schema_version: u16,
    pub supervisor_contract_version: Version,
    pub deployment_adapter_version: Version,
    pub core: ResolvedCoreReleaseV1,
    pub modules: Vec<ResolvedModuleReleaseV1>,
    pub navigation: Vec<NavigationPolicyEntryV1>,
    pub navigation_digest: ArtifactDigest,
    pub roles: Vec<RoleDefinitionV1>,
    pub role_policy_digest: ArtifactDigest,
    pub administrator_enrollment_role: String,
    pub capability_floor_version: String,
    pub secret_references: BTreeMap<String, SecretReferenceV1>,
    pub materialization_plan: MaterializationPlanV1,
    pub materialization_plan_digest: ArtifactDigest,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedCoreReleaseV1 {
    pub version: Version,
    pub core_image: ArtifactDigest,
    pub gateway_image: ArtifactDigest,
    pub database_image: ArtifactDigest,
    pub deployment_profile: String,
    pub configuration_schema_version: String,
    pub configuration: Value,
    pub configuration_digest: ArtifactDigest,
    pub bootstrap: Option<BootstrapInputV1>,
    pub bootstrap_digest: Option<ArtifactDigest>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedModuleReleaseV1 {
    pub definition_id: String,
    pub version: Version,
    pub manifest_digest: ArtifactDigest,
    pub runtime_image: ArtifactDigest,
    pub deployment_profile: String,
    pub enabled: bool,
    pub configuration_schema_version: String,
    pub configuration: Value,
    pub configuration_digest: ArtifactDigest,
    pub bootstrap_schema_version: Option<String>,
    pub bootstrap: Option<BootstrapInputV1>,
    pub bootstrap_digest: Option<ArtifactDigest>,
    pub dependency_bindings: BTreeMap<String, ResolvedContractBindingV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedContractBindingV1 {
    pub provider: String,
    pub contract_id: String,
    pub contract_version: Version,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializationPlanV1 {
    pub api_version: String,
    pub installation_id: Uuid,
    pub desired_revision: u64,
    pub actions: Vec<MaterializationActionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyAuthorizationV1 {
    pub api_version: String,
    pub operation: ApplyOperationKindV1,
    pub installation_id: Uuid,
    pub base_receipt_digest: Option<ArtifactDigest>,
    pub target_plan_digest: ArtifactDigest,
    pub desired_revision: u64,
    pub apply_sequence: u64,
    pub nonce: Uuid,
    pub idempotency_key: String,
    pub initiator: ActorEvidenceV1,
    pub approver: ActorEvidenceV1,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(default)]
    pub approved_effects: BTreeSet<ApprovedEffectV1>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplyOperationKindV1 {
    Materialize,
    EmergencyDisable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorEvidenceV1 {
    pub actor_id: String,
    pub actor_kind: String,
    pub authority: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovedEffectV1 {
    Install,
    Upgrade,
    Configure,
    Bootstrap,
    Enable,
    Disable,
    DestroyData,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionOperationStateV1 {
    Accepted,
    Acquiring,
    Provisioning,
    Migrating,
    Configuring,
    Bootstrapping,
    HealthChecking,
    Switching,
    Verifying,
    Succeeded,
    Failed,
    RolledBack,
}

impl CompositionOperationStateV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Acquiring => "acquiring",
            Self::Provisioning => "provisioning",
            Self::Migrating => "migrating",
            Self::Configuring => "configuring",
            Self::Bootstrapping => "bootstrapping",
            Self::HealthChecking => "health_checking",
            Self::Switching => "switching",
            Self::Verifying => "verifying",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::RolledBack => "rolled_back",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionOperationV1 {
    pub api_version: String,
    pub operation_id: Uuid,
    pub installation_id: Uuid,
    pub idempotency_key: String,
    pub plan_digest: ArtifactDigest,
    pub authorization_digest: ArtifactDigest,
    pub state: CompositionOperationStateV1,
    pub accepted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub finding: Option<CompositionFindingV1>,
    pub receipt_digest: Option<ArtifactDigest>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapReceiptV1 {
    pub owner: String,
    pub schema_version: String,
    pub input_digest: ArtifactDigest,
    pub result_digest: ArtifactDigest,
    pub changed: bool,
    #[serde(default)]
    pub resource_ids: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBootstrapRequestV1<T> {
    pub installation_id: Uuid,
    pub desired_revision: u64,
    pub idempotency_key: String,
    pub input_digest: ArtifactDigest,
    pub input: T,
}

impl<T: Serialize> OwnerBootstrapRequestV1<T> {
    pub fn validate_input_digest(&self) -> Result<bool, serde_json::Error> {
        Ok(canonical_digest(&self.input)? == self.input_digest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBootstrapResponseV1 {
    pub receipt: BootstrapReceiptV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationReceiptV1 {
    pub api_version: String,
    pub installation_id: Uuid,
    pub revision: u64,
    pub lockfile_digest: ArtifactDigest,
    pub plan_digest: ArtifactDigest,
    pub authorization_digest: ArtifactDigest,
    pub composition_engine_version: Version,
    pub supervisor_version: Version,
    pub deployment_adapter_version: Version,
    pub desired_enablement: BTreeMap<String, bool>,
    pub observed_enablement: BTreeMap<String, bool>,
    pub observed_artifacts: BTreeMap<String, ArtifactDigest>,
    pub configuration_digests: BTreeMap<String, ArtifactDigest>,
    pub bootstrap_receipts: Vec<BootstrapReceiptV1>,
    pub applied_at: DateTime<Utc>,
    pub previous_receipt_digest: Option<ArtifactDigest>,
    pub no_op: bool,
}

impl ApplyAuthorizationV1 {
    pub fn validate_for(
        &self,
        plan: &MaterializationPlanV1,
        plan_digest: &ArtifactDigest,
        current_receipt: Option<&ArtifactDigest>,
        now: DateTime<Utc>,
    ) -> Result<(), CompositionFindingV1> {
        let invalid = |code: &str, path: &str, message: &str| CompositionFindingV1 {
            code: code.into(),
            severity: FindingSeverityV1::Error,
            path: path.into(),
            message: message.into(),
        };
        if self.api_version != AUTHORIZATION_API_V1 {
            return Err(invalid(
                "authorization_api_version_unsupported",
                "/api_version",
                "authorization API version is unsupported",
            ));
        }
        if self.installation_id != plan.installation_id {
            return Err(invalid(
                "authorization_installation_mismatch",
                "/installation_id",
                "authorization belongs to another installation",
            ));
        }
        if &self.target_plan_digest != plan_digest {
            return Err(invalid(
                "authorization_plan_mismatch",
                "/target_plan_digest",
                "authorization does not bind this plan",
            ));
        }
        if self.desired_revision != plan.desired_revision || self.apply_sequence == 0 {
            return Err(invalid(
                "authorization_revision_invalid",
                "/desired_revision",
                "authorization revision or apply sequence is invalid",
            ));
        }
        if self.base_receipt_digest.as_ref() != current_receipt {
            return Err(invalid(
                "authorization_stale_base",
                "/base_receipt_digest",
                "authorization does not bind the current receipt",
            ));
        }
        if now < self.issued_at || now >= self.expires_at || self.expires_at <= self.issued_at {
            return Err(invalid(
                "authorization_expired",
                "/expires_at",
                "authorization is not active",
            ));
        }
        if self.idempotency_key.trim().is_empty()
            || self.approver.authority != "composition:approve"
        {
            return Err(invalid(
                "authorization_approver_invalid",
                "/approver",
                "composition approval authority is required",
            ));
        }
        if self
            .approved_effects
            .contains(&ApprovedEffectV1::DestroyData)
        {
            return Err(invalid(
                "authorization_destructive_effect_unsupported",
                "/approved_effects",
                "Sprint 6F does not materialize destructive data effects",
            ));
        }
        if self.operation == ApplyOperationKindV1::EmergencyDisable
            && (self
                .reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
                || self.approved_effects != BTreeSet::from([ApprovedEffectV1::Disable]))
        {
            return Err(invalid(
                "emergency_disable_invalid",
                "/approved_effects",
                "emergency disable requires a reason and only the disable effect",
            ));
        }
        if self.operation == ApplyOperationKindV1::Materialize
            && self.approved_effects != required_effects(plan)
        {
            return Err(invalid(
                "authorization_effect_scope_mismatch",
                "/approved_effects",
                "approved effects must exactly match the materialization plan",
            ));
        }
        Ok(())
    }
}

pub fn required_effects(plan: &MaterializationPlanV1) -> BTreeSet<ApprovedEffectV1> {
    let mut effects = BTreeSet::new();
    for action in &plan.actions {
        match action {
            MaterializationActionV1::AcquireImage { .. }
            | MaterializationActionV1::ProvisionDatabase { .. }
            | MaterializationActionV1::Migrate { .. } => {
                effects.insert(ApprovedEffectV1::Install);
            }
            MaterializationActionV1::Configure { .. } => {
                effects.insert(ApprovedEffectV1::Configure);
            }
            MaterializationActionV1::Bootstrap { .. } => {
                effects.insert(ApprovedEffectV1::Bootstrap);
            }
            MaterializationActionV1::SetEnablement { enabled: true, .. } => {
                effects.insert(ApprovedEffectV1::Enable);
            }
            MaterializationActionV1::SetEnablement { enabled: false, .. } => {
                effects.insert(ApprovedEffectV1::Disable);
            }
            MaterializationActionV1::SwitchTraffic { .. } => {
                effects.insert(ApprovedEffectV1::Upgrade);
            }
            MaterializationActionV1::HealthGate { .. }
            | MaterializationActionV1::VerifyReadBack => {}
        }
    }
    effects
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum MaterializationActionV1 {
    AcquireImage {
        component: String,
        digest: ArtifactDigest,
    },
    ProvisionDatabase {
        owner: String,
    },
    Migrate {
        owner: String,
        image: ArtifactDigest,
    },
    Configure {
        owner: String,
        digest: ArtifactDigest,
    },
    Bootstrap {
        owner: String,
        input_digest: ArtifactDigest,
    },
    SetEnablement {
        definition_id: String,
        enabled: bool,
    },
    HealthGate {
        owner: String,
    },
    SwitchTraffic {
        owner: String,
    },
    VerifyReadBack,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionFindingV1 {
    pub code: String,
    pub severity: FindingSeverityV1,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverityV1 {
    Error,
    Warning,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("application composition failed validation")]
pub struct CompositionError {
    pub findings: Vec<CompositionFindingV1>,
}

pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    serde_jcs::to_vec(value)
}

pub fn canonical_digest<T: Serialize>(value: &T) -> Result<ArtifactDigest, serde_json::Error> {
    let bytes = canonical_json(value)?;
    Ok(
        ArtifactDigest::new(format!("sha256:{:x}", Sha256::digest(bytes)))
            .expect("SHA-256 output is a valid artifact digest"),
    )
}

/// Returns the stable runtime identity shared by Core and the Supervisor for
/// one module in one installation.
pub fn module_instance_id(installation_id: Uuid, definition_id: &str) -> Uuid {
    let digest = canonical_digest(&(installation_id, definition_id, "module-instance"))
        .expect("module instance identity inputs are always serializable");
    let hex = digest
        .as_str()
        .strip_prefix("sha256:")
        .expect("canonical digests use the sha256 prefix");
    let mut bytes = [0_u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .expect("canonical digests contain hexadecimal bytes");
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

pub fn acquire_bootstrap_input(
    input: &BootstrapInputV1,
    local_cas_root: &Path,
) -> Result<Vec<u8>, BootstrapAcquisitionError> {
    match input {
        BootstrapInputV1::Inline { value, .. } => {
            canonical_json(value).map_err(BootstrapAcquisitionError::Json)
        }
        BootstrapInputV1::LocalCas { digest, .. } => {
            let hex = digest
                .as_str()
                .strip_prefix("sha256:")
                .ok_or(BootstrapAcquisitionError::InvalidDigest)?;
            let path = local_cas_root.join("sha256").join(hex);
            let bytes = std::fs::read(path).map_err(BootstrapAcquisitionError::Io)?;
            let observed = ArtifactDigest::new(format!("sha256:{:x}", Sha256::digest(&bytes)))
                .expect("SHA-256 output is valid");
            if &observed != digest {
                return Err(BootstrapAcquisitionError::DigestMismatch {
                    expected: digest.clone(),
                    observed,
                });
            }
            Ok(bytes)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BootstrapAcquisitionError {
    #[error("inline bootstrap canonicalization failed: {0}")]
    Json(serde_json::Error),
    #[error("local CAS bootstrap input could not be read: {0}")]
    Io(std::io::Error),
    #[error("local CAS bootstrap digest is invalid")]
    InvalidDigest,
    #[error("local CAS bootstrap digest mismatch: expected {expected}, observed {observed}")]
    DigestMismatch {
        expected: ArtifactDigest,
        observed: ArtifactDigest,
    },
}

pub fn resolve(
    blueprint: &ApplicationBlueprintV1,
    catalog: &ReleaseCatalogV1,
) -> Result<ApplicationLockfileV1, CompositionError> {
    let mut findings = validate_blueprint(blueprint, catalog);
    if !findings.is_empty() {
        findings.sort_by(|a, b| (&a.path, &a.code).cmp(&(&b.path, &b.code)));
        return Err(CompositionError { findings });
    }

    let core = catalog
        .core_releases
        .iter()
        .filter(|release| blueprint.core.version_requirement.matches(&release.version))
        .max_by(|a, b| a.version.cmp(&b.version))
        .expect("validated Core selection");
    let role = blueprint
        .roles
        .iter()
        .find(|role| role.name == blueprint.administrator_enrollment_role)
        .expect("validated enrollment role");
    debug_assert!(core.capability_floor.is_subset(&role.capabilities));

    let mut modules = Vec::new();
    for selection in sorted_module_selections(&blueprint.modules) {
        let release = select_module(selection, catalog).expect("validated module selection");
        let mut bindings = BTreeMap::new();
        for dependency in &release.dependencies {
            if let Some(provider) = selection.dependency_bindings.get(&dependency.binding_key) {
                let version =
                    provider_contract_version(provider, &dependency.contract_id, core, catalog)
                        .expect("validated dependency binding");
                bindings.insert(
                    dependency.binding_key.clone(),
                    ResolvedContractBindingV1 {
                        provider: provider.clone(),
                        contract_id: dependency.contract_id.clone(),
                        contract_version: version,
                    },
                );
            }
        }
        let configuration_digest = digest_or_infallible(&selection.configuration);
        let bootstrap_digest = selection.bootstrap.as_ref().map(digest_or_infallible);
        modules.push(ResolvedModuleReleaseV1 {
            definition_id: selection.definition_id.clone(),
            version: release.version.clone(),
            manifest_digest: release.manifest_digest.clone(),
            runtime_image: release.runtime_image.clone(),
            deployment_profile: release.deployment_profile.clone(),
            enabled: selection.enabled,
            configuration_schema_version: release.configuration_schema_version.clone(),
            configuration: selection.configuration.clone(),
            configuration_digest,
            bootstrap_schema_version: release.bootstrap_schema_version.clone(),
            bootstrap: selection.bootstrap.clone(),
            bootstrap_digest,
            dependency_bindings: bindings,
        });
    }

    let core_configuration_digest = digest_or_infallible(&blueprint.core.configuration);
    let core_bootstrap_digest = blueprint.core.bootstrap.as_ref().map(digest_or_infallible);
    let resolved_core = ResolvedCoreReleaseV1 {
        version: core.version.clone(),
        core_image: core.core_image.clone(),
        gateway_image: core.gateway_image.clone(),
        database_image: core.database_image.clone(),
        deployment_profile: core.deployment_profile.clone(),
        configuration_schema_version: core.configuration_schema_version.clone(),
        configuration: blueprint.core.configuration.clone(),
        configuration_digest: core_configuration_digest,
        bootstrap: blueprint.core.bootstrap.clone(),
        bootstrap_digest: core_bootstrap_digest,
    };
    let actions = materialization_actions(&resolved_core, &modules);
    let plan = MaterializationPlanV1 {
        api_version: PLAN_API_V1.into(),
        installation_id: blueprint.installation_id,
        desired_revision: blueprint.revision,
        actions,
    };
    let plan_digest = digest_or_infallible(&plan);
    let mut navigation = blueprint.navigation.clone();
    navigation.sort_by(|a, b| {
        (&a.group_id, a.order, &a.destination_id).cmp(&(&b.group_id, b.order, &b.destination_id))
    });
    let mut roles = blueprint.roles.clone();
    roles.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(ApplicationLockfileV1 {
        api_version: LOCKFILE_API_V1.into(),
        installation_id: blueprint.installation_id,
        blueprint_revision: blueprint.revision,
        blueprint_digest: digest_or_infallible(blueprint),
        catalog_digest: digest_or_infallible(catalog),
        composition_engine_version: Version::parse(ENGINE_VERSION_V1).unwrap(),
        composition_schema_version: 1,
        supervisor_contract_version: Version::new(1, 0, 0),
        deployment_adapter_version: Version::new(1, 0, 0),
        core: resolved_core,
        modules,
        navigation_digest: digest_or_infallible(&navigation),
        navigation,
        role_policy_digest: digest_or_infallible(&roles),
        roles,
        administrator_enrollment_role: blueprint.administrator_enrollment_role.clone(),
        capability_floor_version: core.capability_floor_version.clone(),
        secret_references: blueprint.secret_references.clone(),
        materialization_plan: plan,
        materialization_plan_digest: plan_digest,
    })
}

pub fn semantic_diff(
    current: Option<&ApplicationLockfileV1>,
    desired: &ApplicationLockfileV1,
) -> Vec<String> {
    let Some(current) = current else {
        return desired
            .materialization_plan
            .actions
            .iter()
            .map(|action| format!("add:{}", action_name(action)))
            .collect();
    };
    if current.materialization_plan_digest == desired.materialization_plan_digest {
        return Vec::new();
    }
    let mut changes = Vec::new();
    if current.core.version != desired.core.version {
        changes.push(format!(
            "core:{}->{}",
            current.core.version, desired.core.version
        ));
    }
    let current_modules = current
        .modules
        .iter()
        .map(|m| (&m.definition_id, m))
        .collect::<BTreeMap<_, _>>();
    let desired_modules = desired
        .modules
        .iter()
        .map(|m| (&m.definition_id, m))
        .collect::<BTreeMap<_, _>>();
    for (definition, module) in &desired_modules {
        match current_modules.get(definition) {
            None => changes.push(format!("module:add:{definition}@{}", module.version)),
            Some(old) if old.version != module.version => changes.push(format!(
                "module:update:{definition}:{}->{}",
                old.version, module.version
            )),
            Some(old) if old.enabled != module.enabled => changes.push(format!(
                "module:enablement:{definition}:{}->{}",
                old.enabled, module.enabled
            )),
            Some(old) if old.configuration_digest != module.configuration_digest => {
                changes.push(format!("module:configure:{definition}"))
            }
            _ => {}
        }
    }
    for definition in current_modules.keys() {
        if !desired_modules.contains_key(definition) {
            changes.push(format!("module:remove:{definition}"));
        }
    }
    if current.navigation_digest != desired.navigation_digest {
        changes.push("navigation:update".into());
    }
    if current.role_policy_digest != desired.role_policy_digest {
        changes.push("roles:update".into());
    }
    changes.sort();
    changes
}

fn validate_blueprint(
    blueprint: &ApplicationBlueprintV1,
    catalog: &ReleaseCatalogV1,
) -> Vec<CompositionFindingV1> {
    let mut findings = Vec::new();
    if blueprint.api_version != BLUEPRINT_API_V1 {
        push_error(
            &mut findings,
            "blueprint_api_version_unsupported",
            "/api_version",
            BLUEPRINT_API_V1,
        );
    }
    if catalog.api_version != CATALOG_API_V1 {
        push_error(
            &mut findings,
            "catalog_api_version_unsupported",
            "/catalog/api_version",
            CATALOG_API_V1,
        );
    }
    if blueprint.revision == 0 {
        push_error(
            &mut findings,
            "blueprint_revision_invalid",
            "/revision",
            "revision must be greater than zero",
        );
    }
    let core = catalog
        .core_releases
        .iter()
        .filter(|r| blueprint.core.version_requirement.matches(&r.version))
        .max_by(|a, b| a.version.cmp(&b.version));
    if core.is_none() {
        push_error(
            &mut findings,
            "core_release_missing",
            "/core/version_requirement",
            "no compatible Core Release is available",
        );
    }
    let role = blueprint
        .roles
        .iter()
        .find(|role| role.name == blueprint.administrator_enrollment_role);
    match (role, core) {
        (None, _) => push_error(
            &mut findings,
            "enrollment_role_missing",
            "/administrator_enrollment_role",
            "the designated role is not declared",
        ),
        (Some(role), Some(core)) if !core.capability_floor.is_subset(&role.capabilities) => {
            push_error(
                &mut findings,
                "enrollment_role_below_capability_floor",
                "/administrator_enrollment_role",
                "the designated role does not cover the Core capability floor",
            )
        }
        _ => {}
    }
    let mut definitions = BTreeSet::new();
    for (index, selection) in blueprint.modules.iter().enumerate() {
        let base = format!("/modules/{index}");
        if !definitions.insert(&selection.definition_id) {
            push_error(
                &mut findings,
                "module_selection_duplicate",
                format!("{base}/definition_id"),
                "a Module Definition may be selected only once",
            );
            continue;
        }
        let Some(release) = select_module(selection, catalog) else {
            push_error(
                &mut findings,
                "module_release_missing",
                format!("{base}/version_requirement"),
                "no compatible Module Release is available",
            );
            continue;
        };
        if release.deployment_profile != "tessara-oci-v1" {
            push_error(
                &mut findings,
                "deployment_profile_unsupported",
                format!("{base}/deployment_profile"),
                "only tessara-oci-v1 is supported",
            );
        }
        for dependency in &release.dependencies {
            let path = format!("{base}/dependency_bindings/{}", dependency.binding_key);
            let Some(provider) = selection.dependency_bindings.get(&dependency.binding_key) else {
                if !dependency.optional {
                    push_error(
                        &mut findings,
                        "dependency_unbound",
                        path,
                        "required dependency has no provider binding",
                    );
                }
                continue;
            };
            match core.and_then(|core| {
                provider_contract_version(provider, &dependency.contract_id, core, catalog)
            }) {
                None => push_error(
                    &mut findings,
                    "dependency_provider_missing",
                    path,
                    "bound provider does not advertise the required contract",
                ),
                Some(version) if !dependency.version_requirement.matches(&version) => push_error(
                    &mut findings,
                    "dependency_incompatible",
                    path,
                    "bound provider contract version is incompatible",
                ),
                _ => {}
            }
        }
        if selection.bootstrap.is_some() && release.bootstrap_schema_version.is_none() {
            push_error(
                &mut findings,
                "bootstrap_unsupported",
                format!("{base}/bootstrap"),
                "the selected release does not declare a bootstrap contract",
            );
        }
    }
    validate_module_cycles(blueprint, catalog, &mut findings);
    let mut secret_names = BTreeSet::new();
    for (alias, reference) in &blueprint.secret_references {
        if alias.trim().is_empty()
            || reference.name.trim().is_empty()
            || reference.revision.trim().is_empty()
            || !secret_names.insert(&reference.name)
        {
            push_error(
                &mut findings,
                "secret_reference_invalid",
                format!("/secret_references/{alias}"),
                "secret aliases, names, and revisions must be non-empty and names unique",
            );
        }
    }
    findings
}

fn validate_module_cycles(
    blueprint: &ApplicationBlueprintV1,
    catalog: &ReleaseCatalogV1,
    findings: &mut Vec<CompositionFindingV1>,
) {
    let selected = blueprint
        .modules
        .iter()
        .map(|m| (m.definition_id.as_str(), m))
        .collect::<BTreeMap<_, _>>();
    let mut edges = BTreeMap::<&str, Vec<&str>>::new();
    for selection in &blueprint.modules {
        let Some(release) = select_module(selection, catalog) else {
            continue;
        };
        for dependency in &release.dependencies {
            if let Some(provider) = selection.dependency_bindings.get(&dependency.binding_key) {
                if selected.contains_key(provider.as_str()) {
                    edges
                        .entry(&selection.definition_id)
                        .or_default()
                        .push(provider);
                }
            }
        }
    }
    fn visit<'a>(
        node: &'a str,
        edges: &BTreeMap<&'a str, Vec<&'a str>>,
        visiting: &mut BTreeSet<&'a str>,
        visited: &mut BTreeSet<&'a str>,
    ) -> bool {
        if visiting.contains(node) {
            return true;
        }
        if !visited.insert(node) {
            return false;
        }
        visiting.insert(node);
        let cycle = edges.get(node).is_some_and(|next| {
            next.iter()
                .any(|candidate| visit(candidate, edges, visiting, visited))
        });
        visiting.remove(node);
        cycle
    }
    let mut visited = BTreeSet::new();
    for definition in selected.keys() {
        if visit(definition, &edges, &mut BTreeSet::new(), &mut visited) {
            push_error(
                findings,
                "dependency_cycle",
                "/modules",
                "selected module dependency bindings contain a cycle",
            );
            break;
        }
    }
}

fn sorted_module_selections(modules: &[ModuleSelectionV1]) -> Vec<&ModuleSelectionV1> {
    let mut modules = modules.iter().collect::<Vec<_>>();
    modules.sort_by(|a, b| a.definition_id.cmp(&b.definition_id));
    modules
}

fn select_module<'a>(
    selection: &ModuleSelectionV1,
    catalog: &'a ReleaseCatalogV1,
) -> Option<&'a ModuleCatalogReleaseV1> {
    catalog
        .module_releases
        .iter()
        .filter(|r| {
            r.definition_id == selection.definition_id
                && selection.version_requirement.matches(&r.version)
        })
        .max_by(|a, b| a.version.cmp(&b.version))
}

fn provider_contract_version(
    provider: &str,
    contract: &str,
    core: &CoreCatalogReleaseV1,
    catalog: &ReleaseCatalogV1,
) -> Option<Version> {
    if provider == "core" {
        return core.provided_contracts.get(contract).cloned();
    }
    catalog
        .module_releases
        .iter()
        .filter(|r| r.definition_id == provider)
        .max_by(|a, b| a.version.cmp(&b.version))
        .and_then(|r| r.provided_contracts.get(contract).cloned())
}

fn materialization_actions(
    core: &ResolvedCoreReleaseV1,
    modules: &[ResolvedModuleReleaseV1],
) -> Vec<MaterializationActionV1> {
    let mut actions = vec![
        MaterializationActionV1::AcquireImage {
            component: "database".into(),
            digest: core.database_image.clone(),
        },
        MaterializationActionV1::AcquireImage {
            component: "core".into(),
            digest: core.core_image.clone(),
        },
        MaterializationActionV1::AcquireImage {
            component: "gateway".into(),
            digest: core.gateway_image.clone(),
        },
        MaterializationActionV1::ProvisionDatabase {
            owner: "core".into(),
        },
        MaterializationActionV1::Migrate {
            owner: "core".into(),
            image: core.core_image.clone(),
        },
        MaterializationActionV1::Configure {
            owner: "core".into(),
            digest: core.configuration_digest.clone(),
        },
    ];
    if let Some(input_digest) = &core.bootstrap_digest {
        actions.push(MaterializationActionV1::Bootstrap {
            owner: "core".into(),
            input_digest: input_digest.clone(),
        });
    }
    actions.push(MaterializationActionV1::HealthGate {
        owner: "core".into(),
    });
    for module in modules {
        actions.push(MaterializationActionV1::AcquireImage {
            component: module.definition_id.clone(),
            digest: module.runtime_image.clone(),
        });
        actions.push(MaterializationActionV1::ProvisionDatabase {
            owner: module.definition_id.clone(),
        });
        actions.push(MaterializationActionV1::Migrate {
            owner: module.definition_id.clone(),
            image: module.runtime_image.clone(),
        });
        actions.push(MaterializationActionV1::Configure {
            owner: module.definition_id.clone(),
            digest: module.configuration_digest.clone(),
        });
        if let Some(input_digest) = &module.bootstrap_digest {
            actions.push(MaterializationActionV1::Bootstrap {
                owner: module.definition_id.clone(),
                input_digest: input_digest.clone(),
            });
        }
        actions.push(MaterializationActionV1::SetEnablement {
            definition_id: module.definition_id.clone(),
            enabled: module.enabled,
        });
        actions.push(MaterializationActionV1::HealthGate {
            owner: module.definition_id.clone(),
        });
        if module.enabled {
            actions.push(MaterializationActionV1::SwitchTraffic {
                owner: module.definition_id.clone(),
            });
        }
    }
    actions.push(MaterializationActionV1::SwitchTraffic {
        owner: "core".into(),
    });
    actions.push(MaterializationActionV1::VerifyReadBack);
    actions
}

fn digest_or_infallible<T: Serialize>(value: &T) -> ArtifactDigest {
    canonical_digest(value).expect("composition contracts serialize to canonical JSON")
}

fn action_name(action: &MaterializationActionV1) -> &'static str {
    match action {
        MaterializationActionV1::AcquireImage { .. } => "acquire_image",
        MaterializationActionV1::ProvisionDatabase { .. } => "provision_database",
        MaterializationActionV1::Migrate { .. } => "migrate",
        MaterializationActionV1::Configure { .. } => "configure",
        MaterializationActionV1::Bootstrap { .. } => "bootstrap",
        MaterializationActionV1::SetEnablement { .. } => "set_enablement",
        MaterializationActionV1::HealthGate { .. } => "health_gate",
        MaterializationActionV1::SwitchTraffic { .. } => "switch_traffic",
        MaterializationActionV1::VerifyReadBack => "verify_read_back",
    }
}

fn push_error(
    findings: &mut Vec<CompositionFindingV1>,
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) {
    findings.push(CompositionFindingV1 {
        code: code.into(),
        severity: FindingSeverityV1::Error,
        path: path.into(),
        message: message.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn digest(byte: char) -> ArtifactDigest {
        ArtifactDigest::new(format!("sha256:{}", byte.to_string().repeat(64))).unwrap()
    }

    #[test]
    fn module_instance_identity_is_stable_and_definition_scoped() {
        let installation_id = Uuid::parse_str("01980000-0000-7000-8000-00000000006f").unwrap();
        assert_eq!(
            module_instance_id(installation_id, "tessara.dashboards"),
            module_instance_id(installation_id, "tessara.dashboards")
        );
        assert_ne!(
            module_instance_id(installation_id, "tessara.dashboards"),
            module_instance_id(installation_id, "tessara.reference.scoped-records")
        );
    }

    fn catalog() -> ReleaseCatalogV1 {
        ReleaseCatalogV1 {
            api_version: CATALOG_API_V1.into(),
            catalog_id: "tessara.local".into(),
            revision: 1,
            issued_at: "2026-08-01T12:00:00Z".parse().unwrap(),
            core_releases: vec![CoreCatalogReleaseV1 {
                version: Version::new(1, 0, 0),
                core_image: digest('a'),
                gateway_image: digest('b'),
                database_image: digest('c'),
                deployment_profile: "tessara-oci-v1".into(),
                capability_floor_version: "core-administration-v1".into(),
                capability_floor: BTreeSet::from([
                    "composition:approve".into(),
                    "core:admin".into(),
                ]),
                configuration_schema_version: "1.0.0".into(),
                provided_contracts: BTreeMap::from([(
                    "tessara.components.component-version".into(),
                    Version::new(1, 0, 0),
                )]),
            }],
            module_releases: vec![ModuleCatalogReleaseV1 {
                definition_id: "tessara.dashboards".into(),
                version: Version::new(2, 0, 2),
                manifest_digest: digest('d'),
                runtime_image: digest('e'),
                deployment_profile: "tessara-oci-v1".into(),
                configuration_schema_version: "1.0.0".into(),
                bootstrap_schema_version: Some("1.0.0".into()),
                provided_contracts: BTreeMap::from([(
                    "tessara.dashboards.dashboard".into(),
                    Version::new(1, 0, 0),
                )]),
                dependencies: vec![ContractDependencyV1 {
                    binding_key: "components".into(),
                    contract_id: "tessara.components.component-version".into(),
                    version_requirement: VersionReq::parse("^1").unwrap(),
                    optional: false,
                }],
                feature_declarations: vec![json!({"id":"tessara.dashboards.composition"})],
                contribution_schemas: BTreeMap::new(),
                configuration_schema: json!({"type":"object"}),
            }],
        }
    }

    fn blueprint() -> ApplicationBlueprintV1 {
        ApplicationBlueprintV1 {
            api_version: BLUEPRINT_API_V1.into(),
            installation_id: Uuid::nil(),
            revision: 1,
            core: CoreSelectionV1 {
                version_requirement: VersionReq::parse("^1").unwrap(),
                configuration: json!({"terminology":"Organization"}),
                bootstrap: None,
            },
            modules: vec![ModuleSelectionV1 {
                definition_id: "tessara.dashboards".into(),
                version_requirement: VersionReq::parse("^2").unwrap(),
                enabled: true,
                dependency_bindings: BTreeMap::from([("components".into(), "core".into())]),
                configuration: json!({"display_label":"Dashboards"}),
                bootstrap: Some(BootstrapInputV1::Inline {
                    schema_version: "1.0.0".into(),
                    value: json!({"dashboards":[]}),
                }),
            }],
            navigation: vec![],
            roles: vec![RoleDefinitionV1 {
                name: "Core Administrator".into(),
                capabilities: BTreeSet::from(["composition:approve".into(), "core:admin".into()]),
            }],
            administrator_enrollment_role: "Core Administrator".into(),
            secret_references: BTreeMap::new(),
        }
    }

    #[test]
    fn equivalent_inputs_produce_identical_lockfile_and_plan_bytes() {
        let first = resolve(&blueprint(), &catalog()).unwrap();
        let second = resolve(&blueprint(), &catalog()).unwrap();
        assert_eq!(
            canonical_json(&first).unwrap(),
            canonical_json(&second).unwrap()
        );
        assert_eq!(
            first.materialization_plan_digest,
            second.materialization_plan_digest
        );
        assert!(semantic_diff(Some(&first), &second).is_empty());
    }

    #[test]
    fn missing_binding_and_capability_floor_fail_with_stable_paths() {
        let mut blueprint = blueprint();
        blueprint.modules[0].dependency_bindings.clear();
        blueprint.roles[0].capabilities.clear();
        let error = resolve(&blueprint, &catalog()).unwrap_err();
        assert_eq!(
            error
                .findings
                .iter()
                .map(|f| (&f.code, &f.path))
                .collect::<Vec<_>>(),
            vec![
                (
                    &"enrollment_role_below_capability_floor".into(),
                    &"/administrator_enrollment_role".into()
                ),
                (
                    &"dependency_unbound".into(),
                    &"/modules/0/dependency_bindings/components".into()
                ),
            ]
        );
    }

    #[test]
    fn strict_json_rejects_unknown_blueprint_fields() {
        let mut value = serde_json::to_value(blueprint()).unwrap();
        value["approval"] = json!(true);
        assert!(serde_json::from_value::<ApplicationBlueprintV1>(value).is_err());
    }

    #[test]
    fn local_cas_acquisition_rejects_tampered_content() {
        let root = std::env::temp_dir().join(format!("tessara-cas-{}", Uuid::new_v4()));
        let directory = root.join("sha256");
        std::fs::create_dir_all(&directory).unwrap();
        let bytes = b"source-exact bootstrap";
        let digest = ArtifactDigest::new(format!("sha256:{:x}", Sha256::digest(bytes))).unwrap();
        let path = directory.join(digest.as_str().trim_start_matches("sha256:"));
        std::fs::write(&path, bytes).unwrap();
        let input = BootstrapInputV1::LocalCas {
            schema_version: "test/v1".into(),
            digest: digest.clone(),
        };
        assert_eq!(acquire_bootstrap_input(&input, &root).unwrap(), bytes);
        std::fs::write(&path, b"tampered").unwrap();
        assert!(matches!(
            acquire_bootstrap_input(&input, &root),
            Err(BootstrapAcquisitionError::DigestMismatch { .. })
        ));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
