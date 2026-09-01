use axum::{
    Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Serialize;
use sqlx::Row;
use tessara_responses_contract::{
    MAX_RESPONSE_EVENT_PAGE_SIZE, RESPONSE_EVENT_BINDING_KEY, RESPONSE_EVENT_CHECKPOINT_ACTION,
    RESPONSE_EVENT_CHECKPOINT_PATH, RESPONSE_EVENT_CONTRACT_ID, RESPONSE_EVENT_MEDIA_TYPE,
    RESPONSE_EVENT_PAGE_ACTION, RESPONSE_EVENT_PAGE_PATH, RESPONSE_EVENT_SCHEMA_VERSION,
    RESPONSE_EVENT_START_ACTION, RESPONSE_EVENT_START_PATH, ResponseEventCheckpointRequest,
    ResponseEventCheckpointResponse, ResponseEventPageRequest, ResponseEventPageResponse,
    ResponseEventStartRequest, ResponseEventStartResponse, ResponseWorkflowEvent,
};

use crate::ResponseRuntime;

pub(crate) fn routes() -> Router<std::sync::Arc<ResponseRuntime>> {
    Router::new()
        .route(RESPONSE_EVENT_CHECKPOINT_PATH, post(checkpoint))
        .route(RESPONSE_EVENT_START_PATH, post(start))
        .route(RESPONSE_EVENT_PAGE_PATH, post(page))
}

async fn checkpoint(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    event_response(async {
        require_media_type(&headers)?;
        crate::private_provider_auth::authorize(
            &runtime,
            &headers,
            &body,
            crate::private_provider_auth::PrivateProviderContract {
                path: RESPONSE_EVENT_CHECKPOINT_PATH,
                binding: RESPONSE_EVENT_BINDING_KEY,
                contract: RESPONSE_EVENT_CONTRACT_ID,
                action: RESPONSE_EVENT_CHECKPOINT_ACTION,
                capability: "submissions:manage",
            },
        )
        .await?;
        let request: ResponseEventCheckpointRequest =
            serde_json::from_slice(&body).map_err(|_| ())?;
        let (epoch, head, committed_sequence_valid) =
            checkpoint_event_state(&runtime.pool, request.committed_sequence).await?;
        contract_response(&ResponseEventCheckpointResponse {
            schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
            provider_epoch: epoch,
            authenticated_head: head,
            committed_sequence_valid,
            changed: !committed_sequence_valid || request.committed_sequence != head,
        })
    })
    .await
}

async fn start(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    event_response(async {
        require_media_type(&headers)?;
        crate::private_provider_auth::authorize(
            &runtime,
            &headers,
            &body,
            crate::private_provider_auth::PrivateProviderContract {
                path: RESPONSE_EVENT_START_PATH,
                binding: RESPONSE_EVENT_BINDING_KEY,
                contract: RESPONSE_EVENT_CONTRACT_ID,
                action: RESPONSE_EVENT_START_ACTION,
                capability: "submissions:manage",
            },
        )
        .await?;
        let request: ResponseEventStartRequest = serde_json::from_slice(&body).map_err(|_| ())?;
        request.validate().map_err(|_| ())?;
        let (epoch, head) = event_state(&runtime).await?;
        if request.provider_epoch != epoch
            || request.authenticated_head != head
            || request.committed_sequence > head
        {
            return Err(());
        }
        contract_response(&ResponseEventStartResponse {
            schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
            provider_epoch: epoch,
            start_after_sequence: request.committed_sequence,
            snapshot_upper_bound: head,
        })
    })
    .await
}

async fn page(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    event_response(async {
        require_media_type(&headers)?;
        crate::private_provider_auth::authorize(
            &runtime,
            &headers,
            &body,
            crate::private_provider_auth::PrivateProviderContract {
                path: RESPONSE_EVENT_PAGE_PATH,
                binding: RESPONSE_EVENT_BINDING_KEY,
                contract: RESPONSE_EVENT_CONTRACT_ID,
                action: RESPONSE_EVENT_PAGE_ACTION,
                capability: "submissions:manage",
            },
        )
        .await?;
        let request: ResponseEventPageRequest = serde_json::from_slice(&body).map_err(|_| ())?;
        if request.page_size == 0
            || request.page_size > MAX_RESPONSE_EVENT_PAGE_SIZE
            || request.after_sequence > request.snapshot_upper_bound
        {
            return Err(());
        }
        let (epoch, head) = event_state(&runtime).await?;
        if request.provider_epoch != epoch || request.snapshot_upper_bound > head {
            return Err(());
        }
        let rows = sqlx::query(
            "SELECT sequence,payload,content_digest FROM response_workflow_events
             WHERE sequence>$1 AND sequence<=$2 ORDER BY sequence LIMIT $3",
        )
        .bind(i64::try_from(request.after_sequence).map_err(|_| ())?)
        .bind(i64::try_from(request.snapshot_upper_bound).map_err(|_| ())?)
        .bind(i64::from(request.page_size) + 1)
        .fetch_all(&runtime.pool)
        .await
        .map_err(|_| ())?;
        let has_more = rows.len() > usize::from(request.page_size);
        let mut entries = Vec::with_capacity(rows.len().min(usize::from(request.page_size)));
        for row in rows.into_iter().take(usize::from(request.page_size)) {
            let sequence: i64 = row.try_get("sequence").map_err(|_| ())?;
            let event: ResponseWorkflowEvent =
                serde_json::from_value(row.try_get("payload").map_err(|_| ())?).map_err(|_| ())?;
            let stored_digest: String = row.try_get("content_digest").map_err(|_| ())?;
            if event.sequence != u64::try_from(sequence).map_err(|_| ())?
                || event.content_digest != stored_digest
                || event.validate().is_err()
            {
                return Err(());
            }
            entries.push(event);
        }
        let complete = !has_more;
        let next_after_sequence = entries
            .last()
            .map_or(request.after_sequence, |event| event.sequence);
        let response = ResponseEventPageResponse {
            schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
            provider_epoch: epoch,
            snapshot_upper_bound: request.snapshot_upper_bound,
            entries,
            next_after_sequence: if complete {
                request.snapshot_upper_bound
            } else {
                next_after_sequence
            },
            complete,
            page_digest: String::new(),
        }
        .with_recomputed_digest()
        .map_err(|_| ())?;
        contract_response(&response)
    })
    .await
}

async fn event_state(runtime: &ResponseRuntime) -> Result<(uuid::Uuid, u64), ()> {
    let epoch = sqlx::query_scalar(
        "SELECT provider_epoch FROM response_workflow_event_state WHERE singleton=true",
    )
    .fetch_one(&runtime.pool)
    .await
    .map_err(|_| ())?;
    let head: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(sequence),0) FROM response_workflow_events")
            .fetch_one(&runtime.pool)
            .await
            .map_err(|_| ())?;
    Ok((epoch, u64::try_from(head).map_err(|_| ())?))
}

