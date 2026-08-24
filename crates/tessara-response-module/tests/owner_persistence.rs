use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use tessara_forms_contract::{FormVersionField, FormVersionSchemaResponse, FormVersionSection};
use tessara_response_module::{
    CreateResponseCommand, IdempotentCommit, ResponseAccess, ResponseListFilter,
    ResponseMutationCommand, ResponseOwnerError, ResponseOwnerRepository, ResponseValueInput,
    SaveResponseCommand, StartResponseClaimCommand, canonical_digest,
};
use tessara_responses_contract::{ResponseReference, ResponseWorkflowEvent};
use tessara_workflows_contract::{WorkflowResponseStartContext, WorkflowResponseStepSnapshot};
use uuid::Uuid;

fn create_command() -> CreateResponseCommand {
    let section_id = Uuid::from_u128(19);
    let form_snapshot = FormVersionSchemaResponse {
        schema_version: 1,
        form_id: Uuid::from_u128(4),
        form_version_id: Uuid::from_u128(5),
        form_name: "Customer intake".into(),
        form_slug: "customer-intake".into(),
        version_label: Some("Published".into()),
        version_major: Some(1),
        source_scope_node_ids: vec![Uuid::from_u128(6)],
        source_scope_revision: String::new(),
        source_scope_digest: String::new(),
        content_revision: String::new(),
        content_digest: String::new(),
        sections: vec![FormVersionSection {
            section_id,
            key: "customer".into(),
            label: "Customer".into(),
            description: "Customer details".into(),
            position: 0,
        }],
        fields: vec![FormVersionField {
            field_id: Uuid::from_u128(20),
            key: "name".into(),
            label: "Name".into(),
            field_type: "text".into(),
            required: true,
            options: Vec::new(),
            section_id: Some(section_id),
            position: 0,
            grid_row: 1,
            grid_column: 1,
            grid_width: 6,
            grid_height: 1,
        }],
    }
    .with_recomputed_digests()
    .unwrap();
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::minutes(5);
    let workflow_context = WorkflowResponseStartContext {
        schema_version: 1,
        workflow_assignment_id: Uuid::from_u128(7),
        workflow_id: Uuid::from_u128(21),
        workflow_name: "Customer onboarding".into(),
        workflow_description: "Onboard a customer".into(),
        workflow_version_id: Uuid::from_u128(8),
        workflow_version_label: Some("Published".into()),
        workflow_step_id: Uuid::from_u128(9),
        workflow_step_title: "Customer details".into(),
        workflow_step_position: 0,
        workflow_step_count: 2,
        next_workflow_step_title: Some("Review".into()),
        next_workflow_step_form_name: Some("Customer review".into()),
        history: vec![WorkflowResponseStepSnapshot {
            workflow_step_id: Uuid::from_u128(22),
            title: "Invite".into(),
            form_name: "Customer invite".into(),
            status: "completed".into(),
            position: 0,
            completed_at: Some("2026-08-23T12:00:00Z".into()),
        }],
        workflow_instance_id: Uuid::from_u128(10),
        workflow_step_instance_id: Uuid::from_u128(11),
        form_id: Uuid::from_u128(4),
        form_version_id: Uuid::from_u128(5),
        node_id: Uuid::from_u128(6),
        node_name: "North".into(),
        assignee_account_id: Uuid::from_u128(12),
        assignee_display_name: "Ada Lovelace".into(),
        started_by_account_id: Uuid::from_u128(13),
        delegation_basis: Some("direct_assignment".into()),
        one_use_nonce: Uuid::from_u128(23),
        issued_at: issued_at.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
        context_digest: String::new(),
    }
    .with_recomputed_digest()
    .unwrap();
    let form_snapshot = serde_json::to_value(form_snapshot).unwrap();
    let workflow_context = serde_json::to_value(workflow_context).unwrap();
    CreateResponseCommand {
        response: ResponseReference::from_parts(
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            Uuid::from_u128(3),
        )
        .unwrap(),
        form_id: Uuid::from_u128(4),
        form_version_id: Uuid::from_u128(5),
        node_id: Uuid::from_u128(6),
        workflow_assignment_id: Uuid::from_u128(7),
        workflow_version_id: Uuid::from_u128(8),
        workflow_step_id: Uuid::from_u128(9),
        workflow_instance_id: Uuid::from_u128(10),
        workflow_step_instance_id: Uuid::from_u128(11),
        workflow_start_nonce: Uuid::from_u128(23),
        assignee_account_id: Uuid::from_u128(12),
        started_by_account_id: Uuid::from_u128(13),
        delegation_basis: Some("direct_assignment".into()),
        form_snapshot_digest: canonical_digest(&form_snapshot).unwrap(),
        form_snapshot,
        workflow_context_digest: canonical_digest(&workflow_context).unwrap(),
        workflow_context,
        values: vec![ResponseValueInput {
            field_id: Uuid::from_u128(20),
            field_key: "name".into(),
            value: json!("Ada"),
            value_text: Some("Ada".into()),
        }],
        idempotency_key_digest: canonical_digest("start-idempotency-key").unwrap(),
        request_digest: canonical_digest(&json!({"assignment_id": Uuid::from_u128(7)})).unwrap(),
    }
}

