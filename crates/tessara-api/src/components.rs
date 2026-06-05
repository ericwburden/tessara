use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, require_text},
};

#[derive(Deserialize)]
pub struct CreateComponentRequest {
    name: String,
    slug: String,
    description: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateComponentVersionRequest {
    dataset_revision_id: Uuid,
    component_type: String,
    config: Value,
    publish: Option<bool>,
}

#[derive(Serialize)]
pub struct ComponentSummary {
    id: Uuid,
    name: String,
    slug: String,
    description: Option<String>,
    current_version_id: Option<Uuid>,
    current_component_type: Option<String>,
}

#[derive(Serialize)]
pub struct ComponentDefinition {
    id: Uuid,
    name: String,
    slug: String,
    description: Option<String>,
    versions: Vec<ComponentVersionSummary>,
}

#[derive(Serialize)]
pub struct ComponentVersionSummary {
    id: Uuid,
    component_id: Uuid,
    dataset_revision_id: Uuid,
    component_type: String,
    status: String,
    version_label: String,
    config: Value,
}

pub async fn create_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "components:write").await?;
    require_text("component name", &payload.name)?;
    require_text("component slug", &payload.slug)?;

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO components (name, slug, description)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.slug.trim())
    .bind(payload.description)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

pub async fn create_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    Json(payload): Json<CreateComponentVersionRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "components:write").await?;
    require_component_exists(&state.pool, component_id).await?;
    require_dataset_revision_exists(&state.pool, payload.dataset_revision_id).await?;
    validate_component_type(&payload.component_type)?;

    let status = if payload.publish.unwrap_or(false) {
        "published"
    } else {
        "draft"
    };
    let version_number: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_number), 0) + 1 FROM component_versions WHERE component_id = $1",
    )
    .bind(component_id)
    .fetch_one(&state.pool)
    .await?;
    let version_label = version_number.to_string();

    let mut tx = state.pool.begin().await?;
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO component_versions
            (component_id, dataset_revision_id, component_type, version_number, version_label, status, config, published_at)
        VALUES ($1, $2, $3, $4, $5, $6::component_version_status, $7, CASE WHEN $6 = 'published' THEN now() ELSE NULL END)
        RETURNING id
        "#,
    )
    .bind(component_id)
    .bind(payload.dataset_revision_id)
    .bind(payload.component_type)
    .bind(version_number)
    .bind(version_label)
    .bind(status)
    .bind(payload.config)
    .fetch_one(&mut *tx)
    .await?;

    if status == "published" {
        sqlx::query(
            r#"
            UPDATE component_versions
            SET status = 'superseded'::component_version_status
            WHERE component_id = $1
              AND id <> $2
              AND status = 'published'::component_version_status
            "#,
        )
        .bind(component_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(Json(IdResponse { id }))
}

pub async fn list_components(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ComponentSummary>>> {
    auth::require_capability(&state.pool, &headers, "components:read").await?;
    let rows = sqlx::query(
        r#"
        SELECT
            components.id,
            components.name,
            components.slug,
            components.description,
            current_versions.id AS current_version_id,
            current_versions.component_type::text AS current_component_type
        FROM components
        LEFT JOIN component_versions AS current_versions
            ON current_versions.component_id = components.id
           AND current_versions.status = 'published'::component_version_status
        ORDER BY components.name, components.id
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|row| {
                Ok(ComponentSummary {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    slug: row.try_get("slug")?,
                    description: row.try_get("description")?,
                    current_version_id: row.try_get("current_version_id")?,
                    current_component_type: row.try_get("current_component_type")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
    ))
}

pub async fn get_component_by_ref(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> ApiResult<Json<ComponentDefinition>> {
    auth::require_capability(&state.pool, &headers, "components:read").await?;
    let component = sqlx::query(
        "SELECT id, name, slug, description FROM components WHERE id::text = $1 OR slug = $1",
    )
    .bind(component_ref.clone())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("component {component_ref}")))?;

    let versions = load_component_versions(&state.pool, component.try_get("id")?).await?;
    Ok(Json(ComponentDefinition {
        id: component.try_get("id")?,
        name: component.try_get("name")?,
        slug: component.try_get("slug")?,
        description: component.try_get("description")?,
        versions,
    }))
}

async fn load_component_versions(
    pool: &sqlx::PgPool,
    component_id: Uuid,
) -> ApiResult<Vec<ComponentVersionSummary>> {
    Ok(sqlx::query(
        r#"
        SELECT id, component_id, dataset_revision_id, component_type::text AS component_type,
               status::text AS status, version_label, config
        FROM component_versions
        WHERE component_id = $1
        ORDER BY version_number DESC, created_at DESC
        "#,
    )
    .bind(component_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ComponentVersionSummary {
            id: row.try_get("id")?,
            component_id: row.try_get("component_id")?,
            dataset_revision_id: row.try_get("dataset_revision_id")?,
            component_type: row.try_get("component_type")?,
            status: row.try_get("status")?,
            version_label: row.try_get("version_label")?,
            config: row.try_get("config")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

fn validate_component_type(component_type: &str) -> ApiResult<()> {
    match component_type {
        "detail_table" | "aggregate_table" | "bar" | "line" | "pie" | "donut" | "stat_card" => {
            Ok(())
        }
        other => Err(ApiError::BadRequest(format!(
            "unsupported component type '{other}'"
        ))),
    }
}

async fn require_component_exists(pool: &sqlx::PgPool, component_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM components WHERE id = $1)")
        .bind(component_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("component {component_id}")))
    }
}

async fn require_dataset_revision_exists(
    pool: &sqlx::PgPool,
    dataset_revision_id: Uuid,
) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM dataset_revisions WHERE id = $1)")
            .bind(dataset_revision_id)
            .fetch_one(pool)
            .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!(
            "dataset revision {dataset_revision_id}"
        )))
    }
}
