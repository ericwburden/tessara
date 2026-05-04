use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth,
    error::{ApiError, ApiResult},
};

use super::repo;

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

    if !account.is_admin() {
        if account.is_operator() {
            let scope_ids = auth::effective_scope_node_ids(pool, account.account_id).await?;
            if !scope_ids.contains(&row.node_id) {
                return Err(ApiError::Forbidden("submissions:write".into()));
            }
        } else if row.assignment_account_id != Some(account.account_id)
            && !account
                .delegations
                .iter()
                .any(|delegate| Some(delegate.account_id) == row.assignment_account_id)
        {
            return Err(ApiError::Forbidden("submissions:write".into()));
        }
    }

    Ok(SubmissionAccess {
        form_version_id: row.form_version_id,
        status: row.status,
    })
}
