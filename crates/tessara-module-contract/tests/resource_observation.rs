use serde_json::json;
use tessara_module_contract::{
    ProviderContractIdentity, ResourceObservationStrategy, ResourceObservationV1, ResourceOwner,
    ResourceRevision, ResourceTypeId, TypedResourceReference,
};
use uuid::Uuid;

const VALID_OBSERVATION: &str = include_str!("fixtures/valid-resource-observation-v1.json");
const INVALID_ZERO_REVISION: &str =
    include_str!("fixtures/invalid-resource-observation-zero-revision-v1.json");
const INVALID_MIXED_VERSION: &str =
    include_str!("fixtures/invalid-resource-observation-mixed-version-v1.json");

#[test]
fn valid_observation_round_trips_exact_identity_strategy_and_revision() {
    let parsed: ResourceObservationV1 = serde_json::from_str(VALID_OBSERVATION).expect("fixture");
    assert_eq!(parsed.schema_version(), 1);
    assert_eq!(
        parsed.reference().resource_id(),
        "33333333-3333-4333-8333-333333333333"
    );
    assert_eq!(
        parsed.provider_contract().contract_id().as_str(),
        "tessara.components.component-version"
    );
    assert_eq!(
        parsed.provider_contract().contract_version().to_string(),
        "2.0.0"
    );
    assert_eq!(
        parsed.strategy(),
        ResourceObservationStrategy::LiveResolutionWithRevision
    );
    assert_eq!(parsed.resource_revision().get(), 7);
    assert_eq!(
        serde_json::to_value(&parsed).expect("serialize"),
        serde_json::from_str::<serde_json::Value>(VALID_OBSERVATION).expect("json")
    );
}

#[test]
fn malformed_revision_mixed_version_and_unknown_fields_fail_closed() {
    assert!(serde_json::from_str::<ResourceObservationV1>(INVALID_ZERO_REVISION).is_err());
    assert!(serde_json::from_str::<ResourceObservationV1>(INVALID_MIXED_VERSION).is_err());

    let mut extra: serde_json::Value = serde_json::from_str(VALID_OBSERVATION).expect("json");
    extra
        .as_object_mut()
        .expect("object")
        .insert("finding".into(), json!("stale"));
    assert!(serde_json::from_value::<ResourceObservationV1>(extra).is_err());
}

#[test]
fn revision_is_non_zero_ordered_and_identity_is_stable_across_change() {
    assert!(ResourceRevision::new(0).is_err());
    let prior = ResourceRevision::new(7).expect("revision");
    let current = ResourceRevision::new(8).expect("revision");
    assert!(current.is_later_than(prior));

    let installation_id = Uuid::from_u128(1);
    let reference = TypedResourceReference::new(
        installation_id,
        ResourceOwner::CoreInstallation { installation_id },
        ResourceTypeId::new("tessara.transition.component_version").expect("type"),
        "component-version-1",
    )
    .expect("reference");
    let contract = ProviderContractIdentity::new(
        "tessara.components.component-version"
            .parse()
            .expect("contract id"),
        "2.0.0".parse().expect("version"),
    );
    let before = ResourceObservationV1::new(
        reference.clone(),
        contract.clone(),
        ResourceObservationStrategy::LiveResolutionWithRevision,
        prior,
    );
    let after = ResourceObservationV1::new(
        reference,
        contract,
        ResourceObservationStrategy::LiveResolutionWithRevision,
        current,
    );
    assert_eq!(before.reference(), after.reference());
    assert_eq!(before.provider_contract(), after.provider_contract());
}
