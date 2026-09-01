use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    body::{Body, Bytes, to_bytes},
    extract::State,
    http::{HeaderMap, Request, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tessara_dataset_module::{
    DatasetCoreVerifiers, DatasetModuleState, DatasetServiceEndpoints,
    dependency_dag::rebuild_affected_published_closure_in_transaction, router,
};
use tessara_datasets_contract::{
    DATASET_IDEMPOTENCY_HEADER, DatasetProductFieldV1, DatasetProductSourceV1,
    DatasetRefreshResponseV1,
};
use tessara_module_contract::{
    AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2, AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
    AuthorizationAudienceV1, AuthorizationExchangeRequestV2, AuthorizationExchangeResponseV2,
    AuthorizationGrantOperationV1, AuthorizationGrantV3, AuthorizationValidationContextV3,
    CapabilityScopeBindingV1, DependencyBindingKey, FunctionalContractId, ModuleDefinitionId,
    ModuleServiceIdentityRegistryV1, ModuleServicePrincipalV1, ModuleServiceRequestV1,
    ModuleServiceRequestValidationContextV1, ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1,
    PurposeBoundVerifyingKeyV1, SecurityCapabilityId, SignedEnvelopeV1,
};
use tessara_responses_contract::{
    RESPONSE_EXPORT_BINDING_KEY, RESPONSE_EXPORT_CHECKPOINT_ACTION,
    RESPONSE_EXPORT_CHECKPOINT_PATH, RESPONSE_EXPORT_CONTRACT_ID, RESPONSE_EXPORT_MEDIA_TYPE,
    RESPONSE_EXPORT_PAGE_ACTION, RESPONSE_EXPORT_PAGE_PATH, RESPONSE_EXPORT_SCHEMA_VERSION,
    RESPONSE_EXPORT_START_ACTION, RESPONSE_EXPORT_START_PATH, RESPONSE_MODULE_DEFINITION_ID,
    ResponseExportAction, ResponseExportCheckpointRequest, ResponseExportCheckpointResponse,
    ResponseExportCursor, ResponseExportEntry, ResponseExportPageRequest,
    ResponseExportPageResponse, ResponseExportPartition, ResponseExportStartRequest,
    ResponseExportStartResponse, ResponseTombstoneReason, SubmittedResponseChange,
    SubmittedResponseRestrictionTier, SubmittedResponseUpsert, SubmittedResponseValue,
};
use tokio::sync::Barrier;
use tower::ServiceExt;
use uuid::Uuid;

const DIGEST: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
const EXCHANGE_PATH: &str = "/api/private/module-authorization/exchange";
const DATASET_BINDING: &str = "tessara.core.datasets";
const DATASET_CONTRACT: &str = "tessara.datasets.authoring";
const DATASET_DEFINITION: &str = "tessara.datasets";
const MANAGE_CAPABILITY: &str = "datasets:manage";

#[derive(Clone, Copy)]
struct DatasetIds {
    dataset_id: Uuid,
    revision_id: Uuid,
    source_binding_id: Uuid,
}

#[derive(Clone, Copy)]
struct FixtureIds {
    installation_id: Uuid,
    module_instance_id: Uuid,
    actor_id: Uuid,
    allowed_scope_id: Uuid,
    disjoint_scope_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    field_id: Uuid,
    provider_epoch: Uuid,
    initial_response_id: Uuid,
}

#[derive(Clone, Debug)]
struct PageSpec {
    entries: Vec<ResponseExportEntry>,
    next_after_cursor: Option<ResponseExportCursor>,
    complete: bool,
}

#[derive(Clone)]
struct PagePause {
    reached: Arc<Barrier>,
    release: Arc<Barrier>,
}

impl PagePause {
    fn new() -> Self {
        Self {
            reached: Arc::new(Barrier::new(2)),
            release: Arc::new(Barrier::new(2)),
        }
    }
}

#[derive(Clone)]
struct ProviderScenario {
    committed_cursor_valid: bool,
    authenticated_head: ResponseExportCursor,
    pages: HashMap<String, PageSpec>,
    fail_after_cursor: Option<String>,
    failures_remaining: usize,
    pause_after_cursor: Option<(String, PagePause)>,
}

impl ProviderScenario {
    fn unchanged(cursor: &ResponseExportCursor) -> Self {
        Self {
            committed_cursor_valid: true,
            authenticated_head: cursor.clone(),
            pages: HashMap::new(),
            fail_after_cursor: None,
            failures_remaining: 0,
            pause_after_cursor: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ObservedCall {
    Exchange {
        action: String,
    },
    Checkpoint {
        committed_cursor: Option<String>,
    },
    Start {
        committed_cursor: Option<String>,
        authenticated_head: String,
        full_snapshot_rebase: bool,
    },
    Page {
        after_cursor: Option<String>,
        snapshot_upper_bound: String,
    },
}

#[derive(Clone)]
struct MockProviders {
    core_authorization_signer: Arc<PurposeBoundSigningKeyV1>,
    dataset_service_verifier: PurposeBoundVerifyingKeyV1,
    ids: FixtureIds,
    root: DatasetIds,
    scenario: Arc<Mutex<ProviderScenario>>,
    calls: Arc<Mutex<Vec<ObservedCall>>>,
    consumed_service_nonces: Arc<Mutex<HashSet<Uuid>>>,
    consumed_provider_grants: Arc<Mutex<HashSet<Uuid>>>,
}

impl MockProviders {
    fn calls(&self) -> Vec<ObservedCall> {
        self.calls.lock().expect("provider call log").clone()
    }

    fn set_scenario(&self, scenario: ProviderScenario) {
        *self.scenario.lock().expect("provider scenario") = scenario;
        self.calls.lock().expect("provider call log").clear();
        self.consumed_service_nonces
            .lock()
            .expect("service nonce set")
            .clear();
        self.consumed_provider_grants
            .lock()
            .expect("provider grant set")
            .clear();
    }
}

struct RefreshFixture {
    pool: PgPool,
    app: Router,
    core_authorization_signer: Arc<PurposeBoundSigningKeyV1>,
    providers: MockProviders,
    ids: FixtureIds,
    root: DatasetIds,
    derived: Option<DatasetIds>,
    second_hop: Option<DatasetIds>,
    independent: Option<DatasetIds>,
    initial_cursor: ResponseExportCursor,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for RefreshFixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

impl RefreshFixture {
    fn authorization(
        &self,
        correlation_id: Uuid,
        jti: Uuid,
        authorized_scope_ids: &[Uuid],
    ) -> String {
        let now = Utc::now();
        let authorization = self
            .core_authorization_signer
            .sign(AuthorizationGrantV3 {
                schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
                installation_id: self.ids.installation_id,
                original_actor_id: self.ids.actor_id,
                correlation_id,
                presenting_service: ModuleServicePrincipalV1::CoreGateway,
                audience: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id: self.ids.module_instance_id,
                    module_definition_id: ModuleDefinitionId::new(DATASET_DEFINITION)
                        .expect("Dataset definition ID"),
                },
                dependency_binding: DependencyBindingKey::new(DATASET_BINDING)
                    .expect("Dataset binding"),
                functional_contract: FunctionalContractId::new(DATASET_CONTRACT)
                    .expect("Dataset contract"),
                action: "datasets.refresh".into(),
                operation: AuthorizationGrantOperationV1::Mutation,
                capability_scope_bindings: authorized_scope_ids
                    .iter()
                    .map(|scope_id| CapabilityScopeBindingV1 {
                        capability: SecurityCapabilityId::new(MANAGE_CAPABILITY)
                            .expect("Dataset manage capability"),
                        organization_root_id: *scope_id,
                        authorized_organization_ids: Vec::new(),
                    })
                    .collect(),
                resource_assertion: None,
                delegation_basis: Vec::new(),
                authorization_revision: 7,
                organization_revision: 11,
                jti,
                issued_at: now,
                expires_at: now + Duration::seconds(30),
            })
            .expect("sign Dataset refresh authorization");
        URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&authorization).expect("serialize Dataset refresh authorization"),
        )
    }

    async fn refresh(
        &self,
        dataset_id: Uuid,
        authorization: &str,
        correlation_id: Uuid,
        idempotency_key: &str,
    ) -> Response {
        self.app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/admin/datasets/{dataset_id}/refresh"))
                    .header("x-tessara-authorization", authorization)
                    .header("x-tessara-correlation-id", correlation_id.to_string())
                    .header(DATASET_IDEMPOTENCY_HEADER, idempotency_key)
                    .body(Body::empty())
                    .expect("Dataset refresh request"),
            )
            .await
            .expect("Dataset refresh response")
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn unchanged_head_short_circuits_before_start_or_page_and_preserves_published_state(
    pool: PgPool,
) {
    let fixture = refresh_fixture(pool, false).await;
    fixture
        .providers
        .set_scenario(ProviderScenario::unchanged(&fixture.initial_cursor));
    let before = stable_published_state(&fixture).await;
    let correlation_id = Uuid::new_v4();
    let authorization = fixture.authorization(
        correlation_id,
        Uuid::new_v4(),
        &[fixture.ids.allowed_scope_id],
    );

    let response = fixture
        .refresh(
            fixture.root.dataset_id,
            &authorization,
            correlation_id,
            "refresh-unchanged-head",
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let result: DatasetRefreshResponseV1 = response_json(response).await;
    assert!(!result.changed);
    assert!(result.materialization_receipt_ids.is_empty());
    assert_eq!(stable_published_state(&fixture).await, before);
    assert_eq!(idempotency_receipt_count(&fixture.pool).await, 1);
    let calls = fixture.providers.calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, ObservedCall::Checkpoint { .. }))
            .count(),
        1
    );
    assert!(
        !calls
            .iter()
            .any(|call| matches!(call, ObservedCall::Start { .. }))
    );
    assert!(
        !calls
            .iter()
            .any(|call| matches!(call, ObservedCall::Page { .. }))
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn ordered_fixed_bound_pages_promote_each_response_change_once(pool: PgPool) {
    let fixture = refresh_fixture(pool, false).await;
    let response_two = Uuid::new_v4();
    let cursor_two = cursor(2);
    let cursor_three = cursor(3);
    let cursor_four = cursor(4);
    let cursor_five = cursor(5);
    let first_page = page_spec(
        vec![
            upsert_entry(&fixture.ids, fixture.ids.initial_response_id, "old-2", 2),
            upsert_entry(&fixture.ids, response_two, "temporary", 3),
        ],
        Some(cursor_three.clone()),
        false,
    );
    let second_page = page_spec(
        vec![
            upsert_entry(
                &fixture.ids,
                fixture.ids.initial_response_id,
                "corrected",
                4,
            ),
            tombstone_entry(response_two, ResponseTombstoneReason::Deleted, 5),
        ],
        None,
        true,
    );
    fixture.providers.set_scenario(ProviderScenario {
        committed_cursor_valid: true,
        authenticated_head: cursor_five.clone(),
        pages: HashMap::from([
            (cursor_key(Some(&fixture.initial_cursor)), first_page),
            (cursor_key(Some(&cursor_three)), second_page),
        ]),
        fail_after_cursor: None,
        failures_remaining: 0,
        pause_after_cursor: None,
    });
    let before_receipts = materialization_receipt_count(&fixture.pool).await;
    let correlation_id = Uuid::new_v4();
    let authorization = fixture.authorization(
        correlation_id,
        Uuid::new_v4(),
        &[fixture.ids.allowed_scope_id],
    );

    let response = fixture
        .refresh(
            fixture.root.dataset_id,
            &authorization,
            correlation_id,
            "refresh-ordered-pages",
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let result: DatasetRefreshResponseV1 = response_json(response).await;
    assert!(result.changed);
    assert_eq!(result.materialization_receipt_ids.len(), 1);
    assert_eq!(
        materialization_receipt_count(&fixture.pool).await,
        before_receipts + 1
    );
    assert_partition(&fixture, 2, &cursor_five).await;
    assert_eq!(
        imported_value(&fixture, fixture.ids.initial_response_id).await,
        Some("corrected".into())
    );
    assert!(imported_tombstone(&fixture, response_two).await);
    assert_eq!(
        materialized_values(&fixture.pool, fixture.root.revision_id).await,
        vec!["corrected"]
    );
    let pages = fixture
        .providers
        .calls()
        .into_iter()
        .filter_map(|call| match call {
            ObservedCall::Page {
                after_cursor,
                snapshot_upper_bound,
            } => Some((after_cursor, snapshot_upper_bound)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        pages,
        vec![
            (
                Some(fixture.initial_cursor.as_str().into()),
                cursor_five.as_str().into()
            ),
            (
                Some(cursor_three.as_str().into()),
                cursor_five.as_str().into()
            ),
        ]
    );
    assert!(cursor_two < cursor_three && cursor_three < cursor_four && cursor_four < cursor_five);
}

#[sqlx::test(migrations = "./migrations")]
async fn interrupted_page_attempt_retry_converges_once_from_published_cursor(pool: PgPool) {
    let fixture = refresh_fixture(pool, false).await;
    let cursor_two = cursor(2);
    let cursor_three = cursor(3);
    fixture.providers.set_scenario(ProviderScenario {
        committed_cursor_valid: true,
        authenticated_head: cursor_three.clone(),
        pages: HashMap::from([
            (
                cursor_key(Some(&fixture.initial_cursor)),
                page_spec(
                    vec![upsert_entry(
                        &fixture.ids,
                        fixture.ids.initial_response_id,
                        "staged-not-published",
                        2,
                    )],
                    Some(cursor_two.clone()),
                    false,
                ),
            ),
            (
                cursor_key(Some(&cursor_two)),
                page_spec(
                    vec![upsert_entry(
                        &fixture.ids,
                        fixture.ids.initial_response_id,
                        "retried-final",
                        3,
                    )],
                    None,
                    true,
                ),
            ),
        ]),
        fail_after_cursor: Some(cursor_key(Some(&cursor_two))),
        failures_remaining: 2,
        pause_after_cursor: None,
    });
    let before = stable_published_state(&fixture).await;
    let first_correlation = Uuid::new_v4();
    let first_authorization = fixture.authorization(
        first_correlation,
        Uuid::new_v4(),
        &[fixture.ids.allowed_scope_id],
    );

    let interrupted = fixture
        .refresh(
            fixture.root.dataset_id,
            &first_authorization,
            first_correlation,
            "refresh-interrupted",
        )
        .await;

    assert_eq!(interrupted.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(stable_published_state(&fixture).await, before);
    assert_eq!(idempotency_receipt_count(&fixture.pool).await, 0);

    let retry_correlation = Uuid::new_v4();
    let retry_authorization = fixture.authorization(
        retry_correlation,
        Uuid::new_v4(),
        &[fixture.ids.allowed_scope_id],
    );
    let retried = fixture
        .refresh(
            fixture.root.dataset_id,
            &retry_authorization,
            retry_correlation,
            "refresh-interrupted-retry",
        )
        .await;

    assert_eq!(retried.status(), StatusCode::OK);
    let result: DatasetRefreshResponseV1 = response_json(retried).await;
    assert!(result.changed);
    assert_eq!(result.materialization_receipt_ids.len(), 1);
    assert_partition(&fixture, 2, &cursor_three).await;
    assert_eq!(materialization_receipt_count(&fixture.pool).await, 2);
    assert_eq!(idempotency_receipt_count(&fixture.pool).await, 1);
    assert_eq!(
        materialized_values(&fixture.pool, fixture.root.revision_id).await,
        vec!["retried-final"]
    );
    let starts = fixture
        .providers
        .calls()
        .into_iter()
        .filter_map(|call| match call {
            ObservedCall::Start {
                committed_cursor, ..
            } => Some(committed_cursor),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        starts,
        vec![
            Some(fixture.initial_cursor.as_str().into()),
            Some(fixture.initial_cursor.as_str().into()),
        ],
        "the retry must restart from published state, never hidden page staging"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_identical_refreshes_return_one_promotion_and_one_stored_replay(pool: PgPool) {
    let fixture = refresh_fixture(pool, false).await;
    let head = cursor(2);
    fixture.providers.set_scenario(ProviderScenario {
        committed_cursor_valid: true,
        authenticated_head: head.clone(),
        pages: HashMap::from([(
            cursor_key(Some(&fixture.initial_cursor)),
            page_spec(
                vec![upsert_entry(
                    &fixture.ids,
                    fixture.ids.initial_response_id,
                    "concurrent",
                    2,
                )],
                None,
                true,
            ),
        )]),
        fail_after_cursor: None,
        failures_remaining: 0,
        pause_after_cursor: None,
    });
    let correlation_id = Uuid::new_v4();
    let authorization = fixture.authorization(
        correlation_id,
        Uuid::new_v4(),
        &[fixture.ids.allowed_scope_id],
    );
    let first = fixture.refresh(
        fixture.root.dataset_id,
        &authorization,
        correlation_id,
        "refresh-concurrent-identical",
    );
    let second = fixture.refresh(
        fixture.root.dataset_id,
        &authorization,
        correlation_id,
        "refresh-concurrent-identical",
    );

    let (first, second) = tokio::join!(first, second);

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let first_bytes = response_bytes(first).await;
    let second_bytes = response_bytes(second).await;
    assert_eq!(first_bytes, second_bytes);
    let result: DatasetRefreshResponseV1 =
        serde_json::from_slice(&first_bytes).expect("canonical refresh response");
    assert!(result.changed);
    assert_eq!(result.materialization_receipt_ids.len(), 1);
    assert_partition(&fixture, 2, &head).await;
    assert_eq!(materialization_receipt_count(&fixture.pool).await, 2);
    assert_eq!(idempotency_receipt_count(&fixture.pool).await, 1);
    let calls = fixture.providers.calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, ObservedCall::Checkpoint { .. }))
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, ObservedCall::Start { .. }))
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, ObservedCall::Page { .. }))
            .count(),
        1
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn expired_cursor_forces_authenticated_full_rebase_and_atomic_partition_replacement(
    pool: PgPool,
) {
    let fixture = refresh_fixture(pool, false).await;
    let replacement_response = Uuid::new_v4();
    let head = cursor(3);
    let pause = PagePause::new();
    fixture.providers.set_scenario(ProviderScenario {
        committed_cursor_valid: false,
        authenticated_head: head.clone(),
        pages: HashMap::from([(
            cursor_key(None),
            page_spec(
                vec![upsert_entry(
                    &fixture.ids,
                    replacement_response,
                    "rebased",
                    3,
                )],
                None,
                true,
            ),
        )]),
        fail_after_cursor: None,
        failures_remaining: 0,
        pause_after_cursor: Some((cursor_key(None), pause.clone())),
    });
    let before = stable_published_state(&fixture).await;
    let correlation_id = Uuid::new_v4();
    let authorization = fixture.authorization(
        correlation_id,
        Uuid::new_v4(),
        &[fixture.ids.allowed_scope_id],
    );
    let app = fixture.app.clone();
    let root_id = fixture.root.dataset_id;
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/admin/datasets/{root_id}/refresh"))
        .header("x-tessara-authorization", authorization)
        .header("x-tessara-correlation-id", correlation_id.to_string())
        .header(DATASET_IDEMPOTENCY_HEADER, "refresh-expired-cursor")
        .body(Body::empty())
        .expect("expired cursor refresh request");
    let refresh =
        tokio::spawn(async move { app.oneshot(request).await.expect("refresh response") });

    pause.reached.wait().await;
    assert_eq!(stable_published_state(&fixture).await, before);
    assert_eq!(
        materialized_values(&fixture.pool, fixture.root.revision_id).await,
        vec!["old"]
    );
    pause.release.wait().await;
    let response = refresh.await.expect("join expired cursor refresh");

    let response_status = response.status();
    let response_bytes = response_bytes(response).await;
    assert_eq!(
        response_status,
        StatusCode::OK,
        "expired cursor refresh failed: {}",
        String::from_utf8_lossy(&response_bytes)
    );
    let result: DatasetRefreshResponseV1 =
        serde_json::from_slice(&response_bytes).expect("canonical expired cursor response");
    assert!(result.changed);
    assert_partition(&fixture, 2, &head).await;
    assert_eq!(
        imported_value(&fixture, fixture.ids.initial_response_id).await,
        None
    );
    assert_eq!(
        imported_value(&fixture, replacement_response).await,
        Some("rebased".into())
    );
    assert_eq!(
        materialized_values(&fixture.pool, fixture.root.revision_id).await,
        vec!["rebased"]
    );
    let calls = fixture.providers.calls();
    assert!(calls.iter().any(|call| {
        matches!(
            call,
            ObservedCall::Start {
                committed_cursor: None,
                full_snapshot_rebase: true,
                ..
            }
        )
    }));
    assert!(calls.iter().any(|call| {
        matches!(call, ObservedCall::Page { after_cursor: None, snapshot_upper_bound } if snapshot_upper_bound == head.as_str())
    }));
}

#[sqlx::test(migrations = "./migrations")]
async fn refresh_promotes_base_derived_second_hop_as_one_closure_and_preserves_independent_binding(
    pool: PgPool,
) {
    let fixture = refresh_fixture(pool, true).await;
    let head = cursor(2);
    fixture
        .providers
        .set_scenario(single_page_scenario(&fixture, head.clone(), "closure-new"));
    let independent = fixture.independent.expect("independent Dataset");
    let independent_before = dataset_materialized_state(&fixture.pool, independent).await;
    let correlation_id = Uuid::new_v4();
    let authorization = fixture.authorization(
        correlation_id,
        Uuid::new_v4(),
        &[fixture.ids.allowed_scope_id],
    );

    let response = fixture
        .refresh(
            fixture.root.dataset_id,
            &authorization,
            correlation_id,
            "refresh-complete-closure",
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let result: DatasetRefreshResponseV1 = response_json(response).await;
    assert!(result.changed);
    assert_eq!(result.materialization_receipt_ids.len(), 1);
    let derived = fixture.derived.expect("derived Dataset");
    let second_hop = fixture.second_hop.expect("second-hop Dataset");
    assert_eq!(
        materialized_values(&fixture.pool, fixture.root.revision_id).await,
        vec!["closure-new"]
    );
    assert_eq!(
        materialized_values(&fixture.pool, derived.revision_id).await,
        vec!["closure-new-derived"]
    );
    assert_eq!(
        materialized_values(&fixture.pool, second_hop.revision_id).await,
        vec!["closure-new-derived-second"]
    );
    assert_eq!(
        dataset_materialized_state(&fixture.pool, independent).await,
        independent_before
    );
    assert_partition(&fixture, 2, &head).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn derived_rebuild_failure_rolls_back_import_cursor_receipt_and_entire_closure(pool: PgPool) {
    let fixture = refresh_fixture(pool, true).await;
    let derived = fixture.derived.expect("derived Dataset");
    sqlx::query("UPDATE dataset_revisions SET generated_sql=$1 WHERE id=$2")
        .bind("SELECT * FROM dataset_materialized.missing_refresh_rebuild_input")
        .bind(derived.revision_id)
        .execute(&fixture.pool)
        .await
        .expect("inject derived rebuild failure");
    let before = stable_published_state(&fixture).await;
    let head = cursor(2);
    fixture
        .providers
        .set_scenario(single_page_scenario(&fixture, head, "must-not-publish"));
    let correlation_id = Uuid::new_v4();
    let authorization = fixture.authorization(
        correlation_id,
        Uuid::new_v4(),
        &[fixture.ids.allowed_scope_id],
    );

    let response = fixture
        .refresh(
            fixture.root.dataset_id,
            &authorization,
            correlation_id,
            "refresh-derived-failure",
        )
        .await;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(stable_published_state(&fixture).await, before);
    assert_eq!(idempotency_receipt_count(&fixture.pool).await, 0);
    let freshness: (String, Option<String>) = sqlx::query_as(
        "SELECT freshness_state,sanitized_failure_code
         FROM dataset_sync_partitions WHERE source_binding_id=$1",
    )
    .bind(fixture.root.source_binding_id)
    .fetch_one(&fixture.pool)
    .await
    .expect("failure freshness state");
    assert_eq!(
        freshness,
        (
            "degraded".into(),
            Some("dataset.dependency_unavailable".into())
        )
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn refresh_disjoint_restricted_known_and_random_sources_are_nondisclosing_and_write_nothing(
    pool: PgPool,
) {
    let fixture = refresh_fixture(pool, false).await;
    sqlx::query("UPDATE dataset_sources SET source_scope_node_ids=$2 WHERE dataset_id=$1")
        .bind(fixture.root.dataset_id)
        .bind(vec![fixture.ids.disjoint_scope_id])
        .execute(&fixture.pool)
        .await
        .expect("make root source scope disjoint");
    let restricted = insert_restricted_dataset(&fixture).await;
    fixture
        .providers
        .set_scenario(ProviderScenario::unchanged(&fixture.initial_cursor));
    let before = stable_published_state(&fixture).await;
    let correlation_id = Uuid::new_v4();
    let mut bodies = Vec::new();
    for (dataset_id, key) in [
        (fixture.root.dataset_id, "refresh-disjoint-source"),
        (restricted.dataset_id, "refresh-known-restricted"),
        (Uuid::new_v4(), "refresh-random-source"),
    ] {
        let authorization = fixture.authorization(
            correlation_id,
            Uuid::new_v4(),
            &[fixture.ids.allowed_scope_id],
        );
        let response = fixture
            .refresh(dataset_id, &authorization, correlation_id, key)
            .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        bodies.push(response_bytes(response).await);
    }
    assert!(bodies.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(stable_published_state(&fixture).await, before);
    assert_eq!(idempotency_receipt_count(&fixture.pool).await, 0);
    assert!(fixture.providers.calls().is_empty());
}

fn single_page_scenario(
    fixture: &RefreshFixture,
    head: ResponseExportCursor,
    value: &str,
) -> ProviderScenario {
    ProviderScenario {
        committed_cursor_valid: true,
        authenticated_head: head,
        pages: HashMap::from([(
            cursor_key(Some(&fixture.initial_cursor)),
            page_spec(
                vec![upsert_entry(
                    &fixture.ids,
                    fixture.ids.initial_response_id,
                    value,
                    2,
                )],
                None,
                true,
            ),
        )]),
        fail_after_cursor: None,
        failures_remaining: 0,
        pause_after_cursor: None,
    }
}

async fn refresh_fixture(pool: PgPool, with_chain: bool) -> RefreshFixture {
    let ids = FixtureIds {
        installation_id: Uuid::new_v4(),
        module_instance_id: Uuid::new_v4(),
        actor_id: Uuid::new_v4(),
        allowed_scope_id: Uuid::new_v4(),
        disjoint_scope_id: Uuid::new_v4(),
        form_id: Uuid::new_v4(),
        form_version_id: Uuid::new_v4(),
        field_id: Uuid::new_v4(),
        provider_epoch: Uuid::new_v4(),
        initial_response_id: Uuid::new_v4(),
    };
    let root = DatasetIds {
        dataset_id: Uuid::new_v4(),
        revision_id: Uuid::new_v4(),
        source_binding_id: Uuid::new_v4(),
    };
    let initial_cursor = cursor(1);
    sqlx::query(
        "INSERT INTO dataset_security_state
         (singleton,installation_id,module_instance_id,authorization_revision,
          organization_revision,enabled,document_state)
         VALUES(true,$1,$2,7,11,true,'enabled')",
    )
    .bind(ids.installation_id)
    .bind(ids.module_instance_id)
    .execute(&pool)
    .await
    .expect("install Dataset security state");

    let core_authorization_signer = Arc::new(signing_key(
        "tessara.core",
        "refresh-integration-core-authorization",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        [101; 32],
    ));
    let dataset_service_signer = Arc::new(signing_key(
        DATASET_DEFINITION,
        "refresh-integration-dataset-service",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [102; 32],
    ));
    let providers = MockProviders {
        core_authorization_signer: core_authorization_signer.clone(),
        dataset_service_verifier: dataset_service_signer.verifier(),
        ids,
        root,
        scenario: Arc::new(Mutex::new(ProviderScenario::unchanged(&initial_cursor))),
        calls: Arc::new(Mutex::new(Vec::new())),
        consumed_service_nonces: Arc::new(Mutex::new(HashSet::new())),
        consumed_provider_grants: Arc::new(Mutex::new(HashSet::new())),
    };
    let provider_router = Router::new()
        .route(EXCHANGE_PATH, post(mock_authorization_exchange))
        .route(RESPONSE_EXPORT_CHECKPOINT_PATH, post(mock_checkpoint))
        .route(RESPONSE_EXPORT_START_PATH, post(mock_start))
        .route(RESPONSE_EXPORT_PAGE_PATH, post(mock_page))
        .with_state(providers.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind refresh integration providers");
    let address = listener.local_addr().expect("refresh provider address");
    let server = tokio::spawn(async move {
        axum::serve(listener, provider_router)
            .await
            .expect("serve refresh integration providers");
    });
    let origin = format!("http://{address}");

    insert_root_dataset(&pool, ids, root, &initial_cursor).await;
    let (derived, second_hop, independent) = if with_chain {
        let derived = DatasetIds {
            dataset_id: Uuid::new_v4(),
            revision_id: Uuid::new_v4(),
            source_binding_id: Uuid::new_v4(),
        };
        let second_hop = DatasetIds {
            dataset_id: Uuid::new_v4(),
            revision_id: Uuid::new_v4(),
            source_binding_id: Uuid::new_v4(),
        };
        let independent = DatasetIds {
            dataset_id: Uuid::new_v4(),
            revision_id: Uuid::new_v4(),
            source_binding_id: Uuid::new_v4(),
        };
        insert_dependent_dataset(
            &pool,
            ids.allowed_scope_id,
            derived,
            "Derived",
            DatasetProductSourceV1::Dataset {
                alias: "base".into(),
                dataset_id: root.dataset_id.to_string(),
                dataset_revision_id: root.revision_id.to_string(),
            },
            &derived_revision_sql(root.revision_id, "derived"),
        )
        .await;
        insert_dependent_dataset(
            &pool,
            ids.allowed_scope_id,
            second_hop,
            "Second Hop",
            DatasetProductSourceV1::DatasetMajor {
                alias: "derived".into(),
                dataset_id: derived.dataset_id.to_string(),
                version_major: 1,
            },
            &derived_major_sql(derived.dataset_id, 1, "second"),
        )
        .await;
        insert_independent_dataset(&pool, ids, independent).await;
        (Some(derived), Some(second_hop), Some(independent))
    } else {
        (None, None, None)
    };

    let roots = independent
        .map(|independent| vec![root.dataset_id, independent.dataset_id])
        .unwrap_or_else(|| vec![root.dataset_id]);
    let mut transaction = pool.begin().await.expect("begin initial materialization");
    rebuild_affected_published_closure_in_transaction(&mut transaction, &roots)
        .await
        .expect("materialize initial Dataset closure");
    transaction
        .commit()
        .await
        .expect("commit initial materialization");

    let owner_bootstrap_signer = signing_key(
        "tessara.core",
        "refresh-integration-core-owner-bootstrap",
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        [103; 32],
    );
    let core_service_signer = signing_key(
        "tessara.core",
        "refresh-integration-core-service",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [104; 32],
    );
    let bootstrap_validation_signer = signing_key(
        "tessara.core",
        "refresh-integration-core-bootstrap-validation",
        ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
        [105; 32],
    );
    let shell_signer = signing_key(
        "tessara.core",
        "refresh-integration-core-shell",
        ProtocolSignaturePurposeV1::ShellContext,
        [106; 32],
    );
    let receipt_signer = Arc::new(signing_key(
        DATASET_DEFINITION,
        "refresh-integration-dataset-receipt",
        ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
        [107; 32],
    ));
    let registry = ModuleServiceIdentityRegistryV1::from_json(
        r#"{"schema_version":1,"identities":{"tessara.components":{"key_id":"refresh-integration-component","public_key":"11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"}}}"#,
    )
    .expect("refresh integration service registry");
    let provider_urls = tessara_dataset_module::REQUIRED_DATASET_PROVIDER_BINDINGS
        .into_iter()
        .map(|binding| (binding.to_owned(), origin.clone()))
        .collect::<BTreeMap<_, _>>();
    let state = DatasetModuleState::new(
        pool.clone(),
        DatasetCoreVerifiers {
            authorization: core_authorization_signer.verifier(),
            owner_bootstrap: owner_bootstrap_signer.verifier(),
            service_request: core_service_signer.verifier(),
            bootstrap_validation: bootstrap_validation_signer.verifier(),
            shell: shell_signer.verifier(),
        },
        registry,
        dataset_service_signer,
        receipt_signer,
        DatasetServiceEndpoints::new(&origin, provider_urls)
            .expect("refresh integration endpoints"),
        tessara_dataset_module::DatasetValidationFaultControl::disabled(),
    )
    .expect("refresh integration Dataset state");

    RefreshFixture {
        pool,
        app: router(state),
        core_authorization_signer,
        providers,
        ids,
        root,
        derived,
        second_hop,
        independent,
        initial_cursor,
        server,
    }
}

async fn insert_root_dataset(
    pool: &PgPool,
    ids: FixtureIds,
    root: DatasetIds,
    initial_cursor: &ResponseExportCursor,
) {
    insert_dataset_and_revision(
        pool,
        root,
        "Base",
        "base",
        ids.allowed_scope_id,
        &root_revision_sql(root.source_binding_id, ids.form_version_id, ids.field_id),
    )
    .await;
    let source_reference = DatasetProductSourceV1::Form {
        alias: "responses".into(),
        form_id: ids.form_id.to_string(),
        form_version_id: ids.form_version_id.to_string(),
    };
    insert_source(
        pool,
        root,
        "responses",
        "form_version",
        &source_reference,
        ids.allowed_scope_id,
    )
    .await;
    insert_revision_source(
        pool,
        root.revision_id,
        "responses",
        "form_version",
        &source_reference,
        ids.allowed_scope_id,
    )
    .await;
    sqlx::query(
        "INSERT INTO dataset_fields
         (id,dataset_id,key,label,source_alias,source_field_key,source_field_id,field_type,position)
         VALUES($1,$2,'value','Value','responses','value',$3,'text',0)",
    )
    .bind(Uuid::new_v4())
    .bind(root.dataset_id)
    .bind(ids.field_id)
    .execute(pool)
    .await
    .expect("insert root Dataset field");
    sqlx::query(
        "INSERT INTO dataset_sync_partitions
         (source_binding_id,provider_epoch,committed_cursor,committed_snapshot_upper_bound,
          generation,last_checked_at,last_succeeded_at,freshness_state)
         VALUES($1,$2,$3,$3,1,now(),now(),'current')",
    )
    .bind(root.source_binding_id)
    .bind(ids.provider_epoch)
    .bind(initial_cursor.as_str())
    .execute(pool)
    .await
    .expect("insert initial Response sync partition");
    let attempt_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dataset_sync_attempts
         (id,source_binding_id,provider_epoch,start_generation,start_cursor,snapshot_upper_bound,
          last_staged_cursor,full_snapshot_rebase,state,completed_at)
         VALUES($1,$2,$3,0,NULL,$4,$4,true,'promoted',now())",
    )
    .bind(attempt_id)
    .bind(root.source_binding_id)
    .bind(ids.provider_epoch)
    .bind(initial_cursor.as_str())
    .execute(pool)
    .await
    .expect("insert initial Response sync attempt");
    sqlx::query(
        "INSERT INTO dataset_materialization_receipts
         (id,source_binding_id,attempt_id,generation,committed_cursor,snapshot_upper_bound,
          input_digest,result_digest)
         VALUES($1,$2,$3,1,$4,$4,$5,$5)",
    )
    .bind(Uuid::new_v4())
    .bind(root.source_binding_id)
    .bind(attempt_id)
    .bind(initial_cursor.as_str())
    .bind(DIGEST)
    .execute(pool)
    .await
    .expect("insert initial materialization receipt");
    let change = submitted_upsert(&ids, ids.initial_response_id, "old");
    let SubmittedResponseChange::Upsert(upsert) = change else {
        unreachable!("submitted_upsert always returns an upsert")
    };
    sqlx::query(
        "INSERT INTO dataset_imported_responses
         (source_binding_id,response_id,form_id,form_version_id,node_id,node_name,status,
          submitted_at,created_at,last_modified_at,last_modified_by_user_name,
          restriction_tier,scope_node_ids,content_digest,tombstoned,promoted_generation)
         VALUES($1,$2,$3,$4,$5,$6,'submitted',$7::timestamptz,$8::timestamptz,
                $9::timestamptz,$10,'public',$11,$12,false,1)",
    )
    .bind(root.source_binding_id)
    .bind(upsert.response_id)
    .bind(upsert.form_id)
    .bind(upsert.form_version_id)
    .bind(upsert.node_id)
    .bind(&upsert.node_name)
    .bind(&upsert.submitted_at)
    .bind(&upsert.created_at)
    .bind(&upsert.last_modified_at)
    .bind(&upsert.last_modified_by_user_name)
    .bind(&upsert.scope_node_ids)
    .bind(&upsert.content_digest)
    .execute(pool)
    .await
    .expect("insert initial imported Response");
    let value = upsert.values.get("value").expect("initial Response value");
    sqlx::query(
        "INSERT INTO dataset_imported_response_values
         (source_binding_id,response_id,form_version_id,field_id,field_key,value_json,
          value_text,promoted_generation)
         VALUES($1,$2,$3,$4,'value',$5,$6,1)",
    )
    .bind(root.source_binding_id)
    .bind(upsert.response_id)
    .bind(upsert.form_version_id)
    .bind(value.field_id)
    .bind(&value.value)
    .bind(&value.value_text)
    .execute(pool)
    .await
    .expect("insert initial imported Response value");
}

async fn insert_dataset_and_revision(
    pool: &PgPool,
    dataset: DatasetIds,
    name: &str,
    slug: &str,
    scope_id: Uuid,
    generated_sql: &str,
) {
    sqlx::query("INSERT INTO datasets(id,name,slug,grain) VALUES($1,$2,$3,'submission')")
        .bind(dataset.dataset_id)
        .bind(name)
        .bind(slug)
        .execute(pool)
        .await
        .expect("insert Dataset");
    sqlx::query(
        "INSERT INTO dataset_scope_nodes
         (dataset_id,node_id,node_name,node_type_name,node_path,requested_set_revision,
          requested_set_digest)
         VALUES($1,$2,'Allowed Organization','Organization','/allowed','scope:1',$3)",
    )
    .bind(dataset.dataset_id)
    .bind(scope_id)
    .bind(DIGEST)
    .execute(pool)
    .await
    .expect("insert Dataset scope");
    let output_fields = vec![DatasetProductFieldV1 {
        key: "value".into(),
        label: "Value".into(),
        source_alias: "source".into(),
        source_field_key: "value".into(),
        field_type: "text".into(),
        position: 0,
    }];
    sqlx::query(
        "INSERT INTO dataset_revisions
         (id,dataset_id,version_number,version_label,version_major,version_minor,version_patch,
          semantic_bump,started_new_major_line,status,generated_sql,output_fields,published_at)
         VALUES($1,$2,1,'Version 1',1,0,0,'initial',true,'published',$3,$4,now())",
    )
    .bind(dataset.revision_id)
    .bind(dataset.dataset_id)
    .bind(generated_sql)
    .bind(serde_json::to_value(output_fields).expect("serialize Dataset output fields"))
    .execute(pool)
    .await
    .expect("insert published Dataset revision");
}

async fn insert_source(
    pool: &PgPool,
    dataset: DatasetIds,
    alias: &str,
    source_kind: &str,
    source_reference: &DatasetProductSourceV1,
    scope_id: Uuid,
) {
    let binding_source_kind = if source_kind == "form_version" {
        "response_export"
    } else {
        "dataset_major_line"
    };
    sqlx::query(
        "INSERT INTO dataset_source_bindings
         (id,dataset_id,binding_key,source_kind,source_identity,source_identity_digest)
         VALUES($1,$2,$3,$4,$5,$6)",
    )
    .bind(dataset.source_binding_id)
    .bind(dataset.dataset_id)
    .bind(format!("source:{alias}"))
    .bind(binding_source_kind)
    .bind(serde_json::to_value(source_reference).expect("serialize source identity"))
    .bind(DIGEST)
    .execute(pool)
    .await
    .expect("insert Dataset source binding");
    sqlx::query(
        "INSERT INTO dataset_sources
         (id,dataset_id,source_binding_id,source_alias,source_kind,source_reference,source_name,
          source_scope_node_ids,source_scope_revision,source_scope_digest,source_content_revision,
          source_content_digest,position)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,'scope:1',$9,'content:1',$9,0)",
    )
    .bind(Uuid::new_v4())
    .bind(dataset.dataset_id)
    .bind(dataset.source_binding_id)
    .bind(alias)
    .bind(source_kind)
    .bind(serde_json::to_value(source_reference).expect("serialize Dataset source"))
    .bind(alias)
    .bind(vec![scope_id])
    .bind(DIGEST)
    .execute(pool)
    .await
    .expect("insert Dataset source");
}

async fn insert_revision_source(
    pool: &PgPool,
    revision_id: Uuid,
    alias: &str,
    source_kind: &str,
    source_reference: &DatasetProductSourceV1,
    scope_id: Uuid,
) {
    sqlx::query(
        "INSERT INTO dataset_revision_sources
         (revision_id,source_alias,source_kind,source_reference,source_name,source_scope_node_ids,
          source_scope_revision,source_scope_digest,source_content_revision,source_content_digest,
          position)
         VALUES($1,$2,$3,$4,$2,$5,'scope:1',$6,'content:1',$6,0)",
    )
    .bind(revision_id)
    .bind(alias)
    .bind(source_kind)
    .bind(serde_json::to_value(source_reference).expect("serialize revision source"))
    .bind(vec![scope_id])
    .bind(DIGEST)
    .execute(pool)
    .await
    .expect("insert Dataset revision source");
}

async fn insert_dependent_dataset(
    pool: &PgPool,
    scope_id: Uuid,
    dataset: DatasetIds,
    name: &str,
    source: DatasetProductSourceV1,
    generated_sql: &str,
) {
    let slug = name.to_ascii_lowercase().replace(' ', "-");
    insert_dataset_and_revision(pool, dataset, name, &slug, scope_id, generated_sql).await;
    let (alias, source_kind) = match &source {
        DatasetProductSourceV1::Dataset { alias, .. } => (alias.as_str(), "dataset_revision"),
        DatasetProductSourceV1::DatasetMajor { alias, .. } => {
            (alias.as_str(), "dataset_major_line")
        }
        DatasetProductSourceV1::Form { .. } => unreachable!("dependent source is a Dataset"),
    };
    insert_source(pool, dataset, alias, source_kind, &source, scope_id).await;
    insert_revision_source(
        pool,
        dataset.revision_id,
        alias,
        source_kind,
        &source,
        scope_id,
    )
    .await;
    sqlx::query(
        "INSERT INTO dataset_sync_partitions
         (source_binding_id,generation,last_checked_at,last_succeeded_at,freshness_state)
         VALUES($1,0,now(),now(),'current')",
    )
    .bind(dataset.source_binding_id)
    .execute(pool)
    .await
    .expect("insert dependent source partition");
}

async fn insert_independent_dataset(pool: &PgPool, ids: FixtureIds, dataset: DatasetIds) {
    let sql = format!(
        "SELECT 'independent'::text AS __row_id, 'public'::text AS __restriction_tier, \
         ARRAY['{}'::uuid]::uuid[] AS __scope_node_ids, 'independent'::text AS value",
        ids.allowed_scope_id
    );
    insert_dataset_and_revision(
        pool,
        dataset,
        "Independent",
        "independent",
        ids.allowed_scope_id,
        &sql,
    )
    .await;
    let source = DatasetProductSourceV1::Form {
        alias: "independent".into(),
        form_id: Uuid::new_v4().to_string(),
        form_version_id: Uuid::new_v4().to_string(),
    };
    insert_source(
        pool,
        dataset,
        "independent",
        "form_version",
        &source,
        ids.allowed_scope_id,
    )
    .await;
    insert_revision_source(
        pool,
        dataset.revision_id,
        "independent",
        "form_version",
        &source,
        ids.allowed_scope_id,
    )
    .await;
    sqlx::query(
        "INSERT INTO dataset_sync_partitions
         (source_binding_id,generation,last_checked_at,last_succeeded_at,freshness_state)
         VALUES($1,0,now(),now(),'current')",
    )
    .bind(dataset.source_binding_id)
    .execute(pool)
    .await
    .expect("insert independent source partition");
}

async fn insert_restricted_dataset(fixture: &RefreshFixture) -> DatasetIds {
    let dataset = DatasetIds {
        dataset_id: Uuid::new_v4(),
        revision_id: Uuid::new_v4(),
        source_binding_id: Uuid::new_v4(),
    };
    let sql = format!(
        "SELECT 'restricted'::text AS __row_id, 'restricted'::text AS __restriction_tier, \
         ARRAY['{}'::uuid]::uuid[] AS __scope_node_ids, 'restricted'::text AS value",
        fixture.ids.disjoint_scope_id
    );
    insert_dataset_and_revision(
        &fixture.pool,
        dataset,
        "Restricted",
        &format!("restricted-{}", dataset.dataset_id.simple()),
        fixture.ids.disjoint_scope_id,
        &sql,
    )
    .await;
    dataset
}

fn root_revision_sql(source_binding_id: Uuid, form_version_id: Uuid, field_id: Uuid) -> String {
    format!(
        "SELECT imported.response_id::text AS __row_id, \
         imported.restriction_tier::text AS __restriction_tier, \
         imported.scope_node_ids AS __scope_node_ids, imported_value.value_text::text AS value \
         FROM dataset_imported_responses imported \
         LEFT JOIN dataset_imported_response_values imported_value \
           ON imported_value.source_binding_id=imported.source_binding_id \
          AND imported_value.response_id=imported.response_id \
          AND imported_value.form_version_id=imported.form_version_id \
          AND imported_value.field_id='{field_id}'::uuid \
         WHERE imported.source_binding_id='{source_binding_id}'::uuid \
           AND imported.form_version_id='{form_version_id}'::uuid \
           AND NOT imported.tombstoned"
    )
}

fn revision_table(revision_id: Uuid) -> String {
    format!("dataset_{}", revision_id.simple())
}

fn major_table(dataset_id: Uuid, major: i32) -> String {
    format!("dataset_major_{}_v{major}", dataset_id.simple())
}

fn qualified_table(table: &str) -> String {
    format!("\"dataset_materialized\".\"{table}\"")
}

fn derived_revision_sql(source_revision_id: Uuid, suffix: &str) -> String {
    format!(
        "SELECT concat('{suffix}:',__row_id)::text AS __row_id, __restriction_tier, \
         __scope_node_ids, concat(value,'-{suffix}')::text AS value FROM {}",
        qualified_table(&revision_table(source_revision_id))
    )
}

fn derived_major_sql(source_dataset_id: Uuid, source_major: i32, suffix: &str) -> String {
    format!(
        "SELECT concat('{suffix}:',__row_id)::text AS __row_id, __restriction_tier, \
         __scope_node_ids, concat(value,'-{suffix}')::text AS value FROM {}",
        qualified_table(&major_table(source_dataset_id, source_major))
    )
}

async fn mock_authorization_exchange(
    State(state): State<MockProviders>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let result = (|| {
        if content_type(&headers) != Some("application/json") {
            return None;
        }
        let encoded = header_text(&headers, "x-tessara-authorization")?;
        let inbound = decode_envelope::<AuthorizationGrantV3>(encoded)?;
        state
            .core_authorization_signer
            .verifier()
            .verify(&inbound)
            .ok()?;
        inbound
            .payload
            .validate_for(&AuthorizationValidationContextV3 {
                installation_id: state.ids.installation_id,
                correlation_id: inbound.payload.correlation_id,
                presenting_service: ModuleServicePrincipalV1::CoreGateway,
                audience: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id: state.ids.module_instance_id,
                    module_definition_id: ModuleDefinitionId::new(DATASET_DEFINITION).ok()?,
                },
                dependency_binding: DependencyBindingKey::new(DATASET_BINDING).ok()?,
                functional_contract: FunctionalContractId::new(DATASET_CONTRACT).ok()?,
                action: "datasets.refresh".into(),
                operation: AuthorizationGrantOperationV1::Mutation,
                resource_assertion: None,
                authorization_revision: 7,
                organization_revision: 11,
                now: Utc::now(),
            })
            .ok()?;
        validate_service_request(&state, &headers, EXCHANGE_PATH, &body, encoded)?;
        let request: AuthorizationExchangeRequestV2 = serde_json::from_slice(&body).ok()?;
        request.validate().ok()?;
        if request.target
            != (AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: tessara_composition::module_instance_id(
                    state.ids.installation_id,
                    RESPONSE_MODULE_DEFINITION_ID,
                ),
                module_definition_id: ModuleDefinitionId::new(RESPONSE_MODULE_DEFINITION_ID)
                    .ok()?,
            })
            || request.dependency_binding.as_str() != RESPONSE_EXPORT_BINDING_KEY
            || request.functional_contract.as_str() != RESPONSE_EXPORT_CONTRACT_ID
            || ![
                RESPONSE_EXPORT_CHECKPOINT_ACTION,
                RESPONSE_EXPORT_START_ACTION,
                RESPONSE_EXPORT_PAGE_ACTION,
            ]
            .contains(&request.action.as_str())
            || request.resource_assertion.is_some()
        {
            return None;
        }
        state
            .calls
            .lock()
            .expect("provider call log")
            .push(ObservedCall::Exchange {
                action: request.action.clone(),
            });
        let now = Utc::now();
        let authorization = state
            .core_authorization_signer
            .sign(AuthorizationGrantV3 {
                schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
                installation_id: state.ids.installation_id,
                original_actor_id: inbound.payload.original_actor_id,
                correlation_id: inbound.payload.correlation_id,
                presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                    module_instance_id: state.ids.module_instance_id,
                    module_definition_id: ModuleDefinitionId::new(DATASET_DEFINITION).ok()?,
                },
                audience: request.target,
                dependency_binding: request.dependency_binding,
                functional_contract: request.functional_contract,
                action: request.action,
                operation: AuthorizationGrantOperationV1::Read,
                capability_scope_bindings: inbound.payload.capability_scope_bindings,
                resource_assertion: None,
                delegation_basis: inbound.payload.delegation_basis,
                authorization_revision: 7,
                organization_revision: 11,
                jti: Uuid::new_v4(),
                issued_at: now,
                expires_at: now + Duration::seconds(60),
            })
            .ok()?;
        Some(AuthorizationExchangeResponseV2 {
            schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2,
            authorization,
        })
    })();
    match result {
        Some(response) => JsonResponse::json("application/json", &response),
        None => StatusCode::FORBIDDEN.into_response(),
    }
}

async fn mock_checkpoint(
    State(state): State<MockProviders>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(request) = authorize_provider_request::<ResponseExportCheckpointRequest>(
        &state,
        &headers,
        &body,
        RESPONSE_EXPORT_CHECKPOINT_ACTION,
        RESPONSE_EXPORT_CHECKPOINT_PATH,
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if request.schema_version != RESPONSE_EXPORT_SCHEMA_VERSION
        || request.action != ResponseExportAction::Checkpoint
        || !valid_partition(&state, &request.partition)
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let scenario = state.scenario.lock().expect("provider scenario").clone();
    state
        .calls
        .lock()
        .expect("provider call log")
        .push(ObservedCall::Checkpoint {
            committed_cursor: request
                .committed_cursor
                .as_ref()
                .map(|cursor| cursor.as_str().to_owned()),
        });
    let changed = !scenario.committed_cursor_valid
        || request.committed_cursor.as_ref() != Some(&scenario.authenticated_head);
    JsonResponse::json(
        RESPONSE_EXPORT_MEDIA_TYPE,
        &ResponseExportCheckpointResponse {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            provider_epoch: state.ids.provider_epoch,
            authenticated_head: scenario.authenticated_head,
            committed_cursor_valid: scenario.committed_cursor_valid,
            changed,
        },
    )
}

async fn mock_start(
    State(state): State<MockProviders>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(request) = authorize_provider_request::<ResponseExportStartRequest>(
        &state,
        &headers,
        &body,
        RESPONSE_EXPORT_START_ACTION,
        RESPONSE_EXPORT_START_PATH,
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let scenario = state.scenario.lock().expect("provider scenario").clone();
    if request.validate().is_err()
        || !valid_partition(&state, &request.partition)
        || request.provider_epoch != state.ids.provider_epoch
        || request.authenticated_head != scenario.authenticated_head
        || request.full_snapshot_rebase == scenario.committed_cursor_valid
        || (request.full_snapshot_rebase && request.committed_cursor.is_some())
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    state
        .calls
        .lock()
        .expect("provider call log")
        .push(ObservedCall::Start {
            committed_cursor: request
                .committed_cursor
                .as_ref()
                .map(|cursor| cursor.as_str().to_owned()),
            authenticated_head: request.authenticated_head.as_str().to_owned(),
            full_snapshot_rebase: request.full_snapshot_rebase,
        });
    JsonResponse::json(
        RESPONSE_EXPORT_MEDIA_TYPE,
        &ResponseExportStartResponse {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            provider_epoch: request.provider_epoch,
            start_after_cursor: request.committed_cursor,
            snapshot_upper_bound: request.authenticated_head,
            full_snapshot_rebase: request.full_snapshot_rebase,
        },
    )
}

async fn mock_page(
    State(state): State<MockProviders>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(request) = authorize_provider_request::<ResponseExportPageRequest>(
        &state,
        &headers,
        &body,
        RESPONSE_EXPORT_PAGE_ACTION,
        RESPONSE_EXPORT_PAGE_PATH,
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if request.schema_version != RESPONSE_EXPORT_SCHEMA_VERSION
        || request.action != ResponseExportAction::Page
        || !valid_partition(&state, &request.partition)
        || request.provider_epoch != state.ids.provider_epoch
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let after_key = cursor_key(request.after_cursor.as_ref());
    let (head, page, fail, pause) = {
        let mut scenario = state.scenario.lock().expect("provider scenario");
        if request.snapshot_upper_bound != scenario.authenticated_head {
            return StatusCode::NOT_FOUND.into_response();
        }
        let Some(page) = scenario.pages.get(&after_key).cloned() else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let fail = scenario.fail_after_cursor.as_deref() == Some(after_key.as_str())
            && scenario.failures_remaining > 0;
        if fail {
            scenario.failures_remaining -= 1;
        }
        let pause = if scenario
            .pause_after_cursor
            .as_ref()
            .is_some_and(|(key, _)| key == &after_key)
        {
            scenario.pause_after_cursor.take().map(|(_, pause)| pause)
        } else {
            None
        };
        (scenario.authenticated_head.clone(), page, fail, pause)
    };
    state
        .calls
        .lock()
        .expect("provider call log")
        .push(ObservedCall::Page {
            after_cursor: request
                .after_cursor
                .as_ref()
                .map(|cursor| cursor.as_str().to_owned()),
            snapshot_upper_bound: request.snapshot_upper_bound.as_str().to_owned(),
        });
    if fail {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if let Some(pause) = pause {
        pause.reached.wait().await;
        pause.release.wait().await;
    }
    let response = ResponseExportPageResponse {
        schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
        provider_epoch: state.ids.provider_epoch,
        snapshot_upper_bound: head,
        page_digest: ResponseExportPageResponse::canonical_page_digest(&page.entries)
            .expect("canonical mock Response page"),
        entries: page.entries,
        next_after_cursor: page.next_after_cursor,
        complete: page.complete,
    };
    response.validate().expect("valid mock Response page");
    JsonResponse::json(RESPONSE_EXPORT_MEDIA_TYPE, &response)
}

fn authorize_provider_request<T: serde::de::DeserializeOwned>(
    state: &MockProviders,
    headers: &HeaderMap,
    body: &[u8],
    action: &str,
    path: &str,
) -> Option<T> {
    if content_type(headers) != Some(RESPONSE_EXPORT_MEDIA_TYPE) {
        return None;
    }
    let encoded = header_text(headers, "x-tessara-authorization")?;
    let authorization = decode_envelope::<AuthorizationGrantV3>(encoded)?;
    state
        .core_authorization_signer
        .verifier()
        .verify(&authorization)
        .ok()?;
    authorization
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: state.ids.installation_id,
            correlation_id: authorization.payload.correlation_id,
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: state.ids.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(DATASET_DEFINITION).ok()?,
            },
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: tessara_composition::module_instance_id(
                    state.ids.installation_id,
                    RESPONSE_MODULE_DEFINITION_ID,
                ),
                module_definition_id: ModuleDefinitionId::new(RESPONSE_MODULE_DEFINITION_ID)
                    .ok()?,
            },
            dependency_binding: DependencyBindingKey::new(RESPONSE_EXPORT_BINDING_KEY).ok()?,
            functional_contract: FunctionalContractId::new(RESPONSE_EXPORT_CONTRACT_ID).ok()?,
            action: action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: 7,
            organization_revision: 11,
            now: Utc::now(),
        })
        .ok()?;
    validate_service_request(state, headers, path, body, encoded)?;
    if !state
        .consumed_provider_grants
        .lock()
        .expect("provider grant set")
        .insert(authorization.payload.jti)
    {
        return None;
    }
    serde_json::from_slice(body).ok()
}

fn validate_service_request(
    state: &MockProviders,
    headers: &HeaderMap,
    path: &str,
    body: &[u8],
    encoded_authorization: &str,
) -> Option<()> {
    let encoded = header_text(headers, "x-tessara-module-service-request")?;
    let request = decode_envelope::<ModuleServiceRequestV1>(encoded)?;
    state.dataset_service_verifier.verify(&request).ok()?;
    let correlation_id = header_text(headers, "x-tessara-correlation-id")?;
    request
        .payload
        .validate_for(&ModuleServiceRequestValidationContextV1 {
            installation_id: state.ids.installation_id,
            module_instance_id: state.ids.module_instance_id,
            module_definition_id: ModuleDefinitionId::new(DATASET_DEFINITION).ok()?,
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(body),
            inbound_grant_digest: sha256_hex(encoded_authorization.as_bytes()),
            correlation_id: correlation_id.into(),
            now: Utc::now(),
        })
        .ok()?;
    if !state
        .consumed_service_nonces
        .lock()
        .expect("service nonce set")
        .insert(request.payload.nonce)
    {
        return None;
    }
    Some(())
}

fn valid_partition(state: &MockProviders, partition: &ResponseExportPartition) -> bool {
    let presenting = ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id: state.ids.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(DATASET_DEFINITION)
            .expect("Dataset definition ID"),
    };
    partition.source_binding_id == state.root.source_binding_id
        && partition.form_version_ids == vec![state.ids.form_version_id]
        && partition.authorized_scope_node_ids == vec![state.ids.allowed_scope_id]
        && partition
            .validate_authorization_digest(&presenting, state.ids.installation_id)
            .is_ok()
}

struct JsonResponse;

impl JsonResponse {
    fn json<T: Serialize>(media_type: &'static str, value: &T) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, media_type)
            .body(Body::from(
                serde_json::to_vec(value).expect("serialize mock provider response"),
            ))
            .expect("mock provider response")
    }
}

fn content_type(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::CONTENT_TYPE)?.to_str().ok()
}

fn header_text<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

fn decode_envelope<T: serde::de::DeserializeOwned>(encoded: &str) -> Option<SignedEnvelopeV1<T>> {
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(encoded).ok()?).ok()
}

fn signing_key(
    issuer: &str,
    key_id: &str,
    purpose: ProtocolSignaturePurposeV1,
    secret: [u8; 32],
) -> PurposeBoundSigningKeyV1 {
    PurposeBoundSigningKeyV1::from_secret_bytes(issuer, key_id, purpose, secret)
        .expect("refresh integration signing key")
}

fn sha256_hex(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn cursor(sequence: u64) -> ResponseExportCursor {
    ResponseExportCursor::parse(format!("refresh:{sequence:020}"))
        .expect("canonical refresh cursor")
}

fn cursor_key(cursor: Option<&ResponseExportCursor>) -> String {
    cursor
        .map(|cursor| cursor.as_str().to_owned())
        .unwrap_or_else(|| "<start>".into())
}

fn submitted_upsert(ids: &FixtureIds, response_id: Uuid, value: &str) -> SubmittedResponseChange {
    SubmittedResponseChange::Upsert(Box::new(SubmittedResponseUpsert {
        response_id,
        form_id: ids.form_id,
        form_version_id: ids.form_version_id,
        node_id: ids.allowed_scope_id,
        node_name: "Allowed Organization".into(),
        submitted_at: "2026-08-13T12:01:00Z".into(),
        created_at: "2026-08-13T12:00:00Z".into(),
        last_modified_at: "2026-08-13T12:02:00Z".into(),
        last_modified_by_user_name: Some("Refresh Integration".into()),
        status: "submitted".into(),
        restriction_tier: SubmittedResponseRestrictionTier::Public,
        scope_node_ids: vec![ids.allowed_scope_id],
        values: BTreeMap::from([(
            "value".into(),
            SubmittedResponseValue {
                field_id: ids.field_id,
                value: json!(value),
                value_text: Some(value.into()),
            },
        )]),
        content_digest: String::new(),
    }))
    .with_recomputed_content_digest()
    .expect("canonical submitted Response upsert")
}

fn upsert_entry(
    ids: &FixtureIds,
    response_id: Uuid,
    value: &str,
    sequence: u64,
) -> ResponseExportEntry {
    ResponseExportEntry {
        cursor: cursor(sequence),
        change: submitted_upsert(ids, response_id, value),
    }
}

fn tombstone_entry(
    response_id: Uuid,
    reason: ResponseTombstoneReason,
    sequence: u64,
) -> ResponseExportEntry {
    ResponseExportEntry {
        cursor: cursor(sequence),
        change: SubmittedResponseChange::Tombstone {
            response_id,
            reason,
            content_digest: String::new(),
        }
        .with_recomputed_content_digest()
        .expect("canonical Response tombstone"),
    }
}

fn page_spec(
    entries: Vec<ResponseExportEntry>,
    next_after_cursor: Option<ResponseExportCursor>,
    complete: bool,
) -> PageSpec {
    PageSpec {
        entries,
        next_after_cursor,
        complete,
    }
}

async fn response_bytes(response: Response) -> Vec<u8> {
    to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read Dataset refresh response")
        .to_vec()
}

async fn response_json<T: serde::de::DeserializeOwned>(response: Response) -> T {
    serde_json::from_slice(&response_bytes(response).await).expect("canonical Dataset refresh JSON")
}

async fn materialization_receipt_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM dataset_materialization_receipts")
        .fetch_one(pool)
        .await
        .expect("materialization receipt count")
}

async fn idempotency_receipt_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM dataset_idempotency_receipts")
        .fetch_one(pool)
        .await
        .expect("idempotency receipt count")
}

async fn assert_partition(
    fixture: &RefreshFixture,
    expected_generation: i64,
    expected_cursor: &ResponseExportCursor,
) {
    let partition: (i64, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT generation,committed_cursor,committed_snapshot_upper_bound
         FROM dataset_sync_partitions WHERE source_binding_id=$1",
    )
    .bind(fixture.root.source_binding_id)
    .fetch_one(&fixture.pool)
    .await
    .expect("Response sync partition");
    assert_eq!(
        partition,
        (
            expected_generation,
            Some(expected_cursor.as_str().into()),
            Some(expected_cursor.as_str().into()),
        )
    );
}

async fn imported_value(fixture: &RefreshFixture, response_id: Uuid) -> Option<String> {
    sqlx::query_scalar(
        "SELECT value_text FROM dataset_imported_response_values
         WHERE source_binding_id=$1 AND response_id=$2 AND field_id=$3",
    )
    .bind(fixture.root.source_binding_id)
    .bind(response_id)
    .bind(fixture.ids.field_id)
    .fetch_optional(&fixture.pool)
    .await
    .expect("imported Response value")
    .flatten()
}

async fn imported_tombstone(fixture: &RefreshFixture, response_id: Uuid) -> bool {
    sqlx::query_scalar(
        "SELECT tombstoned FROM dataset_imported_responses
         WHERE source_binding_id=$1 AND response_id=$2",
    )
    .bind(fixture.root.source_binding_id)
    .bind(response_id)
    .fetch_one(&fixture.pool)
    .await
    .expect("imported Response tombstone")
}

async fn materialized_values(pool: &PgPool, revision_id: Uuid) -> Vec<String> {
    sqlx::query_scalar::<_, String>(&format!(
        "SELECT value FROM {} ORDER BY __row_id",
        qualified_table(&revision_table(revision_id))
    ))
    .fetch_all(pool)
    .await
    .expect("materialized Dataset values")
}

async fn dataset_materialized_state(pool: &PgPool, dataset: DatasetIds) -> Value {
    let metadata: (i64, i64, Option<String>, Option<i64>) = sqlx::query_as(
        "SELECT d.resource_revision,r.resource_revision,r.materialized_table,r.materialized_row_count
         FROM datasets d JOIN dataset_revisions r ON r.dataset_id=d.id
         WHERE d.id=$1 AND r.id=$2",
    )
    .bind(dataset.dataset_id)
    .bind(dataset.revision_id)
    .fetch_one(pool)
    .await
    .expect("Dataset materialized metadata");
    json!({
        "metadata": metadata,
        "values": materialized_values(pool, dataset.revision_id).await,
    })
}

async fn stable_published_state(fixture: &RefreshFixture) -> Value {
    let partitions: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(to_jsonb(state) ORDER BY state.source_binding_id),'[]'::jsonb)
         FROM (
           SELECT source_binding_id,provider_epoch,committed_cursor,committed_snapshot_upper_bound,
                  generation
           FROM dataset_sync_partitions
         ) state",
    )
    .fetch_one(&fixture.pool)
    .await
    .expect("stable partition state");
    let resources: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(to_jsonb(state) ORDER BY state.dataset_id,state.revision_id),'[]'::jsonb)
         FROM (
           SELECT d.id AS dataset_id,d.resource_revision AS dataset_resource_revision,
                  r.id AS revision_id,r.resource_revision AS revision_resource_revision,
                  r.materialized_schema,r.materialized_table,r.materialized_row_count
           FROM datasets d JOIN dataset_revisions r ON r.dataset_id=d.id
         ) state",
    )
    .fetch_one(&fixture.pool)
    .await
    .expect("stable Dataset resource state");
    let imports: Value = sqlx::query_scalar(
        "SELECT jsonb_build_object(
           'responses',COALESCE((
             SELECT jsonb_agg(to_jsonb(state) ORDER BY state.source_binding_id,state.response_id)
             FROM (
               SELECT source_binding_id,response_id,form_id,form_version_id,node_id,status,
                      restriction_tier,scope_node_ids,content_digest,tombstoned,tombstone_reason,
                      promoted_generation
               FROM dataset_imported_responses
             ) state
           ),'[]'::jsonb),
           'values',COALESCE((
             SELECT jsonb_agg(to_jsonb(state) ORDER BY state.source_binding_id,state.response_id,state.field_id)
             FROM (
               SELECT source_binding_id,response_id,form_version_id,field_id,field_key,value_json,
                      value_text,promoted_generation
               FROM dataset_imported_response_values
             ) state
           ),'[]'::jsonb)
         )",
    )
    .fetch_one(&fixture.pool)
    .await
    .expect("stable imported Response state");
    let attempts: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(to_jsonb(state) ORDER BY state.id),'[]'::jsonb)
         FROM (
           SELECT id,source_binding_id,provider_epoch,start_generation,start_cursor,
                  snapshot_upper_bound,last_staged_cursor,full_snapshot_rebase,state
           FROM dataset_sync_attempts
         ) state",
    )
    .fetch_one(&fixture.pool)
    .await
    .expect("stable sync attempt state");
    let receipts: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(to_jsonb(state) ORDER BY state.id),'[]'::jsonb)
         FROM (
           SELECT id,source_binding_id,attempt_id,generation,committed_cursor,
                  snapshot_upper_bound,input_digest,result_digest
           FROM dataset_materialization_receipts
         ) state",
    )
    .fetch_one(&fixture.pool)
    .await
    .expect("stable materialization receipt state");
    let mut materializations = BTreeMap::new();
    for dataset in [
        Some(fixture.root),
        fixture.derived,
        fixture.second_hop,
        fixture.independent,
    ]
    .into_iter()
    .flatten()
    {
        materializations.insert(
            dataset.dataset_id.to_string(),
            dataset_materialized_state(&fixture.pool, dataset).await,
        );
    }
    json!({
        "partitions": partitions,
        "resources": resources,
        "imports": imports,
        "attempts": attempts,
        "receipts": receipts,
        "materializations": materializations,
    })
}
