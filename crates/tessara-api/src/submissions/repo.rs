use std::collections::HashMap;

use serde_json::Value;
use sqlx::{PgPool, Row, postgres::PgRow};
use tessara_core::FieldType;
use uuid::Uuid;

use crate::{error::ApiResult, hierarchy::parse_field_type};

use super::dto::{
    ResponseNodeSummary, SubmissionAuditEventSummary, SubmissionDetail, SubmissionRuntimeDetail,
    SubmissionRuntimeStepHistory, SubmissionSummary, SubmissionValueDetail,
};

pub struct SubmissionAccessRow {
    pub form_version_id: Uuid,
    pub node_id: Uuid,
    pub status: String,
    pub assignment_account_id: Option<Uuid>,
}

pub struct FormFieldContract {
    pub id: Uuid,
    pub key: String,
    pub field_type: FieldType,
    pub required: bool,
}

pub struct SubmissionListFilters<'a> {
    pub status: Option<&'a str>,
    pub form_id: Option<Uuid>,
    pub node_id: Option<Uuid>,
    pub search: Option<&'a str>,
}

pub async fn list_all_published_form_versions(
    pool: &PgPool,
) -> ApiResult<Vec<crate::forms::PublishedFormVersionSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT
            forms.id AS form_id,
            forms.name AS form_name,
            forms.slug AS form_slug,
            form_versions.id AS form_version_id,
            form_versions.version_label,
            form_versions.published_at,
            COUNT(form_fields.id) AS field_count
        FROM form_versions
        JOIN forms ON forms.id = form_versions.form_id
        LEFT JOIN form_fields ON form_fields.form_version_id = form_versions.id
        WHERE form_versions.status = 'published'::form_version_status
        GROUP BY
            forms.id,
            forms.name,
            forms.slug,
            form_versions.id,
            form_versions.version_label,
            form_versions.published_at,
            form_versions.created_at
        ORDER BY forms.name, form_versions.created_at
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(published_form_version_summary_from_row)
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

pub async fn list_scoped_published_form_versions(
    pool: &PgPool,
    scope_ids: &[Uuid],
) -> ApiResult<Vec<crate::forms::PublishedFormVersionSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT
            forms.id AS form_id,
            forms.name AS form_name,
            forms.slug AS form_slug,
            form_versions.id AS form_version_id,
            form_versions.version_label,
            form_versions.published_at,
            COUNT(form_fields.id) AS field_count
        FROM form_versions
        JOIN forms ON forms.id = form_versions.form_id
        JOIN form_assignments ON form_assignments.form_version_id = form_versions.id
        LEFT JOIN form_fields ON form_fields.form_version_id = form_versions.id
        WHERE form_versions.status = 'published'::form_version_status
          AND form_assignments.node_id = ANY($1)
        GROUP BY
            forms.id,
            forms.name,
            forms.slug,
            form_versions.id,
            form_versions.version_label,
            form_versions.published_at,
            form_versions.created_at
        ORDER BY forms.name, form_versions.created_at
        "#,
    )
    .bind(scope_ids)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(published_form_version_summary_from_row)
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

