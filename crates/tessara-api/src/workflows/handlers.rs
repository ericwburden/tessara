use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use chrono::{DateTime, Utc};
use sqlx::{Postgres, Row, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    auth::{self, AuthenticatedRequest},
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::IdResponse,
};

use super::dto::{
    AssignmentCandidateAssigneeQuery, AssignmentCandidateQuery, BulkWorkflowAssignmentRequest,
    BulkWorkflowAssignmentResponse, BulkWorkflowAssignmentResult, CreateWorkflowAssignmentRequest,
    CreateWorkflowRequest, CreateWorkflowStepRequest, CreateWorkflowVersionRequest,
    PendingWorkflowWork, UpdateWorkflowAssignmentRequest, UpdateWorkflowRequest,
    UpdateWorkflowVersionStepsRequest, WorkflowAssigneeOption, WorkflowAssignmentCandidate,
    WorkflowAssignmentQuery, WorkflowAssignmentSummary, WorkflowDefinition, WorkflowStepSummary,
    WorkflowSummary, WorkflowVersionSummary,
};

pub async fn list_workflows(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<WorkflowSummary>>> {
    auth::require_capability(&state.pool, &headers, "workflows:read").await?;
    Ok(Json(list_workflows_inner(&state.pool).await?))
}

pub async fn get_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_id): Path<Uuid>,
) -> ApiResult<Json<WorkflowDefinition>> {
    auth::require_capability(&state.pool, &headers, "workflows:read").await?;
    Ok(Json(get_workflow_inner(&state.pool, workflow_id).await?))
}

pub async fn create_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateWorkflowRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;
    let form_id = resolve_legacy_workflow_form_id(&state.pool, payload.form_id, None).await?;
    require_workflow_payload(&state.pool, form_id, &payload.name, &payload.slug, None).await?;

    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workflows (form_id, name, slug, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(form_id)
    .bind(payload.name.trim())
    .bind(payload.slug.trim())
    .bind(payload.description.unwrap_or_default().trim())
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

pub async fn update_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_id): Path<Uuid>,
    Json(payload): Json<UpdateWorkflowRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;
    let form_id =
        resolve_legacy_workflow_form_id(&state.pool, payload.form_id, Some(workflow_id)).await?;
    require_workflow_payload(
        &state.pool,
        form_id,
        &payload.name,
        &payload.slug,
        Some(workflow_id),
    )
    .await?;

    let updated = sqlx::query(
        r#"
        UPDATE workflows
        SET form_id = $2,
            name = $3,
            slug = $4,
            description = $5
        WHERE id = $1
        "#,
    )
    .bind(workflow_id)
    .bind(form_id)
    .bind(payload.name.trim())
    .bind(payload.slug.trim())
    .bind(payload.description.unwrap_or_default().trim())
    .execute(&state.pool)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(ApiError::NotFound(format!("workflow {workflow_id}")));
    }

    Ok(Json(IdResponse { id: workflow_id }))
}

pub async fn create_workflow_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_id): Path<Uuid>,
    Json(payload): Json<CreateWorkflowVersionRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;

    let explicit_steps = !payload.steps.is_empty();
    let steps = normalize_workflow_steps(&payload)?;
    let mut tx = state.pool.begin().await?;
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM workflows WHERE id = $1")
        .bind(workflow_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("workflow {workflow_id}")))?;
    let version_row = sqlx::query(
        r#"
        SELECT form_id, version_label, status::text AS status
        FROM form_versions
        WHERE id = $1
        "#,
    )
    .bind(steps[0].form_version_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("form version {}", steps[0].form_version_id)))?;

    if !explicit_steps {
        let existing: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM workflow_versions WHERE workflow_id = $1 AND form_version_id = $2",
        )
        .bind(workflow_id)
        .bind(steps[0].form_version_id)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(id) = existing {
            return Ok(Json(IdResponse { id }));
        }
    }

    let source_status: String = version_row.try_get("status")?;
    let status = if explicit_steps {
        "draft".to_string()
    } else {
        source_status
    };
    let version_label: Option<String> = version_row.try_get("version_label")?;
    if explicit_steps {
        if let Some(existing_draft_id) = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM workflow_versions WHERE workflow_id = $1 AND status = 'draft'::form_version_status ORDER BY created_at DESC LIMIT 1",
        )
        .bind(workflow_id)
        .fetch_optional(&mut *tx)
        .await?
        {
            sqlx::query(
                r#"
                UPDATE workflow_versions
                SET form_version_id = $2,
                    version_label = $3,
                    published_at = NULL
                WHERE id = $1
                "#,
            )
            .bind(existing_draft_id)
            .bind(steps[0].form_version_id)
            .bind(version_label)
            .execute(&mut *tx)
            .await?;
            replace_workflow_steps_tx(&mut tx, existing_draft_id, &steps).await?;
            tx.commit().await?;
            return Ok(Json(IdResponse {
                id: existing_draft_id,
            }));
        }
    }
    let workflow_version_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workflow_versions (workflow_id, form_version_id, version_label, status, published_at)
        VALUES (
            $1,
            $2,
            $3,
            $4::form_version_status,
            CASE WHEN $4 = 'published' THEN now() ELSE NULL END
        )
        RETURNING id
        "#,
    )
    .bind(workflow_id)
    .bind(steps[0].form_version_id)
    .bind(version_label)
    .bind(status)
    .fetch_one(&mut *tx)
    .await?;
    replace_workflow_steps_tx(&mut tx, workflow_version_id, &steps).await?;
    tx.commit().await?;

    Ok(Json(IdResponse {
        id: workflow_version_id,
    }))
}

pub async fn replace_workflow_version_steps(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_version_id): Path<Uuid>,
    Json(payload): Json<UpdateWorkflowVersionStepsRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;
    let steps = normalize_step_collection(payload.steps)?;
    let mut tx = state.pool.begin().await?;
    let status: String =
        sqlx::query_scalar("SELECT status::text FROM workflow_versions WHERE id = $1")
            .bind(workflow_version_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("workflow version {workflow_version_id}")))?;
    if status != "draft" {
        return Err(ApiError::BadRequest(
            "workflow version steps can only be changed while the version is draft".into(),
        ));
    }
    replace_workflow_steps_tx(&mut tx, workflow_version_id, &steps).await?;
    tx.commit().await?;
    Ok(Json(IdResponse {
        id: workflow_version_id,
    }))
}

pub async fn delete_workflow_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_version_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;

    let mut tx = state.pool.begin().await?;
    let status: String =
        sqlx::query_scalar("SELECT status::text FROM workflow_versions WHERE id = $1")
            .bind(workflow_version_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("workflow version {workflow_version_id}")))?;
    if status != "draft" {
        return Err(ApiError::BadRequest(
            "workflow versions can only be deleted while they are draft".into(),
        ));
    }
    sqlx::query("DELETE FROM workflow_versions WHERE id = $1")
        .bind(workflow_version_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(IdResponse {
        id: workflow_version_id,
    }))
}

pub async fn publish_workflow_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_version_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;

    let mut tx = state.pool.begin().await?;
    let row = sqlx::query(
        r#"
        SELECT
            workflow_versions.workflow_id,
            workflow_versions.form_version_id,
            form_versions.status::text AS form_status
        FROM workflow_versions
        JOIN form_versions ON form_versions.id = workflow_versions.form_version_id
        WHERE workflow_versions.id = $1
        "#,
    )
    .bind(workflow_version_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("workflow version {workflow_version_id}")))?;
    let workflow_id: Uuid = row.try_get("workflow_id")?;
    let form_status: String = row.try_get("form_status")?;
    if form_status != "published" {
        return Err(ApiError::BadRequest(
            "workflow versions can only be published when every step form version is published"
                .into(),
        ));
    }
    validate_workflow_version_publish_tx(&mut tx, workflow_version_id).await?;

    sqlx::query(
        r#"
        UPDATE workflow_versions
        SET status = 'superseded'::form_version_status
        WHERE workflow_id = $1
          AND id <> $2
          AND status = 'published'::form_version_status
        "#,
    )
    .bind(workflow_id)
    .bind(workflow_version_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE workflow_versions
        SET status = 'published'::form_version_status,
            published_at = now()
        WHERE id = $1
        "#,
    )
    .bind(workflow_version_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(IdResponse {
        id: workflow_version_id,
    }))
}

