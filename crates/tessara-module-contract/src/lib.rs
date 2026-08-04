//! Framework-neutral Tessara module and transition contracts.
//!
//! This crate owns versioned discovery and identity types shared by Core,
//! future modules, deployment tooling, and machine clients. HTTP transport and
//! persistence belong elsewhere. Most importantly, an in-process transition
//! descriptor is a distinct type: it cannot carry a Module Release, Module
//! Instance, executable artifact, or provider/materialization claim.

mod dependency;
mod deployment;
mod enrollment;
pub mod grid_layout;
mod inventory;
mod protocol;

pub use dependency::{
    DependencyEvaluationFindingCode, DependencyEvaluationInput, DependencyRelationshipKind,
    FunctionalDependencyEvaluation, FunctionalProviderCandidate, FunctionalProviderCandidateOrigin,
    ResolvedFunctionalDependencyBinding, evaluate_functional_dependency,
};
pub use deployment::{
    AppliedComponentV1, AppliedModuleChangeV1, AppliedModuleV1, DeploymentActionV1,
    DeploymentChangeV1, DeploymentFindingSeverityV1, DeploymentFindingV1, DeploymentOperationV1,
    DeploymentPlanV1, DeploymentReceiptV1, DeploymentValidationError, DesiredModuleV1,
    IdentityChangeV1, ReleaseChangeV1, TessaraDeploymentV1, canonical_sha256,
};
pub use enrollment::{
    AdministratorEligibilityDecisionV1, AdministratorEligibilityError,
    AdministratorEnrollmentClaimKindV1, AdministratorEnrollmentClaimStateV1,
    CORE_ELIGIBILITY_MAX_LIFETIME_SECONDS, EnrollmentRedemptionResultV1, EnrollmentReservationV1,
    LocalOperatorAuthorizationV1, RECOVERY_OPERATOR_MAX_LIFETIME_SECONDS,
};
pub use grid_layout::{
    GridConstraints, GridLayoutError, GridMoveDirection, GridMoveRequest, GridPlacement, GridRect,
    GridResizeAxis, GridResizeRequest, GridResizeStep, GridSize, derive_row_major_positions,
    reflow_movement, resolve_move_request, resolve_resize_request, sort_row_major, validate_resize,
};
pub use inventory::{
    IndependentConfigurationV1, IndependentDefinitionV1, IndependentDiagnosticsV1,
    IndependentInstanceV1, IndependentReleaseV1,
};
pub use protocol::{
    AUTHORIZATION_GRANT_SCHEMA_VERSION_V2, AUTHORIZATION_MUTATION_MAX_LIFETIME_SECONDS,
    AUTHORIZATION_READ_MAX_LIFETIME_SECONDS, AuthorizationGrantOperationV1, AuthorizationGrantV2,
    AuthorizationValidationContextV2, AuthorizationValidationError, CapabilityScopeBindingV1,
    DelegationBasisV1, ExternalIdentityAssertionV1, MODULE_SERVICE_REQUEST_MAX_LIFETIME_SECONDS,
    ModuleServiceRequestV1, ModuleServiceRequestValidationContextV1,
    ModuleServiceRequestValidationError, NavigationProjectionV1, OriginalActorProjectionV1,
    ProtocolEnvelopeError, ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1,
    PurposeBoundVerifyingKeyV1, ResourceAuthorizationAssertionV2,
    SHELL_CONTEXT_MAX_LIFETIME_SECONDS, ShellContextV1, ShellContextValidationContextV1,
    ShellContextValidationError, ShellDocumentStateV1, ShellThemeV1, SignedEnvelopeV1,
    SignedWindowError, canonical_protocol_signing_bytes,
};

use std::{collections::BTreeSet, fmt, str::FromStr};

use semver::{Version, VersionReq};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value;
use uuid::Uuid;

/// The first supported module manifest and transition descriptor schema.
pub const CONTRACT_SCHEMA_VERSION_V1: u16 = 1;
/// Exact schema version for the policy-neutral resource observation envelope.
pub const RESOURCE_OBSERVATION_SCHEMA_VERSION_V1: u16 = 1;
pub const MODULE_MANIFEST_SCHEMA_VERSION: u16 = 3;
pub const CURRENT_CORE_RELEASE: &str = "0.1.0";
pub const CURRENT_SHELL_CONTEXT_SCHEMA: &str = "1.0.0";
pub const CURRENT_MODULE_CONTROL_PROTOCOL: &str = "1.1.0";
pub const CURRENT_MODULE_CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CURRENT_MODULE_RUNTIME_VERSION: &str = "0.2.0";
pub const CURRENT_MODULE_UI_VERSION: &str = "0.2.0";
pub const CURRENT_DESIGN_SYSTEM_ASSET_ABI: &str = "1.0.0";
pub const CURRENT_CONFORMANCE_SUITE_VERSION: &str = "1.1.0";

fn deserialize_schema_version_v1<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let schema_version = u16::deserialize(deserializer)?;
    if schema_version != CONTRACT_SCHEMA_VERSION_V1 {
        return Err(de::Error::custom(format!(
            "schema version {schema_version} is unsupported; expected {CONTRACT_SCHEMA_VERSION_V1}"
        )));
    }
    Ok(schema_version)
}

fn deserialize_manifest_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let schema_version = u16::deserialize(deserializer)?;
    if !matches!(schema_version, 2 | MODULE_MANIFEST_SCHEMA_VERSION) {
        return Err(de::Error::custom(format!(
            "module manifest schema version {schema_version} is unsupported; expected {MODULE_MANIFEST_SCHEMA_VERSION}"
        )));
    }
    Ok(schema_version)
}

macro_rules! namespaced_id {
    ($name:ident, $label:literal) => {
        #[doc = concat!("Validated, namespaced ", $label, ".")]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Parses and validates the identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
                let value = value.into();
                validate_namespaced_identifier(&value, $label)?;
                Ok(Self(value))
            }

            /// Returns the stable wire value.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdentifierError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(de::Error::custom)
            }
        }
    };
}

namespaced_id!(ModuleDefinitionId, "Module Definition identifier");
namespaced_id!(PublisherId, "publisher identifier");
namespaced_id!(FeatureId, "Feature Declaration identifier");
namespaced_id!(FunctionalContractId, "functional contract identifier");
namespaced_id!(SecurityCapabilityId, "security capability identifier");
namespaced_id!(ResourceTypeId, "resource type identifier");
namespaced_id!(SemanticRouteName, "semantic route name");
namespaced_id!(
    NavigationContributionId,
    "navigation contribution identifier"
);
namespaced_id!(DependencyBindingKey, "dependency binding key");

/// Error returned when a stable contract identifier is malformed.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum IdentifierError {
    /// An identifier was empty.
    #[error("{kind} cannot be empty")]
    Empty {
        /// Human-readable identifier kind.
        kind: &'static str,
    },
    /// An identifier did not include a namespace separator.
    #[error("{kind} '{value}' must be namespaced with '.' or ':'")]
    NotNamespaced {
        /// Human-readable identifier kind.
        kind: &'static str,
        /// Rejected value.
        value: String,
    },
    /// An identifier used unsupported casing or characters.
    #[error("{kind} '{value}' contains invalid characters or separators")]
    InvalidFormat {
        /// Human-readable identifier kind.
        kind: &'static str,
        /// Rejected value.
        value: String,
    },
}

fn validate_namespaced_identifier(value: &str, kind: &'static str) -> Result<(), IdentifierError> {
    if value.is_empty() {
        return Err(IdentifierError::Empty { kind });
    }
    if !value.contains(['.', ':']) {
        return Err(IdentifierError::NotNamespaced {
            kind,
            value: value.to_string(),
        });
    }

    if !value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
    {
        return Err(IdentifierError::InvalidFormat {
            kind,
            value: value.to_string(),
        });
    }

    let mut previous_was_separator = true;
    for character in value.chars() {
        let separator = matches!(character, '.' | ':' | '_' | '-');
        let valid = character.is_ascii_lowercase() || character.is_ascii_digit() || separator;
        if !valid || (separator && previous_was_separator) {
            return Err(IdentifierError::InvalidFormat {
                kind,
                value: value.to_string(),
            });
        }
        previous_was_separator = separator;
    }

    if previous_was_separator {
        return Err(IdentifierError::InvalidFormat {
            kind,
            value: value.to_string(),
        });
    }

    Ok(())
}

/// Immutable content digest for a release artifact.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArtifactDigest(String);

impl ArtifactDigest {
    /// Parses a lower-case `sha256:<64 hex characters>` digest.
    pub fn new(value: impl Into<String>) -> Result<Self, ArtifactDigestError> {
        let value = value.into();
        let Some(hex) = value.strip_prefix("sha256:") else {
            return Err(ArtifactDigestError(value));
        };
        if hex.len() != 64
            || !hex
                .chars()
                .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
        {
            return Err(ArtifactDigestError(value));
        }
        Ok(Self(value))
    }

    /// Returns the stable digest value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtifactDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for ArtifactDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ArtifactDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// Error returned when an artifact digest is not immutable SHA-256 syntax.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("artifact digest '{0}' must use lower-case sha256:<64 hex characters>")]
pub struct ArtifactDigestError(String);

/// Stable Application Installation identity and its observed Core runtime.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationInstallation {
    /// Installation-scoped durable identifier.
    pub id: Uuid,
    /// Current observed Core runtime, which may predate exact release provenance.
    pub core_runtime: CoreRuntimeObservation,
}

/// What Core can truthfully prove about the currently running release.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "provenance", rename_all = "snake_case", deny_unknown_fields)]
pub enum CoreRuntimeObservation {
    /// The current development runtime has no Supervisor-verified artifacts.
    DevelopmentUnresolved {
        /// Informational source/package version.
        version: Version,
    },
    /// An exact Core Release and component set was verified.
    Exact {
        /// Verified release record.
        release: CoreRelease,
    },
}

/// Exact Core Release, including the same-origin gateway component.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreRelease {
    /// Release identifier.
    pub id: Uuid,
    /// Semantic release version.
    pub version: Version,
    /// Exact Core application artifact.
    pub core_component: CoreComponentArtifact,
    /// Exact same-origin gateway artifact.
    pub gateway_component: CoreComponentArtifact,
}

/// One immutable Core Release component artifact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreComponentArtifact {
    /// Digest-pinned image or equivalent artifact.
    pub digest: ArtifactDigest,
    /// Supported platform, such as `linux`.
    pub platform: String,
    /// Supported architecture, such as `amd64`.
    pub architecture: String,
}

/// Registered Module Definition identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleDefinition {
    /// Stable definition identity across releases.
    pub id: ModuleDefinitionId,
    /// Current definition lifecycle state.
    pub state: ModuleDefinitionState,
}

/// Definition lifecycle is intentionally independent from release/instance state.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleDefinitionState {
    /// Definition is known to Core.
    Registered,
}

/// One exact Module Release and its independently evaluated state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleRelease {
    /// Release identifier.
    pub id: Uuid,
    /// Owning Module Definition.
    pub definition_id: ModuleDefinitionId,
    /// Exact version.
    pub version: Version,
    /// Manifest digest.
    pub manifest_digest: ArtifactDigest,
    /// Trust decision.
    pub trust: ReleaseTrustState,
    /// Core/platform compatibility decision.
    pub compatibility: ReleaseCompatibilityState,
}

/// Trust is not implied by compatibility.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseTrustState {
    Unknown,
    Trusted,
    Rejected,
}

/// Compatibility is not implied by trust.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseCompatibilityState {
    NotEvaluated,
    Compatible,
    Incompatible,
}

/// Installation-scoped Module Instance with independent lifecycle dimensions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleInstance {
    pub id: Uuid,
    pub installation_id: Uuid,
    pub definition_id: ModuleDefinitionId,
    pub release_id: Uuid,
    pub identity_state: InstanceIdentityState,
    pub operation_state: InstanceOperationState,
    pub data_state: InstanceDataState,
}

/// Durable identity state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceIdentityState {
    Live,
    Tombstoned,
}

/// Operational dimensions intentionally remain separate booleans.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstanceOperationState {
    pub installed: bool,
    pub deployed: bool,
    pub configured: bool,
    pub ready: bool,
    pub enabled: bool,
    pub healthy: bool,
}

/// Data retention state is independent from instance identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceDataState {
    Retained,
    Destroyed,
}

/// Discoverable feature with explicit realizing links.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureDeclaration {
    pub id: FeatureId,
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
    pub contracts: Vec<FunctionalContractId>,
    #[serde(default)]
    pub resource_types: Vec<ResourceTypeId>,
    #[serde(default)]
    pub destinations: Vec<SemanticRouteName>,
    #[serde(default)]
    pub capabilities: Vec<SecurityCapabilityId>,
    /// JSON Pointers into the module configuration schema used by this feature.
    #[serde(default)]
    pub configuration_pointers: Vec<String>,
}

/// Kind of a versioned functional contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionalContractKind {
    Api,
    Event,
    Resource,
    Behavior,
}

/// Functional behavior provided by a release or transition contribution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionalContractDeclaration {
    pub id: FunctionalContractId,
    pub version: Version,
    pub kind: FunctionalContractKind,
    pub description: String,
}

/// Required functional contract and provider-binding constraint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionalDependency {
    pub contract_id: FunctionalContractId,
    pub version_requirement: VersionReq,
    pub binding_key: DependencyBindingKey,
    pub optional: bool,
}

/// Security capability advertised to Core-owned RBAC.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityCapabilityDeclaration {
    pub id: SecurityCapabilityId,
    pub description: String,
}

/// Resource type owned by Core or a future module instance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceTypeDeclaration {
    pub id: ResourceTypeId,
    pub description: String,
}

/// Product, administration, configuration, or diagnostic route declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteKind {
    Product,
    Administration,
    Configuration,
    Diagnostics,
}

/// Stable named route without a deployment URL.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteDeclaration {
    pub name: SemanticRouteName,
    pub kind: RouteKind,
    #[serde(default)]
    pub parameters: Vec<RouteParameterDeclaration>,
}

/// Closed value types supported by semantic route parameters.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteParameterType {
    String,
    Integer,
    Boolean,
    Uuid,
}

/// One named parameter accepted by a semantic route.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteParameterDeclaration {
    pub name: String,
    pub value_type: RouteParameterType,
    pub required: bool,
}

/// Navigation metadata whose administrator policy is stored separately.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationContribution {
    pub id: NavigationContributionId,
    pub destination: SemanticRouteName,
    pub label: String,
    pub group: String,
    pub order_hint: i32,
    /// Product capabilities for which this destination is useful. The shell
    /// may display the contribution when the actor has any one of them.
    pub required_capabilities_any_of: Vec<SecurityCapabilityId>,
}

/// Runtime semantic destination after binding to an owner.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticDestination {
    pub owner: ResourceOwner,
    pub route: SemanticRouteName,
    #[serde(default)]
    pub parameters: std::collections::BTreeMap<String, SemanticParameterValue>,
}

/// Typed value supplied to a semantic route.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SemanticParameterValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Uuid(Uuid),
}

impl SemanticParameterValue {
    /// Returns the declaration type represented by this wire value.
    pub const fn value_type(&self) -> RouteParameterType {
        match self {
            Self::String(_) => RouteParameterType::String,
            Self::Integer(_) => RouteParameterType::Integer,
            Self::Boolean(_) => RouteParameterType::Boolean,
            Self::Uuid(_) => RouteParameterType::Uuid,
        }
    }
}

impl SemanticDestination {
    /// Binds this destination to a registered route declaration and rejects
    /// route, name, presence, or parameter-type mismatches.
    pub fn validate_against(
        &self,
        route: &RouteDeclaration,
    ) -> Result<(), SemanticDestinationValidationError> {
        if self.route != route.name {
            return Err(SemanticDestinationValidationError::RouteMismatch);
        }

        let mut declared = std::collections::BTreeMap::new();
        for parameter in &route.parameters {
            if declared
                .insert(parameter.name.as_str(), parameter)
                .is_some()
            {
                return Err(SemanticDestinationValidationError::DuplicateRouteParameter(
                    parameter.name.clone(),
                ));
            }
            match self.parameters.get(&parameter.name) {
                None if parameter.required => {
                    return Err(
                        SemanticDestinationValidationError::MissingRequiredParameter(
                            parameter.name.clone(),
                        ),
                    );
                }
                Some(value) if value.value_type() != parameter.value_type => {
                    return Err(SemanticDestinationValidationError::ParameterTypeMismatch(
                        parameter.name.clone(),
                    ));
                }
                None | Some(_) => {}
            }
        }
        if let Some(name) = self
            .parameters
            .keys()
            .find(|name| !declared.contains_key(name.as_str()))
        {
            return Err(SemanticDestinationValidationError::UnknownParameter(
                name.clone(),
            ));
        }
        Ok(())
    }
}

/// Semantic destination does not conform to its registered named route.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SemanticDestinationValidationError {
    #[error("semantic destination route does not match the route declaration")]
    RouteMismatch,
    #[error("route declares parameter '{0}' more than once")]
    DuplicateRouteParameter(String),
    #[error("semantic destination is missing required parameter '{0}'")]
    MissingRequiredParameter(String),
    #[error("semantic destination contains unknown parameter '{0}'")]
    UnknownParameter(String),
    #[error("semantic destination parameter '{0}' has the wrong value type")]
    ParameterTypeMismatch(String),
}

