use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use tessara_module_contract::{ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1};

fn main() {
    for (label, issuer, key_id, purpose, byte) in [
        (
            "core",
            "tessara.core",
            "core-development-v1",
            ProtocolSignaturePurposeV1::AuthorizationGrant,
            12,
        ),
        (
            "fixture_external",
            "tessara.fixture-identity",
            "fixture-external-v1",
            ProtocolSignaturePurposeV1::FixtureExternalIdentity,
            13,
        ),
        (
            "recovery_operator",
            "tessara.installation-control",
            "recovery-operator-v1",
            ProtocolSignaturePurposeV1::RecoveryOperatorAuthorization,
            14,
        ),
    ] {
        let signer =
            PurposeBoundSigningKeyV1::from_secret_bytes(issuer, key_id, purpose, [byte; 32])
                .unwrap();
        println!(
            "{label}={}",
            URL_SAFE_NO_PAD.encode(signer.verifier().public_key_bytes())
        );
    }
}
