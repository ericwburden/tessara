use axum::{Json, Router, extract::State, http::HeaderMap, routing::get};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::{self, CapabilityBoundary},
    db::AppState,
    error::{ApiError, ApiResult},
};

#[derive(Serialize)]
pub struct OperationsStatus {
    pub summary: OperationsSummary,
    pub workflow_assignments: Vec<WorkflowAssignmentStatus>,
    pub dataset_readiness: DatasetReadiness,
    pub reporting_data: ReportingDataStatus,
}

#[derive(Serialize)]
pub struct OperationsSummary {
    pub open_workflow_assignment_count: i64,
    pub draft_response_count: i64,
    pub dataset_attention_count: i64,
}

#[derive(Serialize)]
pub struct WorkflowAssignmentStatus {
    pub workflow_instance_id: Uuid,
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
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct DatasetReadiness {
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
}

#[derive(Serialize)]
pub struct ReportingDataStatus {
    pub status: String,
    pub reporting_node_count: i64,
    pub submitted_response_count: i64,
    pub response_value_count: i64,
    pub message: String,
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/api/operations/status", get(get_operations_status))
}

pub async fn get_operations_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<OperationsStatus>> {
    let account = auth::require_capability(&state.pool, &headers, "operations:view").await?;
    let boundary = auth::capability_boundary(&state.pool, &account, "operations:view").await?;

    if matches!(boundary, CapabilityBoundary::None) {
        return Err(ApiError::Forbidden("operations:view".into()));
    }

    let workflow_assignments = load_workflow_assignments(&state.pool, &boundary).await?;
    let datasets = load_dataset_readiness(&state.pool, &boundary).await?;
    let reporting_data = load_reporting_data_status(&state.pool, &boundary).await?;

    let summary = OperationsSummary {
        open_workflow_assignment_count: workflow_assignments
            .iter()
            .filter(|assignment| !assignment_has_all_steps_complete(assignment))
            .count() as i64,
        draft_response_count: workflow_assignments
            .iter()
            .map(|assignment| assignment.draft_response_count)
            .sum(),
        dataset_attention_count: datasets
            .iter()
            .filter(|dataset| dataset.readiness != "Ready")
            .count() as i64,
    };

    Ok(Json(OperationsStatus {
        summary,
        workflow_assignments,
        dataset_readiness: DatasetReadiness { datasets },
        reporting_data,
    }))
}

async fn load_workflow_assignments(
    pool: &sqlx::PgPool,
    boundary: &CapabilityBoundary,
) -> ApiResult<Vec<WorkflowAssignmentStatus>> {
    let rows = match boundary {
        CapabilityBoundary::Global => {
            sqlx::query(workflow_assignments_sql(false))
                .fetch_all(pool)
                .await?
        }
        CapabilityBoundary::Scoped(node_ids) => {
            if node_ids.is_empty() {
                return Ok(Vec::new());
            }
            sqlx::query(workflow_assignments_sql(true))
                .bind(node_ids)
                .fetch_all(pool)
                .await?
        }
        CapabilityBoundary::None => return Ok(Vec::new()),
    };

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

fn workflow_assignments_sql(scoped: bool) -> &'static str {
    if scoped {
        r#"
        SELECT
            workflow_instances.id AS workflow_instance_id,
            workflow_assignments.id AS workflow_assignment_id,
            workflows.id AS workflow_id,
            workflows.name AS workflow_name,
            workflow_versions.version_label AS workflow_version_label,
            workflow_instances.node_id,
            nodes.name AS node_name,
            accounts.display_name AS assignee_display_name,
            accounts.email AS assignee_email,
            workflow_instances.status AS assignment_status,
            current_steps.title AS current_step_title,
            COUNT(DISTINCT completed_step_instances.id) AS completed_step_count,
            COUNT(DISTINCT workflow_steps.id) AS total_step_count,
            COUNT(DISTINCT submissions.id) FILTER (WHERE submissions.status = 'draft') AS draft_response_count,
            COUNT(DISTINCT submissions.id) FILTER (WHERE submissions.status = 'submitted') AS submitted_response_count,
            workflow_instances.created_at AS started_at,
            workflow_instances.completed_at
        FROM workflow_instances
        JOIN workflow_assignments ON workflow_assignments.id = workflow_instances.workflow_assignment_id
        JOIN workflow_versions ON workflow_versions.id = workflow_instances.workflow_version_id
        JOIN workflows ON workflows.id = workflow_versions.workflow_id
        JOIN nodes ON nodes.id = workflow_instances.node_id
        JOIN accounts ON accounts.id = workflow_instances.assignee_account_id
        LEFT JOIN workflow_step_instances AS active_step_instances
            ON active_step_instances.workflow_instance_id = workflow_instances.id
           AND active_step_instances.status = 'in_progress'
        LEFT JOIN workflow_steps AS current_steps ON current_steps.id = active_step_instances.workflow_step_id
        LEFT JOIN workflow_step_instances AS completed_step_instances
            ON completed_step_instances.workflow_instance_id = workflow_instances.id
           AND completed_step_instances.status = 'completed'
        LEFT JOIN workflow_steps ON workflow_steps.workflow_version_id = workflow_instances.workflow_version_id
        LEFT JOIN submissions ON submissions.workflow_instance_id = workflow_instances.id
        WHERE workflow_instances.node_id = ANY($1)
        GROUP BY
            workflow_instances.id,
            workflow_assignments.id,
            workflows.id,
            workflows.name,
            workflow_versions.version_label,
            workflow_instances.node_id,
            nodes.name,
            accounts.display_name,
            accounts.email,
            workflow_instances.status,
            current_steps.title,
            workflow_instances.created_at,
            workflow_instances.completed_at
        ORDER BY workflow_instances.created_at DESC, workflow_instances.id
        LIMIT 100
        "#
    } else {
        r#"
        SELECT
            workflow_instances.id AS workflow_instance_id,
            workflow_assignments.id AS workflow_assignment_id,
            workflows.id AS workflow_id,
            workflows.name AS workflow_name,
            workflow_versions.version_label AS workflow_version_label,
            workflow_instances.node_id,
            nodes.name AS node_name,
            accounts.display_name AS assignee_display_name,
            accounts.email AS assignee_email,
            workflow_instances.status AS assignment_status,
            current_steps.title AS current_step_title,
            COUNT(DISTINCT completed_step_instances.id) AS completed_step_count,
            COUNT(DISTINCT workflow_steps.id) AS total_step_count,
            COUNT(DISTINCT submissions.id) FILTER (WHERE submissions.status = 'draft') AS draft_response_count,
            COUNT(DISTINCT submissions.id) FILTER (WHERE submissions.status = 'submitted') AS submitted_response_count,
            workflow_instances.created_at AS started_at,
            workflow_instances.completed_at
        FROM workflow_instances
        JOIN workflow_assignments ON workflow_assignments.id = workflow_instances.workflow_assignment_id
        JOIN workflow_versions ON workflow_versions.id = workflow_instances.workflow_version_id
        JOIN workflows ON workflows.id = workflow_versions.workflow_id
        JOIN nodes ON nodes.id = workflow_instances.node_id
        JOIN accounts ON accounts.id = workflow_instances.assignee_account_id
        LEFT JOIN workflow_step_instances AS active_step_instances
            ON active_step_instances.workflow_instance_id = workflow_instances.id
           AND active_step_instances.status = 'in_progress'
        LEFT JOIN workflow_steps AS current_steps ON current_steps.id = active_step_instances.workflow_step_id
        LEFT JOIN workflow_step_instances AS completed_step_instances
            ON completed_step_instances.workflow_instance_id = workflow_instances.id
           AND completed_step_instances.status = 'completed'
        LEFT JOIN workflow_steps ON workflow_steps.workflow_version_id = workflow_instances.workflow_version_id
        LEFT JOIN submissions ON submissions.workflow_instance_id = workflow_instances.id
        GROUP BY
            workflow_instances.id,
            workflow_assignments.id,
            workflows.id,
            workflows.name,
            workflow_versions.version_label,
            workflow_instances.node_id,
            nodes.name,
            accounts.display_name,
            accounts.email,
            workflow_instances.status,
            current_steps.title,
            workflow_instances.created_at,
            workflow_instances.completed_at
        ORDER BY workflow_instances.created_at DESC, workflow_instances.id
        LIMIT 100
        "#
    }
}

async fn load_dataset_readiness(
    pool: &sqlx::PgPool,
    boundary: &CapabilityBoundary,
) -> ApiResult<Vec<DatasetStatus>> {
    let rows = match boundary {
        CapabilityBoundary::Global => {
            sqlx::query(dataset_readiness_sql(false))
                .fetch_all(pool)
                .await?
        }
        CapabilityBoundary::Scoped(node_ids) => {
            if node_ids.is_empty() {
                return Ok(Vec::new());
            }
            sqlx::query(dataset_readiness_sql(true))
                .bind(node_ids)
                .fetch_all(pool)
                .await?
        }
        CapabilityBoundary::None => return Ok(Vec::new()),
    };

    rows.into_iter()
        .map(|row| {
            let revision_status: Option<String> = row.try_get("revision_status")?;
            let ready_response_count = row.try_get("ready_response_count")?;
            let readiness = match revision_status.as_deref() {
                Some("published") if ready_response_count > 0 => "Ready",
                Some("published") => "No Ready Responses",
                Some("draft") => "Draft",
                Some("superseded") => "Superseded",
                Some(_) => "Unavailable",
                None => "No Published Revision",
            }
            .to_string();

            Ok(DatasetStatus {
                dataset_id: row.try_get("dataset_id")?,
                dataset_name: row.try_get("dataset_name")?,
                revision_status: revision_status
                    .map(display_status)
                    .unwrap_or_else(|| "Unavailable".to_string()),
                readiness,
                source_count: row.try_get("source_count")?,
                field_count: row.try_get("field_count")?,
                ready_response_count,
            })
        })
        .collect()
}

fn dataset_readiness_sql(scoped: bool) -> &'static str {
    if scoped {
        r#"
        SELECT
            datasets.id AS dataset_id,
            datasets.name AS dataset_name,
            dataset_revisions.status::text AS revision_status,
            COUNT(DISTINCT dataset_sources.id) AS source_count,
            COUNT(DISTINCT dataset_fields.id) AS field_count,
            COUNT(DISTINCT analytics.submission_fact.submission_id) AS ready_response_count
        FROM datasets
        JOIN dataset_scope_nodes ON dataset_scope_nodes.dataset_id = datasets.id
        LEFT JOIN dataset_revisions
            ON dataset_revisions.dataset_id = datasets.id
           AND dataset_revisions.status = 'published'
        LEFT JOIN dataset_sources ON dataset_sources.dataset_id = datasets.id
        LEFT JOIN dataset_fields ON dataset_fields.dataset_id = datasets.id
        LEFT JOIN form_versions
            ON form_versions.form_id = dataset_sources.form_id
           AND (
                dataset_sources.form_version_major IS NULL
                OR form_versions.version_major = dataset_sources.form_version_major
           )
        LEFT JOIN analytics.submission_fact
            ON analytics.submission_fact.form_version_id = form_versions.id
           AND analytics.submission_fact.node_id = ANY($1)
        WHERE dataset_scope_nodes.node_id = ANY($1)
        GROUP BY datasets.id, datasets.name, dataset_revisions.status
        ORDER BY datasets.name, datasets.id
        "#
    } else {
        r#"
        SELECT
            datasets.id AS dataset_id,
            datasets.name AS dataset_name,
            dataset_revisions.status::text AS revision_status,
            COUNT(DISTINCT dataset_sources.id) AS source_count,
            COUNT(DISTINCT dataset_fields.id) AS field_count,
            COUNT(DISTINCT analytics.submission_fact.submission_id) AS ready_response_count
        FROM datasets
        LEFT JOIN dataset_revisions
            ON dataset_revisions.dataset_id = datasets.id
           AND dataset_revisions.status = 'published'
        LEFT JOIN dataset_sources ON dataset_sources.dataset_id = datasets.id
        LEFT JOIN dataset_fields ON dataset_fields.dataset_id = datasets.id
        LEFT JOIN form_versions
            ON form_versions.form_id = dataset_sources.form_id
           AND (
                dataset_sources.form_version_major IS NULL
                OR form_versions.version_major = dataset_sources.form_version_major
           )
        LEFT JOIN analytics.submission_fact
            ON analytics.submission_fact.form_version_id = form_versions.id
        GROUP BY datasets.id, datasets.name, dataset_revisions.status
        ORDER BY datasets.name, datasets.id
        "#
    }
}

