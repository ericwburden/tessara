//! Deterministic demo data seeding for local development and smoke tests.

use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    analytics, auth,
    db::AppState,
    error::{ApiError, ApiResult},
};

/// Entity identifiers produced by the deterministic demo seed workflow.
///
/// The values are returned so smoke tests and local UI helpers can immediately
/// navigate to the generated report/dashboard resources.
#[derive(Serialize)]
pub struct DemoSeedSummary {
    /// Runtime organization node used as the demo submission target.
    pub organization_node_id: Uuid,
    /// Demo form family identifier.
    pub form_id: Uuid,
    /// Published demo form version identifier.
    pub form_version_id: Uuid,
    /// Submitted demo response identifier.
    pub submission_id: Uuid,
    /// Report definition that reads the projected submission value.
    pub report_id: Uuid,
    /// Chart definition linked to the report.
    pub chart_id: Uuid,
    /// Dashboard containing the chart component.
    pub dashboard_id: Uuid,
    /// Count of projected analytics values after the seed refresh.
    pub analytics_values: i64,
}

pub(crate) async fn seed_demo_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<DemoSeedSummary>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    Ok(Json(seed_demo(&state.pool).await?))
}

/// Seeds an idempotent end-to-end Tessara demo dataset.
///
/// The seed creates a small hierarchy, publishes a form, records a submitted
/// value, refreshes analytics, and wires a report/chart/dashboard. It is used by
/// both the `seed-demo` CLI mode and the Docker smoke test so the demo path
/// remains repeatable.
pub async fn seed_demo(pool: &PgPool) -> ApiResult<DemoSeedSummary> {
    let org_type_id = ensure_node_type(pool, "Organization", "organization").await?;
    let program_type_id = ensure_node_type(pool, "Program", "program").await?;
    sqlx::query(
        r#"
        INSERT INTO node_type_relationships (parent_node_type_id, child_node_type_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(org_type_id)
    .bind(program_type_id)
    .execute(pool)
    .await?;

    let region_field_id =
        ensure_node_metadata_field(pool, org_type_id, "region", "Region", "text", true).await?;
    let organization_node_id = ensure_node(pool, org_type_id, None, "Demo Organization").await?;
    sqlx::query(
        r#"
        INSERT INTO node_metadata_values (node_id, field_definition_id, value)
        VALUES ($1, $2, $3)
        ON CONFLICT (node_id, field_definition_id)
        DO UPDATE SET value = EXCLUDED.value
        "#,
    )
    .bind(organization_node_id)
    .bind(region_field_id)
    .bind(serde_json::json!("North"))
    .execute(pool)
    .await?;

    let form_id = ensure_form(
        pool,
        "Quarterly Check In",
        "quarterly-check-in",
        Some(org_type_id),
    )
    .await?;
    let compatibility_group_id =
        ensure_compatibility_group(pool, form_id, "Quarterly Check In Compatible").await?;
    let form_version_id = ensure_form_version(pool, form_id, compatibility_group_id, "v1").await?;
    let section_id = ensure_form_section(pool, form_version_id, "Basics", 1).await?;
    ensure_form_field(
        pool,
        form_version_id,
        section_id,
        "participants",
        "Participants",
        "number",
        true,
        1,
    )
    .await?;
    publish_form_version(pool, form_version_id).await?;

    let submission_id = ensure_submitted_participants_submission(
        pool,
        form_version_id,
        organization_node_id,
        "participants",
        42,
    )
    .await?;
    let analytics_status = analytics::refresh_projection(pool).await?;

    let report_id = ensure_report(
        pool,
        form_id,
        "Participants Report",
        "participants",
        "participants",
    )
    .await?;
    let chart_id = ensure_chart(pool, "Participants Table", Some(report_id), "table").await?;
    let dashboard_id = ensure_dashboard(pool, "Demo Dashboard").await?;
    ensure_dashboard_component(pool, dashboard_id, chart_id).await?;

    Ok(DemoSeedSummary {
        organization_node_id,
        form_id,
        form_version_id,
        submission_id,
        report_id,
        chart_id,
        dashboard_id,
        analytics_values: analytics_status.value_count,
    })
}

async fn ensure_node_type(pool: &PgPool, name: &str, slug: &str) -> ApiResult<Uuid> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO node_types (name, slug)
        VALUES ($1, $2)
        ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(slug)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

async fn ensure_node_metadata_field(
    pool: &PgPool,
    node_type_id: Uuid,
    key: &str,
    label: &str,
    field_type: &str,
    required: bool,
) -> ApiResult<Uuid> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO node_metadata_field_definitions
            (node_type_id, key, label, field_type, required)
        VALUES ($1, $2, $3, $4::field_type, $5)
        ON CONFLICT (node_type_id, key)
        DO UPDATE SET label = EXCLUDED.label, required = EXCLUDED.required
        RETURNING id
        "#,
    )
    .bind(node_type_id)
    .bind(key)
    .bind(label)
    .bind(field_type)
    .bind(required)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

async fn ensure_node(
    pool: &PgPool,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    name: &str,
) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT id
        FROM nodes
        WHERE node_type_id = $1
          AND parent_node_id IS NOT DISTINCT FROM $2
          AND name = $3
        "#,
    )
    .bind(node_type_id)
    .bind(parent_node_id)
    .bind(name)
    .fetch_optional(pool)
    .await?
    {
        return Ok(id);
    }

    Ok(sqlx::query_scalar(
        "INSERT INTO nodes (node_type_id, parent_node_id, name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(node_type_id)
    .bind(parent_node_id)
    .bind(name)
    .fetch_one(pool)
    .await?)
}

