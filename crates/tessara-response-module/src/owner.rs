use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tessara_responses_contract::{
    RESPONSE_CONTRACT_SCHEMA_VERSION, RESPONSE_EVENT_SCHEMA_VERSION, ResponseEventKind,
    ResponseLifecycleSnapshot, ResponseLifecycleState, ResponseReference, ResponseWorkflowEvent,
};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseValueInput {
    pub field_id: Uuid,
    pub field_key: String,
    pub value: Value,
    pub value_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateResponseCommand {
    pub response: ResponseReference,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub node_id: Uuid,
    pub workflow_assignment_id: Uuid,
    pub workflow_version_id: Uuid,
    pub workflow_step_id: Uuid,
    pub workflow_instance_id: Uuid,
    pub workflow_step_instance_id: Uuid,
    pub assignee_account_id: Uuid,
    pub started_by_account_id: Uuid,
    pub delegation_basis: Option<String>,
    pub form_snapshot: Value,
    pub form_snapshot_digest: String,
    pub workflow_context: Value,
    pub workflow_context_digest: String,
    pub values: Vec<ResponseValueInput>,
    pub idempotency_key_digest: String,
    pub request_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdempotentCommit<T> {
    Applied(T),
    Replayed(T),
}

#[derive(Clone)]
pub struct ResponseOwnerRepository {
    pub(crate) pool: PgPool,
}

impl ResponseOwnerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        command: &CreateResponseCommand,
    ) -> Result<IdempotentCommit<ResponseLifecycleSnapshot>, ResponseOwnerError> {
        validate_create(command)?;
        if let Some(replay) = self.existing_create_receipt(command).await? {
            return Ok(IdempotentCommit::Replayed(replay));
        }

        let result = self.create_once(command).await;
        match result {
            Ok(snapshot) => Ok(IdempotentCommit::Applied(snapshot)),
            Err(error @ ResponseOwnerError::Persistence(_)) => {
                if let Some(replay) = self.existing_create_receipt(command).await? {
                    Ok(IdempotentCommit::Replayed(replay))
                } else {
                    Err(error)
                }
            }
            Err(error) => Err(error),
        }
    }

    async fn create_once(
        &self,
        command: &CreateResponseCommand,
    ) -> Result<ResponseLifecycleSnapshot, ResponseOwnerError> {
        let response_id = command.response.response_id();
        let mut transaction = self.pool.begin().await?;
        let created_at: DateTime<Utc> = sqlx::query_scalar(
            "INSERT INTO responses(id,form_id,form_version_id,node_id,workflow_assignment_id,workflow_version_id,workflow_step_id,workflow_instance_id,workflow_step_instance_id,assignee_account_id,started_by_account_id,delegation_basis,form_snapshot,form_snapshot_digest,workflow_context,workflow_context_digest) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16) RETURNING created_at",
        )
        .bind(response_id)
        .bind(command.form_id)
        .bind(command.form_version_id)
        .bind(command.node_id)
        .bind(command.workflow_assignment_id)
        .bind(command.workflow_version_id)
        .bind(command.workflow_step_id)
        .bind(command.workflow_instance_id)
        .bind(command.workflow_step_instance_id)
        .bind(command.assignee_account_id)
        .bind(command.started_by_account_id)
        .bind(command.delegation_basis.as_deref())
        .bind(&command.form_snapshot)
        .bind(&command.form_snapshot_digest)
        .bind(&command.workflow_context)
        .bind(&command.workflow_context_digest)
        .fetch_one(&mut *transaction)
        .await?;

        for value in &command.values {
            sqlx::query("INSERT INTO response_values(response_id,field_id,field_key,value,value_text) VALUES($1,$2,$3,$4,$5)")
                .bind(response_id)
                .bind(value.field_id)
                .bind(&value.field_key)
                .bind(&value.value)
                .bind(value.value_text.as_deref())
                .execute(&mut *transaction)
                .await?;
        }

        sqlx::query("INSERT INTO response_audit_events(response_id,response_revision,action,actor_account_id,occurred_at) VALUES($1,1,'started',$2,$3)")
            .bind(response_id)
            .bind(command.started_by_account_id)
            .bind(created_at)
            .execute(&mut *transaction)
            .await?;

        let sequence: i64 = sqlx::query_scalar(
            "SELECT nextval(pg_get_serial_sequence('response_workflow_events','sequence'))",
        )
        .fetch_one(&mut *transaction)
        .await?;
        let event = ResponseWorkflowEvent {
            schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
            sequence: sequence as u64,
            event_id: Uuid::new_v4(),
            kind: ResponseEventKind::Started,
            response: command.response.clone(),
            response_revision: 1,
            workflow_assignment_id: command.workflow_assignment_id,
            workflow_instance_id: command.workflow_instance_id,
            workflow_step_instance_id: command.workflow_step_instance_id,
            occurred_at: created_at.to_rfc3339(),
            content_digest: String::new(),
        }
        .with_recomputed_digest()
        .map_err(|_| ResponseOwnerError::InvalidCommand)?;
        sqlx::query("INSERT INTO response_workflow_events(sequence,event_id,response_id,response_revision,event_kind,payload,content_digest,occurred_at) VALUES($1,$2,$3,1,'started',$4,$5,$6)")
            .bind(sequence)
            .bind(event.event_id)
            .bind(response_id)
            .bind(serde_json::to_value(&event).map_err(|_| ResponseOwnerError::InvalidCommand)?)
            .bind(&event.content_digest)
            .bind(created_at)
            .execute(&mut *transaction)
            .await?;

        let snapshot = lifecycle_snapshot(command, created_at);
        let response_body =
            serde_json::to_value(&snapshot).map_err(|_| ResponseOwnerError::InvalidCommand)?;
        sqlx::query("INSERT INTO response_idempotency_receipts(actor_account_id,action,idempotency_key_digest,request_digest,response_status,response_body) VALUES($1,'responses.start',$2,$3,201,$4)")
            .bind(command.started_by_account_id)
            .bind(&command.idempotency_key_digest)
            .bind(&command.request_digest)
            .bind(response_body)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(snapshot)
    }

    async fn existing_create_receipt(
        &self,
        command: &CreateResponseCommand,
    ) -> Result<Option<ResponseLifecycleSnapshot>, ResponseOwnerError> {
        let row = sqlx::query_as::<_, (String, Value)>(
            "SELECT request_digest,response_body FROM response_idempotency_receipts WHERE actor_account_id=$1 AND action='responses.start' AND idempotency_key_digest=$2",
        )
        .bind(command.started_by_account_id)
        .bind(&command.idempotency_key_digest)
        .fetch_optional(&self.pool)
        .await?;
        let Some((request_digest, body)) = row else {
            return Ok(None);
        };
        if request_digest != command.request_digest {
            return Err(ResponseOwnerError::IdempotencyConflict);
        }
        let snapshot =
            serde_json::from_value(body).map_err(|_| ResponseOwnerError::CorruptReceipt)?;
        Ok(Some(snapshot))
    }
}

