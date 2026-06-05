use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    workflows,
};

pub(super) struct WorkflowStepSeed<'a> {
    pub(super) form_version_id: Uuid,
    pub(super) title: &'a str,
    pub(super) position: i32,
}

pub(super) async fn ensure_program_checkpoint_workflow(
    pool: &PgPool,
    workflow_node_type_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
    steps: &[WorkflowStepSeed<'_>],
) -> ApiResult<(Uuid, Uuid, Uuid)> {
    let workflow_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workflows (workflow_node_type_id, name, slug, description)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (slug)
        DO UPDATE SET
            workflow_node_type_id = EXCLUDED.workflow_node_type_id,
            name = EXCLUDED.name,
            description = EXCLUDED.description
        RETURNING id
        "#,
    )
    .bind(workflow_node_type_id)
    .bind("Demo Program Checkpoint Workflow")
    .bind("demo-program-checkpoint-workflow")
    .bind("Program-scoped workflow that uses Program and Activity form revisions.")
    .fetch_one(pool)
    .await?;

    sqlx::query("DELETE FROM workflow_available_nodes WHERE workflow_id = $1")
        .bind(workflow_id)
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        INSERT INTO workflow_available_nodes (workflow_id, node_id)
        SELECT $1, nodes.id
        FROM nodes
        WHERE nodes.node_type_id = $2
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(workflow_id)
    .bind(workflow_node_type_id)
    .execute(pool)
    .await?;

    steps
        .first()
        .ok_or_else(|| ApiError::BadRequest("demo workflow requires a step".into()))?;
    let workflow_version_id: Uuid = if let Some(existing) = sqlx::query_scalar(
        "SELECT id FROM workflow_versions WHERE workflow_id = $1 AND version_label = '1' LIMIT 1",
    )
    .bind(workflow_id)
    .fetch_optional(pool)
    .await?
    {
        sqlx::query(
            r#"
            UPDATE workflow_versions
            SET status = 'published'::form_version_status,
                published_at = COALESCE(published_at, now())
            WHERE id = $1
            "#,
        )
        .bind(existing)
        .execute(pool)
        .await?;
        existing
    } else {
        sqlx::query_scalar(
            r#"
            INSERT INTO workflow_versions (
                workflow_id,
                version_label,
                status,
                published_at
            )
            VALUES ($1, '1', 'published'::form_version_status, now())
            RETURNING id
            "#,
        )
        .bind(workflow_id)
        .fetch_one(pool)
        .await?
    };

    let mut first_step_id = None;
    for step in steps {
        let step_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO workflow_steps (workflow_version_id, form_version_id, title, position)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (workflow_version_id, position)
            DO UPDATE SET
                form_version_id = EXCLUDED.form_version_id,
                title = EXCLUDED.title
            RETURNING id
            "#,
        )
        .bind(workflow_version_id)
        .bind(step.form_version_id)
        .bind(step.title)
        .bind(step.position)
        .fetch_one(pool)
        .await?;
        if step.position == 0 {
            first_step_id = Some(step_id);
        }
    }

    let first_step_id = first_step_id
        .ok_or_else(|| ApiError::BadRequest("demo workflow requires step 0".into()))?;
    let workflow_assignment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workflow_assignments (
            workflow_version_id,
            workflow_step_id,
            node_id,
            account_id,
            is_active
        )
        VALUES ($1, $2, $3, $4, true)
        ON CONFLICT (workflow_step_id, node_id, account_id)
        DO UPDATE SET
            workflow_version_id = EXCLUDED.workflow_version_id,
            is_active = true
        RETURNING id
        "#,
    )
    .bind(workflow_version_id)
    .bind(first_step_id)
    .bind(node_id)
    .bind(account_id)
    .fetch_one(pool)
    .await?;

    Ok((workflow_id, workflow_version_id, workflow_assignment_id))
}

pub(super) async fn ensure_single_form_workflow_assignment(
    pool: &PgPool,
    form_version_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
) -> ApiResult<Uuid> {
    workflows::ensure_workflow_assignment_for_form_version(
        pool,
        form_version_id,
        node_id,
        account_id,
    )
    .await
}
