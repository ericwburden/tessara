use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth,
    error::{ApiError, ApiResult},
};

use super::repo;

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

    if account.has_capability("workflows:manage")
        && auth::capability_allows_node(pool, account, "workflows:manage", access.node_id).await?
    {
        return Ok(());
    }

    if account.has_capability("submissions:respond")
        && (account.account_id == access.assignee_account_id
            || account
                .delegations
                .iter()
                .any(|delegate| delegate.account_id == access.assignee_account_id))
    {
        return Ok(());
    }

    Err(ApiError::Forbidden("workflow_assignment:start".into()))
}
