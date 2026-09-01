//! Workflow-owned assignment catalog and one-use Response start reservations.

use axum::{
    Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::Response,
    routing::post,
};
use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::{Postgres, Row, Transaction};
use tessara_module_contract::ModuleServicePrincipalV1;
use tessara_responses_contract::RESPONSE_MODULE_DEFINITION_ID;
use tessara_workflows_contract::{
    WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION, WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID,
    WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_MEDIA_TYPE, WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH,
    WORKFLOW_RESPONSE_CONTEXT_ACTION, WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID,
    WORKFLOW_RESPONSE_CONTEXT_MEDIA_TYPE, WORKFLOW_RESPONSE_CONTEXT_PATH,
    WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION, WorkflowResponseAssignmentCatalogItem,
    WorkflowResponseAssignmentCatalogRequest, WorkflowResponseAssignmentCatalogResponse,
    WorkflowResponseContextRequest, WorkflowResponseContextResponse, WorkflowResponseContextState,
    WorkflowResponseStartContext, WorkflowResponseStepSnapshot,
};
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{ApiError, ApiResult},
    module_service_requests::{CoreProviderAuthorizationV1, authorize_core_provider},
};

const RESERVATION_LIFETIME_SECONDS: i64 = 300;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH,
            post(assignment_catalog),
        )
        .route(WORKFLOW_RESPONSE_CONTEXT_PATH, post(response_context))
}

fn require_response_presenter(grant: &CoreProviderAuthorizationV1) -> ApiResult<()> {
    match &grant.payload.presenting_service {
        ModuleServicePrincipalV1::ModuleInstance {
            module_definition_id,
            ..
        } if module_definition_id.as_str() == RESPONSE_MODULE_DEFINITION_ID => Ok(()),
        _ => Err(restricted()),
    }
}

async fn assignment_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    require_media_type(&headers, WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_MEDIA_TYPE)?;
    let grant = authorize_core_provider(
        &state,
        &headers,
        WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID,
        WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION,
        WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH,
        &body,
        "Workflow Response assignments are unavailable",
    )
    .await?;
    require_response_presenter(&grant)?;
    let request: WorkflowResponseAssignmentCatalogRequest =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    if request.schema_version != WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION {
        return Err(restricted());
    }
    if !can_act_for(&grant, request.assignee_account_id) {
        return contract_response(
            &WorkflowResponseAssignmentCatalogResponse {
                schema_version: WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION,
                state: WorkflowResponseContextState::Undisclosed,
                assignments: Vec::new(),
            },
            WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_MEDIA_TYPE,
        );
    }
    let rows = sqlx::query(
        r#"
        SELECT
            wa.id AS workflow_assignment_id,
            w.id AS workflow_id,
            w.name AS workflow_name,
            w.description AS workflow_description,
            wv.id AS workflow_version_id,
            wv.version_label AS workflow_version_label,
            ws.id AS workflow_step_id,
            ws.title AS workflow_step_title,
            ws.position AS workflow_step_position,
            (SELECT COUNT(*) FROM workflow_steps all_steps
             WHERE all_steps.workflow_version_id=ws.workflow_version_id) AS workflow_step_count,
            next_steps.title AS next_workflow_step_title,
            next_forms.name AS next_workflow_step_form_name,
            f.id AS form_id,
            f.name AS form_name,
            ws.form_version_id,
            fv.version_label AS form_version_label,
            n.id AS node_id,
            n.name AS node_name,
            a.id AS assignee_account_id,
            a.display_name AS assignee_display_name
        FROM workflow_assignments wa
        JOIN workflow_versions wv ON wv.id=wa.workflow_version_id
        JOIN workflows w ON w.id=wv.workflow_id
        JOIN workflow_steps ws ON ws.id=wa.workflow_step_id
        JOIN form_versions fv ON fv.id=ws.form_version_id
        JOIN forms f ON f.id=fv.form_id
        JOIN nodes n ON n.id=wa.node_id
        JOIN accounts a ON a.id=wa.account_id
        LEFT JOIN workflow_steps next_steps
          ON next_steps.workflow_version_id=ws.workflow_version_id
         AND next_steps.position=ws.position+1
        LEFT JOIN form_versions next_fv ON next_fv.id=next_steps.form_version_id
        LEFT JOIN forms next_forms ON next_forms.id=next_fv.form_id
        WHERE wa.account_id=$1
          AND wa.is_active=true
          AND wv.status IN ('published'::form_version_status,'superseded'::form_version_status)
          AND NOT EXISTS(SELECT 1 FROM workflow_response_projection p
                         WHERE p.workflow_assignment_id=wa.id
                           AND p.response_state IN ('draft','submitted'))
          AND NOT EXISTS(SELECT 1 FROM workflow_response_reservations r
                         WHERE r.workflow_assignment_id=wa.id
                           AND r.consumed_at IS NULL)
        ORDER BY wa.id
        "#,
    )
    .bind(request.assignee_account_id)
    .fetch_all(&state.pool)
    .await?;
    let assignments = rows
        .into_iter()
        .filter(|row| scope_authorizes(&grant, row.get("node_id")))
        .map(catalog_item)
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let response = WorkflowResponseAssignmentCatalogResponse {
        schema_version: WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION,
        state: WorkflowResponseContextState::Available,
        assignments,
    };
    response
        .validate_for(request.assignee_account_id)
        .map_err(|_| restricted())?;
    contract_response(&response, WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_MEDIA_TYPE)
}

