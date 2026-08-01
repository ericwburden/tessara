use tessara_composition::{ApplicationBlueprintV1, ReleaseCatalogV1, canonical_digest, resolve};
use tessara_module_contract::{
    ProtocolSignaturePurposeV1, PurposeBoundVerifyingKeyV1, SignedEnvelopeV1,
};

fn catalog() -> ReleaseCatalogV1 {
    serde_json::from_str(include_str!(
        "../../../deploy/sprint-6f/catalogs/local-release-catalog.json"
    ))
    .expect("Sprint 6F catalog must satisfy the strict v1 contract")
}

fn blueprint(source: &str) -> ApplicationBlueprintV1 {
    serde_json::from_str(source).expect("Sprint 6F Blueprint must satisfy the strict v1 contract")
}

#[test]
fn reference_fixture_resolves_to_dashboard_and_scoped_records() {
    let blueprint = blueprint(include_str!(
        "../../../deploy/sprint-6f/blueprints/reference.json"
    ));
    let lockfile = resolve(&blueprint, &catalog()).expect("reference Blueprint resolves");
    assert_eq!(lockfile.modules.len(), 2);
    assert_eq!(lockfile.modules[0].definition_id, "tessara.dashboards");
    assert_eq!(
        lockfile.modules[1].definition_id,
        "tessara.reference.scoped-records"
    );
    assert_eq!(
        canonical_digest(&lockfile.materialization_plan).expect("digest"),
        lockfile.materialization_plan_digest
    );
}

#[test]
fn reduced_fixture_resolves_to_core_only() {
    let blueprint = blueprint(include_str!(
        "../../../deploy/sprint-6f/blueprints/reduced.json"
    ));
    let lockfile = resolve(&blueprint, &catalog()).expect("reduced Blueprint resolves");
    assert!(lockfile.modules.is_empty());
    assert!(
        lockfile
            .materialization_plan
            .actions
            .iter()
            .all(|action| !format!("{action:?}").contains("tessara."))
    );
}

#[test]
fn checked_catalog_snapshot_is_purpose_bound_and_signature_valid() {
    let envelope: SignedEnvelopeV1<ReleaseCatalogV1> = serde_json::from_str(include_str!(
        "../../../deploy/sprint-6f/catalogs/local-release-catalog.signed.json"
    ))
    .expect("strict signed catalog envelope");
    let public_hex =
        include_str!("../../../deploy/sprint-6f/catalogs/catalog-dev-v1.public.hex").trim();
    let mut public_key = [0_u8; 32];
    for (index, byte) in public_key.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&public_hex[index * 2..index * 2 + 2], 16).expect("hex public key");
    }
    let verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.local.sprint-6f",
        "catalog-dev-v1",
        ProtocolSignaturePurposeV1::ReleaseCatalog,
        public_key,
    )
    .expect("catalog verifier");
    verifier.verify(&envelope).expect("catalog signature");
    assert_eq!(envelope.payload, catalog());
}
