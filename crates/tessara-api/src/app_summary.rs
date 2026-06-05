use axum::{Json, extract::State};
use serde::Serialize;
use sqlx::Row;

use crate::{
    auth::{self, AuthenticatedRequest},
    db::AppState,
    error::ApiResult,
};

/// High-level counters used by focused application screens.
#[derive(Serialize)]
pub struct ApplicationSummary {
    published_form_versions: i64,
    draft_submissions: i64,
    submitted_submissions: i64,
    datasets: i64,
    dataset_revisions: i64,
    components: i64,
    component_versions: i64,
    dashboards: i64,
}

/// Returns app-readiness counters for the current deployment.
pub async fn get_summary(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<ApplicationSummary>> {
    let account = request.account;

    if matches!(
        auth::capability_boundary(&state.pool, &account, "admin:all").await?,
        auth::CapabilityBoundary::Global
    ) {
        let row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM form_versions WHERE status = 'published') AS published_form_versions,
                (SELECT COUNT(*) FROM submissions WHERE status = 'draft') AS draft_submissions,
                (SELECT COUNT(*) FROM submissions WHERE status = 'submitted') AS submitted_submissions,
                (SELECT COUNT(*) FROM datasets) AS datasets,
                (SELECT COUNT(*) FROM dataset_revisions) AS dataset_revisions,
                (SELECT COUNT(*) FROM components) AS components,
                (SELECT COUNT(*) FROM component_versions) AS component_versions,
                (SELECT COUNT(*) FROM dashboards) AS dashboards
            "#,
        )
        .fetch_one(&state.pool)
        .await?;

        return summary_from_row(row);
    }

    if let auth::CapabilityBoundary::Scoped(scope_ids) =
        auth::capability_boundary(&state.pool, &account, "forms:read").await?
    {
        let row = sqlx::query(
            r#"
            SELECT
                (
                    SELECT COUNT(DISTINCT form_versions.id)
                    FROM form_versions
                    JOIN forms ON forms.id = form_versions.form_id
                    JOIN form_scope_nodes ON form_scope_nodes.form_id = forms.id
                    WHERE form_versions.status = 'published'::form_version_status
                      AND form_scope_nodes.node_id = ANY($1)
                ) AS published_form_versions,
                (
                    SELECT COUNT(*)
                    FROM submissions
                    WHERE submissions.status = 'draft'::submission_status
                      AND submissions.node_id = ANY($1)
                ) AS draft_submissions,
                (
                    SELECT COUNT(*)
                    FROM submissions
                    WHERE submissions.status = 'submitted'::submission_status
                      AND submissions.node_id = ANY($1)
                ) AS submitted_submissions,
                0::bigint AS datasets,
                0::bigint AS dataset_revisions,
                0::bigint AS components,
                0::bigint AS component_versions,
                0::bigint AS dashboards
            "#,
        )
        .bind(scope_ids)
        .fetch_one(&state.pool)
        .await?;

        return summary_from_row(row);
    }

    let accessible_account_ids = {
        let mut ids = vec![account.account_id];
        ids.extend(
            account
                .delegations
                .iter()
                .map(|delegate| delegate.account_id),
        );
        ids
    };
    let row = sqlx::query(
        r#"
        SELECT
            0::bigint AS published_form_versions,
            (
                SELECT COUNT(*)
                FROM submissions
                JOIN workflow_assignments ON workflow_assignments.id = submissions.workflow_assignment_id
                WHERE submissions.status = 'draft'::submission_status
                  AND workflow_assignments.account_id = ANY($1)
            ) AS draft_submissions,
            (
                SELECT COUNT(*)
                FROM submissions
                JOIN workflow_assignments ON workflow_assignments.id = submissions.workflow_assignment_id
                WHERE submissions.status = 'submitted'::submission_status
                  AND workflow_assignments.account_id = ANY($1)
            ) AS submitted_submissions,
            0::bigint AS datasets,
            0::bigint AS dataset_revisions,
            0::bigint AS components,
            0::bigint AS component_versions,
            0::bigint AS dashboards
        "#,
    )
    .bind(accessible_account_ids)
    .fetch_one(&state.pool)
    .await?;

    summary_from_row(row)
}

fn summary_from_row(row: sqlx::postgres::PgRow) -> ApiResult<Json<ApplicationSummary>> {
    Ok(Json(ApplicationSummary {
        published_form_versions: row.try_get("published_form_versions")?,
        draft_submissions: row.try_get("draft_submissions")?,
        submitted_submissions: row.try_get("submitted_submissions")?,
        datasets: row.try_get("datasets")?,
        dataset_revisions: row.try_get("dataset_revisions")?,
        components: row.try_get("components")?,
        component_versions: row.try_get("component_versions")?,
        dashboards: row.try_get("dashboards")?,
    }))
}