async fn response_context(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    require_media_type(&headers, WORKFLOW_RESPONSE_CONTEXT_MEDIA_TYPE)?;
    let grant = authorize_core_provider(
        &state,
        &headers,
        WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID,
        WORKFLOW_RESPONSE_CONTEXT_ACTION,
        WORKFLOW_RESPONSE_CONTEXT_PATH,
        &body,
        "Workflow Response context is unavailable",
    )
    .await?;
    require_response_presenter(&grant)?;
    let request: WorkflowResponseContextRequest =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    if request.schema_version != WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION
        || request.workflow_assignment_id.is_nil()
    {
        return Err(restricted());
    }
    let mut transaction = state.pool.begin().await?;
    let Some(row) = load_assignment(&mut transaction, request.workflow_assignment_id).await? else {
        return context_state(WorkflowResponseContextState::Undisclosed, None);
    };
    let assignee_account_id: Uuid = row.get("assignee_account_id");
    let node_id: Uuid = row.get("node_id");
    if !can_start_assignment(&grant, assignee_account_id, node_id)
        || !row.get::<bool, _>("is_active")
        || !matches!(
            row.get::<String, _>("workflow_status").as_str(),
            "published" | "superseded"
        )
        || response_already_exists(&mut transaction, request.workflow_assignment_id).await?
    {
        return context_state(WorkflowResponseContextState::Undisclosed, None);
    }
    if let Some(existing) = active_reservation(
        &mut transaction,
        request.workflow_assignment_id,
        grant.payload.original_actor_id,
    )
    .await?
    {
        transaction.commit().await?;
        return context_state(WorkflowResponseContextState::Available, Some(existing));
    }
    if unresolved_reservation(&mut transaction, request.workflow_assignment_id).await? {
        transaction.commit().await?;
        return context_state(WorkflowResponseContextState::Unavailable, None);
    }
    let workflow_instance_id =
        resolve_workflow_instance(&mut transaction, &row, grant.payload.original_actor_id).await?;
    let position: i32 = row.get("workflow_step_position");
    if position > 0
        && !previous_step_completed(
            &mut transaction,
            workflow_instance_id,
            row.get("workflow_version_id"),
            position - 1,
        )
        .await?
    {
        return context_state(WorkflowResponseContextState::Undisclosed, None);
    }
    let workflow_step_instance_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_step_instances(workflow_instance_id,workflow_step_id,status) VALUES($1,$2,'in_progress') RETURNING id",
    )
    .bind(workflow_instance_id)
    .bind(row.get::<Uuid, _>("workflow_step_id"))
    .fetch_one(&mut *transaction)
    .await?;
    let history = load_history(&mut transaction, workflow_instance_id).await?;
    let now = Utc::now();
    let expires_at = now + Duration::seconds(RESERVATION_LIFETIME_SECONDS);
    let context = WorkflowResponseStartContext {
        schema_version: WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION,
        workflow_assignment_id: request.workflow_assignment_id,
        workflow_id: row.get("workflow_id"),
        workflow_name: row.get("workflow_name"),
        workflow_description: row.get("workflow_description"),
        workflow_version_id: row.get("workflow_version_id"),
        workflow_version_label: row.get("workflow_version_label"),
        workflow_step_id: row.get("workflow_step_id"),
        workflow_step_title: row.get("workflow_step_title"),
        workflow_step_position: position,
        workflow_step_count: row.get("workflow_step_count"),
        next_workflow_step_title: row.get("next_workflow_step_title"),
        next_workflow_step_form_name: row.get("next_workflow_step_form_name"),
        history,
        workflow_instance_id,
        workflow_step_instance_id,
        form_id: row.get("form_id"),
        form_version_id: row.get("form_version_id"),
        node_id,
        node_name: row.get("node_name"),
        assignee_account_id,
        assignee_display_name: row.get("assignee_display_name"),
        started_by_account_id: grant.payload.original_actor_id,
        delegation_basis: delegation_label(&grant, assignee_account_id),
        one_use_nonce: Uuid::new_v4(),
        issued_at: now.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
        context_digest: String::new(),
    }
    .with_recomputed_digest()
    .map_err(|_| restricted())?;
    sqlx::query("INSERT INTO workflow_response_reservations(workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,started_by_account_id,one_use_nonce,context_payload,context_digest,expires_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(context.workflow_assignment_id)
        .bind(context.workflow_instance_id)
        .bind(context.workflow_step_instance_id)
        .bind(context.started_by_account_id)
        .bind(context.one_use_nonce)
        .bind(serde_json::to_value(&context).map_err(|error| ApiError::Internal(error.into()))?)
        .bind(&context.context_digest)
        .bind(expires_at)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    context_state(WorkflowResponseContextState::Available, Some(context))
}

