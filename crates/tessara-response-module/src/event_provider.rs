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
        let (epoch, head) = event_state(&runtime).await?;
        let committed_sequence_valid = request.committed_sequence <= head;
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
