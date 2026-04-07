use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, parse_field_type, require_text},
};

#[derive(Deserialize)]
pub struct CreateFormRequest {
    name: String,
    slug: String,
    scope_node_type_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateFormVersionRequest {
    version_label: String,
    compatibility_group_name: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateFormSectionRequest {
    title: String,
    position: i32,
}

#[derive(Deserialize)]
pub struct CreateFormFieldRequest {
    section_id: Uuid,
    key: String,
    label: String,
    field_type: String,
    required: bool,
    position: i32,
}

#[derive(Serialize)]
pub struct RenderedForm {
    form_version_id: Uuid,
    form_id: Uuid,
    version_label: String,
    status: String,
    sections: Vec<RenderedSection>,
}

#[derive(Serialize)]
pub struct RenderedSection {
    id: Uuid,
    title: String,
    position: i32,
    fields: Vec<RenderedField>,
}

#[derive(Serialize)]
pub struct RenderedField {
    id: Uuid,
    key: String,
    label: String,
    field_type: String,
    required: bool,
    position: i32,
}

#[derive(Serialize)]
pub struct FormSummary {
    id: Uuid,
    name: String,
    slug: String,
    scope_node_type_id: Option<Uuid>,
    scope_node_type_name: Option<String>,
    versions: Vec<FormVersionSummary>,
}

#[derive(Serialize)]
pub struct FormVersionSummary {
    id: Uuid,
    version_label: String,
    status: String,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
    field_count: i64,
}

pub async fn create_form(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateFormRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "forms:write").await?;
    require_text("form name", &payload.name)?;
    require_text("form slug", &payload.slug)?;
    require_form_slug_available(&state.pool, &payload.slug).await?;
    if let Some(scope_node_type_id) = payload.scope_node_type_id {
        require_node_type_exists(&state.pool, scope_node_type_id).await?;
    }

    let id = sqlx::query_scalar(
        "INSERT INTO forms (name, slug, scope_node_type_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(payload.name)
    .bind(payload.slug)
    .bind(payload.scope_node_type_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

/// Lists forms and their versions for the admin builder shell.
pub async fn list_forms(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<FormSummary>>> {
    auth::require_capability(&state.pool, &headers, "forms:write").await?;

    let form_rows = sqlx::query(
        r#"
        SELECT forms.id, forms.name, forms.slug, forms.scope_node_type_id, node_types.name AS scope_node_type_name
        FROM forms
        LEFT JOIN node_types ON node_types.id = forms.scope_node_type_id
        ORDER BY forms.created_at, forms.name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let version_rows = sqlx::query(
        r#"
        SELECT
            form_versions.id,
            form_versions.form_id,
            form_versions.version_label,
            form_versions.status::text AS status,
            form_versions.published_at,
            COUNT(form_fields.id) AS field_count
        FROM form_versions
        LEFT JOIN form_fields ON form_fields.form_version_id = form_versions.id
        GROUP BY
            form_versions.id,
            form_versions.form_id,
            form_versions.version_label,
            form_versions.status,
            form_versions.published_at,
            form_versions.created_at
        ORDER BY form_versions.created_at, form_versions.version_label
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let mut forms = Vec::new();
    for form in form_rows {
        let form_id: Uuid = form.try_get("id")?;
        let mut versions = Vec::new();
        for version in &version_rows {
            let version_form_id: Uuid = version.try_get("form_id")?;
            if version_form_id == form_id {
                versions.push(FormVersionSummary {
                    id: version.try_get("id")?,
                    version_label: version.try_get("version_label")?,
                    status: version.try_get("status")?,
                    published_at: version.try_get("published_at")?,
                    field_count: version.try_get("field_count")?,
                });
            }
        }

        forms.push(FormSummary {
            id: form_id,
            name: form.try_get("name")?,
            slug: form.try_get("slug")?,
            scope_node_type_id: form.try_get("scope_node_type_id")?,
            scope_node_type_name: form.try_get("scope_node_type_name")?,
            versions,
        });
    }

    Ok(Json(forms))
}

pub async fn create_form_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(form_id): Path<Uuid>,
    Json(payload): Json<CreateFormVersionRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "forms:write").await?;
    require_form_exists(&state.pool, form_id).await?;
    require_text("version label", &payload.version_label)?;
    require_form_version_label_available(&state.pool, form_id, &payload.version_label).await?;

    let group_name = payload
        .compatibility_group_name
        .unwrap_or_else(|| "Default compatibility".into());
    require_text("compatibility group name", &group_name)?;
    let compatibility_group_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO compatibility_groups (form_id, name)
        VALUES ($1, $2)
        ON CONFLICT (form_id, name)
        DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(form_id)
    .bind(group_name)
    .fetch_one(&state.pool)
    .await?;

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO form_versions (form_id, compatibility_group_id, version_label)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(form_id)
    .bind(compatibility_group_id)
    .bind(payload.version_label)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

async fn require_node_type_exists(pool: &sqlx::PgPool, node_type_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM node_types WHERE id = $1)")
        .bind(node_type_id)
        .fetch_one(pool)
        .await?;

    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("node type {node_type_id}")))
    }
}

async fn require_form_exists(pool: &sqlx::PgPool, form_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM forms WHERE id = $1)")
        .bind(form_id)
        .fetch_one(pool)
        .await?;

    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("form {form_id}")))
    }
}

