use std::collections::BTreeSet;

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
    MAX_EXPORT_PAGE_SIZE, RESPONSE_EXPORT_BINDING_KEY, RESPONSE_EXPORT_CHECKPOINT_ACTION,
    RESPONSE_EXPORT_CHECKPOINT_PATH, RESPONSE_EXPORT_CONTRACT_ID, RESPONSE_EXPORT_MEDIA_TYPE,
    RESPONSE_EXPORT_PAGE_ACTION, RESPONSE_EXPORT_PAGE_PATH, RESPONSE_EXPORT_SCHEMA_VERSION,
    RESPONSE_EXPORT_START_ACTION, RESPONSE_EXPORT_START_PATH, ResponseExportAction,
    ResponseExportCheckpointRequest, ResponseExportCheckpointResponse, ResponseExportCursor,
    ResponseExportEntry, ResponseExportPageRequest, ResponseExportPageResponse,
    ResponseExportPartition, ResponseExportStartRequest, ResponseExportStartResponse,
    SubmittedResponseChange,
};
use uuid::Uuid;

use crate::ResponseRuntime;

pub(crate) fn routes() -> Router<std::sync::Arc<ResponseRuntime>> {
    Router::new()
        .route(RESPONSE_EXPORT_CHECKPOINT_PATH, post(checkpoint))
        .route(RESPONSE_EXPORT_START_PATH, post(start))
        .route(RESPONSE_EXPORT_PAGE_PATH, post(page))
}

async fn checkpoint(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    export_response(async {
        let grant = authorize(
            &runtime,
            &headers,
            &body,
            RESPONSE_EXPORT_CHECKPOINT_PATH,
            RESPONSE_EXPORT_CHECKPOINT_ACTION,
        )
        .await?;
        let request: ResponseExportCheckpointRequest =
            serde_json::from_slice(&body).map_err(|_| ())?;
        if request.action != ResponseExportAction::Checkpoint {
            return Err(());
        }
        let digest = validate_partition(&grant.payload, &request.partition)?;
        let epoch = provider_epoch(&runtime).await?;
        let head_sequence = partition_head(&runtime, &request.partition).await?;
        let head = ProviderCursor::new(epoch, digest.clone(), head_sequence).encode()?;
        let committed = request
            .committed_cursor
            .as_ref()
            .and_then(|cursor| ProviderCursor::parse(cursor, &digest).ok());
        let committed_cursor_valid = request.committed_cursor.is_none()
            || committed
                .as_ref()
                .is_some_and(|cursor| cursor.epoch == epoch && cursor.sequence <= head_sequence);
        contract_response(&ResponseExportCheckpointResponse {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            provider_epoch: epoch,
            authenticated_head: head,
            committed_cursor_valid,
            changed: !committed_cursor_valid
                || committed
                    .as_ref()
                    .is_none_or(|cursor| cursor.sequence != head_sequence),
        })
    })
    .await
}

async fn start(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    export_response(async {
        let grant = authorize(
            &runtime,
            &headers,
            &body,
            RESPONSE_EXPORT_START_PATH,
            RESPONSE_EXPORT_START_ACTION,
        )
        .await?;
        let request: ResponseExportStartRequest = serde_json::from_slice(&body).map_err(|_| ())?;
        request.validate().map_err(|_| ())?;
        let digest = validate_partition(&grant.payload, &request.partition)?;
        let epoch = provider_epoch(&runtime).await?;
        let head_sequence = partition_head(&runtime, &request.partition).await?;
        let head = ProviderCursor::parse(&request.authenticated_head, &digest)?;
        if request.provider_epoch != epoch || head.epoch != epoch || head.sequence != head_sequence
        {
            return Err(());
        }
        let start_after_cursor = if request.full_snapshot_rebase {
            None
        } else {
            let cursor = request.committed_cursor.as_ref().ok_or(())?;
            let parsed = ProviderCursor::parse(cursor, &digest)?;
            if parsed.epoch != epoch || parsed.sequence > head_sequence {
                return Err(());
            }
            Some(cursor.clone())
        };
        contract_response(&ResponseExportStartResponse {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            provider_epoch: epoch,
            start_after_cursor,
            snapshot_upper_bound: request.authenticated_head,
            full_snapshot_rebase: request.full_snapshot_rebase,
        })
    })
    .await
}

