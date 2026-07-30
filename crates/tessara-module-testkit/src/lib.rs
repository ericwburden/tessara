//! Shared conformance fixtures for independently deployed Tessara modules.

use chrono::{Duration, Utc};
use tessara_module_contract::{
    ModuleDefinitionId, OriginalActorProjectionV1, ProtocolSignaturePurposeV1,
    PurposeBoundSigningKeyV1, ShellContextV1, ShellDocumentStateV1, ShellThemeV1, SignedEnvelopeV1,
};
use uuid::Uuid;

pub struct SignedShellFixture {
    pub signer: PurposeBoundSigningKeyV1,
    pub envelope: SignedEnvelopeV1<ShellContextV1>,
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
}
