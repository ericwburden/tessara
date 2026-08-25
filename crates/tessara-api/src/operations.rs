use axum::{Json, Router, extract::State, http::HeaderMap, routing::get};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::Row;
use tessara_datasets_contract::{
    DATASET_CORE_BINDING_KEY, DATASET_MODULE_DEFINITION_ID, DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
    DATASET_OPERATIONS_STATUS_ACTION, DATASET_OPERATIONS_STATUS_PATH,
    DATASET_REVERSE_CONTRACT_VERSION, DatasetFreshnessState, DatasetOperationsStatusItem,
    DatasetOperationsStatusRequest, DatasetOperationsStatusResponse, DatasetProviderResultState,
    DatasetReadinessLabel,
};
use tessara_responses_contract::{
    RESPONSE_MODULE_DEFINITION_ID, RESPONSE_OPERATIONAL_STATUS_CONTRACT_ID,
    RESPONSE_OPERATIONS_STATUS_ACTION, RESPONSE_OPERATIONS_STATUS_BINDING_KEY,
    RESPONSE_OPERATIONS_STATUS_MEDIA_TYPE, RESPONSE_OPERATIONS_STATUS_PATH,
    RESPONSE_REVERSE_CONTRACT_VERSION, RESPONSE_REVERSE_SCHEMA_VERSION, ResponseOperationsStatus,
    ResponseOperationsStatusRequest, ResponseOperationsStatusResponse, ResponseProviderResultState,
};
use uuid::Uuid;

use crate::{
    auth::{self, AuthenticatedRequest, CapabilityBoundary},
    core_security::request_correlation_id_or_new,
    db::AppState,
    error::{ApiError, ApiResult},
    module_gateway::{
        CorePrivateProviderRequest, CorePrivateProviderResult, call_private_provider,
    },
};

#[derive(Serialize)]
pub struct OperationsStatus {
    pub summary: OperationsSummary,
    pub workflow_assignments: Vec<WorkflowAssignmentStatus>,
    pub dataset_readiness: DatasetReadiness,
    pub response_owner: ResponseOwnerStatus,
}

#[derive(Serialize)]
pub struct OperationsSummary {
    pub open_workflow_assignment_count: i64,
    pub draft_response_count: i64,
    pub dataset_attention_count: Option<i64>,
}

#[derive(Serialize)]
pub struct WorkflowAssignmentStatus {
    pub workflow_instance_id: Option<Uuid>,
    pub workflow_assignment_id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub workflow_version_label: Option<String>,
    pub node_id: Uuid,
    pub node_name: String,
    pub assignee_display_name: String,
    pub assignee_email: String,
    pub assignment_status: String,
    pub current_step_title: Option<String>,
    pub completed_step_count: i64,
    pub total_step_count: i64,
    pub draft_response_count: i64,
    pub submitted_response_count: i64,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct DatasetReadiness {
    pub state: DatasetProviderResultState,
    pub datasets: Vec<DatasetStatus>,
}

#[derive(Serialize)]
pub struct DatasetStatus {
    pub dataset_id: Uuid,
    pub dataset_name: String,
    pub revision_status: String,
    pub readiness: String,
    pub source_count: i64,
    pub field_count: i64,
    pub ready_response_count: i64,
    pub freshness: DatasetFreshnessState,
    pub sanitized_failure_code: Option<String>,
}

#[derive(Serialize)]
pub struct ResponseOwnerStatus {
    pub state: ResponseProviderResultState,
    pub status: Option<ResponseOperationsStatus>,
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/api/operations/status", get(get_operations_status))
}

pub async fn get_operations_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: AuthenticatedRequest,
) -> ApiResult<Json<OperationsStatus>> {
    request.require_capability("operations:view")?;
    let boundary =
        auth::capability_boundary(&state.pool, &request.account, "operations:view").await?;

    if matches!(boundary, CapabilityBoundary::None) {
        return Err(ApiError::Forbidden("operations:view".into()));
    }

    let workflow_assignments = load_workflow_assignments(&state.pool, &boundary).await?;
    let requested_scope_node_ids = requested_scope_node_ids(&state.pool, &boundary).await?;
    let dataset_readiness = load_dataset_readiness(
        &state,
        &request,
        requested_scope_node_ids,
        request_correlation_id_or_new(&headers),
    )
    .await?;
    let response_owner =
        load_response_owner_status(&state, &request, request_correlation_id_or_new(&headers))
            .await?;

    let summary = OperationsSummary {
        open_workflow_assignment_count: workflow_assignments
            .iter()
            .filter(|assignment| !assignment_has_all_steps_complete(assignment))
            .count() as i64,
        draft_response_count: workflow_assignments
            .iter()
            .map(|assignment| assignment.draft_response_count)
            .sum(),
        dataset_attention_count: dataset_attention_count(&dataset_readiness),
    };

    Ok(Json(OperationsStatus {
        summary,
        workflow_assignments,
        dataset_readiness,
        response_owner,
    }))
}

