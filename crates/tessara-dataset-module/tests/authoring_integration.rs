use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, Ordering},
    },
};

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::State,
    http::{HeaderMap, Request, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use tessara_control_plane_contract::{
    CONTROL_PLANE_SCHEMA_VERSION, SCOPE_CATALOG_MEDIA_TYPE, SCOPE_CATALOG_PATH, ScopeCatalogNode,
    ScopeCatalogRequest, ScopeCatalogResponse,
};
use tessara_dataset_module::{
    DatasetCoreVerifiers, DatasetModuleState, DatasetServiceEndpoints, router,
};
use tessara_datasets_contract::{
    DATASET_AUTHORING_CONTRACT_ID, DATASET_CONTRACT_ID, DATASET_IDEMPOTENCY_HEADER,
    DatasetAuthoringRequestV1, DatasetDraftRevisionResponseV1, DatasetMutationIdResponseV1,
    DatasetProductOperationV1, DatasetProductProjectionFieldV1, DatasetProductSourceV1,
    DatasetSqlPreviewResponseV1,
};
use tessara_forms_contract::{
    FORM_VERSION_SCHEMA_MEDIA_TYPE, FORM_VERSION_SCHEMA_PATH, FORM_VERSION_SCHEMA_VERSION,
    FormVersionField, FormVersionSchemaRequest, FormVersionSchemaResponse, FormVersionSection,
};
use tessara_module_contract::{
    AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2, AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
    AuthorizationAudienceV1, AuthorizationExchangeRequestV2, AuthorizationExchangeResponseV2,
    AuthorizationGrantOperationV1, AuthorizationGrantV3, CapabilityScopeBindingV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId,
    ModuleServiceIdentityRegistryV1, ModuleServicePrincipalV1, ModuleServiceRequestV1,
    ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1, PurposeBoundVerifyingKeyV1,
    SecurityCapabilityId, SignedEnvelopeV1,
};
use tessara_responses_contract::{
    RESPONSE_EXPORT_CHECKPOINT_PATH, RESPONSE_EXPORT_MEDIA_TYPE, RESPONSE_EXPORT_PAGE_PATH,
    RESPONSE_EXPORT_SCHEMA_VERSION, RESPONSE_EXPORT_START_PATH, ResponseExportCheckpointRequest,
    ResponseExportCheckpointResponse, ResponseExportCursor, ResponseExportEntry,
    ResponseExportPageRequest, ResponseExportPageResponse, ResponseExportStartRequest,
    ResponseExportStartResponse, SubmittedResponseChange, SubmittedResponseRestrictionTier,
    SubmittedResponseUpsert, SubmittedResponseValue,
};
use tower::ServiceExt;
use uuid::Uuid;

const PROVIDER_NORMAL: u8 = 0;
const PROVIDER_FORM_FORBIDDEN: u8 = 1;
const PROVIDER_FORM_UNAVAILABLE: u8 = 2;
const PROVIDER_FORM_TRANSIENT: u8 = 3;
const PROVIDER_FORM_MALFORMED: u8 = 4;

#[derive(Clone)]
struct MockProviders {
    authorization_signer: Arc<PurposeBoundSigningKeyV1>,
    installation_id: Uuid,
    module_instance_id: Uuid,
    actor_id: Uuid,
    scope_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    form_field_id: Uuid,
    provider_epoch: Uuid,
    mode: Arc<AtomicU8>,
    calls: Arc<Mutex<HashMap<String, usize>>>,
    service_request_nonces: Arc<Mutex<Vec<Uuid>>>,
    service_request_verifier: PurposeBoundVerifyingKeyV1,
}

struct AuthoringFixture {
    app: Router,
    authorization_signer: Arc<PurposeBoundSigningKeyV1>,
    providers: MockProviders,
    installation_id: Uuid,
    module_instance_id: Uuid,
    actor_id: Uuid,
    scope_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for AuthoringFixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn authoring_preview_create_and_draft_are_owner_atomic_and_replay_safe(pool: sqlx::PgPool) {
    let fixture = authoring_fixture(pool.clone()).await;
    let payload = fixture.payload("Owner Authoring Proof", "owner-authoring-proof", "Score");

    let substituted_contract_correlation = Uuid::new_v4();
    let substituted_contract = fixture.authorization_for_contract(
        "datasets.preview_sql",
        AuthorizationGrantOperationV1::Read,
        substituted_contract_correlation,
        Uuid::new_v4(),
        DATASET_CONTRACT_ID,
    );
    let rejected = fixture
        .send_json(
            "/api/admin/datasets/sql-preview",
            &substituted_contract,
            substituted_contract_correlation,
            None,
            serde_json::to_vec(&payload).expect("serialize substituted-contract payload"),
        )
        .await;
    assert_eq!(rejected.status(), StatusCode::FORBIDDEN);
    assert_eq!(fixture.providers.call_count(), 0);
    assert_database_counts(&pool, (0, 0, 0, 0)).await;

    let preview = fixture
        .call_json(
            "/api/admin/datasets/sql-preview",
            "datasets.preview_sql",
            AuthorizationGrantOperationV1::Read,
            &payload,
            None,
            Uuid::new_v4(),
        )
        .await;
    assert_eq!(preview.status(), StatusCode::OK);
    let preview: DatasetSqlPreviewResponseV1 = response_json(preview).await;
    assert!(preview.generated_sql.contains("dataset_imported_responses"));
    assert!(preview.generated_sql.contains("\"__scope_node_ids\""));
    assert!(preview.generated_sql.contains("\"score\""));
    assert_database_counts(&pool, (0, 0, 0, 0)).await;

    let create_key = "authoring-create-replay";
    let create_correlation = Uuid::new_v4();
    let create_jti = Uuid::new_v4();
    let create_authorization = fixture.authorization(
        "datasets.create",
        AuthorizationGrantOperationV1::Mutation,
        create_correlation,
        create_jti,
    );
    let create_body = serde_json::to_vec(&payload).expect("serialize create payload");
    let create = fixture
        .send_json(
            "/api/admin/datasets",
            &create_authorization,
            create_correlation,
            Some(create_key),
            create_body.clone(),
        )
        .await;
    let create_status = create.status();
    let create_bytes = response_bytes(create).await;
    assert_eq!(
        create_status,
        StatusCode::CREATED,
        "unexpected create response: {}; provider calls: {:?}",
        String::from_utf8_lossy(&create_bytes),
        fixture.providers.calls()
    );
    let created: DatasetMutationIdResponseV1 =
        serde_json::from_slice(&create_bytes).expect("canonical create response");
    let dataset_id = Uuid::parse_str(&created.id).expect("created Dataset ID");

    let provider_calls_after_create = fixture.providers.call_count();
    let replay = fixture
        .send_json(
            "/api/admin/datasets",
            &create_authorization,
            create_correlation,
            Some(create_key),
            create_body.clone(),
        )
        .await;
    assert_eq!(replay.status(), StatusCode::CREATED);
    assert_eq!(response_bytes(replay).await, create_bytes);
    assert_eq!(
        fixture.providers.call_count(),
        provider_calls_after_create,
        "a stored replay must not repeat authorization exchange or provider work"
    );

    let mut mismatched_payload = payload.clone();
    mismatched_payload.name = "Changed replay body".into();
    let mismatch = fixture
        .send_json(
            "/api/admin/datasets",
            &create_authorization,
            create_correlation,
            Some(create_key),
            serde_json::to_vec(&mismatched_payload).expect("serialize mismatch payload"),
        )
        .await;
    assert_eq!(mismatch.status(), StatusCode::CONFLICT);
    assert_eq!(
        response_json_value(mismatch).await["error"]["code"],
        "dataset.idempotency_mismatch"
    );
    assert_eq!(fixture.providers.call_count(), provider_calls_after_create);

    let revision = sqlx::query_as::<_, (Uuid, String, String, i64, String)>(
        "SELECT id,generated_sql,materialized_table,materialized_row_count,status
         FROM dataset_revisions WHERE dataset_id=$1",
    )
    .bind(dataset_id)
    .fetch_one(&pool)
    .await
    .expect("created owner revision");
    assert_eq!(
        normalize_source_binding_ids(&revision.1),
        normalize_source_binding_ids(&preview.generated_sql),
        "create must persist the previewed compiler output except for rebinding the new Dataset's owner identity"
    );
    let persisted_source_binding_id: Uuid =
        sqlx::query_scalar("SELECT id FROM dataset_source_bindings WHERE dataset_id=$1")
            .bind(dataset_id)
            .fetch_one(&pool)
            .await
            .expect("persisted source binding ID");
    assert!(
        revision
            .1
            .contains(&persisted_source_binding_id.to_string())
    );
    assert_eq!(revision.3, 1);
    assert_eq!(revision.4, "published");
    assert!(
        revision
            .2
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    );
    let materialized_score: String = sqlx::query_scalar(&format!(
        "SELECT score FROM dataset_materialized.{}",
        revision.2
    ))
    .fetch_one(&pool)
    .await
    .expect("materialized Dataset row");
    assert_eq!(materialized_score, "42");
    assert_database_counts(&pool, (1, 1, 1, 1)).await;

    let mut changed = payload.clone();
    changed.operations = vec![DatasetProductOperationV1::Projection {
        fields: vec![DatasetProductProjectionFieldV1 {
            key: "score".into(),
            label: "Score revised".into(),
            input_field_key: Some("responses__score".into()),
            position: 0,
        }],
        position: 0,
    }];
    changed.version_label = Some("Review label change".into());

    let existing_preview = fixture
        .call_json(
            &format!("/api/admin/datasets/{dataset_id}/sql-preview"),
            "datasets.preview_existing_sql",
            AuthorizationGrantOperationV1::Read,
            &changed,
            None,
            Uuid::new_v4(),
        )
        .await;
    assert_eq!(existing_preview.status(), StatusCode::OK);
    let existing_preview: DatasetSqlPreviewResponseV1 = response_json(existing_preview).await;
    assert_eq!(
        normalize_source_binding_ids(&existing_preview.generated_sql),
        normalize_source_binding_ids(&preview.generated_sql)
    );
    assert!(
        existing_preview
            .generated_sql
            .contains(&persisted_source_binding_id.to_string())
    );

    let no_op_draft = fixture
        .call_json(
            &format!("/api/admin/datasets/{dataset_id}/draft-revision"),
            "datasets.save_draft_revision",
            AuthorizationGrantOperationV1::Mutation,
            &payload,
            Some("authoring-no-op-draft"),
            Uuid::new_v4(),
        )
        .await;
    assert_eq!(no_op_draft.status(), StatusCode::OK);
    let no_op_draft: DatasetDraftRevisionResponseV1 = response_json(no_op_draft).await;
    assert_eq!(no_op_draft.compatibility.patch_count, 0);
    assert_eq!(no_op_draft.compatibility.minor_count, 0);
    assert_eq!(no_op_draft.compatibility.major_count, 0);
    let provider_calls_before_no_op_publish = fixture.providers.call_count();
    let no_op_publish_path = format!(
        "/api/admin/datasets/{dataset_id}/revisions/{}/publish",
        no_op_draft.revision_id
    );
    let no_op_publish = fixture
        .call_empty(
            &no_op_publish_path,
            "datasets.publish_revision",
            "authoring-no-op-publish",
            Uuid::new_v4(),
        )
        .await;
    assert_eq!(no_op_publish.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response_json_value(no_op_publish).await["error"]["code"],
        "dataset.malformed_request"
    );
    assert_eq!(
        fixture.providers.call_count(),
        provider_calls_before_no_op_publish,
        "an empty changelog must be rejected before provider or materialization work"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM dataset_revisions WHERE dataset_id=$1 AND status='published'",
        )
        .bind(dataset_id)
        .fetch_one(&pool)
        .await
        .expect("published revision count after no-op rejection"),
        1
    );

    let draft_key = "authoring-draft-replay";
    let draft_correlation = Uuid::new_v4();
    let draft_authorization = fixture.authorization(
        "datasets.save_draft_revision",
        AuthorizationGrantOperationV1::Mutation,
        draft_correlation,
        Uuid::new_v4(),
    );
    let draft_body = serde_json::to_vec(&changed).expect("serialize draft payload");
    let draft_path = format!("/api/admin/datasets/{dataset_id}/draft-revision");
    let draft = fixture
        .send_json(
            &draft_path,
            &draft_authorization,
            draft_correlation,
            Some(draft_key),
            draft_body.clone(),
        )
        .await;
    assert_eq!(draft.status(), StatusCode::OK);
    let draft_bytes = response_bytes(draft).await;
    let draft: DatasetDraftRevisionResponseV1 =
        serde_json::from_slice(&draft_bytes).expect("canonical draft response");
    assert_eq!(draft.dataset_id, dataset_id.to_string());
    assert_eq!(draft.revision_id, no_op_draft.revision_id);
    assert_eq!(draft.compatibility.patch_count, 2);
    assert_eq!(draft.compatibility.minor_count, 0);
    assert_eq!(draft.compatibility.major_count, 0);

    let provider_calls_after_draft = fixture.providers.call_count();
    let draft_replay = fixture
        .send_json(
            &draft_path,
            &draft_authorization,
            draft_correlation,
            Some(draft_key),
            draft_body.clone(),
        )
        .await;
    assert_eq!(draft_replay.status(), StatusCode::OK);
    assert_eq!(response_bytes(draft_replay).await, draft_bytes);
    assert_eq!(fixture.providers.call_count(), provider_calls_after_draft);

    changed.name = "Mismatched draft body".into();
    let draft_mismatch = fixture
        .send_json(
            &draft_path,
            &draft_authorization,
            draft_correlation,
            Some(draft_key),
            serde_json::to_vec(&changed).expect("serialize draft mismatch"),
        )
        .await;
    assert_eq!(draft_mismatch.status(), StatusCode::CONFLICT);
    assert_eq!(fixture.providers.call_count(), provider_calls_after_draft);

    let stored_findings: Value = sqlx::query_scalar(
        "SELECT compatibility_findings FROM dataset_revisions
         WHERE dataset_id=$1 AND status='draft'",
    )
    .bind(dataset_id)
    .fetch_one(&pool)
    .await
    .expect("stored Dataset draft findings");
    assert_eq!(stored_findings.as_array().map(Vec::len), Some(2));
    let finding_codes = stored_findings
        .as_array()
        .expect("stored findings array")
        .iter()
        .map(|finding| finding["code"].as_str().expect("finding code"))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        finding_codes,
        std::collections::BTreeSet::from([
            "changed_output_field_label",
            "changed_projection_operation",
        ])
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM dataset_revisions WHERE dataset_id=$1 AND status='draft'",
        )
        .bind(dataset_id)
        .fetch_one(&pool)
        .await
        .expect("draft revision count"),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM dataset_idempotency_receipts")
            .fetch_one(&pool)
            .await
            .expect("authoring receipt count"),
        3
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn authoring_provider_failure_is_nondisclosing_and_writes_nothing(pool: sqlx::PgPool) {
    let fixture = authoring_fixture(pool.clone()).await;

    for (mode, expected_status) in [
        (PROVIDER_FORM_FORBIDDEN, StatusCode::FORBIDDEN),
        (PROVIDER_FORM_UNAVAILABLE, StatusCode::SERVICE_UNAVAILABLE),
    ] {
        fixture.providers.mode.store(mode, Ordering::SeqCst);
        let payload = fixture.payload(
            &format!("Provider failure {mode}"),
            &format!("provider-failure-{mode}"),
            "Score",
        );
        let response = fixture
            .call_json(
                "/api/admin/datasets",
                "datasets.create",
                AuthorizationGrantOperationV1::Mutation,
                &payload,
                Some(&format!("provider-failure-{mode}")),
                Uuid::new_v4(),
            )
            .await;
        let response_status = response.status();
        let body = response_json_value(response).await;
        assert_eq!(
            response_status,
            expected_status,
            "unexpected provider failure response: {body}; provider calls: {:?}",
            fixture.providers.calls()
        );
        assert_eq!(
            body["error"]["code"],
            if mode == PROVIDER_FORM_FORBIDDEN {
                "dataset.not_found_or_forbidden"
            } else {
                "dataset.dependency_unavailable"
            }
        );
        let body_text = body.to_string().to_ascii_lowercase();
        assert!(!body_text.contains("form"));
        assert!(!body_text.contains("provider-failure"));
        assert_database_counts(&pool, (0, 0, 0, 0)).await;
    }

    fixture
        .providers
        .mode
        .store(PROVIDER_NORMAL, Ordering::SeqCst);
    let payload = fixture.payload("Recovered authoring", "recovered-authoring", "Score");
    let recovery_correlation = Uuid::new_v4();
    let recovery_authorization = fixture.authorization(
        "datasets.create",
        AuthorizationGrantOperationV1::Mutation,
        recovery_correlation,
        Uuid::new_v4(),
    );
    let response = fixture
        .send_json(
            "/api/admin/datasets",
            &recovery_authorization,
            recovery_correlation,
            Some("provider-failure-1"),
            serde_json::to_vec(&payload).expect("serialize recovered payload"),
        )
        .await;
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "a failed provider attempt must not poison its idempotency key"
    );
    assert_database_counts(&pool, (1, 1, 1, 1)).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn safe_provider_retry_uses_fresh_service_nonce_and_never_retries_semantic_failure(
    pool: sqlx::PgPool,
) {
    let fixture = authoring_fixture(pool.clone()).await;
    fixture
        .providers
        .mode
        .store(PROVIDER_FORM_TRANSIENT, Ordering::SeqCst);
    let response = fixture
        .call_json(
            "/api/admin/datasets",
            "datasets.create",
            AuthorizationGrantOperationV1::Mutation,
            &fixture.payload("Transient recovery", "transient-recovery", "Score"),
            Some("transient-recovery"),
            Uuid::new_v4(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        fixture.providers.calls().get(FORM_VERSION_SCHEMA_PATH),
        Some(&2),
        "the configured limit of one retry must yield exactly two Form observations"
    );
    let nonces = fixture
        .providers
        .service_request_nonces
        .lock()
        .expect("service request nonce log")
        .clone();
    assert_eq!(nonces.len(), 2);
    assert_ne!(
        nonces[0], nonces[1],
        "every attempt must have a fresh nonce"
    );
    fixture
        .providers
        .mode
        .store(PROVIDER_FORM_MALFORMED, Ordering::SeqCst);
    let calls_before = fixture
        .providers
        .calls()
        .get(FORM_VERSION_SCHEMA_PATH)
        .copied()
        .unwrap_or_default();
    let response = fixture
        .call_json(
            "/api/admin/datasets",
            "datasets.create",
            AuthorizationGrantOperationV1::Mutation,
            &fixture.payload("Malformed provider", "malformed-provider", "Score"),
            Some("malformed-provider"),
            Uuid::new_v4(),
        )
        .await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_eq!(
        fixture
            .providers
            .calls()
            .get(FORM_VERSION_SCHEMA_PATH)
            .copied()
            .unwrap_or_default(),
        calls_before + 1,
        "a successful transport with incompatible semantics must not be retried"
    );
    assert_database_counts(&pool, (1, 1, 1, 1)).await;
}

impl AuthoringFixture {
    fn payload(&self, name: &str, slug: &str, label: &str) -> DatasetAuthoringRequestV1 {
        DatasetAuthoringRequestV1 {
            name: name.into(),
            slug: slug.into(),
            grain: "submission".into(),
            version_label: Some("Initial".into()),
            force_new_major_version: false,
            visibility_node_ids: vec![self.scope_id.to_string()],
            initial_source: DatasetProductSourceV1::Form {
                alias: "responses".into(),
                form_id: self.form_id.to_string(),
                form_version_id: self.form_version_id.to_string(),
            },
            operations: vec![DatasetProductOperationV1::Projection {
                fields: vec![DatasetProductProjectionFieldV1 {
                    key: "score".into(),
                    label: label.into(),
                    input_field_key: Some("responses__score".into()),
                    position: 0,
                }],
                position: 0,
            }],
            restriction_policy: None,
        }
    }

    async fn call_json<T: Serialize>(
        &self,
        path: &str,
        action: &str,
        operation: AuthorizationGrantOperationV1,
        payload: &T,
        idempotency_key: Option<&str>,
        correlation_id: Uuid,
    ) -> Response {
        let authorization = self.authorization(action, operation, correlation_id, Uuid::new_v4());
        self.send_json(
            path,
            &authorization,
            correlation_id,
            idempotency_key,
            serde_json::to_vec(payload).expect("serialize authoring request"),
        )
        .await
    }

    async fn send_json(
        &self,
        path: &str,
        authorization: &str,
        correlation_id: Uuid,
        idempotency_key: Option<&str>,
        body: Vec<u8>,
    ) -> Response {
        let mut request = Request::builder()
            .method("POST")
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-tessara-authorization", authorization)
            .header("x-tessara-correlation-id", correlation_id.to_string());
        if let Some(idempotency_key) = idempotency_key {
            request = request.header(DATASET_IDEMPOTENCY_HEADER, idempotency_key);
        }
        self.app
            .clone()
            .oneshot(request.body(Body::from(body)).expect("authoring request"))
            .await
            .expect("authoring response")
    }

    async fn call_empty(
        &self,
        path: &str,
        action: &str,
        idempotency_key: &str,
        correlation_id: Uuid,
    ) -> Response {
        let authorization = self.authorization(
            action,
            AuthorizationGrantOperationV1::Mutation,
            correlation_id,
            Uuid::new_v4(),
        );
        self.app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("x-tessara-authorization", authorization)
                    .header("x-tessara-correlation-id", correlation_id.to_string())
                    .header(DATASET_IDEMPOTENCY_HEADER, idempotency_key)
                    .body(Body::empty())
                    .expect("empty authoring request"),
            )
            .await
            .expect("empty authoring response")
    }

    fn authorization(
        &self,
        action: &str,
        operation: AuthorizationGrantOperationV1,
        correlation_id: Uuid,
        jti: Uuid,
    ) -> String {
        self.authorization_for_contract(
            action,
            operation,
            correlation_id,
            jti,
            DATASET_AUTHORING_CONTRACT_ID,
        )
    }

    fn authorization_for_contract(
        &self,
        action: &str,
        operation: AuthorizationGrantOperationV1,
        correlation_id: Uuid,
        jti: Uuid,
        functional_contract: &str,
    ) -> String {
        let now = Utc::now();
        let expires_at = now
            + Duration::seconds(match operation {
                AuthorizationGrantOperationV1::Read => 60,
                AuthorizationGrantOperationV1::Mutation => 30,
            });
        let authorization = self
            .authorization_signer
            .sign(AuthorizationGrantV3 {
                schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
                installation_id: self.installation_id,
                original_actor_id: self.actor_id,
                correlation_id,
                presenting_service: ModuleServicePrincipalV1::CoreGateway,
                audience: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id: self.module_instance_id,
                    module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                        .expect("Dataset definition ID"),
                },
                dependency_binding: DependencyBindingKey::new("tessara.core.datasets")
                    .expect("Dataset Core binding"),
                functional_contract: FunctionalContractId::new(functional_contract)
                    .expect("Dataset functional contract"),
                action: action.into(),
                operation,
                capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                    capability: SecurityCapabilityId::new("datasets:manage")
                        .expect("Dataset manage capability"),
                    organization_root_id: self.scope_id,
                    authorized_organization_ids: Vec::new(),
                }],
                resource_assertion: None,
                delegation_basis: Vec::new(),
                authorization_revision: 7,
                organization_revision: 11,
                jti,
                issued_at: now,
                expires_at,
            })
            .expect("signed authoring authorization");
        URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&authorization).expect("serialize authoring authorization"))
    }
}