fn distinct_command(seed: u128) -> CreateResponseCommand {
    let mut command = create_command();
    command.response = ResponseReference::from_parts(
        Uuid::from_u128(1),
        Uuid::from_u128(2),
        Uuid::from_u128(seed),
    )
    .unwrap();
    command.workflow_assignment_id = Uuid::from_u128(seed + 1);
    command.workflow_instance_id = Uuid::from_u128(seed + 2);
    command.workflow_step_instance_id = Uuid::from_u128(seed + 3);
    command.workflow_start_nonce = Uuid::from_u128(seed + 4);
    let mut workflow: WorkflowResponseStartContext =
        serde_json::from_value(command.workflow_context.clone()).unwrap();
    workflow.workflow_assignment_id = command.workflow_assignment_id;
    workflow.workflow_instance_id = command.workflow_instance_id;
    workflow.workflow_step_instance_id = command.workflow_step_instance_id;
    workflow.one_use_nonce = command.workflow_start_nonce;
    workflow = workflow.with_recomputed_digest().unwrap();
    command.workflow_context = serde_json::to_value(workflow).unwrap();
    command.workflow_context_digest = canonical_digest(&command.workflow_context).unwrap();
    command.idempotency_key_digest = canonical_digest(&format!("start-key-{seed}")).unwrap();
    command.request_digest = canonical_digest(&json!({"assignment_seed": seed})).unwrap();
    command
}

fn exact_respond_access(actor_account_id: Uuid, delegated: BTreeSet<Uuid>) -> ResponseAccess {
    ResponseAccess {
        installation_id: Uuid::from_u128(1),
        actor_account_id,
        delegated_account_ids: delegated,
        respond_node_ids: BTreeSet::from([Uuid::from_u128(6)]),
        respond_all: false,
        managed_node_ids: BTreeSet::new(),
        manage_all: false,
    }
}

fn exact_manage_access(actor_account_id: Uuid, node_id: Uuid) -> ResponseAccess {
    ResponseAccess {
        installation_id: Uuid::from_u128(1),
        actor_account_id,
        delegated_account_ids: BTreeSet::new(),
        respond_node_ids: BTreeSet::new(),
        respond_all: false,
        managed_node_ids: BTreeSet::from([node_id]),
        manage_all: false,
    }
}

async fn claim_start(repository: &ResponseOwnerRepository, command: &CreateResponseCommand) {
    repository.claim_start(&start_claim(command)).await.unwrap();
}

fn start_claim(command: &CreateResponseCommand) -> StartResponseClaimCommand {
    let workflow: WorkflowResponseStartContext =
        serde_json::from_value(command.workflow_context.clone()).unwrap();
    StartResponseClaimCommand {
        workflow_assignment_id: command.workflow_assignment_id,
        workflow_instance_id: command.workflow_instance_id,
        workflow_step_instance_id: command.workflow_step_instance_id,
        one_use_nonce: command.workflow_start_nonce,
        actor_account_id: command.started_by_account_id,
        idempotency_key_digest: command.idempotency_key_digest.clone(),
        request_digest: command.request_digest.clone(),
        authorization_grant_jti: Uuid::from_u128(30),
        authorization_correlation_id: Uuid::from_u128(31),
        expires_at: chrono::DateTime::parse_from_rfc3339(&workflow.expires_at)
            .unwrap()
            .with_timezone(&Utc),
    }
}