async fn load_workflow_assignments(
    pool: &sqlx::PgPool,
    boundary: &CapabilityBoundary,
) -> ApiResult<Vec<WorkflowAssignmentStatus>> {
    let scope_node_ids = match boundary {
        CapabilityBoundary::Global => None,
        CapabilityBoundary::Scoped(node_ids) => {
            if node_ids.is_empty() {
                return Ok(Vec::new());
            }
            Some(node_ids.clone())
        }
        CapabilityBoundary::None => return Ok(Vec::new()),
    };
    let rows = sqlx::query(workflow_assignments_sql())
        .bind(scope_node_ids)
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(|row| {
            let completed_step_count = row.try_get("completed_step_count")?;
            let total_step_count = row.try_get("total_step_count")?;
            let raw_assignment_status =
                display_status(row.try_get::<String, _>("assignment_status")?);
            let assignment_status = if raw_assignment_status == "Completed" {
                raw_assignment_status
            } else if total_step_count > 0 && completed_step_count >= total_step_count {
                "Steps Complete".to_string()
            } else {
                raw_assignment_status
            };

            Ok(WorkflowAssignmentStatus {
                workflow_instance_id: row.try_get("workflow_instance_id")?,
                workflow_assignment_id: row.try_get("workflow_assignment_id")?,
                workflow_id: row.try_get("workflow_id")?,
                workflow_name: row.try_get("workflow_name")?,
                workflow_version_label: row.try_get("workflow_version_label")?,
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                assignee_display_name: row.try_get("assignee_display_name")?,
                assignee_email: row.try_get("assignee_email")?,
                assignment_status,
                current_step_title: row.try_get("current_step_title")?,
                completed_step_count,
                total_step_count,
                draft_response_count: row.try_get("draft_response_count")?,
                submitted_response_count: row.try_get("submitted_response_count")?,
                started_at: row.try_get("started_at")?,
                completed_at: row.try_get("completed_at")?,
            })
        })
        .collect()
}

fn assignment_has_all_steps_complete(assignment: &WorkflowAssignmentStatus) -> bool {
    assignment.total_step_count > 0
        && assignment.completed_step_count >= assignment.total_step_count
}

fn workflow_assignments_sql() -> &'static str {
    r#"
        SELECT
            workflow_instances.id AS workflow_instance_id,
            workflow_assignments.id AS workflow_assignment_id,
            workflows.id AS workflow_id,
            workflows.name AS workflow_name,
            workflow_versions.version_label AS workflow_version_label,
            workflow_assignments.node_id,
            nodes.name AS node_name,
            accounts.display_name AS assignee_display_name,
            accounts.email AS assignee_email,
            COALESCE(workflow_instances.status, 'not_started') AS assignment_status,
            (
                SELECT workflow_steps.title
                FROM workflow_step_instances
                JOIN workflow_steps ON workflow_steps.id = workflow_step_instances.workflow_step_id
                WHERE workflow_step_instances.workflow_instance_id = workflow_instances.id
                  AND workflow_step_instances.status = 'in_progress'
                ORDER BY workflow_step_instances.started_at DESC, workflow_step_instances.id DESC
                LIMIT 1
            ) AS current_step_title,
            (
                SELECT COUNT(*)
                FROM workflow_step_instances
                WHERE workflow_step_instances.workflow_instance_id = workflow_instances.id
                  AND workflow_step_instances.status = 'completed'
            ) AS completed_step_count,
            (
                SELECT COUNT(*)
                FROM workflow_steps
                WHERE workflow_steps.workflow_version_id = workflow_assignments.workflow_version_id
            ) AS total_step_count,
            (
                SELECT COUNT(*)
                FROM workflow_response_projection
                WHERE workflow_response_projection.workflow_instance_id = workflow_instances.id
                  AND workflow_response_projection.response_state = 'draft'
            ) AS draft_response_count,
            (
                SELECT COUNT(*)
                FROM workflow_response_projection
                WHERE workflow_response_projection.workflow_instance_id = workflow_instances.id
                  AND workflow_response_projection.response_state = 'submitted'
            ) AS submitted_response_count,
            workflow_instances.created_at AS started_at,
            workflow_instances.completed_at
        FROM workflow_assignments
        JOIN workflow_versions ON workflow_versions.id = workflow_assignments.workflow_version_id
        JOIN workflows ON workflows.id = workflow_versions.workflow_id
        JOIN nodes ON nodes.id = workflow_assignments.node_id
        JOIN accounts ON accounts.id = workflow_assignments.account_id
        LEFT JOIN LATERAL (
            SELECT candidate_instances.*
            FROM workflow_instances AS candidate_instances
            WHERE candidate_instances.workflow_assignment_id = workflow_assignments.id
            ORDER BY
                (candidate_instances.status = 'in_progress') DESC,
                candidate_instances.created_at DESC,
                candidate_instances.id DESC
            LIMIT 1
        ) AS workflow_instances ON true
        WHERE ($1::uuid[] IS NULL OR workflow_assignments.node_id = ANY($1))
          AND (workflow_assignments.is_active OR workflow_instances.id IS NOT NULL)
        ORDER BY
            COALESCE(workflow_instances.created_at, workflow_assignments.created_at) DESC,
            workflow_assignments.id
        LIMIT 100
        "#
}

