use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV2, AuthorizationValidationContextV2,
    DependencyBindingKey, ExternalIdentityAssertionV1, FunctionalContractId, ModuleDefinitionId,
    ProtocolEnvelopeError, ProtocolSignaturePurposeV1, PurposeBoundVerifyingKeyV1, ShellContextV1,
    ShellContextValidationContextV1, SignedEnvelopeV1,
};
use uuid::Uuid;

const VALID_PROTOCOL_MESSAGES: &[u8] = include_bytes!("fixtures/valid-protocol-messages-v2.json");
const VALID_PROTOCOL_MESSAGES_DIGEST: &str =
    include_str!("fixtures/valid-protocol-messages-v2.json.sha256");
const TAMPERED_SHELL_CONTEXT: &[u8] =
    include_bytes!("fixtures/invalid-tampered-shell-context-v1.json");
const TAMPERED_SHELL_CONTEXT_DIGEST: &str =
    include_str!("fixtures/invalid-tampered-shell-context-v1.json.sha256");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtocolFixtures {
    authorization_grant: SignedEnvelopeV1<AuthorizationGrantV2>,
    external_identity: SignedEnvelopeV1<ExternalIdentityAssertionV1>,
    shell_context: SignedEnvelopeV1<ShellContextV1>,
    trust: DevelopmentTrustV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DevelopmentTrustV1 {
    schema_version: u16,
    keys: Vec<DevelopmentTrustKeyV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DevelopmentTrustKeyV1 {
    issuer: String,
    key_id: String,
    purpose: ProtocolSignaturePurposeV1,
    public_key: String,
}

impl DevelopmentTrustV1 {
    fn verifier(&self, purpose: ProtocolSignaturePurposeV1) -> PurposeBoundVerifyingKeyV1 {
        let key = self
            .keys
            .iter()
            .find(|key| key.purpose == purpose)
            .expect("fixture purpose has a verification key");
        let public_key: [u8; 32] = URL_SAFE_NO_PAD
            .decode(&key.public_key)
            .expect("fixture verification key uses base64url")
            .try_into()
            .expect("fixture verification key is 32 bytes");
        PurposeBoundVerifyingKeyV1::from_public_bytes(
            key.issuer.clone(),
            key.key_id.clone(),
            key.purpose,
            public_key,
        )
        .expect("fixture verification key is valid")
    }
}

fn assert_canonical_fixture(bytes: &[u8], sidecar: &str) {
    assert!(!bytes.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(!bytes.contains(&b'\r'));
    assert!(bytes.ends_with(b"\n"));
    assert!(!bytes.ends_with(b"\n\n"));
    let digest = format!("{:x}", Sha256::digest(bytes));
    assert_eq!(sidecar, format!("{digest}\n"));
}

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-07-23T16:00:01Z")
        .unwrap()
        .with_timezone(&Utc)
}

#[test]
fn canonical_protocol_messages_verify_with_purpose_specific_public_keys() {
    assert_canonical_fixture(VALID_PROTOCOL_MESSAGES, VALID_PROTOCOL_MESSAGES_DIGEST);
    let fixtures: ProtocolFixtures = serde_json::from_slice(VALID_PROTOCOL_MESSAGES).unwrap();
    assert_eq!(fixtures.trust.schema_version, 1);
    assert_eq!(fixtures.trust.keys.len(), 3);

    fixtures
        .trust
        .verifier(ProtocolSignaturePurposeV1::ShellContext)
        .verify(&fixtures.shell_context)
        .unwrap();
    fixtures
        .trust
        .verifier(ProtocolSignaturePurposeV1::AuthorizationGrant)
        .verify(&fixtures.authorization_grant)
        .unwrap();
    fixtures
        .trust
        .verifier(ProtocolSignaturePurposeV1::FixtureExternalIdentity)
        .verify(&fixtures.external_identity)
        .unwrap();

    fixtures
        .shell_context
        .payload
        .validate_for(&ShellContextValidationContextV1 {
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.reference.scoped-records")
                .unwrap(),
            module_instance_id: Uuid::from_u128(2),
            correlation_id: Uuid::from_u128(4),
            now: now(),
        })
        .unwrap();
    fixtures
        .authorization_grant
        .payload
        .validate_for(&AuthorizationValidationContextV2 {
            installation_id: Uuid::from_u128(1),
            presenting_service: ModuleDefinitionId::new("tessara.core.gateway").unwrap(),
            audience_module_instance_id: Uuid::from_u128(2),
            dependency_binding: DependencyBindingKey::new("scoped-records.core-authorization")
                .unwrap(),
            functional_contract: FunctionalContractId::new("core.authorization.exchange-v1")
                .unwrap(),
            action: "records.list".into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: 42,
            organization_revision: 17,
            now: now(),
        })
        .unwrap();
}

#[test]
fn tampered_shell_context_fixture_fails_signature_verification() {
    assert_canonical_fixture(TAMPERED_SHELL_CONTEXT, TAMPERED_SHELL_CONTEXT_DIGEST);
    let fixtures: ProtocolFixtures = serde_json::from_slice(VALID_PROTOCOL_MESSAGES).unwrap();
    let tampered: SignedEnvelopeV1<ShellContextV1> =
        serde_json::from_slice(TAMPERED_SHELL_CONTEXT).unwrap();
    assert_eq!(
        fixtures
            .trust
            .verifier(ProtocolSignaturePurposeV1::ShellContext)
            .verify(&tampered),
        Err(ProtocolEnvelopeError::InvalidSignature)
    );
}
