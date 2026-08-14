//! Dataset-owned Response import staging and atomic promotion.
//!
//! A page checkpoint is recoverable work, not published state. Only `promote`
//! changes the imported projection, materialization generation, receipt, and
//! committed provider cursor in one database transaction.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Row, Transaction};
use tessara_responses_contract::{
    ResponseExportEntry, ResponseExportPageResponse, SubmittedResponseChange,
    SubmittedResponseUpsert,
};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeginAttempt {
    pub attempt_id: Uuid,
    pub source_binding_id: Uuid,
    pub provider_epoch: Uuid,
    pub expected_committed_cursor: Option<String>,
    pub snapshot_upper_bound: String,
    pub full_snapshot_rebase: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptCheckpoint {
    pub attempt_id: Uuid,
    pub start_generation: i64,
    pub last_staged_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionReceipt {
    pub receipt_id: Uuid,
    pub source_binding_id: Uuid,
    pub attempt_id: Uuid,
    pub generation: i64,
    pub committed_cursor: String,
    pub snapshot_upper_bound: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncStoreError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("the source binding has no synchronization partition")]
    MissingPartition,
    #[error("the committed cursor changed before the attempt started")]
    CursorConflict,
    #[error("the synchronization attempt is missing or no longer active")]
    InactiveAttempt,
    #[error("the page does not continue from the attempt checkpoint")]
    PageCheckpointMismatch,
    #[error("the provider page failed canonical validation")]
    InvalidProviderPage,
    #[error("a staged Response change does not satisfy the canonical import contract")]
    InvalidStagedChange,
    #[error("a stale attempt cannot promote over a newer generation")]
    StalePromotion,
}

#[derive(Debug)]
enum NormalizedChange {
    Upsert(NormalizedUpsert),
    Tombstone(NormalizedTombstone),
}

#[derive(Debug)]
struct NormalizedUpsert {
    response_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
    node_name: String,
    submitted_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    last_modified_at: DateTime<Utc>,
    last_modified_by_user_name: Option<String>,
    restriction_tier: &'static str,
    scope_node_ids: Vec<Uuid>,
    values: Vec<NormalizedValue>,
    content_digest: String,
}

#[derive(Debug)]
struct NormalizedValue {
    field_id: Uuid,
    field_key: String,
    value_json: Value,
    value_text: Option<String>,
}

#[derive(Debug)]
struct NormalizedTombstone {
    response_id: Uuid,
    reason: &'static str,
    content_digest: String,
}

pub async fn begin_attempt(
    pool: &PgPool,
    input: BeginAttempt,
) -> Result<AttemptCheckpoint, SyncStoreError> {
    let mut transaction = pool.begin().await?;
    let checkpoint = begin_attempt_in_transaction(&mut transaction, input).await?;
    transaction.commit().await?;
    Ok(checkpoint)
}

pub async fn begin_attempt_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    input: BeginAttempt,
) -> Result<AttemptCheckpoint, SyncStoreError> {
    let partition = sqlx::query(
        "SELECT generation, committed_cursor
         FROM dataset_sync_partitions
         WHERE source_binding_id = $1
         FOR UPDATE",
    )
    .bind(input.source_binding_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(SyncStoreError::MissingPartition)?;
    let generation: i64 = partition.try_get("generation")?;
    let committed_cursor: Option<String> = partition.try_get("committed_cursor")?;
    if committed_cursor != input.expected_committed_cursor {
        return Err(SyncStoreError::CursorConflict);
    }
    sqlx::query(
        "INSERT INTO dataset_sync_attempts
         (id, source_binding_id, provider_epoch, start_generation, start_cursor,
          snapshot_upper_bound, last_staged_cursor, full_snapshot_rebase, state)
         VALUES ($1,$2,$3,$4,$5,$6,$5,$7,'active')",
    )
    .bind(input.attempt_id)
    .bind(input.source_binding_id)
    .bind(input.provider_epoch)
    .bind(generation)
    .bind(&input.expected_committed_cursor)
    .bind(&input.snapshot_upper_bound)
    .bind(input.full_snapshot_rebase)
    .execute(&mut **transaction)
    .await?;
    sqlx::query(
        "UPDATE dataset_sync_partitions
         SET freshness_state='refreshing', sanitized_failure_code=NULL, updated_at=now()
         WHERE source_binding_id=$1",
    )
    .bind(input.source_binding_id)
    .execute(&mut **transaction)
    .await?;
    Ok(AttemptCheckpoint {
        attempt_id: input.attempt_id,
        start_generation: generation,
        last_staged_cursor: input.expected_committed_cursor,
    })
}

pub async fn stage_page(
    pool: &PgPool,
    attempt_id: Uuid,
    expected_after_cursor: Option<&str>,
    page: &ResponseExportPageResponse,
) -> Result<AttemptCheckpoint, SyncStoreError> {
    let mut transaction = pool.begin().await?;
    let checkpoint =
        stage_page_in_transaction(&mut transaction, attempt_id, expected_after_cursor, page)
            .await?;
    transaction.commit().await?;
    Ok(checkpoint)
}

pub async fn stage_page_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    attempt_id: Uuid,
    expected_after_cursor: Option<&str>,
    page: &ResponseExportPageResponse,
) -> Result<AttemptCheckpoint, SyncStoreError> {
    page.validate()
        .map_err(|_| SyncStoreError::InvalidProviderPage)?;
    let attempt = sqlx::query(
        "SELECT start_generation, last_staged_cursor, provider_epoch,
                snapshot_upper_bound, full_snapshot_rebase, state
         FROM dataset_sync_attempts WHERE id=$1 FOR UPDATE",
    )
    .bind(attempt_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(SyncStoreError::InactiveAttempt)?;
    let state: String = attempt.try_get("state")?;
    let last_staged_cursor: Option<String> = attempt.try_get("last_staged_cursor")?;
    let provider_epoch: Uuid = attempt.try_get("provider_epoch")?;
    let snapshot_upper_bound: String = attempt.try_get("snapshot_upper_bound")?;
    if state != "active" {
        return Err(SyncStoreError::InactiveAttempt);
    }
    let starting_ordinal: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(ordinal) + 1, 0)
         FROM dataset_sync_staged_changes WHERE attempt_id=$1",
    )
    .bind(attempt_id)
    .fetch_one(&mut **transaction)
    .await?;
    let expected_checkpoint =
        if attempt.try_get::<bool, _>("full_snapshot_rebase")? && starting_ordinal == 0 {
            None
        } else {
            last_staged_cursor.as_deref()
        };
    if expected_checkpoint != expected_after_cursor
        || provider_epoch != page.provider_epoch
        || snapshot_upper_bound != page.snapshot_upper_bound.as_str()
    {
        return Err(SyncStoreError::PageCheckpointMismatch);
    }
    for (offset, entry) in page.entries.iter().enumerate() {
        let (kind, response_id, payload, digest) = staged_entry(entry)?;
        sqlx::query(
            "INSERT INTO dataset_sync_staged_changes
             (attempt_id,ordinal,cursor,change_kind,response_id,payload,content_digest,page_digest)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(attempt_id)
        .bind(starting_ordinal + offset as i64)
        .bind(entry.cursor.as_str())
        .bind(kind)
        .bind(response_id)
        .bind(payload)
        .bind(digest)
        .bind(&page.page_digest)
        .execute(&mut **transaction)
        .await?;
    }
    let page_checkpoint = if page.complete {
        Some(page.snapshot_upper_bound.as_str().to_string())
    } else {
        page.next_after_cursor
            .as_ref()
            .map(|cursor| cursor.as_str().to_string())
    };
    sqlx::query(
        "UPDATE dataset_sync_attempts
         SET last_staged_cursor=$2,
             state=CASE WHEN $3 THEN 'complete' ELSE 'active' END,
             completed_at=CASE WHEN $3 THEN now() ELSE NULL END
         WHERE id=$1",
    )
    .bind(attempt_id)
    .bind(&page_checkpoint)
    .bind(page.complete)
    .execute(&mut **transaction)
    .await?;
    Ok(AttemptCheckpoint {
        attempt_id,
        start_generation: attempt.try_get("start_generation")?,
        last_staged_cursor: page_checkpoint,
    })
}