impl MockProviders {
    fn record(&self, path: &str) {
        let mut calls = self.calls.lock().expect("provider call counter");
        *calls.entry(path.into()).or_default() += 1;
    }

    fn call_count(&self) -> usize {
        self.calls
            .lock()
            .expect("provider call counter")
            .values()
            .sum()
    }

    fn calls(&self) -> HashMap<String, usize> {
        self.calls.lock().expect("provider call counter").clone()
    }
}

async fn authoring_fixture(pool: sqlx::PgPool) -> AuthoringFixture {
    let installation_id = Uuid::new_v4();
    let module_instance_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let scope_id = Uuid::new_v4();
    let form_id = Uuid::new_v4();
    let form_version_id = Uuid::new_v4();
    let form_field_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO dataset_security_state
         (singleton,installation_id,module_instance_id,authorization_revision,
          organization_revision,enabled,document_state)
         VALUES(true,$1,$2,7,11,true,'enabled')",
    )
    .bind(installation_id)
    .bind(module_instance_id)
    .execute(&pool)
    .await
    .expect("install Dataset security state");

    let authorization_signer = Arc::new(signing_key(
        "tessara.core",
        "authoring-integration-core",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        [81; 32],
    ));
    let dataset_service_signer = Arc::new(signing_key(
        "tessara.datasets",
        "authoring-integration-dataset",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [86; 32],
    ));
    let providers = MockProviders {
        authorization_signer: authorization_signer.clone(),
        installation_id,
        module_instance_id,
        actor_id,
        scope_id,
        form_id,
        form_version_id,
        form_field_id,
        provider_epoch: Uuid::new_v4(),
        mode: Arc::new(AtomicU8::new(PROVIDER_NORMAL)),
        calls: Arc::new(Mutex::new(HashMap::new())),
        service_request_nonces: Arc::new(Mutex::new(Vec::new())),
        service_request_verifier: dataset_service_signer.verifier(),
    };
    let provider_router = Router::new()
        .route(
            "/api/private/module-authorization/exchange",
            post(mock_authorization_exchange),
        )
        .route(SCOPE_CATALOG_PATH, post(mock_scope_catalog))
        .route(FORM_VERSION_SCHEMA_PATH, post(mock_form_schema))
        .route(RESPONSE_EXPORT_CHECKPOINT_PATH, post(mock_checkpoint))
        .route(RESPONSE_EXPORT_START_PATH, post(mock_start))
        .route(RESPONSE_EXPORT_PAGE_PATH, post(mock_page))
        .with_state(providers.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock authoring providers");
    let address = listener.local_addr().expect("mock provider address");
    let server = tokio::spawn(async move {
        axum::serve(listener, provider_router)
            .await
            .expect("serve mock authoring providers");
    });
    let origin = format!("http://{address}");

    let owner_bootstrap_signer = signing_key(
        "tessara.core",
        "authoring-integration-core",
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        [82; 32],
    );
    let core_service_request_signer = signing_key(
        "tessara.core",
        "authoring-integration-core",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [83; 32],
    );
    let bootstrap_validation_signer = signing_key(
        "tessara.core",
        "authoring-integration-core",
        ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
        [84; 32],
    );
    let shell_signer = signing_key(
        "tessara.core",
        "authoring-integration-core",
        ProtocolSignaturePurposeV1::ShellContext,
        [85; 32],
    );
    let dataset_receipt_signer = Arc::new(signing_key(
        "tessara.datasets",
        "authoring-integration-dataset",
        ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
        [87; 32],
    ));
    let identity_registry = ModuleServiceIdentityRegistryV1::from_json(
        r#"{"schema_version":1,"identities":{"tessara.components":{"key_id":"authoring-integration-component","public_key":"11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"}}}"#,
    )
    .expect("authoring integration identity registry");
    let provider_urls = tessara_dataset_module::REQUIRED_DATASET_PROVIDER_BINDINGS
        .into_iter()
        .map(|binding| (binding.to_owned(), origin.clone()))
        .collect::<BTreeMap<_, _>>();
    let state = DatasetModuleState::new(
        pool,
        DatasetCoreVerifiers {
            authorization: authorization_signer.verifier(),
            owner_bootstrap: owner_bootstrap_signer.verifier(),
            service_request: core_service_request_signer.verifier(),
            bootstrap_validation: bootstrap_validation_signer.verifier(),
            shell: shell_signer.verifier(),
        },
        identity_registry,
        dataset_service_signer,
        dataset_receipt_signer,
        DatasetServiceEndpoints::new(&origin, provider_urls)
            .expect("authoring integration endpoints"),
        tessara_dataset_module::DatasetValidationFaultControl::disabled(),
    )
    .expect("authoring integration Dataset state");

    AuthoringFixture {
        app: router(state),
        authorization_signer,
        providers,
        installation_id,
        module_instance_id,
        actor_id,
        scope_id,
        form_id,
        form_version_id,
        server,
    }
}

