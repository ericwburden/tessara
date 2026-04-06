use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::Deserialize;
use serde_json::Value;
use sqlx::Row;
use tessara_core::FieldType;
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
    if status.as_deref() != Some("published") {
        return Err(ApiError::BadRequest(
            "submissions can only use published form versions".into(),
        ));
    }

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

    for field in fields.values() {
        if field.required && !submitted_field_ids.contains(&field.id) {
            return Err(ApiError::BadRequest(format!(
                "required field '{}' is missing",
                field.key
            )));
        }
    }

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

async fn require_draft_submission(pool: &sqlx::PgPool, submission_id: Uuid) -> ApiResult<Uuid> {
    let row = sqlx::query(
        "SELECT form_version_id, status::text AS status FROM submissions WHERE id = $1",
    )
    .bind(submission_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("submission {submission_id}")))?;

    let status: String = row.try_get("status")?;
    if status != "draft" {
        return Err(ApiError::BadRequest(
            "submitted records are immutable in the initial workflow".into(),
        ));
    }

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
