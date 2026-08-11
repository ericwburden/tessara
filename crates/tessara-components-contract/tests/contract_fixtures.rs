use sha2::{Digest, Sha256};
use tessara_components_contract::{ComponentResolutionRequest, ComponentResolutionResponse};

const HISTORICAL_V1: &[u8] = include_bytes!("fixtures/historical-components-contract-v1.json");
const HISTORICAL_VALID_V2: &[u8] = include_bytes!("fixtures/valid-components-resolution-v2.json");
const HISTORICAL_INVALID_V2: &[u8] =
    include_bytes!("fixtures/invalid-components-restricted-disclosure-v2.json");
const INVALID_V1: &str = include_str!("fixtures/invalid-components-request-v1.json");
const VALID_V3: &str = include_str!("fixtures/valid-components-resolution-v3.json");
const INVALID_RESTRICTED_DISCLOSURE_V3: &str =
    include_str!("fixtures/invalid-components-restricted-disclosure-v3.json");

#[test]
fn historical_v1_and_v2_fixtures_remain_byte_pinned_without_runtime_readers() {
    assert_eq!(
        format!("sha256:{:x}", Sha256::digest(HISTORICAL_V1)),
        "sha256:621c07b815af4d822e96d7b10ce0344e87d1dac5a3b1eefd0966eee5f2e79117"
    );
    assert!(serde_json::from_slice::<ComponentResolutionRequest>(HISTORICAL_V1).is_err());
    assert_eq!(
        format!("sha256:{:x}", Sha256::digest(HISTORICAL_VALID_V2)),
        "sha256:68378b93d2f47a25e8c51086592f199ab757bfa590912485d82148b7654342a4"
    );
    assert_eq!(
        format!("sha256:{:x}", Sha256::digest(HISTORICAL_INVALID_V2)),
        "sha256:a4893164069d68630ed0138a2698b2ada1d41d3738c38379775964300496f306"
    );
    assert!(serde_json::from_slice::<ComponentResolutionResponse>(HISTORICAL_VALID_V2).is_err());
    assert!(serde_json::from_slice::<ComponentResolutionResponse>(HISTORICAL_INVALID_V2).is_err());
}

#[test]
fn exact_v3_golden_round_trips_and_invalid_shapes_fail_closed() {
    let parsed: ComponentResolutionResponse = serde_json::from_str(VALID_V3).expect("valid V3");
    assert_eq!(
        serde_json::to_value(parsed).expect("serialize"),
        serde_json::from_str::<serde_json::Value>(VALID_V3).expect("fixture JSON")
    );
    assert!(serde_json::from_str::<ComponentResolutionRequest>(INVALID_V1).is_err());
    assert!(
        serde_json::from_str::<ComponentResolutionResponse>(INVALID_RESTRICTED_DISCLOSURE_V3)
            .is_err()
    );
}