/// Tagged authoritative owner for routes and typed resources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResourceOwner {
    CoreInstallation {
        installation_id: Uuid,
    },
    ModuleInstance {
        installation_id: Uuid,
        module_instance_id: Uuid,
    },
}

/// Installation-scoped reference that never grants authority by itself.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TypedResourceReference {
    installation_id: Uuid,
    owner: ResourceOwner,
    resource_type: ResourceTypeId,
    resource_id: String,
}

impl TypedResourceReference {
    /// Constructs a structurally valid typed reference.
    pub fn new(
        installation_id: Uuid,
        owner: ResourceOwner,
        resource_type: ResourceTypeId,
        resource_id: impl Into<String>,
    ) -> Result<Self, ReferenceValidationError> {
        let reference = Self {
            installation_id,
            owner,
            resource_type,
            resource_id: resource_id.into(),
        };
        reference.validate()?;
        Ok(reference)
    }

    /// Returns the installation to which the reference is scoped.
    pub const fn installation_id(&self) -> Uuid {
        self.installation_id
    }

    /// Returns the authoritative owner identity carried by the reference.
    pub const fn owner(&self) -> &ResourceOwner {
        &self.owner
    }

    /// Returns the declared resource type.
    pub const fn resource_type(&self) -> &ResourceTypeId {
        &self.resource_type
    }

    /// Returns the owner's opaque resource identifier.
    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }

    /// Validates owner/install consistency and required opaque identity.
    pub fn validate(&self) -> Result<(), ReferenceValidationError> {
        if self.resource_id.trim().is_empty() {
            return Err(ReferenceValidationError::EmptyResourceId);
        }
        let owner_installation_id = match self.owner {
            ResourceOwner::CoreInstallation { installation_id }
            | ResourceOwner::ModuleInstance {
                installation_id, ..
            } => installation_id,
        };
        if owner_installation_id != self.installation_id {
            return Err(ReferenceValidationError::InstallationMismatch);
        }
        Ok(())
    }

    /// Confirms a module-owned reference against Core's authoritative Module
    /// Instance record before a resolver exposes the referenced resource.
    pub fn validate_module_instance_binding(
        &self,
        instance: &ModuleInstance,
    ) -> Result<(), ReferenceValidationError> {
        self.validate()?;
        let ResourceOwner::ModuleInstance {
            installation_id,
            module_instance_id,
        } = &self.owner
        else {
            return Err(ReferenceValidationError::ExpectedModuleInstanceOwner);
        };
        if *module_instance_id != instance.id {
            return Err(ReferenceValidationError::ModuleInstanceMismatch);
        }
        if *installation_id != instance.installation_id
            || self.installation_id != instance.installation_id
        {
            return Err(ReferenceValidationError::InstallationMismatch);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TypedResourceReferenceWire {
    installation_id: Uuid,
    owner: ResourceOwner,
    resource_type: ResourceTypeId,
    resource_id: String,
}

impl<'de> Deserialize<'de> for TypedResourceReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TypedResourceReferenceWire::deserialize(deserializer)?;
        Self::new(
            wire.installation_id,
            wire.owner,
            wire.resource_type,
            wire.resource_id,
        )
        .map_err(de::Error::custom)
    }
}

/// Exact provider contract identity used to make an observation.
///
/// The version is concrete rather than a requirement because an observation
/// records what was actually resolved, not what a consumer might accept.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderContractIdentity {
    contract_id: FunctionalContractId,
    contract_version: Version,
}

impl ProviderContractIdentity {
    pub const fn new(contract_id: FunctionalContractId, contract_version: Version) -> Self {
        Self {
            contract_id,
            contract_version,
        }
    }

    pub const fn contract_id(&self) -> &FunctionalContractId {
        &self.contract_id
    }

    pub const fn contract_version(&self) -> &Version {
        &self.contract_version
    }
}

/// Provider-declared mechanism by which a consumer observes resource change.
///
/// Sprint 7B deliberately promises no event delivery or scheduled polling.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceObservationStrategy {
    LiveResolutionWithRevision,
}

/// Non-zero monotonic marker in one provider resource's revision domain.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ResourceRevision(u64);

impl ResourceRevision {
    pub fn new(value: u64) -> Result<Self, ResourceRevisionError> {
        if value == 0 {
            return Err(ResourceRevisionError::Zero);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn is_later_than(self, previous: Self) -> bool {
        self.0 > previous.0
    }
}

impl<'de> Deserialize<'de> for ResourceRevision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u64::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResourceRevisionError {
    #[error("resource revision must be greater than zero")]
    Zero,
}

/// Policy-neutral record of one exact live resource resolution.
///
/// Provider lifecycle meaning and consumer findings/actions intentionally do
/// not belong in this envelope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResourceObservationV1 {
    schema_version: u16,
    reference: TypedResourceReference,
    provider_contract: ProviderContractIdentity,
    strategy: ResourceObservationStrategy,
    resource_revision: ResourceRevision,
}

impl ResourceObservationV1 {
    pub fn new(
        reference: TypedResourceReference,
        provider_contract: ProviderContractIdentity,
        strategy: ResourceObservationStrategy,
        resource_revision: ResourceRevision,
    ) -> Self {
        Self {
            schema_version: RESOURCE_OBSERVATION_SCHEMA_VERSION_V1,
            reference,
            provider_contract,
            strategy,
            resource_revision,
        }
    }

    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub const fn reference(&self) -> &TypedResourceReference {
        &self.reference
    }

    pub const fn provider_contract(&self) -> &ProviderContractIdentity {
        &self.provider_contract
    }

    pub const fn strategy(&self) -> ResourceObservationStrategy {
        self.strategy
    }

    pub const fn resource_revision(&self) -> ResourceRevision {
        self.resource_revision
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceObservationV1Wire {
    schema_version: u16,
    reference: TypedResourceReference,
    provider_contract: ProviderContractIdentity,
    strategy: ResourceObservationStrategy,
    resource_revision: ResourceRevision,
}

impl<'de> Deserialize<'de> for ResourceObservationV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ResourceObservationV1Wire::deserialize(deserializer)?;
        if wire.schema_version != RESOURCE_OBSERVATION_SCHEMA_VERSION_V1 {
            return Err(de::Error::custom(format!(
                "resource observation schema version {} is unsupported; expected {}",
                wire.schema_version, RESOURCE_OBSERVATION_SCHEMA_VERSION_V1
            )));
        }
        Ok(Self::new(
            wire.reference,
            wire.provider_contract,
            wire.strategy,
            wire.resource_revision,
        ))
    }
}

/// Typed-reference validation error.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ReferenceValidationError {
    #[error("resource identifier cannot be empty")]
    EmptyResourceId,
    #[error("owner installation does not match reference installation")]
    InstallationMismatch,
    #[error("reference owner is not a Module Instance")]
    ExpectedModuleInstanceOwner,
    #[error("reference owner does not match the authoritative Module Instance")]
    ModuleInstanceMismatch,
}

/// Authorization outcome for a resource-resolution request.
///
/// `NotEvaluated` fails closed and must never permit the requested operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAccessState {
    Authorized,
    Unauthorized,
    NotEvaluated,
}

/// Resolution state for a Core installation named as a resource owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreInstallationOwnerState {
    Live,
    UnknownCoreInstallation,
    InstallationMismatch,
}

/// Resolution state for a Module Instance named as a resource owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleInstanceOwnerState {
    Live,
    OwnerModuleInstanceTombstoned,
    UnknownModuleInstance,
    OwnerMismatch,
}

/// Data-retention state for a Module Instance resource owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerDataState {
    Retained,
    OwnerDataDestroyed,
    Unknown,
    NotEvaluated,
}

/// Tagged authoritative-owner resolution state.
///
/// Core-owned references do not carry Module Instance data state. Module-owned
/// references keep owner identity and owner data as independent dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResourceOwnerState {
    CoreInstallation {
        state: CoreInstallationOwnerState,
    },
    ModuleInstance {
        instance_state: ModuleInstanceOwnerState,
        data_state: OwnerDataState,
    },
    Undisclosed,
    NotEvaluated,
}

/// Resolution state of the provider-owned resource identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceIdentityState {
    Resolved,
    UnknownResource,
    Undisclosed,
    NotEvaluated,
}

/// Provider-defined product-resource lifecycle state.
///
/// Evaluated values such as `active`, `archived`, or `tombstoned` remain
/// provider-owned policy rather than becoming platform lifecycle policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResourceLifecycleState {
    ProviderDefined { state: String },
    Undisclosed,
    NotEvaluated,
}

/// Compatibility of the provider contract required to resolve the resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractCompatibilityState {
    Compatible,
    Incompatible,
    Undisclosed,
    NotEvaluated,
}

/// Runtime availability of the resource provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAvailabilityState {
    Available,
    Unavailable,
    Undisclosed,
    NotEvaluated,
}

/// Version-one multi-dimensional resource-resolution envelope.
///
/// The fields are private so callers cannot construct an unauthorized response
/// that discloses resource-specific state. Use [`Self::authorized`] for an
/// authorized result or [`Self::restricted`] for a fail-closed result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ResourceResolutionV1Wire")]
pub struct ResourceResolutionV1 {
    schema_version: u16,
    access_state: ResourceAccessState,
    owner_state: ResourceOwnerState,
    resource_identity_state: ResourceIdentityState,
    resource_lifecycle_state: ResourceLifecycleState,
    compatibility_state: ContractCompatibilityState,
    availability_state: ProviderAvailabilityState,
}

impl ResourceResolutionV1 {
    /// Constructs a resource-resolution result after access is authorized.
    pub fn authorized(
        owner_state: ResourceOwnerState,
        resource_identity_state: ResourceIdentityState,
        resource_lifecycle_state: ResourceLifecycleState,
        compatibility_state: ContractCompatibilityState,
        availability_state: ProviderAvailabilityState,
    ) -> Result<Self, ResourceResolutionValidationError> {
        let resolution = Self {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
            access_state: ResourceAccessState::Authorized,
            owner_state,
            resource_identity_state,
            resource_lifecycle_state,
            compatibility_state,
            availability_state,
        };
        resolution.validate()?;
        Ok(resolution)
    }

    /// Constructs the stable non-disclosing envelope for a failed-closed
    /// authorization result.
    pub fn restricted(
        access_state: ResourceAccessState,
    ) -> Result<Self, ResourceResolutionValidationError> {
        if access_state == ResourceAccessState::Authorized {
            return Err(ResourceResolutionValidationError::AuthorizedRestrictedProjection);
        }

        Ok(Self {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
            access_state,
            owner_state: ResourceOwnerState::Undisclosed,
            resource_identity_state: ResourceIdentityState::Undisclosed,
            resource_lifecycle_state: ResourceLifecycleState::Undisclosed,
            compatibility_state: ContractCompatibilityState::Undisclosed,
            availability_state: ProviderAvailabilityState::Undisclosed,
        })
    }

    /// Reprojects an envelope to the stable non-disclosing shape.
    ///
    /// Providers must still evaluate authorization before resource-specific
    /// resolution; this helper is a serialization safeguard, not permission to
    /// resolve details before checking access.
    pub fn restricted_projection(
        &self,
        access_state: ResourceAccessState,
    ) -> Result<Self, ResourceResolutionValidationError> {
        Self::restricted(access_state)
    }

    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub const fn access_state(&self) -> ResourceAccessState {
        self.access_state
    }

    pub const fn owner_state(&self) -> ResourceOwnerState {
        self.owner_state
    }

    pub const fn resource_identity_state(&self) -> ResourceIdentityState {
        self.resource_identity_state
    }

    pub const fn resource_lifecycle_state(&self) -> &ResourceLifecycleState {
        &self.resource_lifecycle_state
    }

    pub const fn compatibility_state(&self) -> ContractCompatibilityState {
        self.compatibility_state
    }

    pub const fn availability_state(&self) -> ProviderAvailabilityState {
        self.availability_state
    }

    fn validate(&self) -> Result<(), ResourceResolutionValidationError> {
        if self.schema_version != CONTRACT_SCHEMA_VERSION_V1 {
            return Err(
                ResourceResolutionValidationError::UnsupportedSchemaVersion {
                    expected: CONTRACT_SCHEMA_VERSION_V1,
                    actual: self.schema_version,
                },
            );
        }

        if self.access_state != ResourceAccessState::Authorized
            && (self.owner_state != ResourceOwnerState::Undisclosed
                || self.resource_identity_state != ResourceIdentityState::Undisclosed
                || self.resource_lifecycle_state != ResourceLifecycleState::Undisclosed
                || self.compatibility_state != ContractCompatibilityState::Undisclosed
                || self.availability_state != ProviderAvailabilityState::Undisclosed)
        {
            return Err(ResourceResolutionValidationError::RestrictedEnvelopeDisclosesState);
        }

        if self.access_state == ResourceAccessState::Authorized
            && (self.owner_state == ResourceOwnerState::Undisclosed
                || self.resource_identity_state == ResourceIdentityState::Undisclosed
                || self.resource_lifecycle_state == ResourceLifecycleState::Undisclosed
                || self.compatibility_state == ContractCompatibilityState::Undisclosed
                || self.availability_state == ProviderAvailabilityState::Undisclosed)
        {
            return Err(ResourceResolutionValidationError::AuthorizedEnvelopeUndisclosedState);
        }

        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceResolutionV1Wire {
    schema_version: u16,
    access_state: ResourceAccessState,
    owner_state: ResourceOwnerState,
    resource_identity_state: ResourceIdentityState,
    resource_lifecycle_state: ResourceLifecycleState,
    compatibility_state: ContractCompatibilityState,
    availability_state: ProviderAvailabilityState,
}

impl TryFrom<ResourceResolutionV1Wire> for ResourceResolutionV1 {
    type Error = ResourceResolutionValidationError;

    fn try_from(wire: ResourceResolutionV1Wire) -> Result<Self, Self::Error> {
        let resolution = Self {
            schema_version: wire.schema_version,
            access_state: wire.access_state,
            owner_state: wire.owner_state,
            resource_identity_state: wire.resource_identity_state,
            resource_lifecycle_state: wire.resource_lifecycle_state,
            compatibility_state: wire.compatibility_state,
            availability_state: wire.availability_state,
        };
        resolution.validate()?;
        Ok(resolution)
    }
}

/// Resource-resolution envelope validation error.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResourceResolutionValidationError {
    #[error("resource-resolution schema version {actual} is unsupported; expected {expected}")]
    UnsupportedSchemaVersion { expected: u16, actual: u16 },
    #[error("authorized access cannot use the restricted resource-resolution projection")]
    AuthorizedRestrictedProjection,
    #[error("a restricted resource-resolution envelope must not disclose resolution state")]
    RestrictedEnvelopeDisclosesState,
    #[error("an authorized resource-resolution envelope must not contain undisclosed state")]
    AuthorizedEnvelopeUndisclosedState,
}

/// Digest-pinned OCI image location and command.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OciImageDeclaration {
    /// Non-URL OCI repository reference pinned to `digest` with `@`.
    pub image_reference: String,
    pub digest: ArtifactDigest,
    pub platform: String,
    pub architecture: String,
    pub command: Vec<String>,
}

/// V1 service listen declaration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceListenDeclaration {
    pub protocol: String,
    pub port: u16,
    pub registration_name: String,
}

/// Runtime resource requests/limits.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResourceDeclaration {
    pub cpu_request_millis: u32,
    pub cpu_limit_millis: u32,
    pub memory_request_mebibytes: u32,
    pub memory_limit_mebibytes: u32,
}

/// Sole executable deployment profile supported by v1.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TessaraOciV1 {
    pub runtime_image: OciImageDeclaration,
    pub migration_image: Option<OciImageDeclaration>,
    pub listen: ServiceListenDeclaration,
    #[serde(default)]
    pub configuration_keys: Vec<String>,
    #[serde(default)]
    pub secret_keys: Vec<String>,
    pub runtime_identity: String,
    pub migration_identity: String,
    pub readiness_path: String,
    pub liveness_path: String,
    pub graceful_shutdown_seconds: u32,
    pub resources: RuntimeResourceDeclaration,
}

/// Explicitly tagged deployment profile. Unknown profile versions fail during
/// deserialization rather than being interpreted as the current profile.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "profile", content = "declaration", deny_unknown_fields)]
pub enum DeploymentProfile {
    #[serde(rename = "tessara-oci-v1")]
    TessaraOciV1(TessaraOciV1),
}

/// Publisher-owned support and documentation metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleSupportDeclaration {
    pub support_tier: String,
    pub contact: String,
    pub documentation: String,
}

/// Closed, version-stable schema for the opaque identifier portion of a typed
/// resource reference. This avoids accepting arbitrary, unvalidated JSON
/// Schema dialects at the Core trust boundary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResourceIdentifierSchema {
    /// An opaque non-empty string with explicit wire length bounds.
    OpaqueString { min_length: u32, max_length: u32 },
    /// Canonical hyphenated UUID text.
    Uuid,
}

impl ResourceIdentifierSchema {
    /// Validates the schema declaration independently of any instance value.
    pub fn validate_declaration(&self) -> Result<(), ResourceIdentifierValidationError> {
        match self {
            Self::OpaqueString {
                min_length,
                max_length,
            } if *min_length == 0 || max_length < min_length => {
                Err(ResourceIdentifierValidationError::InvalidLengthBounds)
            }
            Self::OpaqueString { .. } | Self::Uuid => Ok(()),
        }
    }

