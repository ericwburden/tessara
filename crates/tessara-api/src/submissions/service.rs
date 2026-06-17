use std::collections::HashMap;

use serde_json::Value;
use sqlx::PgPool;
use tessara_submissions::{RequiredFieldStatus, ensure_required_values_present};
use uuid::Uuid;

use crate::{
    auth,
    error::{ApiError, ApiResult},
    hierarchy::validate_field_value,
    workflows,
};

use super::dto::{
    AssignmentResponseStartOption, AssignmentResponseStartOptions, ListSubmissionsQuery,
    SubmissionDetail, SubmissionSummary,
};
use super::repo::{self, SubmissionListFilters};

pub struct SubmissionAccess {
    pub form_version_id: Uuid,
    pub status: String,
}

pub fn parse_submission_status_filter(status: Option<String>) -> ApiResult<Option<String>> {
    match status.as_deref() {
        None | Some("") => Ok(None),
        Some("draft" | "submitted") => Ok(status),
        Some(value) => Err(ApiError::BadRequest(format!(
            "unsupported submission status filter '{value}'"
        ))),
    }
}

pub async fn list_response_start_options(
    pool: &PgPool,
    account: &auth::AccountContext,
    query: ListSubmissionsQuery,
) -> ApiResult<AssignmentResponseStartOptions> {
    let delegate_account_id = if account.has_capability("workflows:manage") {
        query.delegate_account_id.unwrap_or(account.account_id)
    } else {
        auth::ensure_capability(account, "submissions:read_own")?;
        auth::resolve_accessible_delegate_account_id(pool, account, query.delegate_account_id)
            .await?
    };
    let assignments = workflows::list_pending_assignments_for_account(pool, delegate_account_id)
        .await?
        .into_iter()
        .map(|item| AssignmentResponseStartOption {
            workflow_assignment_id: item.workflow_assignment_id,
            workflow_name: item.workflow_name,
            workflow_version_label: item.workflow_version_label,
            workflow_step_title: item.workflow_step_title,
            workflow_step_position: item.workflow_step_position,
            workflow_step_count: item.workflow_step_count,
            form_id: item.form_id,
            form_name: item.form_name,
            form_version_id: item.form_version_id,
            form_version_label: item.form_version_label,
            node_id: item.node_id,
            node_name: item.node_name,
            account_id: item.account_id,
            account_display_name: item.account_display_name,
        })
        .collect::<Vec<_>>();

    Ok(AssignmentResponseStartOptions { assignments })
}

pub async fn list_submissions(
    pool: &PgPool,
    account: &auth::AccountContext,
    query: ListSubmissionsQuery,
) -> ApiResult<Vec<SubmissionSummary>> {
    let status = parse_submission_status_filter(query.status)?;
    let search = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let filters = SubmissionListFilters {
        status: status.as_deref(),
        form_id: query.form_id,
        node_id: query.node_id,
        search,
    };

    match auth::capability_boundary(pool, account, "submissions:manage").await? {
        auth::CapabilityBoundary::Global => {
            repo::list_admin_submission_summaries(pool, &filters).await
        }
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            repo::list_operator_submission_summaries(pool, &scope_ids, &filters).await
        }
        auth::CapabilityBoundary::None => {
            auth::ensure_capability(account, "submissions:read_own")?;
            let delegate_account_id = auth::resolve_accessible_delegate_account_id(
                pool,
                account,
                query.delegate_account_id,
            )
            .await?;
            repo::list_assignee_submission_summaries(pool, delegate_account_id, &filters).await
        }
    }
}

pub async fn get_submission(
    pool: &PgPool,
    account: &auth::AccountContext,
    submission_id: Uuid,
) -> ApiResult<SubmissionDetail> {
    require_submission_access(pool, account, submission_id).await?;
    repo::load_submission_detail(pool, submission_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("submission {submission_id}")))
}