async fn require_form_slug_available(pool: &sqlx::PgPool, slug: &str) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM forms WHERE slug = $1)")
        .bind(slug)
        .fetch_one(pool)
        .await?;

    if exists {
        Err(ApiError::BadRequest(format!(
            "form slug '{slug}' is already in use"
        )))
    } else {
        Ok(())
    }
}

async fn require_form_version_label_available(
    pool: &sqlx::PgPool,
    form_id: Uuid,
    version_label: &str,
) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM form_versions WHERE form_id = $1 AND version_label = $2)",
    )
    .bind(form_id)
    .bind(version_label)
    .fetch_one(pool)
    .await?;

    if exists {
        Err(ApiError::BadRequest(format!(
            "form version label '{version_label}' is already in use for form {form_id}"
        )))
    } else {
        Ok(())
    }
}

pub async fn create_form_section(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(form_version_id): Path<Uuid>,
    Json(payload): Json<CreateFormSectionRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "forms:write").await?;
    assert_form_version_draft(&state.pool, form_version_id).await?;
    require_text("section title", &payload.title)?;

    let id = sqlx::query_scalar(
        "INSERT INTO form_sections (form_version_id, title, position) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(form_version_id)
    .bind(payload.title)
    .bind(payload.position)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

pub async fn create_form_field(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(form_version_id): Path<Uuid>,
    Json(payload): Json<CreateFormFieldRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "forms:write").await?;
    assert_form_version_draft(&state.pool, form_version_id).await?;
    require_text("field key", &payload.key)?;
    require_text("field label", &payload.label)?;
    require_form_field_key_available(&state.pool, form_version_id, &payload.key).await?;
    let field_type = parse_field_type(&payload.field_type)?;
    assert_section_belongs_to_form_version(&state.pool, form_version_id, payload.section_id)
        .await?;

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO form_fields
            (form_version_id, section_id, key, label, field_type, required, position)
        VALUES ($1, $2, $3, $4, $5::field_type, $6, $7)
        RETURNING id
        "#,
    )
    .bind(form_version_id)
    .bind(payload.section_id)
    .bind(payload.key)
    .bind(payload.label)
    .bind(field_type.as_str())
    .bind(payload.required)
    .bind(payload.position)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

pub async fn publish_form_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(form_version_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "forms:write").await?;

    let mut tx = state.pool.begin().await?;
    let version = require_publishable_form_version(&mut tx, form_version_id).await?;

    sqlx::query(
        r#"
        UPDATE form_versions
        SET status = 'superseded'::form_version_status
        WHERE form_id = $1
            AND compatibility_group_id IS NOT DISTINCT FROM $2
            AND id != $3
            AND status = 'published'::form_version_status
        "#,
    )
    .bind(version.form_id)
    .bind(version.compatibility_group_id)
    .bind(form_version_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE form_versions
        SET status = 'published'::form_version_status, published_at = now()
        WHERE id = $1 AND status = 'draft'::form_version_status
        "#,
    )
    .bind(form_version_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(IdResponse {
        id: form_version_id,
    }))
}

