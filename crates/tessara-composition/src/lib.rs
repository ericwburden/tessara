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
use tessara_module_contract::{
    ArtifactDigest, AuthorizationAudienceV1, BootstrapValidationAudienceDeclaration,
    CapabilityScopeBindingV1, ModuleManifest, ProtocolEnvelopeError, PurposeBoundSigningKeyV1,
    PurposeBoundVerifyingKeyV1, ServiceActionMethod, SignedEnvelopeV1,
};
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        receipt_bindings: Vec<BootstrapReceiptBindingV1>,
    },
    LocalCas {
        schema_version: String,
        digest: ArtifactDigest,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        receipt_bindings: Vec<BootstrapReceiptBindingV1>,
    },
}

impl BootstrapInputV1 {
    pub fn receipt_bindings(&self) -> &[BootstrapReceiptBindingV1] {
        match self {
            Self::Inline {
                receipt_bindings, ..
            }
            | Self::LocalCas {
                receipt_bindings, ..
            } => receipt_bindings,
        }
    }

    fn locked_payload_digest(&self) -> ArtifactDigest {
        match self {
            Self::Inline { value, .. } => digest_or_infallible(value),
            Self::LocalCas { digest, .. } => digest.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapReceiptBindingV1 {
    pub target_pointer: String,
    pub source_owner: String,
    pub resource_key: String,
    pub value_encoding: BootstrapReceiptValueEncodingV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapReceiptValueEncodingV1 {
    String,
    Json,
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

pub const OWNER_BOOTSTRAP_AUTHORIZATION_SCHEMA_VERSION_V1: u16 = 1;
pub const OWNER_BOOTSTRAP_AUTHORIZATION_MAX_LIFETIME_SECONDS: i64 = 60;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBootstrapAuthorizationV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub owner: AuthorizationAudienceV1,
    pub owner_definition_id: String,
    pub initiator: ActorEvidenceV1,
    pub original_actor_id: Uuid,
    pub capability_scope_bindings: Vec<CapabilityScopeBindingV1>,
    pub provider_actions: Vec<OwnerBootstrapProviderActionV1>,
    pub authorization_revision: u64,
    pub organization_revision: u64,
    pub locked_input_digest: ArtifactDigest,
    pub input_digest: ArtifactDigest,
    pub desired_revision: u64,
    pub apply_sequence: u64,
    pub target_plan_digest: ArtifactDigest,
    pub idempotency_key: String,
    pub correlation_id: Uuid,
    pub jti: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBootstrapProviderActionV1 {
    pub dependency_binding: String,
    pub functional_contract: String,
    pub action: String,
    pub method: ServiceActionMethod,
    pub path: String,
    pub audience: AuthorizationAudienceV1,
}

pub struct OwnerBootstrapAuthorizationContextV1<'a> {
    pub installation_id: Uuid,
    pub owner: &'a AuthorizationAudienceV1,
    pub owner_definition_id: &'a str,
    pub locked_input_digest: &'a ArtifactDigest,
    pub input_digest: &'a ArtifactDigest,
    pub desired_revision: u64,
    pub apply_sequence: u64,
    pub target_plan_digest: &'a ArtifactDigest,
    pub idempotency_key: &'a str,
    pub now: DateTime<Utc>,
}

impl OwnerBootstrapAuthorizationV1 {
    pub fn validate_for(
        &self,
        context: &OwnerBootstrapAuthorizationContextV1<'_>,
    ) -> Result<(), OwnerBootstrapAuthorizationError> {
        if self.schema_version != OWNER_BOOTSTRAP_AUTHORIZATION_SCHEMA_VERSION_V1 {
            return Err(OwnerBootstrapAuthorizationError::UnsupportedSchema);
        }
        if self.installation_id != context.installation_id
            || &self.owner != context.owner
            || self.owner_definition_id != context.owner_definition_id
            || !self.owner.is_valid_for_installation(self.installation_id)
        {
            return Err(OwnerBootstrapAuthorizationError::WrongOwner);
        }
        if &self.locked_input_digest != context.locked_input_digest
            || &self.input_digest != context.input_digest
        {
            return Err(OwnerBootstrapAuthorizationError::WrongInputDigest);
        }
        if self.desired_revision != context.desired_revision
            || self.apply_sequence != context.apply_sequence
            || self.desired_revision == 0
            || self.apply_sequence == 0
            || &self.target_plan_digest != context.target_plan_digest
        {
            return Err(OwnerBootstrapAuthorizationError::WrongApply);
        }
        if self.idempotency_key != context.idempotency_key || self.idempotency_key.trim().is_empty()
        {
            return Err(OwnerBootstrapAuthorizationError::WrongIdempotencyKey);
        }
        if self.initiator.actor_id.trim().is_empty()
            || self.initiator.actor_kind.trim().is_empty()
            || self.initiator.authority.trim().is_empty()
            || self.original_actor_id.is_nil()
            || self.authorization_revision == 0
            || self.organization_revision == 0
            || self.correlation_id.is_nil()
            || self.jti.is_nil()
        {
            return Err(OwnerBootstrapAuthorizationError::InvalidIdentity);
        }
        let lifetime = self.expires_at - self.issued_at;
        if self.issued_at > context.now
            || self.expires_at <= context.now
            || lifetime <= chrono::Duration::zero()
            || lifetime
                > chrono::Duration::seconds(OWNER_BOOTSTRAP_AUTHORIZATION_MAX_LIFETIME_SECONDS)
        {
            return Err(OwnerBootstrapAuthorizationError::Expired);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum OwnerBootstrapAuthorizationError {
    #[error("owner bootstrap authorization schema is unsupported")]
    UnsupportedSchema,
    #[error("owner bootstrap authorization belongs to another owner")]
    WrongOwner,
    #[error("owner bootstrap authorization binds another input")]
    WrongInputDigest,
    #[error("owner bootstrap authorization binds another apply")]
    WrongApply,
    #[error("owner bootstrap authorization binds another idempotency key")]
    WrongIdempotencyKey,
    #[error("owner bootstrap authorization identity is invalid")]
    InvalidIdentity,
    #[error("owner bootstrap authorization is not currently valid")]
    Expired,
}

pub const BOOTSTRAP_DEPENDENCY_VALIDATION_AUTHORIZATION_SCHEMA_VERSION_V1: u16 = 1;
pub const BOOTSTRAP_DEPENDENCY_VALIDATION_AUTHORIZATION_MAX_LIFETIME_SECONDS: i64 = 60;
pub const BOOTSTRAP_DEPENDENCY_VALIDATION_REQUEST_SCHEMA_VERSION_V1: u16 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapDependencyValidationRequestV1 {
    pub schema_version: u16,
    pub input_digest: ArtifactDigest,
    pub desired_revision: u64,
    pub apply_sequence: u64,
    pub target_plan_digest: ArtifactDigest,
    /// Functional-owner payload selected from the exact locked bootstrap
    /// input. Core and the Supervisor move and digest it without interpretation.
    pub payload: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapDependencyValidationAuthorizationV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub module_instance_id: Uuid,
    pub module_definition_id: String,
    pub input_digest: ArtifactDigest,
    pub desired_revision: u64,
    pub apply_sequence: u64,
    pub target_plan_digest: ArtifactDigest,
    pub dependency_binding: String,
    pub functional_contract: String,
    pub functional_contract_version: Version,
    pub action: String,
    pub method: ServiceActionMethod,
    pub path: String,
    pub audience: AuthorizationAudienceV1,
    pub request_digest: ArtifactDigest,
    pub correlation_id: Uuid,
    pub jti: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct BootstrapDependencyValidationContextV1<'a> {
    pub installation_id: Uuid,
    pub module_instance_id: Uuid,
    pub module_definition_id: &'a str,
    pub input_digest: &'a ArtifactDigest,
    pub desired_revision: u64,
    pub apply_sequence: u64,
    pub target_plan_digest: &'a ArtifactDigest,
    pub dependency_binding: &'a str,
    pub functional_contract: &'a str,
    pub functional_contract_version: &'a Version,
    pub action: &'a str,
    pub method: ServiceActionMethod,
    pub path: &'a str,
    pub audience: &'a AuthorizationAudienceV1,
    pub request_digest: &'a ArtifactDigest,
    pub now: DateTime<Utc>,
}

impl BootstrapDependencyValidationAuthorizationV1 {
    pub fn validate_for(
        &self,
        context: &BootstrapDependencyValidationContextV1<'_>,
    ) -> Result<(), BootstrapDependencyValidationAuthorizationError> {
        if self.schema_version != BOOTSTRAP_DEPENDENCY_VALIDATION_AUTHORIZATION_SCHEMA_VERSION_V1 {
            return Err(BootstrapDependencyValidationAuthorizationError::UnsupportedSchema);
        }
        if self.installation_id != context.installation_id
            || self.module_instance_id != context.module_instance_id
            || self.module_definition_id != context.module_definition_id
        {
            return Err(BootstrapDependencyValidationAuthorizationError::WrongOwner);
        }
        if &self.input_digest != context.input_digest {
            return Err(BootstrapDependencyValidationAuthorizationError::WrongInputDigest);
        }
        if self.desired_revision != context.desired_revision
            || self.apply_sequence != context.apply_sequence
            || self.desired_revision == 0
            || self.apply_sequence == 0
            || &self.target_plan_digest != context.target_plan_digest
        {
            return Err(BootstrapDependencyValidationAuthorizationError::WrongApply);
        }
        if self.dependency_binding != context.dependency_binding
            || self.functional_contract != context.functional_contract
            || &self.functional_contract_version != context.functional_contract_version
            || self.action != context.action
            || self.method != context.method
            || self.path != context.path
            || &self.audience != context.audience
        {
            return Err(BootstrapDependencyValidationAuthorizationError::WrongDependency);
        }
        if &self.request_digest != context.request_digest {
            return Err(BootstrapDependencyValidationAuthorizationError::WrongRequestDigest);
        }
        if self.correlation_id.is_nil() || self.jti.is_nil() {
            return Err(BootstrapDependencyValidationAuthorizationError::InvalidIdentity);
        }
        let lifetime = self.expires_at - self.issued_at;
        if self.issued_at > context.now
            || self.expires_at <= context.now
            || lifetime <= chrono::Duration::zero()
            || lifetime
                > chrono::Duration::seconds(
                    BOOTSTRAP_DEPENDENCY_VALIDATION_AUTHORIZATION_MAX_LIFETIME_SECONDS,
                )
        {
            return Err(BootstrapDependencyValidationAuthorizationError::Expired);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum BootstrapDependencyValidationAuthorizationError {
    #[error("bootstrap dependency authorization schema is unsupported")]
    UnsupportedSchema,
    #[error("bootstrap dependency authorization belongs to another owner")]
    WrongOwner,
    #[error("bootstrap dependency authorization binds another input")]
    WrongInputDigest,
    #[error("bootstrap dependency authorization binds another apply")]
    WrongApply,
    #[error("bootstrap dependency authorization grants another dependency")]
    WrongDependency,
    #[error("bootstrap dependency authorization binds another request")]
    WrongRequestDigest,
    #[error("bootstrap dependency authorization identity is invalid")]
    InvalidIdentity,
    #[error("bootstrap dependency authorization is not currently valid")]
    Expired,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapDependencyValidationAuthorizationIssueRequestV1 {
    pub installation_id: Uuid,
    pub owner_definition_id: String,
    pub locked_input_digest: ArtifactDigest,
    pub input_digest: ArtifactDigest,
    pub input: Value,
    pub desired_revision: u64,
    pub apply_sequence: u64,
    pub idempotency_key: String,
    pub apply_authorization: SignedEnvelopeV1<ApplyAuthorizationV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapDependencyValidationTargetV1 {
    pub dependency_binding: String,
    pub functional_contract: String,
    pub functional_contract_version: Version,
    pub action: String,
    pub method: ServiceActionMethod,
    pub path: String,
    pub audience: AuthorizationAudienceV1,
    pub payload_pointer: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapDependencyValidationInvocationV1 {
    pub target: BootstrapDependencyValidationTargetV1,
    pub request: BootstrapDependencyValidationRequestV1,
    pub authorization: SignedEnvelopeV1<BootstrapDependencyValidationAuthorizationV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapDependencyValidationAuthorizationIssueResponseV1 {
    pub authorization: SignedEnvelopeV1<OwnerBootstrapAuthorizationV1>,
    pub validation: Option<BootstrapDependencyValidationInvocationV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBootstrapRequestV1<T> {
    pub installation_id: Uuid,
    pub desired_revision: u64,
    pub apply_sequence: u64,
    pub target_plan_digest: ArtifactDigest,
    pub idempotency_key: String,
    pub locked_input_digest: ArtifactDigest,
    pub input_digest: ArtifactDigest,
    pub authorization: SignedEnvelopeV1<OwnerBootstrapAuthorizationV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_validation: Option<BootstrapDependencyValidationInvocationV1>,
    pub input: T,
}

impl<T: Serialize> OwnerBootstrapRequestV1<T> {
    pub fn validate_input_digest(&self) -> Result<bool, serde_json::Error> {
        Ok(canonical_digest(&self.input)? == self.input_digest)
    }

    pub fn validate_authorization_for(
        &self,
        verifier: &PurposeBoundVerifyingKeyV1,
        owner: &AuthorizationAudienceV1,
        owner_definition_id: &str,
        now: DateTime<Utc>,
    ) -> Result<(), OwnerBootstrapRequestAuthorizationError> {
        verifier.verify(&self.authorization)?;
        self.authorization
            .payload
            .validate_for(&OwnerBootstrapAuthorizationContextV1 {
                installation_id: self.installation_id,
                owner,
                owner_definition_id,
                locked_input_digest: &self.locked_input_digest,
                input_digest: &self.input_digest,
                desired_revision: self.desired_revision,
                apply_sequence: self.apply_sequence,
                target_plan_digest: &self.target_plan_digest,
                idempotency_key: &self.idempotency_key,
                now,
            })?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OwnerBootstrapRequestAuthorizationError {
    #[error(transparent)]
    Envelope(#[from] ProtocolEnvelopeError),
    #[error(transparent)]
    Authorization(#[from] OwnerBootstrapAuthorizationError),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBootstrapResponseV1 {
    pub receipt: BootstrapReceiptV1,
    pub signed_receipt: SignedEnvelopeV1<BootstrapReceiptV1>,
}

impl OwnerBootstrapResponseV1 {
    pub fn signed(
        receipt: BootstrapReceiptV1,
        signer: &PurposeBoundSigningKeyV1,
    ) -> Result<Self, ProtocolEnvelopeError> {
        let signed_receipt = signer.sign(receipt.clone())?;
        Ok(Self {
            receipt,
            signed_receipt,
        })
    }

    pub fn has_exact_signed_receipt(&self) -> bool {
        self.signed_receipt.payload == self.receipt
    }
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

/// Returns an owner-local physical identity derived from a logical bootstrap
/// key. Blueprints retain only the logical key; the owner emits this UUID only
/// through its signed bootstrap read-back.
pub fn owner_resource_id(
    installation_id: Uuid,
    owner_definition_id: &str,
    resource_kind: &str,
    logical_key: &str,
) -> Uuid {
    let digest = canonical_digest(&(
        installation_id,
        owner_definition_id,
        resource_kind,
        logical_key,
        "owner-bootstrap-resource",
    ))
    .expect("owner bootstrap identity inputs are always serializable");
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedBootstrapDependencyValidationV1 {
    pub target: BootstrapDependencyValidationTargetV1,
    pub payload: Value,
}

/// Resolves the optional bootstrap validation declaration using only the
/// source-exact manifest and lockfile. The payload is opaque platform data;
/// only the functional provider interprets it.
pub fn resolve_bootstrap_dependency_validation(
    lockfile: &ApplicationLockfileV1,
    module: &ResolvedModuleReleaseV1,
    manifest: &ModuleManifest,
    resolved_input: Option<&Value>,
) -> Result<Option<ResolvedBootstrapDependencyValidationV1>, BootstrapValidationResolutionError> {
    if manifest.definition_id.as_str() != module.definition_id
        || manifest.release_version != module.version
        || canonical_digest(manifest).map_err(BootstrapValidationResolutionError::Json)?
            != module.manifest_digest
    {
        return Err(BootstrapValidationResolutionError::ManifestMismatch);
    }
    let Some(declaration) = manifest.bootstrap_dependency_validation.as_ref() else {
        return Ok(None);
    };
    let binding = module
        .dependency_bindings
        .get(declaration.dependency_binding.as_str())
        .ok_or(BootstrapValidationResolutionError::BindingMissing)?;
    if binding.contract_id != declaration.functional_contract.as_str()
        || binding.contract_version != declaration.contract_version
        || !manifest.consumed_service_actions.iter().any(|action| {
            action.dependency_binding == declaration.dependency_binding
                && action.functional_contract == declaration.functional_contract
                && action.authorization_action == declaration.authorization_action
        })
    {
        return Err(BootstrapValidationResolutionError::TargetMismatch);
    }
    let audience = match declaration.audience {
        BootstrapValidationAudienceDeclaration::ResolvedDependencyProvider => {
            if binding.provider == "core" {
                AuthorizationAudienceV1::CoreInstallation {
                    installation_id: lockfile.installation_id,
                }
            } else {
                let provider = lockfile
                    .modules
                    .iter()
                    .find(|candidate| {
                        candidate.enabled && candidate.definition_id == binding.provider
                    })
                    .ok_or(BootstrapValidationResolutionError::ProviderMissing)?;
                AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id: module_instance_id(
                        lockfile.installation_id,
                        &provider.definition_id,
                    ),
                    module_definition_id: provider
                        .definition_id
                        .parse()
                        .map_err(|_| BootstrapValidationResolutionError::ProviderInvalid)?,
                }
            }
        }
    };
    let Some(BootstrapInputV1::Inline {
        value,
        receipt_bindings,
        ..
    }) = module.bootstrap.as_ref()
    else {
        return Err(BootstrapValidationResolutionError::InlineBootstrapRequired);
    };
    let payload_source = if receipt_bindings.is_empty() {
        value
    } else {
        resolved_input.ok_or(BootstrapValidationResolutionError::ReceiptBindingsUnresolved)?
    };
    let payload = payload_source
        .pointer(&declaration.payload_pointer)
        .cloned()
        .ok_or(BootstrapValidationResolutionError::PayloadMissing)?;
    Ok(Some(ResolvedBootstrapDependencyValidationV1 {
        target: BootstrapDependencyValidationTargetV1 {
            dependency_binding: declaration.dependency_binding.as_str().into(),
            functional_contract: declaration.functional_contract.as_str().into(),
            functional_contract_version: declaration.contract_version.clone(),
            action: declaration.authorization_action.clone(),
            method: declaration.method,
            path: declaration.path.clone(),
            audience,
            payload_pointer: declaration.payload_pointer.clone(),
        },
        payload,
    }))
}

#[derive(Debug, thiserror::Error)]
pub enum BootstrapValidationResolutionError {
    #[error("bootstrap validation manifest does not match the locked release")]
    ManifestMismatch,
    #[error("bootstrap validation dependency binding is unresolved")]
    BindingMissing,
    #[error("bootstrap validation target does not match the locked dependency/action")]
    TargetMismatch,
    #[error("bootstrap validation provider is absent from the lockfile")]
    ProviderMissing,
    #[error("bootstrap validation provider identity is invalid")]
    ProviderInvalid,
    #[error("bootstrap validation requires an inline bootstrap input")]
    InlineBootstrapRequired,
    #[error("bootstrap validation receipt bindings have not been resolved")]
    ReceiptBindingsUnresolved,
    #[error("bootstrap validation payload is absent at the declared pointer")]
    PayloadMissing,
    #[error("bootstrap validation canonicalization failed: {0}")]
    Json(serde_json::Error),
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

pub fn resolve_bootstrap_receipt_bindings(
    mut input: Value,
    bindings: &[BootstrapReceiptBindingV1],
    receipts: &BTreeMap<String, BootstrapReceiptV1>,
) -> Result<Value, BootstrapBindingError> {
    let mut targets = BTreeSet::new();
    for binding in bindings {
        if !binding.target_pointer.starts_with('/') {
            return Err(BootstrapBindingError::InvalidTargetPointer {
                target_pointer: binding.target_pointer.clone(),
            });
        }
        if !targets.insert(binding.target_pointer.as_str()) {
            return Err(BootstrapBindingError::DuplicateTargetPointer {
                target_pointer: binding.target_pointer.clone(),
            });
        }
        let receipt = receipts.get(&binding.source_owner).ok_or_else(|| {
            BootstrapBindingError::ReceiptMissing {
                source_owner: binding.source_owner.clone(),
            }
        })?;
        if receipt.owner != binding.source_owner {
            return Err(BootstrapBindingError::ReceiptOwnerMismatch {
                source_owner: binding.source_owner.clone(),
                receipt_owner: receipt.owner.clone(),
            });
        }
        let resource = receipt
            .resource_ids
            .get(&binding.resource_key)
            .ok_or_else(|| BootstrapBindingError::ResourceMissing {
                source_owner: binding.source_owner.clone(),
                resource_key: binding.resource_key.clone(),
            })?;
        let resolved = match binding.value_encoding {
            BootstrapReceiptValueEncodingV1::String => Value::String(resource.clone()),
            BootstrapReceiptValueEncodingV1::Json => {
                serde_json::from_str(resource).map_err(|_| {
                    BootstrapBindingError::ResourceJsonInvalid {
                        source_owner: binding.source_owner.clone(),
                        resource_key: binding.resource_key.clone(),
                    }
                })?
            }
        };
        let target = input.pointer_mut(&binding.target_pointer).ok_or_else(|| {
            BootstrapBindingError::TargetMissing {
                target_pointer: binding.target_pointer.clone(),
            }
        })?;
        if !target.is_null() {
            return Err(BootstrapBindingError::TargetOccupied {
                target_pointer: binding.target_pointer.clone(),
            });
        }
        *target = resolved;
    }
    Ok(input)
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

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum BootstrapBindingError {
    #[error("bootstrap receipt binding target '{target_pointer}' is not an RFC 6901 JSON pointer")]
    InvalidTargetPointer { target_pointer: String },
    #[error("bootstrap receipt binding target '{target_pointer}' is declared more than once")]
    DuplicateTargetPointer { target_pointer: String },
    #[error("bootstrap receipt for prior owner '{source_owner}' is unavailable")]
    ReceiptMissing { source_owner: String },
    #[error(
        "bootstrap receipt indexed for owner '{source_owner}' declares owner '{receipt_owner}'"
    )]
    ReceiptOwnerMismatch {
        source_owner: String,
        receipt_owner: String,
    },
    #[error("bootstrap receipt for owner '{source_owner}' has no resource '{resource_key}'")]
    ResourceMissing {
        source_owner: String,
        resource_key: String,
    },
    #[error(
        "bootstrap receipt resource '{source_owner}/{resource_key}' is not valid JSON as declared"
    )]
    ResourceJsonInvalid {
        source_owner: String,
        resource_key: String,
    },
    #[error("bootstrap receipt binding target '{target_pointer}' does not exist")]
    TargetMissing { target_pointer: String },
    #[error("bootstrap receipt binding target '{target_pointer}' must be null before resolution")]
    TargetOccupied { target_pointer: String },
}

pub fn resolve(
    blueprint: &ApplicationBlueprintV1,
    catalog: &ReleaseCatalogV1,
) -> Result<ApplicationLockfileV1, CompositionError> {
    resolve_against(blueprint, catalog, None)
}

/// Resolves a Blueprint against the exact currently applied lockfile.
///
/// The first materialization intentionally emits the complete installation
/// plan. Later revisions emit only the semantic delta plus read-back. This is
/// what keeps a one-module upgrade from reconfiguring, bootstrapping, or
/// reprojecting unrelated owners merely because the Blueprint remains a
/// complete desired-state document.
pub fn resolve_against(
    blueprint: &ApplicationBlueprintV1,
    catalog: &ReleaseCatalogV1,
    current: Option<&ApplicationLockfileV1>,
) -> Result<ApplicationLockfileV1, CompositionError> {
    let mut findings = validate_blueprint(blueprint, catalog);
    if let Some(current) = current
        && current.installation_id != blueprint.installation_id
    {
        push_error(
            &mut findings,
            "current_lockfile_installation_mismatch",
            "/installation_id",
            "the current lockfile belongs to another installation",
        );
    }
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
    for selection in ordered_module_selections(blueprint, catalog) {
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
    let actions = current.map_or_else(
        || materialization_actions(&resolved_core, &modules),
        |current| delta_materialization_actions(current, &resolved_core, &modules),
    );
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
    if current.core.bootstrap_digest != desired.core.bootstrap_digest {
        changes.push("core:bootstrap".into());
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
            Some(old) if old.bootstrap_digest != module.bootstrap_digest => {
                changes.push(format!("module:bootstrap:{definition}"))
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
    validate_bootstrap_bindings(
        blueprint.core.bootstrap.as_ref(),
        "/core/bootstrap",
        None,
        &BTreeMap::new(),
        &mut findings,
    );
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
    let mut selected_bootstrap_owners = blueprint
        .modules
        .iter()
        .map(|selection| {
            (
                selection.definition_id.as_str(),
                selection.bootstrap.is_some(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    selected_bootstrap_owners.insert("core", blueprint.core.bootstrap.is_some());
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
        validate_bootstrap_bindings(
            selection.bootstrap.as_ref(),
            &format!("{base}/bootstrap"),
            Some(selection.definition_id.as_str()),
            &selected_bootstrap_owners,
            &mut findings,
        );
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

fn validate_bootstrap_bindings(
    input: Option<&BootstrapInputV1>,
    path: &str,
    consumer_owner: Option<&str>,
    selected_bootstrap_owners: &BTreeMap<&str, bool>,
    findings: &mut Vec<CompositionFindingV1>,
) {
    let Some(input) = input else {
        return;
    };
    let mut targets = BTreeSet::new();
    for (index, binding) in input.receipt_bindings().iter().enumerate() {
        let binding_path = format!("{path}/receipt_bindings/{index}");
        if !binding.target_pointer.starts_with('/') {
            push_error(
                findings,
                "bootstrap_binding_target_invalid",
                format!("{binding_path}/target_pointer"),
                "receipt binding targets must be RFC 6901 JSON pointers below the input root",
            );
        } else if !targets.insert(binding.target_pointer.as_str()) {
            push_error(
                findings,
                "bootstrap_binding_target_duplicate",
                format!("{binding_path}/target_pointer"),
                "each bootstrap input target may be bound only once",
            );
        }
        if let BootstrapInputV1::Inline { value, .. } = input
            && binding.target_pointer.starts_with('/')
        {
            match value.pointer(&binding.target_pointer) {
                None => push_error(
                    findings,
                    "bootstrap_binding_target_missing",
                    format!("{binding_path}/target_pointer"),
                    "receipt binding targets must exist in the locked bootstrap input",
                ),
                Some(target) if !target.is_null() => push_error(
                    findings,
                    "bootstrap_binding_target_occupied",
                    format!("{binding_path}/target_pointer"),
                    "receipt binding targets must be null before resolution",
                ),
                Some(_) => {}
            }
        }
        if binding.resource_key.trim().is_empty() {
            push_error(
                findings,
                "bootstrap_binding_resource_key_invalid",
                format!("{binding_path}/resource_key"),
                "receipt resource keys must be non-empty",
            );
        }
        let source_is_available = consumer_owner.is_some()
            && selected_bootstrap_owners
                .get(binding.source_owner.as_str())
                .copied()
                == Some(true);
        if !source_is_available {
            push_error(
                findings,
                "bootstrap_binding_source_unavailable",
                format!("{binding_path}/source_owner"),
                "receipt binding sources must identify Core or a selected module with bootstrap input",
            );
        } else if consumer_owner == Some(binding.source_owner.as_str()) {
            push_error(
                findings,
                "bootstrap_binding_self_reference",
                format!("{binding_path}/source_owner"),
                "an owner cannot bind its input from its own bootstrap receipt",
            );
        }
    }
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
            if let Some(provider) = selection.dependency_bindings.get(&dependency.binding_key)
                && selected.contains_key(provider.as_str())
            {
                edges
                    .entry(&selection.definition_id)
                    .or_default()
                    .push(provider);
            }
        }
        if let Some(bootstrap) = &selection.bootstrap {
            for binding in bootstrap.receipt_bindings() {
                if selected.contains_key(binding.source_owner.as_str()) {
                    edges
                        .entry(&selection.definition_id)
                        .or_default()
                        .push(binding.source_owner.as_str());
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
                "selected module dependency or bootstrap receipt bindings contain a cycle",
            );
            break;
        }
    }
}

fn ordered_module_selections<'a>(
    blueprint: &'a ApplicationBlueprintV1,
    catalog: &ReleaseCatalogV1,
) -> Vec<&'a ModuleSelectionV1> {
    let declaration_order = blueprint
        .modules
        .iter()
        .enumerate()
        .map(|(index, selection)| (selection.definition_id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let selected = blueprint
        .modules
        .iter()
        .map(|selection| (selection.definition_id.as_str(), selection))
        .collect::<BTreeMap<_, _>>();
    let mut dependents = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut prerequisites = selected
        .keys()
        .map(|definition| (*definition, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for (consumer, selection) in &selected {
        let mut providers = BTreeSet::new();
        if let Some(release) = select_module(selection, catalog) {
            for dependency in &release.dependencies {
                if let Some(provider) = selection.dependency_bindings.get(&dependency.binding_key)
                    && selected.contains_key(provider.as_str())
                {
                    providers.insert(provider.as_str());
                }
            }
        }
        if let Some(bootstrap) = &selection.bootstrap {
            providers.extend(
                bootstrap
                    .receipt_bindings()
                    .iter()
                    .map(|binding| binding.source_owner.as_str())
                    .filter(|provider| selected.contains_key(provider)),
            );
        }
        for provider in providers {
            prerequisites
                .get_mut(consumer)
                .expect("selected consumer")
                .insert(provider);
            dependents.entry(provider).or_default().insert(consumer);
        }
    }
    let mut ready = prerequisites
        .iter()
        .filter_map(|(definition, providers)| {
            providers
                .is_empty()
                .then_some((declaration_order[definition], *definition))
        })
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(selected.len());
    while let Some((_, definition)) = ready.pop_first() {
        ordered.push(selected[definition]);
        if let Some(consumers) = dependents.get(definition) {
            for consumer in consumers {
                let providers = prerequisites.get_mut(consumer).expect("selected consumer");
                providers.remove(definition);
                if providers.is_empty() {
                    ready.insert((declaration_order[consumer], consumer));
                }
            }
        }
    }
    debug_assert_eq!(
        ordered.len(),
        selected.len(),
        "validated module graph is acyclic"
    );
    ordered
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
    if let Some(input_digest) = core
        .bootstrap
        .as_ref()
        .map(BootstrapInputV1::locked_payload_digest)
    {
        actions.push(MaterializationActionV1::Bootstrap {
            owner: "core".into(),
            input_digest,
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
        if let Some(input_digest) = module
            .bootstrap
            .as_ref()
            .map(BootstrapInputV1::locked_payload_digest)
        {
            actions.push(MaterializationActionV1::Bootstrap {
                owner: module.definition_id.clone(),
                input_digest,
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

fn delta_materialization_actions(
    current: &ApplicationLockfileV1,
    desired_core: &ResolvedCoreReleaseV1,
    desired_modules: &[ResolvedModuleReleaseV1],
) -> Vec<MaterializationActionV1> {
    let mut actions = Vec::new();

    let core_release_changed = current.core.version != desired_core.version
        || current.core.core_image != desired_core.core_image
        || current.core.gateway_image != desired_core.gateway_image
        || current.core.database_image != desired_core.database_image
        || current.core.deployment_profile != desired_core.deployment_profile
        || current.core.configuration_schema_version != desired_core.configuration_schema_version;
    if current.core.database_image != desired_core.database_image {
        actions.push(MaterializationActionV1::AcquireImage {
            component: "database".into(),
            digest: desired_core.database_image.clone(),
        });
    }
    if current.core.core_image != desired_core.core_image {
        actions.push(MaterializationActionV1::AcquireImage {
            component: "core".into(),
            digest: desired_core.core_image.clone(),
        });
    }
    if current.core.gateway_image != desired_core.gateway_image {
        actions.push(MaterializationActionV1::AcquireImage {
            component: "gateway".into(),
            digest: desired_core.gateway_image.clone(),
        });
    }
    if core_release_changed {
        actions.push(MaterializationActionV1::Migrate {
            owner: "core".into(),
            image: desired_core.core_image.clone(),
        });
    }
    let core_configuration_changed =
        current.core.configuration_digest != desired_core.configuration_digest;
    if core_configuration_changed {
        actions.push(MaterializationActionV1::Configure {
            owner: "core".into(),
            digest: desired_core.configuration_digest.clone(),
        });
    }
    let core_bootstrap_changed = current.core.bootstrap_digest != desired_core.bootstrap_digest;
    if core_bootstrap_changed
        && let Some(input_digest) = desired_core
            .bootstrap
            .as_ref()
            .map(BootstrapInputV1::locked_payload_digest)
    {
        actions.push(MaterializationActionV1::Bootstrap {
            owner: "core".into(),
            input_digest,
        });
    }
    if core_release_changed || core_configuration_changed || core_bootstrap_changed {
        actions.push(MaterializationActionV1::HealthGate {
            owner: "core".into(),
        });
    }
    if core_release_changed || core_configuration_changed {
        actions.push(MaterializationActionV1::SwitchTraffic {
            owner: "core".into(),
        });
    }

    let current_modules = current
        .modules
        .iter()
        .map(|module| (module.definition_id.as_str(), module))
        .collect::<BTreeMap<_, _>>();
    let desired_definitions = desired_modules
        .iter()
        .map(|module| module.definition_id.as_str())
        .collect::<BTreeSet<_>>();

    for removed in current.modules.iter().filter(|module| {
        !desired_definitions.contains(module.definition_id.as_str()) && module.enabled
    }) {
        actions.push(MaterializationActionV1::SetEnablement {
            definition_id: removed.definition_id.clone(),
            enabled: false,
        });
    }

    for module in desired_modules {
        let current_module = current_modules.get(module.definition_id.as_str()).copied();
        let is_new = current_module.is_none();
        let release_changed = current_module.is_none_or(|old| {
            old.version != module.version
                || old.manifest_digest != module.manifest_digest
                || old.runtime_image != module.runtime_image
                || old.deployment_profile != module.deployment_profile
                || old.configuration_schema_version != module.configuration_schema_version
                || old.dependency_bindings != module.dependency_bindings
        });
        let runtime_changed = current_module
            .is_none_or(|old| old.runtime_image != module.runtime_image || release_changed);
        let configuration_changed = current_module
            .is_none_or(|old| old.configuration_digest != module.configuration_digest);
        let bootstrap_changed =
            current_module.is_none_or(|old| old.bootstrap_digest != module.bootstrap_digest);
        let enablement_changed = current_module.is_none_or(|old| old.enabled != module.enabled);

        if runtime_changed {
            actions.push(MaterializationActionV1::AcquireImage {
                component: module.definition_id.clone(),
                digest: module.runtime_image.clone(),
            });
        }
        if is_new {
            actions.push(MaterializationActionV1::ProvisionDatabase {
                owner: module.definition_id.clone(),
            });
        }
        if release_changed {
            actions.push(MaterializationActionV1::Migrate {
                owner: module.definition_id.clone(),
                image: module.runtime_image.clone(),
            });
        }
        if configuration_changed {
            actions.push(MaterializationActionV1::Configure {
                owner: module.definition_id.clone(),
                digest: module.configuration_digest.clone(),
            });
        }
        if bootstrap_changed
            && let Some(input_digest) = module
                .bootstrap
                .as_ref()
                .map(BootstrapInputV1::locked_payload_digest)
        {
            actions.push(MaterializationActionV1::Bootstrap {
                owner: module.definition_id.clone(),
                input_digest,
            });
        }
        if enablement_changed {
            actions.push(MaterializationActionV1::SetEnablement {
                definition_id: module.definition_id.clone(),
                enabled: module.enabled,
            });
        }
        if release_changed || configuration_changed || bootstrap_changed || enablement_changed {
            actions.push(MaterializationActionV1::HealthGate {
                owner: module.definition_id.clone(),
            });
        }
        if module.enabled && (release_changed || enablement_changed) {
            actions.push(MaterializationActionV1::SwitchTraffic {
                owner: module.definition_id.clone(),
            });
        }
    }

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
    fn sprint_8b_reference_blueprint_is_uuid_free_and_resolves_owner_order() {
        let blueprint: ApplicationBlueprintV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8b/blueprints/reference.json"
        )))
        .expect("Sprint 8B reference Blueprint");
        let catalog: ReleaseCatalogV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8b/catalogs/local-release-catalog.json"
        )))
        .expect("Sprint 8B release catalog");
        fn assert_no_physical_uuid(value: &Value) {
            match value {
                Value::String(value) => assert!(
                    Uuid::parse_str(value).is_err(),
                    "owner bootstrap input predicts physical UUID {value}"
                ),
                Value::Array(values) => values.iter().for_each(assert_no_physical_uuid),
                Value::Object(values) => values.values().for_each(assert_no_physical_uuid),
                _ => {}
            }
        }
        if let Some(BootstrapInputV1::Inline { value, .. }) = blueprint.core.bootstrap.as_ref() {
            assert_no_physical_uuid(value);
        }
        for module in &blueprint.modules {
            if let Some(BootstrapInputV1::Inline { value, .. }) = module.bootstrap.as_ref() {
                assert_no_physical_uuid(value);
            }
        }
        let lockfile = resolve(&blueprint, &catalog).expect("Sprint 8B reference composition");
        let bootstrap_owners = lockfile
            .materialization_plan
            .actions
            .iter()
            .filter_map(|action| match action {
                MaterializationActionV1::Bootstrap { owner, .. } => Some(owner.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            bootstrap_owners,
            [
                "core",
                "tessara.datasets",
                "tessara.components",
                "tessara.dashboards",
                "tessara.reference.scoped-records"
            ]
        );
    }

    #[test]
    fn sprint_8c_reference_blueprint_resolves_owner_order() {
        let blueprint: ApplicationBlueprintV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8c/blueprints/reference.json"
        )))
        .expect("Sprint 8C reference Blueprint");
        let catalog: ReleaseCatalogV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8c/catalogs/local-release-catalog.json"
        )))
        .expect("Sprint 8C release catalog");
        let lockfile = resolve(&blueprint, &catalog).expect("Sprint 8C reference composition");
        let bootstrap_owners = lockfile
            .materialization_plan
            .actions
            .iter()
            .filter_map(|action| match action {
                MaterializationActionV1::Bootstrap { owner, .. } => Some(owner.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            bootstrap_owners,
            [
                "core",
                "tessara.responses",
                "tessara.datasets",
                "tessara.components",
                "tessara.dashboards",
                "tessara.reference.scoped-records"
            ]
        );
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

    #[test]
    fn owner_bootstrap_resource_identity_is_stable_and_logical_key_scoped() {
        let installation_id = Uuid::from_u128(0x8b);
        let component = owner_resource_id(
            installation_id,
            "tessara.components",
            "component",
            "component.dataset-table",
        );
        assert_eq!(
            component,
            owner_resource_id(
                installation_id,
                "tessara.components",
                "component",
                "component.dataset-table",
            )
        );
        assert_ne!(
            component,
            owner_resource_id(
                installation_id,
                "tessara.components",
                "component-version",
                "component.dataset-table",
            )
        );
        assert_ne!(
            component,
            owner_resource_id(
                installation_id,
                "tessara.dashboards",
                "dashboard",
                "component.dataset-table",
            )
        );
    }

    #[test]
    fn sprint_8a_component_owner_identity_is_source_exact() {
        let installation_id = Uuid::parse_str("01980000-0000-7000-8000-00000000008a").unwrap();
        assert_eq!(
            module_instance_id(installation_id, "tessara.components"),
            Uuid::parse_str("142a1ece-f74b-85f6-8ca0-92f4a02e9409").unwrap()
        );
    }

    #[test]
    fn bootstrap_dependency_authorization_is_exact_owner_input_apply_and_time_bound() {
        let now = Utc::now();
        let installation_id = Uuid::new_v4();
        let module_instance_id = module_instance_id(installation_id, "tessara.components");
        let input_digest = digest('1');
        let plan_digest = digest('2');
        let contract_version = Version::new(1, 0, 0);
        let audience = AuthorizationAudienceV1::CoreInstallation { installation_id };
        let request_digest = digest('4');
        let authorization = BootstrapDependencyValidationAuthorizationV1 {
            schema_version: BOOTSTRAP_DEPENDENCY_VALIDATION_AUTHORIZATION_SCHEMA_VERSION_V1,
            installation_id,
            module_instance_id,
            module_definition_id: "tessara.components".into(),
            input_digest: input_digest.clone(),
            desired_revision: 8,
            apply_sequence: 3,
            target_plan_digest: plan_digest.clone(),
            dependency_binding: "tessara.components.dataset-major-line".into(),
            functional_contract: "tessara.datasets.dataset-major-line".into(),
            functional_contract_version: contract_version.clone(),
            action: "datasets.bootstrap_validate".into(),
            method: ServiceActionMethod::Post,
            path: "/api/private/datasets/bootstrap-validation".into(),
            audience: audience.clone(),
            request_digest: request_digest.clone(),
            correlation_id: Uuid::new_v4(),
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + chrono::Duration::seconds(30),
        };
        let context = BootstrapDependencyValidationContextV1 {
            installation_id,
            module_instance_id,
            module_definition_id: "tessara.components",
            input_digest: &input_digest,
            desired_revision: 8,
            apply_sequence: 3,
            target_plan_digest: &plan_digest,
            dependency_binding: "tessara.components.dataset-major-line",
            functional_contract: "tessara.datasets.dataset-major-line",
            functional_contract_version: &contract_version,
            action: "datasets.bootstrap_validate",
            method: ServiceActionMethod::Post,
            path: "/api/private/datasets/bootstrap-validation",
            audience: &audience,
            request_digest: &request_digest,
            now,
        };
        assert_eq!(authorization.validate_for(&context), Ok(()));

        let mut wrong_owner = authorization.clone();
        wrong_owner.module_instance_id = Uuid::new_v4();
        assert_eq!(
            wrong_owner.validate_for(&context),
            Err(BootstrapDependencyValidationAuthorizationError::WrongOwner)
        );
        let mut wrong_input = authorization.clone();
        wrong_input.input_digest = digest('3');
        assert_eq!(
            wrong_input.validate_for(&context),
            Err(BootstrapDependencyValidationAuthorizationError::WrongInputDigest)
        );
        let mut wrong_apply = authorization.clone();
        wrong_apply.apply_sequence += 1;
        assert_eq!(
            wrong_apply.validate_for(&context),
            Err(BootstrapDependencyValidationAuthorizationError::WrongApply)
        );
        let mut wrong_request = authorization.clone();
        wrong_request.request_digest = digest('5');
        assert_eq!(
            wrong_request.validate_for(&context),
            Err(BootstrapDependencyValidationAuthorizationError::WrongRequestDigest)
        );
        let expired_context = BootstrapDependencyValidationContextV1 {
            now: authorization.expires_at,
            ..context
        };
        assert_eq!(
            authorization.validate_for(&expired_context),
            Err(BootstrapDependencyValidationAuthorizationError::Expired)
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
                    receipt_bindings: Vec::new(),
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
    fn release_update_materializes_only_the_changed_module() {
        let current = resolve(&blueprint(), &catalog()).unwrap();
        let mut desired_blueprint = blueprint();
        desired_blueprint.revision = 2;
        desired_blueprint.modules[0].version_requirement = VersionReq::parse("=2.1.0").unwrap();
        let mut desired_catalog = catalog();
        let mut release = desired_catalog.module_releases[0].clone();
        release.version = Version::new(2, 1, 0);
        release.manifest_digest = digest('f');
        release.runtime_image = digest('9');
        desired_catalog.module_releases.push(release);

        let desired =
            resolve_against(&desired_blueprint, &desired_catalog, Some(&current)).unwrap();

        assert_eq!(
            desired.materialization_plan.actions,
            vec![
                MaterializationActionV1::AcquireImage {
                    component: "tessara.dashboards".into(),
                    digest: digest('9'),
                },
                MaterializationActionV1::Migrate {
                    owner: "tessara.dashboards".into(),
                    image: digest('9'),
                },
                MaterializationActionV1::HealthGate {
                    owner: "tessara.dashboards".into(),
                },
                MaterializationActionV1::SwitchTraffic {
                    owner: "tessara.dashboards".into(),
                },
                MaterializationActionV1::VerifyReadBack,
            ]
        );
        assert_eq!(desired.core, current.core);
    }

    #[test]
    fn unchanged_owner_state_is_not_reconfigured_or_bootstrapped() {
        let current = resolve(&blueprint(), &catalog()).unwrap();
        let mut desired_blueprint = blueprint();
        desired_blueprint.revision = 2;
        let desired = resolve_against(&desired_blueprint, &catalog(), Some(&current)).unwrap();

        assert_eq!(
            desired.materialization_plan.actions,
            vec![MaterializationActionV1::VerifyReadBack]
        );
    }

    #[test]
    fn current_lockfile_from_another_installation_is_rejected() {
        let mut current = resolve(&blueprint(), &catalog()).unwrap();
        current.installation_id = Uuid::new_v4();
        let mut desired_blueprint = blueprint();
        desired_blueprint.revision = 2;

        let error = resolve_against(&desired_blueprint, &catalog(), Some(&current)).unwrap_err();

        assert_eq!(
            error
                .findings
                .iter()
                .map(|finding| finding.code.as_str())
                .collect::<Vec<_>>(),
            vec!["current_lockfile_installation_mismatch"]
        );
    }

    #[test]
    fn bootstrap_input_change_is_visible_in_the_semantic_diff() {
        let current = resolve(&blueprint(), &catalog()).unwrap();
        let mut changed = blueprint();
        let Some(BootstrapInputV1::Inline { value, .. }) = changed.modules[0].bootstrap.as_mut()
        else {
            panic!("test Dashboard bootstrap is inline");
        };
        *value = json!({"dashboards":[{"external_key":"new-dashboard"}]});
        let desired = resolve(&changed, &catalog()).unwrap();

        assert_eq!(
            semantic_diff(Some(&current), &desired),
            vec!["module:bootstrap:tessara.dashboards"]
        );
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

    fn bootstrap_receipt(owner: &str, changed: bool) -> BootstrapReceiptV1 {
        let reference = json!({
            "reference": {
                "installation_id": "01980000-0000-7000-8000-00000000008a",
                "owner": {
                    "kind": "module_instance",
                    "installation_id": "01980000-0000-7000-8000-00000000008a",
                    "module_instance_id": "142a1ece-f74b-85f6-8ca0-92f4a02e9409"
                },
                "resource_type": "tessara.components.component_version",
                "resource_id": "01980000-0001-7000-8000-000000000001"
            }
        });
        BootstrapReceiptV1 {
            owner: owner.into(),
            schema_version: "tessara.io/component-bootstrap/v1".into(),
            input_digest: digest('1'),
            result_digest: digest('2'),
            changed,
            resource_ids: BTreeMap::from([(
                "sprint-8a-row-count".into(),
                serde_json::to_string(&reference).unwrap(),
            )]),
        }
    }

    fn component_receipt_binding() -> BootstrapReceiptBindingV1 {
        BootstrapReceiptBindingV1 {
            target_pointer: "/placements/0/component_reference".into(),
            source_owner: "tessara.components".into(),
            resource_key: "sprint-8a-row-count".into(),
            value_encoding: BootstrapReceiptValueEncodingV1::Json,
        }
    }

    #[test]
    fn bootstrap_plan_authority_binds_the_acquired_payload_not_its_wrapper() {
        let value = json!({"schema_version": "test/v1", "resource": null});
        let inline = BootstrapInputV1::Inline {
            schema_version: "test/v1".into(),
            value: value.clone(),
            receipt_bindings: vec![BootstrapReceiptBindingV1 {
                target_pointer: "/resource".into(),
                source_owner: "core".into(),
                resource_key: "resource.test".into(),
                value_encoding: BootstrapReceiptValueEncodingV1::Json,
            }],
        };
        assert_eq!(
            inline.locked_payload_digest(),
            canonical_digest(&value).expect("payload digest")
        );
        assert_ne!(
            inline.locked_payload_digest(),
            canonical_digest(&inline).expect("wrapper digest"),
            "receipt bindings stay in change detection but are not part of owner payload authority"
        );

        let cas_digest = digest('a');
        let cas = BootstrapInputV1::LocalCas {
            schema_version: "test/v1".into(),
            digest: cas_digest.clone(),
            receipt_bindings: Vec::new(),
        };
        assert_eq!(cas.locked_payload_digest(), cas_digest);
    }

    #[test]
    fn resolved_plan_keeps_wrapper_change_identity_separate_from_owner_payload_authority() {
        let blueprint: ApplicationBlueprintV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8b/blueprints/reference.json"
        )))
        .expect("Sprint 8B reference Blueprint");
        let catalog: ReleaseCatalogV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8b/catalogs/local-release-catalog.json"
        )))
        .expect("Sprint 8B release catalog");
        let bootstrap = blueprint.core.bootstrap.as_ref().expect("Core bootstrap");
        let BootstrapInputV1::Inline { value, .. } = bootstrap else {
            panic!("fixture bootstrap must be inline");
        };
        let payload_digest = canonical_digest(value).expect("payload digest");
        let wrapper_digest = canonical_digest(bootstrap).expect("wrapper digest");
        assert_ne!(payload_digest, wrapper_digest);

        let lockfile = resolve(&blueprint, &catalog).expect("resolved composition");
        assert_eq!(lockfile.core.bootstrap_digest, Some(wrapper_digest));
        assert!(lockfile.materialization_plan.actions.iter().any(|action| {
            matches!(
                action,
                MaterializationActionV1::Bootstrap { owner, input_digest }
                    if owner == "core" && input_digest == &payload_digest
            )
        }));
    }

    #[test]
    fn receipt_binding_resolves_exact_typed_json_and_is_stable_for_unchanged_replay() {
        let input = json!({"placements":[{"component_reference":null}]});
        let binding = component_receipt_binding();
        let first = resolve_bootstrap_receipt_bindings(
            input.clone(),
            std::slice::from_ref(&binding),
            &BTreeMap::from([(
                "tessara.components".into(),
                bootstrap_receipt("tessara.components", true),
            )]),
        )
        .unwrap();
        let replay = resolve_bootstrap_receipt_bindings(
            input,
            &[binding],
            &BTreeMap::from([(
                "tessara.components".into(),
                bootstrap_receipt("tessara.components", false),
            )]),
        )
        .unwrap();

        assert_eq!(first, replay);
        assert_eq!(
            canonical_digest(&first).unwrap(),
            canonical_digest(&replay).unwrap()
        );
        assert_eq!(
            first.pointer("/placements/0/component_reference/reference/resource_type"),
            Some(&json!("tessara.components.component_version"))
        );
    }

    #[test]
    fn receipt_binding_rejects_unknown_key_and_receipt_owner_mismatch() {
        let input = json!({"placements":[{"component_reference":null}]});
        let mut binding = component_receipt_binding();
        binding.resource_key = "unknown-component".into();
        assert_eq!(
            resolve_bootstrap_receipt_bindings(
                input.clone(),
                std::slice::from_ref(&binding),
                &BTreeMap::from([(
                    "tessara.components".into(),
                    bootstrap_receipt("tessara.components", true),
                )]),
            ),
            Err(BootstrapBindingError::ResourceMissing {
                source_owner: "tessara.components".into(),
                resource_key: "unknown-component".into(),
            })
        );

        binding.resource_key = "sprint-8a-row-count".into();
        assert_eq!(
            resolve_bootstrap_receipt_bindings(
                input,
                &[binding],
                &BTreeMap::from([(
                    "tessara.components".into(),
                    bootstrap_receipt("another.owner", true),
                )]),
            ),
            Err(BootstrapBindingError::ReceiptOwnerMismatch {
                source_owner: "tessara.components".into(),
                receipt_owner: "another.owner".into(),
            })
        );
    }

    #[test]
    fn receipt_binding_will_not_overwrite_a_hardcoded_input_value() {
        let input = json!({"placements":[{"component_reference":{"legacy":true}}]});
        let binding = component_receipt_binding();
        assert_eq!(
            resolve_bootstrap_receipt_bindings(
                input,
                &[binding],
                &BTreeMap::from([(
                    "tessara.components".into(),
                    bootstrap_receipt("tessara.components", true),
                )]),
            ),
            Err(BootstrapBindingError::TargetOccupied {
                target_pointer: "/placements/0/component_reference".into(),
            })
        );
    }

    #[test]
    fn sprint_8a_lockfile_orders_receipt_provider_before_consumer() {
        let blueprint: ApplicationBlueprintV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8a/blueprints/reference.json"
        )))
        .unwrap();
        let catalog: ReleaseCatalogV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8a/catalogs/local-release-catalog.json"
        )))
        .unwrap();
        let lockfile = resolve(&blueprint, &catalog).unwrap();
        let components = lockfile
            .modules
            .iter()
            .position(|module| module.definition_id == "tessara.components")
            .unwrap();
        let dashboards = lockfile
            .modules
            .iter()
            .position(|module| module.definition_id == "tessara.dashboards")
            .unwrap();

        assert!(components < dashboards);
        assert_eq!(
            lockfile.modules[dashboards]
                .bootstrap
                .as_ref()
                .unwrap()
                .receipt_bindings()
                .iter()
                .map(|binding| {
                    (
                        binding.target_pointer.as_str(),
                        binding.source_owner.as_str(),
                        binding.resource_key.as_str(),
                    )
                })
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                (
                    "/placements/0/component_reference",
                    "tessara.components",
                    "sprint-8a-row-count",
                ),
                (
                    "/placements/1/component_reference",
                    "tessara.components",
                    "sprint-8a-record-table",
                ),
                (
                    "/placements/2/component_reference",
                    "tessara.components",
                    "sprint-8a-label-bar",
                ),
                (
                    "/placements/3/component_reference",
                    "tessara.components",
                    "sprint-8a-blocked-component",
                ),
                (
                    "/placements/4/component_reference",
                    "tessara.components",
                    "sprint-8a-row-count-inactive",
                ),
                (
                    "/placements/5/component_reference",
                    "tessara.components",
                    "sprint-8a-row-count-inactive",
                ),
                (
                    "/placements/6/component_reference",
                    "tessara.components",
                    "sprint-8a-row-count-inactive",
                ),
            ])
        );
    }

    #[test]
    fn sprint_8b_bootstrap_dependency_validation_is_exact_and_opt_in() {
        let component_manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-component-module/manifest.json"))
                .unwrap();
        let dashboard_manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-dashboard-module/manifest.json"))
                .unwrap();
        let scoped_records_manifest: ModuleManifest = serde_json::from_str(include_str!(
            "../../tessara-reference-scoped-records/manifest.json"
        ))
        .unwrap();
        let mut catalog = catalog();
        catalog.module_releases = vec![
            ModuleCatalogReleaseV1 {
                definition_id: "tessara.datasets".into(),
                version: Version::new(1, 0, 0),
                manifest_digest: digest('8'),
                runtime_image: digest('7'),
                deployment_profile: "tessara-oci-v1".into(),
                configuration_schema_version: "1.0.0".into(),
                bootstrap_schema_version: Some("tessara.io/dataset-bootstrap/v1".into()),
                provided_contracts: BTreeMap::from([(
                    "tessara.datasets.dataset-major-line".into(),
                    Version::new(2, 0, 0),
                )]),
                dependencies: Vec::new(),
                feature_declarations: Vec::new(),
                contribution_schemas: BTreeMap::new(),
                configuration_schema: json!({"type":"object"}),
            },
            ModuleCatalogReleaseV1 {
                definition_id: "tessara.components".into(),
                version: component_manifest.release_version.clone(),
                manifest_digest: canonical_digest(&component_manifest).unwrap(),
                runtime_image: digest('6'),
                deployment_profile: "tessara-oci-v1".into(),
                configuration_schema_version: "1.0.0".into(),
                bootstrap_schema_version: Some("tessara.io/component-bootstrap/v1".into()),
                provided_contracts: BTreeMap::new(),
                dependencies: vec![ContractDependencyV1 {
                    binding_key: "tessara.components.dataset-major-line".into(),
                    contract_id: "tessara.datasets.dataset-major-line".into(),
                    version_requirement: VersionReq::parse("=2.0.0").unwrap(),
                    optional: false,
                }],
                feature_declarations: Vec::new(),
                contribution_schemas: BTreeMap::new(),
                configuration_schema: json!({"type":"object"}),
            },
        ];
        let mut blueprint = blueprint();
        blueprint.modules = vec![
            ModuleSelectionV1 {
                definition_id: "tessara.datasets".into(),
                version_requirement: VersionReq::parse("=1.0.0").unwrap(),
                enabled: true,
                dependency_bindings: BTreeMap::new(),
                configuration: json!({}),
                bootstrap: None,
            },
            ModuleSelectionV1 {
                definition_id: "tessara.components".into(),
                version_requirement: VersionReq::parse(&format!(
                    "={}",
                    component_manifest.release_version
                ))
                .unwrap(),
                enabled: true,
                dependency_bindings: BTreeMap::from([(
                    "tessara.components.dataset-major-line".into(),
                    "tessara.datasets".into(),
                )]),
                configuration: json!({}),
                bootstrap: Some(BootstrapInputV1::Inline {
                    schema_version: "tessara.io/component-bootstrap/v1".into(),
                    value: json!({
                        "schema_version": "tessara.io/component-bootstrap/v1",
                        "dependency_validation": {
                            "schema_version": 2,
                            "items": []
                        },
                        "components": []
                    }),
                    receipt_bindings: Vec::new(),
                }),
            },
        ];
        let lockfile = resolve(&blueprint, &catalog).unwrap();
        let component_module = lockfile
            .modules
            .iter()
            .find(|module| module.definition_id == "tessara.components")
            .unwrap();
        let component = resolve_bootstrap_dependency_validation(
            &lockfile,
            component_module,
            &component_manifest,
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            component.target,
            BootstrapDependencyValidationTargetV1 {
                dependency_binding: "tessara.components.dataset-major-line".into(),
                functional_contract: "tessara.datasets.dataset-major-line".into(),
                functional_contract_version: Version::new(2, 0, 0),
                action: "datasets.bootstrap_validate".into(),
                method: ServiceActionMethod::Post,
                path: "/api/private/datasets/bootstrap-validation".into(),
                audience: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id: module_instance_id(
                        lockfile.installation_id,
                        "tessara.datasets",
                    ),
                    module_definition_id: "tessara.datasets".parse().unwrap(),
                },
                payload_pointer: "/dependency_validation".into(),
            }
        );
        let BootstrapInputV1::Inline { value, .. } = lockfile
            .modules
            .iter()
            .find(|module| module.definition_id == "tessara.components")
            .unwrap()
            .bootstrap
            .as_ref()
            .unwrap()
        else {
            panic!("Component bootstrap must be source-exact inline data");
        };
        assert_eq!(
            component.payload,
            value.pointer("/dependency_validation").unwrap().clone()
        );
        assert!(dashboard_manifest.bootstrap_dependency_validation.is_none());
        assert!(
            scoped_records_manifest
                .bootstrap_dependency_validation
                .is_none()
        );
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
            receipt_bindings: Vec::new(),
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