fn lifecycle_snapshot(
    command: &CreateResponseCommand,
    created_at: DateTime<Utc>,
) -> ResponseLifecycleSnapshot {
    let timestamp = created_at.to_rfc3339();
    ResponseLifecycleSnapshot {
        schema_version: RESPONSE_CONTRACT_SCHEMA_VERSION,
        response: command.response.clone(),
        state: ResponseLifecycleState::Draft,
        revision: 1,
        form_version_id: command.form_version_id,
        workflow_assignment_id: command.workflow_assignment_id,
        workflow_instance_id: command.workflow_instance_id,
        workflow_step_instance_id: command.workflow_step_instance_id,
        node_id: command.node_id,
        created_at: timestamp.clone(),
        modified_at: timestamp,
        submitted_at: None,
        form_snapshot_digest: command.form_snapshot_digest.clone(),
        workflow_context_digest: command.workflow_context_digest.clone(),
    }
}

fn validate_create(command: &CreateResponseCommand) -> Result<(), ResponseOwnerError> {
    if [
        command.response.response_id(),
        command.form_id,
        command.form_version_id,
        command.node_id,
        command.workflow_assignment_id,
        command.workflow_version_id,
        command.workflow_step_id,
        command.workflow_instance_id,
        command.workflow_step_instance_id,
        command.assignee_account_id,
        command.started_by_account_id,
    ]
    .iter()
    .any(Uuid::is_nil)
        || !is_digest(&command.form_snapshot_digest)
        || !is_digest(&command.workflow_context_digest)
        || !is_digest(&command.idempotency_key_digest)
        || !is_digest(&command.request_digest)
        || canonical_digest(&command.form_snapshot)? != command.form_snapshot_digest
        || canonical_digest(&command.workflow_context)? != command.workflow_context_digest
    {
        return Err(ResponseOwnerError::InvalidCommand);
    }
    let mut ids = HashSet::new();
    let mut keys = HashSet::new();
    for value in &command.values {
        if value.field_id.is_nil()
            || value.field_key.trim().is_empty()
            || !ids.insert(value.field_id)
            || !keys.insert(value.field_key.as_str())
        {
            return Err(ResponseOwnerError::InvalidCommand);
        }
    }
    Ok(())
}