async fn requested_scope_node_ids(
    pool: &sqlx::PgPool,
    boundary: &CapabilityBoundary,
) -> ApiResult<Vec<Uuid>> {
    let mut node_ids = match boundary {
        CapabilityBoundary::Global => {
            sqlx::query_scalar("SELECT id FROM nodes ORDER BY id")
                .fetch_all(pool)
                .await?
        }
        CapabilityBoundary::Scoped(node_ids) => node_ids.clone(),
        CapabilityBoundary::None => Vec::new(),
    };
    node_ids.sort_unstable();
    node_ids.dedup();
    Ok(node_ids)
}

async fn load_dataset_readiness(
    state: &AppState,
    actor: &AuthenticatedRequest,
    requested_scope_node_ids: Vec<Uuid>,
    correlation_id: Uuid,
) -> ApiResult<DatasetReadiness> {
    let request = DatasetOperationsStatusRequest {
        schema_version: 1,
        requested_scope_node_ids,
    };
    let response = call_private_provider::<_, DatasetOperationsStatusResponse>(
        state,
        actor,
        CorePrivateProviderRequest {
            module_definition_id: DATASET_MODULE_DEFINITION_ID,
            expected_owner: None,
            dependency_binding: DATASET_CORE_BINDING_KEY,
            functional_contract: DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
            contract_version: DATASET_REVERSE_CONTRACT_VERSION,
            authorization_action: DATASET_OPERATIONS_STATUS_ACTION,
            path: DATASET_OPERATIONS_STATUS_PATH,
            media_type: "application/json",
            correlation_id,
            actor_capability: "operations:view",
            body: &request,
        },
    )
    .await?;
    Ok(match response {
        CorePrivateProviderResult::Response(response) => project_dataset_readiness(response),
        CorePrivateProviderResult::Unavailable => {
            unavailable_dataset_readiness(DatasetProviderResultState::Unavailable)
        }
        CorePrivateProviderResult::Undisclosed => {
            unavailable_dataset_readiness(DatasetProviderResultState::Undisclosed)
        }
    })
}

fn project_dataset_readiness(response: DatasetOperationsStatusResponse) -> DatasetReadiness {
    if response.validate().is_err() {
        return unavailable_dataset_readiness(DatasetProviderResultState::Unavailable);
    }
    let state = response.state;
    let Some(datasets) = response
        .items
        .into_iter()
        .map(project_dataset_status)
        .collect::<Option<Vec<_>>>()
    else {
        return unavailable_dataset_readiness(DatasetProviderResultState::Unavailable);
    };
    DatasetReadiness { state, datasets }
}

fn project_dataset_status(item: DatasetOperationsStatusItem) -> Option<DatasetStatus> {
    Some(DatasetStatus {
        dataset_id: item.dataset.dataset_id(),
        dataset_name: item.dataset_name,
        revision_status: item
            .revision_status
            .map(display_status)
            .unwrap_or_else(|| "Unavailable".into()),
        readiness: readiness_label(item.readiness).into(),
        source_count: i64::try_from(item.source_count).ok()?,
        field_count: i64::try_from(item.field_count).ok()?,
        ready_response_count: i64::try_from(item.ready_response_count).ok()?,
        freshness: item.freshness,
        sanitized_failure_code: item.sanitized_failure_code,
    })
}

const fn readiness_label(readiness: DatasetReadinessLabel) -> &'static str {
    match readiness {
        DatasetReadinessLabel::Ready => "Ready",
        DatasetReadinessLabel::NoReadyResponses => "No Ready Responses",
        DatasetReadinessLabel::Draft => "Draft",
        DatasetReadinessLabel::Superseded => "Superseded",
        DatasetReadinessLabel::Unavailable => "Unavailable",
        DatasetReadinessLabel::NoPublishedRevision => "No Published Revision",
    }
}

