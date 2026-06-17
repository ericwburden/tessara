use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

pub(super) struct FormFieldDef<'a> {
    pub(super) key: &'a str,
    pub(super) label: &'a str,
    pub(super) field_type: &'a str,
    pub(super) required: bool,
    pub(super) position: i32,
}

pub(super) struct DemoFormSpec<'a> {
    pub(super) name: &'a str,
    pub(super) slug: &'a str,
    pub(super) scope_node_type_id: Uuid,
    pub(super) compatibility_group_name: &'a str,
    pub(super) version_label: &'a str,
    pub(super) section_title: &'a str,
    pub(super) fields: Vec<FormFieldDef<'a>>,
}

pub(super) struct EnsuredForm {
    pub(super) form_id: Uuid,
    pub(super) form_version_id: Uuid,
}

pub(super) async fn ensure_demo_form(
    pool: &PgPool,
    spec: DemoFormSpec<'_>,
) -> ApiResult<EnsuredForm> {
    let form_id = ensure_form(pool, spec.name, spec.slug, Some(spec.scope_node_type_id)).await?;
    let compatibility_group_id =
        ensure_compatibility_group(pool, form_id, spec.compatibility_group_name).await?;
    let form_version_id =
        ensure_form_version(pool, form_id, compatibility_group_id, spec.version_label).await?;
    let section_id = ensure_form_section(pool, form_version_id, spec.section_title, 1).await?;

    for field in spec.fields {
        ensure_form_field(
            pool,
            form_version_id,
            section_id,
            field.key,
            field.label,
            field.field_type,
            field.required,
            field.position,
        )
        .await?;
    }

    publish_form_version(pool, form_version_id).await?;

    Ok(EnsuredForm {
        form_id,
        form_version_id,
    })
}

async fn ensure_form(
    pool: &PgPool,
    name: &str,
    slug: &str,
    scope_node_type_id: Option<Uuid>,
) -> ApiResult<Uuid> {
    sqlx::query_scalar(
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
    .await
    .map_err(Into::into)
}

pub(super) async fn replace_form_scope_nodes(
    pool: &PgPool,
    form_id: Uuid,
    node_ids: &[Uuid],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM form_scope_nodes WHERE form_id = $1")
        .bind(form_id)
        .execute(pool)
        .await?;
    for node_id in node_ids {
        sqlx::query(
            "INSERT INTO form_scope_nodes (form_id, node_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(form_id)
        .bind(node_id)
        .execute(pool)
        .await?;
    }
    Ok(())
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

    sqlx::query_scalar(
        "INSERT INTO compatibility_groups (form_id, name) VALUES ($1, $2) RETURNING id",
    )
    .bind(form_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

async fn ensure_form_version(
    pool: &PgPool,
    form_id: Uuid,
    compatibility_group_id: Uuid,
    version_label: &str,
) -> ApiResult<Uuid> {
    let (version_major, version_minor, version_patch) =
        parse_semantic_version_label(version_label)?;
    sqlx::query_scalar(
        r#"
        INSERT INTO form_versions (
            form_id,
            compatibility_group_id,
            version_label,
            version_major,
            version_minor,
            version_patch,
            started_new_major_line
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (form_id, version_label)
        DO UPDATE SET
            compatibility_group_id = EXCLUDED.compatibility_group_id,
            version_major = EXCLUDED.version_major,
            version_minor = EXCLUDED.version_minor,
            version_patch = EXCLUDED.version_patch,
            started_new_major_line = EXCLUDED.started_new_major_line
        RETURNING id
        "#,
    )
    .bind(form_id)
    .bind(compatibility_group_id)
    .bind(version_label)
    .bind(version_major)
    .bind(version_minor)
    .bind(version_patch)
    .bind(version_minor == 0 && version_patch == 0)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
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

    sqlx::query_scalar(
        "INSERT INTO form_sections (form_version_id, title, position) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(form_version_id)
    .bind(title)
    .bind(position)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
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
    sqlx::query_scalar(
        r#"
        INSERT INTO form_fields
            (form_version_id, section_id, key, label, field_type, required, position)
        VALUES ($1, $2, $3, $4, $5::field_type, $6, $7)
        ON CONFLICT (form_version_id, key)
        DO UPDATE SET
            section_id = EXCLUDED.section_id,
            label = EXCLUDED.label,
            required = EXCLUDED.required,
            position = EXCLUDED.position
        RETURNING field_id
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
    .await
    .map_err(Into::into)
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

fn parse_semantic_version_label(version_label: &str) -> ApiResult<(i32, i32, i32)> {
    if let Some((major, minor, patch)) = parse_strict_semantic_version(version_label) {
        return Ok((major, minor, patch));
    }
    if let Some(major) = parse_major_version_suffix(version_label) {
        return Ok((major, 0, 0));
    }
    Err(ApiError::BadRequest(format!(
        "invalid semantic version '{version_label}'"
    )))
}

fn parse_strict_semantic_version(version_label: &str) -> Option<(i32, i32, i32)> {
    let mut parts = version_label.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn parse_major_version_suffix(version_label: &str) -> Option<i32> {
    let digits = version_label
        .trim()
        .rsplit(|character: char| !character.is_ascii_digit())
        .next()?;
    if digits.is_empty() || !digits.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

pub(super) async fn current_form_major(pool: &PgPool, form_id: Uuid) -> ApiResult<Option<i32>> {
    sqlx::query_scalar(
        r#"
        SELECT version_major
        FROM form_versions
        WHERE form_id = $1
          AND status = 'published'::form_version_status
          AND version_major IS NOT NULL
        ORDER BY
            version_major DESC,
            version_minor DESC NULLS LAST,
            version_patch DESC NULLS LAST,
            published_at DESC NULLS LAST,
            created_at DESC
        LIMIT 1
        "#,
    )
    .bind(form_id)
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}