async fn page(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    export_response(async {
        let grant = authorize(
            &runtime,
            &headers,
            &body,
            RESPONSE_EXPORT_PAGE_PATH,
            RESPONSE_EXPORT_PAGE_ACTION,
        )
        .await?;
        let request: ResponseExportPageRequest = serde_json::from_slice(&body).map_err(|_| ())?;
        if request.action != ResponseExportAction::Page
            || request.page_size == 0
            || request.page_size > MAX_EXPORT_PAGE_SIZE
        {
            return Err(());
        }
        let digest = validate_partition(&grant.payload, &request.partition)?;
        let epoch = provider_epoch(&runtime).await?;
        let upper = ProviderCursor::parse(&request.snapshot_upper_bound, &digest)?;
        let after = match &request.after_cursor {
            Some(cursor) => ProviderCursor::parse(cursor, &digest)?,
            None => ProviderCursor::new(epoch, digest.clone(), 0),
        };
        if request.provider_epoch != epoch
            || upper.epoch != epoch
            || after.epoch != epoch
            || after.sequence > upper.sequence
        {
            return Err(());
        }
        let rows = sqlx::query(
            "SELECT sequence,payload,content_digest FROM response_export_changes
             WHERE form_version_id=ANY($1) AND node_id=ANY($2)
               AND sequence>$3 AND sequence<=$4 ORDER BY sequence LIMIT $5",
        )
        .bind(&request.partition.form_version_ids)
        .bind(&request.partition.authorized_scope_node_ids)
        .bind(after.sequence)
        .bind(upper.sequence)
        .bind(i64::from(request.page_size) + 1)
        .fetch_all(&runtime.pool)
        .await
        .map_err(|_| ())?;
        let has_more = rows.len() > usize::from(request.page_size);
        let mut entries = Vec::with_capacity(rows.len().min(usize::from(request.page_size)));
        for row in rows.into_iter().take(usize::from(request.page_size)) {
            let sequence: i64 = row.try_get("sequence").map_err(|_| ())?;
            let change: SubmittedResponseChange =
                serde_json::from_value(row.try_get("payload").map_err(|_| ())?).map_err(|_| ())?;
            let stored_digest: String = row.try_get("content_digest").map_err(|_| ())?;
            let contract_digest = match &change {
                SubmittedResponseChange::Upsert(value) => &value.content_digest,
                SubmittedResponseChange::Tombstone { content_digest, .. } => content_digest,
            };
            if stored_digest != *contract_digest || change.validate_content_digest().is_err() {
                return Err(());
            }
            entries.push(ResponseExportEntry {
                cursor: ProviderCursor::new(epoch, digest.clone(), sequence).encode()?,
                change,
            });
        }
        let complete = !has_more;
        let next_after_cursor = (!complete)
            .then(|| entries.last().map(|entry| entry.cursor.clone()))
            .flatten();
        let response = ResponseExportPageResponse {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            provider_epoch: epoch,
            snapshot_upper_bound: request.snapshot_upper_bound,
            page_digest: ResponseExportPageResponse::canonical_page_digest(&entries)
                .map_err(|_| ())?,
            entries,
            next_after_cursor,
            complete,
        };
        response.validate().map_err(|_| ())?;
        contract_response(&response)
    })
    .await
}

async fn authorize(
    runtime: &ResponseRuntime,
    headers: &HeaderMap,
    body: &[u8],
    path: &'static str,
    action: &'static str,
) -> Result<
    tessara_module_contract::SignedEnvelopeV1<tessara_module_contract::AuthorizationGrantV3>,
    (),
> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(RESPONSE_EXPORT_MEDIA_TYPE)
    {
        return Err(());
    }
    crate::private_provider_auth::authorize(
        runtime,
        headers,
        body,
        crate::private_provider_auth::PrivateProviderContract {
            path,
            binding: RESPONSE_EXPORT_BINDING_KEY,
            contract: RESPONSE_EXPORT_CONTRACT_ID,
            action,
            capability: "submissions:manage",
        },
    )
    .await
}

fn validate_partition(
    grant: &tessara_module_contract::AuthorizationGrantV3,
    partition: &ResponseExportPartition,
) -> Result<String, ()> {
    if partition.form_version_ids.is_empty() || partition.authorized_scope_node_ids.is_empty() {
        return Err(());
    }
    let mut authorized_nodes = BTreeSet::new();
    let mut installation_wide = false;
    for binding in &grant.capability_scope_bindings {
        if binding.capability.as_str() != "submissions:manage" {
            continue;
        }
        installation_wide |= binding.organization_root_id == grant.installation_id;
        authorized_nodes.insert(binding.organization_root_id);
        authorized_nodes.extend(binding.authorized_organization_ids.iter().copied());
    }
    if !installation_wide
        && partition
            .authorized_scope_node_ids
            .iter()
            .any(|node| !authorized_nodes.contains(node))
    {
        return Err(());
    }
    partition
        .validate_authorization_digest(&grant.presenting_service, grant.installation_id)
        .map_err(|_| ())
}

async fn provider_epoch(runtime: &ResponseRuntime) -> Result<Uuid, ()> {
    sqlx::query_scalar("SELECT provider_epoch FROM response_export_state WHERE singleton=true")
        .fetch_one(&runtime.pool)
        .await
        .map_err(|_| ())
}

async fn partition_head(
    runtime: &ResponseRuntime,
    partition: &ResponseExportPartition,
) -> Result<i64, ()> {
    sqlx::query_scalar(
        "SELECT COALESCE(MAX(sequence),0) FROM response_export_changes
         WHERE form_version_id=ANY($1) AND node_id=ANY($2)",
    )
    .bind(&partition.form_version_ids)
    .bind(&partition.authorized_scope_node_ids)
    .fetch_one(&runtime.pool)
    .await
    .map_err(|_| ())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProviderCursor {
    epoch: Uuid,
    partition_digest: String,
    sequence: i64,
}

impl ProviderCursor {
    fn new(epoch: Uuid, partition_digest: String, sequence: i64) -> Self {
        Self {
            epoch,
            partition_digest,
            sequence,
        }
    }

    fn encode(&self) -> Result<ResponseExportCursor, ()> {
        ResponseExportCursor::parse(format!(
            "{}.{}.{:020}",
            self.epoch.simple(),
            self.partition_digest.trim_start_matches("sha256:"),
            self.sequence
        ))
        .map_err(|_| ())
    }

    fn parse(value: &ResponseExportCursor, expected_digest: &str) -> Result<Self, ()> {
        let mut parts = value.as_str().split('.');
        let epoch = parts
            .next()
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or(())?;
        let digest = parts.next().ok_or(())?;
        let sequence_text = parts.next().ok_or(())?;
        if parts.next().is_some()
            || digest != expected_digest.trim_start_matches("sha256:")
            || sequence_text.len() != 20
        {
            return Err(());
        }
        let sequence = sequence_text.parse::<i64>().map_err(|_| ())?;
        if sequence < 0 {
            return Err(());
        }
        Ok(Self::new(epoch, expected_digest.to_string(), sequence))
    }
}

fn contract_response<T: Serialize>(value: &T) -> Result<Response, ()> {
    Response::builder()
        .header(header::CONTENT_TYPE, RESPONSE_EXPORT_MEDIA_TYPE)
        .body(Body::from(serde_json::to_vec(value).map_err(|_| ())?))
        .map_err(|_| ())
}

async fn export_response(future: impl Future<Output = Result<Response, ()>>) -> Response {
    future
        .await
        .unwrap_or_else(|()| StatusCode::NOT_FOUND.into_response())
}
