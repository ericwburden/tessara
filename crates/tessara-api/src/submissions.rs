use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use tessara_core::FieldType;
use tessara_submissions::{
    RequiredFieldStatus, ensure_form_version_accepts_submission, ensure_required_values_present,
    ensure_submission_is_draft,
};
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, parse_field_type, validate_field_value},
};

#[derive(Deserialize)]
pub struct CreateDraftRequest {
    form_version_id: Uuid,
    node_id: Uuid,
}

#[derive(Deserialize)]
pub struct SaveSubmissionValuesRequest {
    values: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub struct ListSubmissionsQuery {
    status: Option<String>,
    form_id: Option<Uuid>,
    node_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct SubmissionSummary {
    id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    form_name: String,
    version_label: String,
    node_id: Uuid,
    node_name: String,
    status: String,
    value_count: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    submitted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct SubmissionDetail {
    id: Uuid,
    form_version_id: Uuid,
    form_name: String,
    version_label: String,
    node_id: Uuid,
    node_name: String,
    status: String,
    submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    values: Vec<SubmissionValueDetail>,
    audit_events: Vec<SubmissionAuditEventSummary>,
}

#[derive(Serialize)]
pub struct SubmissionValueDetail {
    field_id: Uuid,
    key: String,
    label: String,
    field_type: String,
    required: bool,
    value: Option<Value>,
}

#[derive(Serialize)]
pub struct SubmissionAuditEventSummary {
    event_type: String,
    account_email: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

struct FormFieldContract {
    id: Uuid,
    key: String,
    field_type: FieldType,
    required: bool,
}

pub async fn create_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDraftRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "submissions:write").await?;

    let status: Option<String> =
        sqlx::query_scalar("SELECT status::text FROM form_versions WHERE id = $1")
            .bind(payload.form_version_id)
            .fetch_optional(&state.pool)
            .await?;
    ensure_form_version_accepts_submission(status.as_deref().unwrap_or_default())
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    let assignment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO form_assignments (form_version_id, node_id, account_id)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(payload.form_version_id)
    .bind(payload.node_id)
    .bind(account.account_id)
    .fetch_one(&state.pool)
    .await?;

    let submission_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO submissions (assignment_id, form_version_id, node_id)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(assignment_id)
    .bind(payload.form_version_id)
    .bind(payload.node_id)
    .fetch_one(&state.pool)
    .await?;

    audit_submission(
        &state.pool,
        submission_id,
        "create_draft",
        Some(account.account_id),
    )
    .await?;

    Ok(Json(IdResponse { id: submission_id }))
}

/// Lists submissions for the current local workflow shell.
pub async fn list_submissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListSubmissionsQuery>,
) -> ApiResult<Json<Vec<SubmissionSummary>>> {
    auth::require_capability(&state.pool, &headers, "submissions:write").await?;
    let status = parse_submission_status_filter(query.status)?;

    let rows = sqlx::query(
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
            submissions.submitted_at,
            COUNT(submission_values.field_id) AS value_count
        FROM submissions
        JOIN form_versions ON form_versions.id = submissions.form_version_id
        JOIN forms ON forms.id = form_versions.form_id
        JOIN nodes ON nodes.id = submissions.node_id
        LEFT JOIN submission_values ON submission_values.submission_id = submissions.id
        WHERE ($1::submission_status IS NULL OR submissions.status = $1::submission_status)
          AND ($2::uuid IS NULL OR forms.id = $2)
          AND ($3::uuid IS NULL OR submissions.node_id = $3)
        GROUP BY
            submissions.id,
            forms.id,
            submissions.form_version_id,
            forms.name,
            form_versions.version_label,
            submissions.node_id,
            nodes.name,
            submissions.status,
            submissions.created_at,
            submissions.submitted_at,
            submissions.created_at
        ORDER BY submissions.created_at, submissions.id
        "#,
    )
    .bind(status)
    .bind(query.form_id)
    .bind(query.node_id)
    .fetch_all(&state.pool)
    .await?;

    let submissions = rows
        .into_iter()
        .map(|row| {
            Ok(SubmissionSummary {
                id: row.try_get("id")?,
                form_id: row.try_get("form_id")?,
                form_version_id: row.try_get("form_version_id")?,
                form_name: row.try_get("form_name")?,
                version_label: row.try_get("version_label")?,
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                status: row.try_get("status")?,
                value_count: row.try_get("value_count")?,
                created_at: row.try_get("created_at")?,
                submitted_at: row.try_get("submitted_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(submissions))
}

/// Returns a submission with saved values and audit history for inspection.
pub async fn get_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(submission_id): Path<Uuid>,
) -> ApiResult<Json<SubmissionDetail>> {
    auth::require_capability(&state.pool, &headers, "submissions:write").await?;

    let row = sqlx::query(
        r#"
        SELECT
            submissions.id,
            submissions.form_version_id,
            forms.name AS form_name,
            form_versions.version_label,
            submissions.node_id,
            nodes.name AS node_name,
            submissions.status::text AS status,
            submissions.submitted_at
        FROM submissions
        JOIN form_versions ON form_versions.id = submissions.form_version_id
        JOIN forms ON forms.id = form_versions.form_id
        JOIN nodes ON nodes.id = submissions.node_id
        WHERE submissions.id = $1
        "#,
    )
    .bind(submission_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("submission {submission_id}")))?;

    let value_rows = sqlx::query(
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
    .bind(row.try_get::<Uuid, _>("form_version_id")?)
    .fetch_all(&state.pool)
    .await?;

    let mut values = Vec::new();
    for value_row in value_rows {
        values.push(SubmissionValueDetail {
            field_id: value_row.try_get("field_id")?,
            key: value_row.try_get("key")?,
            label: value_row.try_get("label")?,
            field_type: value_row.try_get("field_type")?,
            required: value_row.try_get("required")?,
            value: value_row.try_get("value")?,
        });
    }

    let audit_rows = sqlx::query(
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
    .fetch_all(&state.pool)
    .await?;

    let audit_events = audit_rows
        .into_iter()
        .map(|audit_row| {
            Ok(SubmissionAuditEventSummary {
                event_type: audit_row.try_get("event_type")?,
                account_email: audit_row.try_get("account_email")?,
                created_at: audit_row.try_get("created_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(SubmissionDetail {
        id: row.try_get("id")?,
        form_version_id: row.try_get("form_version_id")?,
        form_name: row.try_get("form_name")?,
        version_label: row.try_get("version_label")?,
        node_id: row.try_get("node_id")?,
        node_name: row.try_get("node_name")?,
        status: row.try_get("status")?,
        submitted_at: row.try_get("submitted_at")?,
        values,
        audit_events,
    }))
}

pub async fn save_submission_values(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(submission_id): Path<Uuid>,
    Json(payload): Json<SaveSubmissionValuesRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "submissions:write").await?;

    let form_version_id = require_draft_submission(&state.pool, submission_id).await?;
    let fields = fields_by_key(&state.pool, form_version_id).await?;

    for (key, value) in payload.values {
        let field = fields
            .get(&key)
            .ok_or_else(|| ApiError::BadRequest(format!("unknown form field '{key}'")))?;
        validate_field_value(field.field_type, &value)?;

        sqlx::query(
            r#"
            INSERT INTO submission_values (submission_id, field_id, value)
            VALUES ($1, $2, $3)
            ON CONFLICT (submission_id, field_id)
            DO UPDATE SET value = EXCLUDED.value
            "#,
        )
        .bind(submission_id)
        .bind(field.id)
        .bind(value)
        .execute(&state.pool)
        .await?;
    }

    audit_submission(
        &state.pool,
        submission_id,
        "save_draft",
        Some(account.account_id),
    )
    .await?;

    Ok(Json(IdResponse { id: submission_id }))
}

pub async fn submit_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(submission_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "submissions:write").await?;

    let form_version_id = require_draft_submission(&state.pool, submission_id).await?;
    let fields = fields_by_key(&state.pool, form_version_id).await?;
    let submitted_field_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT field_id FROM submission_values WHERE submission_id = $1",
    )
    .bind(submission_id)
    .fetch_all(&state.pool)
    .await?;

    ensure_required_values_present(fields.values().map(|field| RequiredFieldStatus {
        key: &field.key,
        required: field.required,
        has_value: submitted_field_ids.contains(&field.id),
    }))
    .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    sqlx::query(
        r#"
        UPDATE submissions
        SET status = 'submitted'::submission_status, submitted_at = now()
        WHERE id = $1 AND status = 'draft'::submission_status
        "#,
    )
    .bind(submission_id)
    .execute(&state.pool)
    .await?;

    audit_submission(
        &state.pool,
        submission_id,
        "submit",
        Some(account.account_id),
    )
    .await?;

    Ok(Json(IdResponse { id: submission_id }))
}

/// Deletes an unsubmitted draft submission.
pub async fn delete_draft_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(submission_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "submissions:write").await?;
    require_draft_submission(&state.pool, submission_id).await?;

    sqlx::query("DELETE FROM submissions WHERE id = $1")
        .bind(submission_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: submission_id }))
}

fn parse_submission_status_filter(status: Option<String>) -> ApiResult<Option<String>> {
    match status.as_deref() {
        None | Some("") => Ok(None),
        Some("draft" | "submitted") => Ok(status),
        Some(value) => Err(ApiError::BadRequest(format!(
            "unsupported submission status filter '{value}'"
        ))),
    }
}

async fn require_draft_submission(pool: &sqlx::PgPool, submission_id: Uuid) -> ApiResult<Uuid> {
    let row = sqlx::query(
        "SELECT form_version_id, status::text AS status FROM submissions WHERE id = $1",
    )
    .bind(submission_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("submission {submission_id}")))?;

    let status: String = row.try_get("status")?;
    ensure_submission_is_draft(&status).map_err(|error| ApiError::BadRequest(error.to_string()))?;

    Ok(row.try_get("form_version_id")?)
}

async fn fields_by_key(
    pool: &sqlx::PgPool,
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

async fn audit_submission(
    pool: &sqlx::PgPool,
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
