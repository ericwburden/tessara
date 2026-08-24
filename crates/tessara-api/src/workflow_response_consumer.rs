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
    auth::AuthenticatedRequest,
    db::AppState,
    error::{ApiError, ApiResult},
    module_gateway::{
        CorePrivateProviderRequest, CorePrivateProviderResult, call_private_provider,
    },
};

const RESPONSE_MODULE_DEFINITION_ID: &str = "tessara.responses";
const WORKFLOW_CONSUMER_CAPABILITY: &str = "workflows:manage";
const EVENT_PAGE_SIZE: u16 = 250;

/// Advances the durable Workflow projection when a globally authorized
/// Workflow manager reaches a Workflow read boundary. Provider outages are an
/// expected retry state and do not invalidate the last committed projection.
pub(crate) async fn synchronize(state: &AppState, actor: &AuthenticatedRequest) -> ApiResult<()> {
    if !actor
        .account
        .has_global_capability(WORKFLOW_CONSUMER_CAPABILITY)
    {
        return Ok(());
    }
    consume_events(state, actor).await?;
    reconcile_expired_reservations(state, actor).await
}

async fn consume_events(state: &AppState, actor: &AuthenticatedRequest) -> ApiResult<()> {
    let (stored_epoch, committed_sequence) = consumer_state(&state.pool).await?;
    let checkpoint_request = ResponseEventCheckpointRequest {
        schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
        committed_sequence,
    };
    let checkpoint = match call::<_, ResponseEventCheckpointResponse>(
        state,
        actor,
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
        Some(value) => value,
        None => return Ok(()),
    };
    validate_checkpoint(&checkpoint)?;

    let committed_sequence = if stored_epoch != Some(checkpoint.provider_epoch)
        || !checkpoint.committed_sequence_valid
    {
        reset_consumer_epoch(&state.pool, checkpoint.provider_epoch).await?;
        0
    } else {
        committed_sequence
    };
    if !checkpoint.changed && committed_sequence == checkpoint.authenticated_head {
        return Ok(());
    }

    let start_request = ResponseEventStartRequest {
        schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
        provider_epoch: checkpoint.provider_epoch,
        committed_sequence,
        authenticated_head: checkpoint.authenticated_head,
        page_size: EVENT_PAGE_SIZE.min(MAX_RESPONSE_EVENT_PAGE_SIZE),
    };
    start_request
        .validate()
        .map_err(|error| ApiError::Internal(error.into()))?;
    let Some(window) = call::<_, ResponseEventStartResponse>(
        state,
        actor,
        RESPONSE_EVENT_BINDING_KEY,
        RESPONSE_EVENT_CONTRACT_ID,
        RESPONSE_EVENT_CONTRACT_VERSION,
        RESPONSE_EVENT_START_ACTION,
        RESPONSE_EVENT_START_PATH,
        RESPONSE_EVENT_MEDIA_TYPE,
        &start_request,
    )
    .await?
    else {
        return Ok(());
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
            page_size: EVENT_PAGE_SIZE,
        };
        let Some(page) = call::<_, ResponseEventPageResponse>(
            state,
            actor,
            RESPONSE_EVENT_BINDING_KEY,
            RESPONSE_EVENT_CONTRACT_ID,
            RESPONSE_EVENT_CONTRACT_VERSION,
            RESPONSE_EVENT_PAGE_ACTION,
            RESPONSE_EVENT_PAGE_PATH,
            RESPONSE_EVENT_MEDIA_TYPE,
            &request,
        )
        .await?
        else {
            return Ok(());
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
    Ok(())
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
        "UPDATE workflow_response_event_consumer_state SET committed_sequence=$1,synchronized_at=now(),last_error_at=NULL WHERE singleton=true",
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
    actor: &AuthenticatedRequest,
) -> ApiResult<()> {
    let rows = sqlx::query(
        "SELECT workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,one_use_nonce FROM workflow_response_reservations WHERE consumed_at IS NULL AND expires_at<=now() ORDER BY created_at LIMIT 250",
    )
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
        let Some(result) = call::<_, ResponseStartReconciliationResponse>(
            state,
            actor,
            RESPONSE_START_RECONCILIATION_BINDING_KEY,
            RESPONSE_START_RECONCILIATION_CONTRACT_ID,
            RESPONSE_START_RECONCILIATION_CONTRACT_VERSION,
            RESPONSE_START_RECONCILIATION_ACTION,
            RESPONSE_START_RECONCILIATION_PATH,
            RESPONSE_START_RECONCILIATION_MEDIA_TYPE,
            &request,
        )
        .await?
        else {
            return Ok(());
        };
        result
            .validate()
            .map_err(|error| ApiError::Internal(error.into()))?;
        apply_reconciliation(&state.pool, &request, &result).await?;
    }
    Ok(())
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

async fn consumer_state(pool: &sqlx::PgPool) -> ApiResult<(Option<Uuid>, u64)> {
    let row = sqlx::query(
        "SELECT provider_epoch,committed_sequence FROM workflow_response_event_consumer_state WHERE singleton=true",
    )
    .fetch_one(pool)
    .await?;
    Ok((
        row.try_get("provider_epoch")?,
        u64::try_from(row.try_get::<i64, _>("committed_sequence")?)
            .map_err(|_| invalid_provider("Response consumer sequence"))?,
    ))
}

async fn reset_consumer_epoch(pool: &sqlx::PgPool, provider_epoch: Uuid) -> ApiResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM workflow_response_consumed_events")
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE workflow_response_event_consumer_state SET provider_epoch=$1,committed_sequence=0,synchronized_at=NULL,last_error_at=NULL WHERE singleton=true",
    )
    .bind(provider_epoch)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn call<TRequest: serde::Serialize, TResponse: serde::de::DeserializeOwned>(
    state: &AppState,
    actor: &AuthenticatedRequest,
    dependency_binding: &str,
    contract: &str,
    version: &str,
    action: &str,
    path: &str,
    media_type: &str,
    body: &TRequest,
) -> ApiResult<Option<TResponse>> {
    match call_private_provider(
        state,
        actor,
        CorePrivateProviderRequest {
            module_definition_id: RESPONSE_MODULE_DEFINITION_ID,
            expected_owner: None,
            dependency_binding,
            functional_contract: contract,
            contract_version: version,
            authorization_action: action,
            path,
            media_type,
            correlation_id: Uuid::new_v4(),
            actor_capability: WORKFLOW_CONSUMER_CAPABILITY,
            body,
        },
    )
    .await?
    {
        CorePrivateProviderResult::Response(value) => Ok(Some(value)),
        CorePrivateProviderResult::Unavailable | CorePrivateProviderResult::Undisclosed => Ok(None),
    }
}

