use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use chrono::{DateTime, Utc};
use sqlx::{Postgres, Row, Transaction};
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
    CreateWorkflowRequest, CreateWorkflowRevisionRequest, CreateWorkflowStepRequest,
    PendingWorkflowWork, UpdateWorkflowAssignmentRequest, UpdateWorkflowRequest,
    UpdateWorkflowRevisionStepsRequest, WorkflowAssigneeOption, WorkflowAssignmentCandidate,
    WorkflowAssignmentQuery, WorkflowAssignmentSummary, WorkflowDefinition, WorkflowStepSummary,
    WorkflowSummary, WorkflowVersionSummary,
};

fn workflow_revision_number(label: &str) -> Option<u64> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(revision) = trimmed.parse::<u64>() {
        return Some(revision);
    }

    trimmed
        .split('.')
        .next()
        .and_then(|part| part.trim().parse::<u64>().ok())
}

fn workflow_revision_label(label: &str) -> Option<String> {
    workflow_revision_number(label).map(|revision| revision.to_string())
}

async fn next_workflow_version_label_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_id: Uuid,
) -> ApiResult<String> {
    let rows = sqlx::query("SELECT version_label FROM workflow_versions WHERE workflow_id = $1")
        .bind(workflow_id)
        .fetch_all(&mut **tx)
        .await?;

    let next = rows
        .iter()
        .filter_map(|row| {
            row.try_get::<Option<String>, _>("version_label")
                .ok()
                .flatten()
        })
        .filter_map(|label| workflow_revision_number(&label))
        .max()
        .map(|revision| revision + 1)
        .unwrap_or(1);

    Ok(next.to_string())
}

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
    require_workflow_payload(
        &state.pool,
        payload.workflow_node_type_id,
        &payload.name,
        &payload.slug,
        None,
    )
    .await?;

    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workflows (workflow_node_type_id, name, slug, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(payload.workflow_node_type_id)
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
    require_workflow_payload(
        &state.pool,
        payload.workflow_node_type_id,
        &payload.name,
        &payload.slug,
        Some(workflow_id),
    )
    .await?;

    let updated = sqlx::query(
        r#"
        UPDATE workflows
        SET workflow_node_type_id = $2,
            name = $3,
            slug = $4,
            description = $5
        WHERE id = $1
        "#,
    )
    .bind(workflow_id)
    .bind(payload.workflow_node_type_id)
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
    Json(payload): Json<CreateWorkflowRevisionRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;

    let steps = normalize_step_collection(payload.steps)?;
    let mut tx = state.pool.begin().await?;
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM workflows WHERE id = $1")
        .bind(workflow_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("workflow {workflow_id}")))?;
    require_workflow_steps_match_scope_tx(&mut tx, workflow_id, &steps).await?;
    if let Some(existing_draft) = sqlx::query(
        r#"
        SELECT id, version_label
        FROM workflow_versions
        WHERE workflow_id = $1
          AND status = 'draft'::form_version_status
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(workflow_id)
    .fetch_optional(&mut *tx)
    .await?
    {
        let existing_draft_id: Uuid = existing_draft.try_get("id")?;
        let existing_version_label: Option<String> = existing_draft.try_get("version_label")?;
        let draft_version_label = existing_version_label
            .as_deref()
            .and_then(workflow_revision_label)
            .unwrap_or(next_workflow_version_label_tx(&mut tx, workflow_id).await?);

        sqlx::query(
            r#"
            UPDATE workflow_versions
            SET version_label = $2,
                published_at = NULL
            WHERE id = $1
            "#,
        )
        .bind(existing_draft_id)
        .bind(draft_version_label)
        .execute(&mut *tx)
        .await?;
        replace_workflow_steps_tx(&mut tx, existing_draft_id, &steps).await?;
        promote_generated_workflow_if_needed_tx(&mut tx, workflow_id).await?;
        tx.commit().await?;
        return Ok(Json(IdResponse {
            id: existing_draft_id,
        }));
    }
    let version_label = next_workflow_version_label_tx(&mut tx, workflow_id).await?;
    let workflow_version_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workflow_versions (workflow_id, version_label, status, published_at)
        VALUES ($1, $2, 'draft'::form_version_status, NULL)
        RETURNING id
        "#,
    )
    .bind(workflow_id)
    .bind(version_label)
    .fetch_one(&mut *tx)
    .await?;
    replace_workflow_steps_tx(&mut tx, workflow_version_id, &steps).await?;
    promote_generated_workflow_if_needed_tx(&mut tx, workflow_id).await?;
    tx.commit().await?;

    Ok(Json(IdResponse {
        id: workflow_version_id,
    }))
}