async fn checkpoint_event_state(
    pool: &sqlx::PgPool,
    committed_sequence: u64,
) -> Result<(uuid::Uuid, u64, bool), ()> {
    let committed_sequence = i64::try_from(committed_sequence).map_err(|_| ())?;
    let mut tx = pool.begin().await.map_err(|_| ())?;
    let state = sqlx::query(
        "SELECT provider_epoch FROM response_workflow_event_state WHERE singleton=true FOR UPDATE",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| ())?;
    let epoch = state.try_get("provider_epoch").map_err(|_| ())?;
    let head: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(sequence),0) FROM response_workflow_events")
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| ())?;
    let committed_sequence_valid = committed_sequence <= head;
    if committed_sequence_valid {
        sqlx::query(
            "UPDATE response_workflow_event_state
             SET workflow_consumer_committed_sequence=GREATEST(workflow_consumer_committed_sequence,$1),
                 workflow_consumer_acknowledged_at=CASE
                   WHEN $1>=workflow_consumer_committed_sequence THEN now()
                   ELSE workflow_consumer_acknowledged_at
                 END
             WHERE singleton=true",
        )
        .bind(committed_sequence)
        .execute(&mut *tx)
        .await
        .map_err(|_| ())?;
    }
    tx.commit().await.map_err(|_| ())?;
    Ok((
        epoch,
        u64::try_from(head).map_err(|_| ())?,
        committed_sequence_valid,
    ))
}