fn unavailable_dataset_readiness(state: DatasetProviderResultState) -> DatasetReadiness {
    DatasetReadiness {
        state,
        datasets: Vec::new(),
    }
}

fn dataset_attention_count(readiness: &DatasetReadiness) -> Option<i64> {
    match readiness.state {
        DatasetProviderResultState::Available | DatasetProviderResultState::Empty => i64::try_from(
            readiness
                .datasets
                .iter()
                .filter(|dataset| dataset.readiness != "Ready")
                .count(),
        )
        .ok(),
        DatasetProviderResultState::Unavailable | DatasetProviderResultState::Undisclosed => None,
    }
}

async fn load_response_owner_status(
    state: &AppState,
    actor: &AuthenticatedRequest,
    correlation_id: Uuid,
) -> ApiResult<ResponseOwnerStatus> {
    let request = ResponseOperationsStatusRequest {
        schema_version: RESPONSE_REVERSE_SCHEMA_VERSION,
    };
    let response = call_private_provider::<_, ResponseOperationsStatusResponse>(
        state,
        actor,
        CorePrivateProviderRequest {
            module_definition_id: RESPONSE_MODULE_DEFINITION_ID,
            expected_owner: None,
            dependency_binding: RESPONSE_OPERATIONS_STATUS_BINDING_KEY,
            functional_contract: RESPONSE_OPERATIONAL_STATUS_CONTRACT_ID,
            contract_version: RESPONSE_REVERSE_CONTRACT_VERSION,
            authorization_action: RESPONSE_OPERATIONS_STATUS_ACTION,
            path: RESPONSE_OPERATIONS_STATUS_PATH,
            media_type: RESPONSE_OPERATIONS_STATUS_MEDIA_TYPE,
            correlation_id,
            actor_capability: "operations:view",
            body: &request,
        },
    )
    .await?;
    Ok(match response {
        CorePrivateProviderResult::Response(response) if response.validate().is_ok() => {
            ResponseOwnerStatus {
                state: response.state,
                status: response.status,
            }
        }
        CorePrivateProviderResult::Response(_) | CorePrivateProviderResult::Unavailable => {
            ResponseOwnerStatus {
                state: ResponseProviderResultState::Unavailable,
                status: None,
            }
        }
        CorePrivateProviderResult::Undisclosed => ResponseOwnerStatus {
            state: ResponseProviderResultState::Undisclosed,
            status: None,
        },
    })
}

fn display_status(status: String) -> String {
    status
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use tessara_datasets_contract::{
        DatasetOperationsStatusResponse, DatasetProviderResultState, DatasetReadinessLabel,
    };

    use super::*;

    #[test]
    fn readiness_vocabulary_is_exactly_the_frozen_operations_vocabulary() {
        assert_eq!(readiness_label(DatasetReadinessLabel::Ready), "Ready");
        assert_eq!(
            readiness_label(DatasetReadinessLabel::NoReadyResponses),
            "No Ready Responses"
        );
        assert_eq!(readiness_label(DatasetReadinessLabel::Draft), "Draft");
        assert_eq!(
            readiness_label(DatasetReadinessLabel::Superseded),
            "Superseded"
        );
        assert_eq!(
            readiness_label(DatasetReadinessLabel::Unavailable),
            "Unavailable"
        );
        assert_eq!(
            readiness_label(DatasetReadinessLabel::NoPublishedRevision),
            "No Published Revision"
        );
    }

    #[test]
    fn dataset_outage_never_becomes_an_attention_zero() {
        for state in [
            DatasetProviderResultState::Unavailable,
            DatasetProviderResultState::Undisclosed,
        ] {
            let readiness = unavailable_dataset_readiness(state);
            assert_eq!(dataset_attention_count(&readiness), None);
            assert!(readiness.datasets.is_empty());
        }
        assert_eq!(
            dataset_attention_count(&unavailable_dataset_readiness(
                DatasetProviderResultState::Empty
            )),
            Some(0)
        );
    }

    #[test]
    fn invalid_owner_response_is_projected_as_unavailable() {
        let readiness = project_dataset_readiness(DatasetOperationsStatusResponse {
            schema_version: 1,
            state: DatasetProviderResultState::Available,
            items: Vec::new(),
        });
        assert_eq!(readiness.state, DatasetProviderResultState::Unavailable);
        assert_eq!(dataset_attention_count(&readiness), None);
    }
}
