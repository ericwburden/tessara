//! Component authoring and read endpoints.
//!
//! Components are presentation assets over dataset revisions. This module keeps
//! route behavior and scope checks together while the public wire types live in
//! `dto`.

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use sqlx::Row;
use uuid::Uuid;

mod dto;

pub use dto::{
    ComponentDefinition, ComponentSummary, ComponentVersionSummary, CreateComponentRequest,
    CreateComponentVersionRequest,
};

use crate::{
    auth, datasets,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, require_text},
};

/// Creates a component shell before versions are attached.
pub async fn create_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "components:manage").await?;
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

/// Creates a draft or published component version over a dataset revision.
pub async fn create_component_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    Json(payload): Json<CreateComponentVersionRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "components:manage").await?;
    require_component_exists(&state.pool, component_id).await?;
    require_dataset_revision_exists(&state.pool, payload.dataset_revision_id).await?;
    require_dataset_revision_fully_in_capability_scope(
        &state.pool,
        &account,
        "components:manage",
        payload.dataset_revision_id,
    )
    .await?;
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
    if status == "published" {
        sqlx::query(
            r#"
            UPDATE component_versions
            SET status = 'superseded'::component_version_status
            WHERE component_id = $1
              AND status = 'published'::component_version_status
            "#,
        )
        .bind(component_id)
        .execute(&mut *tx)
        .await?;
    }
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

    tx.commit().await?;
    Ok(Json(IdResponse { id }))
}

/// Lists components visible to the caller's component-read capability scope.
pub async fn list_components(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ComponentSummary>>> {
    let account = auth::require_capability(&state.pool, &headers, "components:read").await?;
    let rows = match auth::capability_boundary(&state.pool, &account, "components:read").await? {
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            sqlx::query(
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
        JOIN dataset_revisions
            ON dataset_revisions.id = current_versions.dataset_revision_id
        JOIN dataset_scope_nodes
            ON dataset_scope_nodes.dataset_id = dataset_revisions.dataset_id
           AND dataset_scope_nodes.node_id = ANY($1)
        ORDER BY components.name, components.id
        "#,
            )
            .bind(scope_ids)
            .fetch_all(&state.pool)
            .await?
        }
        auth::CapabilityBoundary::Global => {
            sqlx::query(
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
            .await?
        }
        auth::CapabilityBoundary::None => {
            return Err(ApiError::Forbidden("components:read".into()));
        }
    };

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

/// Loads a component by UUID or slug when its dataset revision is readable.
pub async fn get_component_by_ref(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> ApiResult<Json<ComponentDefinition>> {
    let account = auth::require_capability(&state.pool, &headers, "components:read").await?;
    let boundary = auth::capability_boundary(&state.pool, &account, "components:read").await?;
    let component = sqlx::query(
        "SELECT id, name, slug, description FROM components WHERE id::text = $1 OR slug = $1",
    )
    .bind(component_ref.clone())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("component {component_ref}")))?;
    let component_id = component.try_get("id")?;

    require_component_visible_for_boundary(&state.pool, component_id, &boundary, "components:read")
        .await?;

    let versions = load_component_versions(&state.pool, &account, component_id).await?;
    Ok(Json(ComponentDefinition {
        id: component_id,
        name: component.try_get("name")?,
        slug: component.try_get("slug")?,
        description: component.try_get("description")?,
        versions,
    }))
}

async fn load_component_versions(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    component_id: Uuid,
) -> ApiResult<Vec<ComponentVersionSummary>> {
    let rows = match auth::capability_boundary(pool, account, "components:read").await? {
        auth::CapabilityBoundary::Scoped(scope_ids) => sqlx::query(
            r#"
        SELECT component_versions.id, component_versions.component_id, component_versions.dataset_revision_id, component_versions.component_type::text AS component_type,
               component_versions.status::text AS status, component_versions.version_label, component_versions.config
        FROM component_versions
        JOIN dataset_revisions ON dataset_revisions.id = component_versions.dataset_revision_id
        JOIN dataset_scope_nodes ON dataset_scope_nodes.dataset_id = dataset_revisions.dataset_id
        WHERE component_id = $1
          AND dataset_scope_nodes.node_id = ANY($2)
        ORDER BY component_versions.version_number DESC, component_versions.created_at DESC
        "#,
        )
        .bind(component_id)
        .bind(scope_ids)
        .fetch_all(pool)
        .await?,
        auth::CapabilityBoundary::Global => sqlx::query(
            r#"
        SELECT id, component_id, dataset_revision_id, component_type::text AS component_type,
               status::text AS status, version_label, config
        FROM component_versions
        WHERE component_id = $1
        ORDER BY component_versions.version_number DESC, component_versions.created_at DESC
        "#,
        )
        .bind(component_id)
        .fetch_all(pool)
        .await?,
        auth::CapabilityBoundary::None => return Err(ApiError::Forbidden("components:read".into())),
    };
    Ok(rows
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

async fn require_dataset_revision_fully_in_capability_scope(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    capability: &str,
    dataset_revision_id: Uuid,
) -> ApiResult<()> {
    let dataset_id: Uuid =
        sqlx::query_scalar("SELECT dataset_id FROM dataset_revisions WHERE id = $1")
            .bind(dataset_revision_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("dataset revision {dataset_revision_id}")))?;
    let node_ids = datasets::load_dataset_scope_node_ids(pool, dataset_id).await?;
    auth::require_capability_contains_nodes(pool, account, capability, &node_ids).await
}

async fn require_component_visible_for_boundary(
    pool: &sqlx::PgPool,
    component_id: Uuid,
    boundary: &auth::CapabilityBoundary,
    capability: &str,
) -> ApiResult<()> {
    match boundary {
        auth::CapabilityBoundary::Global => Ok(()),
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            let visible = sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS(
                    SELECT 1
                    FROM component_versions
                    JOIN dataset_revisions
                      ON dataset_revisions.id = component_versions.dataset_revision_id
                    JOIN dataset_scope_nodes
                      ON dataset_scope_nodes.dataset_id = dataset_revisions.dataset_id
                    WHERE component_versions.component_id = $1
                      AND dataset_scope_nodes.node_id = ANY($2)
                )
                "#,
            )
            .bind(component_id)
            .bind(scope_ids)
            .fetch_one(pool)
            .await?;
            if visible {
                Ok(())
            } else {
                Err(ApiError::Forbidden(capability.into()))
            }
        }
        auth::CapabilityBoundary::None => Err(ApiError::Forbidden(capability.into())),
    }
}