async fn mock_authorization_exchange(
    State(state): State<MockProviders>,
    headers: HeaderMap,
    Json(request): Json<AuthorizationExchangeRequestV2>,
) -> Response {
    state.record("authorization_exchange");
    let Some(correlation_id) = header_uuid(&headers, "x-tessara-correlation-id") else {
        return StatusCode::FORBIDDEN.into_response();
    };
    if request.schema_version != AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let now = Utc::now();
    let authorization = state
        .authorization_signer
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: state.installation_id,
            original_actor_id: state.actor_id,
            correlation_id,
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: state.module_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                    .expect("Dataset definition ID"),
            },
            audience: request.target,
            dependency_binding: request.dependency_binding,
            functional_contract: request.functional_contract,
            action: request.action,
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("datasets:manage")
                    .expect("Dataset manage capability"),
                organization_root_id: state.scope_id,
                authorized_organization_ids: Vec::new(),
            }],
            resource_assertion: request.resource_assertion,
            delegation_basis: Vec::new(),
            authorization_revision: 7,
            organization_revision: 11,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .expect("sign downstream provider authorization");
    Json(AuthorizationExchangeResponseV2 {
        schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2,
        authorization,
    })
    .into_response()
}

async fn mock_scope_catalog(
    State(state): State<MockProviders>,
    Json(request): Json<ScopeCatalogRequest>,
) -> Response {
    state.record(SCOPE_CATALOG_PATH);
    if request.schema_version != CONTROL_PLANE_SCHEMA_VERSION
        || request.node_ids != vec![state.scope_id]
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    exact_json_response(
        SCOPE_CATALOG_MEDIA_TYPE,
        &ScopeCatalogResponse {
            schema_version: CONTROL_PLANE_SCHEMA_VERSION,
            nodes: vec![ScopeCatalogNode {
                node_id: state.scope_id,
                parent_node_id: None,
                node_type_key: "organization".into(),
                node_type_name: "Organization".into(),
                display_label: "North Region".into(),
                node_path: "/north-region".into(),
            }],
            requested_set_revision: "scope:1".into(),
            requested_set_digest: format!("sha256:{}", "a".repeat(64)),
        },
    )
}