    /// Validates a resource identifier against this declared schema.
    pub fn validate(&self, resource_id: &str) -> Result<(), ResourceIdentifierValidationError> {
        self.validate_declaration()?;
        match self {
            Self::OpaqueString {
                min_length,
                max_length,
            } => {
                let length = resource_id.chars().count() as u64;
                if length < u64::from(*min_length) || length > u64::from(*max_length) {
                    return Err(ResourceIdentifierValidationError::LengthOutOfBounds);
                }
                Ok(())
            }
            Self::Uuid => {
                let parsed = Uuid::parse_str(resource_id)
                    .map_err(|_| ResourceIdentifierValidationError::InvalidUuid)?;
                if parsed.hyphenated().to_string() == resource_id {
                    Ok(())
                } else {
                    Err(ResourceIdentifierValidationError::InvalidUuid)
                }
            }
        }
    }
}

/// Resource identifier does not conform to its owning resource declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResourceIdentifierValidationError {
    #[error("opaque resource identifier length bounds are invalid")]
    InvalidLengthBounds,
    #[error("resource identifier length is outside the declared bounds")]
    LengthOutOfBounds,
    #[error("resource identifier is not canonical UUID text")]
    InvalidUuid,
}

/// Reference schema for exactly one declared resource type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedReferenceSchemaDeclaration {
    pub resource_type: ResourceTypeId,
    pub resource_id: ResourceIdentifierSchema,
}

impl TypedReferenceSchemaDeclaration {
    /// Validates both the resource type and opaque identifier of a reference.
    pub fn validate_reference(
        &self,
        reference: &TypedResourceReference,
    ) -> Result<(), TypedReferenceSchemaValidationError> {
        if self.resource_type != *reference.resource_type() {
            return Err(TypedReferenceSchemaValidationError::ResourceTypeMismatch);
        }
        self.resource_id
            .validate(reference.resource_id())
            .map_err(TypedReferenceSchemaValidationError::InvalidResourceId)
    }
}

/// A typed reference does not conform to a resource declaration's schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TypedReferenceSchemaValidationError {
    #[error("typed reference resource type does not match the schema declaration")]
    ResourceTypeMismatch,
    #[error(transparent)]
    InvalidResourceId(#[from] ResourceIdentifierValidationError),
}

/// Semantic routes used by Core to validate and observe a module instance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleOperationalRoutes {
    pub configuration_validation: SemanticRouteName,
    pub compatibility: SemanticRouteName,
    pub status: SemanticRouteName,
    pub diagnostics: SemanticRouteName,
}

/// Optional functional contracts through which a module contributes to Core
/// Home, work discovery, or global search surfaces.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShellContributionContracts {
    pub home: Option<FunctionalContractId>,
    pub work_discovery: Option<FunctionalContractId>,
    pub search: Option<FunctionalContractId>,
}

const CORE_RESERVED_NAMESPACE_PREFIXES: [&str; 3] = ["admin", "core", "tessara.core"];

/// Trusted registry context used to decide which identifiers a manifest may
/// advertise. A manifest cannot grant itself additional namespaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestNamespaceAuthority {
    definition_id: ModuleDefinitionId,
    publisher: PublisherId,
    allowed_prefixes: BTreeSet<String>,
}

impl ManifestNamespaceAuthority {
    /// Creates a registry authority after validating every granted prefix and
    /// excluding Core-reserved authority namespaces.
    pub fn new(
        definition_id: ModuleDefinitionId,
        publisher: PublisherId,
        allowed_prefixes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, NamespaceAuthorityError> {
        let mut validated = BTreeSet::new();
        for prefix in allowed_prefixes {
            let prefix = prefix.into();
            validate_namespace_prefix(&prefix)?;
            if CORE_RESERVED_NAMESPACE_PREFIXES.iter().any(|reserved| {
                namespace_matches(&prefix, reserved) || namespace_matches(reserved, &prefix)
            }) {
                return Err(NamespaceAuthorityError::Reserved(prefix));
            }
            validated.insert(prefix);
        }
        if validated.is_empty() {
            return Err(NamespaceAuthorityError::Empty);
        }
        Ok(Self {
            definition_id,
            publisher,
            allowed_prefixes: validated,
        })
    }

    /// Returns the registered Definition identity.
    pub const fn definition_id(&self) -> &ModuleDefinitionId {
        &self.definition_id
    }

    /// Returns the registered publisher identity.
    pub const fn publisher(&self) -> &PublisherId {
        &self.publisher
    }

    fn allows(&self, identifier: &str) -> bool {
        !CORE_RESERVED_NAMESPACE_PREFIXES
            .iter()
            .any(|reserved| namespace_matches(identifier, reserved))
            && self
                .allowed_prefixes
                .iter()
                .any(|prefix| namespace_matches(identifier, prefix))
    }
}

/// Invalid trusted-registry namespace grant.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NamespaceAuthorityError {
    #[error("at least one manifest namespace prefix is required")]
    Empty,
    #[error("namespace prefix '{0}' contains invalid characters or separators")]
    Invalid(String),
    #[error("namespace prefix '{0}' overlaps a Core-reserved namespace")]
    Reserved(String),
}

fn validate_namespace_prefix(prefix: &str) -> Result<(), NamespaceAuthorityError> {
    let mut characters = prefix.chars();
    if !characters
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
    {
        return Err(NamespaceAuthorityError::Invalid(prefix.to_string()));
    }

    let mut previous_was_separator = false;
    for character in prefix.chars() {
        let separator = matches!(character, '.' | ':' | '_' | '-');
        let valid = character.is_ascii_lowercase() || character.is_ascii_digit() || separator;
        if !valid || (separator && previous_was_separator) {
            return Err(NamespaceAuthorityError::Invalid(prefix.to_string()));
        }
        previous_was_separator = separator;
    }
    if previous_was_separator {
        return Err(NamespaceAuthorityError::Invalid(prefix.to_string()));
    }
    Ok(())
}

fn namespace_matches(identifier: &str, prefix: &str) -> bool {
    identifier == prefix
        || identifier
            .strip_prefix(prefix)
            .is_some_and(|remainder| remainder.starts_with(['.', ':']))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModulePlatformVersions {
    pub core_release: Version,
    pub shell_context_schema: Version,
    pub module_control_protocol: Version,
    pub module_contract: Version,
    pub module_runtime: Version,
    pub module_ui: Version,
    pub design_system_asset_abi: Version,
    pub conformance_suite: Version,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkedModulePackages {
    pub module_contract: Version,
    pub module_runtime: Option<Version>,
    pub module_ui: Option<Version>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BrowserDocumentMethod {
    Get,
    Head,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserRouteDeclaration {
    pub destination: SemanticRouteName,
    pub path_template: String,
    pub methods: Vec<BrowserDocumentMethod>,
    pub required_capability: SecurityCapabilityId,
    pub authorization_action: String,
    #[serde(default = "legacy_browser_dependency_binding")]
    pub dependency_binding: DependencyBindingKey,
    pub functional_contract: FunctionalContractId,
    pub organization_scope_parameter: Option<String>,
}

/// Framework-neutral browser lifecycle implemented by an interactive module.
///
/// Core imports `entry_asset` as an ECMAScript module and calls its exported
/// `createModule(host)` factory. Asset paths are module-local manifest paths;
/// Core binds them to the installed release and immutable digest at runtime.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserLifecycleDeclaration {
    pub lifecycle_abi: Version,
    pub entry_asset: String,
    #[serde(default)]
    pub stylesheet_assets: Vec<String>,
    /// Whether the declared browser routes also return complete HTML documents
    /// for direct loads, no-JavaScript clients, and compatibility recovery.
    pub complete_document_fallback: bool,
    #[serde(default)]
    pub capabilities: BrowserLifecycleCapabilities,
}

/// Optional lifecycle operations supported by the module instance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserLifecycleCapabilities {
    #[serde(default)]
    pub navigation_guard: bool,
    #[serde(default)]
    pub suspend_resume: bool,
}

/// Immutable, same-origin asset projected to the browser lifecycle host.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserLifecycleAssetV1 {
    pub url: String,
    pub digest: ArtifactDigest,
    pub content_type: String,
}

/// Authorization-filtered route state returned when Core requests a module
/// browser route with the lifecycle-v1 media type. `payload` remains opaque to
/// Core and is interpreted only by the owning module runtime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserLifecycleBootstrapV1 {
    pub schema_version: u16,
    pub definition_id: ModuleDefinitionId,
    pub release_version: Version,
    pub lifecycle_abi: Version,
    pub destination: SemanticRouteName,
    pub path: String,
    pub title: String,
    pub document_state: ShellDocumentStateV1,
    pub entry_asset: BrowserLifecycleAssetV1,
    #[serde(default)]
    pub stylesheet_assets: Vec<BrowserLifecycleAssetV1>,
    pub payload: Value,
}

impl BrowserLifecycleBootstrapV1 {
    pub const SCHEMA_VERSION: u16 = 1;

    /// Fail-closed checks that can be performed by a framework-neutral host.
    pub fn is_supported(&self) -> bool {
        self.schema_version == Self::SCHEMA_VERSION
            && self.lifecycle_abi == Version::new(1, 0, 0)
            && valid_same_origin_path(&self.path)
            && valid_lifecycle_asset(
                &self.entry_asset,
                &self.definition_id,
                &self.release_version,
                "text/javascript",
            )
            && self.stylesheet_assets.iter().all(|asset| {
                valid_lifecycle_asset(
                    asset,
                    &self.definition_id,
                    &self.release_version,
                    "text/css",
                )
            })
            && !self.title.trim().is_empty()
    }
}

fn valid_lifecycle_asset(
    asset: &BrowserLifecycleAssetV1,
    definition_id: &ModuleDefinitionId,
    release_version: &Version,
    media_prefix: &str,
) -> bool {
    valid_same_origin_path(&asset.url)
        && asset.url.starts_with(&format!(
            "/_tessara/modules/{definition_id}/{release_version}/"
        ))
        && asset.url.contains(&format!("/{}/", asset.digest.as_str()))
        && asset.content_type.starts_with(media_prefix)
}

