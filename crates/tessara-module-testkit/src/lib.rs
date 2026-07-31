//! Shared conformance fixtures for independently deployed Tessara modules.

use chrono::{Duration, Utc};
use tessara_module_contract::{
    ArtifactDigest, BrowserLifecycleAssetV1, BrowserLifecycleBootstrapV1, ModuleDefinitionId,
    OriginalActorProjectionV1, ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1,
    SemanticRouteName, ShellContextV1, ShellDocumentStateV1, ShellThemeV1, SignedEnvelopeV1,
};
use uuid::Uuid;

pub struct SignedShellFixture {
    pub signer: PurposeBoundSigningKeyV1,
    pub envelope: SignedEnvelopeV1<ShellContextV1>,
}

/// Deterministic lifecycle-v1 projection for host, gateway, and module
/// conformance tests. The payload deliberately remains caller-owned JSON.
pub fn browser_lifecycle_fixture(
    definition_id: &str,
    release_version: &str,
    destination: &str,
    path: &str,
    payload: serde_json::Value,
) -> BrowserLifecycleBootstrapV1 {
    let digest = ArtifactDigest::new(format!("sha256:{}", "d".repeat(64))).unwrap();
    BrowserLifecycleBootstrapV1 {
        schema_version: BrowserLifecycleBootstrapV1::SCHEMA_VERSION,
        definition_id: ModuleDefinitionId::new(definition_id).expect("definition ID"),
        release_version: release_version.parse().expect("release version"),
        lifecycle_abi: "1.0.0".parse().expect("lifecycle ABI"),
        destination: SemanticRouteName::new(destination).expect("semantic destination"),
        path: path.into(),
        title: "Lifecycle conformance fixture".into(),
        document_state: ShellDocumentStateV1::Active,
        entry_asset: BrowserLifecycleAssetV1 {
            url: format!("/_tessara/modules/{definition_id}/{release_version}/{digest}/module.js"),
            digest,
            content_type: "text/javascript; charset=utf-8".into(),
        },
        stylesheet_assets: Vec::new(),
        payload,
    }
}

pub fn signed_shell_fixture(definition_id: &str) -> SignedShellFixture {
    let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "test-shell",
        ProtocolSignaturePurposeV1::ShellContext,
        [29; 32],
    )
    .expect("fixed signing key");
    let now = Utc::now();
    let context = ShellContextV1 {
        schema_version: 1,
        installation_id: Uuid::from_u128(1),
        module_definition_id: ModuleDefinitionId::new(definition_id).expect("definition ID"),
        module_instance_id: Uuid::from_u128(2),
        original_actor: OriginalActorProjectionV1 {
            actor_id: Uuid::from_u128(3),
            display_name: "Conformance Operator".into(),
            email: None,
        },
        theme: ShellThemeV1::System,
        navigation: vec![],
        return_destination: "/administration/modules".into(),
        locale: "en-US".into(),
        time_zone: "UTC".into(),
        correlation_id: Uuid::from_u128(4),
        document_state: ShellDocumentStateV1::Active,
        issued_at: now,
        expires_at: now + Duration::seconds(60),
    };
    let envelope = signer.sign(context).expect("fixture signing");
    SignedShellFixture { signer, envelope }
}

#[cfg(test)]
mod tests {
    use tessara_module_contract::ShellContextValidationContextV1;
    use tessara_module_runtime::verify_shell_context;

    use super::*;

    #[test]
    fn fixture_conforms_to_runtime_verification() {
        let fixture = signed_shell_fixture("tessara.reference.module-sdk");
        let context = &fixture.envelope.payload;
        verify_shell_context(
            &fixture.envelope,
            &fixture.signer.verifier(),
            &ShellContextValidationContextV1 {
                installation_id: context.installation_id,
                module_definition_id: context.module_definition_id.clone(),
                module_instance_id: context.module_instance_id,
                correlation_id: context.correlation_id,
                now: context.issued_at,
            },
        )
        .unwrap();
    }

    #[test]
    fn browser_fixture_conforms_to_lifecycle_host_validation() {
        let fixture = browser_lifecycle_fixture(
            "tessara.reference.lifecycle",
            "1.0.0",
            "tessara.reference.lifecycle.root",
            "/reference/lifecycle",
            serde_json::json!({"framework":"independent"}),
        );
        assert!(fixture.is_supported());
    }
}