async fn mock_form_schema(
    State(state): State<MockProviders>,
    headers: HeaderMap,
    Json(request): Json<FormVersionSchemaRequest>,
) -> Response {
    state.record(FORM_VERSION_SCHEMA_PATH);
    let Some(nonce) = service_request_nonce(&headers, &state.service_request_verifier) else {
        return StatusCode::FORBIDDEN.into_response();
    };
    state
        .service_request_nonces
        .lock()
        .expect("service request nonce log")
        .push(nonce);
    match state.mode.load(Ordering::SeqCst) {
        PROVIDER_FORM_FORBIDDEN => return StatusCode::NOT_FOUND.into_response(),
        PROVIDER_FORM_UNAVAILABLE => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
        PROVIDER_FORM_TRANSIENT => {
            state.mode.store(PROVIDER_NORMAL, Ordering::SeqCst);
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
        PROVIDER_FORM_MALFORMED => {
            return exact_json_response(
                FORM_VERSION_SCHEMA_MEDIA_TYPE,
                &json!({"schema_version": 1}),
            );
        }
        _ => {}
    }
    if request.form_version_id != state.form_version_id {
        return StatusCode::NOT_FOUND.into_response();
    }
    let section_id = Uuid::from_u128(601);
    let schema = FormVersionSchemaResponse {
        schema_version: FORM_VERSION_SCHEMA_VERSION,
        form_id: state.form_id,
        form_version_id: state.form_version_id,
        form_name: "Authoring Responses".into(),
        form_slug: "authoring-responses".into(),
        version_label: Some("Published".into()),
        version_major: Some(1),
        source_scope_node_ids: vec![state.scope_id],
        source_scope_revision: String::new(),
        source_scope_digest: String::new(),
        content_revision: String::new(),
        content_digest: String::new(),
        sections: vec![FormVersionSection {
            section_id,
            key: section_id.to_string(),
            label: "Metrics".into(),
            position: 0,
        }],
        fields: vec![FormVersionField {
            field_id: state.form_field_id,
            key: "score".into(),
            label: "Score".into(),
            field_type: "number".into(),
            required: true,
            options: Vec::new(),
            section_id: Some(section_id),
            position: 0,
            grid_row: 1,
            grid_column: 1,
        }],
    }
    .with_recomputed_digests()
    .expect("canonical FormVersion schema");
    exact_json_response(FORM_VERSION_SCHEMA_MEDIA_TYPE, &schema)
}

async fn mock_checkpoint(
    State(state): State<MockProviders>,
    Json(request): Json<ResponseExportCheckpointRequest>,
) -> Response {
    state.record(RESPONSE_EXPORT_CHECKPOINT_PATH);
    let head = ResponseExportCursor::parse("authoring:0001").expect("authoring cursor");
    exact_json_response(
        RESPONSE_EXPORT_MEDIA_TYPE,
        &ResponseExportCheckpointResponse {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            provider_epoch: state.provider_epoch,
            authenticated_head: head.clone(),
            committed_cursor_valid: true,
            changed: request.committed_cursor.as_ref() != Some(&head),
        },
    )
}

async fn mock_start(
    State(state): State<MockProviders>,
    Json(request): Json<ResponseExportStartRequest>,
) -> Response {
    state.record(RESPONSE_EXPORT_START_PATH);
    exact_json_response(
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
    Json(request): Json<ResponseExportPageRequest>,
) -> Response {
    state.record(RESPONSE_EXPORT_PAGE_PATH);
    let change = SubmittedResponseChange::Upsert(Box::new(SubmittedResponseUpsert {
        response_id: Uuid::from_u128(700),
        form_id: state.form_id,
        form_version_id: state.form_version_id,
        node_id: state.scope_id,
        node_name: "North Region".into(),
        submitted_at: "2026-08-13T12:01:00Z".into(),
        created_at: "2026-08-13T12:00:00Z".into(),
        last_modified_at: "2026-08-13T12:02:00Z".into(),
        last_modified_by_user_name: Some("Authoring Tester".into()),
        status: "submitted".into(),
        restriction_tier: SubmittedResponseRestrictionTier::Public,
        scope_node_ids: vec![state.scope_id],
        values: BTreeMap::from([(
            "score".into(),
            SubmittedResponseValue {
                field_id: state.form_field_id,
                value: json!(42),
                value_text: Some("42".into()),
            },
        )]),
        content_digest: String::new(),
    }))
    .with_recomputed_content_digest()
    .expect("canonical Response change");
    let entries = vec![ResponseExportEntry {
        cursor: ResponseExportCursor::parse("authoring:0001").expect("authoring cursor"),
        change,
    }];
    let response = ResponseExportPageResponse {
        schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
        provider_epoch: request.provider_epoch,
        snapshot_upper_bound: request.snapshot_upper_bound,
        page_digest: ResponseExportPageResponse::canonical_page_digest(&entries)
            .expect("canonical Response page digest"),
        entries,
        next_after_cursor: None,
        complete: true,
    };
    exact_json_response(RESPONSE_EXPORT_MEDIA_TYPE, &response)
}

fn exact_json_response<T: Serialize>(media_type: &'static str, value: &T) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media_type)
        .body(Body::from(
            serde_json::to_vec(value).expect("serialize mock provider response"),
        ))
        .expect("mock provider response")
}

fn header_uuid(headers: &HeaderMap, name: &str) -> Option<Uuid> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
}

fn service_request_nonce(
    headers: &HeaderMap,
    verifier: &PurposeBoundVerifyingKeyV1,
) -> Option<Uuid> {
    let encoded = headers
        .get("x-tessara-module-service-request")?
        .to_str()
        .ok()?;
    let envelope: SignedEnvelopeV1<ModuleServiceRequestV1> =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(encoded).ok()?).ok()?;
    verifier.verify(&envelope).ok()?;
    Some(envelope.payload.nonce)
}

