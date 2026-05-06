use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth,
    error::{ApiError, ApiResult},
};

use super::repo;

pub use super::handlers::{
    complete_workflow_step_and_advance, ensure_submission_runtime_linkage,
    ensure_workflow_assignment_for_form_assignment, ensure_workflow_for_published_form_version_tx,
    list_pending_assignments_for_account, start_workflow_assignment,
};

pub(crate) async fn ensure_can_start_assignment(
    pool: &PgPool,
    account: &auth::AccountContext,
    workflow_assignment_id: Uuid,
) -> ApiResult<()> {
    let Some(access) = repo::load_workflow_assignment_access(pool, workflow_assignment_id).await?
    else {
        return Err(ApiError::NotFound(format!(
            "workflow assignment {workflow_assignment_id}"
        )));
    };

    if account.is_admin() {
        return Ok(());
    }

    if account.is_operator() {
        let scope_ids = auth::effective_scope_node_ids(pool, account.account_id).await?;
        if scope_ids.contains(&access.node_id) {
            return Ok(());
        }
        return Err(ApiError::Forbidden("workflow_assignment:start".into()));
    }

    if account.account_id == access.assignee_account_id
        || account
            .delegations
            .iter()
            .any(|delegate| delegate.account_id == access.assignee_account_id)
    {
        return Ok(());
    }

    Err(ApiError::Forbidden("workflow_assignment:start".into()))
}
