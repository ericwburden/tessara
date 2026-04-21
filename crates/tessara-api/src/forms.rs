use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use std::collections::BTreeMap;
use tessara_forms::{
    ensure_form_version_editable, ensure_form_version_publishable,
    ensure_section_belongs_to_form_version,
};
use uuid::Uuid;

use crate::{
    auth::{self, AuthenticatedRequest},
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, parse_field_type, require_text},
    workflows,
};

#[derive(Deserialize)]
pub struct CreateFormRequest {
    name: String,
    slug: String,
    scope_node_type_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateFormRequest {
    name: String,
    slug: String,
    scope_node_type_id: Option<Uuid>,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct CreateFormVersionRequest {}

#[derive(Deserialize)]
pub struct CreateFormSectionRequest {
    title: String,
    position: i32,
    #[serde(default = "default_form_section_description")]
    description: String,
    #[serde(default = "default_form_section_column_count")]
    column_count: i32,
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

#[derive(Deserialize)]
pub struct UpdateFormSectionRequest {
    title: String,
    position: i32,
    #[serde(default = "default_form_section_description")]
    description: String,
    #[serde(default = "default_form_section_column_count")]
    column_count: i32,
}

#[derive(Deserialize)]
pub struct UpdateFormFieldRequest {
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
    form_name: String,
    version_label: Option<String>,
    status: String,
    sections: Vec<RenderedSection>,
}

#[derive(Serialize)]
pub struct RenderedSection {
    id: Uuid,
    title: String,
    description: String,
    column_count: i32,
    position: i32,
    fields: Vec<RenderedField>,
}

fn default_form_section_description() -> String {
    String::new()
}

fn default_form_section_column_count() -> i32 {
    1
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
pub struct FormDefinition {
    id: Uuid,
    name: String,
    slug: String,
    scope_node_type_id: Option<Uuid>,
    scope_node_type_name: Option<String>,
    versions: Vec<FormVersionSummary>,
    workflows: Vec<FormWorkflowLink>,
    reports: Vec<FormReportLink>,
    dataset_sources: Vec<FormDatasetSourceLink>,
}

#[derive(Serialize)]
pub struct FormVersionSummary {
    id: Uuid,
    version_label: Option<String>,
    status: String,
    version_major: Option<i32>,
    version_minor: Option<i32>,
    version_patch: Option<i32>,
    compatibility_group_id: Option<Uuid>,
    compatibility_group_name: Option<String>,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
    field_count: i64,
    semantic_bump: Option<String>,
    started_new_major_line: Option<bool>,
    publish_preview: Option<FormPublishPreview>,
}

#[derive(Serialize)]
pub struct FormPublishPreview {
    version_label: String,
    version_major: i32,
    version_minor: i32,
    version_patch: i32,
    semantic_bump: String,
    compatibility_label: String,
    starts_new_major_line: bool,
    dependency_warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct PublishFormVersionResponse {
    id: Uuid,
    version_label: String,
    version_major: i32,
    version_minor: i32,
    version_patch: i32,
    semantic_bump: String,
    compatibility_label: String,
    status: String,
    published_at: chrono::DateTime<chrono::Utc>,
    dependency_warnings: Vec<String>,
    starts_new_major_line: bool,
}

#[derive(Serialize)]
pub struct FormReportLink {
    id: Uuid,
    name: String,
}

#[derive(Serialize)]
pub struct FormDatasetSourceLink {
    dataset_id: Uuid,
    dataset_name: String,
    source_alias: String,
    selection_rule: String,
}

#[derive(Serialize)]
pub struct FormWorkflowLink {
    id: Uuid,
    name: String,
    slug: String,
    current_version_id: Option<Uuid>,
    current_version_label: Option<String>,
    current_status: Option<String>,
    assignment_count: i64,
}

#[derive(Serialize)]
pub struct PublishedFormVersionSummary {
    pub form_id: Uuid,
    pub form_name: String,
    pub form_slug: String,
    pub form_version_id: Uuid,
    pub version_label: String,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub field_count: i64,
}

pub async fn create_form(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Json(payload): Json<CreateFormRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("forms:write")?;
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

/// Updates form-level metadata used by the admin builder and report context.
pub async fn update_form(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(form_id): Path<Uuid>,
    Json(payload): Json<UpdateFormRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("forms:write")?;
    require_text("form name", &payload.name)?;
    require_text("form slug", &payload.slug)?;
    require_form_exists(&state.pool, form_id).await?;
    require_form_slug_available_for_form(&state.pool, form_id, &payload.slug).await?;
    if let Some(scope_node_type_id) = payload.scope_node_type_id {
        require_node_type_exists(&state.pool, scope_node_type_id).await?;
    }

    let id = sqlx::query_scalar(
        r#"
        UPDATE forms
        SET name = $2, slug = $3, scope_node_type_id = $4
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(form_id)
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
    request: AuthenticatedRequest,
) -> ApiResult<Json<Vec<FormSummary>>> {
    request.require_capability("forms:write")?;

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
            form_versions.version_major,
            form_versions.version_minor,
            form_versions.version_patch,
            form_versions.compatibility_group_id,
            compatibility_groups.name AS compatibility_group_name,
            form_versions.published_at,
            form_versions.semantic_bump,
            form_versions.started_new_major_line,
            COUNT(form_fields.id) AS field_count
        FROM form_versions
        LEFT JOIN compatibility_groups
            ON compatibility_groups.id = form_versions.compatibility_group_id
        LEFT JOIN form_fields ON form_fields.form_version_id = form_versions.id
        GROUP BY
            form_versions.id,
            form_versions.form_id,
            form_versions.version_label,
            form_versions.status,
            form_versions.version_major,
            form_versions.version_minor,
            form_versions.version_patch,
            form_versions.compatibility_group_id,
            compatibility_groups.name,
            form_versions.published_at,
            form_versions.semantic_bump,
            form_versions.started_new_major_line,
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
                    version_major: version.try_get("version_major")?,
                    version_minor: version.try_get("version_minor")?,
                    version_patch: version.try_get("version_patch")?,
                    compatibility_group_id: version.try_get("compatibility_group_id")?,
                    compatibility_group_name: version.try_get("compatibility_group_name")?,
                    published_at: version.try_get("published_at")?,
                    field_count: version.try_get("field_count")?,
                    semantic_bump: version.try_get("semantic_bump")?,
                    started_new_major_line: version.try_get("started_new_major_line")?,
                    publish_preview: None,
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

/// Returns a form definition with versions plus downstream reporting links.
pub async fn get_form(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(form_id): Path<Uuid>,
) -> ApiResult<Json<FormDefinition>> {
    request.require_capability("forms:write")?;
    Ok(Json(get_form_definition(&state.pool, form_id).await?))
}

/// Lists published form versions available for submission.
pub async fn list_published_form_versions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<PublishedFormVersionSummary>>> {
    let account = auth::require_capability(&state.pool, &headers, "forms:read").await?;
    let rows = sqlx::query(
        r#"
        SELECT
            forms.id AS form_id,
            forms.name AS form_name,
            forms.slug AS form_slug,
            form_versions.id AS form_version_id,
            form_versions.version_label,
            form_versions.published_at,
            COUNT(form_fields.id) AS field_count
        FROM form_versions
        JOIN forms ON forms.id = form_versions.form_id
        LEFT JOIN form_fields ON form_fields.form_version_id = form_versions.id
        WHERE form_versions.status = 'published'::form_version_status
        GROUP BY
            forms.id,
            forms.name,
            forms.slug,
            form_versions.id,
            form_versions.version_label,
            form_versions.published_at,
            form_versions.created_at
        ORDER BY forms.name, form_versions.created_at
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let forms = rows
        .into_iter()
        .map(|row| {
            Ok(PublishedFormVersionSummary {
                form_id: row.try_get("form_id")?,
                form_name: row.try_get("form_name")?,
                form_slug: row.try_get("form_slug")?,
                form_version_id: row.try_get("form_version_id")?,
                version_label: row.try_get("version_label")?,
                published_at: row.try_get("published_at")?,
                field_count: row.try_get("field_count")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    if account.is_operator() {
        let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
        let allowed_form_version_ids = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT DISTINCT form_versions.id
            FROM form_versions
            JOIN form_assignments ON form_assignments.form_version_id = form_versions.id
            WHERE form_versions.status = 'published'::form_version_status
              AND form_assignments.node_id = ANY($1)
            "#,
        )
        .bind(scope_ids)
        .fetch_all(&state.pool)
        .await?;

        Ok(Json(
            forms
                .into_iter()
                .filter(|form| allowed_form_version_ids.contains(&form.form_version_id))
                .collect(),
        ))
    } else {
        Ok(Json(forms))
    }
}

pub async fn list_readable_forms(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<Vec<FormSummary>>> {
    let account = request.require_capability("forms:read")?;
    let form_rows = if account.is_operator() {
        let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
        sqlx::query(
            r#"
            SELECT DISTINCT
                forms.id,
                forms.name,
                forms.slug,
                forms.scope_node_type_id,
                node_types.name AS scope_node_type_name
            FROM forms
            LEFT JOIN node_types ON node_types.id = forms.scope_node_type_id
            JOIN form_versions ON form_versions.form_id = forms.id
            JOIN form_assignments ON form_assignments.form_version_id = form_versions.id
            WHERE form_assignments.node_id = ANY($1)
            ORDER BY forms.name, forms.id
            "#,
        )
        .bind(scope_ids)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT forms.id, forms.name, forms.slug, forms.scope_node_type_id, node_types.name AS scope_node_type_name
            FROM forms
            LEFT JOIN node_types ON node_types.id = forms.scope_node_type_id
            ORDER BY forms.created_at, forms.name
            "#,
        )
        .fetch_all(&state.pool)
        .await?
    };

    let form_ids = form_rows
        .iter()
        .map(|form| form.try_get::<Uuid, _>("id"))
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let version_rows = if form_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query(
            r#"
            SELECT
                form_versions.id,
                form_versions.form_id,
                form_versions.version_label,
                form_versions.status::text AS status,
                form_versions.version_major,
                form_versions.version_minor,
                form_versions.version_patch,
                form_versions.compatibility_group_id,
                compatibility_groups.name AS compatibility_group_name,
                form_versions.published_at,
                form_versions.semantic_bump,
                form_versions.started_new_major_line,
                COUNT(form_fields.id) AS field_count
            FROM form_versions
            LEFT JOIN compatibility_groups
                ON compatibility_groups.id = form_versions.compatibility_group_id
            LEFT JOIN form_fields ON form_fields.form_version_id = form_versions.id
            WHERE form_versions.form_id = ANY($1)
            GROUP BY
                form_versions.id,
                form_versions.form_id,
                form_versions.version_label,
                form_versions.status,
                form_versions.version_major,
                form_versions.version_minor,
                form_versions.version_patch,
                form_versions.compatibility_group_id,
                compatibility_groups.name,
                form_versions.published_at,
                form_versions.semantic_bump,
                form_versions.started_new_major_line,
                form_versions.created_at
            ORDER BY form_versions.created_at, form_versions.version_label
            "#,
        )
        .bind(form_ids)
        .fetch_all(&state.pool)
        .await?
    };

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
                    version_major: version.try_get("version_major")?,
                    version_minor: version.try_get("version_minor")?,
                    version_patch: version.try_get("version_patch")?,
                    compatibility_group_id: version.try_get("compatibility_group_id")?,
                    compatibility_group_name: version.try_get("compatibility_group_name")?,
                    published_at: version.try_get("published_at")?,
                    field_count: version.try_get("field_count")?,
                    semantic_bump: version.try_get("semantic_bump")?,
                    started_new_major_line: version.try_get("started_new_major_line")?,
                    publish_preview: None,
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

pub async fn get_readable_form(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(form_id): Path<Uuid>,
) -> ApiResult<Json<FormDefinition>> {
    let account = request.require_capability("forms:read")?;
    if account.is_operator() {
        let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
        let visible: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM form_versions
                JOIN form_assignments ON form_assignments.form_version_id = form_versions.id
                WHERE form_versions.form_id = $1
                  AND form_assignments.node_id = ANY($2)
            )
            "#,
        )
        .bind(form_id)
        .bind(scope_ids)
        .fetch_one(&state.pool)
        .await?;
        if !visible {
            return Err(ApiError::Forbidden("forms:read".into()));
        }
    }

    Ok(Json(get_form_definition(&state.pool, form_id).await?))
}

async fn get_form_definition(pool: &sqlx::PgPool, form_id: Uuid) -> ApiResult<FormDefinition> {
    let form = sqlx::query(
        r#"
        SELECT
            forms.id,
            forms.name,
            forms.slug,
            forms.scope_node_type_id,
            node_types.name AS scope_node_type_name
        FROM forms
        LEFT JOIN node_types ON node_types.id = forms.scope_node_type_id
        WHERE forms.id = $1
        "#,
    )
    .bind(form_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("form {form_id}")))?;

    let mut versions = sqlx::query(
        r#"
        SELECT
            form_versions.id,
            form_versions.version_label,
            form_versions.status::text AS status,
            form_versions.version_major,
            form_versions.version_minor,
            form_versions.version_patch,
            form_versions.compatibility_group_id,
            compatibility_groups.name AS compatibility_group_name,
            form_versions.published_at,
            form_versions.semantic_bump,
            form_versions.started_new_major_line,
            COUNT(form_fields.id) AS field_count
        FROM form_versions
        LEFT JOIN compatibility_groups
            ON compatibility_groups.id = form_versions.compatibility_group_id
        LEFT JOIN form_fields ON form_fields.form_version_id = form_versions.id
        WHERE form_versions.form_id = $1
        GROUP BY
            form_versions.id,
            form_versions.version_label,
            form_versions.status,
            form_versions.version_major,
            form_versions.version_minor,
            form_versions.version_patch,
            form_versions.compatibility_group_id,
            compatibility_groups.name,
            form_versions.published_at,
            form_versions.semantic_bump,
            form_versions.started_new_major_line,
            form_versions.created_at
        ORDER BY form_versions.created_at, form_versions.version_label
        "#,
    )
    .bind(form_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(FormVersionSummary {
            id: row.try_get("id")?,
            version_label: row.try_get("version_label")?,
            status: row.try_get("status")?,
            version_major: row.try_get("version_major")?,
            version_minor: row.try_get("version_minor")?,
            version_patch: row.try_get("version_patch")?,
            compatibility_group_id: row.try_get("compatibility_group_id")?,
            compatibility_group_name: row.try_get("compatibility_group_name")?,
            published_at: row.try_get("published_at")?,
            field_count: row.try_get("field_count")?,
            semantic_bump: row.try_get("semantic_bump")?,
            started_new_major_line: row.try_get("started_new_major_line")?,
            publish_preview: None,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    for version in &mut versions {
        if version.status == "draft" {
            version.publish_preview =
                Some(build_form_publish_preview(pool, form_id, version.id).await?);
        }
    }

    let reports = sqlx::query("SELECT id, name FROM reports WHERE form_id = $1 ORDER BY name, id")
        .bind(form_id)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| {
            Ok(FormReportLink {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let workflows = sqlx::query(
        r#"
        SELECT
            workflows.id,
            workflows.name,
            workflows.slug,
            current_versions.id AS current_version_id,
            current_versions.version_label AS current_version_label,
            current_versions.status::text AS current_status,
            COUNT(workflow_assignments.id) FILTER (WHERE workflow_assignments.is_active) AS assignment_count
        FROM workflows
        LEFT JOIN LATERAL (
            SELECT id, version_label, status
            FROM workflow_versions
            WHERE workflow_id = workflows.id
            ORDER BY
                CASE status
                    WHEN 'published' THEN 0
                    WHEN 'draft' THEN 1
                    ELSE 2
                END,
                created_at DESC
            LIMIT 1
        ) AS current_versions ON true
        LEFT JOIN workflow_versions ON workflow_versions.workflow_id = workflows.id
        LEFT JOIN workflow_assignments
            ON workflow_assignments.workflow_version_id = workflow_versions.id
        WHERE workflows.form_id = $1
        GROUP BY
            workflows.id,
            workflows.name,
            workflows.slug,
            current_versions.id,
            current_versions.version_label,
            current_versions.status
        ORDER BY workflows.name, workflows.slug
        "#,
    )
    .bind(form_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(FormWorkflowLink {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            slug: row.try_get("slug")?,
            current_version_id: row.try_get("current_version_id")?,
            current_version_label: row.try_get("current_version_label")?,
            current_status: row.try_get("current_status")?,
            assignment_count: row.try_get("assignment_count")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let dataset_sources = sqlx::query(
        r#"
        SELECT
            datasets.id AS dataset_id,
            datasets.name AS dataset_name,
            dataset_sources.source_alias,
            dataset_sources.selection_rule::text AS selection_rule
        FROM dataset_sources
        JOIN datasets ON datasets.id = dataset_sources.dataset_id
        WHERE dataset_sources.form_id = $1
        ORDER BY datasets.name, dataset_sources.position
        "#,
    )
    .bind(form_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(FormDatasetSourceLink {
            dataset_id: row.try_get("dataset_id")?,
            dataset_name: row.try_get("dataset_name")?,
            source_alias: row.try_get("source_alias")?,
            selection_rule: row.try_get("selection_rule")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(FormDefinition {
        id: form.try_get("id")?,
        name: form.try_get("name")?,
        slug: form.try_get("slug")?,
        scope_node_type_id: form.try_get("scope_node_type_id")?,
        scope_node_type_name: form.try_get("scope_node_type_name")?,
        versions,
        workflows,
        reports,
        dataset_sources,
    })
}

pub async fn create_form_version(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(form_id): Path<Uuid>,
    Json(_payload): Json<CreateFormVersionRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("forms:write")?;
    require_form_exists(&state.pool, form_id).await?;

    let draft_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM form_versions
            WHERE form_id = $1
              AND status = 'draft'
        )
        "#,
    )
    .bind(form_id)
    .fetch_one(&state.pool)
    .await?;

    if draft_exists {
        return Err(ApiError::BadRequest(
            "only one draft form version may exist at a time".into(),
        ));
    }

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO form_versions (form_id)
        VALUES ($1)
        RETURNING id
        "#,
    )
    .bind(form_id)
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

async fn require_form_slug_available_for_form(
    pool: &sqlx::PgPool,
    form_id: Uuid,
    slug: &str,
) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM forms WHERE slug = $1 AND id <> $2)")
            .bind(slug)
            .bind(form_id)
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

pub async fn create_form_section(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(form_version_id): Path<Uuid>,
    Json(payload): Json<CreateFormSectionRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("forms:write")?;
    assert_form_version_draft(&state.pool, form_version_id).await?;
    require_text("section title", &payload.title)?;
    let description = normalize_form_section_description(&payload.description);
    let column_count = require_form_section_column_count(payload.column_count)?;

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO form_sections (form_version_id, title, description, column_count, position)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(form_version_id)
    .bind(payload.title)
    .bind(description)
    .bind(column_count)
    .bind(payload.position)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

pub async fn create_form_field(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(form_version_id): Path<Uuid>,
    Json(payload): Json<CreateFormFieldRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("forms:write")?;
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

/// Updates an editable form section in a draft form version.
pub async fn update_form_section(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(section_id): Path<Uuid>,
    Json(payload): Json<UpdateFormSectionRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("forms:write")?;
    let form_version_id = require_section_form_version(&state.pool, section_id).await?;
    assert_form_version_draft(&state.pool, form_version_id).await?;
    require_text("section title", &payload.title)?;
    let description = normalize_form_section_description(&payload.description);
    let column_count = require_form_section_column_count(payload.column_count)?;

    sqlx::query(
        "UPDATE form_sections SET title = $1, description = $2, column_count = $3, position = $4 WHERE id = $5",
    )
        .bind(payload.title)
        .bind(description)
        .bind(column_count)
        .bind(payload.position)
        .bind(section_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: section_id }))
}

/// Deletes an editable form section and its fields from a draft form version.
pub async fn delete_form_section(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(section_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("forms:write")?;
    let form_version_id = require_section_form_version(&state.pool, section_id).await?;
    assert_form_version_draft(&state.pool, form_version_id).await?;

    sqlx::query("DELETE FROM form_sections WHERE id = $1")
        .bind(section_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: section_id }))
}

/// Updates an editable form field in a draft form version.
pub async fn update_form_field(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(field_id): Path<Uuid>,
    Json(payload): Json<UpdateFormFieldRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("forms:write")?;
    let existing = require_form_field(&state.pool, field_id).await?;
    assert_form_version_draft(&state.pool, existing.form_version_id).await?;
    require_text("field key", &payload.key)?;
    require_text("field label", &payload.label)?;
    if payload.key != existing.key {
        require_form_field_key_available(&state.pool, existing.form_version_id, &payload.key)
            .await?;
    }
    let field_type = parse_field_type(&payload.field_type)?;
    assert_section_belongs_to_form_version(
        &state.pool,
        existing.form_version_id,
        payload.section_id,
    )
    .await?;

    sqlx::query(
        r#"
        UPDATE form_fields
        SET section_id = $1,
            key = $2,
            label = $3,
            field_type = $4::field_type,
            required = $5,
            position = $6
        WHERE id = $7
        "#,
    )
    .bind(payload.section_id)
    .bind(payload.key)
    .bind(payload.label)
    .bind(field_type.as_str())
    .bind(payload.required)
    .bind(payload.position)
    .bind(field_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(IdResponse { id: field_id }))
}

/// Deletes an editable form field from a draft form version.
pub async fn delete_form_field(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(field_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("forms:write")?;
    let existing = require_form_field(&state.pool, field_id).await?;
    assert_form_version_draft(&state.pool, existing.form_version_id).await?;

    sqlx::query("DELETE FROM form_fields WHERE id = $1")
        .bind(field_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: field_id }))
}

pub async fn publish_form_version(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(form_version_id): Path<Uuid>,
) -> ApiResult<Json<PublishFormVersionResponse>> {
    request.require_capability("forms:write")?;

    let preview = build_publish_computation(&state.pool, form_version_id).await?;
    let mut tx = state.pool.begin().await?;
    let version = require_publishable_form_version(&mut tx, form_version_id).await?;
    let current_latest_published =
        load_latest_published_version_tx(&mut tx, version.form_id).await?;
    let current_latest_published_id = current_latest_published.as_ref().map(|item| item.id);
    if current_latest_published_id != preview.latest_published_version_id {
        return Err(ApiError::BadRequest(
            "form publish state changed; reload the form and try publishing again".into(),
        ));
    }

    sqlx::query(
        r#"
        UPDATE form_versions
        SET status = 'superseded'::form_version_status
        WHERE form_id = $1
            AND version_major = $2
            AND id != $3
            AND status = 'published'::form_version_status
        "#,
    )
    .bind(version.form_id)
    .bind(preview.version_major)
    .bind(form_version_id)
    .execute(&mut *tx)
    .await?;

    let published_at: chrono::DateTime<chrono::Utc> = sqlx::query_scalar(
        r#"
        UPDATE form_versions
        SET
            version_label = $2,
            version_major = $3,
            version_minor = $4,
            version_patch = $5,
            semantic_bump = $6,
            started_new_major_line = $7,
            status = 'published'::form_version_status,
            published_at = now()
        WHERE id = $1 AND status = 'draft'::form_version_status
        RETURNING published_at
        "#,
    )
    .bind(form_version_id)
    .bind(&preview.version_label)
    .bind(preview.version_major)
    .bind(preview.version_minor)
    .bind(preview.version_patch)
    .bind(preview.semantic_bump.as_str())
    .bind(preview.starts_new_major_line)
    .fetch_one(&mut *tx)
    .await?;

    let _ =
        workflows::ensure_workflow_for_published_form_version_tx(&mut tx, form_version_id).await?;

    tx.commit().await?;

    Ok(Json(PublishFormVersionResponse {
        id: form_version_id,
        version_label: preview.version_label,
        version_major: preview.version_major,
        version_minor: preview.version_minor,
        version_patch: preview.version_patch,
        semantic_bump: preview.semantic_bump.as_str().into(),
        compatibility_label: preview.compatibility_label,
        status: "published".into(),
        published_at,
        dependency_warnings: preview.dependency_warnings,
        starts_new_major_line: preview.starts_new_major_line,
    }))
}

pub async fn render_form_version(
    State(state): State<AppState>,
    Path(form_version_id): Path<Uuid>,
) -> ApiResult<Json<RenderedForm>> {
    let version = sqlx::query(
        r#"
        SELECT
            form_versions.id,
            form_versions.form_id,
            forms.name AS form_name,
            form_versions.version_label,
            form_versions.status::text AS status
        FROM form_versions
        JOIN forms ON forms.id = form_versions.form_id
        WHERE form_versions.id = $1
        "#,
    )
    .bind(form_version_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("form version {form_version_id}")))?;

    let section_rows = sqlx::query(
        r#"
        SELECT id, title, description, column_count, position
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
            description: section.try_get("description")?,
            column_count: section.try_get("column_count")?,
            position: section.try_get("position")?,
            fields,
        });
    }

    Ok(Json(RenderedForm {
        form_version_id,
        form_id: version.try_get("form_id")?,
        form_name: version.try_get("form_name")?,
        version_label: version.try_get("version_label")?,
        status: version.try_get("status")?,
        sections,
    }))
}

async fn require_section_form_version(pool: &sqlx::PgPool, section_id: Uuid) -> ApiResult<Uuid> {
    sqlx::query_scalar("SELECT form_version_id FROM form_sections WHERE id = $1")
        .bind(section_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("form section {section_id}")))
}

struct ExistingFormField {
    form_version_id: Uuid,
    key: String,
}

async fn require_form_field(pool: &sqlx::PgPool, field_id: Uuid) -> ApiResult<ExistingFormField> {
    let row = sqlx::query("SELECT form_version_id, key FROM form_fields WHERE id = $1")
        .bind(field_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("form field {field_id}")))?;

    Ok(ExistingFormField {
        form_version_id: row.try_get("form_version_id")?,
        key: row.try_get("key")?,
    })
}

fn normalize_form_section_description(description: &str) -> String {
    description.trim().to_string()
}

fn require_form_section_column_count(column_count: i32) -> ApiResult<i32> {
    if matches!(column_count, 1 | 2) {
        Ok(column_count)
    } else {
        Err(ApiError::BadRequest(
            "section column count must be 1 or 2".into(),
        ))
    }
}

async fn assert_form_version_draft(pool: &sqlx::PgPool, form_version_id: Uuid) -> ApiResult<()> {
    let status: Option<String> =
        sqlx::query_scalar("SELECT status::text FROM form_versions WHERE id = $1")
            .bind(form_version_id)
            .fetch_optional(pool)
            .await?;

    match status.as_deref() {
        Some(status) => ensure_form_version_editable(status)
            .map_err(|error| ApiError::BadRequest(error.to_string())),
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
        Some(actual_form_version_id) => {
            ensure_section_belongs_to_form_version(actual_form_version_id == form_version_id)
                .map_err(|error| ApiError::BadRequest(error.to_string()))
        }
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
}

async fn require_publishable_form_version(
    tx: &mut Transaction<'_, Postgres>,
    form_version_id: Uuid,
) -> ApiResult<PublishableFormVersion> {
    let version = sqlx::query(
        r#"
        SELECT form_id, status::text AS status
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
    let section_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM form_sections WHERE form_version_id = $1")
            .bind(form_version_id)
            .fetch_one(&mut **tx)
            .await?;

    let field_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM form_fields WHERE form_version_id = $1")
            .bind(form_version_id)
            .fetch_one(&mut **tx)
            .await?;
    ensure_form_version_publishable(&status, section_count, field_count)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    Ok(PublishableFormVersion {
        form_id: version.try_get("form_id")?,
    })
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SemanticBump {
    Initial,
    Patch,
    Minor,
    Major,
}

impl SemanticBump {
    fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "INITIAL",
            Self::Patch => "PATCH",
            Self::Minor => "MINOR",
            Self::Major => "MAJOR",
        }
    }
}

#[derive(Clone, Copy)]
struct SemanticVersion {
    major: i32,
    minor: i32,
    patch: i32,
}

impl SemanticVersion {
    fn parse(version_label: &str) -> Option<Self> {
        let mut parts = version_label.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }

    fn increment(self, bump: SemanticBump) -> Self {
        match bump {
            SemanticBump::Initial => Self {
                major: 1,
                minor: 0,
                patch: 0,
            },
            SemanticBump::Patch => Self {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
            },
            SemanticBump::Minor => Self {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
            },
            SemanticBump::Major => Self {
                major: self.major + 1,
                minor: 0,
                patch: 0,
            },
        }
    }

    fn label(self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

struct ComparableFormField {
    key: String,
    label: String,
    field_type: String,
    required: bool,
    section_title: String,
    section_position: i32,
    field_position: i32,
}

struct FormVersionContract {
    section_signature: Vec<String>,
    fields_by_key: BTreeMap<String, ComparableFormField>,
}

struct LatestPublishedVersion {
    id: Uuid,
    version_label: Option<String>,
    version_major: Option<i32>,
    version_minor: Option<i32>,
    version_patch: Option<i32>,
}

struct PublishComputation {
    latest_published_version_id: Option<Uuid>,
    version_label: String,
    version_major: i32,
    version_minor: i32,
    version_patch: i32,
    semantic_bump: SemanticBump,
    compatibility_label: String,
    starts_new_major_line: bool,
    dependency_warnings: Vec<String>,
}

async fn build_form_publish_preview(
    pool: &sqlx::PgPool,
    form_id: Uuid,
    form_version_id: Uuid,
) -> ApiResult<FormPublishPreview> {
    let preview = build_publish_computation_for_form(pool, form_id, form_version_id).await?;
    Ok(FormPublishPreview {
        version_label: preview.version_label,
        version_major: preview.version_major,
        version_minor: preview.version_minor,
        version_patch: preview.version_patch,
        semantic_bump: preview.semantic_bump.as_str().into(),
        compatibility_label: preview.compatibility_label,
        starts_new_major_line: preview.starts_new_major_line,
        dependency_warnings: preview.dependency_warnings,
    })
}

async fn build_publish_computation(
    pool: &sqlx::PgPool,
    form_version_id: Uuid,
) -> ApiResult<PublishComputation> {
    let form_id: Uuid = sqlx::query_scalar("SELECT form_id FROM form_versions WHERE id = $1")
        .bind(form_version_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("form version {form_version_id}")))?;
    build_publish_computation_for_form(pool, form_id, form_version_id).await
}

async fn build_publish_computation_for_form(
    pool: &sqlx::PgPool,
    form_id: Uuid,
    form_version_id: Uuid,
) -> ApiResult<PublishComputation> {
    let latest_published = load_latest_published_version(pool, form_id).await?;
    let Some(latest_published) = latest_published else {
        return Ok(PublishComputation {
            latest_published_version_id: None,
            version_label: "1.0.0".into(),
            version_major: 1,
            version_minor: 0,
            version_patch: 0,
            semantic_bump: SemanticBump::Initial,
            compatibility_label: compatibility_label_for_major(1),
            starts_new_major_line: true,
            dependency_warnings: Vec::new(),
        });
    };

    let current_version = current_semantic_version(&latest_published).unwrap_or(SemanticVersion {
        major: 1,
        minor: 0,
        patch: 0,
    });
    let published_contract = load_form_version_contract(pool, latest_published.id).await?;
    let draft_contract = load_form_version_contract(pool, form_version_id).await?;
    let semantic_bump = classify_contract_change(&published_contract, &draft_contract);
    let next_version = current_version.increment(semantic_bump);
    let starts_new_major_line =
        matches!(semantic_bump, SemanticBump::Initial | SemanticBump::Major);
    let dependency_warnings = if starts_new_major_line {
        load_direct_dependency_warnings(pool, form_id).await?
    } else {
        Vec::new()
    };

    Ok(PublishComputation {
        latest_published_version_id: Some(latest_published.id),
        version_label: next_version.label(),
        version_major: next_version.major,
        version_minor: next_version.minor,
        version_patch: next_version.patch,
        semantic_bump,
        compatibility_label: compatibility_label_for_major(next_version.major),
        starts_new_major_line,
        dependency_warnings,
    })
}

fn current_semantic_version(version: &LatestPublishedVersion) -> Option<SemanticVersion> {
    match (
        version.version_major,
        version.version_minor,
        version.version_patch,
    ) {
        (Some(major), Some(minor), Some(patch)) => Some(SemanticVersion {
            major,
            minor,
            patch,
        }),
        _ => version
            .version_label
            .as_deref()
            .and_then(SemanticVersion::parse),
    }
}

fn classify_contract_change(
    published: &FormVersionContract,
    draft: &FormVersionContract,
) -> SemanticBump {
    let mut bump = SemanticBump::Patch;

    for (key, published_field) in &published.fields_by_key {
        let Some(draft_field) = draft.fields_by_key.get(key) else {
            return SemanticBump::Major;
        };

        if published_field.field_type != draft_field.field_type {
            return SemanticBump::Major;
        }
        if !published_field.required && draft_field.required {
            return SemanticBump::Major;
        }
        if published_field.required && !draft_field.required {
            bump = bump.max(SemanticBump::Minor);
        }
        if published_field.label != draft_field.label
            || published_field.section_title != draft_field.section_title
            || published_field.section_position != draft_field.section_position
            || published_field.field_position != draft_field.field_position
        {
            bump = bump.max(SemanticBump::Patch);
        }
    }

    for draft_field in draft.fields_by_key.values() {
        if !published.fields_by_key.contains_key(&draft_field.key) {
            if draft_field.required {
                return SemanticBump::Major;
            }
            bump = bump.max(SemanticBump::Minor);
        }
    }

    if published.section_signature != draft.section_signature {
        bump = bump.max(SemanticBump::Patch);
    }

    bump
}

fn compatibility_label_for_major(major: i32) -> String {
    format!("Compatible with v{major}.x")
}

async fn load_form_version_contract(
    pool: &sqlx::PgPool,
    form_version_id: Uuid,
) -> ApiResult<FormVersionContract> {
    let sections = sqlx::query(
        r#"
        SELECT title, description, column_count, position
        FROM form_sections
        WHERE form_version_id = $1
        ORDER BY position, title
        "#,
    )
    .bind(form_version_id)
    .fetch_all(pool)
    .await?;
    let fields = sqlx::query(
        r#"
        SELECT
            form_fields.key,
            form_fields.label,
            form_fields.field_type::text AS field_type,
            form_fields.required,
            form_fields.position,
            form_sections.title AS section_title,
            form_sections.position AS section_position
        FROM form_fields
        JOIN form_sections ON form_sections.id = form_fields.section_id
        WHERE form_fields.form_version_id = $1
        ORDER BY form_fields.position, form_fields.key
        "#,
    )
    .bind(form_version_id)
    .fetch_all(pool)
    .await?;

    let section_signature = sections
        .into_iter()
        .map(|row| {
            Ok(format!(
                "{}:{}:{}:{}",
                row.try_get::<i32, _>("position")?,
                row.try_get::<String, _>("title")?,
                row.try_get::<String, _>("description")?,
                row.try_get::<i32, _>("column_count")?,
            ))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let mut fields_by_key = BTreeMap::new();
    for row in fields {
        let field = ComparableFormField {
            key: row.try_get("key")?,
            label: row.try_get("label")?,
            field_type: row.try_get("field_type")?,
            required: row.try_get("required")?,
            section_title: row.try_get("section_title")?,
            section_position: row.try_get("section_position")?,
            field_position: row.try_get("position")?,
        };
        fields_by_key.insert(field.key.clone(), field);
    }

    Ok(FormVersionContract {
        section_signature,
        fields_by_key,
    })
}

async fn load_latest_published_version(
    pool: &sqlx::PgPool,
    form_id: Uuid,
) -> ApiResult<Option<LatestPublishedVersion>> {
    let row = sqlx::query(
        r#"
        SELECT id, version_label, version_major, version_minor, version_patch
        FROM form_versions
        WHERE form_id = $1
          AND status = 'published'::form_version_status
        ORDER BY published_at DESC NULLS LAST, created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(form_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => Ok(Some(LatestPublishedVersion {
            id: row.try_get("id")?,
            version_label: row.try_get("version_label")?,
            version_major: row.try_get("version_major")?,
            version_minor: row.try_get("version_minor")?,
            version_patch: row.try_get("version_patch")?,
        })),
        None => Ok(None),
    }
}

async fn load_latest_published_version_tx(
    tx: &mut Transaction<'_, Postgres>,
    form_id: Uuid,
) -> ApiResult<Option<LatestPublishedVersion>> {
    let row = sqlx::query(
        r#"
        SELECT id, version_label, version_major, version_minor, version_patch
        FROM form_versions
        WHERE form_id = $1
          AND status = 'published'::form_version_status
        ORDER BY published_at DESC NULLS LAST, created_at DESC, id DESC
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(form_id)
    .fetch_optional(&mut **tx)
    .await?;

    match row {
        Some(row) => Ok(Some(LatestPublishedVersion {
            id: row.try_get("id")?,
            version_label: row.try_get("version_label")?,
            version_major: row.try_get("version_major")?,
            version_minor: row.try_get("version_minor")?,
            version_patch: row.try_get("version_patch")?,
        })),
        None => Ok(None),
    }
}

async fn load_direct_dependency_warnings(
    pool: &sqlx::PgPool,
    form_id: Uuid,
) -> ApiResult<Vec<String>> {
    let dataset_rows = sqlx::query(
        r#"
        SELECT datasets.name AS dataset_name, dataset_sources.source_alias
        FROM dataset_sources
        JOIN datasets ON datasets.id = dataset_sources.dataset_id
        WHERE dataset_sources.form_id = $1
          AND dataset_sources.form_version_major IS NULL
        ORDER BY datasets.name, dataset_sources.source_alias
        "#,
    )
    .bind(form_id)
    .fetch_all(pool)
    .await?;
    let report_rows = sqlx::query(
        "SELECT name FROM reports WHERE form_id = $1 AND form_version_major IS NULL ORDER BY name, id",
    )
        .bind(form_id)
        .fetch_all(pool)
        .await?;

    let mut warnings = dataset_rows
        .into_iter()
        .map(|row| {
            Ok(format!(
                "Dataset source '{} / {}' is bound directly to this form and should be reviewed.",
                row.try_get::<String, _>("dataset_name")?,
                row.try_get::<String, _>("source_alias")?
            ))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    warnings.extend(
        report_rows
            .into_iter()
            .map(|row| {
                Ok(format!(
                    "Report '{}' is bound directly to this form and should be reviewed.",
                    row.try_get::<String, _>("name")?
                ))
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
    );
    Ok(warnings)
}