pub async fn list_response_nodes(
    pool: &PgPool,
    scope_ids: Option<&[Uuid]>,
) -> ApiResult<Vec<ResponseNodeSummary>> {
    let rows = if let Some(scope_ids) = scope_ids {
        sqlx::query("SELECT id, name FROM nodes WHERE id = ANY($1) ORDER BY name, id")
            .bind(scope_ids)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query("SELECT id, name FROM nodes ORDER BY name, id")
            .fetch_all(pool)
            .await?
    };

    rows.into_iter()
        .map(|row| {
            Ok(ResponseNodeSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

pub async fn form_version_status(
    pool: &PgPool,
    form_version_id: Uuid,
) -> ApiResult<Option<String>> {
    sqlx::query_scalar("SELECT status::text FROM form_versions WHERE id = $1")
        .bind(form_version_id)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn create_form_assignment(
    pool: &PgPool,
    form_version_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
) -> ApiResult<Uuid> {
    sqlx::query_scalar(
        r#"
        INSERT INTO form_assignments (form_version_id, node_id, account_id)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(form_version_id)
    .bind(node_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn find_active_workflow_assignment(
    pool: &PgPool,
    form_version_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
) -> ApiResult<Option<Uuid>> {
    sqlx::query_scalar(
        r#"
        SELECT workflow_assignments.id
        FROM workflow_assignments
        JOIN workflow_steps ON workflow_steps.id = workflow_assignments.workflow_step_id
        WHERE workflow_steps.form_version_id = $1
          AND workflow_assignments.node_id = $2
          AND workflow_assignments.account_id = $3
          AND workflow_assignments.is_active = true
        ORDER BY workflow_assignments.created_at
        LIMIT 1
        "#,
    )
    .bind(form_version_id)
    .bind(node_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

pub async fn load_submission_access(
    pool: &PgPool,
    submission_id: Uuid,
) -> ApiResult<Option<SubmissionAccessRow>> {
    let row = sqlx::query(
        r#"
        SELECT
            submissions.form_version_id,
            submissions.node_id,
            submissions.status::text AS status,
            COALESCE(workflow_assignments.account_id, form_assignments.account_id) AS assignment_account_id
        FROM submissions
        JOIN form_assignments ON form_assignments.id = submissions.assignment_id
        LEFT JOIN workflow_assignments ON workflow_assignments.id = submissions.workflow_assignment_id
        WHERE submissions.id = $1
        "#,
    )
    .bind(submission_id)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok(SubmissionAccessRow {
            form_version_id: row.try_get("form_version_id")?,
            node_id: row.try_get("node_id")?,
            status: row.try_get("status")?,
            assignment_account_id: row.try_get("assignment_account_id")?,
        })
    })
    .transpose()
}

pub async fn fields_by_key(
    pool: &PgPool,
    form_version_id: Uuid,
) -> ApiResult<HashMap<String, FormFieldContract>> {
    let rows = sqlx::query(
        r#"
        SELECT id, key, field_type::text AS field_type, required
        FROM form_fields
        WHERE form_version_id = $1
        "#,
    )
    .bind(form_version_id)
    .fetch_all(pool)
    .await?;

    let mut fields = HashMap::new();
    for row in rows {
        let key: String = row.try_get("key")?;
        fields.insert(
            key.clone(),
            FormFieldContract {
                id: row.try_get("id")?,
                key,
                field_type: parse_field_type(&row.try_get::<String, _>("field_type")?)?,
                required: row.try_get("required")?,
            },
        );
    }

    Ok(fields)
}

pub async fn list_admin_submission_summaries(
    pool: &PgPool,
    filters: &SubmissionListFilters<'_>,
) -> ApiResult<Vec<SubmissionSummary>> {
    let rows = sqlx::query(SUBMISSION_SUMMARY_ADMIN_SQL)
        .bind(filters.status)
        .bind(filters.form_id)
        .bind(filters.node_id)
        .bind(filters.search)
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(submission_summary_from_row)
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

pub async fn list_operator_submission_summaries(
    pool: &PgPool,
    scope_ids: &[Uuid],
    filters: &SubmissionListFilters<'_>,
) -> ApiResult<Vec<SubmissionSummary>> {
    let rows = sqlx::query(SUBMISSION_SUMMARY_OPERATOR_SQL)
        .bind(scope_ids)
        .bind(filters.status)
        .bind(filters.form_id)
        .bind(filters.node_id)
        .bind(filters.search)
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(submission_summary_from_row)
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

pub async fn list_assignee_submission_summaries(
    pool: &PgPool,
    account_id: Uuid,
    filters: &SubmissionListFilters<'_>,
) -> ApiResult<Vec<SubmissionSummary>> {
    let rows = sqlx::query(SUBMISSION_SUMMARY_ASSIGNEE_SQL)
        .bind(account_id)
        .bind(filters.status)
        .bind(filters.form_id)
        .bind(filters.node_id)
        .bind(filters.search)
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(submission_summary_from_row)
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

pub async fn load_submission_detail(
    pool: &PgPool,
    submission_id: Uuid,
) -> ApiResult<Option<SubmissionDetail>> {
    let Some(row) = sqlx::query(
        r#"
        SELECT
            submissions.id,
            forms.id AS form_id,
            submissions.form_version_id,
            forms.name AS form_name,
            form_versions.version_label,
            submissions.node_id,
            nodes.name AS node_name,
            submissions.status::text AS status,
            submissions.created_at,
            submissions.submitted_at
        FROM submissions
        JOIN form_versions ON form_versions.id = submissions.form_version_id
        JOIN forms ON forms.id = form_versions.form_id
        JOIN nodes ON nodes.id = submissions.node_id
        WHERE submissions.id = $1
        "#,
    )
    .bind(submission_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let form_version_id = row.try_get::<Uuid, _>("form_version_id")?;
    let values = load_submission_value_details(pool, submission_id, form_version_id).await?;
    let audit_events = load_submission_audit_events(pool, submission_id).await?;
    let runtime = load_submission_runtime_detail(pool, submission_id).await?;

    Ok(Some(SubmissionDetail {
        id: row.try_get("id")?,
        form_id: row.try_get("form_id")?,
        form_version_id,
        form_name: row.try_get("form_name")?,
        version_label: row.try_get("version_label")?,
        node_id: row.try_get("node_id")?,
        node_name: row.try_get("node_name")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
        submitted_at: row.try_get("submitted_at")?,
        values,
        audit_events,
        runtime,
    }))
}

async fn load_submission_runtime_detail(
    pool: &PgPool,
    submission_id: Uuid,
) -> ApiResult<Option<SubmissionRuntimeDetail>> {
    let Some(current) = sqlx::query(
        r#"
        SELECT
            workflows.name AS workflow_name,
            workflow_instances.id AS workflow_instance_id,
            workflow_steps.workflow_version_id,
            workflow_steps.title AS current_step_title,
            workflow_steps.position AS current_step_position,
            (
                SELECT COUNT(*)
                FROM workflow_steps AS all_steps
                WHERE all_steps.workflow_version_id = workflow_steps.workflow_version_id
            ) AS step_count,
            next_steps.title AS next_step_title
        FROM submissions
        JOIN workflow_instances ON workflow_instances.id = submissions.workflow_instance_id
        JOIN workflow_step_instances ON workflow_step_instances.id = submissions.workflow_step_instance_id
        JOIN workflow_steps ON workflow_steps.id = workflow_step_instances.workflow_step_id
        JOIN workflow_versions ON workflow_versions.id = workflow_steps.workflow_version_id
        JOIN workflows ON workflows.id = workflow_versions.workflow_id
        LEFT JOIN workflow_steps AS next_steps
            ON next_steps.workflow_version_id = workflow_steps.workflow_version_id
           AND next_steps.position = workflow_steps.position + 1
        WHERE submissions.id = $1
        "#,
    )
    .bind(submission_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let workflow_instance_id: Uuid = current.try_get("workflow_instance_id")?;
    let history_rows = sqlx::query(
        r#"
        SELECT
            workflow_steps.title,
            forms.name AS form_name,
            workflow_step_instances.status,
            workflow_steps.position,
            workflow_step_instances.completed_at
        FROM workflow_step_instances
        JOIN workflow_steps ON workflow_steps.id = workflow_step_instances.workflow_step_id
        JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
        JOIN forms ON forms.id = form_versions.form_id
        WHERE workflow_step_instances.workflow_instance_id = $1
        ORDER BY workflow_steps.position
        "#,
    )
    .bind(workflow_instance_id)
    .fetch_all(pool)
    .await?;
    let history = history_rows
        .into_iter()
        .map(|row| {
            Ok(SubmissionRuntimeStepHistory {
                title: row.try_get("title")?,
                form_name: row.try_get("form_name")?,
                status: row.try_get("status")?,
                position: row.try_get("position")?,
                completed_at: row.try_get("completed_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Some(SubmissionRuntimeDetail {
        workflow_name: current.try_get("workflow_name")?,
        current_step_title: current.try_get("current_step_title")?,
        current_step_position: current.try_get("current_step_position")?,
        step_count: current.try_get("step_count")?,
        next_step_title: current.try_get("next_step_title")?,
        history,
    }))
}

pub async fn saved_values_by_field_id(
    pool: &PgPool,
    submission_id: Uuid,
) -> ApiResult<HashMap<Uuid, Value>> {
    let rows = sqlx::query(
        r#"
        SELECT field_id, value
        FROM submission_values
        WHERE submission_id = $1
        "#,
    )
    .bind(submission_id)
    .fetch_all(pool)
    .await?;

    let mut values = HashMap::new();
    for row in rows {
        values.insert(row.try_get("field_id")?, row.try_get("value")?);
    }
    Ok(values)
}

pub async fn upsert_submission_value(
    pool: &PgPool,
    submission_id: Uuid,
    field_id: Uuid,
    value: Value,
) -> ApiResult<()> {
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
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_submission_submitted(pool: &PgPool, submission_id: Uuid) -> ApiResult<bool> {
    let result = sqlx::query(
        r#"
        UPDATE submissions
        SET status = 'submitted'::submission_status, submitted_at = now()
        WHERE id = $1 AND status = 'draft'::submission_status
        "#,
    )
    .bind(submission_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

pub async fn complete_workflow_step_for_submission(
    pool: &PgPool,
    submission_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        UPDATE workflow_step_instances
        SET status = 'completed',
            completed_at = now()
        WHERE id = (
            SELECT workflow_step_instance_id
            FROM submissions
            WHERE id = $1
        )
        "#,
    )
    .bind(submission_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_workflow_step_instance_for_submission(
    pool: &PgPool,
    submission_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        DELETE FROM workflow_step_instances
        WHERE id = (
            SELECT workflow_step_instance_id
            FROM submissions
            WHERE id = $1
        )
        "#,
    )
    .bind(submission_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_workflow_instance_for_submission(
    pool: &PgPool,
    submission_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        DELETE FROM workflow_instances
        WHERE id = (
            SELECT workflow_instance_id
            FROM submissions
            WHERE id = $1
        )
          AND NOT EXISTS (
              SELECT 1
              FROM workflow_step_instances
              WHERE workflow_step_instances.workflow_instance_id = workflow_instances.id
                AND workflow_step_instances.status = 'completed'
          )
        "#,
    )
    .bind(submission_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_submission(pool: &PgPool, submission_id: Uuid) -> ApiResult<()> {
    sqlx::query("DELETE FROM submissions WHERE id = $1")
        .bind(submission_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn audit_submission(
    pool: &PgPool,
    submission_id: Uuid,
    event_type: &str,
    account_id: Option<Uuid>,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        INSERT INTO submission_audit_events (submission_id, event_type, account_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(submission_id)
    .bind(event_type)
    .bind(account_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn load_submission_value_details(
    pool: &PgPool,
    submission_id: Uuid,
    form_version_id: Uuid,
) -> ApiResult<Vec<SubmissionValueDetail>> {
    let rows = sqlx::query(
        r#"
        SELECT
            form_fields.id AS field_id,
            form_fields.key,
            form_fields.label,
            form_fields.field_type::text AS field_type,
            form_fields.required,
            submission_values.value
        FROM form_fields
        LEFT JOIN submission_values
            ON submission_values.field_id = form_fields.id
            AND submission_values.submission_id = $1
        WHERE form_fields.form_version_id = $2
        ORDER BY form_fields.position, form_fields.label
        "#,
    )
    .bind(submission_id)
    .bind(form_version_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(SubmissionValueDetail {
                field_id: row.try_get("field_id")?,
                key: row.try_get("key")?,
                label: row.try_get("label")?,
                field_type: row.try_get("field_type")?,
                required: row.try_get("required")?,
                value: row.try_get("value")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

async fn load_submission_audit_events(
    pool: &PgPool,
    submission_id: Uuid,
) -> ApiResult<Vec<SubmissionAuditEventSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT
            submission_audit_events.event_type,
            accounts.email AS account_email,
            submission_audit_events.created_at
        FROM submission_audit_events
        LEFT JOIN accounts ON accounts.id = submission_audit_events.account_id
        WHERE submission_audit_events.submission_id = $1
        ORDER BY submission_audit_events.created_at, submission_audit_events.id
        "#,
    )
    .bind(submission_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(SubmissionAuditEventSummary {
                event_type: row.try_get("event_type")?,
                account_email: row.try_get("account_email")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

fn published_form_version_summary_from_row(
    row: PgRow,
) -> Result<crate::forms::PublishedFormVersionSummary, sqlx::Error> {
    Ok(crate::forms::PublishedFormVersionSummary {
        form_id: row.try_get("form_id")?,
        form_name: row.try_get("form_name")?,
        form_slug: row.try_get("form_slug")?,
        form_version_id: row.try_get("form_version_id")?,
        version_label: row.try_get("version_label")?,
        published_at: row.try_get("published_at")?,
        field_count: row.try_get("field_count")?,
    })
}

fn submission_summary_from_row(row: PgRow) -> Result<SubmissionSummary, sqlx::Error> {
    Ok(SubmissionSummary {
        id: row.try_get("id")?,
        form_id: row.try_get("form_id")?,
        form_version_id: row.try_get("form_version_id")?,
        form_name: row.try_get("form_name")?,
        workflow_name: row.try_get("workflow_name")?,
        workflow_description: row.try_get("workflow_description")?,
        workflow_step_position: row.try_get("workflow_step_position")?,
        workflow_step_count: row.try_get("workflow_step_count")?,
        workflow_steps_completed: row.try_get("workflow_steps_completed")?,
        current_workflow_step_title: row.try_get("current_workflow_step_title")?,
        next_workflow_step_title: row.try_get("next_workflow_step_title")?,
        next_workflow_step_form_name: row.try_get("next_workflow_step_form_name")?,
        assigned_to_display_name: row.try_get("assigned_to_display_name")?,
        version_label: row.try_get("version_label")?,
        node_id: row.try_get("node_id")?,
        node_name: row.try_get("node_name")?,
        status: row.try_get("status")?,
        value_count: row.try_get("value_count")?,
        created_at: row.try_get("created_at")?,
        last_modified_at: row.try_get("last_modified_at")?,
        submitted_at: row.try_get("submitted_at")?,
    })
}

const SUBMISSION_SUMMARY_ADMIN_SQL: &str = r#"
SELECT
    submissions.id,
    forms.id AS form_id,
    submissions.form_version_id,
    forms.name AS form_name,
    workflows.name AS workflow_name,
    workflows.description AS workflow_description,
    workflow_steps.position AS workflow_step_position,
    (
        SELECT COUNT(*)
        FROM workflow_steps AS all_steps
        WHERE all_steps.workflow_version_id = workflow_versions.id
    ) AS workflow_step_count,
    (
        SELECT COUNT(*)
        FROM workflow_step_instances AS completed_steps
        WHERE completed_steps.workflow_instance_id = submissions.workflow_instance_id
          AND completed_steps.status = 'completed'
    ) AS workflow_steps_completed,
    workflow_steps.title AS current_workflow_step_title,
    next_steps.title AS next_workflow_step_title,
    next_forms.name AS next_workflow_step_form_name,
    workflow_accounts.display_name AS assigned_to_display_name,
    form_versions.version_label,
    submissions.node_id,
    nodes.name AS node_name,
    submissions.status::text AS status,
    submissions.created_at,
    COALESCE((
        SELECT MAX(submission_audit_events.created_at)
        FROM submission_audit_events
        WHERE submission_audit_events.submission_id = submissions.id
    ), submissions.created_at) AS last_modified_at,
    submissions.submitted_at,
    COUNT(submission_values.field_id) AS value_count
FROM submissions
JOIN form_versions ON form_versions.id = submissions.form_version_id
JOIN forms ON forms.id = form_versions.form_id
JOIN nodes ON nodes.id = submissions.node_id
LEFT JOIN workflow_assignments ON workflow_assignments.id = submissions.workflow_assignment_id
LEFT JOIN workflow_versions ON workflow_versions.id = workflow_assignments.workflow_version_id
LEFT JOIN workflows ON workflows.id = workflow_versions.workflow_id
LEFT JOIN accounts AS workflow_accounts ON workflow_accounts.id = workflow_assignments.account_id
LEFT JOIN workflow_step_instances ON workflow_step_instances.id = submissions.workflow_step_instance_id
LEFT JOIN workflow_steps ON workflow_steps.id = workflow_step_instances.workflow_step_id
LEFT JOIN workflow_steps AS next_steps
    ON next_steps.workflow_version_id = workflow_steps.workflow_version_id
   AND next_steps.position = workflow_steps.position + 1
LEFT JOIN form_versions AS next_form_versions ON next_form_versions.id = next_steps.form_version_id
LEFT JOIN forms AS next_forms ON next_forms.id = next_form_versions.form_id
LEFT JOIN submission_values ON submission_values.submission_id = submissions.id
WHERE ($1::submission_status IS NULL OR submissions.status = $1::submission_status)
  AND ($2::uuid IS NULL OR forms.id = $2)
  AND ($3::uuid IS NULL OR submissions.node_id = $3)
  AND (
      $4::text IS NULL
      OR forms.name ILIKE '%' || $4 || '%'
      OR nodes.name ILIKE '%' || $4 || '%'
      OR form_versions.version_label ILIKE '%' || $4 || '%'
      OR workflows.description ILIKE '%' || $4 || '%'
  )
GROUP BY
    submissions.id,
    forms.id,
    submissions.form_version_id,
    forms.name,
    workflows.name,
    workflows.description,
    workflow_versions.id,
    submissions.workflow_instance_id,
    workflow_steps.position,
    workflow_steps.title,
    next_steps.title,
    next_forms.name,
    workflow_accounts.display_name,
    form_versions.version_label,
    submissions.node_id,
    nodes.name,
    submissions.status,
    submissions.created_at,
    submissions.submitted_at,
    submissions.created_at
ORDER BY submissions.created_at, submissions.id
"#;

const SUBMISSION_SUMMARY_OPERATOR_SQL: &str = r#"
SELECT
    submissions.id,
    forms.id AS form_id,
    submissions.form_version_id,
    forms.name AS form_name,
    workflows.name AS workflow_name,
    workflows.description AS workflow_description,
    workflow_steps.position AS workflow_step_position,
    (
        SELECT COUNT(*)
        FROM workflow_steps AS all_steps
        WHERE all_steps.workflow_version_id = workflow_versions.id
    ) AS workflow_step_count,
    (
        SELECT COUNT(*)
        FROM workflow_step_instances AS completed_steps
        WHERE completed_steps.workflow_instance_id = submissions.workflow_instance_id
          AND completed_steps.status = 'completed'
    ) AS workflow_steps_completed,
    workflow_steps.title AS current_workflow_step_title,
    next_steps.title AS next_workflow_step_title,
    next_forms.name AS next_workflow_step_form_name,
    workflow_accounts.display_name AS assigned_to_display_name,
    form_versions.version_label,
    submissions.node_id,
    nodes.name AS node_name,
    submissions.status::text AS status,
    submissions.created_at,
    COALESCE((
        SELECT MAX(submission_audit_events.created_at)
        FROM submission_audit_events
        WHERE submission_audit_events.submission_id = submissions.id
    ), submissions.created_at) AS last_modified_at,
    submissions.submitted_at,
    COUNT(submission_values.field_id) AS value_count
FROM submissions
JOIN form_versions ON form_versions.id = submissions.form_version_id
JOIN forms ON forms.id = form_versions.form_id
JOIN nodes ON nodes.id = submissions.node_id
LEFT JOIN workflow_assignments ON workflow_assignments.id = submissions.workflow_assignment_id
LEFT JOIN workflow_versions ON workflow_versions.id = workflow_assignments.workflow_version_id
LEFT JOIN workflows ON workflows.id = workflow_versions.workflow_id
LEFT JOIN accounts AS workflow_accounts ON workflow_accounts.id = workflow_assignments.account_id
LEFT JOIN workflow_step_instances ON workflow_step_instances.id = submissions.workflow_step_instance_id
LEFT JOIN workflow_steps ON workflow_steps.id = workflow_step_instances.workflow_step_id
LEFT JOIN workflow_steps AS next_steps
    ON next_steps.workflow_version_id = workflow_steps.workflow_version_id
   AND next_steps.position = workflow_steps.position + 1
LEFT JOIN form_versions AS next_form_versions ON next_form_versions.id = next_steps.form_version_id
LEFT JOIN forms AS next_forms ON next_forms.id = next_form_versions.form_id
LEFT JOIN submission_values ON submission_values.submission_id = submissions.id
WHERE submissions.node_id = ANY($1)
  AND ($2::submission_status IS NULL OR submissions.status = $2::submission_status)
  AND ($3::uuid IS NULL OR forms.id = $3)
  AND ($4::uuid IS NULL OR submissions.node_id = $4)
  AND (
      $5::text IS NULL
      OR forms.name ILIKE '%' || $5 || '%'
      OR nodes.name ILIKE '%' || $5 || '%'
      OR form_versions.version_label ILIKE '%' || $5 || '%'
      OR workflows.description ILIKE '%' || $5 || '%'
  )
GROUP BY
    submissions.id,
    forms.id,
    submissions.form_version_id,
    forms.name,
    workflows.name,
    workflows.description,
    workflow_versions.id,
    submissions.workflow_instance_id,
    workflow_steps.position,
    workflow_steps.title,
    next_steps.title,
    next_forms.name,
    workflow_accounts.display_name,
    form_versions.version_label,
    submissions.node_id,
    nodes.name,
    submissions.status,
    submissions.created_at,
    submissions.submitted_at,
    submissions.created_at
ORDER BY submissions.created_at, submissions.id
"#;

const SUBMISSION_SUMMARY_ASSIGNEE_SQL: &str = r#"
SELECT
    submissions.id,
    forms.id AS form_id,
    submissions.form_version_id,
    forms.name AS form_name,
    workflows.name AS workflow_name,
    workflows.description AS workflow_description,
    workflow_steps.position AS workflow_step_position,
    (
        SELECT COUNT(*)
        FROM workflow_steps AS all_steps
        WHERE all_steps.workflow_version_id = workflow_versions.id
    ) AS workflow_step_count,
    (
        SELECT COUNT(*)
        FROM workflow_step_instances AS completed_steps
        WHERE completed_steps.workflow_instance_id = submissions.workflow_instance_id
          AND completed_steps.status = 'completed'
    ) AS workflow_steps_completed,
    workflow_steps.title AS current_workflow_step_title,
    next_steps.title AS next_workflow_step_title,
    next_forms.name AS next_workflow_step_form_name,
    workflow_assignments.account_id,
    accounts.display_name AS assigned_to_display_name,
    form_versions.version_label,
    submissions.node_id,
    nodes.name AS node_name,
    submissions.status::text AS status,
    submissions.created_at,
    COALESCE((
        SELECT MAX(submission_audit_events.created_at)
        FROM submission_audit_events
        WHERE submission_audit_events.submission_id = submissions.id
    ), submissions.created_at) AS last_modified_at,
    submissions.submitted_at,
    COUNT(submission_values.field_id) AS value_count
FROM submissions
JOIN workflow_assignments ON workflow_assignments.id = submissions.workflow_assignment_id
JOIN workflow_versions ON workflow_versions.id = workflow_assignments.workflow_version_id
JOIN workflows ON workflows.id = workflow_versions.workflow_id
JOIN accounts ON accounts.id = workflow_assignments.account_id
LEFT JOIN workflow_step_instances ON workflow_step_instances.id = submissions.workflow_step_instance_id
LEFT JOIN workflow_steps ON workflow_steps.id = workflow_step_instances.workflow_step_id
LEFT JOIN workflow_steps AS next_steps
    ON next_steps.workflow_version_id = workflow_steps.workflow_version_id
   AND next_steps.position = workflow_steps.position + 1
LEFT JOIN form_versions AS next_form_versions ON next_form_versions.id = next_steps.form_version_id
LEFT JOIN forms AS next_forms ON next_forms.id = next_form_versions.form_id
JOIN form_versions ON form_versions.id = submissions.form_version_id
JOIN forms ON forms.id = form_versions.form_id
JOIN nodes ON nodes.id = submissions.node_id
LEFT JOIN submission_values ON submission_values.submission_id = submissions.id
WHERE workflow_assignments.account_id = $1
  AND ($2::submission_status IS NULL OR submissions.status = $2::submission_status)
  AND ($3::uuid IS NULL OR forms.id = $3)
  AND ($4::uuid IS NULL OR submissions.node_id = $4)
  AND (
      $5::text IS NULL
      OR forms.name ILIKE '%' || $5 || '%'
      OR nodes.name ILIKE '%' || $5 || '%'
      OR form_versions.version_label ILIKE '%' || $5 || '%'
      OR workflows.description ILIKE '%' || $5 || '%'
  )
GROUP BY
    submissions.id,
    forms.id,
    submissions.form_version_id,
    forms.name,
    workflows.name,
    workflows.description,
    workflow_versions.id,
    submissions.workflow_instance_id,
    workflow_steps.position,
    workflow_steps.title,
    next_steps.title,
    next_forms.name,
    workflow_assignments.account_id,
    accounts.display_name,
    form_versions.version_label,
    submissions.node_id,
    nodes.name,
    submissions.status,
    submissions.created_at,
    submissions.submitted_at,
    submissions.created_at
ORDER BY submissions.created_at, submissions.id
"#;
