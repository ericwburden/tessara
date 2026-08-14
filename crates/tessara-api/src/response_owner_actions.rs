//! Authenticated Core-owned Response lifecycle mutations.
//!
//! Every action is a single owner transaction: row triggers are deferred while
//! the aggregate, workflow, values, and audit facts change, then one complete
//! export upsert or tombstone is appended and the signed replay receipt is
//! persisted before commit.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json, Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use tessara_core::FieldType;
use tessara_module_contract::ProtocolSignaturePurposeV1;
use tessara_responses_contract::{
    RESPONSE_OWNER_ACTION_IDEMPOTENCY_HEADER, RESPONSE_OWNER_ACTION_MEDIA_TYPE,
    RESPONSE_OWNER_ACTION_PATH, RESPONSE_OWNER_ACTION_SCHEMA_VERSION, ResponseOwnerActionReceipt,
    ResponseOwnerActionRequest, ResponseOwnerActionResponse, ResponseOwnerExportKind,
    ResponseOwnerExportReceipt, ResponseTombstoneReason, SubmittedResponseChange,
};
use tessara_submissions::{RequiredFieldStatus, ensure_required_values_present};
use tracing::error;
use uuid::Uuid;

use crate::{
    auth::{self, AccountContext, AuthenticatedRequest},
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy, response_export_provider, workflows,
};

const MAX_OWNER_ACTION_BODY_BYTES: usize = 256 * 1024;
const REQUEST_METHOD: &str = "POST";

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route(RESPONSE_OWNER_ACTION_PATH, post(owner_action))
}

async fn owner_action(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let correlation_id = crate::core_security::request_correlation_id_or_new(&headers);
    match execute_owner_action(&state, &request.account, &headers, &body).await {
        Ok(response) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, RESPONSE_OWNER_ACTION_MEDIA_TYPE),
                (header::CACHE_CONTROL, "no-store"),
            ],
            Json(response),
        )
            .into_response(),
        Err(failure) => failure.into_response(correlation_id),
    }
}

async fn execute_owner_action(
    state: &AppState,
    account: &AccountContext,
    headers: &HeaderMap,
    raw_body: &[u8],
) -> Result<ResponseOwnerActionResponse, OwnerActionFailure> {
    auth::ensure_capability(account, "submissions:manage")
        .map_err(|_| OwnerActionFailure::Forbidden)?;
    if raw_body.is_empty() || raw_body.len() > MAX_OWNER_ACTION_BODY_BYTES {
        return Err(OwnerActionFailure::MalformedRequest);
    }
    if !headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(is_owner_action_content_type)
    {
        return Err(OwnerActionFailure::MalformedRequest);
    }
    let idempotency_key = parse_idempotency_key(headers)?;
    let request: ResponseOwnerActionRequest =
        serde_json::from_slice(raw_body).map_err(|_| OwnerActionFailure::MalformedRequest)?;
    request
        .validate_logical_key()
        .map_err(|_| OwnerActionFailure::ValidationFailed)?;

    let action = request.action();
    let logical_key = request.logical_key().to_owned();
    let raw_body_digest = sha256_digest(raw_body);
    let idempotency_key_digest = sha256_digest(idempotency_key.as_bytes());

    let mut transaction = state.pool.begin().await.map_err(OwnerActionFailure::from)?;
    // A missing-row lookup cannot lock a future insert. Serialize every key
    // before replay inspection so concurrent first attempts cannot both mutate.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 82402))")
        .bind(idempotency_key)
        .execute(&mut *transaction)
        .await
        .map_err(OwnerActionFailure::from)?;

    if let Some(existing) = load_replay(&mut transaction, idempotency_key).await? {
        if existing.actor_account_id != account.account_id
            || existing.action != action.as_str()
            || existing.logical_key != logical_key
            || existing.request_method != REQUEST_METHOD
            || existing.request_path != RESPONSE_OWNER_ACTION_PATH
            || existing.raw_body_digest != raw_body_digest
            || existing.idempotency_key_digest != idempotency_key_digest
        {
            return Err(OwnerActionFailure::IdempotencyMismatch);
        }
        let signed_receipt = serde_json::from_value(existing.signed_receipt)
            .map_err(|error| OwnerActionFailure::Internal(error.into()))?;
        transaction
            .commit()
            .await
            .map_err(OwnerActionFailure::from)?;
        return Ok(ResponseOwnerActionResponse {
            schema_version: RESPONSE_OWNER_ACTION_SCHEMA_VERSION,
            replayed: true,
            signed_receipt,
        });
    }

    let result = mutate_response(account, &mut transaction, request).await?;
    let receipt = ResponseOwnerActionReceipt {
        schema_version: RESPONSE_OWNER_ACTION_SCHEMA_VERSION,
        logical_key: logical_key.clone(),
        action,
        actor_account_id: account.account_id,
        response_id: result.response_id,
        form_id: result.form_id,
        form_version_id: result.form_version_id,
        node_id: result.node_id,
        method: REQUEST_METHOD.into(),
        path: RESPONSE_OWNER_ACTION_PATH.into(),
        raw_body_digest: raw_body_digest.clone(),
        idempotency_key_digest: idempotency_key_digest.clone(),
        committed_at: result
            .recorded_at
            .to_rfc3339_opts(SecondsFormat::Micros, true),
        export: result.export,
    };
    let signed_receipt = crate::core_security::protocol_signer(
        ProtocolSignaturePurposeV1::ResponseOwnerActionReceipt,
    )
    .map_err(OwnerActionFailure::from)?
    .sign(receipt)
    .map_err(|error| OwnerActionFailure::Internal(error.into()))?;
    let signed_receipt_value = serde_json::to_value(&signed_receipt)
        .map_err(|error| OwnerActionFailure::Internal(error.into()))?;

    sqlx::query(
        "INSERT INTO response_owner_action_receipts(
             idempotency_key,actor_account_id,action,logical_key,request_method,request_path,
             raw_body_digest,idempotency_key_digest,signed_receipt)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",
    )
    .bind(idempotency_key)
    .bind(account.account_id)
    .bind(action.as_str())
    .bind(&logical_key)
    .bind(REQUEST_METHOD)
    .bind(RESPONSE_OWNER_ACTION_PATH)
    .bind(&raw_body_digest)
    .bind(&idempotency_key_digest)
    .bind(signed_receipt_value)
    .execute(&mut *transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    transaction
        .commit()
        .await
        .map_err(OwnerActionFailure::from)?;

    Ok(ResponseOwnerActionResponse {
        schema_version: RESPONSE_OWNER_ACTION_SCHEMA_VERSION,
        replayed: false,
        signed_receipt,
    })
}

