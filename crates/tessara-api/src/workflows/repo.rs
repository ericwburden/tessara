use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::ApiResult;

#[derive(Clone, Copy)]
pub struct WorkflowAssignmentAccessRow {
    pub node_id: Uuid,
    pub assignee_account_id: Uuid,
}

pub async fn load_workflow_assignment_access(
    pool: &PgPool,
    workflow_assignment_id: Uuid,
) -> ApiResult<Option<WorkflowAssignmentAccessRow>> {
    let row = sqlx::query(
        r#"
        SELECT node_id, account_id
        FROM workflow_assignments
        WHERE id = $1
        "#,
    )
    .bind(workflow_assignment_id)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok(WorkflowAssignmentAccessRow {
            node_id: row.try_get("node_id")?,
            assignee_account_id: row.try_get("account_id")?,
        })
    })
    .transpose()
}
