//! Core-hosted Workflow transition consumer for Response-owned lifecycle events.

use chrono::{DateTime, Utc};
use sqlx::{Postgres, Row, Transaction};
use tessara_module_contract::ResourceOwner;
use tessara_responses_contract::{
    MAX_RESPONSE_EVENT_PAGE_SIZE, RESPONSE_EVENT_BINDING_KEY, RESPONSE_EVENT_CHECKPOINT_ACTION,
    RESPONSE_EVENT_CHECKPOINT_PATH, RESPONSE_EVENT_CONTRACT_ID, RESPONSE_EVENT_CONTRACT_VERSION,
    RESPONSE_EVENT_MEDIA_TYPE, RESPONSE_EVENT_PAGE_ACTION, RESPONSE_EVENT_PAGE_PATH,
    RESPONSE_EVENT_SCHEMA_VERSION, RESPONSE_EVENT_START_ACTION, RESPONSE_EVENT_START_PATH,
    RESPONSE_START_RECONCILIATION_ACTION, RESPONSE_START_RECONCILIATION_BINDING_KEY,
    RESPONSE_START_RECONCILIATION_CONTRACT_ID, RESPONSE_START_RECONCILIATION_CONTRACT_VERSION,
    RESPONSE_START_RECONCILIATION_MEDIA_TYPE, RESPONSE_START_RECONCILIATION_PATH,
    RESPONSE_START_RECONCILIATION_SCHEMA_VERSION, ResponseEventCheckpointRequest,
    ResponseEventCheckpointResponse, ResponseEventKind, ResponseEventPageRequest,
    ResponseEventPageResponse, ResponseEventStartRequest, ResponseEventStartResponse,
    ResponseLifecycleState, ResponseStartReconciliationRequest,
    ResponseStartReconciliationResponse, ResponseStartReconciliationState, ResponseWorkflowEvent,
};
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{ApiError, ApiResult},
    module_gateway::{
        CorePrivateProviderResult, CoreSystemJobProviderRequest,
        call_private_provider_for_system_job,
    },
};

const RESPONSE_MODULE_DEFINITION_ID: &str = "tessara.responses";
const WORKFLOW_EVENT_SYSTEM_JOB_ID: &str = "workflow-response-event-consumer";
const WORKFLOW_EVENT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);
const WORKFLOW_EVENT_ADVISORY_LOCK_KEY: i64 = 0x5752_4553_504f_4e53;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConsumerErrorCode {
    ResponseProviderUnavailable,
    ResponseProviderUndisclosed,
    ConsumerFailure,
}

impl ConsumerErrorCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ResponseProviderUnavailable => "response_provider_unavailable",
            Self::ResponseProviderUndisclosed => "response_provider_undisclosed",
            Self::ConsumerFailure => "consumer_failure",
        }
    }
}

enum ConsumerCall<T> {
    Response(T),
    Retry(ConsumerErrorCode),
}

struct ConsumerState {
    provider_epoch: Option<Uuid>,
    committed_sequence: u64,
    observed_head_sequence: u64,
}

/// Runs the Core-owned Response event job independently of browser traffic.
/// A transaction-scoped advisory lock elects at most one worker per database;
/// durable page commits let another process resume after crashes or outages.
pub(crate) async fn run(state: AppState) {
    let mut interval = tokio::time::interval(WORKFLOW_EVENT_POLL_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        if let Err(error) = run_cycle(&state).await {
            tracing::warn!(%error, "Workflow Response consumer cycle could not record its state");
        }
    }
}

async fn run_cycle(state: &AppState) -> ApiResult<()> {
    let mut lease = state.pool.begin().await?;
    let elected: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(WORKFLOW_EVENT_ADVISORY_LOCK_KEY)
        .fetch_one(&mut *lease)
        .await?;
    if !elected {
        lease.commit().await?;
        return Ok(());
    }

    mark_attempt(&state.pool).await?;
    let result = synchronize(state).await;
    match result {
        Ok(None) => mark_stable(&state.pool).await?,
        Ok(Some(error_code)) => mark_error(&state.pool, error_code).await?,
        Err(error) => {
            mark_error(&state.pool, ConsumerErrorCode::ConsumerFailure).await?;
            tracing::warn!(%error, "Workflow Response consumer will retry from its durable cursor");
        }
    }
    lease.commit().await?;
    Ok(())
}

async fn synchronize(state: &AppState) -> ApiResult<Option<ConsumerErrorCode>> {
    let page_size = match consume_events(state).await? {
        ConsumerCall::Response(page_size) => page_size,
        ConsumerCall::Retry(error_code) => return Ok(Some(error_code)),
    };
    match reconcile_expired_reservations(state, page_size).await? {
        ConsumerCall::Response(()) => Ok(None),
        ConsumerCall::Retry(error_code) => Ok(Some(error_code)),
    }
}

