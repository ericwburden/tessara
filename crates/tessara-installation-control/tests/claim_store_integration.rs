use chrono::{Duration, Utc};
use sqlx::postgres::PgPoolOptions;
use tessara_installation_control::{ClaimAccessError, InstallationControlStore};
use tessara_module_contract::{
    AdministratorEligibilityDecisionV1, AdministratorEnrollmentClaimKindV1,
    AdministratorEnrollmentClaimStateV1, EnrollmentRedemptionResultV1, ProtocolSignaturePurposeV1,
    PurposeBoundSigningKeyV1,
};
use uuid::Uuid;

#[tokio::test]
async fn claim_lifecycle_is_concurrent_idempotent_and_secret_free_after_issue() {
    let database_url = std::env::var("TEST_INSTALLATION_CONTROL_DATABASE_URL").expect(
        "TEST_INSTALLATION_CONTROL_DATABASE_URL is required for installation-control integration tests",
    );
    let pool = PgPoolOptions::new()
        .max_connections(6)
        .connect(&database_url)
        .await
        .expect("deployment test database is reachable");
    let store = InstallationControlStore::new(pool);
    store.migrate().await.expect("migrations apply");

    let installation_id = Uuid::new_v4();
    let eligibility_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "core-test-v1",
        ProtocolSignaturePurposeV1::EnrollmentEligibility,
        [71; 32],
    )
    .unwrap();
    let now = Utc::now();
    let eligibility = eligibility_signer
        .sign(AdministratorEligibilityDecisionV1 {
            schema_version: 1,
            installation_id,
            viable_administrator_exists: false,
            has_ever_had_viable_administrator: false,
            recovery_authorization: None,
            nonce: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .unwrap();
    let issued = store
        .issue(
            installation_id,
            AdministratorEnrollmentClaimKindV1::Initial,
            eligibility,
            &eligibility_signer.verifier(),
            None,
            now,
        )
        .await
        .expect("claim issues");
    let claim_id = issued.status.claim_id;
    let generation = issued.status.generation;
    let secret = issued.secret.expose_once();
    let status_json = serde_json::to_string(
        &store
            .status(installation_id)
            .await
            .expect("status reads")
            .expect("status exists"),
    )
    .unwrap();
    assert!(!status_json.contains(&secret));
    assert!(!status_json.contains("verifier"));

    let reservation_a = Uuid::new_v4();
    let reservation_b = Uuid::new_v4();
    let left = store.clone();
    let right = store.clone();
    let (left_result, right_result) = tokio::join!(
        left.reserve(
            installation_id,
            claim_id,
            generation,
            &secret,
            reservation_a,
            now + Duration::seconds(1),
        ),
        right.reserve(
            installation_id,
            claim_id,
            generation,
            &secret,
            reservation_b,
            now + Duration::seconds(1),
        )
    );
    let winner = match (left_result, right_result) {
        (Ok(_), Err(ClaimAccessError::EnrollmentUnavailable)) => reservation_a,
        (Err(ClaimAccessError::EnrollmentUnavailable), Ok(_)) => reservation_b,
        result => panic!("exactly one reservation must win: {result:?}"),
    };
    let result = EnrollmentRedemptionResultV1 {
        schema_version: 1,
        installation_id,
        claim_id,
        generation,
        reservation_id: winner,
        account_id: Uuid::new_v4(),
        role_id: Uuid::new_v4(),
        completed_at: now + Duration::seconds(2),
    };
    let consumed = store.consume(result.clone()).await.expect("claim consumes");
    assert_eq!(
        consumed.state,
        AdministratorEnrollmentClaimStateV1::Consumed
    );
    assert_eq!(
        store
            .consume(result)
            .await
            .expect("same result is idempotent"),
        consumed
    );
    assert_eq!(
        store
            .reserve(
                installation_id,
                claim_id,
                generation,
                &secret,
                Uuid::new_v4(),
                now + Duration::seconds(3),
            )
            .await,
        Err(ClaimAccessError::EnrollmentUnavailable)
    );
}