pub(crate) fn is_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

pub fn canonical_digest<T: Serialize + ?Sized>(value: &T) -> Result<String, ResponseOwnerError> {
    let bytes = serde_jcs::to_vec(value).map_err(|_| ResponseOwnerError::InvalidCommand)?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[derive(Debug, thiserror::Error)]
pub enum ResponseOwnerError {
    #[error("Response owner command is invalid")]
    InvalidCommand,
    #[error("idempotency key was already used for a different request")]
    IdempotencyConflict,
    #[error("stored idempotency receipt is invalid")]
    CorruptReceipt,
    #[error("stored Response source snapshot is invalid")]
    CorruptSnapshot,
    #[error("Response was not found")]
    NotFound,
    #[error("submitted or deleted Responses are immutable")]
    Immutable,
    #[error("Response revision conflict: expected {expected}, actual {actual}")]
    RevisionConflict { expected: u64, actual: u64 },
    #[error("unknown or invalid Response field '{0}'")]
    InvalidField(String),
    #[error("required field '{0}' is missing")]
    MissingRequiredField(String),
    #[error("Response security state is unavailable")]
    SecurityStateUnavailable,
    #[error("Response persistence is unavailable")]
    Persistence(#[from] sqlx::Error),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn create_validation_rejects_duplicate_fields_and_noncanonical_digests() {
        let installation_id = Uuid::from_u128(1);
        let module_instance_id = Uuid::from_u128(2);
        let field_id = Uuid::from_u128(20);
        let form_snapshot = json!({"schema_version": 1});
        let workflow_context = json!({"schema_version": 1});
        let mut command = CreateResponseCommand {
            response: ResponseReference::from_parts(
                installation_id,
                module_instance_id,
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
            delegation_basis: None,
            form_snapshot_digest: canonical_digest(&form_snapshot).unwrap(),
            form_snapshot,
            workflow_context_digest: canonical_digest(&workflow_context).unwrap(),
            workflow_context,
            values: vec![
                ResponseValueInput {
                    field_id,
                    field_key: "name".into(),
                    value: json!("Ada"),
                    value_text: Some("Ada".into()),
                },
                ResponseValueInput {
                    field_id,
                    field_key: "duplicate".into(),
                    value: Value::Null,
                    value_text: None,
                },
            ],
            idempotency_key_digest: format!("sha256:{}", "c".repeat(64)),
            request_digest: format!("sha256:{}", "d".repeat(64)),
        };
        assert!(matches!(
            validate_create(&command),
            Err(ResponseOwnerError::InvalidCommand)
        ));
        command.values.pop();
        assert!(validate_create(&command).is_ok());
        command.request_digest = "sha256:ABC".into();
        assert!(matches!(
            validate_create(&command),
            Err(ResponseOwnerError::InvalidCommand)
        ));
    }

    #[test]
    fn canonical_digest_is_order_independent_for_json_object_keys() {
        assert_eq!(
            canonical_digest(&json!({"b": 2, "a": 1})).unwrap(),
            canonical_digest(&json!({"a": 1, "b": 2})).unwrap()
        );
    }
}
