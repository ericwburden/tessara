use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::IdResponse,
};

#[derive(Deserialize)]
pub struct CreateWorkflowRequest {
    pub form_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateWorkflowRequest {
    pub form_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateWorkflowVersionRequest {
    pub form_version_id: Uuid,
    pub title: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateWorkflowAssignmentRequest {
    pub workflow_version_id: Uuid,
    pub node_id: Uuid,
    pub account_id: Uuid,
}

#[derive(Deserialize)]
pub struct UpdateWorkflowAssignmentRequest {
    pub node_id: Uuid,
    pub account_id: Uuid,
    pub is_active: bool,
}

#[derive(Deserialize, Default)]
pub struct WorkflowAssignmentQuery {
    pub workflow_id: Option<Uuid>,
    pub workflow_version_id: Option<Uuid>,
    pub form_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub node_id: Option<Uuid>,
    pub active: Option<bool>,
    pub delegate_account_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct WorkflowSummary {
    pub id: Uuid,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_slug: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub current_version_id: Option<Uuid>,
    pub current_version_label: Option<String>,
    pub current_form_version_id: Option<Uuid>,
    pub current_status: Option<String>,
    pub assignment_count: i64,
}

#[derive(Serialize)]
pub struct WorkflowVersionSummary {
    pub id: Uuid,
    pub form_version_id: Uuid,
    pub form_version_label: Option<String>,
    pub title: String,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct WorkflowAssignmentSummary {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub workflow_version_id: Uuid,
    pub workflow_version_label: Option<String>,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_version_id: Uuid,
    pub form_version_label: Option<String>,
    pub workflow_step_id: Uuid,
    pub workflow_step_title: String,
    pub node_id: Uuid,
    pub node_name: String,
    pub account_id: Uuid,
    pub account_display_name: String,
    pub account_email: String,
    pub is_active: bool,
    pub has_draft: bool,
    pub has_submitted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct WorkflowDefinition {
    pub id: Uuid,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_slug: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub versions: Vec<WorkflowVersionSummary>,
    pub assignments: Vec<WorkflowAssignmentSummary>,
}

#[derive(Serialize)]
pub struct PendingWorkflowWork {
    pub workflow_assignment_id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub workflow_version_id: Uuid,
    pub workflow_version_label: Option<String>,
    pub workflow_step_title: String,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_version_id: Uuid,
    pub form_version_label: Option<String>,
    pub node_id: Uuid,
    pub node_name: String,
    pub account_id: Uuid,
    pub account_display_name: String,
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
        payload.form_id,
        &payload.name,
        &payload.slug,
        None,
    )
    .await?;

    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO workflows (form_id, name, slug, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(payload.form_id)
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
        payload.form_id,
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
    .bind(payload.form_id)
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

    let mut tx = state.pool.begin().await?;
    let workflow_form_id: Uuid = sqlx::query_scalar("SELECT form_id FROM workflows WHERE id = $1")
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
    .bind(payload.form_version_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("form version {}", payload.form_version_id)))?;
    let form_id: Uuid = version_row.try_get("form_id")?;
    if form_id != workflow_form_id {
        return Err(ApiError::BadRequest(
            "workflow version must reference a form version from the linked form".into(),
        ));
    }

    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM workflow_versions WHERE form_version_id = $1")
            .bind(payload.form_version_id)
            .fetch_optional(&mut *tx)
            .await?;
    if let Some(id) = existing {
        return Ok(Json(IdResponse { id }));
    }

    let status: String = version_row.try_get("status")?;
    let version_label: Option<String> = version_row.try_get("version_label")?;
    let title = payload
        .title
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Primary Response".into());
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
    .bind(payload.form_version_id)
    .bind(version_label)
    .bind(status)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO workflow_steps (workflow_version_id, form_version_id, title, position)
        VALUES ($1, $2, $3, 0)
        "#,
    )
    .bind(workflow_version_id)
    .bind(payload.form_version_id)
    .bind(title)
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
            "workflow versions can only be published when their linked form version is published"
                .into(),
        ));
    }

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
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;
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

pub async fn update_workflow_assignment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_assignment_id): Path<Uuid>,
    Json(payload): Json<UpdateWorkflowAssignmentRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "workflows:write").await?;
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
    headers: HeaderMap,
    Query(query): Query<WorkflowAssignmentQuery>,
) -> ApiResult<Json<Vec<PendingWorkflowWork>>> {
    let account = auth::require_authenticated(&state.pool, &headers).await?;
    let delegate_account_id = auth::resolve_accessible_delegate_account_id(
        &state.pool,
        &account,
        query.delegate_account_id,
    )
    .await?;
    Ok(Json(
        list_pending_assignments_for_account(&state.pool, delegate_account_id).await?,
    ))
}