fn is_owner_action_content_type(value: &str) -> bool {
    let Some((media_type, version)) = value.split_once(';') else {
        return false;
    };
    media_type == "application/vnd.tessara.responses.owner-action+json"
        && version.trim_ascii() == "version=1"
}

fn parse_idempotency_key(headers: &HeaderMap) -> Result<&str, OwnerActionFailure> {
    let value = headers
        .get(RESPONSE_OWNER_ACTION_IDEMPOTENCY_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or(OwnerActionFailure::IdempotencyRequired)?;
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || bytes.len() > 128
        || value.trim() != value
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':'))
        })
    {
        return Err(OwnerActionFailure::IdempotencyRequired);
    }
    Ok(value)
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

struct StoredReplay {
    actor_account_id: Uuid,
    action: String,
    logical_key: String,
    request_method: String,
    request_path: String,
    raw_body_digest: String,
    idempotency_key_digest: String,
    signed_receipt: Value,
}

async fn load_replay(
    transaction: &mut Transaction<'_, Postgres>,
    idempotency_key: &str,
) -> Result<Option<StoredReplay>, OwnerActionFailure> {
    let row = sqlx::query(
        "SELECT actor_account_id,action,logical_key,request_method,request_path,
                raw_body_digest,idempotency_key_digest,signed_receipt
           FROM response_owner_action_receipts
          WHERE idempotency_key=$1",
    )
    .bind(idempotency_key)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    row.map(|row| {
        Ok::<StoredReplay, sqlx::Error>(StoredReplay {
            actor_account_id: row.try_get("actor_account_id")?,
            action: row.try_get("action")?,
            logical_key: row.try_get("logical_key")?,
            request_method: row.try_get("request_method")?,
            request_path: row.try_get("request_path")?,
            raw_body_digest: row.try_get("raw_body_digest")?,
            idempotency_key_digest: row.try_get("idempotency_key_digest")?,
            signed_receipt: row.try_get("signed_receipt")?,
        })
    })
    .transpose()
    .map_err(OwnerActionFailure::from)
}

struct MutationResult {
    response_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
    recorded_at: DateTime<Utc>,
    export: ResponseOwnerExportReceipt,
}

async fn mutate_response(
    account: &AccountContext,
    transaction: &mut Transaction<'_, Postgres>,
    request: ResponseOwnerActionRequest,
) -> Result<MutationResult, OwnerActionFailure> {
    defer_export_capture_tx(transaction).await?;
    match request {
        ResponseOwnerActionRequest::Create {
            form_version_id,
            node_id,
            values,
            ..
        } => create_response(account, transaction, form_version_id, node_id, values).await,
        ResponseOwnerActionRequest::Correct {
            response_id,
            values,
            ..
        } => correct_response(account, transaction, response_id, values).await,
        ResponseOwnerActionRequest::StatusOut { response_id, .. } => {
            status_out_response(account, transaction, response_id).await
        }
        ResponseOwnerActionRequest::StatusIn { response_id, .. } => {
            status_in_response(account, transaction, response_id).await
        }
        ResponseOwnerActionRequest::Redact { response_id, .. } => {
            redact_response(account, transaction, response_id).await
        }
        ResponseOwnerActionRequest::Delete { response_id, .. } => {
            delete_response(account, transaction, response_id).await
        }
    }
}