pub async fn replace_workflow_version_steps(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_version_id): Path<Uuid>,
    Json(payload): Json<UpdateWorkflowRevisionStepsRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;
    let steps = normalize_step_collection(payload.steps)?;
    let mut tx = state.pool.begin().await?;
    let status: String =
        sqlx::query_scalar("SELECT status::text FROM workflow_versions WHERE id = $1")
            .bind(workflow_version_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| {
                ApiError::NotFound(format!("workflow revision {workflow_version_id}"))
            })?;
    if status != "draft" {
        return Err(ApiError::BadRequest(
            "workflow revision steps can only be changed while the revision is draft".into(),
        ));
    }
    let workflow_id: Uuid =
        sqlx::query_scalar("SELECT workflow_id FROM workflow_versions WHERE id = $1")
            .bind(workflow_version_id)
            .fetch_one(&mut *tx)
            .await?;
    require_workflow_steps_match_scope_tx(&mut tx, workflow_id, &steps).await?;
    replace_workflow_steps_tx(&mut tx, workflow_version_id, &steps).await?;
    promote_generated_workflow_if_needed_tx(&mut tx, workflow_id).await?;
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
            .ok_or_else(|| {
                ApiError::NotFound(format!("workflow revision {workflow_version_id}"))
            })?;
    if status != "draft" {
        return Err(ApiError::BadRequest(
            "workflow revisions can only be deleted while they are draft".into(),
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
    let workflow_id: Uuid =
        sqlx::query_scalar("SELECT workflow_id FROM workflow_versions WHERE id = $1")
            .bind(workflow_version_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| {
                ApiError::NotFound(format!("workflow revision {workflow_version_id}"))
            })?;
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
    promote_generated_workflow_if_needed_tx(&mut tx, workflow_id).await?;
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
        SELECT workflow_version_id
        FROM workflow_assignments
        WHERE id = $1
        "#,
    )
    .bind(workflow_assignment_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("workflow assignment {workflow_assignment_id}")))?;
    let workflow_version_id: Uuid = current.try_get("workflow_version_id")?;
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
            forms.scope_node_type_id,
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
    let workflow_node_type_id: Uuid = row
        .try_get::<Option<Uuid>, _>("scope_node_type_id")?
        .ok_or_else(|| {
            ApiError::BadRequest(
                "cannot generate a workflow for a form without a scope node type".into(),
            )
        })?;

    let existing_generated: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id
        FROM workflows
        WHERE source = 'generated_form'
          AND source_form_id = $1
        ORDER BY created_at
        LIMIT 1
        "#,
    )
    .bind(form_id)
    .fetch_optional(&mut **tx)
    .await?;

    let workflow_id: Uuid = if let Some(existing) = existing_generated {
        if generated_workflow_is_single_form_shortcut_tx(tx, existing, form_id).await? {
            existing
        } else {
            promote_generated_workflow_tx(tx, existing).await?;
            create_generated_form_workflow_tx(
                tx,
                form_id,
                workflow_node_type_id,
                &form_name,
                &form_slug,
            )
            .await?
        }
    } else {
        create_generated_form_workflow_tx(
            tx,
            form_id,
            workflow_node_type_id,
            &form_name,
            &form_slug,
        )
        .await?
    };

    sqlx::query(
        r#"
        UPDATE workflows
        SET workflow_node_type_id = $2,
            source = 'generated_form',
            source_form_id = $3
        WHERE id = $1
        "#,
    )
    .bind(workflow_id)
    .bind(workflow_node_type_id)
    .bind(form_id)
    .execute(&mut **tx)
    .await?;

    let status: String = row.try_get("status")?;
    let published_at: Option<DateTime<Utc>> = row.try_get("published_at")?;
    let workflow_version_id: Uuid = if let Some(existing) = sqlx::query_scalar(
        r#"
        SELECT workflow_versions.id
        FROM workflow_versions
        JOIN workflow_steps ON workflow_steps.workflow_version_id = workflow_versions.id
        WHERE workflow_versions.workflow_id = $1
          AND workflow_steps.form_version_id = $2
          AND workflow_steps.position = 0
        LIMIT 1
        "#,
    )
    .bind(workflow_id)
    .bind(form_version_id)
    .fetch_optional(&mut **tx)
    .await?
    {
        sqlx::query(
            r#"
            UPDATE workflow_versions
            SET status = $2::form_version_status,
                published_at = $3
            WHERE id = $1
            "#,
        )
        .bind(existing)
        .bind(status)
        .bind(published_at)
        .execute(&mut **tx)
        .await?;
        existing
    } else {
        let version_label = next_workflow_version_label_tx(tx, workflow_id).await?;
        sqlx::query_scalar(
            r#"
            INSERT INTO workflow_versions (workflow_id, version_label, status, published_at)
            VALUES ($1, $2, $3::form_version_status, $4)
            RETURNING id
            "#,
        )
        .bind(workflow_id)
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

async fn create_generated_form_workflow_tx(
    tx: &mut Transaction<'_, Postgres>,
    form_id: Uuid,
    workflow_node_type_id: Uuid,
    form_name: &str,
    form_slug: &str,
) -> ApiResult<Uuid> {
    let workflow_slug = unique_generated_workflow_slug_tx(tx, form_slug).await?;
    let workflow_id = sqlx::query_scalar(
        r#"
        INSERT INTO workflows (
            workflow_node_type_id,
            name,
            slug,
            description,
            source,
            source_form_id
        )
        VALUES ($1, $2, $3, $4, 'generated_form', $5)
        RETURNING id
        "#,
    )
    .bind(workflow_node_type_id)
    .bind(format!("{form_name} Workflow"))
    .bind(workflow_slug)
    .bind("Generated single-form workflow.")
    .bind(form_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(workflow_id)
}

async fn unique_generated_workflow_slug_tx(
    tx: &mut Transaction<'_, Postgres>,
    form_slug: &str,
) -> ApiResult<String> {
    for suffix in 0..100 {
        let candidate = if suffix == 0 {
            format!("{form_slug}-workflow")
        } else {
            format!("{form_slug}-workflow-{suffix}")
        };
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workflows WHERE slug = $1)")
                .bind(&candidate)
                .fetch_one(&mut **tx)
                .await?;
        if !exists {
            return Ok(candidate);
        }
    }

    Err(ApiError::BadRequest(format!(
        "could not generate a unique workflow slug for form '{form_slug}'"
    )))
}

async fn generated_workflow_is_single_form_shortcut_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_id: Uuid,
    form_id: Uuid,
) -> ApiResult<bool> {
    let Some(row) = sqlx::query(
        r#"
        SELECT
            COUNT(workflow_steps.id) AS step_count,
            BOOL_AND(form_versions.form_id = $2) AS steps_match_form
        FROM workflow_versions
        LEFT JOIN workflow_steps ON workflow_steps.workflow_version_id = workflow_versions.id
        LEFT JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
        WHERE workflow_versions.workflow_id = $1
        GROUP BY workflow_versions.id
        ORDER BY workflow_versions.published_at DESC NULLS LAST, workflow_versions.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(workflow_id)
    .bind(form_id)
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Ok(true);
    };

    let step_count: i64 = row.try_get("step_count")?;
    let steps_match_form: Option<bool> = row.try_get("steps_match_form")?;
    Ok(step_count == 1 && steps_match_form.unwrap_or(false))
}

