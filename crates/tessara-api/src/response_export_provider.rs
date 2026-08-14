//! Responses-owned submitted-response export provider.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json, Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, header},
    routing::post,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::Row;
use tessara_responses_contract::{
    RESPONSE_EXPORT_CHECKPOINT_ACTION, RESPONSE_EXPORT_CHECKPOINT_PATH, RESPONSE_EXPORT_MEDIA_TYPE,
    RESPONSE_EXPORT_PAGE_ACTION, RESPONSE_EXPORT_PAGE_PATH, RESPONSE_EXPORT_SCHEMA_VERSION,
    RESPONSE_EXPORT_START_ACTION, RESPONSE_EXPORT_START_PATH, ResponseExportAction,
    ResponseExportCheckpointRequest, ResponseExportCheckpointResponse, ResponseExportCursor,
    ResponseExportEntry, ResponseExportPageRequest, ResponseExportPageResponse,
    ResponseExportPartition, ResponseExportStartRequest, ResponseExportStartResponse,
    ResponseTombstoneReason, SubmittedResponseChange, SubmittedResponseRestrictionTier,
    SubmittedResponseUpsert, SubmittedResponseValue,
};
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{ApiError, ApiResult},
};

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(RESPONSE_EXPORT_CHECKPOINT_PATH, post(checkpoint))
        .route(RESPONSE_EXPORT_START_PATH, post(start))
        .route(RESPONSE_EXPORT_PAGE_PATH, post(page))
}

async fn authorize(
    state: &AppState,
    headers: &HeaderMap,
    action: &'static str,
    path: &'static str,
    body: &[u8],
) -> ApiResult<crate::module_service_requests::CoreProviderAuthorizationV1> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(RESPONSE_EXPORT_MEDIA_TYPE)
    {
        return Err(ApiError::BadRequest(
            "Response export media type is invalid".into(),
        ));
    }
    crate::module_service_requests::authorize_core_provider(
        state,
        headers,
        crate::core_service_providers::RESPONSE_EXPORT_CONTRACT,
        action,
        path,
        body,
        "Response export is unavailable",
    )
    .await
}

fn restricted() -> ApiError {
    ApiError::NotFound("Response export is unavailable".into())
}