async fn load_assignment(
    transaction: &mut Transaction<'_, Postgres>,
    assignment_id: Uuid,
) -> ApiResult<Option<sqlx::postgres::PgRow>> {
    Ok(sqlx::query(
        r#"
        SELECT wa.id,wa.is_active,wa.workflow_version_id,wa.workflow_step_id,
               wa.node_id,wa.account_id AS assignee_account_id,
               w.id AS workflow_id,w.name AS workflow_name,w.description AS workflow_description,
               wv.version_label AS workflow_version_label,wv.status::text AS workflow_status,
               ws.title AS workflow_step_title,ws.position AS workflow_step_position,
               (SELECT COUNT(*) FROM workflow_steps all_steps
                WHERE all_steps.workflow_version_id=ws.workflow_version_id) AS workflow_step_count,
               next_steps.title AS next_workflow_step_title,
               next_forms.name AS next_workflow_step_form_name,
               f.id AS form_id,f.name AS form_name,ws.form_version_id,
               fv.version_label AS form_version_label,n.name AS node_name,
               a.display_name AS assignee_display_name
        FROM workflow_assignments wa
        JOIN workflow_versions wv ON wv.id=wa.workflow_version_id
        JOIN workflows w ON w.id=wv.workflow_id
        JOIN workflow_steps ws ON ws.id=wa.workflow_step_id
        JOIN form_versions fv ON fv.id=ws.form_version_id
        JOIN forms f ON f.id=fv.form_id
        JOIN nodes n ON n.id=wa.node_id
        JOIN accounts a ON a.id=wa.account_id
        LEFT JOIN workflow_steps next_steps
          ON next_steps.workflow_version_id=ws.workflow_version_id
         AND next_steps.position=ws.position+1
        LEFT JOIN form_versions next_fv ON next_fv.id=next_steps.form_version_id
        LEFT JOIN forms next_forms ON next_forms.id=next_fv.form_id
        WHERE wa.id=$1
        FOR UPDATE OF wa
        "#,
    )
    .bind(assignment_id)
    .fetch_optional(&mut **transaction)
    .await?)
}

async fn response_already_exists(
    transaction: &mut Transaction<'_, Postgres>,
    assignment_id: Uuid,
) -> ApiResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM workflow_response_projection WHERE workflow_assignment_id=$1 AND response_state IN ('draft','submitted'))",
    )
    .bind(assignment_id)
    .fetch_one(&mut **transaction)
    .await?)
}

async fn active_reservation(
    transaction: &mut Transaction<'_, Postgres>,
    assignment_id: Uuid,
    actor_id: Uuid,
) -> ApiResult<Option<WorkflowResponseStartContext>> {
    let payload: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT context_payload FROM workflow_response_reservations WHERE workflow_assignment_id=$1 AND started_by_account_id=$2 AND consumed_at IS NULL AND expires_at>now()",
    )
    .bind(assignment_id)
    .bind(actor_id)
    .fetch_optional(&mut **transaction)
    .await?;
    payload
        .map(|payload| {
            let context: WorkflowResponseStartContext =
                serde_json::from_value(payload).map_err(|_| restricted())?;
            context
                .validate_for(assignment_id)
                .map_err(|_| restricted())?;
            Ok(context)
        })
        .transpose()
}

