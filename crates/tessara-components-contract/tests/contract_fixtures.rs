use sha2::{Digest, Sha256};
use tessara_components_contract::{ComponentResolutionRequest, ComponentResolutionResponse};

const HISTORICAL_V1: &[u8] = include_bytes!("fixtures/historical-components-contract-v1.json");
const VALID_V2: &str = include_str!("fixtures/valid-components-resolution-v2.json");
const INVALID_V1: &str = include_str!("fixtures/invalid-components-request-v1.json");
const INVALID_RESTRICTED_DISCLOSURE: &str =
    include_str!("fixtures/invalid-components-restricted-disclosure-v2.json");

#[test]
fn historical_v1_fixture_remains_byte_pinned_but_has_no_runtime_reader() {
    assert_eq!(
        format!("sha256:{:x}", Sha256::digest(HISTORICAL_V1)),
        "sha256:621c07b815af4d822e96d7b10ce0344e87d1dac5a3b1eefd0966eee5f2e79117"
    );
    assert!(serde_json::from_slice::<ComponentResolutionRequest>(HISTORICAL_V1).is_err());
}

#[test]
fn exact_v2_golden_round_trips_and_invalid_shapes_fail_closed() {
    let parsed: ComponentResolutionResponse = serde_json::from_str(VALID_V2).expect("valid V2");
    assert_eq!(
        serde_json::to_value(parsed).expect("serialize"),
        serde_json::from_str::<serde_json::Value>(VALID_V2).expect("fixture JSON")
    );
    assert!(serde_json::from_str::<ComponentResolutionRequest>(INVALID_V1).is_err());
    assert!(
        serde_json::from_str::<ComponentResolutionResponse>(INVALID_RESTRICTED_DISCLOSURE).is_err()
    );
}