fn staged_entry(
    entry: &ResponseExportEntry,
) -> Result<(&'static str, Uuid, Value, String), SyncStoreError> {
    match &entry.change {
        SubmittedResponseChange::Upsert(upsert) => Ok((
            "upsert",
            upsert.response_id,
            serde_json::to_value(&entry.change)
                .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
            upsert.content_digest.clone(),
        )),
        SubmittedResponseChange::Tombstone {
            response_id,
            content_digest,
            ..
        } => Ok((
            "tombstone",
            *response_id,
            serde_json::to_value(&entry.change)
                .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
            content_digest.clone(),
        )),
    }
}

async fn load_normalized_changes(
    transaction: &mut Transaction<'_, Postgres>,
    attempt_id: Uuid,
) -> Result<Vec<NormalizedChange>, SyncStoreError> {
    let rows = sqlx::query(
        "SELECT change_kind,response_id,payload,content_digest
         FROM (
           SELECT DISTINCT ON (response_id)
                  ordinal,change_kind,response_id,payload,content_digest
           FROM dataset_sync_staged_changes
           WHERE attempt_id=$1
           ORDER BY response_id,ordinal DESC
         ) latest
         ORDER BY ordinal",
    )
    .bind(attempt_id)
    .fetch_all(&mut **transaction)
    .await?;

    rows.into_iter()
        .map(|row| {
            normalize_staged_change(
                row.try_get("change_kind")?,
                row.try_get("response_id")?,
                row.try_get("content_digest")?,
                row.try_get("payload")?,
            )
        })
        .collect()
}

