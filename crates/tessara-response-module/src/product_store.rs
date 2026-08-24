use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Row, Transaction};
use tessara_forms_contract::FormVersionSchemaResponse;
use tessara_responses_contract::{
    RESPONSE_EVENT_SCHEMA_VERSION, ResponseAuditEventSummary, ResponseDetail, ResponseEventKind,
    ResponseFormField, ResponseFormSection, ResponseFormSnapshot, ResponseMutationResult,
    ResponseReference, ResponseRuntimeDetail, ResponseRuntimeStepHistory, ResponseSummary,
    ResponseValueDetail, ResponseWorkflowEvent, SubmittedResponseChange,
    SubmittedResponseRestrictionTier, SubmittedResponseUpsert, SubmittedResponseValue,
};
use tessara_workflows_contract::WorkflowResponseStartContext;
use uuid::Uuid;

use crate::ResponseOwnerError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResponseAccess {
    pub installation_id: Uuid,
    pub actor_account_id: Uuid,
    pub delegated_account_ids: BTreeSet<Uuid>,
    pub managed_node_ids: BTreeSet<Uuid>,
    pub manage_all: bool,
}

impl ResponseAccess {
    fn permits(&self, node_id: Uuid, assignee_account_id: Uuid) -> bool {
        self.manage_all
            || self.managed_node_ids.contains(&node_id)
            || self.actor_account_id == assignee_account_id
            || self.delegated_account_ids.contains(&assignee_account_id)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResponseListFilter {
    pub status: Option<String>,
    pub form_id: Option<Uuid>,
    pub node_id: Option<Uuid>,
    pub search: Option<String>,
    pub assignee_account_id: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaveResponseCommand {
    pub response_id: Uuid,
    pub expected_revision: u64,
    pub values: BTreeMap<String, Value>,
    pub actor_account_id: Uuid,
    pub idempotency_key_digest: String,
    pub request_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResponseMutationCommand {
    pub response_id: Uuid,
    pub expected_revision: u64,
    pub actor_account_id: Uuid,
    pub idempotency_key_digest: String,
    pub request_digest: String,
}

impl super::ResponseOwnerRepository {
    pub async fn list(
        &self,
        access: &ResponseAccess,
        filter: &ResponseListFilter,
    ) -> Result<Vec<ResponseSummary>, ResponseOwnerError> {
        validate_access(access)?;
        if filter
            .status
            .as_deref()
            .is_some_and(|status| !matches!(status, "draft" | "submitted"))
        {
            return Err(ResponseOwnerError::InvalidCommand);
        }
        let rows = sqlx::query(
            "SELECT id,form_id,form_version_id,node_id,assignee_account_id,status::text AS status,revision,form_snapshot,workflow_context,created_at,updated_at,submitted_at,(SELECT COUNT(*) FROM response_values WHERE response_values.response_id=responses.id) AS value_count FROM responses WHERE status <> 'deleted' ORDER BY created_at,id",
        )
        .fetch_all(&self.pool)
        .await?;
        let search = filter
            .search
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_lowercase);
        rows.into_iter()
            .filter_map(|row| {
                let node_id: Uuid = row.get("node_id");
                let assignee: Uuid = row.get("assignee_account_id");
                access.permits(node_id, assignee).then_some(row)
            })
            .map(|row| summary_from_row(row, filter, search.as_deref()))
            .filter_map(Result::transpose)
            .collect()
    }

    pub async fn detail(
        &self,
        access: &ResponseAccess,
        response_id: Uuid,
    ) -> Result<Option<ResponseDetail>, ResponseOwnerError> {
        validate_access(access)?;
        let Some(row) = sqlx::query(
            "SELECT id,form_id,form_version_id,node_id,assignee_account_id,status::text AS status,revision,form_snapshot,workflow_context,created_at,updated_at,submitted_at FROM responses WHERE id=$1 AND status <> 'deleted'",
        )
        .bind(response_id)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };
        if !access.permits(row.get("node_id"), row.get("assignee_account_id")) {
            return Ok(None);
        }
        let form_schema: FormVersionSchemaResponse =
            serde_json::from_value(row.get("form_snapshot"))
                .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
        form_schema
            .validate_for(form_schema.form_version_id)
            .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
        let workflow: WorkflowResponseStartContext =
            serde_json::from_value(row.get("workflow_context"))
                .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
        workflow
            .validate_for(workflow.workflow_assignment_id)
            .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
        let form = product_form_snapshot(&form_schema);
        let values = load_value_details(&self.pool, response_id, &form).await?;
        let audit_events = load_audit_events(
            &self.pool,
            response_id,
            workflow.assignee_account_id,
            &workflow.assignee_display_name,
        )
        .await?;
        Ok(Some(ResponseDetail {
            id: response_id,
            form_id: row.get("form_id"),
            form_version_id: row.get("form_version_id"),
            form_name: form.form_name.clone(),
            version_label: form
                .version_label
                .clone()
                .unwrap_or_else(|| "Published".into()),
            node_id: row.get("node_id"),
            node_name: workflow.node_name.clone(),
            status: row.get("status"),
            revision: u64::try_from(row.get::<i64, _>("revision"))
                .map_err(|_| ResponseOwnerError::CorruptSnapshot)?,
            created_at: timestamp(row.get("created_at")),
            submitted_at: row
                .get::<Option<DateTime<Utc>>, _>("submitted_at")
                .map(timestamp),
            values,
            audit_events,
            runtime: Some(runtime_detail(&workflow)),
            form,
        }))
    }

    pub async fn save(
        &self,
        access: &ResponseAccess,
        command: &SaveResponseCommand,
    ) -> Result<super::IdempotentCommit<ResponseMutationResult>, ResponseOwnerError> {
        validate_mutation_envelope(
            access,
            command.actor_account_id,
            command.response_id,
            command.expected_revision,
            &command.idempotency_key_digest,
            &command.request_digest,
        )?;
        if let Some(replay) = self
            .existing_mutation_receipt(
                command.actor_account_id,
                "responses.save",
                &command.idempotency_key_digest,
                &command.request_digest,
            )
            .await?
        {
            return Ok(super::IdempotentCommit::Replayed(replay));
        }
        let mut transaction = self.pool.begin().await?;
        let locked = lock_response(&mut transaction, command.response_id).await?;
        ensure_mutation_access(access, &locked)?;
        ensure_draft_revision(&locked, command.expected_revision)?;
        let schema: FormVersionSchemaResponse =
            serde_json::from_value(locked.form_snapshot.clone())
                .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
        schema
            .validate_for(schema.form_version_id)
            .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
        let fields = schema
            .fields
            .iter()
            .map(|field| (field.key.as_str(), field))
            .collect::<BTreeMap<_, _>>();
        for (key, value) in &command.values {
            let field = fields
                .get(key.as_str())
                .ok_or_else(|| ResponseOwnerError::InvalidField(key.clone()))?;
            validate_field_value(&field.field_type, &field.options, value)
                .map_err(|_| ResponseOwnerError::InvalidField(key.clone()))?;
            sqlx::query("INSERT INTO response_values(response_id,field_id,field_key,value,value_text) VALUES($1,$2,$3,$4,$5) ON CONFLICT(response_id,field_id) DO UPDATE SET field_key=EXCLUDED.field_key,value=EXCLUDED.value,value_text=EXCLUDED.value_text")
                .bind(command.response_id)
                .bind(field.field_id)
                .bind(&field.key)
                .bind(value)
                .bind(value_text(value))
                .execute(&mut *transaction)
                .await?;
        }
        let revision = command.expected_revision + 1;
        let occurred_at: DateTime<Utc> = sqlx::query_scalar(
            "UPDATE responses SET revision=$2,updated_at=now() WHERE id=$1 RETURNING updated_at",
        )
        .bind(command.response_id)
        .bind(i64::try_from(revision).map_err(|_| ResponseOwnerError::InvalidCommand)?)
        .fetch_one(&mut *transaction)
        .await?;
        append_audit(
            &mut transaction,
            command.response_id,
            revision,
            "draft_saved",
            command.actor_account_id,
            occurred_at,
        )
        .await?;
        append_workflow_event(
            &mut transaction,
            &locked,
            revision,
            ResponseEventKind::DraftSaved,
            occurred_at,
        )
        .await?;
        let result = ResponseMutationResult {
            id: command.response_id,
            revision,
            status: "draft".into(),
        };
        store_mutation_receipt(
            &mut transaction,
            command.actor_account_id,
            "responses.save",
            &command.idempotency_key_digest,
            &command.request_digest,
            &result,
        )
        .await?;
        transaction.commit().await?;
        Ok(super::IdempotentCommit::Applied(result))
    }

    pub async fn submit(
        &self,
        access: &ResponseAccess,
        command: &ResponseMutationCommand,
    ) -> Result<super::IdempotentCommit<ResponseMutationResult>, ResponseOwnerError> {
        self.finish_mutation(access, command, FinishKind::Submit)
            .await
    }

    pub async fn delete(
        &self,
        access: &ResponseAccess,
        command: &ResponseMutationCommand,
    ) -> Result<super::IdempotentCommit<ResponseMutationResult>, ResponseOwnerError> {
        self.finish_mutation(access, command, FinishKind::Delete)
            .await
    }

    async fn finish_mutation(
        &self,
        access: &ResponseAccess,
        command: &ResponseMutationCommand,
        kind: FinishKind,
    ) -> Result<super::IdempotentCommit<ResponseMutationResult>, ResponseOwnerError> {
        validate_mutation_envelope(
            access,
            command.actor_account_id,
            command.response_id,
            command.expected_revision,
            &command.idempotency_key_digest,
            &command.request_digest,
        )?;
        let action = kind.action();
        if let Some(replay) = self
            .existing_mutation_receipt(
                command.actor_account_id,
                action,
                &command.idempotency_key_digest,
                &command.request_digest,
            )
            .await?
        {
            return Ok(super::IdempotentCommit::Replayed(replay));
        }
        let mut transaction = self.pool.begin().await?;
        let locked = lock_response(&mut transaction, command.response_id).await?;
        ensure_mutation_access(access, &locked)?;
        ensure_draft_revision(&locked, command.expected_revision)?;
        let schema: FormVersionSchemaResponse =
            serde_json::from_value(locked.form_snapshot.clone())
                .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
        schema
            .validate_for(schema.form_version_id)
            .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
        if kind == FinishKind::Submit {
            ensure_required_values(&mut transaction, command.response_id, &schema).await?;
        }
        let revision = command.expected_revision + 1;
        let occurred_at: DateTime<Utc> = match kind {
            FinishKind::Submit => sqlx::query_scalar("UPDATE responses SET status='submitted',revision=$2,updated_at=now(),submitted_at=now() WHERE id=$1 RETURNING updated_at")
                .bind(command.response_id)
                .bind(i64::try_from(revision).map_err(|_| ResponseOwnerError::InvalidCommand)?)
                .fetch_one(&mut *transaction)
                .await?,
            FinishKind::Delete => sqlx::query_scalar("UPDATE responses SET status='deleted',revision=$2,updated_at=now(),deleted_at=now() WHERE id=$1 RETURNING updated_at")
                .bind(command.response_id)
                .bind(i64::try_from(revision).map_err(|_| ResponseOwnerError::InvalidCommand)?)
                .fetch_one(&mut *transaction)
                .await?,
        };
        append_audit(
            &mut transaction,
            command.response_id,
            revision,
            kind.event_name(),
            command.actor_account_id,
            occurred_at,
        )
        .await?;
        append_workflow_event(
            &mut transaction,
            &locked,
            revision,
            kind.event_kind(),
            occurred_at,
        )
        .await?;
        if kind == FinishKind::Submit {
            append_export(
                &mut transaction,
                &locked,
                &schema,
                command.actor_account_id,
                occurred_at,
            )
            .await?;
        }
        let result = ResponseMutationResult {
            id: command.response_id,
            revision,
            status: kind.status().into(),
        };
        store_mutation_receipt(
            &mut transaction,
            command.actor_account_id,
            action,
            &command.idempotency_key_digest,
            &command.request_digest,
            &result,
        )
        .await?;
        transaction.commit().await?;
        Ok(super::IdempotentCommit::Applied(result))
    }

    async fn existing_mutation_receipt(
        &self,
        actor_account_id: Uuid,
        action: &str,
        idempotency_key_digest: &str,
        request_digest: &str,
    ) -> Result<Option<ResponseMutationResult>, ResponseOwnerError> {
        let row = sqlx::query_as::<_, (String, Value)>(
            "SELECT request_digest,response_body FROM response_idempotency_receipts WHERE actor_account_id=$1 AND action=$2 AND idempotency_key_digest=$3",
        )
        .bind(actor_account_id)
        .bind(action)
        .bind(idempotency_key_digest)
        .fetch_optional(&self.pool)
        .await?;
        let Some((stored_digest, body)) = row else {
            return Ok(None);
        };
        if stored_digest != request_digest {
            return Err(ResponseOwnerError::IdempotencyConflict);
        }
        serde_json::from_value(body)
            .map(Some)
            .map_err(|_| ResponseOwnerError::CorruptReceipt)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FinishKind {
    Submit,
    Delete,
}

impl FinishKind {
    const fn action(self) -> &'static str {
        match self {
            Self::Submit => "responses.submit",
            Self::Delete => "responses.delete",
        }
    }
    const fn event_name(self) -> &'static str {
        match self {
            Self::Submit => "submitted",
            Self::Delete => "deleted",
        }
    }
    const fn status(self) -> &'static str {
        match self {
            Self::Submit => "submitted",
            Self::Delete => "deleted",
        }
    }
    const fn event_kind(self) -> ResponseEventKind {
        match self {
            Self::Submit => ResponseEventKind::Submitted,
            Self::Delete => ResponseEventKind::Deleted,
        }
    }
}

struct LockedResponse {
    id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
    assignee_account_id: Uuid,
    workflow_assignment_id: Uuid,
    workflow_instance_id: Uuid,
    workflow_step_instance_id: Uuid,
    status: String,
    revision: u64,
    form_snapshot: Value,
    workflow_context: Value,
    created_at: DateTime<Utc>,
}

async fn lock_response(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
) -> Result<LockedResponse, ResponseOwnerError> {
    let row = sqlx::query("SELECT id,form_id,form_version_id,node_id,assignee_account_id,workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,status::text AS status,revision,form_snapshot,workflow_context,created_at FROM responses WHERE id=$1 FOR UPDATE")
        .bind(response_id)
        .fetch_optional(&mut **transaction)
        .await?
        .ok_or(ResponseOwnerError::NotFound)?;
    Ok(LockedResponse {
        id: row.get("id"),
        form_id: row.get("form_id"),
        form_version_id: row.get("form_version_id"),
        node_id: row.get("node_id"),
        assignee_account_id: row.get("assignee_account_id"),
        workflow_assignment_id: row.get("workflow_assignment_id"),
        workflow_instance_id: row.get("workflow_instance_id"),
        workflow_step_instance_id: row.get("workflow_step_instance_id"),
        status: row.get("status"),
        revision: u64::try_from(row.get::<i64, _>("revision"))
            .map_err(|_| ResponseOwnerError::CorruptSnapshot)?,
        form_snapshot: row.get("form_snapshot"),
        workflow_context: row.get("workflow_context"),
        created_at: row.get("created_at"),
    })
}

fn validate_access(access: &ResponseAccess) -> Result<(), ResponseOwnerError> {
    if access.installation_id.is_nil() || access.actor_account_id.is_nil() {
        Err(ResponseOwnerError::InvalidCommand)
    } else {
        Ok(())
    }
}

fn validate_mutation_envelope(
    access: &ResponseAccess,
    actor_account_id: Uuid,
    response_id: Uuid,
    expected_revision: u64,
    idempotency_key_digest: &str,
    request_digest: &str,
) -> Result<(), ResponseOwnerError> {
    validate_access(access)?;
    if access.actor_account_id != actor_account_id
        || response_id.is_nil()
        || expected_revision == 0
        || !crate::owner::is_digest(idempotency_key_digest)
        || !crate::owner::is_digest(request_digest)
    {
        return Err(ResponseOwnerError::InvalidCommand);
    }
    Ok(())
}

fn ensure_mutation_access(
    access: &ResponseAccess,
    response: &LockedResponse,
) -> Result<(), ResponseOwnerError> {
    if access.permits(response.node_id, response.assignee_account_id) {
        Ok(())
    } else {
        Err(ResponseOwnerError::NotFound)
    }
}

fn ensure_draft_revision(
    response: &LockedResponse,
    expected_revision: u64,
) -> Result<(), ResponseOwnerError> {
    if response.status != "draft" {
        return Err(ResponseOwnerError::Immutable);
    }
    if response.revision != expected_revision {
        return Err(ResponseOwnerError::RevisionConflict {
            expected: expected_revision,
            actual: response.revision,
        });
    }
    Ok(())
}

fn product_form_snapshot(schema: &FormVersionSchemaResponse) -> ResponseFormSnapshot {
    let mut sections = schema
        .sections
        .iter()
        .map(|section| ResponseFormSection {
            id: section.section_id,
            title: section.label.clone(),
            description: section.description.clone(),
            position: section.position,
            fields: Vec::new(),
        })
        .collect::<Vec<_>>();
    for field in &schema.fields {
        let product = ResponseFormField {
            field_id: field.field_id,
            key: field.key.clone(),
            label: field.label.clone(),
            field_type: field.field_type.clone(),
            required: field.required,
            options: field.options.clone(),
            position: field.position,
            grid_row: field.grid_row,
            grid_column: field.grid_column,
            grid_width: field.grid_width,
            grid_height: field.grid_height,
        };
        if let Some(section) = field
            .section_id
            .and_then(|id| sections.iter_mut().find(|section| section.id == id))
        {
            section.fields.push(product);
        }
    }
    for section in &mut sections {
        section.fields.sort_by_key(|field| field.position);
    }
    sections.sort_by_key(|section| section.position);
    ResponseFormSnapshot {
        form_version_id: schema.form_version_id,
        form_id: schema.form_id,
        form_name: schema.form_name.clone(),
        version_label: schema.version_label.clone(),
        status: "published".into(),
        sections,
    }
}

fn runtime_detail(workflow: &WorkflowResponseStartContext) -> ResponseRuntimeDetail {
    ResponseRuntimeDetail {
        workflow_name: workflow.workflow_name.clone(),
        current_step_title: workflow.workflow_step_title.clone(),
        current_step_position: workflow.workflow_step_position,
        step_count: workflow.workflow_step_count,
        next_step_title: workflow.next_workflow_step_title.clone(),
        history: workflow
            .history
            .iter()
            .map(|step| ResponseRuntimeStepHistory {
                title: step.title.clone(),
                form_name: step.form_name.clone(),
                status: step.status.clone(),
                position: step.position,
                completed_at: step.completed_at.clone(),
            })
            .collect(),
    }
}

fn summary_from_row(
    row: sqlx::postgres::PgRow,
    filter: &ResponseListFilter,
    search: Option<&str>,
) -> Result<Option<ResponseSummary>, ResponseOwnerError> {
    let form_schema: FormVersionSchemaResponse =
        serde_json::from_value(row.get("form_snapshot"))
            .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
    let workflow: WorkflowResponseStartContext =
        serde_json::from_value(row.get("workflow_context"))
            .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
    let status: String = row.get("status");
    let form_id: Uuid = row.get("form_id");
    let node_id: Uuid = row.get("node_id");
    let assignee_account_id: Uuid = row.get("assignee_account_id");
    if filter
        .status
        .as_deref()
        .is_some_and(|value| value != status)
        || filter.form_id.is_some_and(|value| value != form_id)
        || filter.node_id.is_some_and(|value| value != node_id)
        || filter
            .assignee_account_id
            .is_some_and(|value| value != assignee_account_id)
    {
        return Ok(None);
    }
    if search.is_some_and(|needle| {
        ![
            form_schema.form_name.as_str(),
            workflow.workflow_name.as_str(),
            workflow.workflow_description.as_str(),
            workflow.node_name.as_str(),
            workflow.assignee_display_name.as_str(),
        ]
        .iter()
        .any(|value| value.to_lowercase().contains(needle))
    }) {
        return Ok(None);
    }
    let completed = workflow
        .history
        .iter()
        .filter(|step| step.status == "completed")
        .count();
    Ok(Some(ResponseSummary {
        id: row.get("id"),
        form_id,
        form_version_id: row.get("form_version_id"),
        form_name: form_schema.form_name,
        workflow_name: Some(workflow.workflow_name),
        workflow_description: Some(workflow.workflow_description),
        workflow_step_position: Some(workflow.workflow_step_position),
        workflow_step_count: Some(workflow.workflow_step_count),
        workflow_steps_completed: Some(i64::try_from(completed).unwrap_or(i64::MAX)),
        current_workflow_step_title: Some(workflow.workflow_step_title),
        next_workflow_step_title: workflow.next_workflow_step_title,
        next_workflow_step_form_name: workflow.next_workflow_step_form_name,
        assigned_to_display_name: Some(workflow.assignee_display_name),
        version_label: form_schema
            .version_label
            .unwrap_or_else(|| "Published".into()),
        node_id,
        node_name: workflow.node_name,
        status,
        value_count: row.get("value_count"),
        created_at: timestamp(row.get("created_at")),
        last_modified_at: timestamp(row.get("updated_at")),
        submitted_at: row
            .get::<Option<DateTime<Utc>>, _>("submitted_at")
            .map(timestamp),
    }))
}

async fn load_value_details(
    pool: &PgPool,
    response_id: Uuid,
    form: &ResponseFormSnapshot,
) -> Result<Vec<ResponseValueDetail>, ResponseOwnerError> {
    let values = sqlx::query("SELECT field_id,value FROM response_values WHERE response_id=$1")
        .bind(response_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| (row.get::<Uuid, _>("field_id"), row.get::<Value, _>("value")))
        .collect::<BTreeMap<_, _>>();
    Ok(form
        .sections
        .iter()
        .flat_map(|section| &section.fields)
        .map(|field| ResponseValueDetail {
            field_id: field.field_id,
            key: field.key.clone(),
            label: field.label.clone(),
            field_type: field.field_type.clone(),
            required: field.required,
            value: values.get(&field.field_id).cloned(),
        })
        .collect())
}

async fn load_audit_events(
    pool: &PgPool,
    response_id: Uuid,
    assignee_id: Uuid,
    assignee_name: &str,
) -> Result<Vec<ResponseAuditEventSummary>, ResponseOwnerError> {
    Ok(sqlx::query("SELECT action,actor_account_id,occurred_at FROM response_audit_events WHERE response_id=$1 ORDER BY occurred_at,id")
        .bind(response_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| {
            let actor_id: Uuid = row.get("actor_account_id");
            ResponseAuditEventSummary {
                event_type: row.get("action"),
                actor_display_name: (actor_id == assignee_id).then(|| assignee_name.to_owned()),
                created_at: timestamp(row.get("occurred_at")),
            }
        })
        .collect())
}

async fn append_audit(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
    revision: u64,
    action: &str,
    actor_account_id: Uuid,
    occurred_at: DateTime<Utc>,
) -> Result<(), ResponseOwnerError> {
    sqlx::query("INSERT INTO response_audit_events(response_id,response_revision,action,actor_account_id,occurred_at) VALUES($1,$2,$3,$4,$5)")
        .bind(response_id)
        .bind(i64::try_from(revision).map_err(|_| ResponseOwnerError::InvalidCommand)?)
        .bind(action)
        .bind(actor_account_id)
        .bind(occurred_at)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn append_workflow_event(
    transaction: &mut Transaction<'_, Postgres>,
    response: &LockedResponse,
    revision: u64,
    kind: ResponseEventKind,
    occurred_at: DateTime<Utc>,
) -> Result<(), ResponseOwnerError> {
    let (installation_id, module_instance_id): (Uuid, Uuid) = sqlx::query_as(
        "SELECT installation_id,module_instance_id FROM response_module_security_state WHERE singleton=true",
    )
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or(ResponseOwnerError::SecurityStateUnavailable)?;
    let sequence: i64 = sqlx::query_scalar(
        "SELECT nextval(pg_get_serial_sequence('response_workflow_events','sequence'))",
    )
    .fetch_one(&mut **transaction)
    .await?;
    let reference = ResponseReference::from_parts(installation_id, module_instance_id, response.id)
        .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
    let event = ResponseWorkflowEvent {
        schema_version: RESPONSE_EVENT_SCHEMA_VERSION,
        sequence: u64::try_from(sequence).map_err(|_| ResponseOwnerError::CorruptSnapshot)?,
        event_id: Uuid::new_v4(),
        kind,
        response: reference,
        response_revision: revision,
        workflow_assignment_id: response.workflow_assignment_id,
        workflow_instance_id: response.workflow_instance_id,
        workflow_step_instance_id: response.workflow_step_instance_id,
        occurred_at: timestamp(occurred_at),
        content_digest: String::new(),
    }
    .with_recomputed_digest()
    .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
    sqlx::query("INSERT INTO response_workflow_events(sequence,event_id,response_id,response_revision,event_kind,payload,content_digest,occurred_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(sequence)
        .bind(event.event_id)
        .bind(response.id)
        .bind(i64::try_from(revision).map_err(|_| ResponseOwnerError::InvalidCommand)?)
        .bind(kind_name(kind))
        .bind(serde_json::to_value(&event).map_err(|_| ResponseOwnerError::CorruptSnapshot)?)
        .bind(&event.content_digest)
        .bind(occurred_at)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn ensure_required_values(
    transaction: &mut Transaction<'_, Postgres>,
    response_id: Uuid,
    schema: &FormVersionSchemaResponse,
) -> Result<(), ResponseOwnerError> {
    let present = sqlx::query("SELECT field_id,value FROM response_values WHERE response_id=$1")
        .bind(response_id)
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .map(|row| (row.get::<Uuid, _>("field_id"), row.get::<Value, _>("value")))
        .collect::<BTreeMap<_, _>>();
    for field in &schema.fields {
        if field.required
            && !present
                .get(&field.field_id)
                .is_some_and(nonempty_response_value)
        {
            return Err(ResponseOwnerError::MissingRequiredField(field.key.clone()));
        }
    }
    Ok(())
}

async fn append_export(
    transaction: &mut Transaction<'_, Postgres>,
    response: &LockedResponse,
    schema: &FormVersionSchemaResponse,
    actor_account_id: Uuid,
    occurred_at: DateTime<Utc>,
) -> Result<(), ResponseOwnerError> {
    let workflow: WorkflowResponseStartContext =
        serde_json::from_value(response.workflow_context.clone())
            .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
    let values = sqlx::query("SELECT field_id,field_key,value,value_text FROM response_values WHERE response_id=$1 ORDER BY field_key")
        .bind(response.id)
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .map(|row| {
            (
                row.get::<String, _>("field_key"),
                SubmittedResponseValue {
                    field_id: row.get("field_id"),
                    value: row.get("value"),
                    value_text: row.get("value_text"),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let occurred = timestamp(occurred_at);
    let change = SubmittedResponseChange::Upsert(Box::new(SubmittedResponseUpsert {
        response_id: response.id,
        form_id: response.form_id,
        form_version_id: response.form_version_id,
        node_id: response.node_id,
        node_name: workflow.node_name,
        submitted_at: occurred.clone(),
        created_at: timestamp(response.created_at),
        last_modified_at: occurred,
        last_modified_by_user_name: (actor_account_id == workflow.assignee_account_id)
            .then_some(workflow.assignee_display_name),
        status: "submitted".into(),
        restriction_tier: SubmittedResponseRestrictionTier::Public,
        scope_node_ids: vec![response.node_id],
        values,
        content_digest: String::new(),
    }))
    .with_recomputed_content_digest()
    .map_err(|_| ResponseOwnerError::CorruptSnapshot)?;
    let digest = match &change {
        SubmittedResponseChange::Upsert(upsert) => upsert.content_digest.clone(),
        SubmittedResponseChange::Tombstone { .. } => unreachable!(),
    };
    sqlx::query("INSERT INTO response_export_changes(response_id,form_version_id,node_id,change_kind,payload,content_digest,occurred_at) VALUES($1,$2,$3,'upsert',$4,$5,$6)")
        .bind(response.id)
        .bind(response.form_version_id)
        .bind(response.node_id)
        .bind(serde_json::to_value(change).map_err(|_| ResponseOwnerError::CorruptSnapshot)?)
        .bind(digest)
        .bind(occurred_at)
        .execute(&mut **transaction)
        .await?;
    debug_assert_eq!(schema.form_version_id, response.form_version_id);
    Ok(())
}

async fn store_mutation_receipt(
    transaction: &mut Transaction<'_, Postgres>,
    actor_account_id: Uuid,
    action: &str,
    idempotency_key_digest: &str,
    request_digest: &str,
    result: &ResponseMutationResult,
) -> Result<(), ResponseOwnerError> {
    sqlx::query("INSERT INTO response_idempotency_receipts(actor_account_id,action,idempotency_key_digest,request_digest,response_status,response_body) VALUES($1,$2,$3,$4,200,$5)")
        .bind(actor_account_id)
        .bind(action)
        .bind(idempotency_key_digest)
        .bind(request_digest)
        .bind(serde_json::to_value(result).map_err(|_| ResponseOwnerError::InvalidCommand)?)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

pub(crate) fn validate_field_value(
    field_type: &str,
    options: &[Value],
    value: &Value,
) -> Result<(), ()> {
    let valid_type = match field_type {
        "static_text" | "text" | "date" | "single_choice" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "multi_choice" => value
            .as_array()
            .is_some_and(|items| items.iter().all(Value::is_string)),
        _ => false,
    };
    if !valid_type {
        return Err(());
    }
    if field_type == "single_choice" && !options.is_empty() && !options.contains(value) {
        return Err(());
    }
    if field_type == "multi_choice"
        && !options.is_empty()
        && value
            .as_array()
            .is_some_and(|items| items.iter().any(|item| !options.contains(item)))
    {
        return Err(());
    }
    Ok(())
}

fn nonempty_response_value(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        _ => true,
    }
}

pub(crate) fn value_text(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Array(values) => Some(
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", "),
        ),
        Value::Null | Value::Object(_) => None,
    }
}

fn kind_name(kind: ResponseEventKind) -> &'static str {
    match kind {
        ResponseEventKind::Started => "started",
        ResponseEventKind::DraftSaved => "draft_saved",
        ResponseEventKind::Submitted => "submitted",
        ResponseEventKind::Deleted => "deleted",
    }
}

fn timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn field_validation_is_typed_and_option_bound() {
        assert!(validate_field_value("number", &[], &json!(3)).is_ok());
        assert!(validate_field_value("number", &[], &json!("3")).is_err());
        assert!(validate_field_value("single_choice", &[json!("a")], &json!("a")).is_ok());
        assert!(validate_field_value("single_choice", &[json!("a")], &json!("b")).is_err());
        assert!(validate_field_value("multi_choice", &[json!("a")], &json!(["a"])).is_ok());
        assert!(validate_field_value("multi_choice", &[json!("a")], &json!(["b"])).is_err());
    }

    #[test]
    fn access_is_exact_to_owner_delegation_or_managed_scope() {
        let node = Uuid::from_u128(3);
        let assignee = Uuid::from_u128(4);
        let mut access = ResponseAccess {
            installation_id: Uuid::from_u128(1),
            actor_account_id: Uuid::from_u128(2),
            delegated_account_ids: BTreeSet::new(),
            managed_node_ids: BTreeSet::new(),
            manage_all: false,
        };
        assert!(!access.permits(node, assignee));
        access.delegated_account_ids.insert(assignee);
        assert!(access.permits(node, assignee));
        access.delegated_account_ids.clear();
        access.managed_node_ids.insert(node);
        assert!(access.permits(node, assignee));
    }
}
