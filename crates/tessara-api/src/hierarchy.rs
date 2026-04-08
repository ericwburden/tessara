use std::{collections::HashMap, str::FromStr};

use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use tessara_core::{FieldType, FieldTypeError, validate_required_text};
use tessara_hierarchy::validate_node_type_relationship;
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
};

#[derive(Deserialize)]
pub struct CreateNodeTypeRequest {
    name: String,
    slug: String,
}

#[derive(Deserialize)]
pub struct UpdateNodeTypeRequest {
    name: String,
    slug: String,
}

#[derive(Deserialize)]
pub struct CreateNodeTypeRelationshipRequest {
    parent_node_type_id: Uuid,
    child_node_type_id: Uuid,
}

#[derive(Deserialize)]
pub struct CreateNodeMetadataFieldRequest {
    node_type_id: Uuid,
    key: String,
    label: String,
    field_type: String,
    required: bool,
}

#[derive(Deserialize)]
pub struct UpdateNodeMetadataFieldRequest {
    key: String,
    label: String,
    field_type: String,
    required: bool,
}

#[derive(Deserialize)]
pub struct CreateNodeRequest {
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    name: String,
    #[serde(default)]
    metadata: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub struct UpdateNodeRequest {
    parent_node_id: Option<Uuid>,
    name: String,
    #[serde(default)]
    metadata: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub struct ListNodesQuery {
    q: Option<String>,
}

#[derive(Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}

#[derive(Serialize)]
pub struct NodeResponse {
    id: Uuid,
    node_type_id: Uuid,
    node_type_name: String,
    parent_node_id: Option<Uuid>,
    parent_node_name: Option<String>,
    name: String,
    metadata: Value,
}

#[derive(Serialize)]
pub struct NodeTypeSummary {
    id: Uuid,
    name: String,
    slug: String,
    node_count: i64,
}

#[derive(Serialize)]
pub struct NodeTypeRelationshipSummary {
    parent_node_type_id: Uuid,
    parent_name: String,
    child_node_type_id: Uuid,
    child_name: String,
}

#[derive(Serialize)]
pub struct NodeMetadataFieldSummary {
    id: Uuid,
    node_type_id: Uuid,
    node_type_name: String,
    key: String,
    label: String,
    field_type: String,
    required: bool,
}

pub async fn create_node_type(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNodeTypeRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    require_text("node type name", &payload.name)?;
    require_text("node type slug", &payload.slug)?;
    require_node_type_slug_available(&state.pool, &payload.slug).await?;

    let id = sqlx::query_scalar("INSERT INTO node_types (name, slug) VALUES ($1, $2) RETURNING id")
        .bind(payload.name)
        .bind(payload.slug)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(IdResponse { id }))
}

/// Updates node-type display metadata used by hierarchy and form-builder screens.
pub async fn update_node_type(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(node_type_id): Path<Uuid>,
    Json(payload): Json<UpdateNodeTypeRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    require_text("node type name", &payload.name)?;
    require_text("node type slug", &payload.slug)?;
    require_node_type_exists(&state.pool, node_type_id).await?;
    require_node_type_slug_available_for_type(&state.pool, node_type_id, &payload.slug).await?;

    let id = sqlx::query_scalar(
        r#"
        UPDATE node_types
        SET name = $2, slug = $3
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(node_type_id)
    .bind(payload.name)
    .bind(payload.slug)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

/// Lists configured hierarchy node types for the admin builder shell.
pub async fn list_node_types(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<NodeTypeSummary>>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;

    let rows = sqlx::query(
        r#"
        SELECT node_types.id, node_types.name, node_types.slug, COUNT(nodes.id) AS node_count
        FROM node_types
        LEFT JOIN nodes ON nodes.node_type_id = node_types.id
        GROUP BY node_types.id, node_types.name, node_types.slug, node_types.created_at
        ORDER BY node_types.created_at, node_types.name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let node_types = rows
        .into_iter()
        .map(|row| {
            Ok(NodeTypeSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                node_count: row.try_get("node_count")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(node_types))
}

/// Lists configured parent/child hierarchy relationships for admin screens.
pub async fn list_node_type_relationships(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<NodeTypeRelationshipSummary>>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;

    let rows = sqlx::query(
        r#"
        SELECT
            node_type_relationships.parent_node_type_id,
            parent_node_types.name AS parent_name,
            node_type_relationships.child_node_type_id,
            child_node_types.name AS child_name
        FROM node_type_relationships
        JOIN node_types AS parent_node_types
            ON parent_node_types.id = node_type_relationships.parent_node_type_id
        JOIN node_types AS child_node_types
            ON child_node_types.id = node_type_relationships.child_node_type_id
        ORDER BY parent_node_types.name, child_node_types.name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let relationships = rows
        .into_iter()
        .map(|row| {
            Ok(NodeTypeRelationshipSummary {
                parent_node_type_id: row.try_get("parent_node_type_id")?,
                parent_name: row.try_get("parent_name")?,
                child_node_type_id: row.try_get("child_node_type_id")?,
                child_name: row.try_get("child_name")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(relationships))
}

pub async fn create_node_type_relationship(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNodeTypeRelationshipRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    require_node_type_exists(&state.pool, payload.parent_node_type_id).await?;
    require_node_type_exists(&state.pool, payload.child_node_type_id).await?;
    assert_relationship_is_acyclic(
        &state.pool,
        payload.parent_node_type_id,
        payload.child_node_type_id,
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO node_type_relationships (parent_node_type_id, child_node_type_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(payload.parent_node_type_id)
    .bind(payload.child_node_type_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(IdResponse {
        id: payload.child_node_type_id,
    }))
}

/// Removes a parent/child node-type relationship if no existing nodes depend on it.
pub async fn delete_node_type_relationship(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((parent_node_type_id, child_node_type_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    require_node_type_relationship_exists(&state.pool, parent_node_type_id, child_node_type_id)
        .await?;
    assert_relationship_unused(&state.pool, parent_node_type_id, child_node_type_id).await?;

    sqlx::query(
        r#"
        DELETE FROM node_type_relationships
        WHERE parent_node_type_id = $1 AND child_node_type_id = $2
        "#,
    )
    .bind(parent_node_type_id)
    .bind(child_node_type_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(IdResponse {
        id: child_node_type_id,
    }))
}

/// Lists node metadata field definitions for admin screens.
pub async fn list_node_metadata_fields(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<NodeMetadataFieldSummary>>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;

    let rows = sqlx::query(
        r#"
        SELECT
            node_metadata_field_definitions.id,
            node_metadata_field_definitions.node_type_id,
            node_types.name AS node_type_name,
            node_metadata_field_definitions.key,
            node_metadata_field_definitions.label,
            node_metadata_field_definitions.field_type::text AS field_type,
            node_metadata_field_definitions.required
        FROM node_metadata_field_definitions
        JOIN node_types ON node_types.id = node_metadata_field_definitions.node_type_id
        ORDER BY node_types.name, node_metadata_field_definitions.key
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let fields = rows
        .into_iter()
        .map(|row| {
            Ok(NodeMetadataFieldSummary {
                id: row.try_get("id")?,
                node_type_id: row.try_get("node_type_id")?,
                node_type_name: row.try_get("node_type_name")?,
                key: row.try_get("key")?,
                label: row.try_get("label")?,
                field_type: row.try_get("field_type")?,
                required: row.try_get("required")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(fields))
}

pub async fn create_node_metadata_field(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNodeMetadataFieldRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    require_node_type_exists(&state.pool, payload.node_type_id).await?;
    require_text("metadata key", &payload.key)?;
    require_text("metadata label", &payload.label)?;
    require_node_metadata_key_available(&state.pool, payload.node_type_id, &payload.key).await?;
    let field_type = parse_field_type(&payload.field_type)?;

    if payload.required {
        let existing_node_count = node_count_for_type(&state.pool, payload.node_type_id).await?;
        if existing_node_count > 0 {
            return Err(ApiError::BadRequest(
                "required metadata fields cannot be added after nodes of that type exist".into(),
            ));
        }
    }

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO node_metadata_field_definitions
            (node_type_id, key, label, field_type, required)
        VALUES ($1, $2, $3, $4::field_type, $5)
        RETURNING id
        "#,
    )
    .bind(payload.node_type_id)
    .bind(payload.key)
    .bind(payload.label)
    .bind(field_type.as_str())
    .bind(payload.required)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(IdResponse { id }))
}

struct ExistingMetadataField {
    node_type_id: Uuid,
    key: String,
    field_type: String,
    required: bool,
}

/// Updates metadata field display and safe schema settings for a node type.
pub async fn update_node_metadata_field(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(field_id): Path<Uuid>,
    Json(payload): Json<UpdateNodeMetadataFieldRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    let existing = require_node_metadata_field(&state.pool, field_id).await?;
    require_text("metadata key", &payload.key)?;
    require_text("metadata label", &payload.label)?;
    let field_type = parse_field_type(&payload.field_type)?;

    let existing_node_count = node_count_for_type(&state.pool, existing.node_type_id).await?;
    if existing_node_count > 0 {
        if payload.key != existing.key {
            return Err(ApiError::BadRequest(
                "metadata field keys cannot be changed after nodes of that type exist".into(),
            ));
        }

        if field_type.as_str() != existing.field_type {
            return Err(ApiError::BadRequest(
                "metadata field types cannot be changed after nodes of that type exist".into(),
            ));
        }

        if payload.required && !existing.required {
            return Err(ApiError::BadRequest(
                "metadata fields cannot be made required after nodes of that type exist".into(),
            ));
        }
    }

    if payload.key != existing.key {
        require_node_metadata_key_available(&state.pool, existing.node_type_id, &payload.key)
            .await?;
    }

    sqlx::query(
        r#"
        UPDATE node_metadata_field_definitions
        SET key = $1, label = $2, field_type = $3::field_type, required = $4
        WHERE id = $5
        "#,
    )
    .bind(payload.key)
    .bind(payload.label)
    .bind(field_type.as_str())
    .bind(payload.required)
    .bind(field_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(IdResponse { id: field_id }))
}

pub async fn create_node(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNodeRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    require_node_type_exists(&state.pool, payload.node_type_id).await?;
    require_text("node name", &payload.name)?;

    if let Some(parent_node_id) = payload.parent_node_id {
        let parent_type_id: Uuid =
            sqlx::query_scalar("SELECT node_type_id FROM nodes WHERE id = $1")
                .bind(parent_node_id)
                .fetch_optional(&state.pool)
                .await?
                .ok_or_else(|| ApiError::NotFound(format!("parent node {parent_node_id}")))?;

        let relationship_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM node_type_relationships
                WHERE parent_node_type_id = $1 AND child_node_type_id = $2
            )
            "#,
        )
        .bind(parent_type_id)
        .bind(payload.node_type_id)
        .fetch_one(&state.pool)
        .await?;

        if !relationship_exists {
            return Err(ApiError::BadRequest(
                "node parent type is not allowed for this child type".into(),
            ));
        }
    }

    let field_rows = metadata_field_rows(&state.pool, payload.node_type_id).await?;
    validate_node_metadata_values(&field_rows, &payload.metadata, true)?;

    let node_id: Uuid = sqlx::query_scalar(
        "INSERT INTO nodes (node_type_id, parent_node_id, name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(payload.node_type_id)
    .bind(payload.parent_node_id)
    .bind(payload.name)
    .fetch_one(&state.pool)
    .await?;

    upsert_node_metadata_values(&state.pool, node_id, &field_rows, &payload.metadata).await?;

    Ok(Json(IdResponse { id: node_id }))
}

/// Updates a hierarchy node name, parent, and provided metadata values.
pub async fn update_node(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(node_id): Path<Uuid>,
    Json(payload): Json<UpdateNodeRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    require_text("node name", &payload.name)?;
    let node_type_id = require_node_type_for_node(&state.pool, node_id).await?;
    assert_parent_allowed(&state.pool, node_id, node_type_id, payload.parent_node_id).await?;

    let field_rows = metadata_field_rows(&state.pool, node_type_id).await?;
    validate_node_metadata_values(&field_rows, &payload.metadata, false)?;

    sqlx::query("UPDATE nodes SET parent_node_id = $1, name = $2 WHERE id = $3")
        .bind(payload.parent_node_id)
        .bind(payload.name)
        .bind(node_id)
        .execute(&state.pool)
        .await?;

    upsert_node_metadata_values(&state.pool, node_id, &field_rows, &payload.metadata).await?;

    Ok(Json(IdResponse { id: node_id }))
}

pub async fn list_nodes(
    State(state): State<AppState>,
    Query(query): Query<ListNodesQuery>,
) -> ApiResult<Json<Vec<NodeResponse>>> {
    let search = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let rows = sqlx::query(
        r#"
        SELECT
            nodes.id,
            nodes.node_type_id,
            node_types.name AS node_type_name,
            nodes.parent_node_id,
            parent_nodes.name AS parent_node_name,
            nodes.name,
            COALESCE(
                jsonb_object_agg(
                    node_metadata_field_definitions.key,
                    node_metadata_values.value
                ) FILTER (WHERE node_metadata_field_definitions.key IS NOT NULL),
                '{}'::jsonb
            ) AS metadata
        FROM nodes
        JOIN node_types ON node_types.id = nodes.node_type_id
        LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
        LEFT JOIN node_metadata_values ON node_metadata_values.node_id = nodes.id
        LEFT JOIN node_metadata_field_definitions
            ON node_metadata_field_definitions.id = node_metadata_values.field_definition_id
        WHERE (
            $1::text IS NULL
            OR nodes.name ILIKE '%' || $1 || '%'
            OR node_types.name ILIKE '%' || $1 || '%'
        )
        GROUP BY
            nodes.id,
            nodes.node_type_id,
            node_types.name,
            nodes.parent_node_id,
            parent_nodes.name,
            nodes.name,
            nodes.created_at
        ORDER BY nodes.created_at, nodes.name
        "#,
    )
    .bind(search)
    .fetch_all(&state.pool)
    .await?;

    let nodes = rows
        .into_iter()
        .map(|row| {
            Ok(NodeResponse {
                id: row.try_get("id")?,
                node_type_id: row.try_get("node_type_id")?,
                node_type_name: row.try_get("node_type_name")?,
                parent_node_id: row.try_get("parent_node_id")?,
                parent_node_name: row.try_get("parent_node_name")?,
                name: row.try_get("name")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(nodes))
}

async fn require_node_type_for_node(pool: &sqlx::PgPool, node_id: Uuid) -> ApiResult<Uuid> {
    sqlx::query_scalar("SELECT node_type_id FROM nodes WHERE id = $1")
        .bind(node_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("node {node_id}")))
}

async fn assert_parent_allowed(
    pool: &sqlx::PgPool,
    node_id: Uuid,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
) -> ApiResult<()> {
    let Some(parent_node_id) = parent_node_id else {
        return Ok(());
    };

    if parent_node_id == node_id {
        return Err(ApiError::BadRequest("node cannot be its own parent".into()));
    }

    let parent_type_id: Uuid = sqlx::query_scalar("SELECT node_type_id FROM nodes WHERE id = $1")
        .bind(parent_node_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("parent node {parent_node_id}")))?;

    let relationship_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM node_type_relationships
            WHERE parent_node_type_id = $1 AND child_node_type_id = $2
        )
        "#,
    )
    .bind(parent_type_id)
    .bind(node_type_id)
    .fetch_one(pool)
    .await?;

    if relationship_exists {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "node parent type is not allowed for this child type".into(),
        ))
    }
}

async fn metadata_field_rows(
    pool: &sqlx::PgPool,
    node_type_id: Uuid,
) -> ApiResult<Vec<sqlx::postgres::PgRow>> {
    Ok(sqlx::query(
        r#"
        SELECT id, key, field_type::text AS field_type, required
        FROM node_metadata_field_definitions
        WHERE node_type_id = $1
        "#,
    )
    .bind(node_type_id)
    .fetch_all(pool)
    .await?)
}

fn validate_node_metadata_values(
    field_rows: &[sqlx::postgres::PgRow],
    metadata: &HashMap<String, Value>,
    require_missing: bool,
) -> ApiResult<()> {
    for row in field_rows {
        let key: String = row.try_get("key")?;
        let required: bool = row.try_get("required")?;
        let field_type = parse_field_type(&row.try_get::<String, _>("field_type")?)?;
        match metadata.get(&key) {
            Some(value) => validate_field_value(field_type, value)?,
            None if require_missing && required => {
                return Err(ApiError::BadRequest(format!(
                    "metadata field '{key}' is required"
                )));
            }
            None => {}
        }
    }

    Ok(())
}

async fn upsert_node_metadata_values(
    pool: &sqlx::PgPool,
    node_id: Uuid,
    field_rows: &[sqlx::postgres::PgRow],
    metadata: &HashMap<String, Value>,
) -> ApiResult<()> {
    for row in field_rows {
        let key: String = row.try_get("key")?;
        if let Some(value) = metadata.get(&key) {
            let field_definition_id: Uuid = row.try_get("id")?;
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
        }
    }

    Ok(())
}

pub(crate) fn parse_field_type(field_type: &str) -> ApiResult<FieldType> {
    FieldType::from_str(field_type).map_err(field_type_error)
}

pub(crate) fn validate_field_value(field_type: FieldType, value: &Value) -> ApiResult<()> {
    field_type
        .validate_json_value(value)
        .map_err(field_type_error)
}

pub(crate) fn require_text(field_name: &'static str, value: &str) -> ApiResult<()> {
    validate_required_text(field_name, value)
        .map_err(|error| ApiError::BadRequest(error.to_string()))
}

fn field_type_error(error: FieldTypeError) -> ApiError {
    ApiError::BadRequest(error.to_string())
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

async fn require_node_type_slug_available(pool: &sqlx::PgPool, slug: &str) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM node_types WHERE slug = $1)")
            .bind(slug)
            .fetch_one(pool)
            .await?;

    if exists {
        Err(ApiError::BadRequest(format!(
            "node type slug '{slug}' is already in use"
        )))
    } else {
        Ok(())
    }
}

async fn require_node_type_slug_available_for_type(
    pool: &sqlx::PgPool,
    node_type_id: Uuid,
    slug: &str,
) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM node_types WHERE slug = $1 AND id <> $2)")
            .bind(slug)
            .bind(node_type_id)
            .fetch_one(pool)
            .await?;

    if exists {
        Err(ApiError::BadRequest(format!(
            "node type slug '{slug}' is already in use"
        )))
    } else {
        Ok(())
    }
}

async fn require_node_metadata_key_available(
    pool: &sqlx::PgPool,
    node_type_id: Uuid,
    key: &str,
) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM node_metadata_field_definitions WHERE node_type_id = $1 AND key = $2)",
    )
    .bind(node_type_id)
    .bind(key)
    .fetch_one(pool)
    .await?;

    if exists {
        Err(ApiError::BadRequest(format!(
            "metadata key '{key}' is already in use for node type {node_type_id}"
        )))
    } else {
        Ok(())
    }
}

async fn require_node_type_relationship_exists(
    pool: &sqlx::PgPool,
    parent_node_type_id: Uuid,
    child_node_type_id: Uuid,
) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM node_type_relationships
            WHERE parent_node_type_id = $1 AND child_node_type_id = $2
        )
        "#,
    )
    .bind(parent_node_type_id)
    .bind(child_node_type_id)
    .fetch_one(pool)
    .await?;

    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!(
            "node type relationship {parent_node_type_id} -> {child_node_type_id}"
        )))
    }
}