fn require_media_type(headers: &HeaderMap) -> Result<(), ()> {
    (headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        == Some(RESPONSE_EVENT_MEDIA_TYPE))
    .then_some(())
    .ok_or(())
}

fn contract_response<T: Serialize>(value: &T) -> Result<Response, ()> {
    let body = serde_json::to_vec(value).map_err(|_| ())?;
    Response::builder()
        .header(header::CONTENT_TYPE, RESPONSE_EVENT_MEDIA_TYPE)
        .body(Body::from(body))
        .map_err(|_| ())
}

async fn event_response(future: impl Future<Output = Result<Response, ()>>) -> Response {
    future
        .await
        .unwrap_or_else(|()| StatusCode::NOT_FOUND.into_response())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn workflow_consumer_checkpoint_ack_is_monotonic_and_rejects_unpublished_heads(
        pool: sqlx::PgPool,
    ) {
        let nonce = Uuid::new_v4();
        let response_id = Uuid::new_v4();
        let digest = format!("sha256:{}", "a".repeat(64));
        sqlx::query(
            "INSERT INTO response_start_claims(
               one_use_nonce,workflow_assignment_id,workflow_instance_id,
               workflow_step_instance_id,actor_account_id,idempotency_key_digest,
               request_digest,initial_grant_jti,initial_correlation_id,expires_at,state
             ) VALUES($1,$2,$3,$4,$5,$6,$6,$7,$8,now()+interval '5 minutes','pending')",
        )
        .bind(nonce)
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(&digest)
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO responses(
               id,form_id,form_version_id,node_id,workflow_assignment_id,
               workflow_version_id,workflow_step_id,workflow_instance_id,
               workflow_step_instance_id,workflow_start_nonce,assignee_account_id,
               started_by_account_id,form_snapshot,form_snapshot_digest,
               workflow_context,workflow_context_digest
             ) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$11,'{}',$12,'{}',$12)",
        )
        .bind(response_id)
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(nonce)
        .bind(Uuid::new_v4())
        .bind(&digest)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "UPDATE response_start_claims
             SET state='committed',response_id=$2,finalized_at=now() WHERE one_use_nonce=$1",
        )
        .bind(nonce)
        .bind(response_id)
        .execute(&pool)
        .await
        .unwrap();
        for sequence in 1_i64..=3 {
            sqlx::query(
                "INSERT INTO response_workflow_events(
                   sequence,event_id,response_id,response_revision,event_kind,
                   payload,content_digest
                 ) VALUES($1,$2,$3,$1,'draft_saved','{}',$4)",
            )
            .bind(sequence)
            .bind(Uuid::new_v4())
            .bind(response_id)
            .bind(&digest)
            .execute(&pool)
            .await
            .unwrap();
        }

        let (_, head, valid) = checkpoint_event_state(&pool, 2).await.unwrap();
        assert_eq!((head, valid), (3, true));
        let first_ack: (i64, DateTime<Utc>) = sqlx::query_as(
            "SELECT workflow_consumer_committed_sequence,workflow_consumer_acknowledged_at
             FROM response_workflow_event_state WHERE singleton=true",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(first_ack.0, 2);

        assert!(checkpoint_event_state(&pool, 1).await.unwrap().2);
        let stale_ack: (i64, DateTime<Utc>) = sqlx::query_as(
            "SELECT workflow_consumer_committed_sequence,workflow_consumer_acknowledged_at
             FROM response_workflow_event_state WHERE singleton=true",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(stale_ack, first_ack);

        assert!(!checkpoint_event_state(&pool, 4).await.unwrap().2);
        let unpublished_ack: (i64, DateTime<Utc>) = sqlx::query_as(
            "SELECT workflow_consumer_committed_sequence,workflow_consumer_acknowledged_at
             FROM response_workflow_event_state WHERE singleton=true",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(unpublished_ack, first_ack);

        assert!(checkpoint_event_state(&pool, 3).await.unwrap().2);
        let final_ack: i64 = sqlx::query_scalar(
            "SELECT workflow_consumer_committed_sequence
             FROM response_workflow_event_state WHERE singleton=true",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(final_ack, 3);
    }
}