fn validate_checkpoint(value: &ResponseEventCheckpointResponse) -> ApiResult<()> {
    if value.schema_version != RESPONSE_EVENT_SCHEMA_VERSION || value.provider_epoch.is_nil() {
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
    use super::*;

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
            occurred_at: Utc::now().to_rfc3339(),
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
    async fn workflow_events_consume_once_and_advance_the_same_actor_and_node_once(
        pool: sqlx::PgPool,
    ) {
        crate::db::seed_dev_admin(
            &pool,
            &crate::config::Config {
                database_url: String::new(),
                installation_id: None,
                bind_addr: "127.0.0.1:0".into(),
                dev_admin_email: "admin@tessara.local".into(),
                dev_admin_password: "test-only-password".into(),
                auth_cookie_name: "tessara_session".into(),
                auth_cookie_secure: false,
                auth_session_ttl_hours: 1,
            },
        )
        .await
        .unwrap();
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
        let submitted = event(
            2,
            2,
            ResponseEventKind::Submitted,
            &response,
            assignment_id,
            instance_id,
            step_instance_id,
        );
        let mut tx = pool.begin().await.unwrap();
        apply_event(&mut tx, &started).await.unwrap();
        apply_event(&mut tx, &submitted).await.unwrap();
        apply_event(&mut tx, &submitted).await.unwrap();
        tx.commit().await.unwrap();

        let projection: (i64, String) = sqlx::query_as(
            "SELECT response_revision,response_state FROM workflow_response_projection WHERE response_id=$1",
        )
        .bind(response.response_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(projection, (2, "submitted".into()));
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
            "duplicate events must not double-advance"
        );
        let instance_state: String =
            sqlx::query_scalar("SELECT status FROM workflow_instances WHERE id=$1")
                .bind(instance_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(instance_state, "in_progress");

        let gap = event(
            3,
            4,
            ResponseEventKind::DraftSaved,
            &response,
            assignment_id,
            instance_id,
            step_instance_id,
        );
        let mut tx = pool.begin().await.unwrap();
        assert!(apply_event(&mut tx, &gap).await.is_err());
    }
}