fn legacy_browser_dependency_binding() -> DependencyBindingKey {
    DependencyBindingKey::new("tessara.core.legacy-module-document")
        .expect("static legacy dependency binding is valid")
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PublicApiMethod {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicApiRouteDeclaration {
    pub path_template: String,
    pub method: PublicApiMethod,
    pub required_capability: SecurityCapabilityId,
    pub authorization_action: String,
    pub dependency_binding: DependencyBindingKey,
    pub operation: AuthorizationGrantOperationV1,
    pub functional_contract: FunctionalContractId,
    #[serde(default)]
    pub idempotency: PublicApiIdempotency,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicApiIdempotency {
    #[default]
    None,
    ForwardOrGenerateHeader,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlProjectionKind {
    SecurityState,
    Organization,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlProjectionDeclaration {
    pub kind: ControlProjectionKind,
    pub path: String,
    pub revision_field: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleAssetDeclaration {
    /// Module-local absolute path used by Core when proxying the asset.
    pub path: String,
    /// Content digest included in the public same-origin URL.
    pub digest: ArtifactDigest,
    /// Exact response media type.
    pub content_type: String,
}

/// Sole current real-module manifest contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleManifest {
    #[serde(deserialize_with = "deserialize_manifest_schema_version")]
    pub schema_version: u16,
    pub definition_id: ModuleDefinitionId,
    pub release_version: Version,
    pub publisher: PublisherId,
    pub support: ModuleSupportDeclaration,
    pub platform_versions: ModulePlatformVersions,
    pub linked_packages: LinkedModulePackages,
    pub deployment: DeploymentProfile,
    #[serde(default)]
    pub features: Vec<FeatureDeclaration>,
    #[serde(default)]
    pub provided_contracts: Vec<FunctionalContractDeclaration>,
    #[serde(default)]
    pub dependencies: Vec<FunctionalDependency>,
    #[serde(default)]
    pub resource_types: Vec<ResourceTypeDeclaration>,
    #[serde(default)]
    pub typed_reference_schemas: Vec<TypedReferenceSchemaDeclaration>,
    #[serde(default)]
    pub routes: Vec<RouteDeclaration>,
    #[serde(default)]
    pub browser_routes: Vec<BrowserRouteDeclaration>,
    /// Interactive browser runtime. Absence means every browser route uses
    /// complete-document navigation.
    #[serde(default)]
    pub browser_lifecycle: Option<BrowserLifecycleDeclaration>,
    #[serde(default)]
    pub public_api_routes: Vec<PublicApiRouteDeclaration>,
    #[serde(default)]
    pub control_projections: Vec<ControlProjectionDeclaration>,
    #[serde(default)]
    pub assets: Vec<ModuleAssetDeclaration>,
    #[serde(default)]
    pub navigation: Vec<NavigationContribution>,
    #[serde(default)]
    pub security_capabilities: Vec<SecurityCapabilityDeclaration>,
    pub configuration_schema: Value,
    pub operational_routes: ModuleOperationalRoutes,
    pub shell_contribution_contracts: Option<ShellContributionContracts>,
}

impl ModuleManifest {
    /// Performs deterministic cross-reference and declaration validation.
    pub fn validate(
        &self,
        authority: &ManifestNamespaceAuthority,
    ) -> Result<(), ContractValidationError> {
        let mut findings = Vec::new();
        validate_manifest_schema_version(self.schema_version, &mut findings);
        validate_manifest_authority(self, authority, &mut findings);
        validate_manifest_metadata(self, &mut findings);
        validate_platform_versions(self, &mut findings);
        validate_declaration_graph(
            DeclarationGraph {
                features: &self.features,
                contracts: &self.provided_contracts,
                dependencies: &self.dependencies,
                resource_types: &self.resource_types,
                routes: &self.routes,
                navigation: &self.navigation,
                capabilities: &self.security_capabilities,
            },
            &mut findings,
        );
        validate_manifest_links(self, &mut findings);
        validate_module_assets(&self.assets, &mut findings);
        validate_browser_lifecycle(self, &mut findings);
        match &self.deployment {
            DeploymentProfile::TessaraOciV1(deployment) => {
                validate_deployment(deployment, &mut findings);
            }
        }
        finish_validation(findings)
    }
}

fn validate_module_assets(
    assets: &[ModuleAssetDeclaration],
    findings: &mut Vec<ValidationFinding>,
) {
    let mut paths = BTreeSet::new();
    let mut digests = BTreeSet::new();
    for (index, asset) in assets.iter().enumerate() {
        if !asset.path.starts_with('/')
            || asset.path.contains("..")
            || asset.path.contains(['?', '#'])
        {
            findings.push(ValidationFinding {
                code: "invalid_module_asset_path".into(),
                path: format!("assets[{index}].path"),
                message: "asset path must be an absolute, query-free module-local path".into(),
            });
        }
        if !paths.insert(asset.path.as_str()) {
            findings.push(ValidationFinding {
                code: "duplicate_module_asset_path".into(),
                path: format!("assets[{index}].path"),
                message: "asset paths must be unique".into(),
            });
        }
        if !digests.insert(asset.digest.as_str()) {
            findings.push(ValidationFinding {
                code: "duplicate_module_asset_digest".into(),
                path: format!("assets[{index}].digest"),
                message: "asset digests must be unique within a release".into(),
            });
        }
        require_text(
            &format!("assets[{index}].content_type"),
            &asset.content_type,
            findings,
        );
    }
}

fn validate_browser_lifecycle(manifest: &ModuleManifest, findings: &mut Vec<ValidationFinding>) {
    let Some(lifecycle) = &manifest.browser_lifecycle else {
        return;
    };
    if lifecycle.lifecycle_abi != Version::new(1, 0, 0) {
        findings.push(ValidationFinding {
            code: "unsupported_browser_lifecycle_abi".into(),
            path: "browser_lifecycle.lifecycle_abi".into(),
            message: "Core supports browser lifecycle ABI 1.0.0".into(),
        });
    }
    if manifest.browser_routes.is_empty() {
        findings.push(ValidationFinding {
            code: "browser_lifecycle_without_routes".into(),
            path: "browser_lifecycle".into(),
            message: "an interactive browser lifecycle requires at least one browser route".into(),
        });
    }
    if !lifecycle.complete_document_fallback {
        findings.push(ValidationFinding {
            code: "missing_complete_document_fallback".into(),
            path: "browser_lifecycle.complete_document_fallback".into(),
            message: "lifecycle v1 modules must retain complete-document fallback".into(),
        });
    }

    let assets = manifest
        .assets
        .iter()
        .map(|asset| (asset.path.as_str(), asset.content_type.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    match assets.get(lifecycle.entry_asset.as_str()) {
        Some(content_type) if content_type.starts_with("text/javascript") => {}
        _ => findings.push(ValidationFinding {
            code: "invalid_browser_lifecycle_entry_asset".into(),
            path: "browser_lifecycle.entry_asset".into(),
            message: "entry asset must name a declared JavaScript module asset".into(),
        }),
    }
    let mut stylesheets = BTreeSet::new();
    for (index, path) in lifecycle.stylesheet_assets.iter().enumerate() {
        if !stylesheets.insert(path.as_str()) {
            findings.push(ValidationFinding {
                code: "duplicate_browser_lifecycle_stylesheet".into(),
                path: format!("browser_lifecycle.stylesheet_assets[{index}]"),
                message: "lifecycle stylesheet assets must be unique".into(),
            });
        }
        match assets.get(path.as_str()) {
            Some(content_type) if content_type.starts_with("text/css") => {}
            _ => findings.push(ValidationFinding {
                code: "invalid_browser_lifecycle_stylesheet_asset".into(),
                path: format!("browser_lifecycle.stylesheet_assets[{index}]"),
                message: "stylesheet must name a declared CSS asset".into(),
            }),
        }
    }
}

/// Truthful availability of a current in-process contribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionAvailability {
    ActiveInProcess,
    Unavailable,
    Retired,
}

/// Versioned discovery document for current in-process behavior.
///
/// Deliberately absent: release version, publisher, deployment profile,
/// artifacts, trust/compatibility, instance identity, and operational state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionalContributionDescriptorV1 {
    #[serde(deserialize_with = "deserialize_schema_version_v1")]
    pub schema_version: u16,
    pub reserved_definition_id: ModuleDefinitionId,
    pub display_name: String,
    pub description: String,
    pub availability: TransitionAvailability,
    #[serde(default)]
    pub features: Vec<FeatureDeclaration>,
    #[serde(default)]
    pub provided_contracts: Vec<FunctionalContractDeclaration>,
    #[serde(default)]
    pub dependencies: Vec<FunctionalDependency>,
    #[serde(default)]
    pub resource_types: Vec<ResourceTypeDeclaration>,
    #[serde(default)]
    pub routes: Vec<RouteDeclaration>,
    #[serde(default)]
    pub navigation: Vec<NavigationContribution>,
    #[serde(default)]
    pub security_capabilities: Vec<SecurityCapabilityDeclaration>,
    pub configuration_schema: Option<Value>,
}

impl TransitionalContributionDescriptorV1 {
    /// Transition contributions can never satisfy module dependencies.
    pub const fn provider_eligible(&self) -> bool {
        false
    }

    /// Transition contributions can never be materialized by the Supervisor.
    pub const fn supervisor_materializable(&self) -> bool {
        false
    }

    /// Performs deterministic cross-reference and declaration validation.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        let mut findings = Vec::new();
        validate_schema_version(self.schema_version, &mut findings);
        require_text("display_name", &self.display_name, &mut findings);
        require_text("description", &self.description, &mut findings);
        validate_transition_availability(self, &mut findings);
        validate_declaration_graph(
            DeclarationGraph {
                features: &self.features,
                contracts: &self.provided_contracts,
                dependencies: &self.dependencies,
                resource_types: &self.resource_types,
                routes: &self.routes,
                navigation: &self.navigation,
                capabilities: &self.security_capabilities,
            },
            &mut findings,
        );
        validate_feature_configuration_links(
            &self.features,
            self.configuration_schema.as_ref(),
            &mut findings,
        );
        finish_validation(findings)
    }
}

fn validate_transition_availability(
    descriptor: &TransitionalContributionDescriptorV1,
    findings: &mut Vec<ValidationFinding>,
) {
    if descriptor.availability != TransitionAvailability::Retired {
        return;
    }

    for (path, has_declarations) in [
        ("features", !descriptor.features.is_empty()),
        (
            "provided_contracts",
            !descriptor.provided_contracts.is_empty(),
        ),
        ("dependencies", !descriptor.dependencies.is_empty()),
        ("resource_types", !descriptor.resource_types.is_empty()),
        ("routes", !descriptor.routes.is_empty()),
        ("navigation", !descriptor.navigation.is_empty()),
        (
            "security_capabilities",
            !descriptor.security_capabilities.is_empty(),
        ),
        (
            "configuration_schema",
            descriptor.configuration_schema.is_some(),
        ),
    ] {
        if has_declarations {
            findings.push(ValidationFinding {
                code: "retired_transition_declaration".into(),
                path: path.into(),
                message: format!("retired transition descriptors cannot declare {path}"),
            });
        }
    }
}

/// Inventory projection keeps transition and real-instance shapes disjoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InventoryEntry {
    TransitionalInProcess {
        descriptor: TransitionalContributionDescriptorV1,
    },
    ModuleInstance {
        definition: ModuleDefinition,
        release: ModuleRelease,
        instance: ModuleInstance,
    },
}

impl InventoryEntry {
    /// Verifies that a real inventory entry is internally connected to one
    /// Definition and Release. Transition descriptors are valid by shape.
    pub fn validate_integrity(&self) -> Result<(), InventoryIntegrityError> {
        let Self::ModuleInstance {
            definition,
            release,
            instance,
        } = self
        else {
            return Ok(());
        };

        if definition.id != release.definition_id {
            return Err(InventoryIntegrityError::DefinitionReleaseMismatch);
        }
        if definition.id != instance.definition_id {
            return Err(InventoryIntegrityError::DefinitionInstanceMismatch);
        }
        if release.id != instance.release_id {
            return Err(InventoryIntegrityError::ReleaseInstanceMismatch);
        }
        Ok(())
    }

    /// A transition descriptor is never a real provider candidate.
    pub fn provider_eligible(&self) -> bool {
        if self.validate_integrity().is_err() {
            return false;
        }
        match self {
            Self::TransitionalInProcess { .. } => false,
            Self::ModuleInstance {
                definition,
                release,
                instance,
            } => {
                definition.state == ModuleDefinitionState::Registered
                    && release.trust == ReleaseTrustState::Trusted
                    && release.compatibility == ReleaseCompatibilityState::Compatible
                    && instance.identity_state == InstanceIdentityState::Live
                    && instance.data_state == InstanceDataState::Retained
                    && instance.operation_state.installed
                    && instance.operation_state.deployed
                    && instance.operation_state.configured
                    && instance.operation_state.ready
                    && instance.operation_state.enabled
                    && instance.operation_state.healthy
            }
        }
    }

    /// Only a real Module Release/Instance can be materialization inventory.
    pub fn supervisor_materializable(&self) -> bool {
        matches!(self, Self::ModuleInstance { .. }) && self.validate_integrity().is_ok()
    }

    /// Returns a release only for a real module entry.
    pub const fn release(&self) -> Option<&ModuleRelease> {
        match self {
            Self::TransitionalInProcess { .. } => None,
            Self::ModuleInstance { release, .. } => Some(release),
        }
    }

    /// Returns an instance only for a real module entry.
    pub const fn instance(&self) -> Option<&ModuleInstance> {
        match self {
            Self::TransitionalInProcess { .. } => None,
            Self::ModuleInstance { instance, .. } => Some(instance),
        }
    }
}

/// Integrity failure for a normalized real-module inventory projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum InventoryIntegrityError {
    #[error("Module Definition does not own the projected Module Release")]
    DefinitionReleaseMismatch,
    #[error("Module Definition does not own the projected Module Instance")]
    DefinitionInstanceMismatch,
    #[error("Module Instance does not use the projected Module Release")]
    ReleaseInstanceMismatch,
}

/// Stable validation finding suitable for UI/API projection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationFinding {
    pub code: String,
    pub path: String,
    pub message: String,
}

/// One or more deterministic contract validation findings.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("module contract validation failed with {0} finding(s)", .findings.len())]
pub struct ContractValidationError {
    /// Findings in deterministic declaration order.
    pub findings: Vec<ValidationFinding>,
}

fn validate_schema_version(schema_version: u16, findings: &mut Vec<ValidationFinding>) {
    if schema_version != CONTRACT_SCHEMA_VERSION_V1 {
        findings.push(ValidationFinding {
            code: "unsupported_schema_version".into(),
            path: "schema_version".into(),
            message: format!(
                "expected schema version {CONTRACT_SCHEMA_VERSION_V1}, received {schema_version}"
            ),
        });
    }
}

fn validate_manifest_schema_version(schema_version: u16, findings: &mut Vec<ValidationFinding>) {
    if schema_version != MODULE_MANIFEST_SCHEMA_VERSION {
        findings.push(ValidationFinding {
            code: "unsupported_manifest_schema_version".into(),
            path: "schema_version".into(),
            message: format!(
                "expected module manifest schema version {MODULE_MANIFEST_SCHEMA_VERSION}, received {schema_version}"
            ),
        });
    }
}

fn validate_platform_versions(manifest: &ModuleManifest, findings: &mut Vec<ValidationFinding>) {
    for (path, actual, expected) in [
        (
            "platform_versions.core_release",
            &manifest.platform_versions.core_release,
            CURRENT_CORE_RELEASE,
        ),
        (
            "platform_versions.shell_context_schema",
            &manifest.platform_versions.shell_context_schema,
            CURRENT_SHELL_CONTEXT_SCHEMA,
        ),
        (
            "platform_versions.module_control_protocol",
            &manifest.platform_versions.module_control_protocol,
            CURRENT_MODULE_CONTROL_PROTOCOL,
        ),
        (
            "platform_versions.module_contract",
            &manifest.platform_versions.module_contract,
            CURRENT_MODULE_CONTRACT_VERSION,
        ),
        (
            "platform_versions.module_runtime",
            &manifest.platform_versions.module_runtime,
            CURRENT_MODULE_RUNTIME_VERSION,
        ),
        (
            "platform_versions.module_ui",
            &manifest.platform_versions.module_ui,
            CURRENT_MODULE_UI_VERSION,
        ),
        (
            "platform_versions.design_system_asset_abi",
            &manifest.platform_versions.design_system_asset_abi,
            CURRENT_DESIGN_SYSTEM_ASSET_ABI,
        ),
        (
            "platform_versions.conformance_suite",
            &manifest.platform_versions.conformance_suite,
            CURRENT_CONFORMANCE_SUITE_VERSION,
        ),
    ] {
        let expected = Version::parse(expected).expect("current platform version constant");
        if *actual != expected {
            findings.push(ValidationFinding {
                code: "unsupported_platform_version".into(),
                path: path.into(),
                message: format!("expected exact version {expected}, received {actual}"),
            });
        }
    }
    for (path, linked, declared) in [
        (
            "linked_packages.module_contract",
            Some(&manifest.linked_packages.module_contract),
            &manifest.platform_versions.module_contract,
        ),
        (
            "linked_packages.module_runtime",
            manifest.linked_packages.module_runtime.as_ref(),
            &manifest.platform_versions.module_runtime,
        ),
        (
            "linked_packages.module_ui",
            manifest.linked_packages.module_ui.as_ref(),
            &manifest.platform_versions.module_ui,
        ),
    ] {
        if linked.is_some_and(|linked| linked != declared) {
            findings.push(ValidationFinding {
                code: "linked_package_version_mismatch".into(),
                path: path.into(),
                message: "linked package version must equal the declared current platform version"
                    .into(),
            });
        }
    }
}

struct DeclarationGraph<'a> {
    features: &'a [FeatureDeclaration],
    contracts: &'a [FunctionalContractDeclaration],
    dependencies: &'a [FunctionalDependency],
    resource_types: &'a [ResourceTypeDeclaration],
    routes: &'a [RouteDeclaration],
    navigation: &'a [NavigationContribution],
    capabilities: &'a [SecurityCapabilityDeclaration],
}

fn validate_declaration_graph(graph: DeclarationGraph<'_>, findings: &mut Vec<ValidationFinding>) {
    let DeclarationGraph {
        features,
        contracts,
        dependencies,
        resource_types,
        routes,
        navigation,
        capabilities,
    } = graph;
    let feature_ids = collect_unique(
        "features",
        features.iter().map(|item| item.id.as_str()),
        findings,
    );
    let contract_ids = collect_unique(
        "provided_contracts",
        contracts.iter().map(|item| item.id.as_str()),
        findings,
    );
    let feature_contract_ids = contract_ids
        .iter()
        .cloned()
        .chain(
            dependencies
                .iter()
                .map(|dependency| dependency.contract_id.as_str().to_string()),
        )
        .collect::<BTreeSet<_>>();
    collect_unique(
        "dependencies",
        dependencies
            .iter()
            .map(|dependency| dependency.binding_key.as_str()),
        findings,
    );
    let resource_ids = collect_unique(
        "resource_types",
        resource_types.iter().map(|item| item.id.as_str()),
        findings,
    );
    let route_ids = collect_unique(
        "routes",
        routes.iter().map(|item| item.name.as_str()),
        findings,
    );
    let navigation_ids = collect_unique(
        "navigation",
        navigation.iter().map(|item| item.id.as_str()),
        findings,
    );
    let capability_ids = collect_unique(
        "security_capabilities",
        capabilities.iter().map(|item| item.id.as_str()),
        findings,
    );

    let _ = (feature_ids, navigation_ids);

    for (index, contract) in contracts.iter().enumerate() {
        require_text(
            &format!("provided_contracts[{index}].description"),
            &contract.description,
            findings,
        );
    }
    for (index, resource) in resource_types.iter().enumerate() {
        require_text(
            &format!("resource_types[{index}].description"),
            &resource.description,
            findings,
        );
    }
    for (index, capability) in capabilities.iter().enumerate() {
        require_text(
            &format!("security_capabilities[{index}].description"),
            &capability.description,
            findings,
        );
    }

    for (index, feature) in features.iter().enumerate() {
        require_text(&format!("features[{index}].name"), &feature.name, findings);
        require_text(
            &format!("features[{index}].description"),
            &feature.description,
            findings,
        );
        collect_unique(
            &format!("features[{index}].contracts"),
            feature.contracts.iter().map(|id| id.as_str()),
            findings,
        );
        validate_links(
            &format!("features[{index}].contracts"),
            feature.contracts.iter().map(|id| id.as_str()),
            &feature_contract_ids,
            findings,
        );
        collect_unique(
            &format!("features[{index}].resource_types"),
            feature.resource_types.iter().map(|id| id.as_str()),
            findings,
        );
        validate_links(
            &format!("features[{index}].resource_types"),
            feature.resource_types.iter().map(|id| id.as_str()),
            &resource_ids,
            findings,
        );
        collect_unique(
            &format!("features[{index}].destinations"),
            feature.destinations.iter().map(|id| id.as_str()),
            findings,
        );
        validate_links(
            &format!("features[{index}].destinations"),
            feature.destinations.iter().map(|id| id.as_str()),
            &route_ids,
            findings,
        );
        collect_unique(
            &format!("features[{index}].capabilities"),
            feature.capabilities.iter().map(|id| id.as_str()),
            findings,
        );
        validate_links(
            &format!("features[{index}].capabilities"),
            feature.capabilities.iter().map(|id| id.as_str()),
            &capability_ids,
            findings,
        );
        collect_unique(
            &format!("features[{index}].configuration_pointers"),
            feature.configuration_pointers.iter().map(String::as_str),
            findings,
        );
    }

    for (index, route) in routes.iter().enumerate() {
        collect_unique(
            &format!("routes[{index}].parameters"),
            route
                .parameters
                .iter()
                .map(|parameter| parameter.name.as_str()),
            findings,
        );
        for (parameter_index, parameter) in route.parameters.iter().enumerate() {
            require_text(
                &format!("routes[{index}].parameters[{parameter_index}].name"),
                &parameter.name,
                findings,
            );
        }
    }

    for (index, contribution) in navigation.iter().enumerate() {
        require_text(
            &format!("navigation[{index}].label"),
            &contribution.label,
            findings,
        );
        require_text(
            &format!("navigation[{index}].group"),
            &contribution.group,
            findings,
        );
        if !route_ids.contains(contribution.destination.as_str()) {
            findings.push(ValidationFinding {
                code: "unresolved_navigation_destination".into(),
                path: format!("navigation[{index}].destination"),
                message: format!(
                    "route '{}' is not declared by this document",
                    contribution.destination
                ),
            });
        }
        if contribution.required_capabilities_any_of.is_empty() {
            findings.push(ValidationFinding {
                code: "missing_navigation_capability".into(),
                path: format!("navigation[{index}].required_capabilities_any_of"),
                message: "at least one navigation visibility capability is required".into(),
            });
        }
        collect_unique(
            &format!("navigation[{index}].required_capabilities_any_of"),
            contribution
                .required_capabilities_any_of
                .iter()
                .map(|capability| capability.as_str()),
            findings,
        );
        for (capability_index, capability) in
            contribution.required_capabilities_any_of.iter().enumerate()
        {
            if !capability_ids.contains(capability.as_str()) {
                findings.push(ValidationFinding {
                    code: "unresolved_navigation_capability".into(),
                    path: format!(
                        "navigation[{index}].required_capabilities_any_of[{capability_index}]"
                    ),
                    message: format!(
                        "capability '{}' is not declared by this document",
                        capability
                    ),
                });
            }
        }
    }
}

