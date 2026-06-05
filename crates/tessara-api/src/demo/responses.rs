use std::collections::HashMap;

use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    workflows,
};

use super::workflows::ensure_single_form_workflow_assignment;

pub(super) struct SeedSubmissionSpec<'a> {
    pub(super) seed_key: &'a str,
    pub(super) status: &'a str,
    pub(super) values: Vec<(&'a str, Value)>,
}

pub(super) async fn ensure_seed_submission(
    pool: &PgPool,
    account_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
    spec: SeedSubmissionSpec<'_>,
) -> ApiResult<Uuid> {
    let submission_id = if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT submission_id
        FROM submission_audit_events
        WHERE event_type = $1
        LIMIT 1
        "#,
    )
    .bind(spec.seed_key)
    .fetch_optional(pool)
    .await?
    {
        id
    } else {
        let workflow_assignment_id =
            ensure_single_form_workflow_assignment(pool, form_version_id, node_id, account_id)
                .await?;

        let submission_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO submissions (
                form_version_id,
                node_id,
                workflow_assignment_id,
                status,
                submitted_at
            )
            VALUES (
                $1,
                $2,
                $3,
                $4::submission_status,
                CASE WHEN $4 = 'submitted' THEN now() ELSE NULL END
            )
            RETURNING id
            "#,
        )
        .bind(form_version_id)
        .bind(node_id)
        .bind(workflow_assignment_id)
        .bind(spec.status)
        .fetch_one(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO submission_audit_events (submission_id, event_type, account_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(submission_id)
        .bind(spec.seed_key)
        .bind(account_id)
        .execute(pool)
        .await?;

        submission_id
    };

    let field_rows = sqlx::query(
        r#"
        SELECT id, key
        FROM form_fields
        WHERE form_version_id = $1
        "#,
    )
    .bind(form_version_id)
    .fetch_all(pool)
    .await?;

    let mut field_ids_by_key = HashMap::new();
    for row in field_rows {
        let field_id: Uuid = row.try_get("id")?;
        let key: String = row.try_get("key")?;
        field_ids_by_key.insert(key, field_id);
    }

    let mut retained_field_ids = Vec::new();
    for (key, value) in spec.values {
        let field_id = field_ids_by_key
            .get(key)
            .copied()
            .ok_or_else(|| ApiError::BadRequest(format!("unknown demo seed field '{key}'")))?;
        retained_field_ids.push(field_id);
        upsert_submission_value(pool, submission_id, field_id, value).await?;
    }

    sqlx::query(
        r#"
        DELETE FROM submission_values
        WHERE submission_id = $1
          AND NOT (field_id = ANY($2))
        "#,
    )
    .bind(submission_id)
    .bind(&retained_field_ids)
    .execute(pool)
    .await?;

    sqlx::query(
        "DELETE FROM submission_value_multi WHERE submission_id = $1 AND NOT (field_id = ANY($2))",
    )
    .bind(submission_id)
    .bind(&retained_field_ids)
    .execute(pool)
    .await?;

    if spec.status == "submitted" {
        sqlx::query(
            r#"
            UPDATE submissions
            SET status = 'submitted'::submission_status,
                submitted_at = COALESCE(submitted_at, now())
            WHERE id = $1
            "#,
        )
        .bind(submission_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE submissions
            SET status = 'draft'::submission_status,
                submitted_at = NULL
            WHERE id = $1
            "#,
        )
        .bind(submission_id)
        .execute(pool)
        .await?;
    }

    let workflow_assignment_id =
        ensure_single_form_workflow_assignment(pool, form_version_id, node_id, account_id).await?;

    workflows::ensure_submission_runtime_linkage(
        pool,
        submission_id,
        workflow_assignment_id,
        account_id,
        spec.status == "submitted",
    )
    .await?;

    Ok(submission_id)
}

async fn upsert_submission_value(
    pool: &PgPool,
    submission_id: Uuid,
    field_id: Uuid,
    value: Value,
) -> ApiResult<()> {
    if value.is_null() {
        sqlx::query("DELETE FROM submission_values WHERE submission_id = $1 AND field_id = $2")
            .bind(submission_id)
            .bind(field_id)
            .execute(pool)
            .await?;
        sqlx::query(
            "DELETE FROM submission_value_multi WHERE submission_id = $1 AND field_id = $2",
        )
        .bind(submission_id)
        .bind(field_id)
        .execute(pool)
        .await?;
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO submission_values (submission_id, field_id, value)
        VALUES ($1, $2, $3)
        ON CONFLICT (submission_id, field_id)
        DO UPDATE SET value = EXCLUDED.value
        "#,
    )
    .bind(submission_id)
    .bind(field_id)
    .bind(&value)
    .execute(pool)
    .await?;

    sqlx::query("DELETE FROM submission_value_multi WHERE submission_id = $1 AND field_id = $2")
        .bind(submission_id)
        .bind(field_id)
        .execute(pool)
        .await?;

    if let Some(items) = value.as_array() {
        for item in items {
            let Some(item_value) = item.as_str() else {
                continue;
            };
            sqlx::query(
                r#"
                INSERT INTO submission_value_multi (submission_id, field_id, value)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(submission_id)
            .bind(field_id)
            .bind(item_value)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}
