use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use tessara_module_contract::{
    AUTHORIZATION_GRANT_SCHEMA_VERSION_V2, AuthorizationGrantOperationV1, AuthorizationGrantV2,
    CONTRACT_SCHEMA_VERSION_V1, CapabilityScopeBindingV1, DependencyBindingKey,
    ExternalIdentityAssertionV1, FunctionalContractId, ModuleDefinitionId,
    NavigationContributionId, NavigationProjectionV1, OriginalActorProjectionV1,
    ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1, SecurityCapabilityId, ShellContextV1,
    ShellDocumentStateV1, ShellThemeV1,
};
use uuid::Uuid;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn main() {
    let now = DateTime::parse_from_rfc3339("2026-07-23T16:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let shell_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "shell-context-dev-1",
        ProtocolSignaturePurposeV1::ShellContext,
        [7; 32],
    )
    .unwrap();
    let authorization_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "authorization-grant-dev-1",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        [8; 32],
    )
    .unwrap();
    let fixture_identity_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.fixture-identity",
        "fixture-external-dev-1",
        ProtocolSignaturePurposeV1::FixtureExternalIdentity,
        [9; 32],
    )
    .unwrap();

    let shell = shell_signer
        .sign(ShellContextV1 {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
            installation_id: id(1),
            module_definition_id: ModuleDefinitionId::new("tessara.reference.scoped-records")
                .unwrap(),
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
        })
        .unwrap();

    let authorization = authorization_signer
        .sign(AuthorizationGrantV2 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V2,
            installation_id: id(1),
            original_actor_id: id(3),
            presenting_service: ModuleDefinitionId::new("tessara.core.gateway").unwrap(),
            audience_module_instance_id: id(2),
            dependency_binding: DependencyBindingKey::new("scoped-records.core-authorization")
                .unwrap(),
            functional_contract: FunctionalContractId::new("core.authorization.exchange-v1")
                .unwrap(),
            action: "records.list".into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![
                CapabilityScopeBindingV1 {
                    capability: SecurityCapabilityId::new("scoped_records:read").unwrap(),
                    organization_root_id: id(10),
                    authorized_organization_ids: vec![id(11)],
                },
                CapabilityScopeBindingV1 {
                    capability: SecurityCapabilityId::new("scoped_records:manage").unwrap(),
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
            expires_at: now + Duration::seconds(60),
        })
        .unwrap();

    let external_identity = fixture_identity_signer
        .sign(ExternalIdentityAssertionV1 {
            schema_version: CONTRACT_SCHEMA_VERSION_V1,
            installation_id: id(1),
            audience: "core-administrator-enrollment".into(),
            external_subject: "fixture-admin-001".into(),
            email: "fixture-admin@tessara.local".into(),
            display_name: "Fixture Administrator".into(),
            nonce: id(40),
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .unwrap();

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "trust": {
                "schema_version": 1,
                "keys": [
                    {
                        "issuer": "tessara.core",
                        "key_id": "shell-context-dev-1",
                        "purpose": "shell_context",
                        "public_key": base64::Engine::encode(
                            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                            shell_signer.verifier().public_key_bytes(),
                        ),
                    },
                    {
                        "issuer": "tessara.core",
                        "key_id": "authorization-grant-dev-1",
                        "purpose": "authorization_grant",
                        "public_key": base64::Engine::encode(
                            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                            authorization_signer.verifier().public_key_bytes(),
                        ),
                    },
                    {
                        "issuer": "tessara.fixture-identity",
                        "key_id": "fixture-external-dev-1",
                        "purpose": "fixture_external_identity",
                        "public_key": base64::Engine::encode(
                            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                            fixture_identity_signer.verifier().public_key_bytes(),
                        ),
                    }
                ]
            },
            "shell_context": shell,
            "authorization_grant": authorization,
            "external_identity": external_identity,
        }))
        .unwrap()
    );
}