async fn ensure_form(
    pool: &PgPool,
    name: &str,
    slug: &str,
    scope_node_type_id: Option<Uuid>,
) -> ApiResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO forms (name, slug, scope_node_type_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (slug)
        DO UPDATE SET name = EXCLUDED.name, scope_node_type_id = EXCLUDED.scope_node_type_id
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(slug)
    .bind(scope_node_type_id)
    .fetch_one(pool)
    .await?)
}

async fn ensure_compatibility_group(pool: &PgPool, form_id: Uuid, name: &str) -> ApiResult<Uuid> {
    if let Some(id) =
        sqlx::query_scalar("SELECT id FROM compatibility_groups WHERE form_id = $1 AND name = $2")
            .bind(form_id)
            .bind(name)
            .fetch_optional(pool)
            .await?
    {
        return Ok(id);
    }

    Ok(sqlx::query_scalar(
        "INSERT INTO compatibility_groups (form_id, name) VALUES ($1, $2) RETURNING id",
    )
    .bind(form_id)
    .bind(name)
    .fetch_one(pool)
    .await?)
}

async fn ensure_form_version(
    pool: &PgPool,
    form_id: Uuid,
    compatibility_group_id: Uuid,
    version_label: &str,
) -> ApiResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO form_versions (form_id, compatibility_group_id, version_label)
        VALUES ($1, $2, $3)
        ON CONFLICT (form_id, version_label)
        DO UPDATE SET compatibility_group_id = EXCLUDED.compatibility_group_id
        RETURNING id
        "#,
    )
    .bind(form_id)
    .bind(compatibility_group_id)
    .bind(version_label)
    .fetch_one(pool)
    .await?)
}

async fn ensure_form_section(
    pool: &PgPool,
    form_version_id: Uuid,
    title: &str,
    position: i32,
) -> ApiResult<Uuid> {
    if let Some(id) =
        sqlx::query_scalar("SELECT id FROM form_sections WHERE form_version_id = $1 AND title = $2")
            .bind(form_version_id)
            .bind(title)
            .fetch_optional(pool)
            .await?
    {
        return Ok(id);
    }

    Ok(sqlx::query_scalar(
        "INSERT INTO form_sections (form_version_id, title, position) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(form_version_id)
    .bind(title)
    .bind(position)
    .fetch_one(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
async fn ensure_form_field(
    pool: &PgPool,
    form_version_id: Uuid,
    section_id: Uuid,
    key: &str,
    label: &str,
    field_type: &str,
    required: bool,
    position: i32,
) -> ApiResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO form_fields
            (form_version_id, section_id, key, label, field_type, required, position)
        VALUES ($1, $2, $3, $4, $5::field_type, $6, $7)
        ON CONFLICT (form_version_id, key)
        DO UPDATE SET label = EXCLUDED.label, required = EXCLUDED.required, position = EXCLUDED.position
        RETURNING id
        "#,
    )
    .bind(form_version_id)
    .bind(section_id)
    .bind(key)
    .bind(label)
    .bind(field_type)
    .bind(required)
    .bind(position)
    .fetch_one(pool)
    .await?)
}

async fn publish_form_version(pool: &PgPool, form_version_id: Uuid) -> ApiResult<()> {
    sqlx::query(
        r#"
        UPDATE form_versions
        SET status = 'published'::form_version_status,
            published_at = COALESCE(published_at, now())
        WHERE id = $1
        "#,
    )
    .bind(form_version_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn ensure_submitted_participants_submission(
    pool: &PgPool,
    form_version_id: Uuid,
    node_id: Uuid,
    field_key: &str,
    value: i64,
) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT submissions.id
        FROM submissions
        JOIN submission_values ON submission_values.submission_id = submissions.id
        JOIN form_fields ON form_fields.id = submission_values.field_id
        WHERE submissions.form_version_id = $1
          AND submissions.node_id = $2
          AND submissions.status = 'submitted'::submission_status
          AND form_fields.key = $3
          AND submission_values.value = to_jsonb($4::int)
        LIMIT 1
        "#,
    )
    .bind(form_version_id)
    .bind(node_id)
    .bind(field_key)
    .bind(value as i32)
    .fetch_optional(pool)
    .await?
    {
        return Ok(id);
    }

    let account_id: Uuid =
        sqlx::query_scalar("SELECT id FROM accounts WHERE email = 'admin@tessara.local' LIMIT 1")
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApiError::NotFound("dev admin account".into()))?;
    let field_id: Uuid =
        sqlx::query_scalar("SELECT id FROM form_fields WHERE form_version_id = $1 AND key = $2")
            .bind(form_version_id)
            .bind(field_key)
            .fetch_one(pool)
            .await?;

    let assignment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO form_assignments (form_version_id, node_id, account_id)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(form_version_id)
    .bind(node_id)
    .bind(account_id)
    .fetch_one(pool)
    .await?;
    let submission_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO submissions (assignment_id, form_version_id, node_id, status, submitted_at)
        VALUES ($1, $2, $3, 'submitted'::submission_status, now())
        RETURNING id
        "#,
    )
    .bind(assignment_id)
    .bind(form_version_id)
    .bind(node_id)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        "INSERT INTO submission_values (submission_id, field_id, value) VALUES ($1, $2, $3)",
    )
    .bind(submission_id)
    .bind(field_id)
    .bind(serde_json::json!(value))
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO submission_audit_events (submission_id, event_type, account_id) VALUES ($1, 'seed_demo', $2)",
    )
    .bind(submission_id)
    .bind(account_id)
    .execute(pool)
    .await?;

    Ok(submission_id)
}