pub async fn start_assignment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workflow_assignment_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_authenticated(&state.pool, &headers).await?;
    let id = start_workflow_assignment(&state.pool, &account, workflow_assignment_id).await?;
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
    let workflow_id: Uuid = if let Some(existing) =
        sqlx::query_scalar("SELECT id FROM workflows WHERE form_id = $1")
            .bind(form_id)
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
        .bind(format!("{form_slug}-workflow"))
        .bind("Generated from the linked form for Sprint 2A runtime compatibility.")
        .fetch_one(&mut **tx)
        .await?
    };

    let status: String = row.try_get("status")?;
    let version_label: Option<String> = row.try_get("version_label")?;
    let published_at: Option<DateTime<Utc>> = row.try_get("published_at")?;
    let workflow_version_id: Uuid = if let Some(existing) =
        sqlx::query_scalar("SELECT id FROM workflow_versions WHERE form_version_id = $1")
            .bind(form_version_id)
            .fetch_optional(&mut **tx)
            .await?
    {
        sqlx::query(
            r#"
            UPDATE workflow_versions
            SET workflow_id = $2,
                version_label = $3,
                status = $4::form_version_status,
                published_at = $5
            WHERE id = $1
            "#,
        )
        .bind(existing)
        .bind(workflow_id)
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
    if !can_access_workflow_assignment(pool, account, assignee_account_id).await? {
        return Err(ApiError::Forbidden("workflow_assignment:start".into()));
    }

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
        .bind(node_id)
        .bind(assignee_account_id)
        .fetch_one(&mut *tx)
        .await?
    };

    let workflow_instance_id: Uuid = sqlx::query_scalar(
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
    .await?;

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
    .bind(node_id)
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
            workflow_versions.id AS workflow_version_id,
            workflow_versions.version_label AS workflow_version_label,
            workflow_steps.title AS workflow_step_title,
            forms.id AS form_id,
            forms.name AS form_name,
            workflow_steps.form_version_id,
            form_versions.version_label AS form_version_label,
            nodes.id AS node_id,
            nodes.name AS node_name,
            accounts.id AS account_id,
            accounts.display_name AS account_display_name
        FROM workflow_assignments
        JOIN workflow_versions ON workflow_versions.id = workflow_assignments.workflow_version_id
        JOIN workflows ON workflows.id = workflow_versions.workflow_id
        JOIN forms ON forms.id = workflows.form_id
        JOIN workflow_steps ON workflow_steps.id = workflow_assignments.workflow_step_id
        JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
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
                workflow_version_id: row.try_get("workflow_version_id")?,
                workflow_version_label: row.try_get("workflow_version_label")?,
                workflow_step_title: row.try_get("workflow_step_title")?,
                form_id: row.try_get("form_id")?,
                form_name: row.try_get("form_name")?,
                form_version_id: row.try_get("form_version_id")?,
                form_version_label: row.try_get("form_version_label")?,
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                account_id: row.try_get("account_id")?,
                account_display_name: row.try_get("account_display_name")?,
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
            COUNT(workflow_assignments.id) FILTER (WHERE workflow_assignments.is_active) AS assignment_count
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
            workflow_versions.created_at,
            workflow_steps.title
        FROM workflow_versions
        JOIN workflow_steps
            ON workflow_steps.workflow_version_id = workflow_versions.id
           AND workflow_steps.position = 0
        WHERE workflow_versions.workflow_id = $1
        ORDER BY workflow_versions.created_at DESC, workflow_versions.id DESC
        "#,
    )
    .bind(workflow_id)
    .fetch_all(pool)
    .await?;
    let versions = version_rows
        .into_iter()
        .map(|version_row| {
            Ok(WorkflowVersionSummary {
                id: version_row.try_get("id")?,
                form_version_id: version_row.try_get("form_version_id")?,
                form_version_label: version_row.try_get("form_version_label")?,
                title: version_row.try_get("title")?,
                status: version_row.try_get("status")?,
                published_at: version_row.try_get("published_at")?,
                created_at: version_row.try_get("created_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

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
        JOIN forms ON forms.id = workflows.form_id
        JOIN workflow_steps ON workflow_steps.id = workflow_assignments.workflow_step_id
        JOIN form_versions ON form_versions.id = workflow_steps.form_version_id
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

    let duplicate_form: bool = if let Some(current_workflow_id) = current_workflow_id {
        sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM workflows WHERE form_id = $1 AND id <> $2)",
        )
        .bind(form_id)
        .bind(current_workflow_id)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workflows WHERE form_id = $1)")
            .bind(form_id)
            .fetch_one(pool)
            .await?
    };
    if duplicate_form {
        return Err(ApiError::BadRequest(
            "this Sprint 2A runtime model supports one workflow per form".into(),
        ));
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
        ApiError::BadRequest("workflow versions must have a single primary response step".into())
    })?;
    let step_id: Uuid = row.try_get("id")?;

    if let Some(existing) = sqlx::query_scalar(
        r#"
        SELECT id
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
        sqlx::query(
            r#"
            UPDATE workflow_assignments
            SET workflow_version_id = $2,
                form_assignment_id = COALESCE($3, form_assignment_id),
                is_active = true
            WHERE id = $1
            "#,
        )
        .bind(existing)
        .bind(workflow_version_id)
        .bind(form_assignment_id)
        .execute(&mut **tx)
        .await?;
        return Ok(existing);
    }

    sqlx::query_scalar(
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
    .map_err(Into::into)
}

async fn can_access_workflow_assignment(
    _pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    assignee_account_id: Uuid,
) -> ApiResult<bool> {
    if account.is_admin() || account.is_operator() || account.account_id == assignee_account_id {
        return Ok(true);
    }

    Ok(account
        .delegations
        .iter()
        .any(|delegate| delegate.account_id == assignee_account_id))
}
