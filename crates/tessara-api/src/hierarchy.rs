use std::{collections::HashMap, str::FromStr};

use axum::{Json, extract::State, http::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use tessara_core::{FieldType, FieldTypeError};
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
pub struct CreateNodeRequest {
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    name: String,
    #[serde(default)]
    metadata: HashMap<String, Value>,
}

#[derive(Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}

#[derive(Serialize)]
pub struct NodeResponse {
    id: Uuid,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    name: String,
}

#[derive(Serialize)]
pub struct NodeTypeSummary {
    id: Uuid,
    name: String,
    slug: String,
    node_count: i64,
}

pub async fn create_node_type(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNodeTypeRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;

    let id = sqlx::query_scalar("INSERT INTO node_types (name, slug) VALUES ($1, $2) RETURNING id")
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

pub async fn create_node_metadata_field(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNodeMetadataFieldRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    require_node_type_exists(&state.pool, payload.node_type_id).await?;
    let field_type = parse_field_type(&payload.field_type)?;

    if payload.required {
        let existing_node_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE node_type_id = $1")
                .bind(payload.node_type_id)
                .fetch_one(&state.pool)
                .await?;
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

pub async fn create_node(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNodeRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;
    require_node_type_exists(&state.pool, payload.node_type_id).await?;

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

    let field_rows = sqlx::query(
        r#"
        SELECT id, key, field_type::text AS field_type, required
        FROM node_metadata_field_definitions
        WHERE node_type_id = $1
        "#,
    )
    .bind(payload.node_type_id)
    .fetch_all(&state.pool)
    .await?;

    for row in &field_rows {
        let key: String = row.try_get("key")?;
        let required: bool = row.try_get("required")?;
        let field_type = parse_field_type(&row.try_get::<String, _>("field_type")?)?;
        match payload.metadata.get(&key) {
            Some(value) => validate_field_value(field_type, value)?,
            None if required => {
                return Err(ApiError::BadRequest(format!(
                    "metadata field '{key}' is required"
                )));
            }
            None => {}
        }
    }

    let node_id: Uuid = sqlx::query_scalar(
        "INSERT INTO nodes (node_type_id, parent_node_id, name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(payload.node_type_id)
    .bind(payload.parent_node_id)
    .bind(payload.name)
    .fetch_one(&state.pool)
    .await?;

    for row in field_rows {
        let key: String = row.try_get("key")?;
        if let Some(value) = payload.metadata.get(&key) {
            let field_definition_id: Uuid = row.try_get("id")?;
            sqlx::query(
                r#"
                INSERT INTO node_metadata_values (node_id, field_definition_id, value)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(node_id)
            .bind(field_definition_id)
            .bind(value)
            .execute(&state.pool)
            .await?;
        }
    }

    Ok(Json(IdResponse { id: node_id }))
}

pub async fn list_nodes(State(state): State<AppState>) -> ApiResult<Json<Vec<NodeResponse>>> {
    let rows = sqlx::query(
        r#"
        SELECT id, node_type_id, parent_node_id, name
        FROM nodes
        ORDER BY created_at, name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let nodes = rows
        .into_iter()
        .map(|row| {
            Ok(NodeResponse {
                id: row.try_get("id")?,
                node_type_id: row.try_get("node_type_id")?,
                parent_node_id: row.try_get("parent_node_id")?,
                name: row.try_get("name")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(nodes))
}

pub(crate) fn parse_field_type(field_type: &str) -> ApiResult<FieldType> {
    FieldType::from_str(field_type).map_err(field_type_error)
}

pub(crate) fn validate_field_value(field_type: FieldType, value: &Value) -> ApiResult<()> {
    field_type
        .validate_json_value(value)
        .map_err(field_type_error)
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

async fn assert_relationship_is_acyclic(
    pool: &sqlx::PgPool,
    parent_node_type_id: Uuid,
    child_node_type_id: Uuid,
) -> ApiResult<()> {
    if parent_node_type_id == child_node_type_id {
        return Err(ApiError::BadRequest(
            "node type relationships cannot point to the same type".into(),
        ));
    }

    let would_create_cycle: bool = sqlx::query_scalar(
        r#"
        WITH RECURSIVE descendants(node_type_id) AS (
            SELECT child_node_type_id
            FROM node_type_relationships
            WHERE parent_node_type_id = $1

            UNION

            SELECT node_type_relationships.child_node_type_id
            FROM node_type_relationships
            JOIN descendants
                ON descendants.node_type_id = node_type_relationships.parent_node_type_id
        )
        SELECT EXISTS (
            SELECT 1
            FROM descendants
            WHERE node_type_id = $2
        )
        "#,
    )
    .bind(child_node_type_id)
    .bind(parent_node_type_id)
    .fetch_one(pool)
    .await?;

    if would_create_cycle {
        Err(ApiError::BadRequest(
            "node type relationship would create a cycle".into(),
        ))
    } else {
        Ok(())
    }
}
