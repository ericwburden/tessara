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
    let claim: (String, Option<Uuid>) = sqlx::query_as(
        "SELECT state,response_id FROM response_start_claims WHERE one_use_nonce=$1",
    )
    .bind(command.workflow_start_nonce)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        claim,
        ("committed".into(), Some(command.response.response_id()))
    );

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
async fn create_requires_the_exact_live_start_claim(pool: sqlx::PgPool) {
    let repository = ResponseOwnerRepository::new(pool);
    let command = create_command();
    assert!(matches!(
        repository.create(&command).await,
        Err(ResponseOwnerError::StartLeaseExpired)
    ));

    let claim = start_claim(&command);
    repository.claim_start(&claim).await.unwrap();
    repository.claim_start(&claim).await.unwrap();
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
async fn inaccessible_response_is_nondisclosing(pool: sqlx::PgPool) {
    let repository = ResponseOwnerRepository::new(pool.clone());
    let command = create_command();
    claim_start(&repository, &command).await;
    repository.create(&command).await.unwrap();
    let stranger = ResponseAccess {
        installation_id: Uuid::from_u128(1),
        actor_account_id: Uuid::from_u128(99),
        delegated_account_ids: BTreeSet::new(),
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