pub async fn save_submission_values(
    pool: &PgPool,
    account: &auth::AccountContext,
    submission_id: Uuid,
    values: HashMap<String, Value>,
) -> ApiResult<Uuid> {
    let access = require_submission_access(pool, account, submission_id).await?;
    let form_version_id =
        require_draft_submission_status(submission_id, &access.status, access.form_version_id)?;
    let fields = repo::fields_by_key(pool, form_version_id).await?;

    for (key, value) in values {
        let field = fields
            .get(&key)
            .ok_or_else(|| ApiError::BadRequest(format!("unknown form field '{key}'")))?;
        validate_field_value(field.field_type, &value)?;
        repo::upsert_submission_value(pool, submission_id, form_version_id, field.id, value)
            .await?;
    }

    repo::audit_submission(pool, submission_id, "save_draft", Some(account.account_id)).await?;
    Ok(submission_id)
}

pub async fn submit_submission(
    pool: &PgPool,
    account: &auth::AccountContext,
    submission_id: Uuid,
) -> ApiResult<Uuid> {
    let access = require_submission_access(pool, account, submission_id).await?;
    let form_version_id =
        require_draft_submission_status(submission_id, &access.status, access.form_version_id)?;
    let fields = repo::fields_by_key(pool, form_version_id).await?;
    let saved_values = repo::saved_values_by_field_id(pool, submission_id).await?;

    for field in fields.values() {
        if let Some(value) = saved_values.get(&field.id) {
            validate_field_value(field.field_type, value)?;
        }
    }

    ensure_required_values_present(fields.values().map(|field| {
        RequiredFieldStatus {
            key: &field.key,
            required: field.required,
            has_value: saved_values
                .get(&field.id)
                .map(saved_value_counts_as_present)
                .unwrap_or(false),
        }
    }))
    .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    if !repo::mark_submission_submitted(pool, submission_id).await? {
        return Err(ApiError::BadRequest(
            "submitted records are immutable in the initial workflow".into(),
        ));
    }

    workflows::complete_workflow_step_and_advance(pool, submission_id).await?;
    repo::audit_submission(pool, submission_id, "submit", Some(account.account_id)).await?;
    Ok(submission_id)
}

pub async fn delete_draft_submission(
    pool: &PgPool,
    account: &auth::AccountContext,
    submission_id: Uuid,
) -> ApiResult<Uuid> {
    let access = require_submission_access(pool, account, submission_id).await?;
    require_draft_submission_status(submission_id, &access.status, access.form_version_id)?;

    repo::delete_workflow_step_instance_for_submission(pool, submission_id).await?;
    repo::delete_workflow_instance_for_submission(pool, submission_id).await?;
    repo::delete_submission(pool, submission_id).await?;
    Ok(submission_id)
}

pub fn require_draft_submission_status(
    _submission_id: Uuid,
    status: &str,
    form_version_id: Uuid,
) -> ApiResult<Uuid> {
    tessara_submissions::ensure_submission_is_draft(status)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    Ok(form_version_id)
}

pub async fn require_submission_access(
    pool: &PgPool,
    account: &auth::AccountContext,
    submission_id: Uuid,
) -> ApiResult<SubmissionAccess> {
    let row = repo::load_submission_access(pool, submission_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("submission {submission_id}")))?;

    match auth::capability_boundary(pool, account, "submissions:manage").await? {
        auth::CapabilityBoundary::Global => {}
        auth::CapabilityBoundary::Scoped(scope_ids) if scope_ids.contains(&row.node_id) => {}
        _ => {
            auth::ensure_capability(account, "submissions:respond")?;
            if row.assignment_account_id != Some(account.account_id)
                && !account
                    .delegations
                    .iter()
                    .any(|delegate| Some(delegate.account_id) == row.assignment_account_id)
            {
                return Err(ApiError::Forbidden("submissions:respond".into()));
            }
        }
    }

    Ok(SubmissionAccess {
        form_version_id: row.form_version_id,
        status: row.status,
    })
}

fn saved_value_counts_as_present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(values) => values.iter().any(saved_value_counts_as_present),
        _ => true,
    }
}