async fn promote_generated_workflow_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        UPDATE workflows
        SET source = 'authored',
            source_form_id = NULL
        WHERE id = $1
          AND source = 'generated_form'
        "#,
    )
    .bind(workflow_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn promote_generated_workflow_if_needed_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_id: Uuid,
) -> ApiResult<()> {
    let Some(row) = sqlx::query("SELECT source, source_form_id FROM workflows WHERE id = $1")
        .bind(workflow_id)
        .fetch_optional(&mut **tx)
        .await?
    else {
        return Ok(());
    };

    let source: String = row.try_get("source")?;
    let source_form_id: Option<Uuid> = row.try_get("source_form_id")?;
    if source == "generated_form" {
        if let Some(form_id) = source_form_id {
            if !generated_workflow_is_single_form_shortcut_tx(tx, workflow_id, form_id).await? {
                promote_generated_workflow_tx(tx, workflow_id).await?;
            }
        }
    }
    Ok(())
}

pub async fn ensure_workflow_assignment_for_form_version(
    pool: &sqlx::PgPool,
    form_version_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
) -> ApiResult<Uuid> {
    let mut tx = pool.begin().await?;
    let (_, workflow_version_id, _) =
        ensure_workflow_for_published_form_version_tx(&mut tx, form_version_id).await?;
    let workflow_assignment_id =
        ensure_workflow_assignment_tx(&mut tx, workflow_version_id, node_id, account_id).await?;
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
    if workflow_status != "published" && workflow_status != "superseded" {
        return Err(ApiError::BadRequest(
            "only published or superseded workflow revisions can start response work".into(),
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
            form_version_id,
            node_id,
            workflow_assignment_id,
            workflow_instance_id
        )
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
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
        VALUES ($1, 'start_response', $2)
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
          AND workflow_versions.status IN (
              'published'::form_version_status,
              'superseded'::form_version_status
          )
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
            workflows.workflow_node_type_id,
            node_types.name AS workflow_node_type_name,
            workflows.name,
            workflows.slug,
            workflows.description,
            workflows.source,
            workflows.source_form_id,
            current_versions.id AS current_version_id,
            current_versions.version_label AS current_version_label,
            current_versions.status::text AS current_status,
            COUNT(DISTINCT workflow_assignments.id) FILTER (WHERE workflow_assignments.is_active) AS assignment_count,
            COUNT(DISTINCT workflow_versions.id) AS version_count,
            COALESCE(
                array_remove(array_agg(DISTINCT nodes.name) FILTER (WHERE workflow_assignments.is_active), NULL),
                ARRAY[]::text[]
            ) AS assignment_node_names
        FROM workflows
        JOIN node_types ON node_types.id = workflows.workflow_node_type_id
        LEFT JOIN LATERAL (
            SELECT id, version_label, status
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
            workflows.workflow_node_type_id,
            node_types.name,
            workflows.name,
            workflows.slug,
            workflows.description,
            workflows.source,
            workflows.source_form_id,
            current_versions.id,
            current_versions.version_label,
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
                workflow_node_type_id: row.try_get("workflow_node_type_id")?,
                workflow_node_type_name: row.try_get("workflow_node_type_name")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                description: row.try_get("description")?,
                source: row.try_get("source")?,
                source_form_id: row.try_get("source_form_id")?,
                current_version_id: row.try_get("current_version_id")?,
                current_version_label: row.try_get("current_version_label")?,
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
            workflows.workflow_node_type_id,
            node_types.name AS workflow_node_type_name,
            workflows.name,
            workflows.slug,
            workflows.description,
            workflows.source,
            workflows.source_form_id
        FROM workflows
        JOIN node_types ON node_types.id = workflows.workflow_node_type_id
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
            workflow_versions.version_label AS workflow_revision_label,
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
            workflow_revision_label: version_row.try_get("workflow_revision_label")?,
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
        workflow_node_type_id: row.try_get("workflow_node_type_id")?,
        workflow_node_type_name: row.try_get("workflow_node_type_name")?,
        name: row.try_get("name")?,
        slug: row.try_get("slug")?,
        description: row.try_get("description")?,
        source: row.try_get("source")?,
        source_form_id: row.try_get("source_form_id")?,
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
        JOIN nodes ON ($1::uuid IS NULL OR nodes.id = $1)
        LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
        WHERE workflow_versions.status = 'published'::form_version_status
          AND step_totals.step_count > 0
          AND ($2::uuid[] IS NULL OR nodes.id = ANY($2))
          AND nodes.node_type_id = workflows.workflow_node_type_id
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

fn normalize_step_collection(
    steps: Vec<CreateWorkflowStepRequest>,
) -> ApiResult<Vec<CreateWorkflowStepRequest>> {
    if steps.is_empty() {
        return Err(ApiError::BadRequest(
            "workflow revision requires at least one step".into(),
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

async fn require_workflow_steps_match_scope_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_id: Uuid,
    steps: &[CreateWorkflowStepRequest],
) -> ApiResult<()> {
    let workflow_node_type_id: Uuid =
        sqlx::query_scalar("SELECT workflow_node_type_id FROM workflows WHERE id = $1")
            .bind(workflow_id)
            .fetch_optional(&mut **tx)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("workflow {workflow_id}")))?;

    for step in steps {
        let row = sqlx::query(
            r#"
            SELECT
                forms.name AS form_name,
                forms.scope_node_type_id
            FROM form_versions
            JOIN forms ON forms.id = form_versions.form_id
            WHERE form_versions.id = $1
            "#,
        )
        .bind(step.form_version_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("form version {}", step.form_version_id)))?;

        let form_scope_node_type_id: Option<Uuid> = row.try_get("scope_node_type_id")?;
        let Some(form_scope_node_type_id) = form_scope_node_type_id else {
            continue;
        };

        let is_in_workflow_scope: bool = sqlx::query_scalar(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT node_types.id
                FROM node_types
                WHERE node_types.id = $1
                UNION ALL
                SELECT node_type_relationships.child_node_type_id
                FROM descendants
                JOIN node_type_relationships
                    ON node_type_relationships.parent_node_type_id = descendants.id
            )
            SELECT EXISTS (
                SELECT 1
                FROM descendants
                WHERE id = $2
            )
            "#,
        )
        .bind(workflow_node_type_id)
        .bind(form_scope_node_type_id)
        .fetch_one(&mut **tx)
        .await?;

        if !is_in_workflow_scope {
            let form_name: String = row.try_get("form_name")?;
            return Err(ApiError::BadRequest(format!(
                "form '{form_name}' is outside the workflow node type scope"
            )));
        }
    }

    Ok(())
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
    let workflow_id: Uuid =
        sqlx::query_scalar("SELECT workflow_id FROM workflow_versions WHERE id = $1")
            .bind(workflow_version_id)
            .fetch_one(&mut **tx)
            .await?;
    let rows = sqlx::query(
        r#"
        SELECT
            workflow_steps.title,
            workflow_steps.position,
            workflow_steps.form_version_id,
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
            "workflow revisions require at least one step before publish".into(),
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
    let steps = rows
        .iter()
        .map(|row| {
            Ok(CreateWorkflowStepRequest {
                title: row.try_get("title")?,
                form_version_id: row.try_get("form_version_id")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    require_workflow_steps_match_scope_tx(tx, workflow_id, &steps).await?;
    Ok(())
}

async fn resolve_workflow_step_node_tx(
    tx: &mut Transaction<'_, Postgres>,
    assignment_node_id: Uuid,
    form_version_id: Uuid,
) -> ApiResult<Uuid> {
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

async fn require_workflow_payload(
    pool: &sqlx::PgPool,
    workflow_node_type_id: Uuid,
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
    let node_type_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM node_types WHERE id = $1)")
            .bind(workflow_node_type_id)
            .fetch_one(pool)
            .await?;
    if !node_type_exists {
        return Err(ApiError::NotFound(format!(
            "node type {workflow_node_type_id}"
        )));
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
) -> ApiResult<Uuid> {
    let (id, _) =
        ensure_workflow_assignment_with_status_tx(tx, workflow_version_id, node_id, account_id)
            .await?;
    Ok(id)
}

async fn ensure_workflow_assignment_with_status_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_version_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
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
        ApiError::BadRequest("workflow revisions must have at least one response step".into())
    })?;
    let step_id: Uuid = row.try_get("id")?;

    ensure_specific_workflow_assignment_tx(tx, workflow_version_id, step_id, node_id, account_id)
        .await
}

async fn ensure_specific_workflow_assignment_tx(
    tx: &mut Transaction<'_, Postgres>,
    workflow_version_id: Uuid,
    step_id: Uuid,
    node_id: Uuid,
    account_id: Uuid,
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
                is_active = true
            WHERE id = $1
        "#,
        )
        .bind(existing_id)
        .bind(workflow_version_id)
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
            is_active
        )
        VALUES ($1, $2, $3, $4, true)
        RETURNING id
        "#,
    )
    .bind(workflow_version_id)
    .bind(step_id)
    .bind(node_id)
    .bind(account_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::from)?;
    Ok((id, "created".into()))
}
