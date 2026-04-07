use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;
use sqlx::Row;

use crate::{auth, db::AppState, error::ApiResult};

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
    reports: i64,
    dashboards: i64,
    charts: i64,
}

/// Returns app-readiness counters for the current deployment.
pub async fn get_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ApplicationSummary>> {
    auth::require_capability(&state.pool, &headers, "reports:read").await?;

    let row = sqlx::query(
        r#"
        SELECT
            (SELECT COUNT(*) FROM form_versions WHERE status = 'published') AS published_form_versions,
            (SELECT COUNT(*) FROM submissions WHERE status = 'draft') AS draft_submissions,
            (SELECT COUNT(*) FROM submissions WHERE status = 'submitted') AS submitted_submissions,
            (SELECT COUNT(*) FROM reports) AS reports,
            (SELECT COUNT(*) FROM dashboards) AS dashboards,
            (SELECT COUNT(*) FROM charts) AS charts
        "#,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(ApplicationSummary {
        published_form_versions: row.try_get("published_form_versions")?,
        draft_submissions: row.try_get("draft_submissions")?,
        submitted_submissions: row.try_get("submitted_submissions")?,
        reports: row.try_get("reports")?,
        dashboards: row.try_get("dashboards")?,
        charts: row.try_get("charts")?,
    }))
}