async fn unresolved_reservation(
    transaction: &mut Transaction<'_, Postgres>,
    assignment_id: Uuid,
) -> ApiResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM workflow_response_reservations WHERE workflow_assignment_id=$1 AND consumed_at IS NULL)",
    )
    .bind(assignment_id)
    .fetch_one(&mut **transaction)
    .await?)
}

async fn resolve_workflow_instance(
    transaction: &mut Transaction<'_, Postgres>,
    row: &sqlx::postgres::PgRow,
    started_by_account_id: Uuid,
) -> ApiResult<Uuid> {
    if let Some(existing) = sqlx::query_scalar(
        "SELECT id FROM workflow_instances WHERE workflow_version_id=$1 AND node_id=$2 AND assignee_account_id=$3 AND status='in_progress' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(row.get::<Uuid, _>("workflow_version_id"))
    .bind(row.get::<Uuid, _>("node_id"))
    .bind(row.get::<Uuid, _>("assignee_account_id"))
    .fetch_optional(&mut **transaction)
    .await?
    {
        return Ok(existing);
    }
    Ok(sqlx::query_scalar("INSERT INTO workflow_instances(workflow_assignment_id,workflow_version_id,node_id,assignee_account_id,started_by_account_id) VALUES($1,$2,$3,$4,$5) RETURNING id")
        .bind(row.get::<Uuid, _>("id"))
        .bind(row.get::<Uuid, _>("workflow_version_id"))
        .bind(row.get::<Uuid, _>("node_id"))
        .bind(row.get::<Uuid, _>("assignee_account_id"))
        .bind(started_by_account_id)
        .fetch_one(&mut **transaction)
        .await?)
}

pub(crate) async fn issue_bootstrap_context_tx(
    transaction: &mut Transaction<'_, Postgres>,
    workflow_assignment_id: Uuid,
    actor_account_id: Uuid,
) -> ApiResult<WorkflowResponseStartContext> {
    let row = load_assignment(transaction, workflow_assignment_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("Workflow bootstrap assignment is missing".into()))?;
    if row.get::<Uuid, _>("assignee_account_id") != actor_account_id
        || !row.get::<bool, _>("is_active")
        || !matches!(
            row.get::<String, _>("workflow_status").as_str(),
            "published" | "superseded"
        )
        || response_already_exists(transaction, workflow_assignment_id).await?
        || unresolved_reservation(transaction, workflow_assignment_id).await?
    {
        return Err(ApiError::BadRequest(
            "Workflow bootstrap assignment is not startable".into(),
        ));
    }
    let workflow_instance_id =
        resolve_workflow_instance(transaction, &row, actor_account_id).await?;
    let position: i32 = row.get("workflow_step_position");
    if position > 0
        && !previous_step_completed(
            transaction,
            workflow_instance_id,
            row.get("workflow_version_id"),
            position - 1,
        )
        .await?
    {
        return Err(ApiError::BadRequest(
            "Workflow bootstrap assignment has an incomplete predecessor".into(),
        ));
    }
    let workflow_step_instance_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_step_instances(workflow_instance_id,workflow_step_id,status) VALUES($1,$2,'in_progress') RETURNING id",
    )
    .bind(workflow_instance_id)
    .bind(row.get::<Uuid, _>("workflow_step_id"))
    .fetch_one(&mut **transaction)
    .await?;
    let now = Utc::now();
    let expires_at = now + Duration::seconds(RESERVATION_LIFETIME_SECONDS);
    let context = WorkflowResponseStartContext {
        schema_version: WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION,
        workflow_assignment_id,
        workflow_id: row.get("workflow_id"),
        workflow_name: row.get("workflow_name"),
        workflow_description: row.get("workflow_description"),
        workflow_version_id: row.get("workflow_version_id"),
        workflow_version_label: row.get("workflow_version_label"),
        workflow_step_id: row.get("workflow_step_id"),
        workflow_step_title: row.get("workflow_step_title"),
        workflow_step_position: position,
        workflow_step_count: row.get("workflow_step_count"),
        next_workflow_step_title: row.get("next_workflow_step_title"),
        next_workflow_step_form_name: row.get("next_workflow_step_form_name"),
        history: load_history(transaction, workflow_instance_id).await?,
        workflow_instance_id,
        workflow_step_instance_id,
        form_id: row.get("form_id"),
        form_version_id: row.get("form_version_id"),
        node_id: row.get("node_id"),
        node_name: row.get("node_name"),
        assignee_account_id: actor_account_id,
        assignee_display_name: row.get("assignee_display_name"),
        started_by_account_id: actor_account_id,
        delegation_basis: None,
        one_use_nonce: Uuid::new_v4(),
        issued_at: now.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
        context_digest: String::new(),
    }
    .with_recomputed_digest()
    .map_err(|_| ApiError::BadRequest("Workflow bootstrap context is invalid".into()))?;
    sqlx::query("INSERT INTO workflow_response_reservations(workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,started_by_account_id,one_use_nonce,context_payload,context_digest,expires_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(context.workflow_assignment_id)
        .bind(context.workflow_instance_id)
        .bind(context.workflow_step_instance_id)
        .bind(context.started_by_account_id)
        .bind(context.one_use_nonce)
        .bind(serde_json::to_value(&context).map_err(|error| ApiError::Internal(error.into()))?)
        .bind(&context.context_digest)
        .bind(expires_at)
        .execute(&mut **transaction)
        .await?;
    Ok(context)
}

async fn previous_step_completed(
    transaction: &mut Transaction<'_, Postgres>,
    instance_id: Uuid,
    workflow_version_id: Uuid,
    previous_position: i32,
) -> ApiResult<bool> {
    Ok(sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM workflow_step_instances wsi JOIN workflow_steps ws ON ws.id=wsi.workflow_step_id WHERE wsi.workflow_instance_id=$1 AND ws.workflow_version_id=$2 AND ws.position=$3 AND wsi.status='completed')")
        .bind(instance_id)
        .bind(workflow_version_id)
        .bind(previous_position)
        .fetch_one(&mut **transaction)
        .await?)
}