async fn assert_relationship_unused(
    pool: &sqlx::PgPool,
    parent_node_type_id: Uuid,
    child_node_type_id: Uuid,
) -> ApiResult<()> {
    let usage_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM nodes AS child_nodes
        JOIN nodes AS parent_nodes ON parent_nodes.id = child_nodes.parent_node_id
        WHERE parent_nodes.node_type_id = $1 AND child_nodes.node_type_id = $2
        "#,
    )
    .bind(parent_node_type_id)
    .bind(child_node_type_id)
    .fetch_one(pool)
    .await?;

    if usage_count == 0 {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "node type relationship cannot be removed while existing nodes use it".into(),
        ))
    }
}

async fn require_node_metadata_field(
    pool: &sqlx::PgPool,
    field_id: Uuid,
) -> ApiResult<ExistingMetadataField> {
    let row = sqlx::query(
        r#"
        SELECT node_type_id, key, field_type::text AS field_type, required
        FROM node_metadata_field_definitions
        WHERE id = $1
        "#,
    )
    .bind(field_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("metadata field {field_id}")))?;

    Ok(ExistingMetadataField {
        node_type_id: row.try_get("node_type_id")?,
        key: row.try_get("key")?,
        field_type: row.try_get("field_type")?,
        required: row.try_get("required")?,
    })
}

async fn node_count_for_type(pool: &sqlx::PgPool, node_type_id: Uuid) -> ApiResult<i64> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE node_type_id = $1")
            .bind(node_type_id)
            .fetch_one(pool)
            .await?,
    )
}

async fn assert_relationship_is_acyclic(
    pool: &sqlx::PgPool,
    parent_node_type_id: Uuid,
    child_node_type_id: Uuid,
) -> ApiResult<()> {
    let existing_relationships = sqlx::query_as::<_, (Uuid, Uuid)>(
        "SELECT parent_node_type_id, child_node_type_id FROM node_type_relationships",
    )
    .fetch_all(pool)
    .await?;

    validate_node_type_relationship(
        parent_node_type_id,
        child_node_type_id,
        &existing_relationships,
    )
    .map_err(|error| ApiError::BadRequest(error.to_string()))
}