async fn consume_events(state: &AppState) -> ApiResult<ConsumerCall<u16>> {
    let stored = consumer_state(&state.pool).await?;
    let checkpoint_request = ResponseEventCheckpointRequest {
        schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
        committed_sequence: stored.committed_sequence,
    };
    let checkpoint = match call::<_, ResponseEventCheckpointResponse>(
        state,
        RESPONSE_EVENT_BINDING_KEY,
        RESPONSE_EVENT_CONTRACT_ID,
        RESPONSE_EVENT_CONTRACT_VERSION,
        RESPONSE_EVENT_CHECKPOINT_ACTION,
        RESPONSE_EVENT_CHECKPOINT_PATH,
        RESPONSE_EVENT_MEDIA_TYPE,
        &checkpoint_request,
    )
    .await?
    {
        ConsumerCall::Response(value) => value,
        ConsumerCall::Retry(error_code) => return Ok(ConsumerCall::Retry(error_code)),
    };
    validate_checkpoint(&checkpoint, stored.committed_sequence)?;
    if stored.provider_epoch == Some(checkpoint.provider_epoch)
        && checkpoint.authenticated_head < stored.observed_head_sequence
    {
        return Err(invalid_provider("Response event head regression"));
    }
    observe_head(
        &state.pool,
        checkpoint.provider_epoch,
        checkpoint.authenticated_head,
    )
    .await?;

    let committed_sequence = if stored.provider_epoch != Some(checkpoint.provider_epoch)
        || !checkpoint.committed_sequence_valid
    {
        reset_consumer_epoch(
            &state.pool,
            checkpoint.provider_epoch,
            checkpoint.authenticated_head,
        )
        .await?;
        0
    } else {
        stored.committed_sequence
    };
    let page_size = configured_event_page_size(&state.pool).await?;
    if !checkpoint.changed && committed_sequence == checkpoint.authenticated_head {
        return Ok(ConsumerCall::Response(page_size));
    }

    let start_request = ResponseEventStartRequest {
        schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
        provider_epoch: checkpoint.provider_epoch,
        committed_sequence,
        authenticated_head: checkpoint.authenticated_head,
        page_size,
    };
    start_request
        .validate()
        .map_err(|error| ApiError::Internal(error.into()))?;
    let window = match call::<_, ResponseEventStartResponse>(
        state,
        RESPONSE_EVENT_BINDING_KEY,
        RESPONSE_EVENT_CONTRACT_ID,
        RESPONSE_EVENT_CONTRACT_VERSION,
        RESPONSE_EVENT_START_ACTION,
        RESPONSE_EVENT_START_PATH,
        RESPONSE_EVENT_MEDIA_TYPE,
        &start_request,
    )
    .await?
    {
        ConsumerCall::Response(value) => value,
        ConsumerCall::Retry(error_code) => return Ok(ConsumerCall::Retry(error_code)),
    };
    if window.schema_version != RESPONSE_EVENT_SCHEMA_VERSION
        || window.provider_epoch != checkpoint.provider_epoch
        || window.start_after_sequence != committed_sequence
        || window.snapshot_upper_bound != checkpoint.authenticated_head
    {
        return Err(invalid_provider("Response event window"));
    }

    let mut after_sequence = window.start_after_sequence;
    loop {
        let request = ResponseEventPageRequest {
            schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
            provider_epoch: window.provider_epoch,
            snapshot_upper_bound: window.snapshot_upper_bound,
            after_sequence,
            page_size,
        };
        let page = match call::<_, ResponseEventPageResponse>(
            state,
            RESPONSE_EVENT_BINDING_KEY,
            RESPONSE_EVENT_CONTRACT_ID,
            RESPONSE_EVENT_CONTRACT_VERSION,
            RESPONSE_EVENT_PAGE_ACTION,
            RESPONSE_EVENT_PAGE_PATH,
            RESPONSE_EVENT_MEDIA_TYPE,
            &request,
        )
        .await?
        {
            ConsumerCall::Response(value) => value,
            ConsumerCall::Retry(error_code) => return Ok(ConsumerCall::Retry(error_code)),
        };
        page.validate()
            .map_err(|error| ApiError::Internal(error.into()))?;
        if page.provider_epoch != window.provider_epoch
            || page.snapshot_upper_bound != window.snapshot_upper_bound
            || page
                .entries
                .first()
                .is_some_and(|event| event.sequence <= after_sequence)
            || (!page.complete && page.next_after_sequence <= after_sequence)
            || (page.complete && page.next_after_sequence != window.snapshot_upper_bound)
        {
            return Err(invalid_provider("Response event page"));
        }
        apply_page(&state.pool, window.provider_epoch, after_sequence, &page).await?;
        after_sequence = page.next_after_sequence;
        if page.complete {
            break;
        }
    }

    let final_checkpoint_request = ResponseEventCheckpointRequest {
        schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
        committed_sequence: after_sequence,
    };
    let final_checkpoint = match call::<_, ResponseEventCheckpointResponse>(
        state,
        RESPONSE_EVENT_BINDING_KEY,
        RESPONSE_EVENT_CONTRACT_ID,
        RESPONSE_EVENT_CONTRACT_VERSION,
        RESPONSE_EVENT_CHECKPOINT_ACTION,
        RESPONSE_EVENT_CHECKPOINT_PATH,
        RESPONSE_EVENT_MEDIA_TYPE,
        &final_checkpoint_request,
    )
    .await?
    {
        ConsumerCall::Response(value) => value,
        ConsumerCall::Retry(error_code) => return Ok(ConsumerCall::Retry(error_code)),
    };
    validate_checkpoint(&final_checkpoint, after_sequence)?;
    if final_checkpoint.provider_epoch != window.provider_epoch
        || !final_checkpoint.committed_sequence_valid
        || final_checkpoint.authenticated_head < window.snapshot_upper_bound
    {
        return Err(invalid_provider("Response event final checkpoint"));
    }
    observe_head(
        &state.pool,
        final_checkpoint.provider_epoch,
        final_checkpoint.authenticated_head,
    )
    .await?;
    Ok(ConsumerCall::Response(page_size))
}

