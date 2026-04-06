use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
};

#[derive(Serialize)]
pub struct AnalyticsStatus {
    node_count: i64,
    submitted_count: i64,
    value_count: i64,
}

pub async fn refresh_analytics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<AnalyticsStatus>> {
    auth::require_capability(&state.pool, &headers, "analytics:refresh").await?;

    let statements = [
        "DELETE FROM analytics.submission_value_fact",
        "DELETE FROM analytics.submission_fact",
        "DELETE FROM analytics.field_dim",
        "DELETE FROM analytics.form_version_dim",
        "DELETE FROM analytics.form_dim",
        "DELETE FROM analytics.compatibility_group_dim",
        "DELETE FROM analytics.node_dim",
        r#"
        INSERT INTO analytics.node_dim (node_id, node_name, node_type_id, parent_node_id)
        SELECT id, name, node_type_id, parent_node_id
        FROM nodes
        "#,
        r#"
        INSERT INTO analytics.form_dim (form_id, form_name, form_slug)
        SELECT id, name, slug
        FROM forms
        "#,
        r#"
        INSERT INTO analytics.compatibility_group_dim (compatibility_group_id, form_id, name)
        SELECT id, form_id, name
        FROM compatibility_groups
        "#,
        r#"
        INSERT INTO analytics.form_version_dim
            (form_version_id, form_id, version_label, compatibility_group_id)
        SELECT id, form_id, version_label, compatibility_group_id
        FROM form_versions
        "#,
        r#"
        INSERT INTO analytics.field_dim
            (field_id, form_version_id, field_key, field_label, field_type)
        SELECT id, form_version_id, key, label, field_type::text
        FROM form_fields
        "#,
        r#"
        INSERT INTO analytics.submission_fact
            (submission_id, form_version_id, node_id, status, submitted_at)
        SELECT id, form_version_id, node_id, status::text, submitted_at
        FROM submissions
        WHERE status = 'submitted'::submission_status
        "#,
        r#"
        INSERT INTO analytics.submission_value_fact
            (submission_id, field_id, field_key, value_text, value_json)
        SELECT
            submission_values.submission_id,
            submission_values.field_id,
            form_fields.key,
            CASE jsonb_typeof(submission_values.value)
                WHEN 'string' THEN trim(both '"' from submission_values.value::text)
                ELSE submission_values.value::text
            END AS value_text,
            submission_values.value
        FROM submission_values
        JOIN form_fields ON form_fields.id = submission_values.field_id
        JOIN submissions ON submissions.id = submission_values.submission_id
        WHERE submissions.status = 'submitted'::submission_status
        "#,
    ];

    for statement in statements {
        sqlx::query(statement)
            .execute(&state.pool)
            .await
            .map_err(ApiError::Database)?;
    }

    analytics_status(State(state)).await
}

pub async fn analytics_status(State(state): State<AppState>) -> ApiResult<Json<AnalyticsStatus>> {
    let node_count = sqlx::query_scalar("SELECT COUNT(*) FROM analytics.node_dim")
        .fetch_one(&state.pool)
        .await?;
    let submitted_count = sqlx::query_scalar("SELECT COUNT(*) FROM analytics.submission_fact")
        .fetch_one(&state.pool)
        .await?;
    let value_count = sqlx::query_scalar("SELECT COUNT(*) FROM analytics.submission_value_fact")
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(AnalyticsStatus {
        node_count,
        submitted_count,
        value_count,
    }))
}