fn normalize_staged_change(
    expected_kind: String,
    expected_response_id: Uuid,
    expected_content_digest: String,
    payload: Value,
) -> Result<NormalizedChange, SyncStoreError> {
    let change = serde_json::from_value::<SubmittedResponseChange>(payload)
        .map_err(|_| SyncStoreError::InvalidStagedChange)?;
    change
        .validate_content_digest()
        .map_err(|_| SyncStoreError::InvalidStagedChange)?;

    match change {
        SubmittedResponseChange::Upsert(upsert) => {
            let SubmittedResponseUpsert {
                response_id,
                form_id,
                form_version_id,
                node_id,
                node_name,
                submitted_at,
                created_at,
                last_modified_at,
                last_modified_by_user_name,
                status,
                restriction_tier,
                scope_node_ids,
                values,
                content_digest,
            } = *upsert;
            if expected_kind != "upsert"
                || response_id != expected_response_id
                || content_digest != expected_content_digest
                || response_id.is_nil()
                || form_id.is_nil()
                || form_version_id.is_nil()
                || node_id.is_nil()
                || node_name.trim().is_empty()
                || status != "submitted"
                || scope_node_ids.is_empty()
                || !scope_node_ids.contains(&node_id)
                || scope_node_ids.iter().any(Uuid::is_nil)
            {
                return Err(SyncStoreError::InvalidStagedChange);
            }
            let submitted_at = parse_contract_timestamp(&submitted_at)?;
            let created_at = parse_contract_timestamp(&created_at)?;
            let last_modified_at = parse_contract_timestamp(&last_modified_at)?;
            let mut field_ids = BTreeSet::new();
            let values = values
                .into_iter()
                .map(|(field_key, value)| {
                    if field_key.trim().is_empty()
                        || value.field_id.is_nil()
                        || !field_ids.insert(value.field_id)
                    {
                        return Err(SyncStoreError::InvalidStagedChange);
                    }
                    Ok(NormalizedValue {
                        field_id: value.field_id,
                        field_key,
                        value_json: value.value,
                        value_text: value.value_text,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(NormalizedChange::Upsert(NormalizedUpsert {
                response_id,
                form_id,
                form_version_id,
                node_id,
                node_name,
                submitted_at,
                created_at,
                last_modified_at,
                last_modified_by_user_name,
                restriction_tier: restriction_tier.as_str(),
                scope_node_ids,
                values,
                content_digest,
            }))
        }
        SubmittedResponseChange::Tombstone {
            response_id,
            reason,
            content_digest,
        } => {
            if expected_kind != "tombstone"
                || response_id != expected_response_id
                || content_digest != expected_content_digest
                || response_id.is_nil()
            {
                return Err(SyncStoreError::InvalidStagedChange);
            }
            Ok(NormalizedChange::Tombstone(NormalizedTombstone {
                response_id,
                reason: reason.as_str(),
                content_digest,
            }))
        }
    }
}

fn parse_contract_timestamp(value: &str) -> Result<DateTime<Utc>, SyncStoreError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| SyncStoreError::InvalidStagedChange)
}

async fn apply_normalized_change(
    transaction: &mut Transaction<'_, Postgres>,
    source_binding_id: Uuid,
    generation: i64,
    change: NormalizedChange,
) -> Result<(), SyncStoreError> {
    match change {
        NormalizedChange::Upsert(upsert) => {
            sqlx::query(
                "DELETE FROM dataset_imported_responses
                 WHERE source_binding_id=$1 AND response_id=$2",
            )
            .bind(source_binding_id)
            .bind(upsert.response_id)
            .execute(&mut **transaction)
            .await?;
            sqlx::query(
                "INSERT INTO dataset_imported_responses
                 (source_binding_id,response_id,form_id,form_version_id,node_id,node_name,status,
                  submitted_at,created_at,last_modified_at,last_modified_by_user_name,
                  restriction_tier,scope_node_ids,content_digest,tombstoned,tombstone_reason,
                  promoted_generation)
                 VALUES ($1,$2,$3,$4,$5,$6,'submitted',$7,$8,$9,$10,$11,$12,$13,false,NULL,$14)",
            )
            .bind(source_binding_id)
            .bind(upsert.response_id)
            .bind(upsert.form_id)
            .bind(upsert.form_version_id)
            .bind(upsert.node_id)
            .bind(upsert.node_name)
            .bind(upsert.submitted_at)
            .bind(upsert.created_at)
            .bind(upsert.last_modified_at)
            .bind(upsert.last_modified_by_user_name)
            .bind(upsert.restriction_tier)
            .bind(upsert.scope_node_ids)
            .bind(upsert.content_digest)
            .bind(generation)
            .execute(&mut **transaction)
            .await?;
            for value in upsert.values {
                sqlx::query(
                    "INSERT INTO dataset_imported_response_values
                     (source_binding_id,response_id,form_version_id,field_id,field_key,
                      value_json,value_text,promoted_generation)
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
                )
                .bind(source_binding_id)
                .bind(upsert.response_id)
                .bind(upsert.form_version_id)
                .bind(value.field_id)
                .bind(value.field_key)
                .bind(value.value_json)
                .bind(value.value_text)
                .bind(generation)
                .execute(&mut **transaction)
                .await?;
            }
        }
        NormalizedChange::Tombstone(tombstone) => {
            sqlx::query(
                "DELETE FROM dataset_imported_responses
                 WHERE source_binding_id=$1 AND response_id=$2",
            )
            .bind(source_binding_id)
            .bind(tombstone.response_id)
            .execute(&mut **transaction)
            .await?;
            sqlx::query(
                "INSERT INTO dataset_imported_responses
                 (source_binding_id,response_id,content_digest,tombstoned,tombstone_reason,
                  promoted_generation)
                 VALUES ($1,$2,$3,true,$4,$5)",
            )
            .bind(source_binding_id)
            .bind(tombstone.response_id)
            .bind(tombstone.content_digest)
            .bind(tombstone.reason)
            .bind(generation)
            .execute(&mut **transaction)
            .await?;
        }
    }
    Ok(())
}

pub async fn promote(
    pool: &PgPool,
    attempt_id: Uuid,
    receipt_id: Uuid,
    input_digest: &str,
    result_digest: &str,
) -> Result<PromotionReceipt, SyncStoreError> {
    let mut transaction = pool.begin().await?;
    let result = promote_in_transaction(
        &mut transaction,
        attempt_id,
        receipt_id,
        input_digest,
        result_digest,
    )
    .await;
    match result {
        Ok(receipt) => {
            transaction.commit().await?;
            Ok(receipt)
        }
        Err(SyncStoreError::StalePromotion) => {
            let state: Option<String> =
                sqlx::query_scalar("SELECT state FROM dataset_sync_attempts WHERE id=$1")
                    .bind(attempt_id)
                    .fetch_optional(&mut *transaction)
                    .await?;
            if state.as_deref() == Some("superseded") {
                transaction.commit().await?;
            }
            Err(SyncStoreError::StalePromotion)
        }
        Err(error) => Err(error),
    }
}

/// Applies one complete staged snapshot to the normalized import projection,
/// advances the partition, and writes its receipt without committing. Callers
/// that own a larger Dataset mutation can therefore publish all related rows
/// with one outer commit.
pub async fn promote_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    attempt_id: Uuid,
    receipt_id: Uuid,
    input_digest: &str,
    result_digest: &str,
) -> Result<PromotionReceipt, SyncStoreError> {
    let attempt = sqlx::query(
        "SELECT source_binding_id, start_generation, snapshot_upper_bound,
                last_staged_cursor, full_snapshot_rebase, state
         FROM dataset_sync_attempts WHERE id=$1 FOR UPDATE",
    )
    .bind(attempt_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(SyncStoreError::InactiveAttempt)?;
    let state: String = attempt.try_get("state")?;
    if state != "complete" {
        return Err(SyncStoreError::InactiveAttempt);
    }
    let source_binding_id: Uuid = attempt.try_get("source_binding_id")?;
    let start_generation: i64 = attempt.try_get("start_generation")?;
    let partition_generation: i64 = sqlx::query_scalar(
        "SELECT generation FROM dataset_sync_partitions
         WHERE source_binding_id=$1 FOR UPDATE",
    )
    .bind(source_binding_id)
    .fetch_one(&mut **transaction)
    .await?;
    if partition_generation != start_generation {
        sqlx::query("UPDATE dataset_sync_attempts SET state='superseded' WHERE id=$1")
            .bind(attempt_id)
            .execute(&mut **transaction)
            .await?;
        return Err(SyncStoreError::StalePromotion);
    }
    let generation = start_generation + 1;
    let normalized_changes = load_normalized_changes(&mut *transaction, attempt_id).await?;
    let full_snapshot_rebase: bool = attempt.try_get("full_snapshot_rebase")?;
    if full_snapshot_rebase {
        sqlx::query("DELETE FROM dataset_imported_responses WHERE source_binding_id=$1")
            .bind(source_binding_id)
            .execute(&mut **transaction)
            .await?;
    }
    for change in normalized_changes {
        apply_normalized_change(&mut *transaction, source_binding_id, generation, change).await?;
    }
    let committed_cursor: String = attempt
        .try_get::<Option<String>, _>("last_staged_cursor")?
        .unwrap_or_else(|| {
            attempt
                .try_get::<String, _>("snapshot_upper_bound")
                .expect("selected snapshot bound")
        });
    let snapshot_upper_bound: String = attempt.try_get("snapshot_upper_bound")?;
    let updated = sqlx::query(
        "UPDATE dataset_sync_partitions
         SET provider_epoch=a.provider_epoch, committed_cursor=$2,
             committed_snapshot_upper_bound=$3, generation=$4,
             freshness_state='current', sanitized_failure_code=NULL,
             last_checked_at=now(), last_succeeded_at=now(), updated_at=now()
         FROM dataset_sync_attempts a
         WHERE dataset_sync_partitions.source_binding_id=$1
           AND a.id=$5 AND dataset_sync_partitions.generation=$6",
    )
    .bind(source_binding_id)
    .bind(&committed_cursor)
    .bind(&snapshot_upper_bound)
    .bind(generation)
    .bind(attempt_id)
    .bind(start_generation)
    .execute(&mut **transaction)
    .await?;
    if updated.rows_affected() != 1 {
        return Err(SyncStoreError::StalePromotion);
    }
    sqlx::query(
        "UPDATE dataset_sync_attempts SET state='promoted', completed_at=now() WHERE id=$1",
    )
    .bind(attempt_id)
    .execute(&mut **transaction)
    .await?;
    sqlx::query(
        "INSERT INTO dataset_materialization_receipts
         (id,source_binding_id,attempt_id,generation,committed_cursor,
          snapshot_upper_bound,input_digest,result_digest)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(receipt_id)
    .bind(source_binding_id)
    .bind(attempt_id)
    .bind(generation)
    .bind(&committed_cursor)
    .bind(&snapshot_upper_bound)
    .bind(input_digest)
    .bind(result_digest)
    .execute(&mut **transaction)
    .await?;
    Ok(PromotionReceipt {
        receipt_id,
        source_binding_id,
        attempt_id,
        generation,
        committed_cursor,
        snapshot_upper_bound,
    })
}