async fn apply_page(
    pool: &sqlx::PgPool,
    provider_epoch: Uuid,
    expected_after_sequence: u64,
    page: &ResponseEventPageResponse,
) -> ApiResult<()> {
    let mut tx = pool.begin().await?;
    let state = sqlx::query(
        "SELECT provider_epoch,committed_sequence FROM workflow_response_event_consumer_state WHERE singleton=true FOR UPDATE",
    )
    .fetch_one(&mut *tx)
    .await?;
    if state.try_get::<Option<Uuid>, _>("provider_epoch")? != Some(provider_epoch)
        || state.try_get::<i64, _>("committed_sequence")? != sequence_i64(expected_after_sequence)?
    {
        return Err(invalid_provider("Response event consumer cursor"));
    }

    for event in &page.entries {
        if consumed_event(&mut tx, event).await? {
            continue;
        }
        apply_event(&mut tx, event).await?;
        sqlx::query(
            "INSERT INTO workflow_response_consumed_events(event_id,sequence,content_digest) VALUES($1,$2,$3)",
        )
        .bind(event.event_id)
        .bind(sequence_i64(event.sequence)?)
        .bind(&event.content_digest)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "UPDATE workflow_response_event_consumer_state SET committed_sequence=$1 WHERE singleton=true",
    )
    .bind(sequence_i64(page.next_after_sequence)?)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn consumed_event(
    tx: &mut Transaction<'_, Postgres>,
    event: &ResponseWorkflowEvent,
) -> ApiResult<bool> {
    let rows = sqlx::query(
        "SELECT event_id,sequence,content_digest FROM workflow_response_consumed_events WHERE event_id=$1 OR sequence=$2 FOR UPDATE",
    )
    .bind(event.event_id)
    .bind(sequence_i64(event.sequence)?)
    .fetch_all(&mut **tx)
    .await?;
    match rows.as_slice() {
        [] => Ok(false),
        [row]
            if row.try_get::<Uuid, _>("event_id")? == event.event_id
                && row.try_get::<i64, _>("sequence")? == sequence_i64(event.sequence)?
                && row.try_get::<String, _>("content_digest")? == event.content_digest =>
        {
            Ok(true)
        }
        _ => Err(invalid_provider("Response event identity")),
    }
}

async fn apply_event(
    tx: &mut Transaction<'_, Postgres>,
    event: &ResponseWorkflowEvent,
) -> ApiResult<()> {
    event
        .validate()
        .map_err(|error| ApiError::Internal(error.into()))?;
    match event.kind {
        ResponseEventKind::Started => apply_started(tx, event).await,
        ResponseEventKind::DraftSaved => apply_projection_change(tx, event, "draft").await,
        ResponseEventKind::Submitted => {
            apply_projection_change(tx, event, "submitted").await?;
            advance_workflow(
                tx,
                event.workflow_instance_id,
                event.workflow_step_instance_id,
            )
            .await
        }
        ResponseEventKind::Deleted => {
            apply_projection_change(tx, event, "deleted").await?;
            release_deleted_runtime(
                tx,
                event.workflow_instance_id,
                event.workflow_step_instance_id,
            )
            .await
        }
    }
}

async fn apply_started(
    tx: &mut Transaction<'_, Postgres>,
    event: &ResponseWorkflowEvent,
) -> ApiResult<()> {
    let reservation = sqlx::query(
        "SELECT workflow_instance_id,workflow_step_instance_id,consumed_response_id FROM workflow_response_reservations WHERE workflow_assignment_id=$1 AND workflow_instance_id=$2 AND workflow_step_instance_id=$3 FOR UPDATE",
    )
    .bind(event.workflow_assignment_id)
    .bind(event.workflow_instance_id)
    .bind(event.workflow_step_instance_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| invalid_provider("Response started reservation"))?;
    if reservation.try_get::<Uuid, _>("workflow_instance_id")? != event.workflow_instance_id
        || reservation.try_get::<Uuid, _>("workflow_step_instance_id")?
            != event.workflow_step_instance_id
        || reservation
            .try_get::<Option<Uuid>, _>("consumed_response_id")?
            .is_some_and(|id| id != event.response.response_id())
    {
        return Err(invalid_provider("Response started context"));
    }
    sqlx::query(
        "UPDATE workflow_response_reservations SET consumed_response_id=$2,consumed_at=COALESCE(consumed_at,now()) WHERE workflow_assignment_id=$1 AND workflow_instance_id=$3 AND workflow_step_instance_id=$4",
    )
    .bind(event.workflow_assignment_id)
    .bind(event.response.response_id())
    .bind(event.workflow_instance_id)
    .bind(event.workflow_step_instance_id)
    .execute(&mut **tx)
    .await?;
    upsert_projection(tx, event, "draft", false).await
}

async fn apply_projection_change(
    tx: &mut Transaction<'_, Postgres>,
    event: &ResponseWorkflowEvent,
    state: &str,
) -> ApiResult<()> {
    upsert_projection(tx, event, state, true).await
}

async fn upsert_projection(
    tx: &mut Transaction<'_, Postgres>,
    event: &ResponseWorkflowEvent,
    state: &str,
    require_existing: bool,
) -> ApiResult<()> {
    let existing = sqlx::query(
        "SELECT workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,response_revision,response_state,last_event_sequence FROM workflow_response_projection WHERE response_id=$1 FOR UPDATE",
    )
    .bind(event.response.response_id())
    .fetch_optional(&mut **tx)
    .await?;
    let Some(existing) = existing else {
        if require_existing {
            return Err(invalid_provider("Response event projection"));
        }
        sqlx::query(
            "INSERT INTO workflow_response_projection(workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,response_installation_id,response_module_instance_id,response_id,response_revision,response_state,last_event_sequence,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(event.workflow_assignment_id)
        .bind(event.workflow_instance_id)
        .bind(event.workflow_step_instance_id)
        .bind(response_owner(&event.response).0)
        .bind(response_owner(&event.response).1)
        .bind(event.response.response_id())
        .bind(sequence_i64(event.response_revision)?)
        .bind(state)
        .bind(sequence_i64(event.sequence)?)
        .bind(parse_time(&event.occurred_at)?)
        .execute(&mut **tx)
        .await?;
        return Ok(());
    };
    if existing.try_get::<Uuid, _>("workflow_assignment_id")? != event.workflow_assignment_id
        || existing.try_get::<Uuid, _>("workflow_instance_id")? != event.workflow_instance_id
        || existing.try_get::<Uuid, _>("workflow_step_instance_id")?
            != event.workflow_step_instance_id
    {
        return Err(invalid_provider("Response projection context"));
    }
    let revision = u64::try_from(existing.try_get::<i64, _>("response_revision")?)
        .map_err(|_| invalid_provider("Response projection revision"))?;
    let current_state: String = existing.try_get("response_state")?;
    match projection_revision_action(
        revision,
        &current_state,
        existing.try_get("last_event_sequence")?,
        event.response_revision,
        state,
    )? {
        ProjectionRevisionAction::IgnoreStale => return Ok(()),
        ProjectionRevisionAction::Apply => {}
    }
    sqlx::query(
        "UPDATE workflow_response_projection SET response_revision=$2,response_state=$3,last_event_sequence=$4,updated_at=$5 WHERE response_id=$1",
    )
    .bind(event.response.response_id())
    .bind(sequence_i64(event.response_revision)?)
    .bind(state)
    .bind(sequence_i64(event.sequence)?)
    .bind(parse_time(&event.occurred_at)?)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn advance_workflow(
    tx: &mut Transaction<'_, Postgres>,
    workflow_instance_id: Uuid,
    workflow_step_instance_id: Uuid,
) -> ApiResult<()> {
    let row = sqlx::query(
        "SELECT wi.workflow_version_id,wi.node_id,wi.assignee_account_id,wi.status,ws.position,wsi.status AS step_status FROM workflow_step_instances wsi JOIN workflow_instances wi ON wi.id=wsi.workflow_instance_id JOIN workflow_steps ws ON ws.id=wsi.workflow_step_id WHERE wi.id=$1 AND wsi.id=$2 FOR UPDATE OF wi,wsi",
    )
    .bind(workflow_instance_id)
    .bind(workflow_step_instance_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| invalid_provider("Workflow event runtime"))?;
    if row.try_get::<String, _>("step_status")? == "completed" {
        return Ok(());
    }
    if row.try_get::<String, _>("status")? != "in_progress" {
        return Err(invalid_provider("Workflow event instance state"));
    }
    sqlx::query(
        "UPDATE workflow_step_instances SET status='completed',completed_at=now() WHERE id=$1 AND status='in_progress'",
    )
    .bind(workflow_step_instance_id)
    .execute(&mut **tx)
    .await?;
    let workflow_version_id: Uuid = row.try_get("workflow_version_id")?;
    let next_step: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM workflow_steps WHERE workflow_version_id=$1 AND position=$2",
    )
    .bind(workflow_version_id)
    .bind(row.try_get::<i32, _>("position")? + 1)
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(next_step) = next_step {
        crate::workflows::ensure_specific_workflow_assignment_tx(
            tx,
            workflow_version_id,
            next_step,
            row.try_get("node_id")?,
            row.try_get("assignee_account_id")?,
        )
        .await?;
    } else {
        sqlx::query(
            "UPDATE workflow_instances SET status='completed',completed_at=now() WHERE id=$1 AND status='in_progress'",
        )
        .bind(workflow_instance_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn release_deleted_runtime(
    tx: &mut Transaction<'_, Postgres>,
    workflow_instance_id: Uuid,
    workflow_step_instance_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        "DELETE FROM workflow_step_instances WHERE id=$1 AND workflow_instance_id=$2 AND status='in_progress'",
    )
    .bind(workflow_step_instance_id)
    .bind(workflow_instance_id)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "DELETE FROM workflow_instances wi WHERE wi.id=$1 AND wi.status='in_progress' AND NOT EXISTS(SELECT 1 FROM workflow_step_instances wsi WHERE wsi.workflow_instance_id=wi.id)",
    )
    .bind(workflow_instance_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn reconcile_expired_reservations(
    state: &AppState,
    page_size: u16,
) -> ApiResult<ConsumerCall<()>> {
    let rows = sqlx::query(
        "SELECT workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,one_use_nonce FROM workflow_response_reservations WHERE consumed_at IS NULL AND expires_at<=now() ORDER BY created_at LIMIT $1",
    )
    .bind(i64::from(page_size))
    .fetch_all(&state.pool)
    .await?;
    for row in rows {
        let request = ResponseStartReconciliationRequest {
            schema_version: RESPONSE_START_RECONCILIATION_SCHEMA_VERSION,
            workflow_assignment_id: row.try_get("workflow_assignment_id")?,
            workflow_instance_id: row.try_get("workflow_instance_id")?,
            workflow_step_instance_id: row.try_get("workflow_step_instance_id")?,
            one_use_nonce: row.try_get("one_use_nonce")?,
        };
        let result = match call::<_, ResponseStartReconciliationResponse>(
            state,
            RESPONSE_START_RECONCILIATION_BINDING_KEY,
            RESPONSE_START_RECONCILIATION_CONTRACT_ID,
            RESPONSE_START_RECONCILIATION_CONTRACT_VERSION,
            RESPONSE_START_RECONCILIATION_ACTION,
            RESPONSE_START_RECONCILIATION_PATH,
            RESPONSE_START_RECONCILIATION_MEDIA_TYPE,
            &request,
        )
        .await?
        {
            ConsumerCall::Response(value) => value,
            ConsumerCall::Retry(error_code) => return Ok(ConsumerCall::Retry(error_code)),
        };
        result
            .validate()
            .map_err(|error| ApiError::Internal(error.into()))?;
        apply_reconciliation(&state.pool, &request, &result).await?;
    }
    Ok(ConsumerCall::Response(()))
}

async fn apply_reconciliation(
    pool: &sqlx::PgPool,
    request: &ResponseStartReconciliationRequest,
    result: &ResponseStartReconciliationResponse,
) -> ApiResult<()> {
    if result.state == ResponseStartReconciliationState::Pending {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    let locked = sqlx::query(
        "SELECT expires_at,consumed_at FROM workflow_response_reservations WHERE one_use_nonce=$1 AND workflow_assignment_id=$2 AND workflow_instance_id=$3 AND workflow_step_instance_id=$4 FOR UPDATE",
    )
    .bind(request.one_use_nonce)
    .bind(request.workflow_assignment_id)
    .bind(request.workflow_instance_id)
    .bind(request.workflow_step_instance_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(locked) = locked else {
        tx.commit().await?;
        return Ok(());
    };
    if locked
        .try_get::<Option<DateTime<Utc>>, _>("consumed_at")?
        .is_some()
        || locked.try_get::<DateTime<Utc>, _>("expires_at")? > Utc::now()
    {
        tx.commit().await?;
        return Ok(());
    }
    match result.state {
        ResponseStartReconciliationState::Pending => {}
        ResponseStartReconciliationState::Absent => {
            sqlx::query("DELETE FROM workflow_response_reservations WHERE one_use_nonce=$1")
                .bind(request.one_use_nonce)
                .execute(&mut *tx)
                .await?;
            release_deleted_runtime(
                &mut tx,
                request.workflow_instance_id,
                request.workflow_step_instance_id,
            )
            .await?;
        }
        ResponseStartReconciliationState::Committed => {
            let commit = result
                .commit
                .as_ref()
                .ok_or_else(|| invalid_provider("Response reconciliation commit"))?;
            sqlx::query(
                "UPDATE workflow_response_reservations SET consumed_response_id=$2,consumed_at=now() WHERE one_use_nonce=$1 AND consumed_at IS NULL",
            )
            .bind(request.one_use_nonce)
            .bind(commit.response.response_id())
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO workflow_response_projection(workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,response_installation_id,response_module_instance_id,response_id,response_revision,response_state,last_event_sequence,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,0,now()) ON CONFLICT(response_id) DO UPDATE SET response_revision=EXCLUDED.response_revision,response_state=EXCLUDED.response_state,updated_at=EXCLUDED.updated_at WHERE workflow_response_projection.last_event_sequence=0",
            )
            .bind(request.workflow_assignment_id)
            .bind(request.workflow_instance_id)
            .bind(request.workflow_step_instance_id)
            .bind(response_owner(&commit.response).0)
            .bind(response_owner(&commit.response).1)
            .bind(commit.response.response_id())
            .bind(sequence_i64(commit.revision)?)
            .bind(lifecycle_state(commit.lifecycle_state))
            .execute(&mut *tx)
            .await?;
            if commit.lifecycle_state == ResponseLifecycleState::Submitted {
                advance_workflow(
                    &mut tx,
                    request.workflow_instance_id,
                    request.workflow_step_instance_id,
                )
                .await?;
            } else if commit.lifecycle_state == ResponseLifecycleState::Deleted {
                release_deleted_runtime(
                    &mut tx,
                    request.workflow_instance_id,
                    request.workflow_step_instance_id,
                )
                .await?;
            }
        }
    }
    tx.commit().await?;
    Ok(())
}

async fn consumer_state(pool: &sqlx::PgPool) -> ApiResult<ConsumerState> {
    let row = sqlx::query(
        "SELECT provider_epoch,committed_sequence,observed_head_sequence FROM workflow_response_event_consumer_state WHERE singleton=true",
    )
    .fetch_one(pool)
    .await?;
    Ok(ConsumerState {
        provider_epoch: row.try_get("provider_epoch")?,
        committed_sequence: u64::try_from(row.try_get::<i64, _>("committed_sequence")?)
            .map_err(|_| invalid_provider("Response consumer sequence"))?,
        observed_head_sequence: u64::try_from(row.try_get::<i64, _>("observed_head_sequence")?)
            .map_err(|_| invalid_provider("Response observed head sequence"))?,
    })
}

async fn reset_consumer_epoch(
    pool: &sqlx::PgPool,
    provider_epoch: Uuid,
    observed_head_sequence: u64,
) -> ApiResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM workflow_response_consumed_events")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE workflow_response_event_consumer_state SET provider_epoch=$1,committed_sequence=0,observed_head_sequence=$2,synchronized_at=NULL WHERE singleton=true",
    )
    .bind(provider_epoch)
    .bind(sequence_i64(observed_head_sequence)?)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

async fn configured_event_page_size(pool: &sqlx::PgPool) -> ApiResult<u16> {
    let row = sqlx::query(
        "SELECT (instances.configuration->>'workflow_event_page_size')::integer AS page_size
         FROM module_instances instances
         JOIN application_installations installations
           ON installations.id=instances.installation_id AND installations.singleton=true
         WHERE instances.definition_id=$1 AND instances.identity_state='live'
           AND instances.installed AND instances.deployed AND instances.configured
           AND instances.enabled",
    )
    .bind(RESPONSE_MODULE_DEFINITION_ID)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| invalid_provider("Response event page-size configuration"))?;
    let value: i32 = row
        .try_get::<Option<i32>, _>("page_size")?
        .ok_or_else(|| invalid_provider("Response event page-size configuration"))?;
    let value = u16::try_from(value)
        .map_err(|_| invalid_provider("Response event page-size configuration"))?;
    if !(1..=MAX_RESPONSE_EVENT_PAGE_SIZE).contains(&value) {
        return Err(invalid_provider("Response event page-size configuration"));
    }
    Ok(value)
}

async fn mark_attempt(pool: &sqlx::PgPool) -> ApiResult<()> {
    sqlx::query(
        "UPDATE workflow_response_event_consumer_state SET last_attempt_at=now() WHERE singleton=true",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn observe_head(
    pool: &sqlx::PgPool,
    provider_epoch: Uuid,
    observed_head_sequence: u64,
) -> ApiResult<()> {
    sqlx::query(
        "UPDATE workflow_response_event_consumer_state
         SET observed_head_sequence=CASE
           WHEN provider_epoch=$1 THEN GREATEST(observed_head_sequence,$2)
           ELSE $2
         END
         WHERE singleton=true",
    )
    .bind(provider_epoch)
    .bind(sequence_i64(observed_head_sequence)?)
    .execute(pool)
    .await?;
    Ok(())
}

async fn mark_stable(pool: &sqlx::PgPool) -> ApiResult<()> {
    sqlx::query(
        "UPDATE workflow_response_event_consumer_state
         SET synchronized_at=now(),last_error_at=NULL,last_error_code=NULL
         WHERE singleton=true",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn mark_error(pool: &sqlx::PgPool, error_code: ConsumerErrorCode) -> ApiResult<()> {
    sqlx::query(
        "UPDATE workflow_response_event_consumer_state
         SET last_error_at=now(),last_error_code=$1 WHERE singleton=true",
    )
    .bind(error_code.as_str())
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn call<TRequest: serde::Serialize, TResponse: serde::de::DeserializeOwned>(
    state: &AppState,
    dependency_binding: &str,
    contract: &str,
    version: &str,
    action: &str,
    path: &str,
    media_type: &str,
    body: &TRequest,
) -> ApiResult<ConsumerCall<TResponse>> {
    match call_private_provider_for_system_job(
        state,
        CoreSystemJobProviderRequest {
            system_job_id: WORKFLOW_EVENT_SYSTEM_JOB_ID,
            module_definition_id: RESPONSE_MODULE_DEFINITION_ID,
            expected_owner: None,
            dependency_binding,
            functional_contract: contract,
            contract_version: version,
            authorization_action: action,
            path,
            media_type,
            correlation_id: Uuid::new_v4(),
            body,
        },
    )
    .await?
    {
        CorePrivateProviderResult::Response(value) => Ok(ConsumerCall::Response(value)),
        CorePrivateProviderResult::Unavailable => Ok(ConsumerCall::Retry(
            ConsumerErrorCode::ResponseProviderUnavailable,
        )),
        CorePrivateProviderResult::Undisclosed => Ok(ConsumerCall::Retry(
            ConsumerErrorCode::ResponseProviderUndisclosed,
        )),
    }
}

fn validate_checkpoint(
    value: &ResponseEventCheckpointResponse,
    requested_committed_sequence: u64,
) -> ApiResult<()> {
    let expected_valid = requested_committed_sequence <= value.authenticated_head;
    if value.schema_version != RESPONSE_EVENT_SCHEMA_VERSION
        || value.provider_epoch.is_nil()
        || value.committed_sequence_valid != expected_valid
        || value.changed
            != (!expected_valid || requested_committed_sequence != value.authenticated_head)
    {
        return Err(invalid_provider("Response event checkpoint"));
    }
    Ok(())
}

fn lifecycle_state(value: ResponseLifecycleState) -> &'static str {
    match value {
        ResponseLifecycleState::Draft => "draft",
        ResponseLifecycleState::Submitted => "submitted",
        ResponseLifecycleState::Deleted => "deleted",
    }
}

fn response_owner(reference: &tessara_responses_contract::ResponseReference) -> (Uuid, Uuid) {
    match reference.reference().owner() {
        ResourceOwner::ModuleInstance {
            installation_id,
            module_instance_id,
        } => (*installation_id, *module_instance_id),
        ResourceOwner::CoreInstallation { .. } => {
            unreachable!("ResponseReference validates module-instance ownership")
        }
    }
}

fn parse_time(value: &str) -> ApiResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| invalid_provider("Response event timestamp"))
}

fn sequence_i64(value: u64) -> ApiResult<i64> {
    i64::try_from(value).map_err(|_| invalid_provider("Response event sequence"))
}

fn invalid_provider(context: &str) -> ApiError {
    ApiError::Internal(anyhow::anyhow!("{context} violated its contract"))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProjectionRevisionAction {
    IgnoreStale,
    Apply,
}

fn projection_revision_action(
    current_revision: u64,
    current_state: &str,
    last_event_sequence: i64,
    incoming_revision: u64,
    incoming_state: &str,
) -> ApiResult<ProjectionRevisionAction> {
    if incoming_revision < current_revision {
        return Ok(ProjectionRevisionAction::IgnoreStale);
    }
    if incoming_revision > current_revision.saturating_add(1) {
        return Err(invalid_provider("Response event revision gap"));
    }
    if incoming_revision == current_revision
        && current_state != incoming_state
        && last_event_sequence != 0
    {
        return Err(invalid_provider("Response event revision conflict"));
    }
    Ok(ProjectionRevisionAction::Apply)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{
        Router,
        body::{Body, to_bytes},
        extract::{Request, State},
        http::{Method, StatusCode},
        response::{IntoResponse, Response},
        routing::any,
    };
    use serde::Serialize;
    use serde_json::json;
    use tessara_module_contract::ModuleManifest;

    use super::*;

    #[derive(Clone)]
    struct MockEventProvider {
        provider_epoch: Uuid,
        events: Arc<Mutex<Vec<ResponseWorkflowEvent>>>,
        fail_page_after: Arc<Mutex<Option<u64>>>,
        reorder_next_page: Arc<Mutex<bool>>,
        requested_page_sizes: Arc<Mutex<Vec<u16>>>,
        checkpoint_requests: Arc<Mutex<Vec<u64>>>,
    }

    impl MockEventProvider {
        fn new(provider_epoch: Uuid) -> Self {
            Self {
                provider_epoch,
                events: Arc::new(Mutex::new(Vec::new())),
                fail_page_after: Arc::new(Mutex::new(None)),
                reorder_next_page: Arc::new(Mutex::new(false)),
                requested_page_sizes: Arc::new(Mutex::new(Vec::new())),
                checkpoint_requests: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    async fn mock_event_provider(
        State(provider): State<MockEventProvider>,
        request: Request,
    ) -> Response {
        if request.method() == Method::PUT {
            return Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty())
                .unwrap();
        }
        if request.headers().get("x-tessara-authorization").is_none()
            || request
                .headers()
                .get("x-tessara-core-service-request")
                .is_none()
        {
            return StatusCode::NOT_FOUND.into_response();
        }
        let path = request.uri().path().to_owned();
        let body = match to_bytes(request.into_body(), 1024 * 1024).await {
            Ok(body) => body,
            Err(_) => return StatusCode::NOT_FOUND.into_response(),
        };
        let events = provider.events.lock().unwrap().clone();
        let head = events.last().map_or(0, |event| event.sequence);
        match path.as_str() {
            RESPONSE_EVENT_CHECKPOINT_PATH => {
                let Ok(request) = serde_json::from_slice::<ResponseEventCheckpointRequest>(&body)
                else {
                    return StatusCode::NOT_FOUND.into_response();
                };
                provider
                    .checkpoint_requests
                    .lock()
                    .unwrap()
                    .push(request.committed_sequence);
                contract_response(&ResponseEventCheckpointResponse {
                    schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
                    provider_epoch: provider.provider_epoch,
                    authenticated_head: head,
                    committed_sequence_valid: request.committed_sequence <= head,
                    changed: request.committed_sequence != head,
                })
            }
            RESPONSE_EVENT_START_PATH => {
                let Ok(request) = serde_json::from_slice::<ResponseEventStartRequest>(&body) else {
                    return StatusCode::NOT_FOUND.into_response();
                };
                provider
                    .requested_page_sizes
                    .lock()
                    .unwrap()
                    .push(request.page_size);
                contract_response(&ResponseEventStartResponse {
                    schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
                    provider_epoch: provider.provider_epoch,
                    start_after_sequence: request.committed_sequence,
                    snapshot_upper_bound: request.authenticated_head,
                })
            }
            RESPONSE_EVENT_PAGE_PATH => {
                let Ok(request) = serde_json::from_slice::<ResponseEventPageRequest>(&body) else {
                    return StatusCode::NOT_FOUND.into_response();
                };
                provider
                    .requested_page_sizes
                    .lock()
                    .unwrap()
                    .push(request.page_size);
                if provider
                    .fail_page_after
                    .lock()
                    .unwrap()
                    .is_some_and(|after| after == request.after_sequence)
                {
                    return StatusCode::SERVICE_UNAVAILABLE.into_response();
                }
                let matching = events
                    .into_iter()
                    .filter(|event| {
                        event.sequence > request.after_sequence
                            && event.sequence <= request.snapshot_upper_bound
                    })
                    .collect::<Vec<_>>();
                let complete = matching.len() <= usize::from(request.page_size);
                let mut entries = matching
                    .into_iter()
                    .take(usize::from(request.page_size))
                    .collect::<Vec<_>>();
                let next_after_sequence = if complete {
                    request.snapshot_upper_bound
                } else {
                    entries
                        .last()
                        .map_or(request.after_sequence, |event| event.sequence)
                };
                let mut page = ResponseEventPageResponse {
                    schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
                    provider_epoch: provider.provider_epoch,
                    snapshot_upper_bound: request.snapshot_upper_bound,
                    entries: entries.clone(),
                    next_after_sequence,
                    complete,
                    page_digest: String::new(),
                }
                .with_recomputed_digest()
                .unwrap();
                if std::mem::take(&mut *provider.reorder_next_page.lock().unwrap()) {
                    entries.reverse();
                    page.entries = entries;
                }
                contract_response(&page)
            }
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    }

    fn contract_response(value: &impl Serialize) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, RESPONSE_EVENT_MEDIA_TYPE)
            .body(Body::from(serde_json::to_vec(value).unwrap()))
            .unwrap()
    }

    async fn install_response_provider(pool: &sqlx::PgPool, endpoint_port: u16, page_size: u16) {
        let installation_id: Uuid =
            sqlx::query_scalar("SELECT id FROM application_installations WHERE singleton=true")
                .fetch_one(pool)
                .await
                .unwrap();
        let mut manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../tessara-response-module/manifest.json"))
                .unwrap();
        manifest["deployment"]["declaration"]["listen"]["registration_name"] = json!("127.0.0.1");
        manifest["deployment"]["declaration"]["listen"]["port"] = json!(endpoint_port);
        let manifest: ModuleManifest = serde_json::from_value(manifest).unwrap();
        sqlx::query(
            "INSERT INTO module_definition_reservations(definition_id,display_name)
             VALUES($1,'Responses') ON CONFLICT(definition_id) DO NOTHING",
        )
        .bind(RESPONSE_MODULE_DEFINITION_ID)
        .execute(pool)
        .await
        .unwrap();
        let release_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO module_releases(id,definition_id,version,manifest_digest,manifest,runtime_image_digest,publisher,trust_state,compatibility_state)
             VALUES($1,$2,'1.0.0',$3,$4,$5,'tessara.first_party','curated','compatible')",
        )
        .bind(release_id)
        .bind(RESPONSE_MODULE_DEFINITION_ID)
        .bind(format!("sha256:{}", "a".repeat(64)))
        .bind(sqlx::types::Json(manifest))
        .bind(format!("sha256:{}", "b".repeat(64)))
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO module_instances(id,installation_id,definition_id,release_id,identity_state,data_state,database_name,configuration,route_prefix,installed,deployed,configured,ready,enabled,healthy,last_observed_at)
             VALUES($1,$2,$3,$4,'live','retained','responses',$5,'/responses',true,true,true,true,true,true,now())",
        )
        .bind(Uuid::new_v4())
        .bind(installation_id)
        .bind(RESPONSE_MODULE_DEFINITION_ID)
        .bind(release_id)
        .bind(json!({
            "schema_version": 1,
            "display_label": "Responses",
            "provider_request_timeout_seconds": 5,
            "workflow_event_page_size": page_size
        }))
        .execute(pool)
        .await
        .unwrap();
    }

    fn event(
        sequence: u64,
        revision: u64,
        kind: ResponseEventKind,
        response: &tessara_responses_contract::ResponseReference,
        assignment_id: Uuid,
        instance_id: Uuid,
        step_instance_id: Uuid,
    ) -> ResponseWorkflowEvent {
        ResponseWorkflowEvent {
            schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
            sequence,
            event_id: Uuid::new_v4(),
            kind,
            response: response.clone(),
            response_revision: revision,
            workflow_assignment_id: assignment_id,
            workflow_instance_id: instance_id,
            workflow_step_instance_id: step_instance_id,
            occurred_at: (Utc::now() - chrono::Duration::seconds(61)).to_rfc3339(),
            content_digest: String::new(),
        }
        .with_recomputed_digest()
        .unwrap()
    }

    #[test]
    fn projection_revision_policy_is_stale_safe_gap_intolerant_and_reconciliation_aware() {
        assert_eq!(
            projection_revision_action(3, "submitted", 9, 2, "draft").unwrap(),
            ProjectionRevisionAction::IgnoreStale
        );
        assert_eq!(
            projection_revision_action(1, "draft", 8, 2, "submitted").unwrap(),
            ProjectionRevisionAction::Apply
        );
        assert!(projection_revision_action(1, "draft", 8, 3, "submitted").is_err());
        assert!(projection_revision_action(2, "draft", 8, 2, "deleted").is_err());
        assert_eq!(
            projection_revision_action(2, "deleted", 0, 2, "deleted").unwrap(),
            ProjectionRevisionAction::Apply
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn background_consumer_cancels_cleanly_without_losing_the_durable_cursor(
        pool: sqlx::PgPool,
    ) {
        let state = AppState {
            pool: pool.clone(),
            config: crate::config::Config {
                database_url: String::new(),
                installation_id: None,
                bind_addr: "127.0.0.1:0".into(),
                dev_admin_email: "admin@tessara.local".into(),
                dev_admin_password: "test-only-password".into(),
                auth_cookie_name: "tessara_session".into(),
                auth_cookie_secure: false,
                auth_session_ttl_hours: 1,
            },
        };
        let task = crate::spawn_workflow_response_event_consumer(state);
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let error_code: Option<String> = sqlx::query_scalar(
                    "SELECT last_error_code FROM workflow_response_event_consumer_state WHERE singleton=true",
                )
                .fetch_one(&pool)
                .await
                .unwrap();
                if error_code.as_deref() == Some("response_provider_unavailable") {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("background consumer recorded its retry state");
        task.abort();
        let cancellation = task
            .await
            .expect_err("consumer runs until service shutdown");
        assert!(cancellation.is_cancelled());
        let durable_state: (i64, i64, Option<DateTime<Utc>>) = sqlx::query_as(
            "SELECT committed_sequence,observed_head_sequence,last_attempt_at
             FROM workflow_response_event_consumer_state WHERE singleton=true",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!((durable_state.0, durable_state.1), (0, 0));
        assert!(durable_state.2.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn autonomous_consumer_recovers_owner_start_save_submit_backlog_while_unready(
        pool: sqlx::PgPool,
    ) {
        let config = crate::config::Config {
            database_url: String::new(),
            installation_id: None,
            bind_addr: "127.0.0.1:0".into(),
            dev_admin_email: "admin@tessara.local".into(),
            dev_admin_password: "test-only-password".into(),
            auth_cookie_name: "tessara_session".into(),
            auth_cookie_secure: false,
            auth_session_ttl_hours: 1,
        };
        crate::db::seed_dev_admin(&pool, &config).await.unwrap();
        crate::demo::seed_demo(&pool).await.unwrap();
        let assignment = sqlx::query(
            "SELECT wa.id,wa.workflow_version_id,wa.workflow_step_id,wa.node_id,wa.account_id FROM workflow_assignments wa JOIN workflow_versions wv ON wv.id=wa.workflow_version_id WHERE wa.is_active=true AND wv.status IN ('published'::form_version_status,'superseded'::form_version_status) ORDER BY wa.created_at LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let assignment_id: Uuid = assignment.get("id");
        let next_form_version_id: Uuid = sqlx::query_scalar(
            "SELECT id FROM form_versions WHERE id<>(SELECT form_version_id FROM workflow_steps WHERE id=$1) AND status='published'::form_version_status ORDER BY id LIMIT 1",
        )
        .bind(assignment.get::<Uuid, _>("workflow_step_id"))
        .fetch_one(&pool)
        .await
        .unwrap();
        let next_step_id: Uuid = sqlx::query_scalar(
            "INSERT INTO workflow_steps(workflow_version_id,form_version_id,title,position) VALUES($1,$2,'Second response step',1) RETURNING id",
        )
        .bind(assignment.get::<Uuid, _>("workflow_version_id"))
        .bind(next_form_version_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        let instance_id: Uuid = sqlx::query_scalar(
            "INSERT INTO workflow_instances(workflow_assignment_id,workflow_version_id,node_id,assignee_account_id,started_by_account_id) VALUES($1,$2,$3,$4,$4) RETURNING id",
        )
        .bind(assignment_id)
        .bind(assignment.get::<Uuid, _>("workflow_version_id"))
        .bind(assignment.get::<Uuid, _>("node_id"))
        .bind(assignment.get::<Uuid, _>("account_id"))
        .fetch_one(&pool)
        .await
        .unwrap();
        let step_instance_id: Uuid = sqlx::query_scalar(
            "INSERT INTO workflow_step_instances(workflow_instance_id,workflow_step_id) VALUES($1,$2) RETURNING id",
        )
        .bind(instance_id)
        .bind(assignment.get::<Uuid, _>("workflow_step_id"))
        .fetch_one(&pool)
        .await
        .unwrap();
        let nonce = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO workflow_response_reservations(workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,started_by_account_id,one_use_nonce,context_payload,context_digest,expires_at) VALUES($1,$2,$3,$4,$5,'{}',$6,now()+interval '5 minutes')",
        )
        .bind(assignment_id)
        .bind(instance_id)
        .bind(step_instance_id)
        .bind(assignment.get::<Uuid, _>("account_id"))
        .bind(nonce)
        .bind(format!("sha256:{}", "1".repeat(64)))
        .execute(&pool)
        .await
        .unwrap();
        let response = tessara_responses_contract::ResponseReference::from_parts(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
        .unwrap();
        let started = event(
            1,
            1,
            ResponseEventKind::Started,
            &response,
            assignment_id,
            instance_id,
            step_instance_id,
        );
        let saved = event(
            2,
            2,
            ResponseEventKind::DraftSaved,
            &response,
            assignment_id,
            instance_id,
            step_instance_id,
        );
        let submitted = event(
            3,
            3,
            ResponseEventKind::Submitted,
            &response,
            assignment_id,
            instance_id,
            step_instance_id,
        );

        let provider = MockEventProvider::new(Uuid::new_v4());
        *provider.events.lock().unwrap() = vec![started, saved, submitted];
        *provider.fail_page_after.lock().unwrap() = Some(2);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint_port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(
            axum::serve(
                listener,
                Router::new()
                    .fallback(any(mock_event_provider))
                    .with_state(provider.clone()),
            )
            .into_future(),
        );
        install_response_provider(&pool, endpoint_port, 2).await;
        let state = AppState {
            pool: pool.clone(),
            config,
        };

        run_cycle(&state).await.unwrap();
        let outage_state: (i64, Option<DateTime<Utc>>, Option<String>) = sqlx::query_as(
            "SELECT committed_sequence,synchronized_at,last_error_code
             FROM workflow_response_event_consumer_state WHERE singleton=true",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            outage_state,
            (2, None, Some("response_provider_unavailable".into()))
        );
        let outage_projection: (i64, String) = sqlx::query_as(
            "SELECT response_revision,response_state FROM workflow_response_projection WHERE response_id=$1",
        )
        .bind(response.response_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(outage_projection, (2, "draft".into()));
        let current_step_state: String =
            sqlx::query_scalar("SELECT status FROM workflow_step_instances WHERE id=$1")
                .bind(step_instance_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(current_step_state, "in_progress");
        let premature_next_assignment_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM workflow_assignments WHERE workflow_step_id=$1",
        )
        .bind(next_step_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(premature_next_assignment_count, 0);

        sqlx::query("UPDATE module_instances SET ready=false,healthy=false WHERE definition_id=$1")
            .bind(RESPONSE_MODULE_DEFINITION_ID)
            .execute(&pool)
            .await
            .unwrap();
        *provider.fail_page_after.lock().unwrap() = None;
        run_cycle(&state).await.unwrap();

        let projection: (i64, String) = sqlx::query_as(
            "SELECT response_revision,response_state FROM workflow_response_projection WHERE response_id=$1",
        )
        .bind(response.response_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(projection, (3, "submitted".into()));
        let step_state: String =
            sqlx::query_scalar("SELECT status FROM workflow_step_instances WHERE id=$1")
                .bind(step_instance_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(step_state, "completed");
        let next_assignment: (Uuid, Uuid, i64) = sqlx::query_as(
            "SELECT node_id,account_id,COUNT(*) OVER() FROM workflow_assignments WHERE workflow_step_id=$1",
        )
        .bind(next_step_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(next_assignment.0, assignment.get::<Uuid, _>("node_id"));
        assert_eq!(next_assignment.1, assignment.get::<Uuid, _>("account_id"));
        assert_eq!(
            next_assignment.2, 1,
            "an outage retry must not double-advance"
        );
        let instance_state: String =
            sqlx::query_scalar("SELECT status FROM workflow_instances WHERE id=$1")
                .bind(instance_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(instance_state, "in_progress");

        provider.events.lock().unwrap().push(event(
            4,
            2,
            ResponseEventKind::DraftSaved,
            &response,
            assignment_id,
            instance_id,
            step_instance_id,
        ));
        run_cycle(&state).await.unwrap();
        let stale_projection: (i64, String, i64) = sqlx::query_as(
            "SELECT response_revision,response_state,last_event_sequence
             FROM workflow_response_projection WHERE response_id=$1",
        )
        .bind(response.response_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(stale_projection, (3, "submitted".into(), 3));
        let stable_before_reordered: DateTime<Utc> = sqlx::query_scalar(
            "SELECT synchronized_at FROM workflow_response_event_consumer_state WHERE singleton=true",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        {
            let mut events = provider.events.lock().unwrap();
            events.push(event(
                5,
                2,
                ResponseEventKind::DraftSaved,
                &response,
                assignment_id,
                instance_id,
                step_instance_id,
            ));
            events.push(event(
                6,
                1,
                ResponseEventKind::DraftSaved,
                &response,
                assignment_id,
                instance_id,
                step_instance_id,
            ));
        }
        *provider.reorder_next_page.lock().unwrap() = true;
        run_cycle(&state).await.unwrap();
        let reordered_state: (i64, DateTime<Utc>, Option<String>) = sqlx::query_as(
            "SELECT committed_sequence,synchronized_at,last_error_code
             FROM workflow_response_event_consumer_state WHERE singleton=true",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(reordered_state.0, 4);
        assert_eq!(reordered_state.1, stable_before_reordered);
        assert_eq!(reordered_state.2.as_deref(), Some("consumer_failure"));

        run_cycle(&state).await.unwrap();
        let final_state: (i64, i64, Option<String>) = sqlx::query_as(
            "SELECT committed_sequence,observed_head_sequence,last_error_code
             FROM workflow_response_event_consumer_state WHERE singleton=true",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(final_state, (6, 6, None));
        let consumed_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM workflow_response_consumed_events")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(consumed_count, 6);
        let checkpoint_requests = provider.checkpoint_requests.lock().unwrap().clone();
        assert_eq!(&checkpoint_requests[..3], &[0, 2, 3]);
        let requested_page_sizes = provider.requested_page_sizes.lock().unwrap().clone();
        assert!(!requested_page_sizes.is_empty());
        assert!(requested_page_sizes.iter().all(|page_size| *page_size == 2));

        server.abort();
    }
}
