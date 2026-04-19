use axum::{Json, extract::State};
use serde::Serialize;
use sqlx::Row;

use crate::{
    auth::{self, AuthenticatedRequest},
    db::AppState,
    error::ApiResult,
};

/// High-level counters used by focused application screens.
///
/// This endpoint is intentionally read-only and compact. It gives the
/// replacement-oriented frontend one stable place to discover whether the local
/// deployment has enough configured data for submission, reporting, dashboard,
/// and migration-review workflows without coupling those screens to multiple
/// list endpoints.
#[derive(Serialize)]
pub struct ApplicationSummary {
    published_form_versions: i64,
    draft_submissions: i64,
    submitted_submissions: i64,
    datasets: i64,
    reports: i64,
    aggregations: i64,
    dashboards: i64,
    charts: i64,
}

/// Returns app-readiness counters for the current deployment.
pub async fn get_summary(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<ApplicationSummary>> {
    let account = request.account;

    if account.is_admin() {
        let row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM form_versions WHERE status = 'published') AS published_form_versions,
                (SELECT COUNT(*) FROM submissions WHERE status = 'draft') AS draft_submissions,
                (SELECT COUNT(*) FROM submissions WHERE status = 'submitted') AS submitted_submissions,
                (SELECT COUNT(*) FROM datasets) AS datasets,
                (SELECT COUNT(*) FROM reports) AS reports,
                (SELECT COUNT(*) FROM aggregations) AS aggregations,
                (SELECT COUNT(*) FROM dashboards) AS dashboards,
                (SELECT COUNT(*) FROM charts) AS charts
            "#,
        )
        .fetch_one(&state.pool)
        .await?;

        return Ok(Json(ApplicationSummary {
            published_form_versions: row.try_get("published_form_versions")?,
            draft_submissions: row.try_get("draft_submissions")?,
            submitted_submissions: row.try_get("submitted_submissions")?,
            datasets: row.try_get("datasets")?,
            reports: row.try_get("reports")?,
            aggregations: row.try_get("aggregations")?,
            dashboards: row.try_get("dashboards")?,
            charts: row.try_get("charts")?,
        }));
    }

    if account.is_operator() {
        let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
        let row = sqlx::query(
            r#"
            SELECT
                (
                    SELECT COUNT(DISTINCT form_versions.id)
                    FROM form_versions
                    JOIN form_assignments ON form_assignments.form_version_id = form_versions.id
                    WHERE form_versions.status = 'published'::form_version_status
                      AND form_assignments.node_id = ANY($1)
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
                (
                    SELECT COUNT(DISTINCT reports.id)
                    FROM reports
                    JOIN forms ON forms.id = reports.form_id
                    JOIN form_versions ON form_versions.form_id = forms.id
                    JOIN form_assignments ON form_assignments.form_version_id = form_versions.id
                    WHERE form_assignments.node_id = ANY($1)
                ) AS reports,
                0::bigint AS aggregations,
                (
                    SELECT COUNT(DISTINCT dashboards.id)
                    FROM dashboards
                    JOIN dashboard_components ON dashboard_components.dashboard_id = dashboards.id
                    JOIN charts ON charts.id = dashboard_components.chart_id
                    LEFT JOIN reports ON reports.id = charts.report_id
                    LEFT JOIN aggregations ON aggregations.id = charts.aggregation_id
                    LEFT JOIN reports AS aggregation_reports ON aggregation_reports.id = aggregations.report_id
                    LEFT JOIN forms AS direct_forms ON direct_forms.id = reports.form_id
                    LEFT JOIN forms AS aggregation_forms ON aggregation_forms.id = aggregation_reports.form_id
                    LEFT JOIN form_versions AS direct_form_versions ON direct_form_versions.form_id = direct_forms.id
                    LEFT JOIN form_versions AS aggregation_form_versions ON aggregation_form_versions.form_id = aggregation_forms.id
                    LEFT JOIN form_assignments AS direct_assignments ON direct_assignments.form_version_id = direct_form_versions.id
                    LEFT JOIN form_assignments AS aggregation_assignments ON aggregation_assignments.form_version_id = aggregation_form_versions.id
                    WHERE direct_assignments.node_id = ANY($1)
                       OR aggregation_assignments.node_id = ANY($1)
                ) AS dashboards,
                0::bigint AS charts
            "#,
        )
        .bind(scope_ids)
        .fetch_one(&state.pool)
        .await?;

        return Ok(Json(ApplicationSummary {
            published_form_versions: row.try_get("published_form_versions")?,
            draft_submissions: row.try_get("draft_submissions")?,
            submitted_submissions: row.try_get("submitted_submissions")?,
            datasets: row.try_get("datasets")?,
            reports: row.try_get("reports")?,
            aggregations: row.try_get("aggregations")?,
            dashboards: row.try_get("dashboards")?,
            charts: row.try_get("charts")?,
        }));
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
                JOIN form_assignments ON form_assignments.id = submissions.assignment_id
                WHERE submissions.status = 'draft'::submission_status
                  AND form_assignments.account_id = ANY($1)
            ) AS draft_submissions,
            (
                SELECT COUNT(*)
                FROM submissions
                JOIN form_assignments ON form_assignments.id = submissions.assignment_id
                WHERE submissions.status = 'submitted'::submission_status
                  AND form_assignments.account_id = ANY($1)
            ) AS submitted_submissions,
            0::bigint AS datasets,
            0::bigint AS reports,
            0::bigint AS aggregations,
            0::bigint AS dashboards,
            0::bigint AS charts
        "#,
    )
    .bind(accessible_account_ids)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(ApplicationSummary {
        published_form_versions: row.try_get("published_form_versions")?,
        draft_submissions: row.try_get("draft_submissions")?,
        submitted_submissions: row.try_get("submitted_submissions")?,
        datasets: row.try_get("datasets")?,
        reports: row.try_get("reports")?,
        aggregations: row.try_get("aggregations")?,
        dashboards: row.try_get("dashboards")?,
        charts: row.try_get("charts")?,
    }))
}