async fn ensure_report(
    pool: &PgPool,
    form_id: Uuid,
    name: &str,
    logical_key: &str,
    source_field_key: &str,
) -> ApiResult<Uuid> {
    let report_id = if let Some(id) =
        sqlx::query_scalar("SELECT id FROM reports WHERE form_id = $1 AND name = $2")
            .bind(form_id)
            .bind(name)
            .fetch_optional(pool)
            .await?
    {
        id
    } else {
        sqlx::query_scalar("INSERT INTO reports (name, form_id) VALUES ($1, $2) RETURNING id")
            .bind(name)
            .bind(form_id)
            .fetch_one(pool)
            .await?
    };

    sqlx::query("DELETE FROM report_field_bindings WHERE report_id = $1")
        .bind(report_id)
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        INSERT INTO report_field_bindings
            (report_id, logical_key, source_field_key, missing_policy, position)
        VALUES ($1, $2, $3, 'null'::missing_data_policy, 0)
        "#,
    )
    .bind(report_id)
    .bind(logical_key)
    .bind(source_field_key)
    .execute(pool)
    .await?;

    Ok(report_id)
}

async fn ensure_chart(
    pool: &PgPool,
    name: &str,
    report_id: Option<Uuid>,
    chart_type: &str,
) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar("SELECT id FROM charts WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        sqlx::query("UPDATE charts SET report_id = $1, chart_type = $2 WHERE id = $3")
            .bind(report_id)
            .bind(chart_type)
            .bind(id)
            .execute(pool)
            .await?;
        return Ok(id);
    }

    Ok(sqlx::query_scalar(
        "INSERT INTO charts (name, report_id, chart_type) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(name)
    .bind(report_id)
    .bind(chart_type)
    .fetch_one(pool)
    .await?)
}

async fn ensure_dashboard(pool: &PgPool, name: &str) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar("SELECT id FROM dashboards WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        return Ok(id);
    }

    Ok(
        sqlx::query_scalar("INSERT INTO dashboards (name) VALUES ($1) RETURNING id")
            .bind(name)
            .fetch_one(pool)
            .await?,
    )
}

async fn ensure_dashboard_component(
    pool: &PgPool,
    dashboard_id: Uuid,
    chart_id: Uuid,
) -> ApiResult<Uuid> {
    sqlx::query("DELETE FROM dashboard_components WHERE dashboard_id = $1")
        .bind(dashboard_id)
        .execute(pool)
        .await?;

    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO dashboard_components (dashboard_id, chart_id, position, config)
        VALUES ($1, $2, 0, '{"title":"Participants"}'::jsonb)
        RETURNING id
        "#,
    )
    .bind(dashboard_id)
    .bind(chart_id)
    .fetch_one(pool)
    .await?)
}
