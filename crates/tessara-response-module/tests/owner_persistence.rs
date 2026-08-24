use serde_json::{Value, json};
use tessara_response_module::{
    CreateResponseCommand, IdempotentCommit, ResponseOwnerError, ResponseOwnerRepository,
    ResponseValueInput, canonical_digest,
};
use tessara_responses_contract::{ResponseReference, ResponseWorkflowEvent};
use uuid::Uuid;

fn create_command() -> CreateResponseCommand {
    let form_snapshot = json!({
        "schema_version": 1,
        "form": {"id": Uuid::from_u128(4), "name": "Customer intake"},
        "fields": [{"id": Uuid::from_u128(20), "key": "name", "kind": "text"}]
    });
    let workflow_context = json!({
        "schema_version": 1,
        "assignment_id": Uuid::from_u128(7),
        "instance_id": Uuid::from_u128(10),
        "step_instance_id": Uuid::from_u128(11)
    });
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

#[sqlx::test(migrations = "./migrations")]
async fn create_is_atomic_audited_evented_and_idempotent(pool: sqlx::PgPool) {
    let repository = ResponseOwnerRepository::new(pool.clone());
    let command = create_command();
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
    let event_payload: Value =
        sqlx::query_scalar("SELECT payload FROM response_workflow_events")
            .fetch_one(&pool)
            .await
            .unwrap();
    let event: ResponseWorkflowEvent = serde_json::from_value(event_payload).unwrap();
    event.validate().unwrap();

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