fn validate_manifest_authority(
    manifest: &ModuleManifest,
    authority: &ManifestNamespaceAuthority,
    findings: &mut Vec<ValidationFinding>,
) {
    if manifest.definition_id != authority.definition_id {
        findings.push(ValidationFinding {
            code: "definition_authority_mismatch".into(),
            path: "definition_id".into(),
            message: format!(
                "manifest Definition '{}' does not match registered Definition '{}'",
                manifest.definition_id, authority.definition_id
            ),
        });
    }
    if manifest.publisher != authority.publisher {
        findings.push(ValidationFinding {
            code: "publisher_authority_mismatch".into(),
            path: "publisher".into(),
            message: format!(
                "manifest publisher '{}' does not match registered publisher '{}'",
                manifest.publisher, authority.publisher
            ),
        });
    }

    for (index, feature) in manifest.features.iter().enumerate() {
        validate_identifier_authority(
            &format!("features[{index}].id"),
            feature.id.as_str(),
            authority,
            findings,
        );
    }
    for (index, contract) in manifest.provided_contracts.iter().enumerate() {
        validate_identifier_authority(
            &format!("provided_contracts[{index}].id"),
            contract.id.as_str(),
            authority,
            findings,
        );
    }
    for (index, dependency) in manifest.dependencies.iter().enumerate() {
        validate_identifier_authority(
            &format!("dependencies[{index}].binding_key"),
            dependency.binding_key.as_str(),
            authority,
            findings,
        );
    }
    for (index, resource_type) in manifest.resource_types.iter().enumerate() {
        validate_identifier_authority(
            &format!("resource_types[{index}].id"),
            resource_type.id.as_str(),
            authority,
            findings,
        );
    }
    for (index, route) in manifest.routes.iter().enumerate() {
        validate_identifier_authority(
            &format!("routes[{index}].name"),
            route.name.as_str(),
            authority,
            findings,
        );
    }
    for (index, navigation) in manifest.navigation.iter().enumerate() {
        validate_identifier_authority(
            &format!("navigation[{index}].id"),
            navigation.id.as_str(),
            authority,
            findings,
        );
    }
    for (index, capability) in manifest.security_capabilities.iter().enumerate() {
        validate_identifier_authority(
            &format!("security_capabilities[{index}].id"),
            capability.id.as_str(),
            authority,
            findings,
        );
    }
}

fn validate_identifier_authority(
    path: &str,
    identifier: &str,
    authority: &ManifestNamespaceAuthority,
    findings: &mut Vec<ValidationFinding>,
) {
    if !authority.allows(identifier) {
        findings.push(ValidationFinding {
            code: "unauthorized_identifier_namespace".into(),
            path: path.into(),
            message: format!(
                "identifier '{identifier}' is not owned by the registered manifest authority"
            ),
        });
    }
}

fn validate_manifest_metadata(manifest: &ModuleManifest, findings: &mut Vec<ValidationFinding>) {
    require_text(
        "support.support_tier",
        &manifest.support.support_tier,
        findings,
    );
    require_text("support.contact", &manifest.support.contact, findings);
    require_text(
        "support.documentation",
        &manifest.support.documentation,
        findings,
    );
    if !manifest.configuration_schema.is_object() {
        findings.push(ValidationFinding {
            code: "invalid_configuration_schema".into(),
            path: "configuration_schema".into(),
            message: "configuration schema must be a JSON object".into(),
        });
    } else {
        validate_managed_configuration_schema(&manifest.configuration_schema, findings);
    }
}

fn validate_managed_configuration_schema(schema: &Value, findings: &mut Vec<ValidationFinding>) {
    if schema.get("type").and_then(Value::as_str) != Some("object") {
        findings.push(ValidationFinding {
            code: "invalid_configuration_schema".into(),
            path: "configuration_schema.type".into(),
            message: "managed module configuration must use an object schema".into(),
        });
    }
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        findings.push(ValidationFinding {
            code: "invalid_configuration_schema".into(),
            path: "configuration_schema.properties".into(),
            message: "managed module configuration must declare object properties".into(),
        });
        return;
    };
    for (name, property) in properties {
        let kind = property.get("type").and_then(Value::as_str);
        if !matches!(kind, Some("string" | "integer" | "number" | "boolean")) {
            findings.push(ValidationFinding {
                code: "unsupported_configuration_field_type".into(),
                path: format!("configuration_schema.properties.{name}.type"),
                message: "managed configuration fields must be string, integer, number, or boolean"
                    .into(),
            });
        }
        if let Some(choices) = property.get("enum")
            && !choices.is_array()
        {
            findings.push(ValidationFinding {
                code: "invalid_configuration_field_enum".into(),
                path: format!("configuration_schema.properties.{name}.enum"),
                message: "configuration field enum must be an array".into(),
            });
        }
    }
}

fn validate_manifest_links(manifest: &ModuleManifest, findings: &mut Vec<ValidationFinding>) {
    let resource_ids = manifest
        .resource_types
        .iter()
        .map(|resource| resource.id.as_str())
        .collect::<BTreeSet<_>>();
    let reference_schema_ids = collect_unique(
        "typed_reference_schemas",
        manifest
            .typed_reference_schemas
            .iter()
            .map(|declaration| declaration.resource_type.as_str()),
        findings,
    );
    for (index, declaration) in manifest.typed_reference_schemas.iter().enumerate() {
        if !resource_ids.contains(declaration.resource_type.as_str()) {
            findings.push(ValidationFinding {
                code: "unresolved_reference_schema_resource_type".into(),
                path: format!("typed_reference_schemas[{index}].resource_type"),
                message: format!(
                    "resource type '{}' is not declared by this manifest",
                    declaration.resource_type
                ),
            });
        }
        if let Err(error) = declaration.resource_id.validate_declaration() {
            findings.push(ValidationFinding {
                code: "invalid_reference_schema".into(),
                path: format!("typed_reference_schemas[{index}].resource_id"),
                message: error.to_string(),
            });
        }
    }
    for (index, resource_type) in manifest.resource_types.iter().enumerate() {
        if !reference_schema_ids.contains(resource_type.id.as_str()) {
            findings.push(ValidationFinding {
                code: "missing_reference_schema".into(),
                path: format!("resource_types[{index}].id"),
                message: format!(
                    "resource type '{}' must declare exactly one reference schema",
                    resource_type.id
                ),
            });
        }
    }

    validate_feature_configuration_links(
        &manifest.features,
        Some(&manifest.configuration_schema),
        findings,
    );

    let route_kinds = manifest
        .routes
        .iter()
        .map(|route| (route.name.as_str(), route.kind))
        .collect::<std::collections::BTreeMap<_, _>>();
    let capability_ids = manifest
        .security_capabilities
        .iter()
        .map(|capability| capability.id.as_str())
        .collect::<BTreeSet<_>>();
    let contract_ids = manifest
        .provided_contracts
        .iter()
        .map(|contract| contract.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut browser_paths: Vec<&str> = Vec::new();
    for (index, browser_route) in manifest.browser_routes.iter().enumerate() {
        let base = format!("browser_routes[{index}]");
        if !route_kinds.contains_key(browser_route.destination.as_str()) {
            findings.push(ValidationFinding {
                code: "unresolved_browser_destination".into(),
                path: format!("{base}.destination"),
                message: "browser destination must name a declared semantic route".into(),
            });
        }
        if !browser_route.path_template.starts_with('/')
            || browser_route.path_template.contains("..")
            || browser_route.path_template.starts_with("/api/")
            || browser_route.path_template.starts_with("/administration/")
            || browser_route.path_template.starts_with("/_tessara/")
        {
            findings.push(ValidationFinding {
                code: "invalid_browser_path_template".into(),
                path: format!("{base}.path_template"),
                message:
                    "browser path must be absolute, traversal-free, and outside reserved Core paths"
                        .into(),
            });
        }
        if let Some(previous_index) = browser_paths.iter().position(|previous| {
            browser_path_templates_overlap(previous, &browser_route.path_template)
        }) {
            findings.push(ValidationFinding {
                code: "ambiguous_browser_path_template".into(),
                path: format!("{base}.path_template"),
                message: format!(
                    "browser path overlaps browser_routes[{previous_index}].path_template"
                ),
            });
        }
        browser_paths.push(browser_route.path_template.as_str());
        let method_count = browser_route.methods.iter().collect::<BTreeSet<_>>().len();
        if browser_route.methods.is_empty() || method_count != browser_route.methods.len() {
            findings.push(ValidationFinding {
                code: "invalid_browser_methods".into(),
                path: format!("{base}.methods"),
                message: "browser document routes require unique GET/HEAD methods".into(),
            });
        }
        if !capability_ids.contains(browser_route.required_capability.as_str()) {
            findings.push(ValidationFinding {
                code: "unresolved_browser_capability".into(),
                path: format!("{base}.required_capability"),
                message: "browser route capability must be declared by the manifest".into(),
            });
        }
        if !contract_ids.contains(browser_route.functional_contract.as_str()) {
            findings.push(ValidationFinding {
                code: "unresolved_browser_contract".into(),
                path: format!("{base}.functional_contract"),
                message: "browser route functional contract must be provided by the manifest"
                    .into(),
            });
        }
        require_text(
            &format!("{base}.authorization_action"),
            &browser_route.authorization_action,
            findings,
        );
        if let Some(scope_parameter) = &browser_route.organization_scope_parameter
            && !browser_route
                .path_template
                .split('/')
                .any(|segment| segment == format!("{{{scope_parameter}}}"))
        {
            findings.push(ValidationFinding {
                code: "unresolved_browser_scope_parameter".into(),
                path: format!("{base}.organization_scope_parameter"),
                message: "scope parameter must name a path-template parameter".into(),
            });
        }
    }
    let mut public_api_keys = BTreeSet::new();
    for (index, api_route) in manifest.public_api_routes.iter().enumerate() {
        let base = format!("public_api_routes[{index}]");
        let key = (api_route.method, api_route.path_template.as_str());
        if !public_api_keys.insert(key) {
            findings.push(ValidationFinding {
                code: "duplicate_public_api_route".into(),
                path: base.clone(),
                message: "public API method and path must be unique".into(),
            });
        }
        if !api_route.path_template.starts_with("/api/")
            || api_route.path_template.contains("..")
            || api_route.path_template.starts_with("/api/private/")
        {
            findings.push(ValidationFinding {
                code: "invalid_public_api_path_template".into(),
                path: format!("{base}.path_template"),
                message: "public API paths must be local /api paths outside /api/private".into(),
            });
        }
        if !capability_ids.contains(api_route.required_capability.as_str()) {
            findings.push(ValidationFinding {
                code: "unresolved_public_api_capability".into(),
                path: format!("{base}.required_capability"),
                message: "public API capability must be declared by the manifest".into(),
            });
        }
        if !contract_ids.contains(api_route.functional_contract.as_str()) {
            findings.push(ValidationFinding {
                code: "unresolved_public_api_contract".into(),
                path: format!("{base}.functional_contract"),
                message: "public API contract must be provided by the manifest".into(),
            });
        }
        require_text(
            &format!("{base}.authorization_action"),
            &api_route.authorization_action,
            findings,
        );
        if api_route.idempotency == PublicApiIdempotency::ForwardOrGenerateHeader
            && api_route.operation != AuthorizationGrantOperationV1::Mutation
        {
            findings.push(ValidationFinding {
                code: "invalid_public_api_idempotency".into(),
                path: format!("{base}.idempotency"),
                message: "idempotency headers are declared only for mutation routes".into(),
            });
        }
    }
    let mut projection_kinds = BTreeSet::new();
    for (index, projection) in manifest.control_projections.iter().enumerate() {
        let base = format!("control_projections[{index}]");
        if !projection_kinds.insert(projection.kind) {
            findings.push(ValidationFinding {
                code: "duplicate_control_projection".into(),
                path: base.clone(),
                message: "control projection kinds must be unique".into(),
            });
        }
        if !projection.path.starts_with("/api/private/") || projection.path.contains("..") {
            findings.push(ValidationFinding {
                code: "invalid_control_projection_path".into(),
                path: format!("{base}.path"),
                message: "control projections require local /api/private paths".into(),
            });
        }
        require_text(
            &format!("{base}.revision_field"),
            &projection.revision_field,
            findings,
        );
    }
    for (path, route_name, expected_kind) in [
        (
            "operational_routes.configuration_validation",
            &manifest.operational_routes.configuration_validation,
            RouteKind::Configuration,
        ),
        (
            "operational_routes.compatibility",
            &manifest.operational_routes.compatibility,
            RouteKind::Diagnostics,
        ),
        (
            "operational_routes.status",
            &manifest.operational_routes.status,
            RouteKind::Diagnostics,
        ),
        (
            "operational_routes.diagnostics",
            &manifest.operational_routes.diagnostics,
            RouteKind::Diagnostics,
        ),
    ] {
        match route_kinds.get(route_name.as_str()) {
            None => findings.push(ValidationFinding {
                code: "unresolved_operational_route".into(),
                path: path.into(),
                message: format!("route '{route_name}' is not declared by this manifest"),
            }),
            Some(actual_kind) if *actual_kind != expected_kind => {
                findings.push(ValidationFinding {
                    code: "invalid_operational_route_kind".into(),
                    path: path.into(),
                    message: format!("route '{route_name}' must be declared as {expected_kind:?}"),
                });
            }
            Some(_) => {}
        }
    }

    if let Some(contributions) = manifest.shell_contribution_contracts.as_ref() {
        for (path, contract) in [
            ("shell_contribution_contracts.home", &contributions.home),
            (
                "shell_contribution_contracts.work_discovery",
                &contributions.work_discovery,
            ),
            ("shell_contribution_contracts.search", &contributions.search),
        ] {
            if let Some(contract) = contract
                && !contract_ids.contains(contract.as_str())
            {
                findings.push(ValidationFinding {
                    code: "unresolved_shell_contribution_contract".into(),
                    path: path.into(),
                    message: format!("contract '{contract}' is not provided by this manifest"),
                });
            }
        }
    }
}

fn browser_path_templates_overlap(left: &str, right: &str) -> bool {
    let left = left.trim_matches('/').split('/').collect::<Vec<_>>();
    let right = right.trim_matches('/').split('/').collect::<Vec<_>>();
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            *left == right
                || (left.starts_with('{')
                    && left.ends_with('}')
                    && right.starts_with('{')
                    && right.ends_with('}'))
        })
}

fn validate_feature_configuration_links(
    features: &[FeatureDeclaration],
    configuration_schema: Option<&Value>,
    findings: &mut Vec<ValidationFinding>,
) {
    for (feature_index, feature) in features.iter().enumerate() {
        for (pointer_index, pointer) in feature.configuration_pointers.iter().enumerate() {
            let path = format!("features[{feature_index}].configuration_pointers[{pointer_index}]");
            if !pointer.starts_with('/') {
                findings.push(ValidationFinding {
                    code: "invalid_configuration_pointer".into(),
                    path,
                    message: "configuration link must be a JSON Pointer beginning with '/'".into(),
                });
            } else if configuration_schema.is_none_or(|schema| schema.pointer(pointer).is_none()) {
                findings.push(ValidationFinding {
                    code: "unresolved_configuration_pointer".into(),
                    path,
                    message: format!(
                        "configuration pointer '{pointer}' is not present in the configuration schema"
                    ),
                });
            }
        }
    }
}

fn validate_deployment(deployment: &TessaraOciV1, findings: &mut Vec<ValidationFinding>) {
    validate_oci_image(
        "deployment.declaration.runtime_image",
        &deployment.runtime_image,
        "runtime",
        findings,
    );
    if let Some(image) = deployment.migration_image.as_ref() {
        validate_oci_image(
            "deployment.declaration.migration_image",
            image,
            "migration",
            findings,
        );
    }

    if deployment.listen.protocol != "http" {
        findings.push(ValidationFinding {
            code: "unsupported_service_protocol".into(),
            path: "deployment.declaration.listen.protocol".into(),
            message: "tessara-oci-v1 supports only same-origin HTTP registration".into(),
        });
    }
    if deployment.listen.port == 0 {
        findings.push(ValidationFinding {
            code: "invalid_service_port".into(),
            path: "deployment.declaration.listen.port".into(),
            message: "service port must be greater than zero".into(),
        });
    }
    require_text(
        "deployment.declaration.listen.registration_name",
        &deployment.listen.registration_name,
        findings,
    );

    let configuration_keys = collect_unique(
        "deployment.declaration.configuration_keys",
        deployment.configuration_keys.iter().map(String::as_str),
        findings,
    );
    for (index, key) in deployment.configuration_keys.iter().enumerate() {
        require_text(
            &format!("deployment.declaration.configuration_keys[{index}]"),
            key,
            findings,
        );
    }
    collect_unique(
        "deployment.declaration.secret_keys",
        deployment.secret_keys.iter().map(String::as_str),
        findings,
    );
    for (index, key) in deployment.secret_keys.iter().enumerate() {
        require_text(
            &format!("deployment.declaration.secret_keys[{index}]"),
            key,
            findings,
        );
        if configuration_keys.contains(key) {
            findings.push(ValidationFinding {
                code: "configuration_secret_key_overlap".into(),
                path: format!("deployment.declaration.secret_keys[{index}]"),
                message: format!("key '{key}' cannot be both configuration and secret input"),
            });
        }
    }

    require_text(
        "deployment.declaration.runtime_identity",
        &deployment.runtime_identity,
        findings,
    );
    require_text(
        "deployment.declaration.migration_identity",
        &deployment.migration_identity,
        findings,
    );
    if deployment.runtime_identity == deployment.migration_identity {
        findings.push(ValidationFinding {
            code: "shared_runtime_migration_identity".into(),
            path: "deployment.declaration.migration_identity".into(),
            message: "runtime and migration identities must be distinct".into(),
        });
    }

    for (path, value) in [
        (
            "deployment.declaration.readiness_path",
            &deployment.readiness_path,
        ),
        (
            "deployment.declaration.liveness_path",
            &deployment.liveness_path,
        ),
    ] {
        if !valid_same_origin_path(value) {
            findings.push(ValidationFinding {
                code: "invalid_probe_path".into(),
                path: path.into(),
                message: "probe path must be a local absolute path without an authority, query, fragment, or traversal segment".into(),
            });
        }
    }
    if deployment.graceful_shutdown_seconds == 0 {
        findings.push(ValidationFinding {
            code: "invalid_graceful_shutdown".into(),
            path: "deployment.declaration.graceful_shutdown_seconds".into(),
            message: "graceful shutdown duration must be greater than zero".into(),
        });
    }

    if deployment.resources.cpu_request_millis == 0
        || deployment.resources.cpu_limit_millis == 0
        || deployment.resources.memory_request_mebibytes == 0
        || deployment.resources.memory_limit_mebibytes == 0
    {
        findings.push(ValidationFinding {
            code: "non_positive_resource_requirement".into(),
            path: "deployment.declaration.resources".into(),
            message: "resource requests and limits must all be greater than zero".into(),
        });
    }
    if deployment.resources.cpu_limit_millis < deployment.resources.cpu_request_millis
        || deployment.resources.memory_limit_mebibytes
            < deployment.resources.memory_request_mebibytes
    {
        findings.push(ValidationFinding {
            code: "invalid_resource_limits".into(),
            path: "deployment.declaration.resources".into(),
            message: "resource limits must be greater than or equal to requests".into(),
        });
    }
}