async fn create_response(
    account: &AccountContext,
    transaction: &mut Transaction<'_, Postgres>,
    form_version_id: Uuid,
    node_id: Uuid,
    values: BTreeMap<String, Value>,
) -> Result<MutationResult, OwnerActionFailure> {
    authorize_node_tx(transaction, account.account_id, node_id).await?;
    let form_row = sqlx::query(
        "SELECT form_versions.form_id,form_versions.status::text AS status
           FROM form_versions
           JOIN nodes ON nodes.id=$2
           JOIN form_scope_nodes
             ON form_scope_nodes.form_id=form_versions.form_id
            AND form_scope_nodes.node_id=nodes.id
          WHERE form_versions.id=$1
          FOR SHARE OF form_versions,nodes,form_scope_nodes",
    )
    .bind(form_version_id)
    .bind(node_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?
    .ok_or(OwnerActionFailure::NotFoundOrForbidden)?;
    let form_id: Uuid = form_row
        .try_get("form_id")
        .map_err(OwnerActionFailure::from)?;
    let status: String = form_row
        .try_get("status")
        .map_err(OwnerActionFailure::from)?;
    if status != "published" {
        return Err(OwnerActionFailure::InvalidTransition);
    }
    let fields = load_response_fields_tx(transaction, form_version_id).await?;
    validate_complete_values(&fields, &values)?;

    let workflow_assignment_id = workflows::ensure_workflow_assignment_for_form_version_tx(
        transaction,
        form_version_id,
        node_id,
        account.account_id,
    )
    .await
    .map_err(OwnerActionFailure::from)?;
    let response_id: Uuid = sqlx::query_scalar(
        "INSERT INTO submissions(form_version_id,node_id,workflow_assignment_id,status)
         VALUES($1,$2,$3,'draft'::submission_status)
         RETURNING id",
    )
    .bind(form_version_id)
    .bind(node_id)
    .bind(workflow_assignment_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    replace_complete_values_tx(transaction, response_id, form_version_id, &fields, values).await?;
    workflows::ensure_submission_runtime_linkage_tx(
        transaction,
        response_id,
        workflow_assignment_id,
        account.account_id,
        true,
    )
    .await
    .map_err(OwnerActionFailure::from)?;
    sqlx::query(
        "UPDATE submissions
            SET status='submitted'::submission_status,submitted_at=now()
          WHERE id=$1",
    )
    .bind(response_id)
    .execute(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    audit_tx(
        transaction,
        response_id,
        "response_owner.create",
        account.account_id,
    )
    .await?;
    final_upsert(transaction, response_id, form_id, form_version_id, node_id).await
}

async fn correct_response(
    account: &AccountContext,
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
    values: BTreeMap<String, Value>,
) -> Result<MutationResult, OwnerActionFailure> {
    let target = load_target_tx(transaction, response_id).await?;
    authorize_node_tx(transaction, account.account_id, target.node_id).await?;
    target.require_visible()?;
    let fields = load_response_fields_tx(transaction, target.form_version_id).await?;
    validate_complete_values(&fields, &values)?;
    replace_complete_values_tx(
        transaction,
        response_id,
        target.form_version_id,
        &fields,
        values,
    )
    .await?;
    audit_tx(
        transaction,
        response_id,
        "response_owner.correct",
        account.account_id,
    )
    .await?;
    final_upsert(
        transaction,
        response_id,
        target.form_id,
        target.form_version_id,
        target.node_id,
    )
    .await
}

async fn status_out_response(
    account: &AccountContext,
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
) -> Result<MutationResult, OwnerActionFailure> {
    let target = load_target_tx(transaction, response_id).await?;
    authorize_node_tx(transaction, account.account_id, target.node_id).await?;
    target.require_visible()?;
    sqlx::query(
        "UPDATE submissions
            SET status='draft'::submission_status,submitted_at=NULL
          WHERE id=$1",
    )
    .bind(response_id)
    .execute(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    audit_tx(
        transaction,
        response_id,
        "response_owner.status_out",
        account.account_id,
    )
    .await?;
    final_tombstone(
        transaction,
        &target,
        ResponseTombstoneReason::StatusExcluded,
    )
    .await
}

async fn status_in_response(
    account: &AccountContext,
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
) -> Result<MutationResult, OwnerActionFailure> {
    let target = load_target_tx(transaction, response_id).await?;
    authorize_node_tx(transaction, account.account_id, target.node_id).await?;
    if target.status != "draft" || target.exclusion_reason.is_some() {
        return Err(OwnerActionFailure::InvalidTransition);
    }
    validate_stored_values_tx(transaction, response_id, target.form_version_id).await?;
    sqlx::query(
        "UPDATE submissions
            SET status='submitted'::submission_status,submitted_at=now()
          WHERE id=$1",
    )
    .bind(response_id)
    .execute(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    audit_tx(
        transaction,
        response_id,
        "response_owner.status_in",
        account.account_id,
    )
    .await?;
    final_upsert(
        transaction,
        response_id,
        target.form_id,
        target.form_version_id,
        target.node_id,
    )
    .await
}

async fn redact_response(
    account: &AccountContext,
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
) -> Result<MutationResult, OwnerActionFailure> {
    let target = load_target_tx(transaction, response_id).await?;
    authorize_node_tx(transaction, account.account_id, target.node_id).await?;
    target.require_visible()?;
    sqlx::query(
        "UPDATE submissions
            SET status='draft'::submission_status,submitted_at=NULL,
                response_export_exclusion_reason='redacted'
          WHERE id=$1",
    )
    .bind(response_id)
    .execute(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    sqlx::query("DELETE FROM submission_value_multi WHERE submission_id=$1")
        .bind(response_id)
        .execute(&mut **transaction)
        .await
        .map_err(OwnerActionFailure::from)?;
    sqlx::query("DELETE FROM submission_values WHERE submission_id=$1")
        .bind(response_id)
        .execute(&mut **transaction)
        .await
        .map_err(OwnerActionFailure::from)?;
    audit_tx(
        transaction,
        response_id,
        "response_owner.redact",
        account.account_id,
    )
    .await?;
    final_tombstone(transaction, &target, ResponseTombstoneReason::Redacted).await
}

async fn delete_response(
    account: &AccountContext,
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
) -> Result<MutationResult, OwnerActionFailure> {
    let target = load_target_tx(transaction, response_id).await?;
    authorize_node_tx(transaction, account.account_id, target.node_id).await?;
    target.require_visible()?;
    audit_tx(
        transaction,
        response_id,
        "response_owner.delete",
        account.account_id,
    )
    .await?;
    sqlx::query("DELETE FROM submissions WHERE id=$1")
        .bind(response_id)
        .execute(&mut **transaction)
        .await
        .map_err(OwnerActionFailure::from)?;
    if let Some(step_instance_id) = target.workflow_step_instance_id {
        sqlx::query("DELETE FROM workflow_step_instances WHERE id=$1")
            .bind(step_instance_id)
            .execute(&mut **transaction)
            .await
            .map_err(OwnerActionFailure::from)?;
    }
    if let Some(workflow_instance_id) = target.workflow_instance_id {
        sqlx::query(
            "DELETE FROM workflow_instances
              WHERE id=$1
                AND NOT EXISTS(SELECT 1 FROM workflow_step_instances
                                WHERE workflow_instance_id=$1)",
        )
        .bind(workflow_instance_id)
        .execute(&mut **transaction)
        .await
        .map_err(OwnerActionFailure::from)?;
    }
    final_tombstone(transaction, &target, ResponseTombstoneReason::Deleted).await
}

struct TargetResponse {
    response_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
    status: String,
    exclusion_reason: Option<String>,
    workflow_instance_id: Option<Uuid>,
    workflow_step_instance_id: Option<Uuid>,
}

impl TargetResponse {
    fn require_visible(&self) -> Result<(), OwnerActionFailure> {
        if self.status == "submitted" && self.exclusion_reason.is_none() {
            Ok(())
        } else {
            Err(OwnerActionFailure::InvalidTransition)
        }
    }
}

async fn load_target_tx(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
) -> Result<TargetResponse, OwnerActionFailure> {
    let row = sqlx::query(
        "SELECT submissions.id,form_versions.form_id,submissions.form_version_id,
                submissions.node_id,submissions.status::text AS status,
                submissions.response_export_exclusion_reason,
                submissions.workflow_instance_id,submissions.workflow_step_instance_id
           FROM submissions
           JOIN form_versions ON form_versions.id=submissions.form_version_id
          WHERE submissions.id=$1
          FOR UPDATE OF submissions
          FOR SHARE OF form_versions",
    )
    .bind(response_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?
    .ok_or(OwnerActionFailure::NotFoundOrForbidden)?;
    Ok(TargetResponse {
        response_id: row.try_get("id").map_err(OwnerActionFailure::from)?,
        form_id: row.try_get("form_id").map_err(OwnerActionFailure::from)?,
        form_version_id: row
            .try_get("form_version_id")
            .map_err(OwnerActionFailure::from)?,
        node_id: row.try_get("node_id").map_err(OwnerActionFailure::from)?,
        status: row.try_get("status").map_err(OwnerActionFailure::from)?,
        exclusion_reason: row
            .try_get("response_export_exclusion_reason")
            .map_err(OwnerActionFailure::from)?,
        workflow_instance_id: row
            .try_get("workflow_instance_id")
            .map_err(OwnerActionFailure::from)?,
        workflow_step_instance_id: row
            .try_get("workflow_step_instance_id")
            .map_err(OwnerActionFailure::from)?,
    })
}

async fn authorize_node_tx(
    transaction: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    node_id: Uuid,
) -> Result<(), OwnerActionFailure> {
    // Node reparenting changes the meaning of every descendant-scoped grant.
    // A SHARE table lock serializes this authorization read with all INSERT,
    // UPDATE, and DELETE hierarchy mutations until the owner action commits.
    // It avoids relying on a pre-transaction token projection or a revision
    // check that could race between evaluation and aggregate mutation.
    sqlx::query("LOCK TABLE nodes IN SHARE MODE")
        .execute(&mut **transaction)
        .await
        .map_err(OwnerActionFailure::from)?;
    sqlx::query(
        "SELECT role_assignments.id
           FROM role_assignments
           JOIN role_capabilities
             ON role_capabilities.role_id=role_assignments.role_id
           JOIN capabilities
             ON capabilities.id=role_capabilities.capability_id
          WHERE role_assignments.account_id=$1
            AND capabilities.key IN ('admin:all','submissions:manage')
          FOR SHARE OF role_assignments,role_capabilities,capabilities",
    )
    .bind(account_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    let allowed: bool = sqlx::query_scalar(
        "WITH RECURSIVE authorized_nodes(node_id) AS (
             SELECT role_assignments.node_id
               FROM role_assignments
               JOIN role_capabilities
                 ON role_capabilities.role_id=role_assignments.role_id
               JOIN capabilities
                 ON capabilities.id=role_capabilities.capability_id
              WHERE role_assignments.account_id=$1
                AND capabilities.key='submissions:manage'
                AND capabilities.scope_mode='scope_aware'
                AND role_assignments.node_id IS NOT NULL
             UNION
             SELECT nodes.id
               FROM nodes
               JOIN authorized_nodes ON nodes.parent_node_id=authorized_nodes.node_id
         )
         SELECT
             EXISTS(
                 SELECT 1
                   FROM role_assignments
                   JOIN role_capabilities
                     ON role_capabilities.role_id=role_assignments.role_id
                   JOIN capabilities
                     ON capabilities.id=role_capabilities.capability_id
                  WHERE role_assignments.account_id=$1
                    AND role_assignments.node_id IS NULL
                    AND capabilities.key IN ('admin:all','submissions:manage')
             )
             OR EXISTS(SELECT 1 FROM authorized_nodes WHERE node_id=$2)",
    )
    .bind(account_id)
    .bind(node_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    if allowed {
        Ok(())
    } else {
        Err(OwnerActionFailure::NotFoundOrForbidden)
    }
}

struct ResponseField {
    id: Uuid,
    key: String,
    field_type: FieldType,
    required: bool,
}

async fn load_response_fields_tx(
    transaction: &mut Transaction<'_, Postgres>,
    form_version_id: Uuid,
) -> ApiResult<Vec<ResponseField>> {
    let rows = sqlx::query(
        "SELECT field_id,key,field_type::text AS field_type,required
           FROM form_fields
          WHERE form_version_id=$1 AND field_type <> 'static_text'::field_type
          ORDER BY position,field_id
          FOR SHARE OF form_fields",
    )
    .bind(form_version_id)
    .fetch_all(&mut **transaction)
    .await?;
    if rows.is_empty() {
        return Err(ApiError::BadRequest(
            "the Response FormVersion has no response fields".into(),
        ));
    }
    rows.into_iter()
        .map(|row| {
            let field_type: String = row.try_get("field_type")?;
            Ok(ResponseField {
                id: row.try_get("field_id")?,
                key: row.try_get("key")?,
                field_type: hierarchy::parse_field_type(&field_type)?,
                required: row.try_get("required")?,
            })
        })
        .collect::<ApiResult<Vec<_>>>()
}

fn validate_complete_values(
    fields: &[ResponseField],
    values: &BTreeMap<String, Value>,
) -> Result<(), OwnerActionFailure> {
    let expected = fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<BTreeSet<_>>();
    let actual = values.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if expected != actual {
        return Err(OwnerActionFailure::ValidationFailed);
    }
    for field in fields {
        let value = &values[&field.key];
        if value.is_null() {
            if field.required {
                return Err(OwnerActionFailure::ValidationFailed);
            }
            continue;
        }
        hierarchy::validate_field_value(field.field_type, value)
            .map_err(|_| OwnerActionFailure::ValidationFailed)?;
        if field.required && !value_counts_as_present(value) {
            return Err(OwnerActionFailure::ValidationFailed);
        }
    }
    Ok(())
}

async fn validate_stored_values_tx(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
    form_version_id: Uuid,
) -> Result<(), OwnerActionFailure> {
    let fields = load_response_fields_tx(transaction, form_version_id).await?;
    let rows = sqlx::query("SELECT field_id,value FROM submission_values WHERE submission_id=$1")
        .bind(response_id)
        .fetch_all(&mut **transaction)
        .await
        .map_err(OwnerActionFailure::from)?;
    let values = rows
        .into_iter()
        .map(|row| Ok((row.try_get::<Uuid, _>("field_id")?, row.try_get("value")?)))
        .collect::<Result<BTreeMap<_, _>, sqlx::Error>>()
        .map_err(OwnerActionFailure::from)?;
    for field in fields {
        let value = values.get(&field.id);
        if field.required && !value.is_some_and(value_counts_as_present) {
            return Err(OwnerActionFailure::ValidationFailed);
        }
        if let Some(value) = value
            && !value.is_null()
        {
            hierarchy::validate_field_value(field.field_type, value)
                .map_err(|_| OwnerActionFailure::ValidationFailed)?;
        }
    }
    Ok(())
}

fn value_counts_as_present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => values.iter().any(value_counts_as_present),
        _ => true,
    }
}

async fn replace_complete_values_tx(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
    form_version_id: Uuid,
    fields: &[ResponseField],
    values: BTreeMap<String, Value>,
) -> Result<(), OwnerActionFailure> {
    sqlx::query("DELETE FROM submission_value_multi WHERE submission_id=$1")
        .bind(response_id)
        .execute(&mut **transaction)
        .await
        .map_err(OwnerActionFailure::from)?;
    sqlx::query("DELETE FROM submission_values WHERE submission_id=$1")
        .bind(response_id)
        .execute(&mut **transaction)
        .await
        .map_err(OwnerActionFailure::from)?;
    for field in fields {
        let value = &values[&field.key];
        sqlx::query(
            "INSERT INTO submission_values(submission_id,form_version_id,field_id,value)
             VALUES($1,$2,$3,$4)",
        )
        .bind(response_id)
        .bind(form_version_id)
        .bind(field.id)
        .bind(value)
        .execute(&mut **transaction)
        .await
        .map_err(OwnerActionFailure::from)?;
        if let Some(items) = value.as_array() {
            for item in items.iter().filter_map(Value::as_str) {
                sqlx::query(
                    "INSERT INTO submission_value_multi(
                         submission_id,form_version_id,field_id,value)
                     VALUES($1,$2,$3,$4)",
                )
                .bind(response_id)
                .bind(form_version_id)
                .bind(field.id)
                .bind(item)
                .execute(&mut **transaction)
                .await
                .map_err(OwnerActionFailure::from)?;
            }
        }
    }
    Ok(())
}

async fn audit_tx(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
    event_type: &str,
    actor_account_id: Uuid,
) -> Result<(), OwnerActionFailure> {
    sqlx::query(
        "INSERT INTO submission_audit_events(submission_id,event_type,account_id)
         VALUES($1,$2,$3)",
    )
    .bind(response_id)
    .bind(event_type)
    .bind(actor_account_id)
    .execute(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    Ok(())
}

pub(crate) async fn defer_export_capture_tx(
    transaction: &mut Transaction<'_, Postgres>,
) -> ApiResult<()> {
    sqlx::query("SELECT set_config('tessara.response_export_capture','deferred',true)")
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

pub(crate) async fn append_final_upsert_tx(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
) -> ApiResult<i64> {
    sqlx::query_scalar("SELECT append_current_response_export_upsert_owned($1)")
        .bind(response_id)
        .fetch_optional(&mut **transaction)
        .await?
        .flatten()
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("final Response upsert was not emitted")))
}

async fn final_upsert(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
) -> Result<MutationResult, OwnerActionFailure> {
    let sequence = append_final_upsert_tx(transaction, response_id)
        .await
        .map_err(OwnerActionFailure::from)?;
    export_result(
        transaction,
        sequence,
        response_id,
        form_id,
        form_version_id,
        node_id,
    )
    .await
}

pub(crate) async fn append_final_tombstone_tx(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
    reason: ResponseTombstoneReason,
) -> ApiResult<i64> {
    sqlx::query_scalar("SELECT append_response_export_change_owned($1,$2,$3,'tombstone',$4)")
        .bind(response_id)
        .bind(form_version_id)
        .bind(node_id)
        .bind(json!({"response_id": response_id, "reason": reason.as_str()}))
        .fetch_one(&mut **transaction)
        .await
        .map_err(Into::into)
}

async fn final_tombstone(
    transaction: &mut Transaction<'_, Postgres>,
    target: &TargetResponse,
    reason: ResponseTombstoneReason,
) -> Result<MutationResult, OwnerActionFailure> {
    let sequence = append_final_tombstone_tx(
        transaction,
        target.response_id,
        target.form_version_id,
        target.node_id,
        reason,
    )
    .await
    .map_err(OwnerActionFailure::from)?;
    export_result(
        transaction,
        sequence,
        target.response_id,
        target.form_id,
        target.form_version_id,
        target.node_id,
    )
    .await
}

async fn export_result(
    transaction: &mut Transaction<'_, Postgres>,
    sequence: i64,
    response_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
) -> Result<MutationResult, OwnerActionFailure> {
    let row = sqlx::query(
        "SELECT change_kind,payload,recorded_at
           FROM response_export_changes
          WHERE change_sequence=$1 AND response_id=$2",
    )
    .bind(sequence)
    .bind(response_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(OwnerActionFailure::from)?;
    let kind: String = row
        .try_get("change_kind")
        .map_err(OwnerActionFailure::from)?;
    let payload: Value = row.try_get("payload").map_err(OwnerActionFailure::from)?;
    let recorded_at: DateTime<Utc> = row
        .try_get("recorded_at")
        .map_err(OwnerActionFailure::from)?;
    let change = response_export_provider::decode_stored_change(&kind, payload)
        .map_err(OwnerActionFailure::from)?;
    let (change_kind, tombstone_reason, content_digest) = match change {
        SubmittedResponseChange::Upsert(upsert) => {
            (ResponseOwnerExportKind::Upsert, None, upsert.content_digest)
        }
        SubmittedResponseChange::Tombstone {
            reason,
            content_digest,
            ..
        } => (
            ResponseOwnerExportKind::Tombstone,
            Some(reason),
            content_digest,
        ),
    };
    let change_sequence =
        u64::try_from(sequence).map_err(|error| OwnerActionFailure::Internal(error.into()))?;
    Ok(MutationResult {
        response_id,
        form_id,
        form_version_id,
        node_id,
        recorded_at,
        export: ResponseOwnerExportReceipt {
            change_sequence,
            change_kind,
            tombstone_reason,
            content_digest,
        },
    })
}

/// Existing public submit path: keep the product access check in the
/// submission service, then lock and validate the draft, advance workflow,
/// audit, and append one final export envelope in this owner transaction.
pub(crate) async fn submit_existing_response(
    pool: &sqlx::PgPool,
    actor_account_id: Uuid,
    response_id: Uuid,
) -> ApiResult<()> {
    let mut transaction = pool.begin().await?;
    defer_export_capture_tx(&mut transaction).await?;
    let response = sqlx::query(
        "SELECT form_version_id,status::text AS status,response_export_exclusion_reason
           FROM submissions WHERE id=$1 FOR UPDATE",
    )
    .bind(response_id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("submission {response_id}")))?;
    let form_version_id: Uuid = response.try_get("form_version_id")?;
    let status: String = response.try_get("status")?;
    let exclusion_reason: Option<String> = response.try_get("response_export_exclusion_reason")?;
    if status != "draft" || exclusion_reason.is_some() {
        return Err(ApiError::BadRequest(
            "submitted records are immutable in the initial workflow".into(),
        ));
    }
    validate_public_submission_tx(&mut transaction, response_id, form_version_id).await?;
    sqlx::query(
        "UPDATE submissions
            SET status='submitted'::submission_status,submitted_at=now()
          WHERE id=$1",
    )
    .bind(response_id)
    .execute(&mut *transaction)
    .await?;
    workflows::complete_workflow_step_and_advance_tx(&mut transaction, response_id).await?;
    sqlx::query(
        "INSERT INTO submission_audit_events(submission_id,event_type,account_id)
         VALUES($1,'submit',$2)",
    )
    .bind(response_id)
    .bind(actor_account_id)
    .execute(&mut *transaction)
    .await?;
    append_final_upsert_tx(&mut transaction, response_id).await?;
    transaction.commit().await?;
    Ok(())
}

async fn validate_public_submission_tx(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
    form_version_id: Uuid,
) -> ApiResult<()> {
    let fields = load_response_fields_tx(transaction, form_version_id).await?;
    let rows = sqlx::query("SELECT field_id,value FROM submission_values WHERE submission_id=$1")
        .bind(response_id)
        .fetch_all(&mut **transaction)
        .await?;
    let values = rows
        .into_iter()
        .map(|row| Ok((row.try_get::<Uuid, _>("field_id")?, row.try_get("value")?)))
        .collect::<Result<BTreeMap<Uuid, Value>, sqlx::Error>>()?;
    for field in &fields {
        if let Some(value) = values.get(&field.id) {
            hierarchy::validate_field_value(field.field_type, value)?;
        }
    }
    ensure_required_values_present(fields.iter().map(|field| RequiredFieldStatus {
        key: &field.key,
        required: field.required,
        has_value: values.get(&field.id).is_some_and(value_counts_as_present),
    }))
    .map_err(|error| ApiError::BadRequest(error.to_string()))
}

#[derive(Debug)]
enum OwnerActionFailure {
    MalformedRequest,
    IdempotencyRequired,
    IdempotencyMismatch,
    Forbidden,
    NotFoundOrForbidden,
    InvalidTransition,
    ValidationFailed,
    Internal(anyhow::Error),
}

impl From<sqlx::Error> for OwnerActionFailure {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(error.into())
    }
}

impl From<ApiError> for OwnerActionFailure {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::BadRequest(_) => Self::ValidationFailed,
            ApiError::Conflict(_) => Self::InvalidTransition,
            ApiError::Forbidden(_) => Self::Forbidden,
            ApiError::NotFound(_) => Self::NotFoundOrForbidden,
            ApiError::Unauthorized
            | ApiError::InvalidCredentials
            | ApiError::SessionExpired
            | ApiError::SessionRevoked => Self::Forbidden,
            ApiError::MixedCapabilityScopeModes
            | ApiError::GlobalCapabilityRequiresGlobalRoleAssignment => Self::ValidationFailed,
            ApiError::ServiceUnavailable(message) => Self::Internal(anyhow::anyhow!(message)),
            ApiError::Database(error) => Self::Internal(error.into()),
            ApiError::Internal(error) => Self::Internal(error),
        }
    }
}

#[derive(Serialize)]
struct OwnerActionErrorEnvelope {
    schema_version: u16,
    error: OwnerActionErrorBody,
    correlation_id: Uuid,
}

#[derive(Serialize)]
struct OwnerActionErrorBody {
    code: &'static str,
    message: &'static str,
}

impl OwnerActionFailure {
    fn into_response(self, correlation_id: Uuid) -> Response {
        let (status, code, message) = match self {
            Self::MalformedRequest => (
                StatusCode::BAD_REQUEST,
                "response_owner.malformed_request",
                "The Response owner action request is malformed.",
            ),
            Self::IdempotencyRequired => (
                StatusCode::BAD_REQUEST,
                "response_owner.idempotency_required",
                "A valid Response owner idempotency key is required.",
            ),
            Self::IdempotencyMismatch => (
                StatusCode::CONFLICT,
                "response_owner.idempotency_mismatch",
                "The Response owner idempotency key is bound to another request.",
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "response_owner.forbidden",
                "The Response owner action is not authorized.",
            ),
            Self::NotFoundOrForbidden => (
                StatusCode::NOT_FOUND,
                "response_owner.not_found_or_forbidden",
                "The Response owner target is unavailable.",
            ),
            Self::InvalidTransition => (
                StatusCode::CONFLICT,
                "response_owner.invalid_transition",
                "The Response owner lifecycle transition is invalid.",
            ),
            Self::ValidationFailed => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "response_owner.validation_failed",
                "The Response owner action values are invalid.",
            ),
            Self::Internal(error) => {
                error!(error = ?error, %correlation_id, "Response owner action failed internally");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "response_owner.internal",
                    "The Response owner action could not be completed.",
                )
            }
        };
        (
            status,
            [(header::CACHE_CONTROL, "no-store")],
            Json(OwnerActionErrorEnvelope {
                schema_version: RESPONSE_OWNER_ACTION_SCHEMA_VERSION,
                error: OwnerActionErrorBody { code, message },
                correlation_id,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::http::{HeaderMap, HeaderValue};
    use serde_json::json;

    use super::*;

    #[test]
    fn owner_action_media_type_accepts_only_insignificant_parameter_whitespace() {
        assert!(is_owner_action_content_type(
            RESPONSE_OWNER_ACTION_MEDIA_TYPE
        ));
        assert!(is_owner_action_content_type(
            "application/vnd.tessara.responses.owner-action+json; version=1"
        ));
        assert!(!is_owner_action_content_type("application/json"));
        assert!(!is_owner_action_content_type(
            "application/vnd.tessara.responses.owner-action+json;version=1;charset=utf-8"
        ));
        assert!(!is_owner_action_content_type(
            "application/vnd.tessara.responses.owner-action+json;version=2"
        ));
    }

    #[test]
    fn idempotency_keys_are_strict_and_digest_exact_bytes() {
        let mut headers = HeaderMap::new();
        headers.insert(
            RESPONSE_OWNER_ACTION_IDEMPOTENCY_HEADER,
            HeaderValue::from_static("fixture.response.corrected.v1"),
        );
        assert_eq!(
            parse_idempotency_key(&headers).unwrap(),
            "fixture.response.corrected.v1"
        );
        assert_ne!(
            sha256_digest(br#"{"a":1}"#),
            sha256_digest(br#"{ "a": 1 }"#)
        );

        headers.insert(
            RESPONSE_OWNER_ACTION_IDEMPOTENCY_HEADER,
            HeaderValue::from_static(" invalid "),
        );
        assert!(matches!(
            parse_idempotency_key(&headers),
            Err(OwnerActionFailure::IdempotencyRequired)
        ));
    }

    #[test]
    fn complete_value_validation_rejects_omission_unknown_and_required_null() {
        let fields = vec![
            ResponseField {
                id: Uuid::from_u128(1),
                key: "answer".into(),
                field_type: FieldType::Text,
                required: true,
            },
            ResponseField {
                id: Uuid::from_u128(2),
                key: "optional".into(),
                field_type: FieldType::Number,
                required: false,
            },
        ];
        assert!(
            validate_complete_values(
                &fields,
                &BTreeMap::from([
                    ("answer".into(), json!("ok")),
                    ("optional".into(), Value::Null)
                ])
            )
            .is_ok()
        );
        assert!(matches!(
            validate_complete_values(&fields, &BTreeMap::from([("answer".into(), json!("ok"))])),
            Err(OwnerActionFailure::ValidationFailed)
        ));
        assert!(matches!(
            validate_complete_values(
                &fields,
                &BTreeMap::from([
                    ("answer".into(), Value::Null),
                    ("optional".into(), json!(2))
                ])
            ),
            Err(OwnerActionFailure::ValidationFailed)
        ));
    }

    #[tokio::test]
    async fn every_failure_has_one_distinct_sanitized_code() {
        let cases = [
            (
                OwnerActionFailure::MalformedRequest,
                StatusCode::BAD_REQUEST,
                "response_owner.malformed_request",
            ),
            (
                OwnerActionFailure::IdempotencyRequired,
                StatusCode::BAD_REQUEST,
                "response_owner.idempotency_required",
            ),
            (
                OwnerActionFailure::IdempotencyMismatch,
                StatusCode::CONFLICT,
                "response_owner.idempotency_mismatch",
            ),
            (
                OwnerActionFailure::Forbidden,
                StatusCode::FORBIDDEN,
                "response_owner.forbidden",
            ),
            (
                OwnerActionFailure::NotFoundOrForbidden,
                StatusCode::NOT_FOUND,
                "response_owner.not_found_or_forbidden",
            ),
            (
                OwnerActionFailure::InvalidTransition,
                StatusCode::CONFLICT,
                "response_owner.invalid_transition",
            ),
            (
                OwnerActionFailure::ValidationFailed,
                StatusCode::UNPROCESSABLE_ENTITY,
                "response_owner.validation_failed",
            ),
            (
                OwnerActionFailure::Internal(anyhow::anyhow!("private fixture diagnostic")),
                StatusCode::INTERNAL_SERVER_ERROR,
                "response_owner.internal",
            ),
        ];
        let mut observed_codes = BTreeSet::new();
        for (failure, expected_status, expected_code) in cases {
            let response = failure.into_response(Uuid::nil());
            assert_eq!(response.status(), expected_status);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(body["error"]["code"], expected_code);
            assert!(!body.to_string().contains("private fixture diagnostic"));
            assert!(observed_codes.insert(expected_code));
        }
        assert_eq!(observed_codes.len(), 8);
    }
}
