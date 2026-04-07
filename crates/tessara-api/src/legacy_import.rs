//! Fixture-based legacy migration rehearsal support.
//!
//! This module intentionally imports a small JSON fixture instead of connecting
//! directly to the legacy Django database. The goal for Slice 10 is a repeatable
//! end-to-end migration rehearsal while the final import model is still being
//! defined.

use std::{collections::HashMap, path::Path, str::FromStr};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use tessara_core::FieldType;
use uuid::Uuid;

use crate::{
    analytics,
    error::{ApiError, ApiResult},
};

/// Summary returned after importing a legacy rehearsal fixture.
#[derive(Serialize)]
pub struct LegacyImportSummary {
    /// Fixture identifier used to make imported submission audit events stable.
    pub fixture_name: String,
    /// Imported Partner node.
    pub partner_node_id: Uuid,
    /// Imported Program node.
    pub program_node_id: Uuid,
    /// Imported Activity node.
    pub activity_node_id: Uuid,
    /// Imported Session node.
    pub session_node_id: Uuid,
    /// Imported form family identifier.
    pub form_id: Uuid,
    /// Published imported form version identifier.
    pub form_version_id: Uuid,
    /// Imported submitted response identifier.
    pub submission_id: Uuid,
    /// Report created for rehearsal validation.
    pub report_id: Uuid,
    /// Dashboard created for rehearsal validation.
    pub dashboard_id: Uuid,
    /// Count of projected analytics values after import refresh.
    pub analytics_values: i64,
}

#[derive(Deserialize)]
struct LegacyFixture {
    name: String,
    partner: LegacyNode,
    program: LegacyNode,
    activity: LegacyNode,
    session: LegacySession,
    #[serde(default)]
    choice_lists: Vec<LegacyChoiceList>,
    form: LegacyForm,
    submission: LegacySubmission,
    report: LegacyReport,
    dashboard: LegacyDashboard,
}

#[derive(Deserialize)]
struct LegacyNode {
    legacy_id: String,
    name: String,
    #[serde(default = "default_true")]
    is_active: bool,
    #[serde(default)]
    locked: bool,
}

#[derive(Deserialize)]
struct LegacySession {
    legacy_id: String,
    name: String,
    date: String,
    #[serde(default = "default_true")]
    is_active: bool,
    #[serde(default)]
    locked: bool,
}

#[derive(Deserialize)]
struct LegacyChoiceList {
    legacy_id: String,
    name: String,
    #[serde(default = "default_true")]
    is_active: bool,
    choices: Vec<LegacyChoice>,
}

#[derive(Deserialize)]
struct LegacyChoice {
    legacy_id: String,
    label: String,
    #[serde(default)]
    description: Option<String>,
    ordering: i32,
    #[serde(default = "default_true")]
    is_active: bool,
}

#[derive(Deserialize)]
struct LegacyForm {
    legacy_id: String,
    name: String,
    slug: String,
    scope_node_type: String,
    compatibility_group: String,
    version_label: String,
    sections: Vec<LegacySection>,
}

#[derive(Deserialize)]
struct LegacySection {
    legacy_id: String,
    label: String,
    position: i32,
    fields: Vec<LegacyField>,
}

#[derive(Deserialize)]
struct LegacyField {
    legacy_id: String,
    key: String,
    label: String,
    field_type: String,
    required: bool,
    position: i32,
}

#[derive(Deserialize)]
struct LegacySubmission {
    legacy_id: String,
    node_type: String,
    values: HashMap<String, Value>,
}

#[derive(Deserialize)]
struct LegacyReport {
    name: String,
    logical_key: String,
    source_field_key: String,
    #[serde(default = "default_missing_policy")]
    missing_policy: String,
}

#[derive(Deserialize)]
struct LegacyDashboard {
    name: String,
    chart_name: String,
}

/// Imports a legacy rehearsal fixture from a JSON file path.
pub async fn import_legacy_fixture_file(
    pool: &PgPool,
    path: impl AsRef<Path>,
) -> anyhow::Result<LegacyImportSummary> {
    let path = path.as_ref();
    let fixture = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read legacy fixture {}", path.display()))?;
    import_legacy_fixture_str(pool, &fixture)
        .await
        .map_err(anyhow::Error::from)
}

