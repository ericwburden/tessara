use std::collections::BTreeSet;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    CONTRACT_SCHEMA_VERSION_V1, DependencyBindingKey, FunctionalContractId, ModuleDefinitionId,
    NavigationContributionId, ResourceTypeId, SecurityCapabilityId, ServiceActionMethod,
};

pub const SHELL_CONTEXT_MAX_LIFETIME_SECONDS: i64 = 60;
pub const SHELL_CONTEXT_SCHEMA_VERSION_V2: u16 = 2;
pub const AUTHORIZATION_READ_MAX_LIFETIME_SECONDS: i64 = 60;
pub const AUTHORIZATION_MUTATION_MAX_LIFETIME_SECONDS: i64 = 30;
pub const AUTHORIZATION_GRANT_SCHEMA_VERSION_V2: u16 = 2;
pub const AUTHORIZATION_GRANT_SCHEMA_VERSION_V3: u16 = 3;
pub const MODULE_SERVICE_REQUEST_MAX_LIFETIME_SECONDS: i64 = 30;
pub const MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1: u16 = 1;
pub const MODULE_PROVIDER_COMPATIBILITY_PATH: &str = "/api/private/module-provider-compatibility";
pub const MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE: &str =
    "application/vnd.tessara.module-provider-compatibility+json;version=1";
pub const MODULE_PROVIDER_COMPATIBILITY_SERVICE_CONTEXT: &str =
    "tessara.module-provider-compatibility/v1";
pub const MODULE_PROVIDER_COMPATIBILITY_MAX_LIFETIME_SECONDS: i64 = 30;
pub const MODULE_PROVIDER_COMPATIBILITY_MAX_EXPECTATIONS: usize = 16;
pub const AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V1: u16 = 1;
pub const AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2: u16 = 2;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolSignaturePurposeV1 {
    ShellContext,
    AuthorizationGrant,
    ModuleServiceRequest,
    ProviderCompatibilityResponse,
    EnrollmentEligibility,
    EnrollmentRedemption,
    RecoveryOperatorAuthorization,
    FixtureExternalIdentity,
    ReleaseCatalog,
    ResolvedComposition,
    ApplyAuthorization,
    OwnerBootstrapAuthorization,
    OwnerBootstrapReceipt,
    BootstrapValidationAuthorization,
    SupervisorRequest,
    SupervisorResponse,
    InstallationReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedEnvelopeV1<T> {
    pub schema_version: u16,
    pub issuer: String,
    pub key_id: String,
    pub purpose: ProtocolSignaturePurposeV1,
    pub payload: T,
    pub signature: String,
}

#[derive(Serialize)]
struct SigningInputV1<'a, T> {
    schema_version: u16,
    issuer: &'a str,
    key_id: &'a str,
    purpose: ProtocolSignaturePurposeV1,
    payload: &'a T,
}

pub struct PurposeBoundSigningKeyV1 {
    issuer: String,
    key_id: String,
    purpose: ProtocolSignaturePurposeV1,
    signing_key: SigningKey,
}

impl PurposeBoundSigningKeyV1 {
    pub fn from_secret_bytes(
        issuer: impl Into<String>,
        key_id: impl Into<String>,
        purpose: ProtocolSignaturePurposeV1,
        secret_key: [u8; 32],
    ) -> Result<Self, ProtocolEnvelopeError> {
        let issuer = required_identifier("issuer", issuer.into())?;
        let key_id = required_identifier("key_id", key_id.into())?;
        Ok(Self {
            issuer,
            key_id,
            purpose,
            signing_key: SigningKey::from_bytes(&secret_key),
        })
    }

    pub fn sign<T>(&self, payload: T) -> Result<SignedEnvelopeV1<T>, ProtocolEnvelopeError>
    where
        T: Serialize,
    {
        let bytes = canonical_protocol_signing_bytes(
            CONTRACT_SCHEMA_VERSION_V1,
            &self.issuer,
            &self.key_id,
            self.purpose,
            &payload,
        )?;
        let signature = self.signing_key.sign(&bytes);
        Ok(SignedEnvelopeV1 {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
            issuer: self.issuer.clone(),
            key_id: self.key_id.clone(),
            purpose: self.purpose,
            payload,
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        })
    }

    pub fn verifier(&self) -> PurposeBoundVerifyingKeyV1 {
        PurposeBoundVerifyingKeyV1 {
            issuer: self.issuer.clone(),
            key_id: self.key_id.clone(),
            purpose: self.purpose,
            verifying_key: self.signing_key.verifying_key(),
        }
    }
}

#[derive(Clone)]
pub struct PurposeBoundVerifyingKeyV1 {
    issuer: String,
    key_id: String,
    purpose: ProtocolSignaturePurposeV1,
    verifying_key: VerifyingKey,
}

impl PurposeBoundVerifyingKeyV1 {
    pub fn from_public_bytes(
        issuer: impl Into<String>,
        key_id: impl Into<String>,
        purpose: ProtocolSignaturePurposeV1,
        public_key: [u8; 32],
    ) -> Result<Self, ProtocolEnvelopeError> {
        let issuer = required_identifier("issuer", issuer.into())?;
        let key_id = required_identifier("key_id", key_id.into())?;
        let verifying_key = VerifyingKey::from_bytes(&public_key)
            .map_err(|_| ProtocolEnvelopeError::InvalidVerificationKey)?;
        Ok(Self {
            issuer,
            key_id,
            purpose,
            verifying_key,
        })
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    pub fn verify<T>(&self, envelope: &SignedEnvelopeV1<T>) -> Result<(), ProtocolEnvelopeError>
    where
        T: Serialize,
    {
        if envelope.schema_version != CONTRACT_SCHEMA_VERSION_V1 {
            return Err(ProtocolEnvelopeError::UnsupportedSchemaVersion(
                envelope.schema_version,
            ));
        }
        if envelope.issuer != self.issuer {
            return Err(ProtocolEnvelopeError::WrongIssuer);
        }
        if envelope.key_id != self.key_id {
            return Err(ProtocolEnvelopeError::WrongKeyId);
        }
        if envelope.purpose != self.purpose {
            return Err(ProtocolEnvelopeError::WrongPurpose);
        }
        let signature_bytes = URL_SAFE_NO_PAD
            .decode(&envelope.signature)
            .map_err(|_| ProtocolEnvelopeError::MalformedSignature)?;
        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|_| ProtocolEnvelopeError::MalformedSignature)?;
        let bytes = canonical_protocol_signing_bytes(
            envelope.schema_version,
            &envelope.issuer,
            &envelope.key_id,
            envelope.purpose,
            &envelope.payload,
        )?;
        self.verifying_key
            .verify(&bytes, &signature)
            .map_err(|_| ProtocolEnvelopeError::InvalidSignature)
    }
}

/// Produces the compact, recursively key-sorted JSON bytes signed by protocol envelopes.
///
/// Object keys use ascending Unicode scalar order, arrays retain declared order, and strings
/// and numbers use `serde_json`'s standard JSON encoding. The envelope signature field is not
/// included.
pub fn canonical_protocol_signing_bytes<T>(
    schema_version: u16,
    issuer: &str,
    key_id: &str,
    purpose: ProtocolSignaturePurposeV1,
    payload: &T,
) -> Result<Vec<u8>, ProtocolEnvelopeError>
where
    T: Serialize,
{
    serde_jcs::to_vec(&SigningInputV1 {
        schema_version,
        issuer,
        key_id,
        purpose,
        payload,
    })
    .map_err(|error| ProtocolEnvelopeError::Serialization(error.to_string()))
}