fn access(actor_account_id: Uuid) -> ResponseAccess {
    ResponseAccess {
        installation_id: Uuid::from_u128(1),
        actor_account_id,
        delegated_account_ids: BTreeSet::from([Uuid::from_u128(12)]),
        respond_node_ids: BTreeSet::new(),
        respond_all: true,
        managed_node_ids: BTreeSet::new(),
        manage_all: false,
    }
}

async fn seed_security(pool: &sqlx::PgPool) {
    sqlx::query("INSERT INTO response_module_security_state(singleton,schema_version,installation_id,module_instance_id,authorization_revision,organization_revision,enabled,document_state) VALUES(true,1,$1,$2,1,1,true,'enabled')")
        .bind(Uuid::from_u128(1))
        .bind(Uuid::from_u128(2))
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn create_is_atomic_audited_evented_and_idempotent(pool: sqlx::PgPool) {
    let repository = ResponseOwnerRepository::new(pool.clone());
    let command = create_command();
    claim_start(&repository, &command).await;
    let first = repository.create(&command).await.unwrap();
    let IdempotentCommit::Applied(snapshot) = first else {
        panic!("first create must apply");
    };
    snapshot.validate().unwrap();

    for (table, expected) in [
        ("responses", 1_i64),
        ("response_values", 1),
        ("response_audit_events", 1),
        ("response_idempotency_receipts", 1),
        ("response_workflow_events", 1),
    ] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, expected, "unexpected row count for {table}");
    }
    let event_payload: Value = sqlx::query_scalar("SELECT payload FROM response_workflow_events")
        .fetch_one(&pool)
        .await
        .unwrap();
    let event: ResponseWorkflowEvent = serde_json::from_value(event_payload).unwrap();
    event.validate().unwrap();
    let claim: (String, Option<Uuid>, Uuid, Uuid) = sqlx::query_as(
        "SELECT state,response_id,initial_grant_jti,initial_correlation_id
         FROM response_start_claims WHERE one_use_nonce=$1",
    )
    .bind(command.workflow_start_nonce)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        claim,
        (
            "committed".into(),
            Some(command.response.response_id()),
            Uuid::from_u128(30),
            Uuid::from_u128(31),
        )
    );
    let receipt_binding: (Uuid, Uuid) = sqlx::query_as(
        "SELECT initial_grant_jti,initial_correlation_id
         FROM response_idempotency_receipts WHERE idempotency_key_digest=$1",
    )
    .bind(&command.idempotency_key_digest)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(receipt_binding, (Uuid::from_u128(30), Uuid::from_u128(31)));

    let replay = repository.create(&command).await.unwrap();
    assert!(matches!(replay, IdempotentCommit::Replayed(value) if value == snapshot));
    let response_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM responses")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(response_count, 1);

    let mut conflicting = command;
    conflicting.request_digest = canonical_digest("different request").unwrap();
    assert!(matches!(
        repository.create(&conflicting).await,
        Err(ResponseOwnerError::IdempotencyConflict)
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_identical_start_is_one_apply_and_one_replay(pool: sqlx::PgPool) {
    let repository = ResponseOwnerRepository::new(pool.clone());
    let first_command = create_command();
    let mut second_command = first_command.clone();
    second_command.response =
        ResponseReference::from_parts(Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(300))
            .unwrap();

    let claim = start_claim(&first_command);
    let first_repository = repository.clone();
    let second_repository = repository.clone();
    let (first_claim, second_claim) = tokio::join!(
        first_repository.claim_start(&claim),
        second_repository.claim_start(&claim)
    );
    first_claim.unwrap();
    second_claim.unwrap();

    let first_repository = repository.clone();
    let second_repository = repository.clone();
    let (first, second) = tokio::join!(
        first_repository.create(&first_command),
        second_repository.create(&second_command)
    );
    let (first, second) = (first.unwrap(), second.unwrap());
    match (first, second) {
        (IdempotentCommit::Applied(applied), IdempotentCommit::Replayed(replayed))
        | (IdempotentCommit::Replayed(replayed), IdempotentCommit::Applied(applied)) => {
            assert_eq!(applied, replayed);
        }
        result => panic!("expected one applied start and one replay, got {result:?}"),
    }
    let counts: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM responses),(SELECT COUNT(*) FROM response_idempotency_receipts)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn create_requires_the_exact_live_start_claim(pool: sqlx::PgPool) {
    let repository = ResponseOwnerRepository::new(pool.clone());
    let command = create_command();
    assert!(matches!(
        repository.create(&command).await,
        Err(ResponseOwnerError::StartLeaseExpired)
    ));

    let claim = start_claim(&command);
    repository.claim_start(&claim).await.unwrap();
    repository.claim_start(&claim).await.unwrap();
    let mut refreshed_grant = claim.clone();
    refreshed_grant.authorization_grant_jti = Uuid::from_u128(32);
    refreshed_grant.authorization_correlation_id = Uuid::from_u128(33);
    repository.claim_start(&refreshed_grant).await.unwrap();
    let initial_binding: (Uuid, Uuid) = sqlx::query_as(
        "SELECT initial_grant_jti,initial_correlation_id
         FROM response_start_claims WHERE idempotency_key_digest=$1",
    )
    .bind(&claim.idempotency_key_digest)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(initial_binding, (Uuid::from_u128(30), Uuid::from_u128(31)));
    let mut conflicting = claim;
    conflicting.request_digest = canonical_digest("different start request").unwrap();
    assert!(matches!(
        repository.claim_start(&conflicting).await,
        Err(ResponseOwnerError::IdempotencyConflict)
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn expired_start_claim_cannot_create_a_response(pool: sqlx::PgPool) {
    let repository = ResponseOwnerRepository::new(pool.clone());
    let command = create_command();
    let mut claim = start_claim(&command);
    claim.expires_at = Utc::now() - Duration::seconds(1);
    assert!(matches!(
        repository.claim_start(&claim).await,
        Err(ResponseOwnerError::StartLeaseExpired)
    ));
    assert!(matches!(
        repository.create(&command).await,
        Err(ResponseOwnerError::StartLeaseExpired)
    ));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM responses")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn pinned_draft_saves_submits_and_exports_without_live_providers(pool: sqlx::PgPool) {
    seed_security(&pool).await;
    let repository = ResponseOwnerRepository::new(pool.clone());
    let command = create_command();
    claim_start(&repository, &command).await;
    repository.create(&command).await.unwrap();
    let actor = access(Uuid::from_u128(13));

    let listed = repository
        .list(&actor, &ResponseListFilter::default())
        .await
        .unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(
        listed[0].workflow_name.as_deref(),
        Some("Customer onboarding")
    );
    let detail = repository
        .detail(&actor, Uuid::from_u128(3))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(detail.form.sections[0].fields[0].grid_width, 6);
    assert_eq!(detail.revision, 1);

    let save = SaveResponseCommand {
        response_id: Uuid::from_u128(3),
        expected_revision: 1,
        values: BTreeMap::from([("name".into(), json!("Grace"))]),
        actor_account_id: Uuid::from_u128(13),
        idempotency_key_digest: canonical_digest("save-key").unwrap(),
        request_digest: canonical_digest(&json!({"revision": 1, "name": "Grace"})).unwrap(),
        authorization_grant_jti: Uuid::from_u128(32),
        authorization_correlation_id: Uuid::from_u128(33),
    };
    let IdempotentCommit::Applied(saved) = repository.save(&actor, &save).await.unwrap() else {
        panic!("first save must apply");
    };
    assert_eq!((saved.revision, saved.status.as_str()), (2, "draft"));
    assert!(
        matches!(repository.save(&actor, &save).await.unwrap(), IdempotentCommit::Replayed(value) if value == saved)
    );

    let submit = ResponseMutationCommand {
        response_id: Uuid::from_u128(3),
        expected_revision: 2,
        actor_account_id: Uuid::from_u128(13),
        idempotency_key_digest: canonical_digest("submit-key").unwrap(),
        request_digest: canonical_digest(&json!({"revision": 2})).unwrap(),
        authorization_grant_jti: Uuid::from_u128(34),
        authorization_correlation_id: Uuid::from_u128(35),
    };
    let IdempotentCommit::Applied(submitted) = repository.submit(&actor, &submit).await.unwrap()
    else {
        panic!("first submit must apply");
    };
    assert_eq!(
        (submitted.revision, submitted.status.as_str()),
        (3, "submitted")
    );
    assert!(
        matches!(repository.submit(&actor, &submit).await.unwrap(), IdempotentCommit::Replayed(value) if value == submitted)
    );

    for (table, expected) in [
        ("responses", 1_i64),
        ("response_values", 1),
        ("response_audit_events", 3),
        ("response_workflow_events", 3),
        ("response_export_changes", 1),
        ("response_idempotency_receipts", 3),
    ] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, expected, "unexpected row count for {table}");
    }
    let export: Value = sqlx::query_scalar("SELECT payload FROM response_export_changes")
        .fetch_one(&pool)
        .await
        .unwrap();
    let export: tessara_responses_contract::SubmittedResponseChange =
        serde_json::from_value(export).unwrap();
    export.validate_content_digest().unwrap();
    assert!(matches!(
        repository
            .save(
                &actor,
                &SaveResponseCommand {
                    expected_revision: 3,
                    idempotency_key_digest: canonical_digest("post-submit-save-key").unwrap(),
                    request_digest: canonical_digest(&json!({"revision": 3, "name": "Grace"}))
                        .unwrap(),
                    ..save
                }
            )
            .await,
        Err(ResponseOwnerError::Immutable)
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_identical_save_is_one_apply_and_one_replay(pool: sqlx::PgPool) {
    seed_security(&pool).await;
    let repository = ResponseOwnerRepository::new(pool.clone());
    let create = create_command();
    claim_start(&repository, &create).await;
    repository.create(&create).await.unwrap();
    let actor = access(Uuid::from_u128(13));
    let save = SaveResponseCommand {
        response_id: create.response.response_id(),
        expected_revision: 1,
        values: BTreeMap::from([("name".into(), json!("Grace"))]),
        actor_account_id: actor.actor_account_id,
        idempotency_key_digest: canonical_digest("concurrent-save-key").unwrap(),
        request_digest: canonical_digest("exact-save-wire-and-grant").unwrap(),
        authorization_grant_jti: Uuid::from_u128(36),
        authorization_correlation_id: Uuid::from_u128(37),
    };
    let first_repository = repository.clone();
    let second_repository = repository.clone();
    let (first, second) = tokio::join!(
        first_repository.save(&actor, &save),
        second_repository.save(&actor, &save)
    );
    match (first.unwrap(), second.unwrap()) {
        (IdempotentCommit::Applied(applied), IdempotentCommit::Replayed(replayed))
        | (IdempotentCommit::Replayed(replayed), IdempotentCommit::Applied(applied)) => {
            assert_eq!(applied, replayed);
            assert_eq!(applied.revision, 2);
        }
        result => panic!("expected one applied save and one replay, got {result:?}"),
    }
    let counts: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT revision FROM responses WHERE id=$1),(SELECT COUNT(*) FROM response_audit_events WHERE action='draft_saved'),(SELECT COUNT(*) FROM response_idempotency_receipts WHERE idempotency_key_digest=$2)",
    )
    .bind(create.response.response_id())
    .bind(&save.idempotency_key_digest)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (2, 1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn fresh_gateway_grant_replays_stable_request_and_preserves_initial_binding(
    pool: sqlx::PgPool,
) {
    seed_security(&pool).await;
    let repository = ResponseOwnerRepository::new(pool.clone());
    let create = create_command();
    claim_start(&repository, &create).await;
    repository.create(&create).await.unwrap();
    let actor = access(Uuid::from_u128(13));
    let initial = SaveResponseCommand {
        response_id: create.response.response_id(),
        expected_revision: 1,
        values: BTreeMap::from([("name".into(), json!("Grace"))]),
        actor_account_id: actor.actor_account_id,
        idempotency_key_digest: canonical_digest("fresh-gateway-grant-save-key").unwrap(),
        request_digest: canonical_digest("stable-exact-save-wire-and-authority").unwrap(),
        authorization_grant_jti: Uuid::from_u128(80),
        authorization_correlation_id: Uuid::from_u128(81),
    };
    let refreshed = SaveResponseCommand {
        authorization_grant_jti: Uuid::from_u128(82),
        authorization_correlation_id: Uuid::from_u128(83),
        ..initial.clone()
    };

    let IdempotentCommit::Applied(applied) = repository.save(&actor, &initial).await.unwrap()
    else {
        panic!("the initial gateway grant must apply");
    };
    let IdempotentCommit::Replayed(replayed) = repository.save(&actor, &refreshed).await.unwrap()
    else {
        panic!("a fresh valid gateway grant must replay the stable request");
    };
    assert_eq!(replayed, applied);
    let receipt: (Uuid, Uuid, i64) = sqlx::query_as(
        "SELECT initial_grant_jti,initial_correlation_id,
                (SELECT COUNT(*) FROM response_audit_events WHERE action='draft_saved')
         FROM response_idempotency_receipts WHERE idempotency_key_digest=$1",
    )
    .bind(&initial.idempotency_key_digest)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        receipt,
        (
            initial.authorization_grant_jti,
            initial.authorization_correlation_id,
            1,
        )
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_different_save_input_with_one_key_applies_once_and_conflicts(
    pool: sqlx::PgPool,
) {
    seed_security(&pool).await;
    let repository = ResponseOwnerRepository::new(pool.clone());
    let create = create_command();
    claim_start(&repository, &create).await;
    repository.create(&create).await.unwrap();
    let actor = access(Uuid::from_u128(13));
    let first = SaveResponseCommand {
        response_id: create.response.response_id(),
        expected_revision: 1,
        values: BTreeMap::from([("name".into(), json!("Grace"))]),
        actor_account_id: actor.actor_account_id,
        idempotency_key_digest: canonical_digest("concurrent-conflicting-save-key").unwrap(),
        request_digest: canonical_digest("first-exact-save-wire-and-grant").unwrap(),
        authorization_grant_jti: Uuid::from_u128(38),
        authorization_correlation_id: Uuid::from_u128(39),
    };
    let second = SaveResponseCommand {
        values: BTreeMap::from([("name".into(), json!("Katherine"))]),
        request_digest: canonical_digest("second-exact-save-wire-and-grant").unwrap(),
        ..first.clone()
    };

    let first_repository = repository.clone();
    let second_repository = repository.clone();
    let (first_result, second_result) = tokio::join!(
        first_repository.save(&actor, &first),
        second_repository.save(&actor, &second)
    );
    match (first_result, second_result) {
        (Ok(IdempotentCommit::Applied(applied)), Err(ResponseOwnerError::IdempotencyConflict))
        | (Err(ResponseOwnerError::IdempotencyConflict), Ok(IdempotentCommit::Applied(applied))) => {
            assert_eq!(applied.revision, 2);
        }
        result => panic!("expected one applied save and one idempotency conflict, got {result:?}"),
    }
    let counts: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT revision FROM responses WHERE id=$1),(SELECT COUNT(*) FROM response_audit_events WHERE action='draft_saved'),(SELECT COUNT(*) FROM response_idempotency_receipts WHERE idempotency_key_digest=$2)",
    )
    .bind(create.response.response_id())
    .bind(&first.idempotency_key_digest)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (2, 1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_identical_submit_is_one_apply_and_one_replay(pool: sqlx::PgPool) {
    seed_security(&pool).await;
    let repository = ResponseOwnerRepository::new(pool.clone());
    let create = create_command();
    claim_start(&repository, &create).await;
    repository.create(&create).await.unwrap();
    let actor = access(Uuid::from_u128(13));
    let submit = ResponseMutationCommand {
        response_id: create.response.response_id(),
        expected_revision: 1,
        actor_account_id: actor.actor_account_id,
        idempotency_key_digest: canonical_digest("concurrent-submit-key").unwrap(),
        request_digest: canonical_digest("exact-submit-wire-and-grant").unwrap(),
        authorization_grant_jti: Uuid::from_u128(40),
        authorization_correlation_id: Uuid::from_u128(41),
    };
    let first_repository = repository.clone();
    let second_repository = repository.clone();
    let (first, second) = tokio::join!(
        first_repository.submit(&actor, &submit),
        second_repository.submit(&actor, &submit)
    );
    match (first.unwrap(), second.unwrap()) {
        (IdempotentCommit::Applied(applied), IdempotentCommit::Replayed(replayed))
        | (IdempotentCommit::Replayed(replayed), IdempotentCommit::Applied(applied)) => {
            assert_eq!(applied, replayed);
            assert_eq!(applied.status, "submitted");
        }
        result => panic!("expected one applied submit and one replay, got {result:?}"),
    }
    let counts: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT revision FROM responses WHERE id=$1),(SELECT COUNT(*) FROM response_export_changes WHERE response_id=$1),(SELECT COUNT(*) FROM response_idempotency_receipts WHERE idempotency_key_digest=$2)",
    )
    .bind(create.response.response_id())
    .bind(&submit.idempotency_key_digest)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (2, 1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_identical_delete_is_one_apply_and_one_replay(pool: sqlx::PgPool) {
    seed_security(&pool).await;
    let repository = ResponseOwnerRepository::new(pool.clone());
    let create = create_command();
    claim_start(&repository, &create).await;
    repository.create(&create).await.unwrap();
    let actor = exact_respond_access(Uuid::from_u128(12), BTreeSet::new());
    let delete = ResponseMutationCommand {
        response_id: create.response.response_id(),
        expected_revision: 1,
        actor_account_id: actor.actor_account_id,
        idempotency_key_digest: canonical_digest("concurrent-delete-key").unwrap(),
        request_digest: canonical_digest("exact-delete-wire-and-grant").unwrap(),
        authorization_grant_jti: Uuid::from_u128(42),
        authorization_correlation_id: Uuid::from_u128(43),
    };
    let first_repository = repository.clone();
    let second_repository = repository.clone();
    let (first, second) = tokio::join!(
        first_repository.delete(&actor, &delete),
        second_repository.delete(&actor, &delete)
    );
    match (first.unwrap(), second.unwrap()) {
        (IdempotentCommit::Applied(applied), IdempotentCommit::Replayed(replayed))
        | (IdempotentCommit::Replayed(replayed), IdempotentCommit::Applied(applied)) => {
            assert_eq!(applied, replayed);
            assert_eq!(applied.status, "deleted");
        }
        result => panic!("expected one applied delete and one replay, got {result:?}"),
    }
    let counts: (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT revision FROM responses WHERE id=$1),(SELECT COUNT(*) FROM response_workflow_events WHERE response_id=$1 AND event_kind='deleted'),(SELECT COUNT(*) FROM response_idempotency_receipts WHERE idempotency_key_digest=$2)",
    )
    .bind(create.response.response_id())
    .bind(&delete.idempotency_key_digest)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (2, 1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn delete_authority_separates_respond_ownership_from_manage_scope(pool: sqlx::PgPool) {
    seed_security(&pool).await;
    let repository = ResponseOwnerRepository::new(pool.clone());
    let owner_response = distinct_command(400);
    let delegate_response = distinct_command(500);
    let manager_response = distinct_command(600);
    let denied_response = distinct_command(700);
    let stranger_response = distinct_command(800);
    for command in [
        &owner_response,
        &delegate_response,
        &manager_response,
        &denied_response,
        &stranger_response,
    ] {
        claim_start(&repository, command).await;
        repository.create(command).await.unwrap();
    }

    let owner = exact_respond_access(Uuid::from_u128(12), BTreeSet::new());
    let delegate = exact_respond_access(Uuid::from_u128(13), BTreeSet::from([Uuid::from_u128(12)]));
    let manager = exact_manage_access(Uuid::from_u128(14), Uuid::from_u128(6));
    for (index, (command, actor)) in [
        (&owner_response, &owner),
        (&delegate_response, &delegate),
        (&manager_response, &manager),
    ]
    .into_iter()
    .enumerate()
    {
        let result = repository
            .delete(
                actor,
                &ResponseMutationCommand {
                    response_id: command.response.response_id(),
                    expected_revision: 1,
                    actor_account_id: actor.actor_account_id,
                    idempotency_key_digest: canonical_digest(&format!("delete-mode-{index}"))
                        .unwrap(),
                    request_digest: canonical_digest(&format!("delete-mode-wire-{index}")).unwrap(),
                    authorization_grant_jti: Uuid::from_u128(50 + index as u128),
                    authorization_correlation_id: Uuid::from_u128(60 + index as u128),
                },
            )
            .await
            .unwrap();
        assert!(matches!(result, IdempotentCommit::Applied(value) if value.status == "deleted"));
    }

    let cross_mode = exact_manage_access(Uuid::from_u128(12), Uuid::from_u128(999));
    let denied = ResponseMutationCommand {
        response_id: denied_response.response.response_id(),
        expected_revision: 1,
        actor_account_id: cross_mode.actor_account_id,
        idempotency_key_digest: canonical_digest("cross-mode-delete-key").unwrap(),
        request_digest: canonical_digest("cross-mode-delete-wire").unwrap(),
        authorization_grant_jti: Uuid::from_u128(70),
        authorization_correlation_id: Uuid::from_u128(71),
    };
    assert!(matches!(
        repository.delete(&cross_mode, &denied).await,
        Err(ResponseOwnerError::NotFound)
    ));
    let state: (String, i64, i64) = sqlx::query_as(
        "SELECT status::text,revision,(SELECT COUNT(*) FROM response_idempotency_receipts WHERE idempotency_key_digest=$2) FROM responses WHERE id=$1",
    )
    .bind(denied.response_id)
    .bind(&denied.idempotency_key_digest)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(state, ("draft".into(), 1, 0));

    let scoped_stranger = exact_respond_access(Uuid::from_u128(99), BTreeSet::new());
    let stranger_denied = ResponseMutationCommand {
        response_id: stranger_response.response.response_id(),
        expected_revision: 1,
        actor_account_id: scoped_stranger.actor_account_id,
        idempotency_key_digest: canonical_digest("respond-cannot-manage-delete-key").unwrap(),
        request_digest: canonical_digest("respond-cannot-manage-delete-wire").unwrap(),
        authorization_grant_jti: Uuid::from_u128(72),
        authorization_correlation_id: Uuid::from_u128(73),
    };
    assert!(matches!(
        repository.delete(&scoped_stranger, &stranger_denied).await,
        Err(ResponseOwnerError::NotFound)
    ));
    let state: (String, i64, i64) = sqlx::query_as(
        "SELECT status::text,revision,(SELECT COUNT(*) FROM response_idempotency_receipts WHERE idempotency_key_digest=$2) FROM responses WHERE id=$1",
    )
    .bind(stranger_denied.response_id)
    .bind(&stranger_denied.idempotency_key_digest)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(state, ("draft".into(), 1, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn inaccessible_response_is_nondisclosing(pool: sqlx::PgPool) {
    let repository = ResponseOwnerRepository::new(pool.clone());
    let command = create_command();
    claim_start(&repository, &command).await;
    repository.create(&command).await.unwrap();
    let stranger = ResponseAccess {
        installation_id: Uuid::from_u128(1),
        actor_account_id: Uuid::from_u128(99),
        delegated_account_ids: BTreeSet::new(),
        respond_node_ids: BTreeSet::new(),
        respond_all: false,
        managed_node_ids: BTreeSet::new(),
        manage_all: false,
    };
    assert!(
        repository
            .detail(&stranger, Uuid::from_u128(3))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repository
            .detail(&stranger, Uuid::from_u128(404))
            .await
            .unwrap()
            .is_none()
    );
}