async fn load_history(
    transaction: &mut Transaction<'_, Postgres>,
    workflow_instance_id: Uuid,
) -> ApiResult<Vec<WorkflowResponseStepSnapshot>> {
    Ok(sqlx::query("SELECT ws.id AS workflow_step_id,ws.title,f.name AS form_name,wsi.status,ws.position,wsi.completed_at FROM workflow_step_instances wsi JOIN workflow_steps ws ON ws.id=wsi.workflow_step_id JOIN form_versions fv ON fv.id=ws.form_version_id JOIN forms f ON f.id=fv.form_id WHERE wsi.workflow_instance_id=$1 AND wsi.status='completed' ORDER BY ws.position")
        .bind(workflow_instance_id)
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .map(|row| Ok(WorkflowResponseStepSnapshot {
            workflow_step_id: row.try_get("workflow_step_id")?,
            title: row.try_get("title")?,
            form_name: row.try_get("form_name")?,
            status: row.try_get("status")?,
            position: row.try_get("position")?,
            completed_at: row.try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at")?.map(|value| value.to_rfc3339()),
        }))
        .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

fn catalog_item(
    row: sqlx::postgres::PgRow,
) -> Result<WorkflowResponseAssignmentCatalogItem, sqlx::Error> {
    Ok(WorkflowResponseAssignmentCatalogItem {
        workflow_assignment_id: row.try_get("workflow_assignment_id")?,
        workflow_id: row.try_get("workflow_id")?,
        workflow_name: row.try_get("workflow_name")?,
        workflow_description: row.try_get("workflow_description")?,
        workflow_version_id: row.try_get("workflow_version_id")?,
        workflow_version_label: row.try_get("workflow_version_label")?,
        workflow_step_id: row.try_get("workflow_step_id")?,
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
        assignee_account_id: row.try_get("assignee_account_id")?,
        assignee_display_name: row.try_get("assignee_display_name")?,
    })
}

fn can_act_for(grant: &CoreProviderAuthorizationV1, assignee_account_id: Uuid) -> bool {
    grant.payload.original_actor_id == assignee_account_id
        || grant
            .payload
            .delegation_basis
            .iter()
            .any(|basis| basis.delegated_by_actor_id == assignee_account_id)
}

fn scope_authorizes(grant: &CoreProviderAuthorizationV1, node_id: Uuid) -> bool {
    capability_scope_authorizes(grant, node_id, "submissions:respond")
        || capability_scope_authorizes(grant, node_id, "submissions:manage")
}

fn manage_scope_authorizes(grant: &CoreProviderAuthorizationV1, node_id: Uuid) -> bool {
    capability_scope_authorizes(grant, node_id, "submissions:manage")
}

fn capability_scope_authorizes(
    grant: &CoreProviderAuthorizationV1,
    node_id: Uuid,
    capability: &str,
) -> bool {
    grant
        .payload
        .capability_scope_bindings
        .iter()
        .any(|binding| {
            binding.capability.as_str() == capability
                && (binding.organization_root_id == grant.payload.installation_id
                    || binding.organization_root_id == node_id
                    || binding.authorized_organization_ids.contains(&node_id))
        })
}

fn can_start_assignment(
    grant: &CoreProviderAuthorizationV1,
    assignee_account_id: Uuid,
    node_id: Uuid,
) -> bool {
    scope_authorizes(grant, node_id)
        && (can_act_for(grant, assignee_account_id) || manage_scope_authorizes(grant, node_id))
}

fn delegation_label(
    grant: &CoreProviderAuthorizationV1,
    assignee_account_id: Uuid,
) -> Option<String> {
    grant
        .payload
        .delegation_basis
        .iter()
        .find(|basis| basis.delegated_by_actor_id == assignee_account_id)
        .map(|basis| format!("delegation:{}", basis.delegation_id))
}

fn require_media_type(headers: &HeaderMap, expected: &str) -> ApiResult<()> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(expected)
    {
        return Err(restricted());
    }
    Ok(())
}

fn context_state(
    state: WorkflowResponseContextState,
    context: Option<WorkflowResponseStartContext>,
) -> ApiResult<Response> {
    let response = WorkflowResponseContextResponse {
        schema_version: WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION,
        state,
        context,
    };
    contract_response(&response, WORKFLOW_RESPONSE_CONTEXT_MEDIA_TYPE)
}

fn contract_response<T: Serialize>(value: &T, media_type: &str) -> ApiResult<Response> {
    let body = serde_json::to_vec(value)
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media_type)
        .body(Body::from(body))
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))
}