pub async fn render_form_version(
    State(state): State<AppState>,
    Path(form_version_id): Path<Uuid>,
) -> ApiResult<Json<RenderedForm>> {
    let version = sqlx::query(
        r#"
        SELECT id, form_id, version_label, status::text AS status
        FROM form_versions
        WHERE id = $1
        "#,
    )
    .bind(form_version_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("form version {form_version_id}")))?;

    let section_rows = sqlx::query(
        r#"
        SELECT id, title, position
        FROM form_sections
        WHERE form_version_id = $1
        ORDER BY position, title
        "#,
    )
    .bind(form_version_id)
    .fetch_all(&state.pool)
    .await?;

    let field_rows = sqlx::query(
        r#"
        SELECT id, section_id, key, label, field_type::text AS field_type, required, position
        FROM form_fields
        WHERE form_version_id = $1
        ORDER BY position, label
        "#,
    )
    .bind(form_version_id)
    .fetch_all(&state.pool)
    .await?;

    let mut sections = Vec::new();
    for section in section_rows {
        let section_id: Uuid = section.try_get("id")?;
        let mut fields = Vec::new();
        for field in &field_rows {
            let field_section_id: Uuid = field.try_get("section_id")?;
            if field_section_id == section_id {
                fields.push(RenderedField {
                    id: field.try_get("id")?,
                    key: field.try_get("key")?,
                    label: field.try_get("label")?,
                    field_type: field.try_get("field_type")?,
                    required: field.try_get("required")?,
                    position: field.try_get("position")?,
                });
            }
        }

        sections.push(RenderedSection {
            id: section_id,
            title: section.try_get("title")?,
            position: section.try_get("position")?,
            fields,
        });
    }

    Ok(Json(RenderedForm {
        form_version_id,
        form_id: version.try_get("form_id")?,
        version_label: version.try_get("version_label")?,
        status: version.try_get("status")?,
        sections,
    }))
}

async fn assert_form_version_draft(pool: &sqlx::PgPool, form_version_id: Uuid) -> ApiResult<()> {
    let status: Option<String> =
        sqlx::query_scalar("SELECT status::text FROM form_versions WHERE id = $1")
            .bind(form_version_id)
            .fetch_optional(pool)
            .await?;

    match status.as_deref() {
        Some("draft") => Ok(()),
        Some(_) => Err(ApiError::BadRequest(
            "published form versions cannot be modified".into(),
        )),
        None => Err(ApiError::NotFound(format!(
            "form version {form_version_id}"
        ))),
    }
}

async fn assert_section_belongs_to_form_version(
    pool: &sqlx::PgPool,
    form_version_id: Uuid,
    section_id: Uuid,
) -> ApiResult<()> {
    let section_form_version_id: Option<Uuid> =
        sqlx::query_scalar("SELECT form_version_id FROM form_sections WHERE id = $1")
            .bind(section_id)
            .fetch_optional(pool)
            .await?;

    match section_form_version_id {
        Some(actual_form_version_id) if actual_form_version_id == form_version_id => Ok(()),
        Some(_) => Err(ApiError::BadRequest(
            "field section must belong to the same form version".into(),
        )),
        None => Err(ApiError::NotFound(format!("form section {section_id}"))),
    }
}

async fn require_form_field_key_available(
    pool: &sqlx::PgPool,
    form_version_id: Uuid,
    key: &str,
) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM form_fields WHERE form_version_id = $1 AND key = $2)",
    )
    .bind(form_version_id)
    .bind(key)
    .fetch_one(pool)
    .await?;

    if exists {
        Err(ApiError::BadRequest(format!(
            "field key '{key}' is already in use for form version {form_version_id}"
        )))
    } else {
        Ok(())
    }
}

struct PublishableFormVersion {
    form_id: Uuid,
    compatibility_group_id: Option<Uuid>,
}

async fn require_publishable_form_version(
    tx: &mut Transaction<'_, Postgres>,
    form_version_id: Uuid,
) -> ApiResult<PublishableFormVersion> {
    let version = sqlx::query(
        r#"
        SELECT form_id, compatibility_group_id, status::text AS status
        FROM form_versions
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(form_version_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("form version {form_version_id}")))?;

    let status: String = version.try_get("status")?;
    if status != "draft" {
        return Err(ApiError::BadRequest(
            "only draft form versions can be published".into(),
        ));
    }

    let section_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM form_sections WHERE form_version_id = $1")
            .bind(form_version_id)
            .fetch_one(&mut **tx)
            .await?;
    if section_count == 0 {
        return Err(ApiError::BadRequest(
            "cannot publish a form version without sections".into(),
        ));
    }

    let field_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM form_fields WHERE form_version_id = $1")
            .bind(form_version_id)
            .fetch_one(&mut **tx)
            .await?;
    if field_count == 0 {
        return Err(ApiError::BadRequest(
            "cannot publish a form version without fields".into(),
        ));
    }

    Ok(PublishableFormVersion {
        form_id: version.try_get("form_id")?,
        compatibility_group_id: version.try_get("compatibility_group_id")?,
    })
}
