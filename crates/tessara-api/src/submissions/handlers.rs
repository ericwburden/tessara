use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use sqlx::Row;
use tessara_submissions::{
    RequiredFieldStatus, ensure_form_version_accepts_submission, ensure_required_values_present,
};
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, validate_field_value},
    workflows,
};

use super::dto::{
    CreateDraftRequest, ListSubmissionsQuery, ResponseNodeSummary, ResponseStartAssignment,
    ResponseStartOptions, SaveSubmissionValuesRequest, SubmissionAuditEventSummary,
    SubmissionDetail, SubmissionSummary, SubmissionValueDetail,
};
use super::{repo, service};

pub async fn list_response_start_options(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListSubmissionsQuery>,
) -> ApiResult<Json<ResponseStartOptions>> {
    let account = auth::require_authenticated(&state.pool, &headers).await?;

    if account.is_admin() || account.is_operator() {
        let published_forms = if account.is_operator() {
            let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
            sqlx::query(
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
            .fetch_all(&state.pool)
            .await?
            .into_iter()
            .map(|row| {
                Ok(crate::forms::PublishedFormVersionSummary {
                    form_id: row.try_get("form_id")?,
                    form_name: row.try_get("form_name")?,
                    form_slug: row.try_get("form_slug")?,
                    form_version_id: row.try_get("form_version_id")?,
                    version_label: row.try_get("version_label")?,
                    published_at: row.try_get("published_at")?,
                    field_count: row.try_get("field_count")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?
        } else {
            crate::forms::list_published_form_versions(State(state.clone()), headers.clone())
                .await?
                .0
        };

        let nodes = if account.is_operator() {
            let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
            sqlx::query("SELECT id, name FROM nodes WHERE id = ANY($1) ORDER BY name, id")
                .bind(scope_ids)
                .fetch_all(&state.pool)
                .await?
        } else {
            sqlx::query("SELECT id, name FROM nodes ORDER BY name, id")
                .fetch_all(&state.pool)
                .await?
        }
        .into_iter()
        .map(|row| {
            Ok(ResponseNodeSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

        return Ok(Json(ResponseStartOptions {
            mode: "scoped".into(),
            published_forms,
            nodes,
            assignments: Vec::new(),
        }));
    }

    let delegate_account_id = auth::resolve_accessible_delegate_account_id(
        &state.pool,
        &account,
        query.delegate_account_id,
    )
    .await?;
    let assignments =
        workflows::list_pending_assignments_for_account(&state.pool, delegate_account_id)
            .await?
            .into_iter()
            .map(|item| ResponseStartAssignment {
                form_id: item.form_id,
                form_name: item.form_name,
                form_version_id: item.form_version_id,
                version_label: item
                    .form_version_label
                    .unwrap_or_else(|| "Published".into()),
                node_id: item.node_id,
                node_name: item.node_name,
                delegate_account_id: Some(item.account_id),
                delegate_display_name: Some(item.account_display_name),
            })
            .collect::<Vec<_>>();

    Ok(Json(ResponseStartOptions {
        mode: "assignment".into(),
        published_forms: Vec::new(),
        nodes: Vec::new(),
        assignments,
    }))
}

pub async fn create_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDraftRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_authenticated(&state.pool, &headers).await?;

    let status: Option<String> =
        sqlx::query_scalar("SELECT status::text FROM form_versions WHERE id = $1")
            .bind(payload.form_version_id)
            .fetch_optional(&state.pool)
            .await?;
    ensure_form_version_accepts_submission(status.as_deref().unwrap_or_default())
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    let workflow_assignment_id: Uuid = if account.is_admin() || account.is_operator() {
        if account.is_operator() {
            let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
            if !scope_ids.contains(&payload.node_id) {
                return Err(ApiError::Forbidden("submissions:write".into()));
            }
        }

        let form_assignment_id: Uuid = sqlx::query_scalar(
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
        workflows::ensure_workflow_assignment_for_form_assignment(&state.pool, form_assignment_id)
            .await?
    } else {
        let delegate_account_id = auth::resolve_accessible_delegate_account_id(
            &state.pool,
            &account,
            payload.delegate_account_id,
        )
        .await?;

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
        .bind(payload.form_version_id)
        .bind(payload.node_id)
        .bind(delegate_account_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError::Forbidden("submissions:write".into()))?
    };
    let submission_id =
        workflows::start_workflow_assignment(&state.pool, &account, workflow_assignment_id).await?;

    Ok(Json(IdResponse { id: submission_id }))
}

/// Lists submissions for the current local workflow shell.
pub async fn list_submissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListSubmissionsQuery>,
) -> ApiResult<Json<Vec<SubmissionSummary>>> {
    let account = auth::require_authenticated(&state.pool, &headers).await?;
    let status = service::parse_submission_status_filter(query.status)?;
    let search = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let rows = if account.is_admin() {
        sqlx::query(
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
          AND (
              $4::text IS NULL
              OR forms.name ILIKE '%' || $4 || '%'
              OR nodes.name ILIKE '%' || $4 || '%'
              OR form_versions.version_label ILIKE '%' || $4 || '%'
          )
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
        .bind(search)
        .fetch_all(&state.pool)
        .await?
    } else if account.is_operator() {
        let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
        sqlx::query(
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
            WHERE submissions.node_id = ANY($1)
              AND ($2::submission_status IS NULL OR submissions.status = $2::submission_status)
              AND ($3::uuid IS NULL OR forms.id = $3)
              AND ($4::uuid IS NULL OR submissions.node_id = $4)
              AND (
                  $5::text IS NULL
                  OR forms.name ILIKE '%' || $5 || '%'
                  OR nodes.name ILIKE '%' || $5 || '%'
                  OR form_versions.version_label ILIKE '%' || $5 || '%'
              )
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
        .bind(scope_ids)
        .bind(status)
        .bind(query.form_id)
        .bind(query.node_id)
        .bind(search)
        .fetch_all(&state.pool)
        .await?
    } else {
        let delegate_account_id = auth::resolve_accessible_delegate_account_id(
            &state.pool,
            &account,
            query.delegate_account_id,
        )
        .await?;
        sqlx::query(
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
            JOIN workflow_assignments ON workflow_assignments.id = submissions.workflow_assignment_id
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
              )
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
        .bind(delegate_account_id)
        .bind(status)
        .bind(query.form_id)
        .bind(query.node_id)
        .bind(search)
        .fetch_all(&state.pool)
        .await?
    };

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
    let account = auth::require_authenticated(&state.pool, &headers).await?;
    service::require_submission_access(&state.pool, &account, submission_id).await?;

    let row = sqlx::query(
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
        form_id: row.try_get("form_id")?,
        form_version_id: row.try_get("form_version_id")?,
        form_name: row.try_get("form_name")?,
        version_label: row.try_get("version_label")?,
        node_id: row.try_get("node_id")?,
        node_name: row.try_get("node_name")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
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
    let account = auth::require_authenticated(&state.pool, &headers).await?;

    let access = service::require_submission_access(&state.pool, &account, submission_id).await?;
    let form_version_id = service::require_draft_submission_status(
        submission_id,
        &access.status,
        access.form_version_id,
    )?;
    let fields = repo::fields_by_key(&state.pool, form_version_id).await?;

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

    repo::audit_submission(
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
    let account = auth::require_authenticated(&state.pool, &headers).await?;

    let access = service::require_submission_access(&state.pool, &account, submission_id).await?;
    let form_version_id = service::require_draft_submission_status(
        submission_id,
        &access.status,
        access.form_version_id,
    )?;
    let fields = repo::fields_by_key(&state.pool, form_version_id).await?;
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
    .execute(&state.pool)
    .await?;

    repo::audit_submission(
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
    let account = auth::require_authenticated(&state.pool, &headers).await?;
    let access = service::require_submission_access(&state.pool, &account, submission_id).await?;
    service::require_draft_submission_status(
        submission_id,
        &access.status,
        access.form_version_id,
    )?;

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
    .execute(&state.pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM workflow_instances
        WHERE id = (
            SELECT workflow_instance_id
            FROM submissions
            WHERE id = $1
        )
        "#,
    )
    .bind(submission_id)
    .execute(&state.pool)
    .await?;

    sqlx::query("DELETE FROM submissions WHERE id = $1")
        .bind(submission_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: submission_id }))
}