fn validate_partition(
    inbound: &crate::module_service_requests::CoreProviderAuthorizationV1,
    partition: &ResponseExportPartition,
) -> ApiResult<String> {
    if partition.form_version_ids.is_empty() || partition.authorized_scope_node_ids.is_empty() {
        return Err(restricted());
    }
    let authorized_nodes = inbound
        .payload
        .capability_scope_bindings
        .iter()
        .filter(|binding| binding.capability.as_str() == "datasets:manage")
        .flat_map(|binding| {
            std::iter::once(binding.organization_root_id)
                .chain(binding.authorized_organization_ids.iter().copied())
        })
        .collect::<BTreeSet<_>>();
    if partition
        .authorized_scope_node_ids
        .iter()
        .any(|node| !authorized_nodes.contains(node))
    {
        return Err(restricted());
    }
    partition
        .validate_authorization_digest(
            &inbound.payload.presenting_service,
            inbound.payload.installation_id,
        )
        .map_err(|_| restricted())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProviderCursor {
    epoch: Uuid,
    partition_digest: String,
    sequence: i64,
}

impl ProviderCursor {
    fn encode(&self) -> ApiResult<ResponseExportCursor> {
        ResponseExportCursor::parse(format!(
            "{}.{}.{:020}",
            self.epoch.simple(),
            self.partition_digest.trim_start_matches("sha256:"),
            self.sequence
        ))
        .map_err(|_| restricted())
    }

    fn parse(value: &ResponseExportCursor, expected_digest: &str) -> ApiResult<Self> {
        let mut parts = value.as_str().split('.');
        let epoch = parts
            .next()
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or_else(restricted)?;
        let digest = parts.next().ok_or_else(restricted)?;
        let sequence_text = parts.next().ok_or_else(restricted)?;
        if parts.next().is_some()
            || digest != expected_digest.trim_start_matches("sha256:")
            || sequence_text.len() != 20
        {
            return Err(restricted());
        }
        let sequence = sequence_text.parse::<i64>().map_err(|_| restricted())?;
        if sequence < 0 {
            return Err(restricted());
        }
        Ok(Self {
            epoch,
            partition_digest: expected_digest.to_string(),
            sequence,
        })
    }
}

async fn provider_epoch(state: &AppState) -> ApiResult<Uuid> {
    sqlx::query_scalar("SELECT provider_epoch FROM response_export_state WHERE singleton")
        .fetch_one(&state.pool)
        .await
        .map_err(Into::into)
}

async fn partition_head(state: &AppState, partition: &ResponseExportPartition) -> ApiResult<i64> {
    sqlx::query_scalar(
        "SELECT COALESCE(MAX(change_sequence),0)
         FROM response_export_changes
         WHERE form_version_id = ANY($1) AND node_id = ANY($2)",
    )
    .bind(&partition.form_version_ids)
    .bind(&partition.authorized_scope_node_ids)
    .fetch_one(&state.pool)
    .await
    .map_err(Into::into)
}

async fn checkpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<ResponseExportCheckpointResponse>> {
    let inbound = authorize(
        &state,
        &headers,
        RESPONSE_EXPORT_CHECKPOINT_ACTION,
        RESPONSE_EXPORT_CHECKPOINT_PATH,
        &body,
    )
    .await?;
    let request: ResponseExportCheckpointRequest =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    if request.action != ResponseExportAction::Checkpoint {
        return Err(restricted());
    }
    let digest = validate_partition(&inbound, &request.partition)?;
    let epoch = provider_epoch(&state).await?;
    let head_sequence = partition_head(&state, &request.partition).await?;
    let head = ProviderCursor {
        epoch,
        partition_digest: digest.clone(),
        sequence: head_sequence,
    }
    .encode()?;
    let committed = request
        .committed_cursor
        .as_ref()
        .and_then(|cursor| ProviderCursor::parse(cursor, &digest).ok());
    let committed_cursor_valid = request.committed_cursor.is_none()
        || committed
            .as_ref()
            .is_some_and(|cursor| cursor.epoch == epoch && cursor.sequence <= head_sequence);
    let changed = !committed_cursor_valid
        || committed
            .as_ref()
            .is_none_or(|cursor| cursor.sequence != head_sequence);
    Ok(Json(ResponseExportCheckpointResponse {
        schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
        provider_epoch: epoch,
        authenticated_head: head,
        committed_cursor_valid,
        changed,
    }))
}

async fn start(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<ResponseExportStartResponse>> {
    let inbound = authorize(
        &state,
        &headers,
        RESPONSE_EXPORT_START_ACTION,
        RESPONSE_EXPORT_START_PATH,
        &body,
    )
    .await?;
    let request: ResponseExportStartRequest =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    request.validate().map_err(|_| restricted())?;
    let digest = validate_partition(&inbound, &request.partition)?;
    let epoch = provider_epoch(&state).await?;
    let head_sequence = partition_head(&state, &request.partition).await?;
    let head = ProviderCursor::parse(&request.authenticated_head, &digest)?;
    if request.provider_epoch != epoch || head.epoch != epoch || head.sequence != head_sequence {
        return Err(restricted());
    }
    let start_after_cursor = if request.full_snapshot_rebase {
        None
    } else {
        let cursor = request.committed_cursor.as_ref().ok_or_else(restricted)?;
        let parsed = ProviderCursor::parse(cursor, &digest)?;
        if parsed.epoch != epoch || parsed.sequence > head_sequence {
            return Err(restricted());
        }
        Some(cursor.clone())
    };
    Ok(Json(ResponseExportStartResponse {
        schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
        provider_epoch: epoch,
        start_after_cursor,
        snapshot_upper_bound: request.authenticated_head,
        full_snapshot_rebase: request.full_snapshot_rebase,
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredUpsert {
    response_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
    node_name: String,
    submitted_at: String,
    created_at: String,
    last_modified_at: String,
    last_modified_by_user_name: Option<String>,
    status: String,
    restriction_tier: SubmittedResponseRestrictionTier,
    scope_node_ids: Vec<Uuid>,
    values: BTreeMap<String, SubmittedResponseValue>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredTombstone {
    response_id: Uuid,
    reason: String,
}

pub(crate) fn decode_stored_change(
    kind: &str,
    payload: Value,
) -> ApiResult<SubmittedResponseChange> {
    let change = match kind {
        "upsert" => {
            let stored: StoredUpsert = serde_json::from_value(payload).map_err(|_| restricted())?;
            SubmittedResponseChange::Upsert(Box::new(SubmittedResponseUpsert {
                response_id: stored.response_id,
                form_id: stored.form_id,
                form_version_id: stored.form_version_id,
                node_id: stored.node_id,
                node_name: stored.node_name,
                submitted_at: stored.submitted_at,
                created_at: stored.created_at,
                last_modified_at: stored.last_modified_at,
                last_modified_by_user_name: stored.last_modified_by_user_name,
                status: stored.status,
                restriction_tier: stored.restriction_tier,
                scope_node_ids: stored.scope_node_ids,
                values: stored.values,
                content_digest: String::new(),
            }))
        }
        "tombstone" => {
            let stored: StoredTombstone =
                serde_json::from_value(payload).map_err(|_| restricted())?;
            let reason = match stored.reason.as_str() {
                "deleted" => ResponseTombstoneReason::Deleted,
                "redacted" => ResponseTombstoneReason::Redacted,
                "status_excluded" => ResponseTombstoneReason::StatusExcluded,
                "scope_excluded" => ResponseTombstoneReason::ScopeExcluded,
                _ => return Err(restricted()),
            };
            SubmittedResponseChange::Tombstone {
                response_id: stored.response_id,
                reason,
                content_digest: String::new(),
            }
        }
        _ => return Err(restricted()),
    };
    change
        .with_recomputed_content_digest()
        .map_err(|_| restricted())
}

pub fn validate_stored_export_row(
    kind: &str,
    payload_text: &str,
    stored_content_digest: &str,
) -> ApiResult<SubmittedResponseChange> {
    let expected_storage_digest = format!("sha256:{:x}", Sha256::digest(payload_text.as_bytes()));
    if expected_storage_digest != stored_content_digest {
        return Err(restricted());
    }
    let payload = serde_json::from_str(payload_text).map_err(|_| restricted())?;
    let change = decode_stored_change(kind, payload)?;
    Ok(change)
}

async fn page(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<ResponseExportPageResponse>> {
    let inbound = authorize(
        &state,
        &headers,
        RESPONSE_EXPORT_PAGE_ACTION,
        RESPONSE_EXPORT_PAGE_PATH,
        &body,
    )
    .await?;
    let request: ResponseExportPageRequest =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    if request.action != ResponseExportAction::Page
        || request.page_size == 0
        || request.page_size > tessara_responses_contract::MAX_EXPORT_PAGE_SIZE
    {
        return Err(restricted());
    }
    let digest = validate_partition(&inbound, &request.partition)?;
    let epoch = provider_epoch(&state).await?;
    let upper = ProviderCursor::parse(&request.snapshot_upper_bound, &digest)?;
    let after = match &request.after_cursor {
        Some(cursor) => ProviderCursor::parse(cursor, &digest)?,
        None => ProviderCursor {
            epoch,
            partition_digest: digest.clone(),
            sequence: 0,
        },
    };
    if request.provider_epoch != epoch
        || upper.epoch != epoch
        || after.epoch != epoch
        || after.sequence > upper.sequence
    {
        return Err(restricted());
    }
    let rows = sqlx::query(
        "SELECT change_sequence,change_kind,payload::text AS payload_text,content_digest
         FROM response_export_changes
         WHERE form_version_id=ANY($1) AND node_id=ANY($2)
           AND change_sequence>$3 AND change_sequence<=$4
         ORDER BY change_sequence
         LIMIT $5",
    )
    .bind(&request.partition.form_version_ids)
    .bind(&request.partition.authorized_scope_node_ids)
    .bind(after.sequence)
    .bind(upper.sequence)
    .bind(i64::from(request.page_size) + 1)
    .fetch_all(&state.pool)
    .await?;
    let has_more = rows.len() > usize::from(request.page_size);
    let mut entries = Vec::with_capacity(rows.len().min(usize::from(request.page_size)));
    for row in rows.into_iter().take(usize::from(request.page_size)) {
        let sequence: i64 = row.try_get("change_sequence")?;
        let kind: String = row.try_get("change_kind")?;
        let payload_text: String = row.try_get("payload_text")?;
        let stored_content_digest: String = row.try_get("content_digest")?;
        let change = validate_stored_export_row(&kind, &payload_text, &stored_content_digest)?;
        entries.push(ResponseExportEntry {
            cursor: ProviderCursor {
                epoch,
                partition_digest: digest.clone(),
                sequence,
            }
            .encode()?,
            change,
        });
    }
    let complete = !has_more;
    let next_after_cursor = if complete {
        None
    } else {
        entries.last().map(|entry| entry.cursor.clone())
    };
    let page_digest =
        ResponseExportPageResponse::canonical_page_digest(&entries).map_err(|_| restricted())?;
    let response = ResponseExportPageResponse {
        schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
        provider_epoch: epoch,
        snapshot_upper_bound: request.snapshot_upper_bound,
        entries,
        next_after_cursor,
        complete,
        page_digest,
    };
    response.validate().map_err(|_| restricted())?;
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn cursor_is_bound_to_epoch_partition_and_canonical_sequence() {
        let epoch = Uuid::from_u128(1);
        let digest = format!("sha256:{}", "a".repeat(64));
        let cursor = ProviderCursor {
            epoch,
            partition_digest: digest.clone(),
            sequence: 42,
        }
        .encode()
        .unwrap();
        assert_eq!(
            ProviderCursor::parse(&cursor, &digest).unwrap().sequence,
            42
        );
        assert!(ProviderCursor::parse(&cursor, &format!("sha256:{}", "b".repeat(64))).is_err());
    }

    #[test]
    fn stored_upsert_decodes_only_the_complete_strict_envelope() {
        let payload = json!({
            "response_id": Uuid::from_u128(1),
            "form_id": Uuid::from_u128(2),
            "form_version_id": Uuid::from_u128(3),
            "node_id": Uuid::from_u128(4),
            "node_name": "North Division",
            "submitted_at": "2026-08-13T12:00:00Z",
            "created_at": "2026-08-13T11:00:00Z",
            "last_modified_at": "2026-08-13T12:05:00Z",
            "last_modified_by_user_name": "Export Owner",
            "status": "submitted",
            "restriction_tier": "restricted",
            "scope_node_ids": [Uuid::from_u128(4)],
            "values": {
                "answer": {
                    "field_id": Uuid::from_u128(5),
                    "value": "current",
                    "value_text": "current"
                }
            }
        });
        let change = decode_stored_change("upsert", payload.clone()).unwrap();
        let SubmittedResponseChange::Upsert(upsert) = &change else {
            panic!("stored upsert decoded as a tombstone")
        };
        assert_eq!(upsert.form_id, Uuid::from_u128(2));
        assert_eq!(upsert.node_name, "North Division");
        assert_eq!(
            upsert.restriction_tier,
            SubmittedResponseRestrictionTier::Restricted
        );
        assert_eq!(
            upsert.values.get("answer").unwrap().field_id,
            Uuid::from_u128(5)
        );
        assert!(change.validate_content_digest().is_ok());

        let mut missing_restriction = payload.clone();
        missing_restriction
            .as_object_mut()
            .unwrap()
            .remove("restriction_tier");
        assert!(decode_stored_change("upsert", missing_restriction).is_err());
        let mut unknown_field = payload;
        unknown_field["legacy_fallback"] = json!(true);
        assert!(decode_stored_change("upsert", unknown_field).is_err());
        assert!(decode_stored_change("unknown", json!({})).is_err());
    }

    #[test]
    fn stored_canonical_digest_tampering_is_rejected_before_page_output() {
        let payload = json!({
            "response_id": Uuid::from_u128(1),
            "form_id": Uuid::from_u128(2),
            "form_version_id": Uuid::from_u128(3),
            "node_id": Uuid::from_u128(4),
            "node_name": "North Division",
            "submitted_at": "2026-08-13T12:00:00Z",
            "created_at": "2026-08-13T11:00:00Z",
            "last_modified_at": "2026-08-13T12:05:00Z",
            "last_modified_by_user_name": "Export Owner",
            "status": "submitted",
            "restriction_tier": "public",
            "scope_node_ids": [Uuid::from_u128(4)],
            "values": {}
        });
        let payload_text = serde_json::to_string(&payload).unwrap();
        let storage_digest = format!("sha256:{:x}", Sha256::digest(payload_text.as_bytes()));
        assert!(validate_stored_export_row("upsert", &payload_text, &storage_digest).is_ok());
        assert!(
            validate_stored_export_row("upsert", &format!("{payload_text} "), &storage_digest,)
                .is_err()
        );

        let mut semantic_tamper = payload;
        semantic_tamper["unexpected"] = json!(true);
        let semantic_tamper_text = serde_json::to_string(&semantic_tamper).unwrap();
        let semantic_tamper_digest = format!(
            "sha256:{:x}",
            Sha256::digest(semantic_tamper_text.as_bytes())
        );
        assert!(
            validate_stored_export_row("upsert", &semantic_tamper_text, &semantic_tamper_digest,)
                .is_err(),
            "raw storage integrity must not make a non-contract envelope valid"
        );
    }

    #[test]
    fn stored_tombstones_accept_only_the_frozen_reason_vocabulary() {
        for (wire, expected) in [
            ("deleted", ResponseTombstoneReason::Deleted),
            ("redacted", ResponseTombstoneReason::Redacted),
            ("status_excluded", ResponseTombstoneReason::StatusExcluded),
            ("scope_excluded", ResponseTombstoneReason::ScopeExcluded),
        ] {
            let change = decode_stored_change(
                "tombstone",
                json!({"response_id": Uuid::from_u128(1), "reason": wire}),
            )
            .unwrap();
            let SubmittedResponseChange::Tombstone { reason, .. } = change else {
                panic!("stored tombstone decoded as an upsert")
            };
            assert_eq!(reason, expected);
        }
        assert!(
            decode_stored_change(
                "tombstone",
                json!({"response_id": Uuid::from_u128(1), "reason": "legacy_deleted"}),
            )
            .is_err()
        );
    }
}