async fn load_reporting_data_status(
    pool: &sqlx::PgPool,
    boundary: &CapabilityBoundary,
) -> ApiResult<ReportingDataStatus> {
    let (reporting_node_count, submitted_response_count, response_value_count): (i64, i64, i64) =
        match boundary {
            CapabilityBoundary::Global => {
                let row = sqlx::query(
                    r#"
                SELECT
                    (SELECT COUNT(*) FROM analytics.node_dim) AS node_count,
                    (SELECT COUNT(*) FROM analytics.submission_fact) AS submitted_count,
                    (SELECT COUNT(*) FROM analytics.submission_value_fact) AS value_count
                "#,
                )
                .fetch_one(pool)
                .await?;
                (
                    row.try_get("node_count")?,
                    row.try_get("submitted_count")?,
                    row.try_get("value_count")?,
                )
            }
            CapabilityBoundary::Scoped(node_ids) => {
                if node_ids.is_empty() {
                    (0, 0, 0)
                } else {
                    let row = sqlx::query(
                    r#"
                    SELECT
                        (SELECT COUNT(*) FROM analytics.node_dim WHERE node_id = ANY($1)) AS node_count,
                        (SELECT COUNT(*) FROM analytics.submission_fact WHERE node_id = ANY($1)) AS submitted_count,
                        (
                            SELECT COUNT(*)
                            FROM analytics.submission_value_fact
                            JOIN analytics.submission_fact
                                ON analytics.submission_fact.submission_id = analytics.submission_value_fact.submission_id
                            WHERE analytics.submission_fact.node_id = ANY($1)
                        ) AS value_count
                    "#,
                )
                .bind(node_ids)
                .fetch_one(pool)
                .await?;
                    (
                        row.try_get("node_count")?,
                        row.try_get("submitted_count")?,
                        row.try_get("value_count")?,
                    )
                }
            }
            CapabilityBoundary::None => (0, 0, 0),
        };

    let (status, message) = if submitted_response_count > 0 && response_value_count > 0 {
        (
            "Available".to_string(),
            "Submitted responses and field values are available for reporting.".to_string(),
        )
    } else {
        (
            "Unavailable".to_string(),
            "No submitted response values are available for reporting in this scope.".to_string(),
        )
    };

    Ok(ReportingDataStatus {
        status,
        reporting_node_count,
        submitted_response_count,
        response_value_count,
        message,
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