fn restricted() -> ApiError {
    ApiError::NotFound("Workflow Response context is unavailable".into())
}

#[cfg(test)]
mod tests {
    use tessara_module_contract::{
        CapabilityScopeBindingV1, DelegationBasisV1, ModuleServicePrincipalV1, SecurityCapabilityId,
    };

    use super::*;

    fn grant(actor: Uuid, assignee: Uuid, node: Uuid) -> CoreProviderAuthorizationV1 {
        CoreProviderAuthorizationV1 {
            payload: crate::module_service_requests::CoreProviderAuthorizationPayloadV1 {
                installation_id: Uuid::from_u128(1),
                original_actor_id: actor,
                presenting_service: ModuleServicePrincipalV1::CoreGateway,
                capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                    capability: SecurityCapabilityId::new("submissions:respond").unwrap(),
                    organization_root_id: node,
                    authorized_organization_ids: Vec::new(),
                }],
                delegation_basis: (actor != assignee)
                    .then(|| DelegationBasisV1 {
                        delegation_id: Uuid::from_u128(9),
                        delegated_by_actor_id: assignee,
                        capability: SecurityCapabilityId::new("submissions:respond").unwrap(),
                        organization_root_id: node,
                    })
                    .into_iter()
                    .collect(),
            },
        }
    }

    #[test]
    fn provider_authority_is_actor_delegation_and_scope_bound() {
        let actor = Uuid::from_u128(2);
        let assignee = Uuid::from_u128(3);
        let node = Uuid::from_u128(4);
        let direct = grant(actor, actor, node);
        assert!(can_act_for(&direct, actor));
        assert!(scope_authorizes(&direct, node));
        assert!(!can_act_for(&direct, assignee));
        let delegated = grant(actor, assignee, node);
        assert!(can_act_for(&delegated, assignee));
        assert_eq!(
            delegation_label(&delegated, assignee),
            Some(format!("delegation:{}", Uuid::from_u128(9)))
        );
        assert!(!scope_authorizes(&delegated, Uuid::from_u128(99)));
    }

    #[test]
    fn scoped_management_can_start_another_assignees_work_without_broadening_respond() {
        let actor = Uuid::from_u128(2);
        let assignee = Uuid::from_u128(3);
        let node = Uuid::from_u128(4);
        let mut respondent = grant(actor, assignee, node);
        respondent.payload.delegation_basis.clear();
        assert!(!can_start_assignment(&respondent, assignee, node));

        let mut manager = respondent;
        manager.payload.capability_scope_bindings[0].capability =
            SecurityCapabilityId::new("submissions:manage").unwrap();
        assert!(can_start_assignment(&manager, assignee, node));
        assert!(!can_start_assignment(
            &manager,
            assignee,
            Uuid::from_u128(99)
        ));
    }
}