fn signing_key(
    issuer: &str,
    key_id: &str,
    purpose: ProtocolSignaturePurposeV1,
    secret: [u8; 32],
) -> PurposeBoundSigningKeyV1 {
    PurposeBoundSigningKeyV1::from_secret_bytes(issuer, key_id, purpose, secret)
        .expect("authoring integration signing key")
}

async fn response_bytes(response: Response) -> Vec<u8> {
    to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read authoring response")
        .to_vec()
}

async fn response_json<T: serde::de::DeserializeOwned>(response: Response) -> T {
    serde_json::from_slice(&response_bytes(response).await).expect("canonical authoring JSON")
}

async fn response_json_value(response: Response) -> Value {
    response_json(response).await
}

async fn assert_database_counts(pool: &sqlx::PgPool, expected: (i64, i64, i64, i64)) {
    let actual = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT
          (SELECT COUNT(*) FROM datasets),
          (SELECT COUNT(*) FROM dataset_revisions),
          (SELECT COUNT(*) FROM dataset_idempotency_receipts),
          (SELECT COUNT(*) FROM dataset_materialization_receipts)",
    )
    .fetch_one(pool)
    .await
    .expect("Dataset authoring database counts");
    assert_eq!(actual, expected);
}

fn normalize_source_binding_ids(sql: &str) -> String {
    let marker = "imported.source_binding_id = '";
    let mut normalized = sql.to_owned();
    let mut search_from = 0;
    while let Some(relative_start) = normalized[search_from..].find(marker) {
        let value_start = search_from + relative_start + marker.len();
        let Some(relative_end) = normalized[value_start..].find('\'') else {
            break;
        };
        let value_end = value_start + relative_end;
        normalized.replace_range(value_start..value_end, "<source-binding-id>");
        search_from = value_start + "<source-binding-id>".len();
    }
    normalized
}