fn validate_oci_image(
    path: &str,
    image: &OciImageDeclaration,
    purpose: &str,
    findings: &mut Vec<ValidationFinding>,
) {
    if !valid_oci_image_reference(&image.image_reference, &image.digest) {
        findings.push(ValidationFinding {
            code: "invalid_oci_image_reference".into(),
            path: format!("{path}.image_reference"),
            message: "OCI image reference must be a non-URL repository reference pinned with '@' to the declared digest".into(),
        });
    }
    if image.platform != "linux" {
        findings.push(ValidationFinding {
            code: "unsupported_image_platform".into(),
            path: format!("{path}.platform"),
            message: "tessara-oci-v1 supports only the linux platform".into(),
        });
    }
    if !matches!(image.architecture.as_str(), "amd64" | "arm64") {
        findings.push(ValidationFinding {
            code: "unsupported_image_architecture".into(),
            path: format!("{path}.architecture"),
            message: "tessara-oci-v1 supports amd64 or arm64 images".into(),
        });
    }
    if image.command.is_empty() {
        findings.push(ValidationFinding {
            code: format!("missing_{purpose}_command"),
            path: format!("{path}.command"),
            message: format!("{purpose} image must declare a command"),
        });
    }
    for (index, argument) in image.command.iter().enumerate() {
        require_text(&format!("{path}.command[{index}]"), argument, findings);
    }
}

fn valid_oci_image_reference(reference: &str, digest: &ArtifactDigest) -> bool {
    let Some((repository, pinned_digest)) = reference.rsplit_once('@') else {
        return false;
    };
    !repository.is_empty()
        && repository == repository.trim()
        && !repository.chars().any(char::is_whitespace)
        && !repository.contains("://")
        && !repository.contains(['@', '?', '#', '\\'])
        && !repository.starts_with('/')
        && !repository.ends_with('/')
        && !repository
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
        && pinned_digest == digest.as_str()
}

fn valid_same_origin_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains(['?', '#', '\\'])
        && !path.contains("://")
        && !path.split('/').any(|segment| matches!(segment, "." | ".."))
}

fn collect_unique<'a>(
    path: &str,
    values: impl Iterator<Item = &'a str>,
    findings: &mut Vec<ValidationFinding>,
) -> BTreeSet<String> {
    let mut collected = BTreeSet::new();
    for (index, value) in values.enumerate() {
        if !collected.insert(value.to_string()) {
            findings.push(ValidationFinding {
                code: "duplicate_identifier".into(),
                path: format!("{path}[{index}]"),
                message: format!("identifier '{value}' is declared more than once"),
            });
        }
    }
    collected
}

fn validate_links<'a>(
    path: &str,
    values: impl Iterator<Item = &'a str>,
    declared: &BTreeSet<String>,
    findings: &mut Vec<ValidationFinding>,
) {
    for (index, value) in values.enumerate() {
        if !declared.contains(value) {
            findings.push(ValidationFinding {
                code: "unresolved_feature_link".into(),
                path: format!("{path}[{index}]"),
                message: format!("identifier '{value}' is not declared by this document"),
            });
        }
    }
}

fn require_text(path: &str, value: &str, findings: &mut Vec<ValidationFinding>) {
    if value.trim().is_empty() {
        findings.push(ValidationFinding {
            code: "required_text".into(),
            path: path.into(),
            message: "value is required".into(),
        });
    }
}