/// Imports a legacy rehearsal fixture from a JSON string.
pub async fn import_legacy_fixture_str(
    pool: &PgPool,
    fixture_json: &str,
) -> ApiResult<LegacyImportSummary> {
    let fixture: LegacyFixture =
        serde_json::from_str(fixture_json).map_err(|err| ApiError::BadRequest(err.to_string()))?;
    import_legacy_fixture(pool, fixture).await
}

async fn import_legacy_fixture(
    pool: &PgPool,
    fixture: LegacyFixture,
) -> ApiResult<LegacyImportSummary> {
    let partner_type_id = ensure_node_type(pool, "Partner", "partner").await?;
    let program_type_id = ensure_node_type(pool, "Program", "program").await?;
    let activity_type_id = ensure_node_type(pool, "Activity", "activity").await?;
    let session_type_id = ensure_node_type(pool, "Session", "session").await?;

    ensure_node_type_relationship(pool, partner_type_id, program_type_id).await?;
    ensure_node_type_relationship(pool, program_type_id, activity_type_id).await?;
    ensure_node_type_relationship(pool, activity_type_id, session_type_id).await?;

    ensure_standard_node_metadata(pool, partner_type_id, false).await?;
    ensure_standard_node_metadata(pool, program_type_id, false).await?;
    ensure_standard_node_metadata(pool, activity_type_id, false).await?;
    ensure_standard_node_metadata(pool, session_type_id, true).await?;

    let partner_node_id = ensure_node(pool, partner_type_id, None, &fixture.partner, None).await?;
    let program_node_id = ensure_node(
        pool,
        program_type_id,
        Some(partner_node_id),
        &fixture.program,
        None,
    )
    .await?;
    let activity_node_id = ensure_node(
        pool,
        activity_type_id,
        Some(program_node_id),
        &fixture.activity,
        None,
    )
    .await?;
    let session_node_id = ensure_node(
        pool,
        session_type_id,
        Some(activity_node_id),
        &LegacyNode {
            legacy_id: fixture.session.legacy_id.clone(),
            name: fixture.session.name.clone(),
            is_active: fixture.session.is_active,
            locked: fixture.session.locked,
        },
        Some(&fixture.session.date),
    )
    .await?;

    let scope_node_type_id = node_type_id_for_scope(
        &fixture.form.scope_node_type,
        [
            ("partner", partner_type_id),
            ("program", program_type_id),
            ("activity", activity_type_id),
            ("session", session_type_id),
        ],
    )?;
    let form_id = ensure_form(pool, &fixture.form, scope_node_type_id).await?;
    let compatibility_group_id =
        ensure_compatibility_group(pool, form_id, &fixture.form.compatibility_group).await?;
    let form_version_id = ensure_form_version(
        pool,
        form_id,
        compatibility_group_id,
        &fixture.form.version_label,
    )
    .await?;
    ensure_form_sections_and_fields(pool, form_version_id, &fixture.form).await?;
    for choice_list in &fixture.choice_lists {
        ensure_choice_list(pool, form_version_id, choice_list).await?;
    }
    publish_form_version(pool, form_id, compatibility_group_id, form_version_id).await?;

    let submission_node_id = node_id_for_submission_node_type(
        &fixture.submission.node_type,
        [
            ("partner", partner_node_id),
            ("program", program_node_id),
            ("activity", activity_node_id),
            ("session", session_node_id),
        ],
    )?;
    let submission_id = ensure_submission(
        pool,
        &fixture.name,
        form_version_id,
        submission_node_id,
        &fixture.submission,
    )
    .await?;
    let analytics_status = analytics::refresh_projection(pool).await?;

    let report_id = ensure_report(pool, form_id, &fixture.report).await?;
    let chart_id = ensure_chart(pool, &fixture.dashboard.chart_name, report_id).await?;
    let dashboard_id = ensure_dashboard(pool, &fixture.dashboard.name).await?;
    ensure_dashboard_component(pool, dashboard_id, chart_id).await?;

    Ok(LegacyImportSummary {
        fixture_name: fixture.name,
        partner_node_id,
        program_node_id,
        activity_node_id,
        session_node_id,
        form_id,
        form_version_id,
        submission_id,
        report_id,
        dashboard_id,
        analytics_values: analytics_status.value_count,
    })
}

