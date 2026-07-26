use std::collections::BTreeSet;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    CONTRACT_SCHEMA_VERSION_V1, DependencyBindingKey, FunctionalContractId, ModuleDefinitionId,
    NavigationContributionId, ResourceTypeId, SecurityCapabilityId,
};

pub const SHELL_CONTEXT_MAX_LIFETIME_SECONDS: i64 = 60;
pub const AUTHORIZATION_READ_MAX_LIFETIME_SECONDS: i64 = 60;
pub const AUTHORIZATION_MUTATION_MAX_LIFETIME_SECONDS: i64 = 30;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolSignaturePurposeV1 {
    ShellContext,
    AuthorizationGrant,
    EnrollmentEligibility,
    EnrollmentRedemption,
    RecoveryOperatorAuthorization,
    FixtureExternalIdentity,
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
    let value = serde_json::to_value(SigningInputV1 {
        schema_version,
        issuer,
        key_id,
        purpose,
        payload,
    })
    .map_err(|error| ProtocolEnvelopeError::Serialization(error.to_string()))?;
    let mut output = Vec::new();
    write_canonical_json(&value, &mut output)?;
    Ok(output)
}

fn write_canonical_json(
    value: &serde_json::Value,
    output: &mut Vec<u8>,
) -> Result<(), ProtocolEnvelopeError> {
    match value {
        serde_json::Value::Null => output.extend_from_slice(b"null"),
        serde_json::Value::Bool(value) => {
            output.extend_from_slice(if *value { b"true" } else { b"false" })
        }
        serde_json::Value::Number(value) => output.extend_from_slice(value.to_string().as_bytes()),
        serde_json::Value::String(value) => {
            let encoded = serde_json::to_string(value)
                .map_err(|error| ProtocolEnvelopeError::Serialization(error.to_string()))?;
            output.extend_from_slice(encoded.as_bytes());
        }
        serde_json::Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        serde_json::Value::Object(values) => {
            output.push(b'{');
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                let encoded_key = serde_json::to_string(key)
                    .map_err(|error| ProtocolEnvelopeError::Serialization(error.to_string()))?;
                output.extend_from_slice(encoded_key.as_bytes());
                output.push(b':');
                write_canonical_json(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
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
pub struct NavigationProjectionV1 {
    pub contribution_id: NavigationContributionId,
    pub label: String,
    pub href: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellContextV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub module_definition_id: ModuleDefinitionId,
    pub module_instance_id: Uuid,
    pub original_actor: OriginalActorProjectionV1,
    pub theme: ShellThemeV1,
    pub navigation: Vec<NavigationProjectionV1>,
    pub return_destination: String,
    pub locale: String,
    pub time_zone: String,
    pub correlation_id: Uuid,
    pub document_state: ShellDocumentStateV1,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellContextValidationContextV1 {
    pub installation_id: Uuid,
    pub module_definition_id: ModuleDefinitionId,
    pub module_instance_id: Uuid,
    pub correlation_id: Uuid,
    pub now: DateTime<Utc>,
}

impl ShellContextV1 {
    pub fn validate_for(
        &self,
        expected: &ShellContextValidationContextV1,
    ) -> Result<(), ShellContextValidationError> {
        if self.schema_version != CONTRACT_SCHEMA_VERSION_V1 {
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
pub struct ResourceAuthorizationAssertionV1 {
    pub resource_type: ResourceTypeId,
    pub resource_id: String,
    pub owner_organization_id: Uuid,
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
pub struct AuthorizationGrantV1 {
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
    pub resource_assertion: Option<ResourceAuthorizationAssertionV1>,
    pub delegation_basis: Vec<DelegationBasisV1>,
    pub authorization_revision: u64,
    pub organization_revision: u64,
    pub jti: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationValidationContextV1 {
    pub installation_id: Uuid,
    pub presenting_service: ModuleDefinitionId,
    pub audience_module_instance_id: Uuid,
    pub dependency_binding: DependencyBindingKey,
    pub functional_contract: FunctionalContractId,
    pub action: String,
    pub operation: AuthorizationGrantOperationV1,
    pub authorization_revision: u64,
    pub organization_revision: u64,
    pub now: DateTime<Utc>,
}

impl AuthorizationGrantV1 {
    pub fn validate_for(
        &self,
        expected: &AuthorizationValidationContextV1,
    ) -> Result<(), AuthorizationValidationError> {
        if self.schema_version != CONTRACT_SCHEMA_VERSION_V1 {
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
        if self.authorization_revision != expected.authorization_revision {
            return Err(AuthorizationValidationError::StaleAuthorizationRevision);
        }
        if self.organization_revision != expected.organization_revision {
            return Err(AuthorizationValidationError::StaleOrganizationRevision);
        }
        if self.jti.is_nil() {
            return Err(AuthorizationValidationError::MissingReplayIdentifier);
        }
        if self.capability_scope_bindings.is_empty() {
            return Err(AuthorizationValidationError::MissingCapabilityBindings);
        }
        for binding in &self.capability_scope_bindings {
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

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AuthorizationValidationError {
    #[error("authorization grant schema version is unsupported")]
    UnsupportedSchemaVersion,
    #[error("authorization grant is bound to another installation")]
    WrongInstallation,
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

    fn shell_context() -> ShellContextV1 {
        let now = now();
        ShellContextV1 {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
            installation_id: id(1),
            module_definition_id: module("tessara.reference.scoped-records"),
            module_instance_id: id(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: id(3),
                display_name: "Tessara Administrator".into(),
                email: Some("admin@tessara.local".into()),
            },
            theme: ShellThemeV1::Dark,
            navigation: vec![NavigationProjectionV1 {
                contribution_id: NavigationContributionId::new(
                    "tessara.reference.scoped-records.main",
                )
                .unwrap(),
                label: "Scoped Records".into(),
                href: "/modules/scoped-records/".into(),
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

    fn grant(operation: AuthorizationGrantOperationV1) -> AuthorizationGrantV1 {
        let now = now();
        AuthorizationGrantV1 {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
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
    ) -> AuthorizationValidationContextV1 {
        AuthorizationValidationContextV1 {
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
            authorization_revision: 42,
            organization_revision: 17,
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
            serde_json::from_value::<SignedEnvelopeV1<ShellContextV1>>(wire).is_err(),
            "unknown envelope fields must fail at the wire boundary"
        );

        let mut payload_wire = serde_json::to_value(shell_context()).unwrap();
        payload_wire
            .as_object_mut()
            .unwrap()
            .insert("authoritative".into(), serde_json::json!(true));
        assert!(
            serde_json::from_value::<ShellContextV1>(payload_wire).is_err(),
            "shell context cannot acquire product authority through an unknown field"
        );
    }

    #[test]
    fn shell_context_validates_installation_audience_correlation_and_window() {
        let shell = shell_context();
        let expected = ShellContextValidationContextV1 {
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
}