pub async fn list_workflow_assignments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WorkflowAssignmentQuery>,
) -> ApiResult<Json<Vec<WorkflowAssignmentSummary>>> {
    auth::require_capability(&state.pool, &headers, "workflows:read").await?;
    Ok(Json(
        list_workflow_assignments_inner(&state.pool, &query).await?,
    ))
}

pub async fn create_workflow_assignment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateWorkflowAssignmentRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "workflows:write").await?;
    ensure_assignment_candidate_is_valid(
        &state.pool,
        &account,
        payload.workflow_version_id,
        payload.node_id,
    )
    .await?;
    let mut tx = state.pool.begin().await?;
    let id = ensure_workflow_assignment_tx(
        &mut tx,
        payload.workflow_version_id,
        payload.node_id,
        payload.account_id,
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(IdResponse { id }))
}

pub async fn list_assignment_candidates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AssignmentCandidateQuery>,
) -> ApiResult<Json<Vec<WorkflowAssignmentCandidate>>> {
    let account = auth::require_capability(&state.pool, &headers, "workflows:write").await?;
    Ok(Json(
        list_assignment_candidates_inner(&state.pool, &account, query.node_id, query.q.as_deref())
            .await?,
    ))
}

pub async fn list_assignment_candidate_assignees(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AssignmentCandidateAssigneeQuery>,
) -> ApiResult<Json<Vec<WorkflowAssigneeOption>>> {
    let account = auth::require_capability(&state.pool, &headers, "workflows:write").await?;
    ensure_assignment_candidate_is_valid(
        &state.pool,
        &account,
        query.workflow_version_id,
        query.node_id,
    )
    .await?;
    Ok(Json(
        list_assignee_options_inner(&state.pool, query.node_id).await?,
    ))
}

pub async fn bulk_create_workflow_assignments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<BulkWorkflowAssignmentRequest>,
) -> ApiResult<Json<BulkWorkflowAssignmentResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "workflows:write").await?;
    if payload.account_ids.is_empty() {
        return Err(ApiError::BadRequest(
            "bulk workflow assignment requires at least one assignee".into(),
        ));
    }
    ensure_assignment_candidate_is_valid(
        &state.pool,
        &account,
        payload.workflow_version_id,
        payload.node_id,
    )
    .await?;
    let valid_assignees = list_assignee_options_inner(&state.pool, payload.node_id).await?;
    let mut results = Vec::with_capacity(payload.account_ids.len());
    let mut tx = state.pool.begin().await?;
    for account_id in payload.account_ids {
        let assignee = valid_assignees
            .iter()
            .find(|option| option.account_id == account_id)
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "account {account_id} is not eligible for this workflow assignment"
                ))
            })?;
        let (workflow_assignment_id, status) = ensure_workflow_assignment_with_status_tx(
            &mut tx,
            payload.workflow_version_id,
            payload.node_id,
            account_id,
            None,
        )
        .await?;
        results.push(BulkWorkflowAssignmentResult {
            account_id,
            email: assignee.email.clone(),
            display_name: assignee.display_name.clone(),
            status,
            workflow_assignment_id,
        });
    }
    tx.commit().await?;
    Ok(Json(BulkWorkflowAssignmentResponse { results }))
}

pub async fn update_workflow_assignment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_assignment_id): Path<Uuid>,
    Json(payload): Json<UpdateWorkflowAssignmentRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "workflows:write").await?;
    let current = sqlx::query(
        r#"
        SELECT workflow_version_id, form_assignment_id
        FROM workflow_assignments
        WHERE id = $1
        "#,
    )
    .bind(workflow_assignment_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("workflow assignment {workflow_assignment_id}")))?;
    let workflow_version_id: Uuid = current.try_get("workflow_version_id")?;
    let form_assignment_id: Option<Uuid> = current.try_get("form_assignment_id")?;
    let step_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM workflow_steps WHERE workflow_version_id = $1 ORDER BY position LIMIT 1",
    )
    .bind(workflow_version_id)
    .fetch_one(&state.pool)
    .await?;
    ensure_assignment_candidate_is_valid(
        &state.pool,
        &account,
        workflow_version_id,
        payload.node_id,
    )
    .await?;

    sqlx::query(
        r#"
        UPDATE workflow_assignments
        SET node_id = $2,
            account_id = $3,
            workflow_step_id = $4,
            is_active = $5
        WHERE id = $1
        "#,
    )
    .bind(workflow_assignment_id)
    .bind(payload.node_id)
    .bind(payload.account_id)
    .bind(step_id)
    .bind(payload.is_active)
    .execute(&state.pool)
    .await?;

    if let Some(form_assignment_id) = form_assignment_id {
        sqlx::query(
            r#"
            UPDATE form_assignments
            SET node_id = $2,
                account_id = $3
            WHERE id = $1
            "#,
        )
        .bind(form_assignment_id)
        .bind(payload.node_id)
        .bind(payload.account_id)
        .execute(&state.pool)
        .await?;
    }

    Ok(Json(IdResponse {
        id: workflow_assignment_id,
    }))
}

pub async fn list_pending_work(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Query(query): Query<WorkflowAssignmentQuery>,
) -> ApiResult<Json<Vec<PendingWorkflowWork>>> {
    let delegate_account_id = auth::resolve_accessible_delegate_account_id(
        &state.pool,
        &request.account,
        query.delegate_account_id,
    )
    .await?;
    Ok(Json(
        list_pending_assignments_for_account(&state.pool, delegate_account_id).await?,
    ))
}

pub async fn start_assignment(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(workflow_assignment_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    let id =
        start_workflow_assignment(&state.pool, &request.account, workflow_assignment_id).await?;
    Ok(Json(IdResponse { id }))
}

pub async fn ensure_workflow_for_published_form_version_tx(
    tx: &mut Transaction<'_, Postgres>,
    form_version_id: Uuid,
) -> ApiResult<(Uuid, Uuid, Uuid)> {
    let row = sqlx::query(
        r#"
        SELECT
            forms.id AS form_id,
            forms.name AS form_name,
            forms.slug AS form_slug,
            form_versions.version_label,
            form_versions.status::text AS status,
            form_versions.published_at
        FROM form_versions
        JOIN forms ON forms.id = form_versions.form_id
        WHERE form_versions.id = $1
        "#,
    )
    .bind(form_version_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("form version {form_version_id}")))?;

    let form_id: Uuid = row.try_get("form_id")?;
    let form_name: String = row.try_get("form_name")?;
    let form_slug: String = row.try_get("form_slug")?;
    let default_workflow_slug = format!("{form_slug}-workflow");
    let workflow_id: Uuid = if let Some(existing) =
        sqlx::query_scalar("SELECT id FROM workflows WHERE slug = $1")
            .bind(&default_workflow_slug)
            .fetch_optional(&mut **tx)
            .await?
    {
        existing
    } else {
        sqlx::query_scalar(
            r#"
            INSERT INTO workflows (form_id, name, slug, description)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(form_id)
        .bind(format!("{form_name} Workflow"))
        .bind(default_workflow_slug)
        .bind("Generated for Sprint 2A single-step runtime compatibility.")
        .fetch_one(&mut **tx)
        .await?
    };

    let status: String = row.try_get("status")?;
    let version_label: Option<String> = row.try_get("version_label")?;
    let published_at: Option<DateTime<Utc>> = row.try_get("published_at")?;
    let workflow_version_id: Uuid = if let Some(existing) = sqlx::query_scalar(
        "SELECT id FROM workflow_versions WHERE workflow_id = $1 AND form_version_id = $2",
    )
    .bind(workflow_id)
    .bind(form_version_id)
    .fetch_optional(&mut **tx)
    .await?
    {
        sqlx::query(
            r#"
            UPDATE workflow_versions
            SET version_label = $2,
                status = $3::form_version_status,
                published_at = $4
            WHERE id = $1
            "#,
        )
        .bind(existing)
        .bind(version_label)
        .bind(status)
        .bind(published_at)
        .execute(&mut **tx)
        .await?;
        existing
    } else {
        sqlx::query_scalar(
            r#"
            INSERT INTO workflow_versions (workflow_id, form_version_id, version_label, status, published_at)
            VALUES ($1, $2, $3, $4::form_version_status, $5)
            RETURNING id
            "#,
        )
        .bind(workflow_id)
        .bind(form_version_id)
        .bind(version_label)
        .bind(status)
        .bind(published_at)
        .fetch_one(&mut **tx)
        .await?
    };

    let step_id: Uuid = if let Some(existing) = sqlx::query_scalar(
        "SELECT id FROM workflow_steps WHERE workflow_version_id = $1 AND position = 0",
    )
    .bind(workflow_version_id)
    .fetch_optional(&mut **tx)
    .await?
    {
        sqlx::query(
            r#"
            UPDATE workflow_steps
            SET form_version_id = $2,
                title = $3
            WHERE id = $1
            "#,
        )
        .bind(existing)
        .bind(form_version_id)
        .bind(format!("{form_name} Response"))
        .execute(&mut **tx)
        .await?;
        existing
    } else {
        sqlx::query_scalar(
            r#"
            INSERT INTO workflow_steps (workflow_version_id, form_version_id, title, position)
            VALUES ($1, $2, $3, 0)
            RETURNING id
            "#,
        )
        .bind(workflow_version_id)
        .bind(form_version_id)
        .bind(format!("{form_name} Response"))
        .fetch_one(&mut **tx)
        .await?
    };

    Ok((workflow_id, workflow_version_id, step_id))
}

pub async fn ensure_workflow_assignment_for_form_assignment(
    pool: &sqlx::PgPool,
    form_assignment_id: Uuid,
) -> ApiResult<Uuid> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"
        SELECT form_version_id, node_id, account_id
        FROM form_assignments
        WHERE id = $1
        "#,
    )
    .bind(form_assignment_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("form assignment {form_assignment_id}")))?;
    let form_version_id: Uuid = row.try_get("form_version_id")?;
    let node_id: Uuid = row.try_get("node_id")?;
    let account_id: Option<Uuid> = row.try_get("account_id")?;
    let account_id = account_id.ok_or_else(|| {
        ApiError::BadRequest(
            "cannot convert an unassigned form assignment into workflow work".into(),
        )
    })?;
    let (_, workflow_version_id, _) =
        ensure_workflow_for_published_form_version_tx(&mut tx, form_version_id).await?;
    let workflow_assignment_id = ensure_workflow_assignment_tx(
        &mut tx,
        workflow_version_id,
        node_id,
        account_id,
        Some(form_assignment_id),
    )
    .await?;
    tx.commit().await?;
    Ok(workflow_assignment_id)
}