async fn ensure_node_type(pool: &PgPool, name: &str, slug: &str) -> ApiResult<Uuid> {
    Ok(sqlx::query_scalar(
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
    .await?)
}

async fn ensure_node_type_relationship(
    pool: &PgPool,
    parent_node_type_id: Uuid,
    child_node_type_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        INSERT INTO node_type_relationships (parent_node_type_id, child_node_type_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(parent_node_type_id)
    .bind(child_node_type_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn ensure_standard_node_metadata(
    pool: &PgPool,
    node_type_id: Uuid,
    include_session_date: bool,
) -> ApiResult<()> {
    for (key, label, field_type, required) in [
        ("legacy_id", "Legacy ID", "text", true),
        ("is_active", "Legacy Active", "boolean", true),
        ("locked", "Legacy Locked", "boolean", true),
    ] {
        ensure_node_metadata_field(pool, node_type_id, key, label, field_type, required).await?;
    }

    if include_session_date {
        ensure_node_metadata_field(
            pool,
            node_type_id,
            "session_date",
            "Session Date",
            "date",
            true,
        )
        .await?;
    }

    Ok(())
}

async fn ensure_node_metadata_field(
    pool: &PgPool,
    node_type_id: Uuid,
    key: &str,
    label: &str,
    field_type: &str,
    required: bool,
) -> ApiResult<Uuid> {
    Ok(sqlx::query_scalar(
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
    .await?)
}

async fn ensure_node(
    pool: &PgPool,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    node: &LegacyNode,
    session_date: Option<&str>,
) -> ApiResult<Uuid> {
    let node_id = if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT nodes.id
        FROM nodes
        JOIN node_metadata_values ON node_metadata_values.node_id = nodes.id
        JOIN node_metadata_field_definitions
            ON node_metadata_field_definitions.id = node_metadata_values.field_definition_id
        WHERE nodes.node_type_id = $1
          AND nodes.parent_node_id IS NOT DISTINCT FROM $2
          AND node_metadata_field_definitions.key = 'legacy_id'
          AND node_metadata_values.value = to_jsonb($3::text)
        "#,
    )
    .bind(node_type_id)
    .bind(parent_node_id)
    .bind(&node.legacy_id)
    .fetch_optional(pool)
    .await?
    {
        sqlx::query("UPDATE nodes SET name = $1 WHERE id = $2")
            .bind(&node.name)
            .bind(id)
            .execute(pool)
            .await?;
        id
    } else {
        sqlx::query_scalar(
            "INSERT INTO nodes (node_type_id, parent_node_id, name) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(node_type_id)
        .bind(parent_node_id)
        .bind(&node.name)
        .fetch_one(pool)
        .await?
    };

    set_node_metadata(
        pool,
        node_id,
        node_type_id,
        "legacy_id",
        Value::String(node.legacy_id.clone()),
    )
    .await?;
    set_node_metadata(
        pool,
        node_id,
        node_type_id,
        "is_active",
        Value::Bool(node.is_active),
    )
    .await?;
    set_node_metadata(
        pool,
        node_id,
        node_type_id,
        "locked",
        Value::Bool(node.locked),
    )
    .await?;
    if let Some(session_date) = session_date {
        set_node_metadata(
            pool,
            node_id,
            node_type_id,
            "session_date",
            Value::String(session_date.to_string()),
        )
        .await?;
    }

    Ok(node_id)
}

async fn set_node_metadata(
    pool: &PgPool,
    node_id: Uuid,
    node_type_id: Uuid,
    key: &str,
    value: Value,
) -> ApiResult<()> {
    let field_definition_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM node_metadata_field_definitions WHERE node_type_id = $1 AND key = $2",
    )
    .bind(node_type_id)
    .bind(key)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO node_metadata_values (node_id, field_definition_id, value)
        VALUES ($1, $2, $3)
        ON CONFLICT (node_id, field_definition_id)
        DO UPDATE SET value = EXCLUDED.value
        "#,
    )
    .bind(node_id)
    .bind(field_definition_id)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

async fn ensure_choice_list(
    pool: &PgPool,
    form_version_id: Uuid,
    choice_list: &LegacyChoiceList,
) -> ApiResult<Uuid> {
    let choice_list_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO choice_lists (form_version_id, name)
        VALUES ($1, $2)
        ON CONFLICT (form_version_id, name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(form_version_id)
    .bind(&choice_list.name)
    .fetch_one(pool)
    .await?;

    let active_item_value = format!("legacy-active:{}", choice_list.legacy_id);
    sqlx::query(
        r#"
        INSERT INTO choice_list_items (choice_list_id, value, label, position)
        VALUES ($1, $2, $3, -1)
        ON CONFLICT (choice_list_id, value)
        DO UPDATE SET label = EXCLUDED.label, position = EXCLUDED.position
        "#,
    )
    .bind(choice_list_id)
    .bind(active_item_value)
    .bind(format!("legacy is_active={}", choice_list.is_active))
    .execute(pool)
    .await?;

    for choice in &choice_list.choices {
        sqlx::query(
            r#"
            INSERT INTO choice_list_items (choice_list_id, value, label, position)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (choice_list_id, value)
            DO UPDATE SET label = EXCLUDED.label, position = EXCLUDED.position
            "#,
        )
        .bind(choice_list_id)
        .bind(&choice.legacy_id)
        .bind(choice.description.as_deref().unwrap_or(&choice.label))
        .bind(choice.ordering)
        .execute(pool)
        .await?;

        if !choice.is_active {
            let inactive_item_value = format!("legacy-inactive:{}", choice.legacy_id);
            sqlx::query(
                r#"
                INSERT INTO choice_list_items (choice_list_id, value, label, position)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (choice_list_id, value)
                DO UPDATE SET label = EXCLUDED.label, position = EXCLUDED.position
                "#,
            )
            .bind(choice_list_id)
            .bind(inactive_item_value)
            .bind(format!("inactive: {}", choice.label))
            .bind(choice.ordering)
            .execute(pool)
            .await?;
        }
    }

    Ok(choice_list_id)
}

async fn ensure_form(
    pool: &PgPool,
    form: &LegacyForm,
    scope_node_type_id: Uuid,
) -> ApiResult<Uuid> {
    let _form_legacy_id = &form.legacy_id;
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO forms (name, slug, scope_node_type_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (slug)
        DO UPDATE SET name = EXCLUDED.name, scope_node_type_id = EXCLUDED.scope_node_type_id
        RETURNING id
        "#,
    )
    .bind(&form.name)
    .bind(&form.slug)
    .bind(scope_node_type_id)
    .fetch_one(pool)
    .await?)
}

async fn ensure_compatibility_group(pool: &PgPool, form_id: Uuid, name: &str) -> ApiResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO compatibility_groups (form_id, name)
        VALUES ($1, $2)
        ON CONFLICT (form_id, name)
        DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
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

async fn ensure_form_sections_and_fields(
    pool: &PgPool,
    form_version_id: Uuid,
    form: &LegacyForm,
) -> ApiResult<()> {
    for section in &form.sections {
        let section_id: Uuid = if let Some(id) = sqlx::query_scalar(
            "SELECT id FROM form_sections WHERE form_version_id = $1 AND title = $2",
        )
        .bind(form_version_id)
        .bind(&section.label)
        .fetch_optional(pool)
        .await?
        {
            sqlx::query("UPDATE form_sections SET position = $1 WHERE id = $2")
                .bind(section.position)
                .bind(id)
                .execute(pool)
                .await?;
            id
        } else {
            sqlx::query_scalar(
                "INSERT INTO form_sections (form_version_id, title, position) VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(form_version_id)
            .bind(&section.label)
            .bind(section.position)
            .fetch_one(pool)
            .await?
        };

        let _section_legacy_id = &section.legacy_id;
        for field in &section.fields {
            let field_type = FieldType::from_str(&field.field_type)
                .map_err(|error| ApiError::BadRequest(error.to_string()))?;
            let _field_legacy_id = &field.legacy_id;
            sqlx::query(
                r#"
                INSERT INTO form_fields
                    (form_version_id, section_id, key, label, field_type, required, position)
                VALUES ($1, $2, $3, $4, $5::field_type, $6, $7)
                ON CONFLICT (form_version_id, key)
                DO UPDATE SET
                    section_id = EXCLUDED.section_id,
                    label = EXCLUDED.label,
                    field_type = EXCLUDED.field_type,
                    required = EXCLUDED.required,
                    position = EXCLUDED.position
                "#,
            )
            .bind(form_version_id)
            .bind(section_id)
            .bind(&field.key)
            .bind(&field.label)
            .bind(field_type.as_str())
            .bind(field.required)
            .bind(field.position)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

async fn publish_form_version(
    pool: &PgPool,
    form_id: Uuid,
    compatibility_group_id: Uuid,
    form_version_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        UPDATE form_versions
        SET status = 'superseded'::form_version_status
        WHERE form_id = $1
          AND compatibility_group_id = $2
          AND id != $3
          AND status = 'published'::form_version_status
        "#,
    )
    .bind(form_id)
    .bind(compatibility_group_id)
    .bind(form_version_id)
    .execute(pool)
    .await?;

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

async fn ensure_submission(
    pool: &PgPool,
    fixture_name: &str,
    form_version_id: Uuid,
    node_id: Uuid,
    submission: &LegacySubmission,
) -> ApiResult<Uuid> {
    let event_type = format!("legacy_import:{fixture_name}:{}", submission.legacy_id);
    if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT submission_id
        FROM submission_audit_events
        WHERE event_type = $1
        LIMIT 1
        "#,
    )
    .bind(&event_type)
    .fetch_optional(pool)
    .await?
    {
        return Ok(id);
    }

    let account_id: Uuid =
        sqlx::query_scalar("SELECT id FROM accounts WHERE email = 'admin@tessara.local' LIMIT 1")
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

    for (field_key, value) in &submission.values {
        let row = sqlx::query(
            "SELECT id, field_type::text AS field_type FROM form_fields WHERE form_version_id = $1 AND key = $2",
        )
        .bind(form_version_id)
        .bind(field_key)
        .fetch_one(pool)
        .await?;
        let field_id: Uuid = row.try_get("id")?;
        let field_type = FieldType::from_str(&row.try_get::<String, _>("field_type")?)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        field_type
            .validate_json_value(value)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;

        sqlx::query(
            "INSERT INTO submission_values (submission_id, field_id, value) VALUES ($1, $2, $3)",
        )
        .bind(submission_id)
        .bind(field_id)
        .bind(value)
        .execute(pool)
        .await?;
    }

    sqlx::query(
        "INSERT INTO submission_audit_events (submission_id, event_type, account_id) VALUES ($1, $2, $3)",
    )
    .bind(submission_id)
    .bind(event_type)
    .bind(account_id)
    .execute(pool)
    .await?;

    Ok(submission_id)
}

async fn ensure_report(pool: &PgPool, form_id: Uuid, report: &LegacyReport) -> ApiResult<Uuid> {
    let report_id: Uuid = if let Some(id) =
        sqlx::query_scalar("SELECT id FROM reports WHERE form_id = $1 AND name = $2")
            .bind(form_id)
            .bind(&report.name)
            .fetch_optional(pool)
            .await?
    {
        id
    } else {
        sqlx::query_scalar("INSERT INTO reports (name, form_id) VALUES ($1, $2) RETURNING id")
            .bind(&report.name)
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
        VALUES ($1, $2, $3, $4::missing_data_policy, 0)
        "#,
    )
    .bind(report_id)
    .bind(&report.logical_key)
    .bind(&report.source_field_key)
    .bind(&report.missing_policy)
    .execute(pool)
    .await?;

    Ok(report_id)
}

async fn ensure_chart(pool: &PgPool, name: &str, report_id: Uuid) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar("SELECT id FROM charts WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        sqlx::query("UPDATE charts SET report_id = $1, chart_type = 'table' WHERE id = $2")
            .bind(report_id)
            .bind(id)
            .execute(pool)
            .await?;
        return Ok(id);
    }

    Ok(sqlx::query_scalar(
        "INSERT INTO charts (name, report_id, chart_type) VALUES ($1, $2, 'table') RETURNING id",
    )
    .bind(name)
    .bind(report_id)
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
        VALUES ($1, $2, 0, '{"title":"Legacy rehearsal report"}'::jsonb)
        RETURNING id
        "#,
    )
    .bind(dashboard_id)
    .bind(chart_id)
    .fetch_one(pool)
    .await?)
}

fn node_type_id_for_scope<'a>(
    scope: &str,
    node_types: impl IntoIterator<Item = (&'a str, Uuid)>,
) -> ApiResult<Uuid> {
    node_types
        .into_iter()
        .find_map(|(key, id)| key.eq_ignore_ascii_case(scope).then_some(id))
        .ok_or_else(|| ApiError::BadRequest(format!("unknown form scope node type '{scope}'")))
}

fn node_id_for_submission_node_type<'a>(
    scope: &str,
    nodes: impl IntoIterator<Item = (&'a str, Uuid)>,
) -> ApiResult<Uuid> {
    nodes
        .into_iter()
        .find_map(|(key, id)| key.eq_ignore_ascii_case(scope).then_some(id))
        .ok_or_else(|| ApiError::BadRequest(format!("unknown submission node type '{scope}'")))
}

fn default_true() -> bool {
    true
}

fn default_missing_policy() -> String {
    "null".to_string()
}