fn required_identifier(
    field: &'static str,
    value: String,
) -> Result<String, ProtocolEnvelopeError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 128 {
        return Err(ProtocolEnvelopeError::InvalidIdentifier(field));
    }
    Ok(trimmed.to_owned())
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ProtocolEnvelopeError {
    #[error("{0} must contain between 1 and 128 non-whitespace characters")]
    InvalidIdentifier(&'static str),
    #[error("protocol schema version {0} is unsupported")]
    UnsupportedSchemaVersion(u16),
    #[error("protocol envelope issuer does not match the trusted issuer")]
    WrongIssuer,
    #[error("protocol envelope key ID does not match the trusted key")]
    WrongKeyId,
    #[error("protocol envelope signature purpose does not match the trusted key purpose")]
    WrongPurpose,
    #[error("protocol envelope signature is malformed")]
    MalformedSignature,
    #[error("protocol envelope signature is invalid")]
    InvalidSignature,
    #[error("verification key is invalid")]
    InvalidVerificationKey,
    #[error("protocol envelope serialization failed: {0}")]
    Serialization(String),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellThemeV1 {
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellDocumentStateV1 {
    Active,
    Disabled,
    Degraded,
    StaleContext,
    Recovery,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OriginalActorProjectionV1 {
    pub actor_id: Uuid,
    pub display_name: String,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellNavigationItemProjectionV2 {
    pub contribution_id: NavigationContributionId,
    pub key: String,
    pub label: String,
    pub href: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellNavigationGroupProjectionV2 {
    pub id: String,
    pub label: String,
    pub items: Vec<ShellNavigationItemProjectionV2>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellContextV2 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub module_definition_id: ModuleDefinitionId,
    pub module_instance_id: Uuid,
    pub original_actor: OriginalActorProjectionV1,
    pub theme: ShellThemeV1,
    pub navigation: Vec<ShellNavigationGroupProjectionV2>,
    pub return_destination: String,
    pub locale: String,
    pub time_zone: String,
    pub correlation_id: Uuid,
    pub document_state: ShellDocumentStateV1,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellContextValidationContextV2 {
    pub installation_id: Uuid,
    pub module_definition_id: ModuleDefinitionId,
    pub module_instance_id: Uuid,
    pub correlation_id: Uuid,
    pub now: DateTime<Utc>,
}

impl ShellContextV2 {
    pub fn validate_for(
        &self,
        expected: &ShellContextValidationContextV2,
    ) -> Result<(), ShellContextValidationError> {
        if self.schema_version != SHELL_CONTEXT_SCHEMA_VERSION_V2 {
            return Err(ShellContextValidationError::UnsupportedSchemaVersion);
        }
        if self.installation_id != expected.installation_id {
            return Err(ShellContextValidationError::WrongInstallation);
        }
        if self.module_definition_id != expected.module_definition_id
            || self.module_instance_id != expected.module_instance_id
        {
            return Err(ShellContextValidationError::WrongAudience);
        }
        if self.correlation_id != expected.correlation_id {
            return Err(ShellContextValidationError::WrongCorrelation);
        }
        validate_window(
            self.issued_at,
            self.expires_at,
            expected.now,
            SHELL_CONTEXT_MAX_LIFETIME_SECONDS,
        )
        .map_err(ShellContextValidationError::Window)?;
        if self.original_actor.display_name.trim().is_empty()
            || self.return_destination.trim().is_empty()
            || self.locale.trim().is_empty()
            || self.time_zone.trim().is_empty()
        {
            return Err(ShellContextValidationError::MissingDisplayContext);
        }
        let mut group_ids = BTreeSet::new();
        let mut item_keys = BTreeSet::new();
        let mut contribution_ids = BTreeSet::new();
        for group in &self.navigation {
            if group.id.trim().is_empty()
                || group.label.trim().is_empty()
                || group.items.is_empty()
                || !group_ids.insert(group.id.as_str())
            {
                return Err(ShellContextValidationError::InvalidNavigation);
            }
            for item in &group.items {
                if item.key.trim().is_empty()
                    || item.label.trim().is_empty()
                    || !item.href.starts_with('/')
                    || item.href.starts_with("//")
                    || item.href.contains(['\r', '\n'])
                    || !item_keys.insert(item.key.as_str())
                    || !contribution_ids.insert(item.contribution_id.as_str())
                {
                    return Err(ShellContextValidationError::InvalidNavigation);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ShellContextValidationError {
    #[error("shell context schema version is unsupported")]
    UnsupportedSchemaVersion,
    #[error("shell context is bound to another installation")]
    WrongInstallation,
    #[error("shell context is bound to another module audience")]
    WrongAudience,
    #[error("shell context correlation binding does not match")]
    WrongCorrelation,
    #[error("shell context display projection is incomplete")]
    MissingDisplayContext,
    #[error("shell context navigation projection is invalid")]
    InvalidNavigation,
    #[error(transparent)]
    Window(#[from] SignedWindowError),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationGrantOperationV1 {
    Read,
    Mutation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityScopeBindingV1 {
    pub capability: SecurityCapabilityId,
    pub organization_root_id: Uuid,
    pub authorized_organization_ids: Vec<Uuid>,
}

impl CapabilityScopeBindingV1 {
    pub fn authorizes(&self, capability: &SecurityCapabilityId, organization_id: Uuid) -> bool {
        &self.capability == capability
            && (self.organization_root_id == organization_id
                || self.authorized_organization_ids.contains(&organization_id))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceAuthorizationAssertionV2 {
    pub resource_type: ResourceTypeId,
    pub resource_id: String,
    pub authority_revision: u64,
    pub governing_organization_ids: Vec<Uuid>,
}

impl ResourceAuthorizationAssertionV2 {
    fn validate(&self) -> Result<(), AuthorizationValidationError> {
        if self.resource_id.trim().is_empty()
            || self.authority_revision == 0
            || self.governing_organization_ids.is_empty()
        {
            return Err(AuthorizationValidationError::InvalidResourceAssertion);
        }
        if self
            .governing_organization_ids
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(AuthorizationValidationError::InvalidResourceAssertion);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DelegationBasisV1 {
    pub delegation_id: Uuid,
    pub delegated_by_actor_id: Uuid,
    pub capability: SecurityCapabilityId,
    pub organization_root_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationGrantV2 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub original_actor_id: Uuid,
    pub presenting_service: ModuleDefinitionId,
    pub audience_module_instance_id: Uuid,
    pub dependency_binding: DependencyBindingKey,
    pub functional_contract: FunctionalContractId,
    pub action: String,
    pub operation: AuthorizationGrantOperationV1,
    pub capability_scope_bindings: Vec<CapabilityScopeBindingV1>,
    pub resource_assertion: Option<ResourceAuthorizationAssertionV2>,
    pub delegation_basis: Vec<DelegationBasisV1>,
    pub authorization_revision: u64,
    pub organization_revision: u64,
    pub jti: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationValidationContextV2 {
    pub installation_id: Uuid,
    pub presenting_service: ModuleDefinitionId,
    pub audience_module_instance_id: Uuid,
    pub dependency_binding: DependencyBindingKey,
    pub functional_contract: FunctionalContractId,
    pub action: String,
    pub operation: AuthorizationGrantOperationV1,
    pub resource_assertion: Option<ResourceAuthorizationAssertionV2>,
    pub authorization_revision: u64,
    pub organization_revision: u64,
    pub now: DateTime<Utc>,
}

impl AuthorizationGrantV2 {
    pub fn validate_for(
        &self,
        expected: &AuthorizationValidationContextV2,
    ) -> Result<(), AuthorizationValidationError> {
        if self.schema_version != AUTHORIZATION_GRANT_SCHEMA_VERSION_V2 {
            return Err(AuthorizationValidationError::UnsupportedSchemaVersion);
        }
        if self.installation_id != expected.installation_id {
            return Err(AuthorizationValidationError::WrongInstallation);
        }
        if self.presenting_service != expected.presenting_service {
            return Err(AuthorizationValidationError::WrongPresentingService);
        }
        if self.audience_module_instance_id != expected.audience_module_instance_id {
            return Err(AuthorizationValidationError::WrongAudience);
        }
        if self.dependency_binding != expected.dependency_binding
            || self.functional_contract != expected.functional_contract
        {
            return Err(AuthorizationValidationError::WrongDeclaredContract);
        }
        if self.action != expected.action || self.operation != expected.operation {
            return Err(AuthorizationValidationError::WrongAction);
        }
        if self.resource_assertion != expected.resource_assertion {
            return Err(AuthorizationValidationError::StaleResourceAssertion);
        }
        if let Some(assertion) = &self.resource_assertion {
            assertion.validate()?;
        }
        if self.authorization_revision != expected.authorization_revision {
            return Err(AuthorizationValidationError::StaleAuthorizationRevision);
        }
        if self.organization_revision != expected.organization_revision {
            return Err(AuthorizationValidationError::StaleOrganizationRevision);
        }
        if self.jti.is_nil() {
            return Err(AuthorizationValidationError::MissingReplayIdentifier);
        }
        validate_capability_bindings(&self.capability_scope_bindings)?;
        let max_lifetime = match self.operation {
            AuthorizationGrantOperationV1::Read => AUTHORIZATION_READ_MAX_LIFETIME_SECONDS,
            AuthorizationGrantOperationV1::Mutation => AUTHORIZATION_MUTATION_MAX_LIFETIME_SECONDS,
        };
        validate_window(self.issued_at, self.expires_at, expected.now, max_lifetime)
            .map_err(AuthorizationValidationError::Window)
    }

    pub fn authorizes(&self, capability: &SecurityCapabilityId, organization_id: Uuid) -> bool {
        self.capability_scope_bindings
            .iter()
            .any(|binding| binding.authorizes(capability, organization_id))
    }
}

/// Exact service principal presenting an authorization grant to its audience.
/// A module principal always carries both its stable definition and its
/// installation-scoped instance identity; callers never derive one from the
/// other at the authorization boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModuleServicePrincipalV1 {
    CoreGateway,
    ModuleInstance {
        module_instance_id: Uuid,
        module_definition_id: ModuleDefinitionId,
    },
}

/// Authoritative provider audience for a current authorization grant.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuthorizationAudienceV1 {
    CoreInstallation {
        installation_id: Uuid,
    },
    ModuleInstance {
        module_instance_id: Uuid,
        module_definition_id: ModuleDefinitionId,
    },
}

impl AuthorizationAudienceV1 {
    pub fn is_valid_for_installation(&self, installation_id: Uuid) -> bool {
        match self {
            Self::CoreInstallation {
                installation_id: audience_installation_id,
            } => !audience_installation_id.is_nil() && *audience_installation_id == installation_id,
            Self::ModuleInstance {
                module_instance_id, ..
            } => !module_instance_id.is_nil(),
        }
    }
}

/// Current authorization grant. V3 binds the correlation, audience, and
/// presenting-service identities without changing the immutable historical V2
/// wire fixture.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationGrantV3 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub original_actor_id: Uuid,
    pub correlation_id: Uuid,
    pub presenting_service: ModuleServicePrincipalV1,
    pub audience: AuthorizationAudienceV1,
    pub dependency_binding: DependencyBindingKey,
    pub functional_contract: FunctionalContractId,
    pub action: String,
    pub operation: AuthorizationGrantOperationV1,
    pub capability_scope_bindings: Vec<CapabilityScopeBindingV1>,
    pub resource_assertion: Option<ResourceAuthorizationAssertionV2>,
    pub delegation_basis: Vec<DelegationBasisV1>,
    pub authorization_revision: u64,
    pub organization_revision: u64,
    pub jti: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationValidationContextV3 {
    pub installation_id: Uuid,
    pub correlation_id: Uuid,
    pub presenting_service: ModuleServicePrincipalV1,
    pub audience: AuthorizationAudienceV1,
    pub dependency_binding: DependencyBindingKey,
    pub functional_contract: FunctionalContractId,
    pub action: String,
    pub operation: AuthorizationGrantOperationV1,
    pub resource_assertion: Option<ResourceAuthorizationAssertionV2>,
    pub authorization_revision: u64,
    pub organization_revision: u64,
    pub now: DateTime<Utc>,
}

impl AuthorizationGrantV3 {
    pub fn validate_for(
        &self,
        expected: &AuthorizationValidationContextV3,
    ) -> Result<(), AuthorizationValidationError> {
        if self.schema_version != AUTHORIZATION_GRANT_SCHEMA_VERSION_V3 {
            return Err(AuthorizationValidationError::UnsupportedSchemaVersion);
        }
        if self.installation_id != expected.installation_id {
            return Err(AuthorizationValidationError::WrongInstallation);
        }
        if self.correlation_id.is_nil() || self.correlation_id != expected.correlation_id {
            return Err(AuthorizationValidationError::WrongCorrelation);
        }
        if self.presenting_service != expected.presenting_service {
            return Err(AuthorizationValidationError::WrongPresentingService);
        }
        if self.audience != expected.audience
            || !self
                .audience
                .is_valid_for_installation(self.installation_id)
        {
            return Err(AuthorizationValidationError::WrongAudience);
        }
        if self.dependency_binding != expected.dependency_binding
            || self.functional_contract != expected.functional_contract
        {
            return Err(AuthorizationValidationError::WrongDeclaredContract);
        }
        if self.action != expected.action || self.operation != expected.operation {
            return Err(AuthorizationValidationError::WrongAction);
        }
        if self.resource_assertion != expected.resource_assertion {
            return Err(AuthorizationValidationError::StaleResourceAssertion);
        }
        if let Some(assertion) = &self.resource_assertion {
            assertion.validate()?;
        }
        if self.authorization_revision != expected.authorization_revision {
            return Err(AuthorizationValidationError::StaleAuthorizationRevision);
        }
        if self.organization_revision != expected.organization_revision {
            return Err(AuthorizationValidationError::StaleOrganizationRevision);
        }
        if self.jti.is_nil() {
            return Err(AuthorizationValidationError::MissingReplayIdentifier);
        }
        validate_capability_bindings(&self.capability_scope_bindings)?;
        let max_lifetime = match self.operation {
            AuthorizationGrantOperationV1::Read => AUTHORIZATION_READ_MAX_LIFETIME_SECONDS,
            AuthorizationGrantOperationV1::Mutation => AUTHORIZATION_MUTATION_MAX_LIFETIME_SECONDS,
        };
        validate_window(self.issued_at, self.expires_at, expected.now, max_lifetime)
            .map_err(AuthorizationValidationError::Window)
    }

    pub fn authorizes(&self, capability: &SecurityCapabilityId, organization_id: Uuid) -> bool {
        self.capability_scope_bindings
            .iter()
            .any(|binding| binding.authorizes(capability, organization_id))
    }
}

/// Exact request for exchanging a verified inbound module grant for a
/// least-privilege grant addressed to one downstream module instance.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationExchangeRequestV1 {
    pub schema_version: u16,
    pub target_module_instance_id: Uuid,
    pub target_module_definition_id: ModuleDefinitionId,
    pub dependency_binding: DependencyBindingKey,
    pub functional_contract: FunctionalContractId,
    pub action: String,
    pub operation: AuthorizationGrantOperationV1,
    pub required_capability: SecurityCapabilityId,
    pub resource_assertion: Option<ResourceAuthorizationAssertionV2>,
}

impl AuthorizationExchangeRequestV1 {
    pub fn validate(&self) -> Result<(), AuthorizationExchangeValidationError> {
        if self.schema_version != AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V1 {
            return Err(AuthorizationExchangeValidationError::UnsupportedSchemaVersion);
        }
        if self.target_module_instance_id.is_nil() || self.action.trim().is_empty() {
            return Err(AuthorizationExchangeValidationError::InvalidTarget);
        }
        if let Some(assertion) = &self.resource_assertion {
            assertion
                .validate()
                .map_err(|_| AuthorizationExchangeValidationError::InvalidResourceAssertion)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationExchangeResponseV1 {
    pub schema_version: u16,
    pub authorization: SignedEnvelopeV1<AuthorizationGrantV2>,
}

/// Current exchange request. Operation and required capability are resolved
/// from the target provider's enrolled service-action declaration rather than
/// accepted from the caller.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationExchangeRequestV2 {
    pub schema_version: u16,
    pub target: AuthorizationAudienceV1,
    pub dependency_binding: DependencyBindingKey,
    pub functional_contract: FunctionalContractId,
    pub action: String,
    pub resource_assertion: Option<ResourceAuthorizationAssertionV2>,
}

impl AuthorizationExchangeRequestV2 {
    pub fn validate(&self) -> Result<(), AuthorizationExchangeValidationError> {
        if self.schema_version != AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2 {
            return Err(AuthorizationExchangeValidationError::UnsupportedSchemaVersion);
        }
        let target_valid = match &self.target {
            AuthorizationAudienceV1::CoreInstallation { installation_id } => {
                !installation_id.is_nil()
            }
            AuthorizationAudienceV1::ModuleInstance {
                module_instance_id, ..
            } => !module_instance_id.is_nil(),
        };
        if !target_valid || self.action.trim().is_empty() {
            return Err(AuthorizationExchangeValidationError::InvalidTarget);
        }
        if let Some(assertion) = &self.resource_assertion {
            assertion
                .validate()
                .map_err(|_| AuthorizationExchangeValidationError::InvalidResourceAssertion)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationExchangeResponseV2 {
    pub schema_version: u16,
    pub authorization: SignedEnvelopeV1<AuthorizationGrantV3>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AuthorizationExchangeValidationError {
    #[error("authorization exchange schema version is unsupported")]
    UnsupportedSchemaVersion,
    #[error("authorization exchange target or action is invalid")]
    InvalidTarget,
    #[error("authorization exchange resource assertion is invalid")]
    InvalidResourceAssertion,
}

fn validate_capability_bindings(
    bindings: &[CapabilityScopeBindingV1],
) -> Result<(), AuthorizationValidationError> {
    if bindings.is_empty() {
        return Err(AuthorizationValidationError::MissingCapabilityBindings);
    }
    for binding in bindings {
        let mut organizations = BTreeSet::new();
        organizations.insert(binding.organization_root_id);
        if binding
            .authorized_organization_ids
            .iter()
            .any(|organization_id| !organizations.insert(*organization_id))
        {
            return Err(AuthorizationValidationError::DuplicateOrganizationBinding);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleServiceRequestV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub module_instance_id: Uuid,
    pub module_definition_id: ModuleDefinitionId,
    pub method: String,
    pub path: String,
    pub canonical_body_digest: String,
    pub inbound_grant_digest: String,
    pub correlation_id: String,
    pub nonce: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleServiceRequestValidationContextV1 {
    pub installation_id: Uuid,
    pub module_instance_id: Uuid,
    pub module_definition_id: ModuleDefinitionId,
    pub method: String,
    pub path: String,
    pub canonical_body_digest: String,
    pub inbound_grant_digest: String,
    pub correlation_id: String,
    pub now: DateTime<Utc>,
}

/// One exact Core-owned action a module expects to consume through a declared
/// dependency binding. The canonical order prevents omitted, duplicated, or
/// broadened expectation sets from being signed accidentally.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleProviderCompatibilityExpectationV1 {
    pub dependency_binding: String,
    pub functional_contract: String,
    pub contract_version: String,
    pub authorization_action: String,
    pub method: ServiceActionMethod,
    pub path: String,
}

impl ModuleProviderCompatibilityExpectationV1 {
    fn validate(&self) -> Result<(), ModuleProviderCompatibilityError> {
        for value in [
            self.dependency_binding.as_str(),
            self.functional_contract.as_str(),
            self.authorization_action.as_str(),
        ] {
            if value.is_empty()
                || value.len() > 256
                || value.trim() != value
                || value.chars().any(char::is_whitespace)
            {
                return Err(ModuleProviderCompatibilityError::InvalidExpectation);
            }
        }
        if semver::Version::parse(&self.contract_version).is_err()
            || !self.path.starts_with('/')
            || self.path.len() > 512
            || self.path.contains(['?', '#'])
            || self.path.split('/').any(|segment| segment == "..")
        {
            return Err(ModuleProviderCompatibilityError::InvalidExpectation);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleProviderCompatibilityRequestV1 {
    pub schema_version: u16,
    pub expectations: Vec<ModuleProviderCompatibilityExpectationV1>,
}

impl ModuleProviderCompatibilityRequestV1 {
    pub fn validate(&self) -> Result<(), ModuleProviderCompatibilityError> {
        if self.schema_version != MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1
            || self.expectations.is_empty()
            || self.expectations.len() > MODULE_PROVIDER_COMPATIBILITY_MAX_EXPECTATIONS
        {
            return Err(ModuleProviderCompatibilityError::InvalidExpectationSet);
        }
        let mut previous = None;
        for expectation in &self.expectations {
            expectation.validate()?;
            if previous.is_some_and(|previous| previous >= expectation) {
                return Err(ModuleProviderCompatibilityError::InvalidExpectationSet);
            }
            previous = Some(expectation);
        }
        Ok(())
    }

    pub fn canonical_digest(&self) -> Result<String, ModuleProviderCompatibilityError> {
        self.validate()?;
        let bytes = serde_jcs::to_vec(self)
            .map_err(|_| ModuleProviderCompatibilityError::Canonicalization)?;
        Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleProviderCompatibilityResponseV1 {
    pub schema_version: u16,
    pub provider: AuthorizationAudienceV1,
    pub consumer: ModuleServicePrincipalV1,
    pub request_digest: String,
    pub expectations: Vec<ModuleProviderCompatibilityExpectationV1>,
    pub correlation_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl ModuleProviderCompatibilityResponseV1 {
    pub fn validate_for(
        &self,
        request: &ModuleProviderCompatibilityRequestV1,
        provider: &AuthorizationAudienceV1,
        consumer: &ModuleServicePrincipalV1,
        correlation_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<(), ModuleProviderCompatibilityError> {
        request.validate()?;
        if self.schema_version != MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1
            || &self.provider != provider
            || &self.consumer != consumer
            || self.expectations != request.expectations
            || self.request_digest != request.canonical_digest()?
            || correlation_id.is_nil()
            || self.correlation_id != correlation_id
            || self.expires_at <= self.issued_at
            || self.expires_at - self.issued_at
                > Duration::seconds(MODULE_PROVIDER_COMPATIBILITY_MAX_LIFETIME_SECONDS)
            || now < self.issued_at
            || now > self.expires_at
        {
            return Err(ModuleProviderCompatibilityError::ResponseMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ModuleProviderCompatibilityError {
    #[error("module provider compatibility expectation is invalid")]
    InvalidExpectation,
    #[error(
        "module provider compatibility expectations must be non-empty, bounded, unique, and canonical"
    )]
    InvalidExpectationSet,
    #[error("module provider compatibility identity could not be canonicalized")]
    Canonicalization,
    #[error("module provider compatibility response does not match the authenticated request")]
    ResponseMismatch,
}

/// One-use request proof emitted by Core when it calls a module-owned private
/// provider action. This is deliberately a different wire type from a module
/// service request: Core is the presenting service and therefore cannot claim
/// a synthetic Module Instance identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoreServiceRequestV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub method: String,
    pub path: String,
    pub canonical_body_digest: String,
    pub inbound_grant_digest: String,
    pub correlation_id: String,
    pub nonce: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreServiceRequestValidationContextV1 {
    pub installation_id: Uuid,
    pub method: String,
    pub path: String,
    pub canonical_body_digest: String,
    pub inbound_grant_digest: String,
    pub correlation_id: String,
    pub now: DateTime<Utc>,
}

impl ModuleServiceRequestV1 {
    pub fn validate_for(
        &self,
        expected: &ModuleServiceRequestValidationContextV1,
    ) -> Result<(), ModuleServiceRequestValidationError> {
        if self.schema_version != CONTRACT_SCHEMA_VERSION_V1 {
            return Err(ModuleServiceRequestValidationError::UnsupportedSchemaVersion);
        }
        if self.installation_id != expected.installation_id {
            return Err(ModuleServiceRequestValidationError::WrongInstallation);
        }
        if self.module_instance_id != expected.module_instance_id
            || self.module_definition_id != expected.module_definition_id
        {
            return Err(ModuleServiceRequestValidationError::WrongService);
        }
        if self.method != expected.method || self.path != expected.path {
            return Err(ModuleServiceRequestValidationError::WrongTarget);
        }
        if self.canonical_body_digest != expected.canonical_body_digest
            || self.inbound_grant_digest != expected.inbound_grant_digest
        {
            return Err(ModuleServiceRequestValidationError::WrongDigest);
        }
        if !is_sha256_digest(&self.canonical_body_digest)
            || !is_sha256_digest(&self.inbound_grant_digest)
        {
            return Err(ModuleServiceRequestValidationError::InvalidDigest);
        }
        if self.correlation_id.trim().is_empty() || self.nonce.is_nil() {
            return Err(ModuleServiceRequestValidationError::MissingReplayIdentity);
        }
        if self.correlation_id != expected.correlation_id {
            return Err(ModuleServiceRequestValidationError::WrongCorrelation);
        }
        validate_window(
            self.issued_at,
            self.expires_at,
            expected.now,
            MODULE_SERVICE_REQUEST_MAX_LIFETIME_SECONDS,
        )
        .map_err(ModuleServiceRequestValidationError::Window)
    }
}

impl CoreServiceRequestV1 {
    pub fn validate_for(
        &self,
        expected: &CoreServiceRequestValidationContextV1,
    ) -> Result<(), CoreServiceRequestValidationError> {
        if self.schema_version != CONTRACT_SCHEMA_VERSION_V1 {
            return Err(CoreServiceRequestValidationError::UnsupportedSchemaVersion);
        }
        if self.installation_id != expected.installation_id {
            return Err(CoreServiceRequestValidationError::WrongInstallation);
        }
        if self.method != expected.method || self.path != expected.path {
            return Err(CoreServiceRequestValidationError::WrongTarget);
        }
        if self.canonical_body_digest != expected.canonical_body_digest
            || self.inbound_grant_digest != expected.inbound_grant_digest
        {
            return Err(CoreServiceRequestValidationError::WrongDigest);
        }
        if !is_sha256_digest(&self.canonical_body_digest)
            || !is_sha256_digest(&self.inbound_grant_digest)
        {
            return Err(CoreServiceRequestValidationError::InvalidDigest);
        }
        if self.correlation_id.trim().is_empty() || self.nonce.is_nil() {
            return Err(CoreServiceRequestValidationError::MissingReplayIdentity);
        }
        if self.correlation_id != expected.correlation_id {
            return Err(CoreServiceRequestValidationError::WrongCorrelation);
        }
        validate_window(
            self.issued_at,
            self.expires_at,
            expected.now,
            MODULE_SERVICE_REQUEST_MAX_LIFETIME_SECONDS,
        )
        .map_err(CoreServiceRequestValidationError::Window)
    }
}

fn is_sha256_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AuthorizationValidationError {
    #[error("authorization grant schema version is unsupported")]
    UnsupportedSchemaVersion,
    #[error("authorization grant is bound to another installation")]
    WrongInstallation,
    #[error("authorization grant correlation identity does not match")]
    WrongCorrelation,
    #[error("authorization grant presenting service does not match")]
    WrongPresentingService,
    #[error("authorization grant audience does not match")]
    WrongAudience,
    #[error("authorization grant dependency or contract does not match")]
    WrongDeclaredContract,
    #[error("authorization grant action or operation does not match")]
    WrongAction,
    #[error("authorization revision is stale")]
    StaleAuthorizationRevision,
    #[error("organization revision is stale")]
    StaleOrganizationRevision,
    #[error("authorization grant is missing its replay identifier")]
    MissingReplayIdentifier,
    #[error("authorization grant has no capability/scope bindings")]
    MissingCapabilityBindings,
    #[error("authorization grant repeats an organization inside one capability binding")]
    DuplicateOrganizationBinding,
    #[error("authorization grant resource assertion is invalid")]
    InvalidResourceAssertion,
    #[error("authorization grant resource assertion is stale or targets another resource")]
    StaleResourceAssertion,
    #[error(transparent)]
    Window(#[from] SignedWindowError),
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ModuleServiceRequestValidationError {
    #[error("module service request schema version is unsupported")]
    UnsupportedSchemaVersion,
    #[error("module service request is bound to another installation")]
    WrongInstallation,
    #[error("module service request identity does not match")]
    WrongService,
    #[error("module service request target does not match")]
    WrongTarget,
    #[error("module service request digest does not match")]
    WrongDigest,
    #[error("module service request digest is not a SHA-256 hex digest")]
    InvalidDigest,
    #[error("module service request correlation identity does not match its grant")]
    WrongCorrelation,
    #[error("module service request is missing correlation or nonce identity")]
    MissingReplayIdentity,
    #[error(transparent)]
    Window(#[from] SignedWindowError),
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CoreServiceRequestValidationError {
    #[error("Core service request schema version is unsupported")]
    UnsupportedSchemaVersion,
    #[error("Core service request is bound to another installation")]
    WrongInstallation,
    #[error("Core service request target does not match")]
    WrongTarget,
    #[error("Core service request digest does not match")]
    WrongDigest,
    #[error("Core service request digest is not a SHA-256 hex digest")]
    InvalidDigest,
    #[error("Core service request correlation identity does not match its grant")]
    WrongCorrelation,
    #[error("Core service request is missing correlation or nonce identity")]
    MissingReplayIdentity,
    #[error(transparent)]
    Window(#[from] SignedWindowError),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalIdentityAssertionV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub audience: String,
    pub external_subject: String,
    pub email: String,
    pub display_name: String,
    pub nonce: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SignedWindowError {
    #[error("signed message expiry must be later than issuance")]
    InvalidOrder,
    #[error("signed message lifetime exceeds the protocol maximum")]
    LifetimeTooLong,
    #[error("signed message was issued in the future")]
    NotYetValid,
    #[error("signed message is expired")]
    Expired,
}

pub(crate) fn validate_window(
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    now: DateTime<Utc>,
    max_lifetime_seconds: i64,
) -> Result<(), SignedWindowError> {
    if expires_at <= issued_at {
        return Err(SignedWindowError::InvalidOrder);
    }
    if expires_at - issued_at > Duration::seconds(max_lifetime_seconds) {
        return Err(SignedWindowError::LifetimeTooLong);
    }
    if issued_at > now {
        return Err(SignedWindowError::NotYetValid);
    }
    if expires_at <= now {
        return Err(SignedWindowError::Expired);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn module(value: &str) -> ModuleDefinitionId {
        ModuleDefinitionId::new(value).unwrap()
    }

    fn capability(value: &str) -> SecurityCapabilityId {
        SecurityCapabilityId::new(value).unwrap()
    }

    fn dependency(value: &str) -> DependencyBindingKey {
        DependencyBindingKey::new(value).unwrap()
    }

    fn contract(value: &str) -> FunctionalContractId {
        FunctionalContractId::new(value).unwrap()
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-23T16:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn shell_context() -> ShellContextV2 {
        let now = now();
        ShellContextV2 {
            schema_version: SHELL_CONTEXT_SCHEMA_VERSION_V2,
            installation_id: id(1),
            module_definition_id: module("tessara.reference.scoped-records"),
            module_instance_id: id(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: id(3),
                display_name: "Tessara Administrator".into(),
                email: Some("admin@tessara.local".into()),
            },
            theme: ShellThemeV1::Dark,
            navigation: vec![ShellNavigationGroupProjectionV2 {
                id: "core.main".into(),
                label: "Main".into(),
                items: vec![ShellNavigationItemProjectionV2 {
                    contribution_id: NavigationContributionId::new(
                        "tessara.reference.scoped-records.main",
                    )
                    .unwrap(),
                    key: "scoped_records".into(),
                    label: "Scoped Records".into(),
                    href: "/modules/scoped-records/".into(),
                }],
            }],
            return_destination: "/admin/modules".into(),
            locale: "en-US".into(),
            time_zone: "America/New_York".into(),
            correlation_id: id(4),
            document_state: ShellDocumentStateV1::Active,
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        }
    }

    fn grant(operation: AuthorizationGrantOperationV1) -> AuthorizationGrantV2 {
        let now = now();
        AuthorizationGrantV2 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V2,
            installation_id: id(1),
            original_actor_id: id(3),
            presenting_service: module("tessara.core.gateway"),
            audience_module_instance_id: id(2),
            dependency_binding: dependency("scoped-records.core-authorization"),
            functional_contract: contract("core.authorization.exchange-v1"),
            action: match operation {
                AuthorizationGrantOperationV1::Read => "records.list",
                AuthorizationGrantOperationV1::Mutation => "records.create",
            }
            .into(),
            operation,
            capability_scope_bindings: vec![
                CapabilityScopeBindingV1 {
                    capability: capability("scoped_records:read"),
                    organization_root_id: id(10),
                    authorized_organization_ids: vec![id(11)],
                },
                CapabilityScopeBindingV1 {
                    capability: capability("scoped_records:manage"),
                    organization_root_id: id(20),
                    authorized_organization_ids: vec![id(21)],
                },
            ],
            resource_assertion: None,
            delegation_basis: vec![],
            authorization_revision: 42,
            organization_revision: 17,
            jti: id(30),
            issued_at: now,
            expires_at: now
                + Duration::seconds(match operation {
                    AuthorizationGrantOperationV1::Read => 60,
                    AuthorizationGrantOperationV1::Mutation => 30,
                }),
        }
    }

    fn grant_validation(
        operation: AuthorizationGrantOperationV1,
    ) -> AuthorizationValidationContextV2 {
        AuthorizationValidationContextV2 {
            installation_id: id(1),
            presenting_service: module("tessara.core.gateway"),
            audience_module_instance_id: id(2),
            dependency_binding: dependency("scoped-records.core-authorization"),
            functional_contract: contract("core.authorization.exchange-v1"),
            action: match operation {
                AuthorizationGrantOperationV1::Read => "records.list",
                AuthorizationGrantOperationV1::Mutation => "records.create",
            }
            .into(),
            operation,
            resource_assertion: None,
            authorization_revision: 42,
            organization_revision: 17,
            now: now() + Duration::seconds(1),
        }
    }

    fn resource_assertion(revision: u64) -> ResourceAuthorizationAssertionV2 {
        ResourceAuthorizationAssertionV2 {
            resource_type: ResourceTypeId::new("tessara.components.component-version").unwrap(),
            resource_id: id(40).to_string(),
            authority_revision: revision,
            governing_organization_ids: vec![id(10), id(11)],
        }
    }

    fn service_request() -> ModuleServiceRequestV1 {
        let now = now();
        ModuleServiceRequestV1 {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
            installation_id: id(1),
            module_instance_id: id(2),
            module_definition_id: module("tessara.dashboard"),
            method: "POST".into(),
            path: "/api/private/dashboard-components/render".into(),
            canonical_body_digest: "a".repeat(64),
            inbound_grant_digest: "b".repeat(64),
            correlation_id: id(4).to_string(),
            nonce: id(50),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        }
    }

    fn service_request_validation() -> ModuleServiceRequestValidationContextV1 {
        let request = service_request();
        ModuleServiceRequestValidationContextV1 {
            installation_id: request.installation_id,
            module_instance_id: request.module_instance_id,
            module_definition_id: request.module_definition_id,
            method: request.method,
            path: request.path,
            canonical_body_digest: request.canonical_body_digest,
            inbound_grant_digest: request.inbound_grant_digest,
            correlation_id: request.correlation_id,
            now: now() + Duration::seconds(1),
        }
    }

    fn core_service_request() -> CoreServiceRequestV1 {
        let now = now();
        CoreServiceRequestV1 {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
            installation_id: id(1),
            method: "POST".into(),
            path: "/api/private/datasets/summary".into(),
            canonical_body_digest: "a".repeat(64),
            inbound_grant_digest: "b".repeat(64),
            correlation_id: id(4).to_string(),
            nonce: id(51),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        }
    }

    fn core_service_request_validation() -> CoreServiceRequestValidationContextV1 {
        let request = core_service_request();
        CoreServiceRequestValidationContextV1 {
            installation_id: request.installation_id,
            method: request.method,
            path: request.path,
            canonical_body_digest: request.canonical_body_digest,
            inbound_grant_digest: request.inbound_grant_digest,
            correlation_id: request.correlation_id,
            now: now() + Duration::seconds(1),
        }
    }

    #[test]
    fn purpose_bound_ed25519_envelope_is_deterministic_and_tamper_evident() {
        let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "shell-context-dev-1",
            ProtocolSignaturePurposeV1::ShellContext,
            [7; 32],
        )
        .unwrap();
        let envelope = signer.sign(shell_context()).unwrap();
        let duplicate = signer.sign(shell_context()).unwrap();
        assert_eq!(envelope, duplicate);
        signer.verifier().verify(&envelope).unwrap();

        let wrong_purpose = PurposeBoundVerifyingKeyV1::from_public_bytes(
            "tessara.core",
            "shell-context-dev-1",
            ProtocolSignaturePurposeV1::AuthorizationGrant,
            signer.verifier().public_key_bytes(),
        )
        .unwrap();
        assert_eq!(
            wrong_purpose.verify(&envelope),
            Err(ProtocolEnvelopeError::WrongPurpose)
        );

        let mut tampered = envelope.clone();
        tampered.payload.original_actor.display_name = "Another actor".into();
        assert_eq!(
            signer.verifier().verify(&tampered),
            Err(ProtocolEnvelopeError::InvalidSignature)
        );
    }

    #[test]
    fn signed_envelopes_and_payloads_reject_unknown_wire_fields() {
        let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "shell-context-dev-1",
            ProtocolSignaturePurposeV1::ShellContext,
            [7; 32],
        )
        .unwrap();
        let envelope = signer.sign(shell_context()).unwrap();
        let mut wire = serde_json::to_value(envelope).unwrap();
        wire.as_object_mut()
            .unwrap()
            .insert("unexpected".into(), serde_json::json!(true));
        assert!(
            serde_json::from_value::<SignedEnvelopeV1<ShellContextV2>>(wire).is_err(),
            "unknown envelope fields must fail at the wire boundary"
        );

        let mut payload_wire = serde_json::to_value(shell_context()).unwrap();
        payload_wire
            .as_object_mut()
            .unwrap()
            .insert("authoritative".into(), serde_json::json!(true));
        assert!(
            serde_json::from_value::<ShellContextV2>(payload_wire).is_err(),
            "shell context cannot acquire product authority through an unknown field"
        );

        let mut retired_flat_navigation = serde_json::to_value(shell_context()).unwrap();
        retired_flat_navigation["navigation"] = serde_json::json!([{
            "contribution_id": "tessara.reference.scoped-records.main",
            "label": "Scoped Records",
            "href": "/modules/scoped-records/"
        }]);
        assert!(
            serde_json::from_value::<ShellContextV2>(retired_flat_navigation).is_err(),
            "the flat Shell Context v1 navigation shape must not bypass grouped shell projection"
        );
    }

    #[test]
    fn shell_context_validates_installation_audience_correlation_and_window() {
        let shell = shell_context();
        let expected = ShellContextValidationContextV2 {
            installation_id: id(1),
            module_definition_id: module("tessara.reference.scoped-records"),
            module_instance_id: id(2),
            correlation_id: id(4),
            now: now() + Duration::seconds(1),
        };
        shell.validate_for(&expected).unwrap();

        let mut wrong_audience = expected.clone();
        wrong_audience.module_instance_id = id(99);
        assert_eq!(
            shell.validate_for(&wrong_audience),
            Err(ShellContextValidationError::WrongAudience)
        );

        let mut too_long = shell;
        too_long.expires_at += Duration::seconds(1);
        assert_eq!(
            too_long.validate_for(&expected),
            Err(ShellContextValidationError::Window(
                SignedWindowError::LifetimeTooLong
            ))
        );
    }

    #[test]
    fn capability_scope_bindings_never_form_a_cross_product() {
        let grant = grant(AuthorizationGrantOperationV1::Read);
        let read = capability("scoped_records:read");
        let manage = capability("scoped_records:manage");

        assert!(grant.authorizes(&read, id(10)));
        assert!(grant.authorizes(&read, id(11)));
        assert!(grant.authorizes(&manage, id(20)));
        assert!(grant.authorizes(&manage, id(21)));
        assert!(!grant.authorizes(&read, id(21)));
        assert!(!grant.authorizes(&manage, id(11)));
    }

    #[test]
    fn authorization_validation_rejects_wrong_context_and_stale_revisions() {
        let grant = grant(AuthorizationGrantOperationV1::Read);
        let expected = grant_validation(AuthorizationGrantOperationV1::Read);
        grant.validate_for(&expected).unwrap();

        let mut wrong_action = expected.clone();
        wrong_action.action = "records.detail".into();
        assert_eq!(
            grant.validate_for(&wrong_action),
            Err(AuthorizationValidationError::WrongAction)
        );

        let mut stale_authorization = expected.clone();
        stale_authorization.authorization_revision += 1;
        assert_eq!(
            grant.validate_for(&stale_authorization),
            Err(AuthorizationValidationError::StaleAuthorizationRevision)
        );

        let mut stale_organization = expected;
        stale_organization.organization_revision += 1;
        assert_eq!(
            grant.validate_for(&stale_organization),
            Err(AuthorizationValidationError::StaleOrganizationRevision)
        );
    }

    #[test]
    fn v3_binds_exact_presenting_instance_and_core_or_module_audience() {
        let now = now();
        let presenter = ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: id(70),
            module_definition_id: module("tessara.components"),
        };
        let audience = AuthorizationAudienceV1::CoreInstallation {
            installation_id: id(1),
        };
        let current = AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: id(1),
            original_actor_id: id(3),
            correlation_id: id(4),
            presenting_service: presenter.clone(),
            audience: audience.clone(),
            dependency_binding: dependency("tessara.components.dataset-major-line"),
            functional_contract: contract("tessara.datasets.dataset-major-line"),
            action: "datasets.execute".into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: capability("datasets:read"),
                organization_root_id: id(10),
                authorized_organization_ids: vec![],
            }],
            resource_assertion: None,
            delegation_basis: vec![],
            authorization_revision: 42,
            organization_revision: 17,
            jti: id(71),
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        };
        let mut context = AuthorizationValidationContextV3 {
            installation_id: id(1),
            correlation_id: id(4),
            presenting_service: presenter,
            audience,
            dependency_binding: dependency("tessara.components.dataset-major-line"),
            functional_contract: contract("tessara.datasets.dataset-major-line"),
            action: "datasets.execute".into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: 42,
            organization_revision: 17,
            now: now + Duration::seconds(1),
        };
        current.validate_for(&context).unwrap();

        context.correlation_id = id(99);
        assert_eq!(
            current.validate_for(&context),
            Err(AuthorizationValidationError::WrongCorrelation)
        );
        context.correlation_id = id(4);

        context.presenting_service = ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: id(72),
            module_definition_id: module("tessara.components"),
        };
        assert_eq!(
            current.validate_for(&context),
            Err(AuthorizationValidationError::WrongPresentingService)
        );

        let historical = serde_json::to_value(grant(AuthorizationGrantOperationV1::Read)).unwrap();
        assert!(
            serde_json::from_value::<AuthorizationGrantV3>(historical).is_err(),
            "normal V3 readers must reject the immutable historical V2 shape"
        );
    }

    #[test]
    fn exchange_v2_target_is_tagged_and_provider_policy_is_not_caller_selected() {
        let request = AuthorizationExchangeRequestV2 {
            schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2,
            target: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: id(80),
                module_definition_id: module("tessara.components"),
            },
            dependency_binding: dependency("tessara.dashboards.component-version"),
            functional_contract: contract("tessara.components.component-version"),
            action: "components.render".into(),
            resource_assertion: None,
        };
        request.validate().unwrap();
        let wire = serde_json::to_value(request).unwrap();
        assert_eq!(wire["target"]["kind"], "module_instance");
        assert!(wire.get("operation").is_none());
        assert!(wire.get("required_capability").is_none());
    }

    #[test]
    fn read_and_mutation_lifetimes_are_enforced_independently() {
        grant(AuthorizationGrantOperationV1::Read)
            .validate_for(&grant_validation(AuthorizationGrantOperationV1::Read))
            .unwrap();
        grant(AuthorizationGrantOperationV1::Mutation)
            .validate_for(&grant_validation(AuthorizationGrantOperationV1::Mutation))
            .unwrap();

        let mut mutation = grant(AuthorizationGrantOperationV1::Mutation);
        mutation.expires_at += Duration::seconds(1);
        assert_eq!(
            mutation.validate_for(&grant_validation(AuthorizationGrantOperationV1::Mutation)),
            Err(AuthorizationValidationError::Window(
                SignedWindowError::LifetimeTooLong
            ))
        );

        let mut duplicate_scope = grant(AuthorizationGrantOperationV1::Read);
        duplicate_scope.capability_scope_bindings[0]
            .authorized_organization_ids
            .push(id(10));
        assert_eq!(
            duplicate_scope.validate_for(&grant_validation(AuthorizationGrantOperationV1::Read)),
            Err(AuthorizationValidationError::DuplicateOrganizationBinding)
        );
    }

    #[test]
    fn v2_resource_assertion_is_exact_and_provider_fresh() {
        let assertion = resource_assertion(7);
        let mut grant = grant(AuthorizationGrantOperationV1::Read);
        grant.resource_assertion = Some(assertion.clone());
        let mut context = grant_validation(AuthorizationGrantOperationV1::Read);
        context.resource_assertion = Some(assertion);
        grant.validate_for(&context).unwrap();

        context
            .resource_assertion
            .as_mut()
            .unwrap()
            .authority_revision += 1;
        assert_eq!(
            grant.validate_for(&context),
            Err(AuthorizationValidationError::StaleResourceAssertion)
        );
    }

    #[test]
    fn service_request_binds_target_and_digests() {
        let request = service_request();
        let mut expected = service_request_validation();
        request.validate_for(&expected).unwrap();

        expected.path = "/api/private/dashboard-components/catalog".into();
        assert_eq!(
            request.validate_for(&expected),
            Err(ModuleServiceRequestValidationError::WrongTarget)
        );
        expected = service_request_validation();
        expected.inbound_grant_digest = "c".repeat(64);
        assert_eq!(
            request.validate_for(&expected),
            Err(ModuleServiceRequestValidationError::WrongDigest)
        );
        expected = service_request_validation();
        expected.correlation_id = id(99).to_string();
        assert_eq!(
            request.validate_for(&expected),
            Err(ModuleServiceRequestValidationError::WrongCorrelation)
        );
    }

    #[test]
    fn service_request_lifetime_is_capped_at_thirty_seconds() {
        let mut request = service_request();
        request.expires_at += Duration::seconds(1);
        assert_eq!(
            request.validate_for(&service_request_validation()),
            Err(ModuleServiceRequestValidationError::Window(
                SignedWindowError::LifetimeTooLong
            ))
        );
    }

    #[test]
    fn core_service_request_binds_installation_target_body_grant_and_correlation() {
        let request = core_service_request();
        request
            .validate_for(&core_service_request_validation())
            .unwrap();

        let mut expected = core_service_request_validation();
        expected.installation_id = id(99);
        assert_eq!(
            request.validate_for(&expected),
            Err(CoreServiceRequestValidationError::WrongInstallation)
        );

        expected = core_service_request_validation();
        expected.path = "/api/private/datasets/operations-status".into();
        assert_eq!(
            request.validate_for(&expected),
            Err(CoreServiceRequestValidationError::WrongTarget)
        );

        expected = core_service_request_validation();
        expected.canonical_body_digest = "c".repeat(64);
        assert_eq!(
            request.validate_for(&expected),
            Err(CoreServiceRequestValidationError::WrongDigest)
        );

        expected = core_service_request_validation();
        expected.correlation_id = id(98).to_string();
        assert_eq!(
            request.validate_for(&expected),
            Err(CoreServiceRequestValidationError::WrongCorrelation)
        );
    }

    #[test]
    fn core_service_request_lifetime_is_capped_at_thirty_seconds() {
        let mut request = core_service_request();
        request.expires_at += Duration::seconds(1);
        assert_eq!(
            request.validate_for(&core_service_request_validation()),
            Err(CoreServiceRequestValidationError::Window(
                SignedWindowError::LifetimeTooLong
            ))
        );
    }

    #[test]
    fn provider_compatibility_identity_is_canonical_exact_and_purpose_bound() {
        let first = ModuleProviderCompatibilityExpectationV1 {
            dependency_binding: "example.consumer.forms".into(),
            functional_contract: "example.forms.schema".into(),
            contract_version: "1.0.0".into(),
            authorization_action: "forms.resolve_schema".into(),
            method: ServiceActionMethod::Post,
            path: "/api/private/forms/schema".into(),
        };
        let second = ModuleProviderCompatibilityExpectationV1 {
            dependency_binding: "example.consumer.workflows".into(),
            functional_contract: "example.workflows.context".into(),
            contract_version: "1.0.0".into(),
            authorization_action: "workflows.issue_context".into(),
            method: ServiceActionMethod::Post,
            path: "/api/private/workflows/context".into(),
        };
        let request = ModuleProviderCompatibilityRequestV1 {
            schema_version: MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1,
            expectations: vec![first.clone(), second.clone()],
        };
        request.validate().unwrap();
        assert!(request.canonical_digest().unwrap().starts_with("sha256:"));

        let provider = AuthorizationAudienceV1::CoreInstallation {
            installation_id: id(1),
        };
        let consumer = ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: id(2),
            module_definition_id: module("example.consumer"),
        };
        let correlation_id = id(3);
        let issued_at = now();
        let response = ModuleProviderCompatibilityResponseV1 {
            schema_version: MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1,
            provider: provider.clone(),
            consumer: consumer.clone(),
            request_digest: request.canonical_digest().unwrap(),
            expectations: request.expectations.clone(),
            correlation_id,
            issued_at,
            expires_at: issued_at
                + Duration::seconds(MODULE_PROVIDER_COMPATIBILITY_MAX_LIFETIME_SECONDS),
        };
        response
            .validate_for(
                &request,
                &provider,
                &consumer,
                correlation_id,
                issued_at + Duration::seconds(1),
            )
            .unwrap();

        let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "provider-compatibility-dev-1",
            ProtocolSignaturePurposeV1::ProviderCompatibilityResponse,
            [9; 32],
        )
        .unwrap();
        let envelope = signer.sign(response).unwrap();
        signer.verifier().verify(&envelope).unwrap();
        let wrong_purpose = PurposeBoundVerifyingKeyV1::from_public_bytes(
            "tessara.core",
            "provider-compatibility-dev-1",
            ProtocolSignaturePurposeV1::ModuleServiceRequest,
            signer.verifier().public_key_bytes(),
        )
        .unwrap();
        assert_eq!(
            wrong_purpose.verify(&envelope),
            Err(ProtocolEnvelopeError::WrongPurpose)
        );

        let duplicate = ModuleProviderCompatibilityRequestV1 {
            schema_version: MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1,
            expectations: vec![first.clone(), first],
        };
        assert_eq!(
            duplicate.validate(),
            Err(ModuleProviderCompatibilityError::InvalidExpectationSet)
        );
        let reversed = ModuleProviderCompatibilityRequestV1 {
            schema_version: MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1,
            expectations: vec![second, request.expectations[0].clone()],
        };
        assert_eq!(
            reversed.validate(),
            Err(ModuleProviderCompatibilityError::InvalidExpectationSet)
        );
    }
}
