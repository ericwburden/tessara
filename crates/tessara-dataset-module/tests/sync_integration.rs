use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use tessara_dataset_module::sync::{
    BeginAttempt, SyncStoreError, begin_attempt, begin_attempt_in_transaction, promote,
    promote_in_transaction, stage_page, stage_page_in_transaction,
};
use tessara_responses_contract::{
    ResponseExportCursor, ResponseExportEntry, ResponseExportPageResponse, ResponseTombstoneReason,
    SubmittedResponseChange, SubmittedResponseRestrictionTier, SubmittedResponseUpsert,
    SubmittedResponseValue,
};
use uuid::Uuid;

async fn partition(pool: &PgPool) -> (Uuid, Uuid) {
    let dataset_id = Uuid::new_v4();
    let binding_id = Uuid::new_v4();
    sqlx::query("INSERT INTO datasets(id,name,slug,grain) VALUES($1,'Base','base','submission')")
        .bind(dataset_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO dataset_source_bindings(id,dataset_id,binding_key,source_kind,source_identity,source_identity_digest) VALUES($1,$2,'responses','response_export',$3,'sha256:source')")
        .bind(binding_id)
        .bind(dataset_id)
        .bind(json!({"form_version_id":Uuid::new_v4()}))
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO dataset_sync_partitions(source_binding_id) VALUES($1)")
        .bind(binding_id)
        .execute(pool)
        .await
        .unwrap();
    (dataset_id, binding_id)
}

#[derive(Clone, Copy)]
struct ResponseIds {
    response_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
}

fn response_ids() -> ResponseIds {
    ResponseIds {
        response_id: Uuid::new_v4(),
        form_id: Uuid::new_v4(),
        form_version_id: Uuid::new_v4(),
        node_id: Uuid::new_v4(),
    }
}

fn upsert(
    ids: ResponseIds,
    node_name: &str,
    last_modified_at: &str,
    last_modified_by_user_name: Option<&str>,
    restriction_tier: SubmittedResponseRestrictionTier,
    values: BTreeMap<String, SubmittedResponseValue>,
) -> SubmittedResponseChange {
    SubmittedResponseChange::Upsert(Box::new(SubmittedResponseUpsert {
        response_id: ids.response_id,
        form_id: ids.form_id,
        form_version_id: ids.form_version_id,
        node_id: ids.node_id,
        node_name: node_name.into(),
        submitted_at: "2026-08-12T00:01:00Z".into(),
        created_at: "2026-08-12T00:00:00Z".into(),
        last_modified_at: last_modified_at.into(),
        last_modified_by_user_name: last_modified_by_user_name.map(str::to_string),
        status: "submitted".into(),
        restriction_tier,
        scope_node_ids: vec![ids.node_id],
        values,
        content_digest: String::new(),
    }))
    .with_recomputed_content_digest()
    .unwrap()
}

fn value(field_id: Uuid, value: Value, value_text: &str) -> SubmittedResponseValue {
    SubmittedResponseValue {
        field_id,
        value,
        value_text: Some(value_text.into()),
    }
}

fn null_value(field_id: Uuid) -> SubmittedResponseValue {
    SubmittedResponseValue {
        field_id,
        value: Value::Null,
        value_text: None,
    }
}

fn tombstone(response_id: Uuid, reason: ResponseTombstoneReason) -> SubmittedResponseChange {
    SubmittedResponseChange::Tombstone {
        response_id,
        reason,
        content_digest: String::new(),
    }
    .with_recomputed_content_digest()
    .unwrap()
}

fn page(
    epoch: Uuid,
    upper: &str,
    cursor: &str,
    change: SubmittedResponseChange,
) -> ResponseExportPageResponse {
    let entries = vec![ResponseExportEntry {
        cursor: ResponseExportCursor::parse(cursor).unwrap(),
        change,
    }];
    ResponseExportPageResponse {
        schema_version: 1,
        provider_epoch: epoch,
        snapshot_upper_bound: ResponseExportCursor::parse(upper).unwrap(),
        page_digest: ResponseExportPageResponse::canonical_page_digest(&entries).unwrap(),
        entries,
        next_after_cursor: None,
        complete: true,
    }
}

async fn promote_change(
    pool: &PgPool,
    binding_id: Uuid,
    expected_committed_cursor: Option<&str>,
    upper: &str,
    entry_cursor: &str,
    full_snapshot_rebase: bool,
    change: SubmittedResponseChange,
) {
    let attempt_id = Uuid::new_v4();
    let epoch = Uuid::new_v4();
    begin_attempt(
        pool,
        BeginAttempt {
            attempt_id,
            source_binding_id: binding_id,
            provider_epoch: epoch,
            expected_committed_cursor: expected_committed_cursor.map(str::to_string),
            snapshot_upper_bound: upper.into(),
            full_snapshot_rebase,
        },
    )
    .await
    .unwrap();
    stage_page(
        pool,
        attempt_id,
        if full_snapshot_rebase {
            None
        } else {
            expected_committed_cursor
        },
        &page(epoch, upper, entry_cursor, change),
    )
    .await
    .unwrap();
    promote(
        pool,
        attempt_id,
        Uuid::new_v4(),
        "sha256:input",
        "sha256:result",
    )
    .await
    .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn staged_pages_are_invisible_until_atomic_cursor_and_projection_promotion(pool: PgPool) {
    let (_, binding_id) = partition(&pool).await;
    let attempt_id = Uuid::new_v4();
    let epoch = Uuid::new_v4();
    let ids = response_ids();
    let text_field_id = Uuid::new_v4();
    let array_field_id = Uuid::new_v4();
    let null_field_id = Uuid::new_v4();
    let change = upsert(
        ids,
        "North Division",
        "2026-08-12T00:03:00Z",
        Some("Riley Reviewer"),
        SubmittedResponseRestrictionTier::Restricted,
        BTreeMap::from([
            (
                "answer".into(),
                value(text_field_id, json!("current"), "current"),
            ),
            (
                "labels".into(),
                value(array_field_id, json!(["a", "b"]), "[\"a\",\"b\"]"),
            ),
            ("optional".into(), null_value(null_field_id)),
        ]),
    );
    let expected_digest = match &change {
        SubmittedResponseChange::Upsert(upsert) => upsert.content_digest.clone(),
        SubmittedResponseChange::Tombstone { .. } => unreachable!(),
    };
    begin_attempt(
        &pool,
        BeginAttempt {
            attempt_id,
            source_binding_id: binding_id,
            provider_epoch: epoch,
            expected_committed_cursor: None,
            snapshot_upper_bound: "cursor:0002".into(),
            full_snapshot_rebase: false,
        },
    )
    .await
    .unwrap();
    stage_page(
        &pool,
        attempt_id,
        None,
        &page(epoch, "cursor:0002", "cursor:0001", change),
    )
    .await
    .unwrap();

    let before: (i64, Option<String>, i64, i64) = sqlx::query_as(
        "SELECT p.generation,p.committed_cursor,
                (SELECT COUNT(*) FROM dataset_imported_responses WHERE source_binding_id=$1),
                (SELECT COUNT(*) FROM dataset_imported_response_values WHERE source_binding_id=$1)
         FROM dataset_sync_partitions p WHERE p.source_binding_id=$1",
    )
    .bind(binding_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(before, (0, None, 0, 0));

    let receipt = promote(
        &pool,
        attempt_id,
        Uuid::new_v4(),
        "sha256:input",
        "sha256:result",
    )
    .await
    .unwrap();
    assert_eq!(receipt.generation, 1);
    assert_eq!(receipt.committed_cursor, "cursor:0002");
    let after: (i64, Option<String>, i64, i64) = sqlx::query_as(
        "SELECT p.generation,p.committed_cursor,
                (SELECT COUNT(*) FROM dataset_imported_responses WHERE source_binding_id=$1),
                (SELECT COUNT(*) FROM dataset_imported_response_values WHERE source_binding_id=$1)
         FROM dataset_sync_partitions p WHERE p.source_binding_id=$1",
    )
    .bind(binding_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(after, (1, Some("cursor:0002".into()), 1, 3));

    let imported = sqlx::query(
        "SELECT form_id,form_version_id,node_id,node_name,status,submitted_at,created_at,
                last_modified_at,last_modified_by_user_name,restriction_tier,scope_node_ids,
                content_digest,tombstoned,tombstone_reason,promoted_generation
         FROM dataset_imported_responses
         WHERE source_binding_id=$1 AND response_id=$2",
    )
    .bind(binding_id)
    .bind(ids.response_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(imported.try_get::<Uuid, _>("form_id").unwrap(), ids.form_id);
    assert_eq!(
        imported.try_get::<Uuid, _>("form_version_id").unwrap(),
        ids.form_version_id
    );
    assert_eq!(imported.try_get::<Uuid, _>("node_id").unwrap(), ids.node_id);
    assert_eq!(
        imported.try_get::<String, _>("node_name").unwrap(),
        "North Division"
    );
    assert_eq!(
        imported.try_get::<String, _>("status").unwrap(),
        "submitted"
    );
    assert_eq!(
        imported
            .try_get::<DateTime<Utc>, _>("submitted_at")
            .unwrap(),
        "2026-08-12T00:01:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert_eq!(
        imported.try_get::<DateTime<Utc>, _>("created_at").unwrap(),
        "2026-08-12T00:00:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert_eq!(
        imported
            .try_get::<DateTime<Utc>, _>("last_modified_at")
            .unwrap(),
        "2026-08-12T00:03:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert_eq!(
        imported
            .try_get::<Option<String>, _>("last_modified_by_user_name")
            .unwrap(),
        Some("Riley Reviewer".into())
    );
    assert_eq!(
        imported.try_get::<String, _>("restriction_tier").unwrap(),
        "restricted"
    );
    assert_eq!(
        imported.try_get::<Vec<Uuid>, _>("scope_node_ids").unwrap(),
        vec![ids.node_id]
    );
    assert_eq!(
        imported.try_get::<String, _>("content_digest").unwrap(),
        expected_digest
    );
    assert!(!imported.try_get::<bool, _>("tombstoned").unwrap());
    assert_eq!(
        imported
            .try_get::<Option<String>, _>("tombstone_reason")
            .unwrap(),
        None
    );
    assert_eq!(
        imported.try_get::<i64, _>("promoted_generation").unwrap(),
        1
    );

    let values: Vec<(Uuid, String, Value, Option<String>, i64)> = sqlx::query_as(
        "SELECT field_id,field_key,value_json,value_text,promoted_generation
         FROM dataset_imported_response_values
         WHERE source_binding_id=$1 AND response_id=$2
         ORDER BY field_key",
    )
    .bind(binding_id)
    .bind(ids.response_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        values,
        vec![
            (
                text_field_id,
                "answer".into(),
                json!("current"),
                Some("current".into()),
                1
            ),
            (
                array_field_id,
                "labels".into(),
                json!(["a", "b"]),
                Some("[\"a\",\"b\"]".into()),
                1
            ),
            (null_field_id, "optional".into(), Value::Null, None, 1),
        ]
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn corrections_replace_values_and_tombstones_remove_live_facts(pool: PgPool) {
    let (_, binding_id) = partition(&pool).await;
    let ids = response_ids();
    let retired_field_id = Uuid::new_v4();
    let retained_field_id = Uuid::new_v4();
    promote_change(
        &pool,
        binding_id,
        None,
        "cursor:0002",
        "cursor:0001",
        false,
        upsert(
            ids,
            "Original Node",
            "2026-08-12T00:02:00Z",
            Some("Original Author"),
            SubmittedResponseRestrictionTier::Internal,
            BTreeMap::from([
                ("retired".into(), value(retired_field_id, json!(1), "1")),
                (
                    "retained".into(),
                    value(retained_field_id, json!("old"), "old"),
                ),
            ]),
        ),
    )
    .await;

    promote_change(
        &pool,
        binding_id,
        Some("cursor:0002"),
        "cursor:0004",
        "cursor:0003",
        false,
        upsert(
            ids,
            "Renamed Node",
            "2026-08-12T00:04:00Z",
            Some("Correction Author"),
            SubmittedResponseRestrictionTier::Confidential,
            BTreeMap::from([(
                "retained".into(),
                value(retained_field_id, json!("corrected"), "corrected"),
            )]),
        ),
    )
    .await;

    let corrected: (String, DateTime<Utc>, Option<String>, String, i64) = sqlx::query_as(
        "SELECT node_name,last_modified_at,last_modified_by_user_name,restriction_tier,
                promoted_generation
         FROM dataset_imported_responses
         WHERE source_binding_id=$1 AND response_id=$2",
    )
    .bind(binding_id)
    .bind(ids.response_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        corrected,
        (
            "Renamed Node".into(),
            "2026-08-12T00:04:00Z".parse().unwrap(),
            Some("Correction Author".into()),
            "confidential".into(),
            2,
        )
    );
    let corrected_values: Vec<(Uuid, String, Value, String, i64)> = sqlx::query_as(
        "SELECT field_id,field_key,value_json,value_text,promoted_generation
         FROM dataset_imported_response_values
         WHERE source_binding_id=$1 AND response_id=$2",
    )
    .bind(binding_id)
    .bind(ids.response_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        corrected_values,
        vec![(
            retained_field_id,
            "retained".into(),
            json!("corrected"),
            "corrected".into(),
            2,
        )]
    );

    promote_change(
        &pool,
        binding_id,
        Some("cursor:0004"),
        "cursor:0006",
        "cursor:0005",
        false,
        tombstone(ids.response_id, ResponseTombstoneReason::Redacted),
    )
    .await;

    let tombstone = sqlx::query(
        "SELECT form_id,form_version_id,node_id,node_name,status,submitted_at,created_at,
                last_modified_at,last_modified_by_user_name,restriction_tier,scope_node_ids,
                tombstoned,tombstone_reason,promoted_generation
         FROM dataset_imported_responses
         WHERE source_binding_id=$1 AND response_id=$2",
    )
    .bind(binding_id)
    .bind(ids.response_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    for column in ["form_id", "form_version_id", "node_id"] {
        assert_eq!(tombstone.try_get::<Option<Uuid>, _>(column).unwrap(), None);
    }
    for column in [
        "node_name",
        "status",
        "last_modified_by_user_name",
        "restriction_tier",
    ] {
        assert_eq!(
            tombstone.try_get::<Option<String>, _>(column).unwrap(),
            None
        );
    }
    for column in ["submitted_at", "created_at", "last_modified_at"] {
        assert_eq!(
            tombstone
                .try_get::<Option<DateTime<Utc>>, _>(column)
                .unwrap(),
            None
        );
    }
    assert_eq!(
        tombstone.try_get::<Vec<Uuid>, _>("scope_node_ids").unwrap(),
        Vec::<Uuid>::new()
    );
    assert!(tombstone.try_get::<bool, _>("tombstoned").unwrap());
    assert_eq!(
        tombstone.try_get::<String, _>("tombstone_reason").unwrap(),
        "redacted"
    );
    assert_eq!(
        tombstone.try_get::<i64, _>("promoted_generation").unwrap(),
        3
    );
    let remaining_values: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM dataset_imported_response_values
         WHERE source_binding_id=$1 AND response_id=$2",
    )
    .bind(binding_id)
    .bind(ids.response_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(remaining_values, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn full_snapshot_rebase_replaces_the_entire_normalized_partition(pool: PgPool) {
    let (_, binding_id) = partition(&pool).await;
    let old_ids = response_ids();
    let new_ids = response_ids();
    promote_change(
        &pool,
        binding_id,
        None,
        "cursor:0002",
        "cursor:0001",
        false,
        upsert(
            old_ids,
            "Old Node",
            "2026-08-12T00:02:00Z",
            None,
            SubmittedResponseRestrictionTier::Public,
            BTreeMap::from([("old".into(), value(Uuid::new_v4(), json!("old"), "old"))]),
        ),
    )
    .await;

    promote_change(
        &pool,
        binding_id,
        Some("cursor:0002"),
        "cursor:0004",
        "cursor:0003",
        true,
        upsert(
            new_ids,
            "New Node",
            "2026-08-12T00:04:00Z",
            None,
            SubmittedResponseRestrictionTier::Public,
            BTreeMap::from([("new".into(), value(Uuid::new_v4(), json!("new"), "new"))]),
        ),
    )
    .await;

    let imported: Vec<Uuid> = sqlx::query_scalar(
        "SELECT response_id FROM dataset_imported_responses WHERE source_binding_id=$1",
    )
    .bind(binding_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(imported, vec![new_ids.response_id]);
    let values: Vec<Uuid> = sqlx::query_scalar(
        "SELECT response_id FROM dataset_imported_response_values WHERE source_binding_id=$1",
    )
    .bind(binding_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(values, vec![new_ids.response_id]);
}

#[sqlx::test(migrations = "./migrations")]
async fn malformed_persisted_change_fails_closed_before_projection_or_cursor_mutation(
    pool: PgPool,
) {
    let (_, binding_id) = partition(&pool).await;
    let attempt_id = Uuid::new_v4();
    let epoch = Uuid::new_v4();
    begin_attempt(
        &pool,
        BeginAttempt {
            attempt_id,
            source_binding_id: binding_id,
            provider_epoch: epoch,
            expected_committed_cursor: None,
            snapshot_upper_bound: "cursor:0002".into(),
            full_snapshot_rebase: false,
        },
    )
    .await
    .unwrap();
    stage_page(
        &pool,
        attempt_id,
        None,
        &page(
            epoch,
            "cursor:0002",
            "cursor:0001",
            upsert(
                response_ids(),
                "Node",
                "2026-08-12T00:02:00Z",
                None,
                SubmittedResponseRestrictionTier::Public,
                BTreeMap::new(),
            ),
        ),
    )
    .await
    .unwrap();
    sqlx::query(
        "UPDATE dataset_sync_staged_changes
         SET payload=payload - 'node_name'
         WHERE attempt_id=$1",
    )
    .bind(attempt_id)
    .execute(&pool)
    .await
    .unwrap();

    let error = promote(
        &pool,
        attempt_id,
        Uuid::new_v4(),
        "sha256:input",
        "sha256:result",
    )
    .await
    .unwrap_err();
    assert!(matches!(error, SyncStoreError::InvalidStagedChange));
    let committed: (i64, Option<String>, i64, i64) = sqlx::query_as(
        "SELECT p.generation,p.committed_cursor,
                (SELECT count(*) FROM dataset_imported_responses WHERE source_binding_id=$1),
                (SELECT count(*) FROM dataset_materialization_receipts WHERE source_binding_id=$1)
         FROM dataset_sync_partitions p WHERE p.source_binding_id=$1",
    )
    .bind(binding_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(committed, (0, None, 0, 0));
    let state: String = sqlx::query_scalar("SELECT state FROM dataset_sync_attempts WHERE id=$1")
        .bind(attempt_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(state, "complete");
}

#[sqlx::test(migrations = "./migrations")]
async fn transaction_scoped_initial_snapshot_rolls_back_with_dataset_creation(pool: PgPool) {
    let dataset_id = Uuid::new_v4();
    let binding_id = Uuid::new_v4();
    let attempt_id = Uuid::new_v4();
    let epoch = Uuid::new_v4();
    let ids = response_ids();
    let mut transaction = pool.begin().await.unwrap();
    sqlx::query("INSERT INTO datasets(id,name,slug,grain) VALUES($1,'Atomic',$2,'submission')")
        .bind(dataset_id)
        .bind(format!("atomic-{dataset_id}"))
        .execute(&mut *transaction)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO dataset_source_bindings
         (id,dataset_id,binding_key,source_kind,source_identity,source_identity_digest)
         VALUES($1,$2,'responses','response_export',$3,'sha256:source')",
    )
    .bind(binding_id)
    .bind(dataset_id)
    .bind(json!({"form_version_id":ids.form_version_id}))
    .execute(&mut *transaction)
    .await
    .unwrap();
    sqlx::query("INSERT INTO dataset_sync_partitions(source_binding_id) VALUES($1)")
        .bind(binding_id)
        .execute(&mut *transaction)
        .await
        .unwrap();

    begin_attempt_in_transaction(
        &mut transaction,
        BeginAttempt {
            attempt_id,
            source_binding_id: binding_id,
            provider_epoch: epoch,
            expected_committed_cursor: None,
            snapshot_upper_bound: "cursor:0002".into(),
            full_snapshot_rebase: false,
        },
    )
    .await
    .unwrap();
    stage_page_in_transaction(
        &mut transaction,
        attempt_id,
        None,
        &page(
            epoch,
            "cursor:0002",
            "cursor:0001",
            upsert(
                ids,
                "Atomic Node",
                "2026-08-12T00:02:00Z",
                None,
                SubmittedResponseRestrictionTier::Public,
                BTreeMap::from([(
                    "answer".into(),
                    value(Uuid::new_v4(), json!("atomic"), "atomic"),
                )]),
            ),
        ),
    )
    .await
    .unwrap();
    let receipt = promote_in_transaction(
        &mut transaction,
        attempt_id,
        Uuid::new_v4(),
        "sha256:input",
        "sha256:result",
    )
    .await
    .unwrap();
    assert_eq!(receipt.generation, 1);
    let visible_inside: (i64, i64, i64) = sqlx::query_as(
        "SELECT
           (SELECT count(*) FROM datasets WHERE id=$1),
           (SELECT count(*) FROM dataset_imported_responses WHERE source_binding_id=$2),
           (SELECT count(*) FROM dataset_materialization_receipts WHERE attempt_id=$3)",
    )
    .bind(dataset_id)
    .bind(binding_id)
    .bind(attempt_id)
    .fetch_one(&mut *transaction)
    .await
    .unwrap();
    assert_eq!(visible_inside, (1, 1, 1));
    transaction.rollback().await.unwrap();

    let visible_after_rollback: (i64, i64, i64) = sqlx::query_as(
        "SELECT
           (SELECT count(*) FROM datasets WHERE id=$1),
           (SELECT count(*) FROM dataset_imported_responses WHERE source_binding_id=$2),
           (SELECT count(*) FROM dataset_materialization_receipts WHERE attempt_id=$3)",
    )
    .bind(dataset_id)
    .bind(binding_id)
    .bind(attempt_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(visible_after_rollback, (0, 0, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn wrong_page_partition_identity_leaves_committed_state_unchanged(pool: PgPool) {
    let (_, binding_id) = partition(&pool).await;
    let attempt_id = Uuid::new_v4();
    let epoch = Uuid::new_v4();
    begin_attempt(
        &pool,
        BeginAttempt {
            attempt_id,
            source_binding_id: binding_id,
            provider_epoch: epoch,
            expected_committed_cursor: None,
            snapshot_upper_bound: "cursor:0002".into(),
            full_snapshot_rebase: false,
        },
    )
    .await
    .unwrap();
    let error = stage_page(
        &pool,
        attempt_id,
        None,
        &page(
            Uuid::new_v4(),
            "cursor:0002",
            "cursor:0001",
            upsert(
                response_ids(),
                "Node",
                "2026-08-12T00:02:00Z",
                None,
                SubmittedResponseRestrictionTier::Public,
                BTreeMap::new(),
            ),
        ),
    )
    .await
    .unwrap_err();
    assert!(matches!(error, SyncStoreError::PageCheckpointMismatch));
    let committed: (i64, Option<String>, i64) = sqlx::query_as(
        "SELECT p.generation,p.committed_cursor,(SELECT COUNT(*) FROM dataset_imported_responses WHERE source_binding_id=$1) FROM dataset_sync_partitions p WHERE p.source_binding_id=$1",
    )
    .bind(binding_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(committed, (0, None, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn tampered_page_digest_is_rejected_before_staging(pool: PgPool) {
    let (_, binding_id) = partition(&pool).await;
    let attempt_id = Uuid::new_v4();
    let epoch = Uuid::new_v4();
    begin_attempt(
        &pool,
        BeginAttempt {
            attempt_id,
            source_binding_id: binding_id,
            provider_epoch: epoch,
            expected_committed_cursor: None,
            snapshot_upper_bound: "cursor:0002".into(),
            full_snapshot_rebase: false,
        },
    )
    .await
    .unwrap();
    let mut tampered = page(
        epoch,
        "cursor:0002",
        "cursor:0001",
        upsert(
            response_ids(),
            "Node",
            "2026-08-12T00:02:00Z",
            None,
            SubmittedResponseRestrictionTier::Public,
            BTreeMap::new(),
        ),
    );
    tampered.page_digest = format!("sha256:{}", "0".repeat(64));
    let error = stage_page(&pool, attempt_id, None, &tampered)
        .await
        .unwrap_err();
    assert!(matches!(error, SyncStoreError::InvalidProviderPage));
    let staged: i64 =
        sqlx::query_scalar("SELECT count(*) FROM dataset_sync_staged_changes WHERE attempt_id=$1")
            .bind(attempt_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(staged, 0);
}
