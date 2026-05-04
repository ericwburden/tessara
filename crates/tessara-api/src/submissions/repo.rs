use std::collections::HashMap;

use sqlx::{PgPool, Row};
use tessara_core::FieldType;
use uuid::Uuid;

use crate::{error::ApiResult, hierarchy::parse_field_type};

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