pub async fn ensure_submission_runtime_linkage(
    pool: &sqlx::PgPool,
    submission_id: Uuid,
    workflow_assignment_id: Uuid,
    started_by_account_id: Uuid,
    is_completed: bool,
) -> ApiResult<()> {
    let mut tx = pool.begin().await?;
    let assignment_row = sqlx::query(
        r#"
        SELECT
            workflow_assignments.workflow_version_id,
            workflow_assignments.workflow_step_id,
            workflow_assignments.node_id,
            workflow_assignments.account_id
        FROM workflow_assignments
        WHERE workflow_assignments.id = $1
        "#,
    )
    .bind(workflow_assignment_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("workflow assignment {workflow_assignment_id}")))?;

    let existing_runtime = sqlx::query(
        r#"
        SELECT workflow_instance_id, workflow_step_instance_id
        FROM submissions
        WHERE id = $1
        "#,
    )
    .bind(submission_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("submission {submission_id}")))?;

    let workflow_version_id: Uuid = assignment_row.try_get("workflow_version_id")?;
    let workflow_step_id: Uuid = assignment_row.try_get("workflow_step_id")?;
    let node_id: Uuid = assignment_row.try_get("node_id")?;
    let assignee_account_id: Uuid = assignment_row.try_get("account_id")?;
    let completed_at = if is_completed { Some(Utc::now()) } else { None };

    let workflow_instance_id: Uuid = if let Some(existing) =
        existing_runtime.try_get::<Option<Uuid>, _>("workflow_instance_id")?
    {
        existing
    } else {
        sqlx::query_scalar(
            r#"
            INSERT INTO workflow_instances (
                workflow_assignment_id,
                workflow_version_id,
                node_id,
                assignee_account_id,
                started_by_account_id
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(workflow_assignment_id)
        .bind(workflow_version_id)
        .bind(node_id)
        .bind(assignee_account_id)
        .bind(started_by_account_id)
        .fetch_one(&mut *tx)
        .await?
    };

    let workflow_step_instance_id: Uuid = if let Some(existing) =
        existing_runtime.try_get::<Option<Uuid>, _>("workflow_step_instance_id")?
    {
        sqlx::query(
            r#"
            UPDATE workflow_step_instances
            SET status = $2,
                completed_at = $3
            WHERE id = $1
            "#,
        )
        .bind(existing)
        .bind(if is_completed {
            "completed"
        } else {
            "in_progress"
        })
        .bind(completed_at)
        .execute(&mut *tx)
        .await?;
        existing
    } else {
        sqlx::query_scalar(
            r#"
            INSERT INTO workflow_step_instances (
                workflow_instance_id,
                workflow_step_id,
                submission_id,
                status,
                completed_at
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(workflow_instance_id)
        .bind(workflow_step_id)
        .bind(submission_id)
        .bind(if is_completed {
            "completed"
        } else {
            "in_progress"
        })
        .bind(completed_at)
        .fetch_one(&mut *tx)
        .await?
    };

    sqlx::query(
        r#"
        UPDATE submissions
        SET workflow_assignment_id = $2,
            workflow_instance_id = $3,
            workflow_step_instance_id = $4
        WHERE id = $1
        "#,
    )
    .bind(submission_id)
    .bind(workflow_assignment_id)
    .bind(workflow_instance_id)
    .bind(workflow_step_instance_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn complete_workflow_step_and_advance(
    pool: &sqlx::PgPool,
    submission_id: Uuid,
) -> ApiResult<()> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"
        SELECT
            submissions.workflow_instance_id,
            submissions.workflow_step_instance_id,
            workflow_instances.workflow_version_id,
            workflow_instances.node_id,
            workflow_instances.assignee_account_id,
            workflow_steps.position
        FROM submissions
        JOIN workflow_instances ON workflow_instances.id = submissions.workflow_instance_id
        JOIN workflow_step_instances ON workflow_step_instances.id = submissions.workflow_step_instance_id
        JOIN workflow_steps ON workflow_steps.id = workflow_step_instances.workflow_step_id
        WHERE submissions.id = $1
        "#,
    )
    .bind(submission_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(row) = row else {
        tx.commit().await?;
        return Ok(());
    };

    let workflow_instance_id: Uuid = row.try_get("workflow_instance_id")?;
    let workflow_step_instance_id: Uuid = row.try_get("workflow_step_instance_id")?;
    let workflow_version_id: Uuid = row.try_get("workflow_version_id")?;
    let node_id: Uuid = row.try_get("node_id")?;
    let assignee_account_id: Uuid = row.try_get("assignee_account_id")?;
    let position: i32 = row.try_get("position")?;

    sqlx::query(
        r#"
        UPDATE workflow_step_instances
        SET status = 'completed',
            completed_at = now()
        WHERE id = $1
        "#,
    )
    .bind(workflow_step_instance_id)
    .execute(&mut *tx)
    .await?;

    let next_step_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM workflow_steps
        WHERE workflow_version_id = $1
          AND position = $2
        "#,
    )
    .bind(workflow_version_id)
    .bind(position + 1)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(next_step_id) = next_step_id {
        ensure_specific_workflow_assignment_tx(
            &mut tx,
            workflow_version_id,
            next_step_id,
            node_id,
            assignee_account_id,
            None,
        )
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE workflow_instances
            SET status = 'completed'
            WHERE id = $1
            "#,
        )
        .bind(workflow_instance_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn start_workflow_assignment(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    workflow_assignment_id: Uuid,
) -> ApiResult<Uuid> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"
        SELECT
            workflow_assignments.id,
            workflow_assignments.workflow_version_id,
            workflow_assignments.workflow_step_id,
            workflow_assignments.node_id,
            workflow_assignments.account_id,
            workflow_assignments.form_assignment_id,
            workflow_assignments.is_active,
            workflow_steps.form_version_id,
            workflow_steps.position AS workflow_step_position,
            workflow_versions.status::text AS workflow_status
        FROM workflow_assignments
        JOIN workflow_steps ON workflow_steps.id = workflow_assignments.workflow_step_id
        JOIN workflow_versions ON workflow_versions.id = workflow_assignments.workflow_version_id
        WHERE workflow_assignments.id = $1
        "#,
    )
    .bind(workflow_assignment_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("workflow assignment {workflow_assignment_id}")))?;

    let assignee_account_id: Uuid = row.try_get("account_id")?;
    super::service::ensure_can_start_assignment(pool, account, workflow_assignment_id).await?;

    let is_active: bool = row.try_get("is_active")?;
    if !is_active {
        return Err(ApiError::BadRequest(
            "inactive workflow assignments cannot start new response work".into(),
        ));
    }

    let workflow_status: String = row.try_get("workflow_status")?;
    if workflow_status != "published" {
        return Err(ApiError::BadRequest(
            "only published workflow versions can start response work".into(),
        ));
    }

    let existing_draft_submission: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM submissions
        WHERE workflow_assignment_id = $1
          AND status = 'draft'::submission_status
        ORDER BY created_at
        LIMIT 1
        "#,
    )
    .bind(workflow_assignment_id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(existing_submission) = existing_draft_submission {
        return Ok(existing_submission);
    }

    let has_submitted_submission: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM submissions
            WHERE workflow_assignment_id = $1
              AND status = 'submitted'::submission_status
        )
        "#,
    )
    .bind(workflow_assignment_id)
    .fetch_one(&mut *tx)
    .await?;
    if has_submitted_submission {
        return Err(ApiError::BadRequest(
            "submitted workflow assignments cannot start new response work".into(),
        ));
    }

    let workflow_version_id: Uuid = row.try_get("workflow_version_id")?;
    let workflow_step_id: Uuid = row.try_get("workflow_step_id")?;
    let node_id: Uuid = row.try_get("node_id")?;
    let form_version_id: Uuid = row.try_get("form_version_id")?;
    let form_assignment_id: Option<Uuid> = row.try_get("form_assignment_id")?;
    let step_node_id = resolve_workflow_step_node_tx(&mut tx, node_id, form_version_id).await?;
    let workflow_step_position: i32 = row.try_get("workflow_step_position")?;

    let in_progress_instance_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM workflow_instances
        WHERE workflow_version_id = $1
          AND node_id = $2
          AND assignee_account_id = $3
          AND status = 'in_progress'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(workflow_version_id)
    .bind(node_id)
    .bind(assignee_account_id)
    .fetch_optional(&mut *tx)
    .await?;

    if workflow_step_position > 0 {
        let Some(instance_id) = in_progress_instance_id else {
            return Err(ApiError::BadRequest(
                "workflow steps must be completed in order".into(),
            ));
        };
        let previous_step_completed: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM workflow_step_instances
                JOIN workflow_steps
                    ON workflow_steps.id = workflow_step_instances.workflow_step_id
                WHERE workflow_step_instances.workflow_instance_id = $1
                  AND workflow_steps.workflow_version_id = $2
                  AND workflow_steps.position = $3
                  AND workflow_step_instances.status = 'completed'
            )
            "#,
        )
        .bind(instance_id)
        .bind(workflow_version_id)
        .bind(workflow_step_position - 1)
        .fetch_one(&mut *tx)
        .await?;
        if !previous_step_completed {
            return Err(ApiError::BadRequest(
                "workflow steps must be completed in order".into(),
            ));
        }
    }

    let form_assignment_id = if let Some(existing) = form_assignment_id {
        existing
    } else {
        sqlx::query_scalar(
            r#"
            INSERT INTO form_assignments (form_version_id, node_id, account_id)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(form_version_id)
        .bind(step_node_id)
        .bind(assignee_account_id)
        .fetch_one(&mut *tx)
        .await?
    };

    let workflow_instance_id: Uuid = if let Some(existing) = in_progress_instance_id {
        existing
    } else {
        sqlx::query_scalar(
            r#"
        INSERT INTO workflow_instances (
            workflow_assignment_id,
            workflow_version_id,
            node_id,
            assignee_account_id,
            started_by_account_id
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        )
        .bind(workflow_assignment_id)
        .bind(workflow_version_id)
        .bind(node_id)
        .bind(assignee_account_id)
        .bind(account.account_id)
        .fetch_one(&mut *tx)
        .await?
    };

    let submission_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO submissions (
            assignment_id,
            form_version_id,
            node_id,
            workflow_assignment_id,
            workflow_instance_id
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(form_assignment_id)
    .bind(form_version_id)
    .bind(step_node_id)
    .bind(workflow_assignment_id)
    .bind(workflow_instance_id)
    .fetch_one(&mut *tx)
    .await?;

    let workflow_step_instance_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workflow_step_instances (
            workflow_instance_id,
            workflow_step_id,
            submission_id,
            status
        )
        VALUES ($1, $2, $3, 'in_progress')
        RETURNING id
        "#,
    )
    .bind(workflow_instance_id)
    .bind(workflow_step_id)
    .bind(submission_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE submissions
        SET workflow_step_instance_id = $2
        WHERE id = $1
        "#,
    )
    .bind(submission_id)
    .bind(workflow_step_instance_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO submission_audit_events (submission_id, event_type, account_id)
        VALUES ($1, 'create_draft', $2)
        "#,
    )
    .bind(submission_id)
    .bind(account.account_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(submission_id)
}

pub async fn list_pending_assignments_for_account(
    pool: &sqlx::PgPool,
    account_id: Uuid,
) -> ApiResult<Vec<PendingWorkflowWork>> {
    let rows = sqlx::query(
        r#"
        SELECT
            workflow_assignments.id AS workflow_assignment_id,
            workflows.id AS workflow_id,
            workflows.name AS workflow_name,
            workflows.description AS workflow_description,
            workflow_versions.id AS workflow_version_id,
            workflow_versions.version_label AS workflow_version_label,
            workflow_steps.title AS workflow_step_title,
            workflow_steps.position AS workflow_step_position,
            (
                SELECT COUNT(*)
                FROM workflow_steps AS all_steps
                WHERE all_steps.workflow_version_id = workflow_steps.workflow_version_id
            ) AS workflow_step_count,
            next_steps.title AS next_workflow_step_title,
            next_forms.name AS next_workflow_step_form_name,
            forms.id AS form_id,
            forms.name AS form_name,
            workflow_steps.form_version_id,
            form_versions.version_label AS form_version_label,
            nodes.id AS node_id,
            nodes.name AS node_name,
            accounts.id AS account_id,
            accounts.display_name AS account_display_name,
            workflow_assignments.created_at AS assigned_at
        FROM workflow_assignments
        JOIN workflow_versions ON workflow_versions.id = workflow_assignments.workflow_version_id
        JOIN workflows ON workflows.id = workflow_versions.workflow_id
        JOIN workflow_steps ON workflow_steps.id = workflow_assignments.workflow_step_id
        JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
        JOIN forms ON forms.id = form_versions.form_id
        LEFT JOIN workflow_steps AS next_steps
            ON next_steps.workflow_version_id = workflow_steps.workflow_version_id
           AND next_steps.position = workflow_steps.position + 1
        LEFT JOIN form_versions AS next_form_versions
            ON next_form_versions.id = next_steps.form_version_id
        LEFT JOIN forms AS next_forms
            ON next_forms.id = next_form_versions.form_id
        JOIN nodes ON nodes.id = workflow_assignments.node_id
        JOIN accounts ON accounts.id = workflow_assignments.account_id
        WHERE workflow_assignments.account_id = $1
          AND workflow_assignments.is_active = true
          AND workflow_versions.status = 'published'::form_version_status
          AND NOT EXISTS (
              SELECT 1
              FROM submissions
              WHERE submissions.workflow_assignment_id = workflow_assignments.id
                AND submissions.status = 'draft'::submission_status
          )
          AND NOT EXISTS (
              SELECT 1
              FROM submissions
              WHERE submissions.workflow_assignment_id = workflow_assignments.id
                AND submissions.status = 'submitted'::submission_status
          )
        ORDER BY workflows.name, nodes.name, workflow_assignments.created_at
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(PendingWorkflowWork {
                workflow_assignment_id: row.try_get("workflow_assignment_id")?,
                workflow_id: row.try_get("workflow_id")?,
                workflow_name: row.try_get("workflow_name")?,
                workflow_description: row.try_get("workflow_description")?,
                workflow_version_id: row.try_get("workflow_version_id")?,
                workflow_version_label: row.try_get("workflow_version_label")?,
                workflow_step_title: row.try_get("workflow_step_title")?,
                workflow_step_position: row.try_get("workflow_step_position")?,
                workflow_step_count: row.try_get("workflow_step_count")?,
                next_workflow_step_title: row.try_get("next_workflow_step_title")?,
                next_workflow_step_form_name: row.try_get("next_workflow_step_form_name")?,
                form_id: row.try_get("form_id")?,
                form_name: row.try_get("form_name")?,
                form_version_id: row.try_get("form_version_id")?,
                form_version_label: row.try_get("form_version_label")?,
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                account_id: row.try_get("account_id")?,
                account_display_name: row.try_get("account_display_name")?,
                assigned_at: row.try_get("assigned_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

async fn list_workflows_inner(pool: &sqlx::PgPool) -> ApiResult<Vec<WorkflowSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT
            workflows.id,
            workflows.form_id,
            forms.name AS form_name,
            forms.slug AS form_slug,
            workflows.name,
            workflows.slug,
            workflows.description,
            current_versions.id AS current_version_id,
            current_versions.version_label AS current_version_label,
            current_versions.form_version_id AS current_form_version_id,
            current_versions.status::text AS current_status,
            COUNT(DISTINCT workflow_assignments.id) FILTER (WHERE workflow_assignments.is_active) AS assignment_count,
            COUNT(DISTINCT workflow_versions.id) AS version_count,
            COALESCE(
                array_remove(array_agg(DISTINCT nodes.name) FILTER (WHERE workflow_assignments.is_active), NULL),
                ARRAY[]::text[]
            ) AS assignment_node_names
        FROM workflows
        JOIN forms ON forms.id = workflows.form_id
        LEFT JOIN LATERAL (
            SELECT id, form_version_id, version_label, status
            FROM workflow_versions
            WHERE workflow_id = workflows.id
            ORDER BY
                CASE status
                    WHEN 'published' THEN 0
                    WHEN 'draft' THEN 1
                    ELSE 2
                END,
                created_at DESC
            LIMIT 1
        ) AS current_versions ON true
        LEFT JOIN workflow_versions ON workflow_versions.workflow_id = workflows.id
        LEFT JOIN workflow_assignments
            ON workflow_assignments.workflow_version_id = workflow_versions.id
        LEFT JOIN nodes ON nodes.id = workflow_assignments.node_id
        GROUP BY
            workflows.id,
            workflows.form_id,
            forms.name,
            forms.slug,
            workflows.name,
            workflows.slug,
            workflows.description,
            current_versions.id,
            current_versions.version_label,
            current_versions.form_version_id,
            current_versions.status
        ORDER BY workflows.name, workflows.slug
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(WorkflowSummary {
                id: row.try_get("id")?,
                form_id: row.try_get("form_id")?,
                form_name: row.try_get("form_name")?,
                form_slug: row.try_get("form_slug")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                description: row.try_get("description")?,
                current_version_id: row.try_get("current_version_id")?,
                current_version_label: row.try_get("current_version_label")?,
                current_form_version_id: row.try_get("current_form_version_id")?,
                current_status: row.try_get("current_status")?,
                assignment_count: row.try_get("assignment_count")?,
                version_count: row.try_get("version_count")?,
                assignment_node_names: row.try_get("assignment_node_names")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

async fn get_workflow_inner(
    pool: &sqlx::PgPool,
    workflow_id: Uuid,
) -> ApiResult<WorkflowDefinition> {
    let row = sqlx::query(
        r#"
        SELECT
            workflows.id,
            workflows.form_id,
            forms.name AS form_name,
            forms.slug AS form_slug,
            workflows.name,
            workflows.slug,
            workflows.description
        FROM workflows
        JOIN forms ON forms.id = workflows.form_id
        WHERE workflows.id = $1
        "#,
    )
    .bind(workflow_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("workflow {workflow_id}")))?;

    let version_rows = sqlx::query(
        r#"
        SELECT
            workflow_versions.id,
            workflow_versions.form_version_id,
            workflow_versions.version_label AS form_version_label,
            workflow_versions.status::text AS status,
            workflow_versions.published_at,
            workflow_versions.created_at
        FROM workflow_versions
        WHERE workflow_versions.workflow_id = $1
        ORDER BY workflow_versions.created_at DESC, workflow_versions.id DESC
        "#,
    )
    .bind(workflow_id)
    .fetch_all(pool)
    .await?;
    let mut versions = Vec::with_capacity(version_rows.len());
    for version_row in version_rows {
        let version_id: Uuid = version_row.try_get("id")?;
        let steps = load_workflow_steps(pool, version_id).await?;
        let title = steps
            .first()
            .map(|step| step.title.clone())
            .unwrap_or_else(|| "Primary Response".into());
        versions.push(WorkflowVersionSummary {
            id: version_id,
            form_version_id: version_row.try_get("form_version_id")?,
            form_version_label: version_row.try_get("form_version_label")?,
            title,
            status: version_row.try_get("status")?,
            published_at: version_row.try_get("published_at")?,
            created_at: version_row.try_get("created_at")?,
            step_count: steps.len() as i64,
            steps,
        });
    }

    let assignments = list_workflow_assignments_inner(
        pool,
        &WorkflowAssignmentQuery {
            workflow_id: Some(workflow_id),
            ..Default::default()
        },
    )
    .await?;

    Ok(WorkflowDefinition {
        id: row.try_get("id")?,
        form_id: row.try_get("form_id")?,
        form_name: row.try_get("form_name")?,
        form_slug: row.try_get("form_slug")?,
        name: row.try_get("name")?,
        slug: row.try_get("slug")?,
        description: row.try_get("description")?,
        versions,
        assignments,
    })
}

async fn load_workflow_steps(
    pool: &sqlx::PgPool,
    workflow_version_id: Uuid,
) -> ApiResult<Vec<WorkflowStepSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT
            workflow_steps.id,
            forms.id AS form_id,
            forms.name AS form_name,
            workflow_steps.form_version_id,
            form_versions.version_label AS form_version_label,
            workflow_steps.title,
            workflow_steps.position
        FROM workflow_steps
        JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
        JOIN forms ON forms.id = form_versions.form_id
        WHERE workflow_steps.workflow_version_id = $1
        ORDER BY workflow_steps.position
        "#,
    )
    .bind(workflow_version_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(WorkflowStepSummary {
                id: row.try_get("id")?,
                form_id: row.try_get("form_id")?,
                form_name: row.try_get("form_name")?,
                form_version_id: row.try_get("form_version_id")?,
                form_version_label: row.try_get("form_version_label")?,
                title: row.try_get("title")?,
                position: row.try_get("position")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

async fn list_assignment_candidates_inner(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    node_id: Option<Uuid>,
    search: Option<&str>,
) -> ApiResult<Vec<WorkflowAssignmentCandidate>> {
    let scope_ids = if account.is_operator() {
        Some(auth::effective_scope_node_ids(pool, account.account_id).await?)
    } else {
        None
    };
    let rows = sqlx::query(
        r#"
        WITH RECURSIVE node_descendants AS (
            SELECT
                nodes.id AS root_id,
                nodes.id AS node_id,
                nodes.node_type_id,
                0 AS depth
            FROM nodes
            UNION ALL
            SELECT
                node_descendants.root_id,
                child_nodes.id AS node_id,
                child_nodes.node_type_id,
                node_descendants.depth + 1 AS depth
            FROM node_descendants
            JOIN nodes AS child_nodes ON child_nodes.parent_node_id = node_descendants.node_id
        ),
        node_type_lineage AS (
            SELECT
                node_types.id AS ancestor_node_type_id,
                node_types.id AS descendant_node_type_id,
                0 AS depth
            FROM node_types
            UNION ALL
            SELECT
                node_type_lineage.ancestor_node_type_id,
                node_type_relationships.child_node_type_id AS descendant_node_type_id,
                node_type_lineage.depth + 1 AS depth
            FROM node_type_lineage
            JOIN node_type_relationships
                ON node_type_relationships.parent_node_type_id = node_type_lineage.descendant_node_type_id
        ),
        step_totals AS (
            SELECT
                workflow_steps.workflow_version_id,
                COUNT(*) AS step_count
            FROM workflow_steps
            GROUP BY workflow_steps.workflow_version_id
        ),
        step_scopes AS (
            SELECT DISTINCT
                workflow_steps.workflow_version_id,
                forms.scope_node_type_id
            FROM workflow_steps
            JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
            JOIN forms ON forms.id = form_versions.form_id
            WHERE forms.scope_node_type_id IS NOT NULL
        ),
        workflow_anchor AS (
            SELECT
                scoped.workflow_version_id,
                scoped.scope_node_type_id AS anchor_node_type_id
            FROM step_scopes AS scoped
            WHERE NOT EXISTS (
                SELECT 1
                FROM step_scopes AS other_scoped
                JOIN node_type_lineage
                    ON node_type_lineage.ancestor_node_type_id = other_scoped.scope_node_type_id
                   AND node_type_lineage.descendant_node_type_id = scoped.scope_node_type_id
                WHERE other_scoped.workflow_version_id = scoped.workflow_version_id
                  AND other_scoped.scope_node_type_id <> scoped.scope_node_type_id
            )
        )
        SELECT
            workflow_versions.id AS workflow_version_id,
            workflows.id AS workflow_id,
            workflows.name AS workflow_name,
            workflow_versions.version_label AS workflow_version_label,
            nodes.id AS node_id,
            nodes.name AS node_name,
            COALESCE(parent_nodes.name || ' / ' || nodes.name, nodes.name) AS node_path,
            step_totals.step_count
        FROM workflow_versions
        JOIN workflows ON workflows.id = workflow_versions.workflow_id
        JOIN step_totals ON step_totals.workflow_version_id = workflow_versions.id
        LEFT JOIN workflow_anchor ON workflow_anchor.workflow_version_id = workflow_versions.id
        JOIN nodes ON ($1::uuid IS NULL OR nodes.id = $1)
        LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
        WHERE workflow_versions.status = 'published'::form_version_status
          AND step_totals.step_count > 0
          AND ($2::uuid[] IS NULL OR nodes.id = ANY($2))
          AND (
              workflow_anchor.anchor_node_type_id IS NULL
              OR nodes.node_type_id = workflow_anchor.anchor_node_type_id
          )
          AND NOT EXISTS (
              SELECT 1
              FROM step_scopes
              WHERE step_scopes.workflow_version_id = workflow_versions.id
                AND NOT EXISTS (
                    SELECT 1
                    FROM node_descendants
                    WHERE node_descendants.root_id = nodes.id
                      AND node_descendants.node_type_id = step_scopes.scope_node_type_id
                )
          )
          AND NOT EXISTS (
              SELECT 1
              FROM workflow_steps AS assigned_steps
              WHERE assigned_steps.workflow_version_id = workflow_versions.id
                AND EXISTS (
                    SELECT 1
                    FROM form_assignments AS any_assignment
                    WHERE any_assignment.form_version_id = assigned_steps.form_version_id
                )
                AND NOT EXISTS (
                    SELECT 1
                    FROM form_assignments AS step_assignment
                    JOIN node_descendants
                        ON node_descendants.root_id = nodes.id
                       AND node_descendants.node_id = step_assignment.node_id
                    WHERE step_assignment.form_version_id = assigned_steps.form_version_id
                )
          )
          AND (
              $3::text IS NULL
              OR workflows.name ILIKE '%' || $3 || '%'
              OR nodes.name ILIKE '%' || $3 || '%'
              OR parent_nodes.name ILIKE '%' || $3 || '%'
          )
        ORDER BY node_path, workflows.name, workflow_versions.created_at DESC
        "#,
    )
    .bind(node_id)
    .bind(scope_ids.as_deref())
    .bind(
        search
            .filter(|value| !value.trim().is_empty())
            .map(str::trim),
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let node_path: String = row.try_get("node_path")?;
            let workflow_name: String = row.try_get("workflow_name")?;
            Ok(WorkflowAssignmentCandidate {
                workflow_version_id: row.try_get("workflow_version_id")?,
                workflow_id: row.try_get("workflow_id")?,
                workflow_name: workflow_name.clone(),
                workflow_version_label: row.try_get("workflow_version_label")?,
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                node_path: node_path.clone(),
                label: format!("{node_path} - {workflow_name}"),
                step_count: row.try_get("step_count")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

async fn ensure_assignment_candidate_is_valid(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    workflow_version_id: Uuid,
    node_id: Uuid,
) -> ApiResult<()> {
    let candidates = list_assignment_candidates_inner(pool, account, Some(node_id), None).await?;
    if candidates
        .iter()
        .any(|candidate| candidate.workflow_version_id == workflow_version_id)
    {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "workflow assignment candidate is not eligible for this node".into(),
        ))
    }
}

async fn list_assignee_options_inner(
    pool: &sqlx::PgPool,
    _node_id: Uuid,
) -> ApiResult<Vec<WorkflowAssigneeOption>> {
    let rows = sqlx::query(
        r#"
        SELECT id AS account_id, email, display_name
        FROM accounts
        WHERE is_active = true
        ORDER BY display_name, email
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(WorkflowAssigneeOption {
                account_id: row.try_get("account_id")?,
                email: row.try_get("email")?,
                display_name: row.try_get("display_name")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

async fn list_workflow_assignments_inner(
    pool: &sqlx::PgPool,
    query: &WorkflowAssignmentQuery,
) -> ApiResult<Vec<WorkflowAssignmentSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT
            workflow_assignments.id,
            workflows.id AS workflow_id,
            workflows.name AS workflow_name,
            workflow_versions.id AS workflow_version_id,
            workflow_versions.version_label AS workflow_version_label,
            forms.id AS form_id,
            forms.name AS form_name,
            workflow_steps.form_version_id,
            form_versions.version_label AS form_version_label,
            workflow_steps.id AS workflow_step_id,
            workflow_steps.title AS workflow_step_title,
            nodes.id AS node_id,
            nodes.name AS node_name,
            accounts.id AS account_id,
            accounts.display_name AS account_display_name,
            accounts.email AS account_email,
            workflow_assignments.is_active,
            EXISTS (
                SELECT 1
                FROM submissions
                WHERE submissions.workflow_assignment_id = workflow_assignments.id
                  AND submissions.status = 'draft'::submission_status
            ) AS has_draft,
            EXISTS (
                SELECT 1
                FROM submissions
                WHERE submissions.workflow_assignment_id = workflow_assignments.id
                  AND submissions.status = 'submitted'::submission_status
            ) AS has_submitted,
            workflow_assignments.created_at
        FROM workflow_assignments
        JOIN workflow_versions ON workflow_versions.id = workflow_assignments.workflow_version_id
        JOIN workflows ON workflows.id = workflow_versions.workflow_id
        JOIN workflow_steps ON workflow_steps.id = workflow_assignments.workflow_step_id
        JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
        JOIN forms ON forms.id = form_versions.form_id
        JOIN nodes ON nodes.id = workflow_assignments.node_id
        JOIN accounts ON accounts.id = workflow_assignments.account_id
        WHERE ($1::uuid IS NULL OR workflows.id = $1)
          AND ($2::uuid IS NULL OR workflow_versions.id = $2)
          AND ($3::uuid IS NULL OR forms.id = $3)
          AND ($4::uuid IS NULL OR workflow_assignments.account_id = $4)
          AND ($5::uuid IS NULL OR workflow_assignments.node_id = $5)
          AND ($6::boolean IS NULL OR workflow_assignments.is_active = $6)
        ORDER BY workflows.name, nodes.name, accounts.display_name, workflow_assignments.created_at
        "#,
    )
    .bind(query.workflow_id)
    .bind(query.workflow_version_id)
    .bind(query.form_id)
    .bind(query.account_id)
    .bind(query.node_id)
    .bind(query.active)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(WorkflowAssignmentSummary {
                id: row.try_get("id")?,
                workflow_id: row.try_get("workflow_id")?,
                workflow_name: row.try_get("workflow_name")?,
                workflow_version_id: row.try_get("workflow_version_id")?,
                workflow_version_label: row.try_get("workflow_version_label")?,
                form_id: row.try_get("form_id")?,
                form_name: row.try_get("form_name")?,
                form_version_id: row.try_get("form_version_id")?,
                form_version_label: row.try_get("form_version_label")?,
                workflow_step_id: row.try_get("workflow_step_id")?,
                workflow_step_title: row.try_get("workflow_step_title")?,
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                account_id: row.try_get("account_id")?,
                account_display_name: row.try_get("account_display_name")?,
                account_email: row.try_get("account_email")?,
                is_active: row.try_get("is_active")?,
                has_draft: row.try_get("has_draft")?,
                has_submitted: row.try_get("has_submitted")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

fn normalize_workflow_steps(
    payload: &CreateWorkflowVersionRequest,
) -> ApiResult<Vec<CreateWorkflowStepRequest>> {
    if payload.steps.is_empty() {
        let form_version_id = payload.form_version_id.ok_or_else(|| {
            ApiError::BadRequest("workflow version requires at least one step".into())
        })?;
        let title = payload
            .title
            .clone()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "Primary Response".into());
        return Ok(vec![CreateWorkflowStepRequest {
            title,
            form_version_id,
        }]);
    }
    normalize_step_collection(payload.steps.clone())
}

fn normalize_step_collection(
    steps: Vec<CreateWorkflowStepRequest>,
) -> ApiResult<Vec<CreateWorkflowStepRequest>> {
    if steps.is_empty() {
        return Err(ApiError::BadRequest(
            "workflow version requires at least one step".into(),
        ));
    }
    steps
        .into_iter()
        .enumerate()
        .map(|(index, step)| {
            let title = step.title.trim().to_string();
            if title.is_empty() {
                return Err(ApiError::BadRequest(format!(
                    "workflow step {} title is required",
                    index + 1
                )));
            }
            Ok(CreateWorkflowStepRequest {
                title,
                form_version_id: step.form_version_id,
            })
        })
        .collect()
}

async fn replace_workflow_steps_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_version_id: Uuid,
    steps: &[CreateWorkflowStepRequest],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM workflow_steps WHERE workflow_version_id = $1")
        .bind(workflow_version_id)
        .execute(&mut **tx)
        .await?;

    for (position, step) in steps.iter().enumerate() {
        let form_version_exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM form_versions WHERE id = $1)")
                .bind(step.form_version_id)
                .fetch_one(&mut **tx)
                .await?;
        if !form_version_exists {
            return Err(ApiError::NotFound(format!(
                "form version {}",
                step.form_version_id
            )));
        }

        sqlx::query(
            r#"
            INSERT INTO workflow_steps (workflow_version_id, form_version_id, title, position)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(workflow_version_id)
        .bind(step.form_version_id)
        .bind(&step.title)
        .bind(position as i32)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn validate_workflow_version_publish_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_version_id: Uuid,
) -> ApiResult<()> {
    let rows = sqlx::query(
        r#"
        SELECT
            workflow_steps.title,
            workflow_steps.position,
            form_versions.status::text AS form_status
        FROM workflow_steps
        JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
        WHERE workflow_steps.workflow_version_id = $1
        ORDER BY workflow_steps.position
        "#,
    )
    .bind(workflow_version_id)
    .fetch_all(&mut **tx)
    .await?;

    if rows.is_empty() {
        return Err(ApiError::BadRequest(
            "workflow versions require at least one step before publish".into(),
        ));
    }

    for (expected_position, row) in rows.iter().enumerate() {
        let title: String = row.try_get("title")?;
        if title.trim().is_empty() {
            return Err(ApiError::BadRequest(format!(
                "workflow step {} title is required",
                expected_position + 1
            )));
        }
        let position: i32 = row.try_get("position")?;
        if position != expected_position as i32 {
            return Err(ApiError::BadRequest(
                "workflow steps must be ordered without gaps before publish".into(),
            ));
        }
        let form_status: String = row.try_get("form_status")?;
        if form_status != "published" {
            return Err(ApiError::BadRequest(
                "workflow steps can only reference published form versions".into(),
            ));
        }
    }

    let has_branching_scope: bool = sqlx::query_scalar(
        r#"
        WITH RECURSIVE node_type_lineage AS (
            SELECT
                node_types.id AS ancestor_node_type_id,
                node_types.id AS descendant_node_type_id
            FROM node_types
            UNION ALL
            SELECT
                node_type_lineage.ancestor_node_type_id,
                node_type_relationships.child_node_type_id AS descendant_node_type_id
            FROM node_type_lineage
            JOIN node_type_relationships
                ON node_type_relationships.parent_node_type_id = node_type_lineage.descendant_node_type_id
        ),
        step_scopes AS (
            SELECT DISTINCT forms.scope_node_type_id
            FROM workflow_steps
            JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
            JOIN forms ON forms.id = form_versions.form_id
            WHERE workflow_steps.workflow_version_id = $1
              AND forms.scope_node_type_id IS NOT NULL
        )
        SELECT EXISTS (
            SELECT 1
            FROM step_scopes AS left_scope
            JOIN step_scopes AS right_scope
                ON left_scope.scope_node_type_id::text < right_scope.scope_node_type_id::text
            WHERE NOT EXISTS (
                SELECT 1
                FROM node_type_lineage
                WHERE node_type_lineage.ancestor_node_type_id = left_scope.scope_node_type_id
                  AND node_type_lineage.descendant_node_type_id = right_scope.scope_node_type_id
            )
              AND NOT EXISTS (
                SELECT 1
                FROM node_type_lineage
                WHERE node_type_lineage.ancestor_node_type_id = right_scope.scope_node_type_id
                  AND node_type_lineage.descendant_node_type_id = left_scope.scope_node_type_id
            )
        )
        "#,
    )
    .bind(workflow_version_id)
    .fetch_one(&mut **tx)
    .await?;
    if has_branching_scope {
        return Err(ApiError::BadRequest(
            "workflow step form scopes must stay on one hierarchy lineage".into(),
        ));
    }
    validate_workflow_step_node_lineage_tx(tx, workflow_version_id).await?;
    Ok(())
}

struct StepLineageOptions {
    form_version_id: Uuid,
    assignment_node_ids: Vec<Uuid>,
}

async fn validate_workflow_step_node_lineage_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_version_id: Uuid,
) -> ApiResult<()> {
    let rows = sqlx::query(
        r#"
        SELECT
            workflow_steps.form_version_id,
            COALESCE(
                array_remove(array_agg(DISTINCT form_assignments.node_id), NULL),
                ARRAY[]::uuid[]
            ) AS assignment_node_ids
        FROM workflow_steps
        JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
        LEFT JOIN form_assignments
            ON form_assignments.form_version_id = workflow_steps.form_version_id
        WHERE workflow_steps.workflow_version_id = $1
        GROUP BY workflow_steps.position, workflow_steps.form_version_id
        ORDER BY workflow_steps.position
        "#,
    )
    .bind(workflow_version_id)
    .fetch_all(&mut **tx)
    .await?;

    let steps = rows
        .into_iter()
        .map(|row| {
            Ok(StepLineageOptions {
                form_version_id: row.try_get("form_version_id")?,
                assignment_node_ids: row.try_get("assignment_node_ids")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let mut all_node_ids = Vec::new();
    for step in &steps {
        for node_id in &step.assignment_node_ids {
            if !all_node_ids.contains(node_id) {
                all_node_ids.push(*node_id);
            }
        }
    }
    let parent_map = load_node_parent_map_tx(tx, &all_node_ids).await?;

    for left_index in 0..steps.len() {
        for right_index in (left_index + 1)..steps.len() {
            let left = &steps[left_index];
            let right = &steps[right_index];
            if !step_lineage_options_are_composable(left, right, &parent_map) {
                return Err(ApiError::BadRequest(format!(
                    "workflow step form assignments must stay on one hierarchy lineage ({} and {})",
                    left.form_version_id, right.form_version_id
                )));
            }
        }
    }

    Ok(())
}

async fn load_node_parent_map_tx(
    tx: &mut Transaction<'_, Postgres>,
    seed_node_ids: &[Uuid],
) -> ApiResult<HashMap<Uuid, Option<Uuid>>> {
    if seed_node_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query(
        r#"
        WITH RECURSIVE ancestors AS (
            SELECT nodes.id, nodes.parent_node_id
            FROM nodes
            WHERE nodes.id = ANY($1)
            UNION
            SELECT parent_nodes.id, parent_nodes.parent_node_id
            FROM nodes AS parent_nodes
            JOIN ancestors ON ancestors.parent_node_id = parent_nodes.id
        )
        SELECT id, parent_node_id
        FROM ancestors
        "#,
    )
    .bind(seed_node_ids)
    .fetch_all(&mut **tx)
    .await?;

    rows.into_iter()
        .map(|row| Ok((row.try_get("id")?, row.try_get("parent_node_id")?)))
        .collect::<Result<HashMap<Uuid, Option<Uuid>>, sqlx::Error>>()
        .map_err(Into::into)
}

fn step_lineage_options_are_composable(
    left: &StepLineageOptions,
    right: &StepLineageOptions,
    parent_map: &HashMap<Uuid, Option<Uuid>>,
) -> bool {
    if !left.assignment_node_ids.is_empty() && !right.assignment_node_ids.is_empty() {
        return left.assignment_node_ids.iter().any(|left_node_id| {
            right.assignment_node_ids.iter().any(|right_node_id| {
                node_ids_are_comparable(*left_node_id, *right_node_id, parent_map)
            })
        });
    }

    true
}

fn node_ids_are_comparable(
    left_node_id: Uuid,
    right_node_id: Uuid,
    parent_map: &HashMap<Uuid, Option<Uuid>>,
) -> bool {
    left_node_id == right_node_id
        || node_id_is_ancestor(left_node_id, right_node_id, parent_map)
        || node_id_is_ancestor(right_node_id, left_node_id, parent_map)
}

fn node_id_is_ancestor(
    ancestor_node_id: Uuid,
    descendant_node_id: Uuid,
    parent_map: &HashMap<Uuid, Option<Uuid>>,
) -> bool {
    let mut current = Some(descendant_node_id);
    while let Some(node_id) = current {
        if node_id == ancestor_node_id {
            return true;
        }
        current = parent_map.get(&node_id).copied().flatten();
    }
    false
}

async fn resolve_workflow_step_node_tx(
    tx: &mut Transaction<'_, Postgres>,
    assignment_node_id: Uuid,
    form_version_id: Uuid,
) -> ApiResult<Uuid> {
    let assigned_node_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        WITH RECURSIVE descendants AS (
            SELECT
                nodes.id,
                nodes.name,
                0 AS depth
            FROM nodes
            WHERE nodes.id = $1
            UNION ALL
            SELECT
                child_nodes.id,
                child_nodes.name,
                descendants.depth + 1 AS depth
            FROM descendants
            JOIN nodes AS child_nodes ON child_nodes.parent_node_id = descendants.id
        )
        SELECT form_assignments.node_id
        FROM form_assignments
        JOIN descendants ON descendants.id = form_assignments.node_id
        WHERE form_assignments.form_version_id = $2
        ORDER BY descendants.depth, descendants.name, form_assignments.node_id
        LIMIT 1
        "#,
    )
    .bind(assignment_node_id)
    .bind(form_version_id)
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(node_id) = assigned_node_id {
        return Ok(node_id);
    }

    let has_form_assignments: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM form_assignments WHERE form_version_id = $1)",
    )
    .bind(form_version_id)
    .fetch_one(&mut **tx)
    .await?;
    if has_form_assignments {
        return Err(ApiError::BadRequest(
            "workflow step form is not linked to the assignment node or descendants".into(),
        ));
    }

    let scope_node_type_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT forms.scope_node_type_id
        FROM form_versions
        JOIN forms ON forms.id = form_versions.form_id
        WHERE form_versions.id = $1
        "#,
    )
    .bind(form_version_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("form version {form_version_id}")))?;

    let Some(scope_node_type_id) = scope_node_type_id else {
        return Ok(assignment_node_id);
    };

    sqlx::query_scalar(
        r#"
        WITH RECURSIVE descendants AS (
            SELECT
                nodes.id,
                nodes.node_type_id,
                nodes.name,
                0 AS depth
            FROM nodes
            WHERE nodes.id = $1
            UNION ALL
            SELECT
                child_nodes.id,
                child_nodes.node_type_id,
                child_nodes.name,
                descendants.depth + 1 AS depth
            FROM descendants
            JOIN nodes AS child_nodes ON child_nodes.parent_node_id = descendants.id
        )
        SELECT id
        FROM descendants
        WHERE node_type_id = $2
        ORDER BY depth, name, id
        LIMIT 1
        "#,
    )
    .bind(assignment_node_id)
    .bind(scope_node_type_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| {
        ApiError::BadRequest(
            "workflow step form is not compatible with the assignment node or descendants".into(),
        )
    })
}

async fn resolve_legacy_workflow_form_id(
    pool: &sqlx::PgPool,
    requested_form_id: Option<Uuid>,
    workflow_id: Option<Uuid>,
) -> ApiResult<Uuid> {
    if let Some(form_id) = requested_form_id {
        return Ok(form_id);
    }

    if let Some(workflow_id) = workflow_id {
        let existing_form_id: Option<Uuid> =
            sqlx::query_scalar("SELECT form_id FROM workflows WHERE id = $1")
                .bind(workflow_id)
                .fetch_optional(pool)
                .await?;
        if let Some(form_id) = existing_form_id {
            return Ok(form_id);
        }
        return Err(ApiError::NotFound(format!("workflow {workflow_id}")));
    }

    sqlx::query_scalar("SELECT id FROM forms ORDER BY created_at, id LIMIT 1")
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            ApiError::BadRequest("create at least one form before creating a workflow".into())
        })
}

async fn require_workflow_payload(
    pool: &sqlx::PgPool,
    form_id: Uuid,
    name: &str,
    slug: &str,
    current_workflow_id: Option<Uuid>,
) -> ApiResult<()> {
    if name.trim().is_empty() {
        return Err(ApiError::BadRequest("workflow name is required".into()));
    }
    if slug.trim().is_empty() {
        return Err(ApiError::BadRequest("workflow slug is required".into()));
    }
    let form_exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM forms WHERE id = $1)")
        .bind(form_id)
        .fetch_one(pool)
        .await?;
    if !form_exists {
        return Err(ApiError::NotFound(format!("form {form_id}")));
    }

    let duplicate_slug: bool = if let Some(current_workflow_id) = current_workflow_id {
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workflows WHERE slug = $1 AND id <> $2)")
            .bind(slug.trim())
            .bind(current_workflow_id)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workflows WHERE slug = $1)")
            .bind(slug.trim())
            .fetch_one(pool)
            .await?
    };
    if duplicate_slug {
        return Err(ApiError::BadRequest(format!(
            "workflow slug '{}' is already in use",
            slug.trim()
        )));
    }

    Ok(())
}

async fn ensure_workflow_assignment_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_version_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
    form_assignment_id: Option<Uuid>,
) -> ApiResult<Uuid> {
    let (id, _) = ensure_workflow_assignment_with_status_tx(
        tx,
        workflow_version_id,
        node_id,
        account_id,
        form_assignment_id,
    )
    .await?;
    Ok(id)
}

async fn ensure_workflow_assignment_with_status_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_version_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
    form_assignment_id: Option<Uuid>,
) -> ApiResult<(Uuid, String)> {
    let row = sqlx::query(
        r#"
        SELECT id
        FROM workflow_steps
        WHERE workflow_version_id = $1
        ORDER BY position
        LIMIT 1
        "#,
    )
    .bind(workflow_version_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| {
        ApiError::BadRequest("workflow versions must have at least one response step".into())
    })?;
    let step_id: Uuid = row.try_get("id")?;

    ensure_specific_workflow_assignment_tx(
        tx,
        workflow_version_id,
        step_id,
        node_id,
        account_id,
        form_assignment_id,
    )
    .await
}

async fn ensure_specific_workflow_assignment_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_version_id: Uuid,
    step_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
    form_assignment_id: Option<Uuid>,
) -> ApiResult<(Uuid, String)> {
    if let Some(existing) = sqlx::query(
        r#"
        SELECT id, is_active
        FROM workflow_assignments
        WHERE workflow_step_id = $1
          AND node_id = $2
          AND account_id = $3
        "#,
    )
    .bind(step_id)
    .bind(node_id)
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await?
    {
        let existing_id: Uuid = existing.try_get("id")?;
        let was_active: bool = existing.try_get("is_active")?;
        sqlx::query(
            r#"
            UPDATE workflow_assignments
            SET workflow_version_id = $2,
                form_assignment_id = COALESCE($3, form_assignment_id),
                is_active = true
            WHERE id = $1
        "#,
        )
        .bind(existing_id)
        .bind(workflow_version_id)
        .bind(form_assignment_id)
        .execute(&mut **tx)
        .await?;
        return Ok((
            existing_id,
            if was_active { "skipped" } else { "reactivated" }.into(),
        ));
    }

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO workflow_assignments (
            workflow_version_id,
            workflow_step_id,
            node_id,
            account_id,
            form_assignment_id,
            is_active
        )
        VALUES ($1, $2, $3, $4, $5, true)
        RETURNING id
        "#,
    )
    .bind(workflow_version_id)
    .bind(step_id)
    .bind(node_id)
    .bind(account_id)
    .bind(form_assignment_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::from)?;
    Ok((id, "created".into()))
}