fn finish_validation(findings: Vec<ValidationFinding>) -> Result<(), ContractValidationError> {
    if findings.is_empty() {
        Ok(())
    } else {
        Err(ContractValidationError { findings })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use semver::{Version, VersionReq};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn id<T: FromStr>(value: &str) -> T
    where
        T::Err: fmt::Debug,
    {
        value.parse().expect("valid test identifier")
    }

    fn forms_transition() -> TransitionalContributionDescriptorV1 {
        TransitionalContributionDescriptorV1 {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
            reserved_definition_id: id("tessara.forms"),
            display_name: "Forms".into(),
            description: "Current in-process form authoring contribution.".into(),
            availability: TransitionAvailability::ActiveInProcess,
            features: vec![FeatureDeclaration {
                id: id("tessara.forms.authoring"),
                name: "Form authoring".into(),
                description: "Author and publish forms.".into(),
                use_cases: vec!["Build a data collection instrument".into()],
                inputs: vec!["Field definitions".into()],
                outcomes: vec!["Published FormVersion".into()],
                constraints: vec!["Publication policy remains Forms-owned".into()],
                contracts: vec![id("tessara.forms.form-version")],
                resource_types: vec![id("tessara.transition.form_version")],
                destinations: vec![id("forms.directory")],
                capabilities: vec![id("forms:read"), id("forms:manage")],
                configuration_pointers: Vec::new(),
            }],
            provided_contracts: vec![FunctionalContractDeclaration {
                id: id("tessara.forms.form-version"),
                version: Version::new(1, 0, 0),
                kind: FunctionalContractKind::Resource,
                description: "Resolve a FormVersion.".into(),
            }],
            dependencies: Vec::new(),
            resource_types: vec![ResourceTypeDeclaration {
                id: id("tessara.transition.form_version"),
                description: "Core-owned transition FormVersion reference.".into(),
            }],
            routes: vec![RouteDeclaration {
                name: id("forms.directory"),
                kind: RouteKind::Product,
                parameters: Vec::new(),
            }],
            navigation: vec![NavigationContribution {
                id: id("tessara.forms.navigation"),
                destination: id("forms.directory"),
                label: "Forms".into(),
                group: "Main".into(),
                order_hint: 20,
                required_capabilities_any_of: vec![id("forms:read"), id("forms:manage")],
            }],
            security_capabilities: vec![
                SecurityCapabilityDeclaration {
                    id: id("forms:read"),
                    description: "Inspect forms.".into(),
                },
                SecurityCapabilityDeclaration {
                    id: id("forms:manage"),
                    description: "Manage forms.".into(),
                },
            ],
            configuration_schema: None,
        }
    }

    fn forms_authority() -> ManifestNamespaceAuthority {
        ManifestNamespaceAuthority::new(
            id("tessara.forms"),
            id("tessara.first_party"),
            ["tessara.forms", "forms"],
        )
        .expect("valid manifest authority")
    }

    fn forms_manifest() -> ModuleManifest {
        let mut transition = forms_transition();
        let form_version_type: ResourceTypeId = id("tessara.forms.form-version");
        transition.features[0].resource_types = vec![form_version_type.clone()];
        transition.features[0].configuration_pointers = vec!["/properties/default_locale".into()];
        transition.resource_types[0] = ResourceTypeDeclaration {
            id: form_version_type.clone(),
            description: "Forms-owned FormVersion reference.".into(),
        };
        transition.routes.extend([
            RouteDeclaration {
                name: id("forms.configuration.validate"),
                kind: RouteKind::Configuration,
                parameters: Vec::new(),
            },
            RouteDeclaration {
                name: id("forms.compatibility"),
                kind: RouteKind::Diagnostics,
                parameters: Vec::new(),
            },
            RouteDeclaration {
                name: id("forms.status"),
                kind: RouteKind::Diagnostics,
                parameters: Vec::new(),
            },
            RouteDeclaration {
                name: id("forms.diagnostics"),
                kind: RouteKind::Diagnostics,
                parameters: Vec::new(),
            },
        ]);

        ModuleManifest {
            schema_version: MODULE_MANIFEST_SCHEMA_VERSION,
            definition_id: transition.reserved_definition_id,
            release_version: Version::new(1, 0, 0),
            publisher: id("tessara.first_party"),
            support: ModuleSupportDeclaration {
                support_tier: "first_party".into(),
                contact: "support@tessara.example".into(),
                documentation: "https://docs.tessara.example/modules/forms".into(),
            },
            platform_versions: ModulePlatformVersions {
                core_release: Version::new(0, 1, 0),
                shell_context_schema: Version::new(1, 0, 0),
                module_control_protocol: Version::new(1, 1, 0),
                module_contract: Version::new(0, 2, 0),
                module_runtime: Version::new(0, 2, 0),
                module_ui: Version::new(0, 2, 0),
                design_system_asset_abi: Version::new(1, 0, 0),
                conformance_suite: Version::new(1, 1, 0),
            },
            linked_packages: LinkedModulePackages {
                module_contract: Version::new(0, 2, 0),
                module_runtime: Some(Version::new(0, 2, 0)),
                module_ui: Some(Version::new(0, 2, 0)),
            },
            deployment: DeploymentProfile::TessaraOciV1(TessaraOciV1 {
                runtime_image: OciImageDeclaration {
                    image_reference: format!(
                        "registry.example/tessara/forms@sha256:{}",
                        "a".repeat(64)
                    ),
                    digest: ArtifactDigest::new(format!("sha256:{}", "a".repeat(64))).unwrap(),
                    platform: "linux".into(),
                    architecture: "amd64".into(),
                    command: vec!["/app/forms".into(), "serve".into()],
                },
                migration_image: Some(OciImageDeclaration {
                    image_reference: format!(
                        "registry.example/tessara/forms-migrations@sha256:{}",
                        "b".repeat(64)
                    ),
                    digest: ArtifactDigest::new(format!("sha256:{}", "b".repeat(64))).unwrap(),
                    platform: "linux".into(),
                    architecture: "amd64".into(),
                    command: vec!["/app/forms".into(), "migrate".into()],
                }),
                listen: ServiceListenDeclaration {
                    protocol: "http".into(),
                    port: 8080,
                    registration_name: "forms".into(),
                },
                configuration_keys: vec!["DEFAULT_LOCALE".into()],
                secret_keys: vec!["DATABASE_URL".into()],
                runtime_identity: "forms-runtime".into(),
                migration_identity: "forms-migration".into(),
                readiness_path: "/health/ready".into(),
                liveness_path: "/health/live".into(),
                graceful_shutdown_seconds: 30,
                resources: RuntimeResourceDeclaration {
                    cpu_request_millis: 250,
                    cpu_limit_millis: 500,
                    memory_request_mebibytes: 128,
                    memory_limit_mebibytes: 256,
                },
            }),
            features: transition.features,
            provided_contracts: transition.provided_contracts,
            dependencies: transition.dependencies,
            resource_types: transition.resource_types,
            typed_reference_schemas: vec![TypedReferenceSchemaDeclaration {
                resource_type: form_version_type,
                resource_id: ResourceIdentifierSchema::OpaqueString {
                    min_length: 1,
                    max_length: 256,
                },
            }],
            routes: transition.routes,
            browser_routes: Vec::new(),
            browser_lifecycle: None,
            public_api_routes: Vec::new(),
            control_projections: Vec::new(),
            assets: Vec::new(),
            navigation: transition.navigation,
            security_capabilities: transition.security_capabilities,
            configuration_schema: json!({
                "type": "object",
                "properties": {"default_locale": {"type": "string"}}
            }),
            operational_routes: ModuleOperationalRoutes {
                configuration_validation: id("forms.configuration.validate"),
                compatibility: id("forms.compatibility"),
                status: id("forms.status"),
                diagnostics: id("forms.diagnostics"),
            },
            shell_contribution_contracts: None,
        }
    }

    fn forms_inventory() -> InventoryEntry {
        let definition_id: ModuleDefinitionId = id("tessara.forms");
        let release_id = Uuid::new_v4();
        InventoryEntry::ModuleInstance {
            definition: ModuleDefinition {
                id: definition_id.clone(),
                state: ModuleDefinitionState::Registered,
            },
            release: ModuleRelease {
                id: release_id,
                definition_id: definition_id.clone(),
                version: Version::new(1, 0, 0),
                manifest_digest: ArtifactDigest::new(format!("sha256:{}", "c".repeat(64))).unwrap(),
                trust: ReleaseTrustState::Trusted,
                compatibility: ReleaseCompatibilityState::Compatible,
            },
            instance: ModuleInstance {
                id: Uuid::new_v4(),
                installation_id: Uuid::new_v4(),
                definition_id,
                release_id,
                identity_state: InstanceIdentityState::Live,
                operation_state: InstanceOperationState {
                    installed: true,
                    deployed: true,
                    configured: true,
                    ready: true,
                    enabled: true,
                    healthy: true,
                },
                data_state: InstanceDataState::Retained,
            },
        }
    }

    #[test]
    fn identifiers_require_stable_lowercase_namespaces() {
        assert!(ModuleDefinitionId::new("tessara.forms").is_ok());
        assert!(SecurityCapabilityId::new("forms:read").is_ok());
        assert!(ModuleDefinitionId::new("forms").is_err());
        assert!(ModuleDefinitionId::new("1.tessara").is_err());
        assert!(ModuleDefinitionId::new("Tessara.Forms").is_err());
        assert!(ModuleDefinitionId::new("tessara..forms").is_err());
        assert!(ModuleDefinitionId::new("tessara.forms/").is_err());
    }

    #[test]
    fn invalid_identifiers_are_rejected_during_deserialization() {
        let result = serde_json::from_value::<ModuleDefinitionId>(json!("Tessara.Forms"));
        assert!(result.is_err());
    }

    #[test]
    fn validation_findings_are_owned_wire_values() {
        let finding = ValidationFinding {
            code: "test_finding".into(),
            path: "features[0]".into(),
            message: "Test finding".into(),
        };
        let encoded = serde_json::to_string(&finding).expect("serialize finding");
        let decoded: ValidationFinding =
            serde_json::from_str(&encoded).expect("deserialize finding");
        assert_eq!(decoded, finding);

        let mut hostile = serde_json::to_value(finding).expect("serialize finding value");
        hostile["future_detail"] = json!("must not be ignored");
        assert!(serde_json::from_value::<ValidationFinding>(hostile).is_err());
    }

    #[test]
    fn identity_and_lifecycle_wire_shapes_reject_unknown_fields_at_every_nested_layer() {
        let component = CoreComponentArtifact {
            digest: ArtifactDigest::new(format!("sha256:{}", "d".repeat(64))).unwrap(),
            platform: "linux".into(),
            architecture: "amd64".into(),
        };
        let installation = ApplicationInstallation {
            id: Uuid::new_v4(),
            core_runtime: CoreRuntimeObservation::Exact {
                release: CoreRelease {
                    id: Uuid::new_v4(),
                    version: Version::new(1, 0, 0),
                    core_component: component.clone(),
                    gateway_component: component,
                },
            },
        };
        let installation_wire =
            serde_json::to_value(installation).expect("serialize Application Installation");
        let mut installation_cases = Vec::new();
        let mut unknown_installation = installation_wire.clone();
        unknown_installation["future_state"] = json!("unknown");
        installation_cases.push(unknown_installation);
        let mut unknown_observation = installation_wire.clone();
        unknown_observation["core_runtime"]["future_state"] = json!("unknown");
        installation_cases.push(unknown_observation);
        let mut unknown_release = installation_wire.clone();
        unknown_release["core_runtime"]["release"]["future_state"] = json!("unknown");
        installation_cases.push(unknown_release);
        let mut unknown_component = installation_wire;
        unknown_component["core_runtime"]["release"]["core_component"]["future_state"] =
            json!("unknown");
        installation_cases.push(unknown_component);

        for wire in installation_cases {
            let error = serde_json::from_value::<ApplicationInstallation>(wire)
                .expect_err("identity wire must reject unknown fields");
            assert!(error.to_string().contains("unknown field `future_state`"));
        }

        let inventory_wire = serde_json::to_value(forms_inventory()).expect("serialize inventory");
        let mut inventory_cases = Vec::new();
        let mut unknown_definition = inventory_wire.clone();
        unknown_definition["definition"]["future_state"] = json!("unknown");
        inventory_cases.push(unknown_definition);
        let mut unknown_module_release = inventory_wire.clone();
        unknown_module_release["release"]["future_state"] = json!("unknown");
        inventory_cases.push(unknown_module_release);
        let mut unknown_instance = inventory_wire.clone();
        unknown_instance["instance"]["future_state"] = json!("unknown");
        inventory_cases.push(unknown_instance);
        let mut unknown_operation = inventory_wire;
        unknown_operation["instance"]["operation_state"]["future_state"] = json!("unknown");
        inventory_cases.push(unknown_operation);

        for wire in inventory_cases {
            let error = serde_json::from_value::<InventoryEntry>(wire)
                .expect_err("lifecycle wire must reject unknown fields");
            assert!(error.to_string().contains("unknown field `future_state`"));
        }
    }

    #[test]
    fn valid_manifest_round_trips_with_an_explicit_deployment_profile() {
        let manifest = forms_manifest();
        manifest
            .validate(&forms_authority())
            .expect("manifest is valid");

        let json = serde_json::to_value(&manifest).expect("serialize manifest");
        assert_eq!(json["deployment"]["profile"], "tessara-oci-v1");
        let decoded: ModuleManifest = serde_json::from_value(json).expect("deserialize manifest");
        assert_eq!(decoded, manifest);
    }

    #[test]
    fn manifest_support_window_is_the_exact_current_tuple() {
        let mut manifest = forms_manifest();
        manifest.platform_versions.module_runtime = Version::new(0, 0, 9);
        let error = manifest
            .validate(&forms_authority())
            .expect_err("obsolete runtime version must fail");
        assert!(error.findings.iter().any(|finding| {
            finding.code == "unsupported_platform_version"
                && finding.path == "platform_versions.module_runtime"
        }));

        let mut linked = forms_manifest();
        linked.linked_packages.module_ui = Some(Version::new(0, 0, 9));
        let error = linked
            .validate(&forms_authority())
            .expect_err("linked package inventory must be truthful");
        assert!(error.findings.iter().any(|finding| {
            finding.code == "linked_package_version_mismatch"
                && finding.path == "linked_packages.module_ui"
        }));
    }

    #[test]
    fn browser_path_templates_allow_static_precedence_and_reject_parameter_ambiguity() {
        assert!(!browser_path_templates_overlap(
            "/reference/example/{organization_id}",
            "/reference/example/current",
        ));
        assert!(browser_path_templates_overlap(
            "/reference/example/{left}",
            "/reference/example/{right}",
        ));
        assert!(!browser_path_templates_overlap(
            "/reference/example",
            "/reference/example/{organization_id}",
        ));
        assert!(!browser_path_templates_overlap(
            "/reference/example/{organization_id}",
            "/reference/other/{organization_id}",
        ));
    }

    #[test]
    fn every_manifest_resource_type_requires_one_valid_reference_schema() {
        let mut missing = forms_manifest();
        missing.typed_reference_schemas.clear();
        let error = missing
            .validate(&forms_authority())
            .expect_err("reference schema is required");
        assert!(error.findings.iter().any(|finding| {
            finding.code == "missing_reference_schema" && finding.path == "resource_types[0].id"
        }));

        let mut invalid = forms_manifest();
        invalid.typed_reference_schemas[0].resource_id = ResourceIdentifierSchema::OpaqueString {
            min_length: 20,
            max_length: 10,
        };
        let error = invalid
            .validate(&forms_authority())
            .expect_err("reference schema bounds are invalid");
        assert!(error.findings.iter().any(|finding| {
            finding.code == "invalid_reference_schema"
                && finding.path == "typed_reference_schemas[0].resource_id"
        }));

        let declaration = &forms_manifest().typed_reference_schemas[0];
        let installation_id = Uuid::new_v4();
        let reference = TypedResourceReference::new(
            installation_id,
            ResourceOwner::CoreInstallation { installation_id },
            id("tessara.forms.form-version"),
            "form-version-42",
        )
        .unwrap();
        declaration
            .validate_reference(&reference)
            .expect("reference conforms to its resource schema");
    }

    #[test]
    fn manifest_authority_rejects_foreign_and_core_namespaces() {
        let mut foreign_feature = forms_manifest();
        foreign_feature.features[0].id = id("other.forms.authoring");
        let error = foreign_feature
            .validate(&forms_authority())
            .expect_err("foreign Feature namespace must be rejected");
        assert_eq!(
            error
                .findings
                .iter()
                .map(|finding| (finding.code.as_str(), finding.path.as_str()))
                .collect::<Vec<_>>(),
            vec![("unauthorized_identifier_namespace", "features[0].id")]
        );

        let mut core_capability = forms_manifest();
        let core_capability_id: SecurityCapabilityId = id("admin:all");
        core_capability.features[0].capabilities = vec![core_capability_id.clone()];
        core_capability.navigation[0].required_capabilities_any_of =
            vec![core_capability_id.clone()];
        core_capability.security_capabilities[0].id = core_capability_id;
        let error = core_capability
            .validate(&forms_authority())
            .expect_err("Core capability namespace must be rejected");
        assert_eq!(error.findings.len(), 1);
        assert_eq!(error.findings[0].path, "security_capabilities[0].id");
        assert_eq!(error.findings[0].code, "unauthorized_identifier_namespace");

        assert_eq!(
            ManifestNamespaceAuthority::new(
                id("tessara.forms"),
                id("tessara.first_party"),
                ["admin"]
            ),
            Err(NamespaceAuthorityError::Reserved("admin".into()))
        );
    }

    #[test]
    fn valid_transition_descriptor_round_trips_and_validates() {
        let descriptor = forms_transition();
        descriptor.validate().expect("descriptor is valid");

        let json = serde_json::to_value(&descriptor).expect("serialize descriptor");
        assert_eq!(json["reserved_definition_id"], "tessara.forms");
        assert_eq!(json["availability"], "active_in_process");

        let decoded: TransitionalContributionDescriptorV1 =
            serde_json::from_value(json).expect("deserialize descriptor");
        assert_eq!(decoded, descriptor);
    }

    #[test]
    fn transition_descriptor_rejects_deployment_or_instance_claims() {
        for forbidden in ["deployment", "release", "instance"] {
            let mut json =
                serde_json::to_value(forms_transition()).expect("serialize transition descriptor");
            json[forbidden] = json!({"fabricated": true});
            assert!(
                serde_json::from_value::<TransitionalContributionDescriptorV1>(json).is_err(),
                "transition accepted forbidden field {forbidden}"
            );
        }
    }

    #[test]
    fn feature_links_and_duplicates_produce_stable_findings() {
        let mut descriptor = forms_transition();
        descriptor.features[0].contracts = vec![id("tessara.forms.missing")];
        descriptor
            .security_capabilities
            .push(descriptor.security_capabilities[0].clone());

        let error = descriptor.validate().expect_err("descriptor is invalid");
        assert_eq!(
            error
                .findings
                .iter()
                .map(|finding| finding.code.as_str())
                .collect::<Vec<_>>(),
            vec!["duplicate_identifier", "unresolved_feature_link"]
        );
    }

    #[test]
    fn duplicate_feature_realization_links_have_exact_indexed_findings() {
        let mut descriptor = forms_transition();
        let feature = &mut descriptor.features[0];
        feature.contracts.push(feature.contracts[0].clone());
        feature
            .resource_types
            .push(feature.resource_types[0].clone());
        feature.destinations.push(feature.destinations[0].clone());
        feature.capabilities.push(feature.capabilities[0].clone());

        let error = descriptor.validate().expect_err("descriptor is invalid");
        assert_eq!(
            error
                .findings
                .iter()
                .map(|finding| (finding.code.as_str(), finding.path.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("duplicate_identifier", "features[0].contracts[1]"),
                ("duplicate_identifier", "features[0].resource_types[1]"),
                ("duplicate_identifier", "features[0].destinations[1]"),
                ("duplicate_identifier", "features[0].capabilities[2]"),
            ]
        );
    }

    #[test]
    fn retired_transition_rejects_every_declaration_dimension_in_order() {
        let mut descriptor = forms_transition();
        descriptor.availability = TransitionAvailability::Retired;
        descriptor.dependencies = vec![FunctionalDependency {
            contract_id: id("tessara.workflows.workflow-version"),
            version_requirement: VersionReq::parse("^1.0").unwrap(),
            binding_key: id("tessara.forms.workflow-version"),
            optional: false,
        }];
        descriptor.configuration_schema = Some(json!({"type": "object"}));

        let error = descriptor
            .validate()
            .expect_err("retired source is not empty");
        assert_eq!(
            error
                .findings
                .iter()
                .map(|finding| (finding.code.as_str(), finding.path.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("retired_transition_declaration", "features"),
                ("retired_transition_declaration", "provided_contracts"),
                ("retired_transition_declaration", "dependencies"),
                ("retired_transition_declaration", "resource_types"),
                ("retired_transition_declaration", "routes"),
                ("retired_transition_declaration", "navigation"),
                ("retired_transition_declaration", "security_capabilities"),
                ("retired_transition_declaration", "configuration_schema"),
            ]
        );
    }

    #[test]
    fn navigation_any_of_capabilities_have_stable_reference_and_duplicate_findings() {
        let mut descriptor = forms_transition();
        descriptor.navigation[0].required_capabilities_any_of = vec![
            id("forms:read"),
            id("forms:read"),
            id("forms:missing-read"),
            id("forms:missing-manage"),
        ];

        let error = descriptor.validate().expect_err("descriptor is invalid");
        assert_eq!(
            error
                .findings
                .iter()
                .map(|finding| (finding.code.as_str(), finding.path.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (
                    "duplicate_identifier",
                    "navigation[0].required_capabilities_any_of[1]"
                ),
                (
                    "unresolved_navigation_capability",
                    "navigation[0].required_capabilities_any_of[2]"
                ),
                (
                    "unresolved_navigation_capability",
                    "navigation[0].required_capabilities_any_of[3]"
                ),
            ]
        );

        descriptor.navigation[0]
            .required_capabilities_any_of
            .clear();
        let error = descriptor
            .validate()
            .expect_err("a contribution must declare visibility capability");
        assert_eq!(
            error
                .findings
                .iter()
                .map(|finding| (finding.code.as_str(), finding.path.as_str()))
                .collect::<Vec<_>>(),
            vec![(
                "missing_navigation_capability",
                "navigation[0].required_capabilities_any_of"
            )]
        );
    }

    #[test]
    fn navigation_any_of_lists_represent_current_product_visibility_exactly() {
        let make_navigation =
            |identifier: &str,
             destination: &str,
             label: &str,
             group: &str,
             order_hint: i32,
             capabilities: &[&str]| NavigationContribution {
                id: id(identifier),
                destination: id(destination),
                label: label.into(),
                group: group.into(),
                order_hint,
                required_capabilities_any_of: capabilities
                    .iter()
                    .map(|capability| id(capability))
                    .collect(),
            };
        let contributions = vec![
            make_navigation(
                "tessara.forms.navigation",
                "forms.directory",
                "Forms",
                "Main",
                20,
                &["forms:read", "forms:manage"],
            ),
            make_navigation(
                "tessara.workflows.navigation",
                "workflows.directory",
                "Workflows",
                "Main",
                30,
                &["workflows:read", "workflows:manage"],
            ),
            make_navigation(
                "tessara.responses.navigation",
                "responses.directory",
                "Responses",
                "Main",
                40,
                &[
                    "submissions:read_own",
                    "submissions:respond",
                    "submissions:manage",
                ],
            ),
            make_navigation(
                "tessara.components.navigation",
                "components.directory",
                "Components",
                "Main",
                60,
                &["components:read", "components:manage"],
            ),
            make_navigation(
                "tessara.dashboards.navigation",
                "dashboards.directory",
                "Dashboards",
                "Main",
                70,
                &["dashboards:read"],
            ),
            make_navigation(
                "tessara.datasets.navigation",
                "datasets.directory",
                "Datasets",
                "Admin",
                20,
                &["datasets:read", "datasets:manage"],
            ),
        ];

        let represented = contributions
            .iter()
            .map(|contribution| {
                (
                    contribution.label.as_str(),
                    contribution
                        .required_capabilities_any_of
                        .iter()
                        .map(|capability| capability.as_str())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            represented,
            BTreeMap::from([
                ("Components", vec!["components:read", "components:manage"]),
                ("Dashboards", vec!["dashboards:read"]),
                ("Datasets", vec!["datasets:read", "datasets:manage"]),
                ("Forms", vec!["forms:read", "forms:manage"]),
                (
                    "Responses",
                    vec![
                        "submissions:read_own",
                        "submissions:respond",
                        "submissions:manage"
                    ]
                ),
                ("Workflows", vec!["workflows:read", "workflows:manage"]),
            ])
        );
        assert!(contributions.iter().all(|contribution| {
            contribution
                .required_capabilities_any_of
                .iter()
                .all(|capability| capability.as_str() != "admin:all")
        }));

        let wire = serde_json::to_value(&contributions).expect("serialize navigation");
        assert!(wire.as_array().unwrap().iter().all(|contribution| {
            contribution.get("required_capabilities_any_of").is_some()
                && contribution.get("required_capability").is_none()
        }));
        assert_eq!(
            serde_json::from_value::<Vec<NavigationContribution>>(wire)
                .expect("deserialize navigation"),
            contributions
        );
    }

    #[test]
    fn transition_inventory_cannot_masquerade_as_deployable_or_provider_inventory() {
        let entry = InventoryEntry::TransitionalInProcess {
            descriptor: forms_transition(),
        };

        assert!(!entry.provider_eligible());
        assert!(!entry.supervisor_materializable());
        assert!(entry.release().is_none());
        assert!(entry.instance().is_none());

        let json = serde_json::to_value(&entry).expect("serialize inventory entry");
        assert_eq!(json["kind"], "transitional_in_process");
        assert!(json.get("release").is_none());
        assert!(json.get("instance").is_none());
        let descriptor = json
            .get("descriptor")
            .and_then(Value::as_object)
            .expect("transition descriptor object");
        for forbidden in [
            "deployment",
            "runtime_image",
            "installed",
            "enabled",
            "healthy",
        ] {
            assert!(
                descriptor.get(forbidden).is_none(),
                "unexpected transition field {forbidden}"
            );
        }

        let mut hostile = json;
        hostile["release"] = json!({"fabricated": true});
        hostile["instance"] = json!({"fabricated": true});
        assert!(serde_json::from_value::<InventoryEntry>(hostile).is_err());
    }

    #[test]
    fn real_inventory_requires_connected_identity_and_eligible_state() {
        let entry = forms_inventory();
        entry.validate_integrity().expect("inventory is connected");
        assert!(entry.provider_eligible());
        assert!(entry.supervisor_materializable());

        let mut cross_wired = forms_inventory();
        let InventoryEntry::ModuleInstance { release, .. } = &mut cross_wired else {
            panic!("real module inventory");
        };
        release.definition_id = id("tessara.datasets");
        assert_eq!(
            cross_wired.validate_integrity(),
            Err(InventoryIntegrityError::DefinitionReleaseMismatch)
        );
        assert!(!cross_wired.provider_eligible());
        assert!(!cross_wired.supervisor_materializable());

        let mut ineligible = forms_inventory();
        let InventoryEntry::ModuleInstance { instance, .. } = &mut ineligible else {
            panic!("real module inventory");
        };
        instance.operation_state.installed = false;
        instance.data_state = InstanceDataState::Destroyed;
        assert!(!ineligible.provider_eligible());
    }

    #[test]
    fn dependency_binding_keys_are_unique() {
        let mut descriptor = forms_transition();
        let dependency = FunctionalDependency {
            contract_id: id("tessara.datasets.major-line"),
            version_requirement: VersionReq::parse("^1").unwrap(),
            binding_key: id("tessara.forms.dataset-provider"),
            optional: false,
        };
        descriptor.dependencies = vec![dependency.clone(), dependency];

        let error = descriptor.validate().expect_err("descriptor is invalid");
        assert_eq!(error.findings.len(), 1);
        assert_eq!(error.findings[0].code, "duplicate_identifier");
        assert_eq!(error.findings[0].path, "dependencies[1]");
    }

    #[test]
    fn typed_core_reference_is_installation_bound() {
        let installation_id = Uuid::new_v4();
        let reference = TypedResourceReference::new(
            installation_id,
            ResourceOwner::CoreInstallation { installation_id },
            id("tessara.transition.form_version"),
            "form-version-42",
        )
        .expect("reference is valid");
        reference.validate().expect("reference is valid");

        let mismatched = TypedResourceReference::new(
            installation_id,
            ResourceOwner::CoreInstallation {
                installation_id: Uuid::new_v4(),
            },
            id("tessara.transition.form_version"),
            "form-version-42",
        );
        assert_eq!(
            mismatched,
            Err(ReferenceValidationError::InstallationMismatch)
        );
    }

    #[test]
    fn typed_references_reject_invalid_json_before_exposure() {
        let installation_id = Uuid::new_v4();
        let other_installation_id = Uuid::new_v4();
        let resource_type = "tessara.forms.form-version";

        let empty_id = json!({
            "installation_id": installation_id,
            "owner": {
                "kind": "core_installation",
                "installation_id": installation_id
            },
            "resource_type": resource_type,
            "resource_id": "  "
        });
        assert!(serde_json::from_value::<TypedResourceReference>(empty_id).is_err());

        let mismatched_module_owner = json!({
            "installation_id": installation_id,
            "owner": {
                "kind": "module_instance",
                "installation_id": other_installation_id,
                "module_instance_id": Uuid::new_v4()
            },
            "resource_type": resource_type,
            "resource_id": "form-version-42"
        });
        assert!(serde_json::from_value::<TypedResourceReference>(mismatched_module_owner).is_err());
    }

    #[test]
    fn module_owned_reference_requires_authoritative_instance_binding() {
        let InventoryEntry::ModuleInstance { instance, .. } = forms_inventory() else {
            panic!("real module inventory");
        };
        let reference = TypedResourceReference::new(
            instance.installation_id,
            ResourceOwner::ModuleInstance {
                installation_id: instance.installation_id,
                module_instance_id: instance.id,
            },
            id("tessara.forms.form-version"),
            "form-version-42",
        )
        .expect("structurally valid reference");
        reference
            .validate_module_instance_binding(&instance)
            .expect("reference matches authoritative instance");

        let mut other_instance = instance;
        other_instance.id = Uuid::new_v4();
        assert_eq!(
            reference.validate_module_instance_binding(&other_instance),
            Err(ReferenceValidationError::ModuleInstanceMismatch)
        );
    }

    #[test]
    fn resource_resolution_dimensions_are_independent() {
        let resolution = ResourceResolutionV1::authorized(
            ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::OwnerModuleInstanceTombstoned,
                data_state: OwnerDataState::OwnerDataDestroyed,
            },
            ResourceIdentityState::Resolved,
            ResourceLifecycleState::ProviderDefined {
                state: "active".into(),
            },
            ContractCompatibilityState::Incompatible,
            ProviderAvailabilityState::Unavailable,
        )
        .expect("authorized dimensions are disclosed");

        assert_eq!(resolution.schema_version(), CONTRACT_SCHEMA_VERSION_V1);
        assert_eq!(resolution.access_state(), ResourceAccessState::Authorized);
        assert_eq!(
            resolution.owner_state(),
            ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::OwnerModuleInstanceTombstoned,
                data_state: OwnerDataState::OwnerDataDestroyed,
            }
        );
        assert_eq!(
            resolution.resource_identity_state(),
            ResourceIdentityState::Resolved
        );
        assert_eq!(
            resolution.resource_lifecycle_state(),
            &ResourceLifecycleState::ProviderDefined {
                state: "active".into(),
            }
        );
        assert_eq!(
            resolution.compatibility_state(),
            ContractCompatibilityState::Incompatible
        );
        assert_eq!(
            resolution.availability_state(),
            ProviderAvailabilityState::Unavailable
        );

        let wire = serde_json::to_value(&resolution).expect("serialize resource resolution");
        assert_eq!(
            wire,
            json!({
                "schema_version": 1,
                "access_state": "authorized",
                "owner_state": {
                    "kind": "module_instance",
                    "instance_state": "owner_module_instance_tombstoned",
                    "data_state": "owner_data_destroyed"
                },
                "resource_identity_state": "resolved",
                "resource_lifecycle_state": {
                    "kind": "provider_defined",
                    "state": "active"
                },
                "compatibility_state": "incompatible",
                "availability_state": "unavailable"
            })
        );
        assert_eq!(
            serde_json::from_value::<ResourceResolutionV1>(wire)
                .expect("deserialize resource resolution"),
            resolution
        );
    }

    #[test]
    fn authorized_resource_resolution_rejects_every_undisclosed_dimension() {
        let expected = Err(ResourceResolutionValidationError::AuthorizedEnvelopeUndisclosedState);
        let disclosed_owner = ResourceOwnerState::CoreInstallation {
            state: CoreInstallationOwnerState::Live,
        };

        assert_eq!(
            ResourceResolutionV1::authorized(
                ResourceOwnerState::Undisclosed,
                ResourceIdentityState::Resolved,
                ResourceLifecycleState::ProviderDefined {
                    state: "active".into(),
                },
                ContractCompatibilityState::Compatible,
                ProviderAvailabilityState::Available,
            ),
            expected
        );
        assert_eq!(
            ResourceResolutionV1::authorized(
                disclosed_owner,
                ResourceIdentityState::Undisclosed,
                ResourceLifecycleState::ProviderDefined {
                    state: "active".into(),
                },
                ContractCompatibilityState::Compatible,
                ProviderAvailabilityState::Available,
            ),
            expected
        );
        assert_eq!(
            ResourceResolutionV1::authorized(
                disclosed_owner,
                ResourceIdentityState::Resolved,
                ResourceLifecycleState::Undisclosed,
                ContractCompatibilityState::Compatible,
                ProviderAvailabilityState::Available,
            ),
            expected
        );
        assert_eq!(
            ResourceResolutionV1::authorized(
                disclosed_owner,
                ResourceIdentityState::Resolved,
                ResourceLifecycleState::ProviderDefined {
                    state: "active".into(),
                },
                ContractCompatibilityState::Undisclosed,
                ProviderAvailabilityState::Available,
            ),
            expected
        );
        assert_eq!(
            ResourceResolutionV1::authorized(
                disclosed_owner,
                ResourceIdentityState::Resolved,
                ResourceLifecycleState::ProviderDefined {
                    state: "active".into(),
                },
                ContractCompatibilityState::Compatible,
                ProviderAvailabilityState::Undisclosed,
            ),
            expected
        );
    }

    #[test]
    fn restricted_resource_resolution_projection_is_stable_and_non_disclosing() {
        let detailed = ResourceResolutionV1::authorized(
            ResourceOwnerState::CoreInstallation {
                state: CoreInstallationOwnerState::Live,
            },
            ResourceIdentityState::Resolved,
            ResourceLifecycleState::ProviderDefined {
                state: "archived".into(),
            },
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        )
        .expect("authorized dimensions are disclosed");

        let unauthorized = detailed
            .restricted_projection(ResourceAccessState::Unauthorized)
            .expect("unauthorized is a restricted access state");
        let not_evaluated = ResourceResolutionV1::restricted(ResourceAccessState::NotEvaluated)
            .expect("not evaluated is a restricted access state");
        let unauthorized_wire =
            serde_json::to_value(&unauthorized).expect("serialize unauthorized projection");
        let not_evaluated_wire =
            serde_json::to_value(&not_evaluated).expect("serialize not-evaluated projection");

        assert_eq!(
            unauthorized_wire,
            json!({
                "schema_version": 1,
                "access_state": "unauthorized",
                "owner_state": { "kind": "undisclosed" },
                "resource_identity_state": "undisclosed",
                "resource_lifecycle_state": { "kind": "undisclosed" },
                "compatibility_state": "undisclosed",
                "availability_state": "undisclosed"
            })
        );
        assert_eq!(
            not_evaluated_wire,
            json!({
                "schema_version": 1,
                "access_state": "not_evaluated",
                "owner_state": { "kind": "undisclosed" },
                "resource_identity_state": "undisclosed",
                "resource_lifecycle_state": { "kind": "undisclosed" },
                "compatibility_state": "undisclosed",
                "availability_state": "undisclosed"
            })
        );

        assert_eq!(
            ResourceResolutionV1::restricted(ResourceAccessState::Authorized),
            Err(ResourceResolutionValidationError::AuthorizedRestrictedProjection)
        );

        let mut disclosing_restricted =
            serde_json::to_value(detailed).expect("serialize detailed resolution");
        disclosing_restricted["access_state"] = json!("unauthorized");
        assert!(
            serde_json::from_value::<ResourceResolutionV1>(disclosing_restricted).is_err(),
            "deserialization must reject restricted envelopes that disclose detail"
        );
    }

    #[test]
    fn semantic_destination_contains_no_deployment_url_field() {
        let destination = SemanticDestination {
            owner: ResourceOwner::CoreInstallation {
                installation_id: Uuid::new_v4(),
            },
            route: id("forms.detail"),
            parameters: BTreeMap::from([(
                "form_id".into(),
                SemanticParameterValue::String("42".into()),
            )]),
        };
        let json = serde_json::to_value(destination).expect("serialize destination");
        assert!(json.get("url").is_none());
        assert!(json.get("host").is_none());
        assert!(json.get("port").is_none());

        let mut hostile = json;
        hostile["url"] = json!("https://attacker.example/forms/42");
        assert!(serde_json::from_value::<SemanticDestination>(hostile).is_err());
    }

    #[test]
    fn semantic_destination_parameters_are_route_bound_and_typed() {
        let form_id = Uuid::new_v4();
        let route = RouteDeclaration {
            name: id("forms.detail"),
            kind: RouteKind::Product,
            parameters: vec![
                RouteParameterDeclaration {
                    name: "form_id".into(),
                    value_type: RouteParameterType::Uuid,
                    required: true,
                },
                RouteParameterDeclaration {
                    name: "mode".into(),
                    value_type: RouteParameterType::String,
                    required: false,
                },
            ],
        };
        let mut destination = SemanticDestination {
            owner: ResourceOwner::CoreInstallation {
                installation_id: Uuid::new_v4(),
            },
            route: id("forms.detail"),
            parameters: BTreeMap::from([("form_id".into(), SemanticParameterValue::Uuid(form_id))]),
        };
        destination
            .validate_against(&route)
            .expect("destination matches route");

        destination.parameters.insert(
            "form_id".into(),
            SemanticParameterValue::String(form_id.to_string()),
        );
        assert_eq!(
            destination.validate_against(&route),
            Err(SemanticDestinationValidationError::ParameterTypeMismatch(
                "form_id".into()
            ))
        );

        destination.parameters.remove("form_id");
        assert_eq!(
            destination.validate_against(&route),
            Err(SemanticDestinationValidationError::MissingRequiredParameter("form_id".into()))
        );
    }

    #[test]
    fn deployment_profile_and_nested_wire_shapes_are_strict() {
        let manifest = forms_manifest();
        let mut json = serde_json::to_value(manifest).expect("serialize manifest");
        json["deployment"]["profile"] = json!("tessara-oci-v2");
        assert!(serde_json::from_value::<ModuleManifest>(json).is_err());

        let mut json = serde_json::to_value(forms_manifest()).expect("serialize manifest");
        json["deployment"]["declaration"]["listen"]["host"] = json!("attacker.example");
        assert!(serde_json::from_value::<ModuleManifest>(json).is_err());
    }

    #[test]
    fn oci_image_references_are_non_url_and_pinned_to_the_declared_digest() {
        let digest = ArtifactDigest::new(format!("sha256:{}", "a".repeat(64))).unwrap();
        let valid = format!("registry.example/tessara/forms@{digest}");
        assert!(valid_oci_image_reference(&valid, &digest));

        for invalid in [
            "registry.example/tessara/forms:latest".to_string(),
            format!("registry.example/tessara/forms@sha256:{}", "b".repeat(64)),
            format!("https://registry.example/tessara/forms@{digest}"),
            format!("registry.example/tessara/../forms@{digest}"),
        ] {
            assert!(
                !valid_oci_image_reference(&invalid, &digest),
                "unexpected valid OCI reference: {invalid}"
            );
        }
    }

    #[test]
    fn deployment_validation_rejects_unsafe_or_incomplete_fields() {
        let mut manifest = forms_manifest();
        let DeploymentProfile::TessaraOciV1(deployment) = &mut manifest.deployment;
        deployment.runtime_image.image_reference = format!(
            "https://registry.example/tessara/forms@{}",
            deployment.runtime_image.digest
        );
        deployment.runtime_image.platform = String::new();
        deployment.migration_image.as_mut().unwrap().command.clear();
        deployment.listen.protocol = "https".into();
        deployment.listen.port = 0;
        deployment.configuration_keys = vec!["SHARED".into(), "SHARED".into()];
        deployment.secret_keys = vec!["SHARED".into()];
        deployment.migration_identity = deployment.runtime_identity.clone();
        deployment.readiness_path = "//attacker.example/ready".into();
        deployment.liveness_path = "/health/../secret".into();
        deployment.graceful_shutdown_seconds = 0;
        deployment.resources.cpu_request_millis = 0;

        let error = manifest
            .validate(&forms_authority())
            .expect_err("deployment is invalid");
        let codes = error
            .findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<BTreeSet<_>>();
        for expected in [
            "invalid_oci_image_reference",
            "unsupported_image_platform",
            "missing_migration_command",
            "unsupported_service_protocol",
            "invalid_service_port",
            "duplicate_identifier",
            "configuration_secret_key_overlap",
            "shared_runtime_migration_identity",
            "invalid_probe_path",
            "invalid_graceful_shutdown",
            "non_positive_resource_requirement",
        ] {
            assert!(codes.contains(expected), "missing finding {expected}");
        }
    }

    #[test]
    fn manifest_validation_rejects_bad_deployment_and_unresolved_navigation() {
        let mut manifest = forms_manifest();
        manifest.navigation[0].destination = id("forms.missing");
        let DeploymentProfile::TessaraOciV1(deployment) = &mut manifest.deployment;
        deployment.runtime_image.command.clear();
        deployment.readiness_path = "ready".into();
        deployment.resources.cpu_request_millis = 500;
        deployment.resources.cpu_limit_millis = 250;
        deployment.resources.memory_request_mebibytes = 256;
        deployment.resources.memory_limit_mebibytes = 128;

        let error = manifest
            .validate(&forms_authority())
            .expect_err("manifest is invalid");
        assert_eq!(
            error.findings,
            vec![
                ValidationFinding {
                    code: "unresolved_navigation_destination".into(),
                    path: "navigation[0].destination".into(),
                    message: "route 'forms.missing' is not declared by this document".into(),
                },
                ValidationFinding {
                    code: "missing_runtime_command".into(),
                    path: "deployment.declaration.runtime_image.command".into(),
                    message: "runtime image must declare a command".into(),
                },
                ValidationFinding {
                    code: "invalid_probe_path".into(),
                    path: "deployment.declaration.readiness_path".into(),
                    message: "probe path must be a local absolute path without an authority, query, fragment, or traversal segment".into(),
                },
                ValidationFinding {
                    code: "invalid_resource_limits".into(),
                    path: "deployment.declaration.resources".into(),
                    message: "resource limits must be greater than or equal to requests".into(),
                },
            ]
        );
    }

    #[test]
    fn manifest_validation_rejects_configuration_fields_the_shared_ui_cannot_manage() {
        let mut manifest = forms_manifest();
        manifest.configuration_schema = json!({
            "type": "object",
            "properties": {
                "supported": {"type": "boolean"},
                "unsupported": {"type": "array"}
            }
        });

        let error = manifest
            .validate(&forms_authority())
            .expect_err("unsupported managed configuration is invalid");
        assert!(error.findings.iter().any(|finding| {
            finding.code == "unsupported_configuration_field_type"
                && finding.path == "configuration_schema.properties.unsupported.type"
        }));
    }

    #[test]
    fn browser_lifecycle_v1_requires_declared_typed_assets_and_document_fallback() {
        let mut manifest = forms_manifest();
        manifest.browser_routes.push(BrowserRouteDeclaration {
            destination: manifest.routes[0].name.clone(),
            path_template: "/forms".into(),
            methods: vec![BrowserDocumentMethod::Get],
            required_capability: manifest.security_capabilities[0].id.clone(),
            authorization_action: "forms.list".into(),
            dependency_binding: id("tessara.core.forms"),
            functional_contract: manifest.provided_contracts[0].id.clone(),
            organization_scope_parameter: None,
        });
        manifest.assets = vec![
            ModuleAssetDeclaration {
                path: "/module.js".into(),
                digest: ArtifactDigest::new(format!("sha256:{}", "a".repeat(64))).unwrap(),
                content_type: "text/javascript; charset=utf-8".into(),
            },
            ModuleAssetDeclaration {
                path: "/module.css".into(),
                digest: ArtifactDigest::new(format!("sha256:{}", "b".repeat(64))).unwrap(),
                content_type: "text/css; charset=utf-8".into(),
            },
        ];
        manifest.browser_lifecycle = Some(BrowserLifecycleDeclaration {
            lifecycle_abi: Version::new(1, 0, 0),
            entry_asset: "/module.js".into(),
            stylesheet_assets: vec!["/module.css".into()],
            complete_document_fallback: true,
            capabilities: BrowserLifecycleCapabilities {
                navigation_guard: true,
                suspend_resume: true,
            },
        });

        let mut findings = Vec::new();
        validate_browser_lifecycle(&manifest, &mut findings);
        assert!(findings.is_empty(), "{findings:?}");

        let lifecycle = manifest.browser_lifecycle.as_mut().unwrap();
        lifecycle.entry_asset = "/module.css".into();
        lifecycle.complete_document_fallback = false;
        lifecycle.stylesheet_assets.push("/missing.css".into());
        validate_browser_lifecycle(&manifest, &mut findings);
        let codes = findings
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<BTreeSet<_>>();
        assert!(codes.contains("invalid_browser_lifecycle_entry_asset"));
        assert!(codes.contains("invalid_browser_lifecycle_stylesheet_asset"));
        assert!(codes.contains("missing_complete_document_fallback"));
    }

    #[test]
    fn browser_lifecycle_bootstrap_is_opaque_but_release_and_assets_are_fail_closed() {
        let digest = ArtifactDigest::new(format!("sha256:{}", "c".repeat(64))).unwrap();
        let bootstrap = BrowserLifecycleBootstrapV1 {
            schema_version: 1,
            definition_id: id("tessara.example"),
            release_version: Version::new(1, 2, 3),
            lifecycle_abi: Version::new(1, 0, 0),
            destination: id("example.directory"),
            path: "/example".into(),
            title: "Example".into(),
            document_state: ShellDocumentStateV1::Active,
            entry_asset: BrowserLifecycleAssetV1 {
                url: format!("/_tessara/modules/tessara.example/1.2.3/{digest}/module.js"),
                digest,
                content_type: "text/javascript; charset=utf-8".into(),
            },
            stylesheet_assets: Vec::new(),
            payload: json!({"module_owned": true}),
        };
        assert!(bootstrap.is_supported());

        let mut incompatible = bootstrap;
        incompatible.entry_asset.url = "https://example.invalid/module.js".into();
        assert!(!incompatible.is_supported());
    }
}
