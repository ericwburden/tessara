use std::{collections::HashMap, str::FromStr};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use tessara_core::{FieldType, FieldTypeError, validate_required_text};
use tessara_hierarchy::validate_node_type_relationship;
use uuid::Uuid;

use crate::{
    auth::{self, AuthenticatedRequest},
    db::AppState,
    error::{ApiError, ApiResult},
};

#[derive(Deserialize)]
pub struct CreateNodeTypeRequest {
    name: String,
    slug: String,
    plural_label: Option<String>,
    #[serde(default)]
    parent_node_type_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    child_node_type_ids: Option<Vec<Uuid>>,
}

#[derive(Deserialize)]
pub struct UpdateNodeTypeRequest {
    name: String,
    slug: String,
    plural_label: Option<String>,
    #[serde(default)]
    parent_node_type_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    child_node_type_ids: Option<Vec<Uuid>>,
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
    node_type_slug: String,
    node_type_singular_label: String,
    node_type_plural_label: String,
    parent_node_id: Option<Uuid>,
    parent_node_name: Option<String>,
    name: String,
    metadata: Value,
}

#[derive(Serialize)]
pub struct NodeDetail {
    id: Uuid,
    node_type_id: Uuid,
    node_type_name: String,
    node_type_slug: String,
    node_type_singular_label: String,
    node_type_plural_label: String,
    parent_node_id: Option<Uuid>,
    parent_node_name: Option<String>,
    name: String,
    metadata: Value,
    related_forms: Vec<NodeFormLink>,
    related_responses: Vec<NodeSubmissionLink>,
    related_dashboards: Vec<NodeDashboardLink>,
}

#[derive(Serialize)]
pub struct NodeFormLink {
    form_id: Uuid,
    form_name: String,
    form_slug: String,
    published_version_count: i64,
}

#[derive(Serialize)]
pub struct NodeSubmissionLink {
    submission_id: Uuid,
    form_id: Uuid,
    form_name: String,
    form_version_id: Uuid,
    version_label: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    submitted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct NodeDashboardLink {
    dashboard_id: Uuid,
    dashboard_name: String,
    component_count: i64,
}

#[derive(Serialize)]
pub struct NodeTypeSummary {
    id: Uuid,
    name: String,
    slug: String,
    singular_label: String,
    plural_label: String,
    is_root_type: bool,
    node_count: i64,
}

#[derive(Serialize, Clone)]
pub struct NodeTypePeerLink {
    node_type_id: Uuid,
    node_type_name: String,
    node_type_slug: String,
    singular_label: String,
    plural_label: String,
}

#[derive(Serialize)]
pub struct NodeTypeCatalogEntry {
    id: Uuid,
    name: String,
    slug: String,
    singular_label: String,
    plural_label: String,
    is_root_type: bool,
    node_count: i64,
    parent_relationships: Vec<NodeTypePeerLink>,
    child_relationships: Vec<NodeTypePeerLink>,
}

#[derive(Serialize)]
pub struct NodeTypeDefinition {
    id: Uuid,
    name: String,
    slug: String,
    singular_label: String,
    plural_label: String,
    is_root_type: bool,
    node_count: i64,
    parent_relationships: Vec<NodeTypePeerLink>,
    child_relationships: Vec<NodeTypePeerLink>,
    metadata_fields: Vec<NodeMetadataFieldSummary>,
    scoped_forms: Vec<NodeTypeFormLink>,
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

#[derive(Serialize)]
pub struct NodeTypeFormLink {
    form_id: Uuid,
    form_name: String,
    form_slug: String,
}

pub async fn create_node_type(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Json(payload): Json<CreateNodeTypeRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
    require_text("node type name", &payload.name)?;
    require_text("node type slug", &payload.slug)?;
    require_node_type_slug_available(&state.pool, &payload.slug).await?;
    let node_type_id = Uuid::new_v4();
    let relationship_selection = resolve_node_type_relationship_selection(
        &state.pool,
        node_type_id,
        payload.parent_node_type_ids.as_deref(),
        payload.child_node_type_ids.as_deref(),
    )
    .await?;
    validate_node_type_relationship_selection(&state.pool, node_type_id, &relationship_selection)
        .await?;

    let mut transaction = state.pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO node_types (
            id,
            name,
            slug,
            plural_label
        ) VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(node_type_id)
    .bind(payload.name)
    .bind(payload.slug)
    .bind(payload.plural_label)
    .execute(&mut *transaction)
    .await?;

    sync_node_type_relationships(&mut transaction, node_type_id, &relationship_selection).await?;
    transaction.commit().await?;

    Ok(Json(IdResponse { id: node_type_id }))
}

/// Updates node-type display metadata used by hierarchy and form-builder screens.
pub async fn update_node_type(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(node_type_id): Path<Uuid>,
    Json(payload): Json<UpdateNodeTypeRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
    require_text("node type name", &payload.name)?;
    require_text("node type slug", &payload.slug)?;
    require_node_type_exists(&state.pool, node_type_id).await?;
    require_node_type_slug_available_for_type(&state.pool, node_type_id, &payload.slug).await?;
    let relationship_selection = resolve_node_type_relationship_selection(
        &state.pool,
        node_type_id,
        payload.parent_node_type_ids.as_deref(),
        payload.child_node_type_ids.as_deref(),
    )
    .await?;
    validate_node_type_relationship_selection(&state.pool, node_type_id, &relationship_selection)
        .await?;
    assert_removed_relationships_unused(&state.pool, node_type_id, &relationship_selection).await?;

    let mut transaction = state.pool.begin().await?;
    let id = sqlx::query_scalar(
        r#"
        UPDATE node_types
        SET name = $2,
            slug = $3,
            plural_label = $4
        WHERE id = $1
        RETURNING id
        "#,
    )
    .bind(node_type_id)
    .bind(payload.name)
    .bind(payload.slug)
    .bind(payload.plural_label)
    .fetch_one(&mut *transaction)
    .await?;

    sync_node_type_relationships(&mut transaction, node_type_id, &relationship_selection).await?;
    transaction.commit().await?;

    Ok(Json(IdResponse { id }))
}

/// Lists configured hierarchy node types for the admin builder shell.
pub async fn list_node_types(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<Vec<NodeTypeSummary>>> {
    request.require_capability("admin:all")?;

    let node_types = load_node_type_catalog(&state.pool)
        .await?
        .into_iter()
        .map(|entry| NodeTypeSummary {
            id: entry.id,
            name: entry.name,
            slug: entry.slug,
            singular_label: entry.singular_label,
            plural_label: entry.plural_label,
            is_root_type: entry.is_root_type,
            node_count: entry.node_count,
        })
        .collect();

    Ok(Json(node_types))
}

pub async fn list_readable_node_types(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<Vec<NodeTypeCatalogEntry>>> {
    request.require_capability("hierarchy:read")?;
    Ok(Json(load_node_type_catalog(&state.pool).await?))
}

/// Returns one node type with linked relationships, metadata fields, and scoped forms.
pub async fn get_node_type(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(node_type_id): Path<Uuid>,
) -> ApiResult<Json<NodeTypeDefinition>> {
    request.require_capability("admin:all")?;
    let catalog_entry = load_node_type_catalog_entry(&state.pool, node_type_id).await?;

    let metadata_fields = sqlx::query(
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
        WHERE node_metadata_field_definitions.node_type_id = $1
        ORDER BY node_metadata_field_definitions.key
        "#,
    )
    .bind(node_type_id)
    .fetch_all(&state.pool)
    .await?
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

    let scoped_forms = sqlx::query(
        r#"
        SELECT id AS form_id, name AS form_name, slug AS form_slug
        FROM forms
        WHERE scope_node_type_id = $1
        ORDER BY name, id
        "#,
    )
    .bind(node_type_id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(NodeTypeFormLink {
            form_id: row.try_get("form_id")?,
            form_name: row.try_get("form_name")?,
            form_slug: row.try_get("form_slug")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(NodeTypeDefinition {
        id: catalog_entry.id,
        name: catalog_entry.name,
        slug: catalog_entry.slug,
        singular_label: catalog_entry.singular_label,
        plural_label: catalog_entry.plural_label,
        is_root_type: catalog_entry.is_root_type,
        node_count: catalog_entry.node_count,
        parent_relationships: catalog_entry.parent_relationships,
        child_relationships: catalog_entry.child_relationships,
        metadata_fields,
        scoped_forms,
    }))
}

/// Lists configured parent/child hierarchy relationships for admin screens.
pub async fn list_node_type_relationships(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<Vec<NodeTypeRelationshipSummary>>> {
    request.require_capability("admin:all")?;

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
    request: AuthenticatedRequest,
    Json(payload): Json<CreateNodeTypeRelationshipRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
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
    request: AuthenticatedRequest,
    Path((parent_node_type_id, child_node_type_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
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
    request: AuthenticatedRequest,
) -> ApiResult<Json<Vec<NodeMetadataFieldSummary>>> {
    request.require_capability("admin:all")?;

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
    request: AuthenticatedRequest,
    Json(payload): Json<CreateNodeMetadataFieldRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
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

struct NodeTypeRelationshipSelection {
    parent_node_type_ids: Vec<Uuid>,
    child_node_type_ids: Vec<Uuid>,
}

/// Updates metadata field display and safe schema settings for a node type.
pub async fn update_node_metadata_field(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(field_id): Path<Uuid>,
    Json(payload): Json<UpdateNodeMetadataFieldRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
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

/// Deletes a node metadata field definition and any collected values for that field.
pub async fn delete_node_metadata_field(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(field_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
    require_node_metadata_field(&state.pool, field_id).await?;

    sqlx::query(
        r#"
        DELETE FROM node_metadata_field_definitions
        WHERE id = $1
        "#,
    )
    .bind(field_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(IdResponse { id: field_id }))
}

pub async fn create_node(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Json(payload): Json<CreateNodeRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("hierarchy:write")?;
    require_node_type_exists(&state.pool, payload.node_type_id).await?;
    require_text("node name", &payload.name)?;

    assert_parent_allowed(
        &state.pool,
        None,
        payload.node_type_id,
        payload.parent_node_id,
    )
    .await?;

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
    request: AuthenticatedRequest,
    Path(node_id): Path<Uuid>,
    Json(payload): Json<UpdateNodeRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("hierarchy:write")?;
    require_text("node name", &payload.name)?;
    let node_type_id = require_node_type_for_node(&state.pool, node_id).await?;
    assert_parent_allowed(
        &state.pool,
        Some(node_id),
        node_type_id,
        payload.parent_node_id,
    )
    .await?;

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
    request: AuthenticatedRequest,
    Query(query): Query<ListNodesQuery>,
) -> ApiResult<Json<Vec<NodeResponse>>> {
    let account = request.require_capability("hierarchy:read")?;
    let search = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let rows = if account.is_operator() {
        let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
        sqlx::query(
            r#"
            SELECT
                nodes.id,
                nodes.node_type_id,
                node_types.name AS node_type_name,
                node_types.slug AS node_type_slug,
                node_types.plural_label,
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
            WHERE nodes.id = ANY($1)
              AND (
                $2::text IS NULL
                OR nodes.name ILIKE '%' || $2 || '%'
                OR node_types.name ILIKE '%' || $2 || '%'
              )
            GROUP BY
                nodes.id,
                nodes.node_type_id,
                node_types.name,
                node_types.slug,
                node_types.plural_label,
                nodes.parent_node_id,
                parent_nodes.name,
                nodes.name,
                nodes.created_at
            ORDER BY nodes.created_at, nodes.name
            "#,
        )
        .bind(scope_ids)
        .bind(search)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                nodes.id,
                nodes.node_type_id,
                node_types.name AS node_type_name,
                node_types.slug AS node_type_slug,
                node_types.plural_label,
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
                node_types.slug,
                node_types.plural_label,
                nodes.parent_node_id,
                parent_nodes.name,
                nodes.name,
                nodes.created_at
            ORDER BY nodes.created_at, nodes.name
            "#,
        )
        .bind(search)
        .fetch_all(&state.pool)
        .await?
    };

    let nodes = rows
        .into_iter()
        .map(|row| {
            let node_type_name: String = row.try_get("node_type_name")?;
            let raw_plural_label: Option<String> = row.try_get("plural_label")?;
            Ok(NodeResponse {
                id: row.try_get("id")?,
                node_type_id: row.try_get("node_type_id")?,
                node_type_name: node_type_name.clone(),
                node_type_slug: row.try_get("node_type_slug")?,
                node_type_singular_label: derive_node_type_label(&node_type_name),
                node_type_plural_label: derive_node_type_plural_label(
                    &node_type_name,
                    raw_plural_label,
                ),
                parent_node_id: row.try_get("parent_node_id")?,
                parent_node_name: row.try_get("parent_node_name")?,
                name: row.try_get("name")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(nodes))
}

pub async fn get_node(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(node_id): Path<Uuid>,
) -> ApiResult<Json<NodeDetail>> {
    let account = request.require_capability("hierarchy:read")?;
    if account.is_operator() {
        let scope_ids = auth::effective_scope_node_ids(&state.pool, account.account_id).await?;
        if !scope_ids.contains(&node_id) {
            return Err(ApiError::Forbidden("hierarchy:read".into()));
        }
    }

    let node = sqlx::query(
        r#"
        SELECT
            nodes.id,
            nodes.node_type_id,
            node_types.name AS node_type_name,
            node_types.slug AS node_type_slug,
            node_types.plural_label,
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
        WHERE nodes.id = $1
        GROUP BY
            nodes.id,
            nodes.node_type_id,
            node_types.name,
            node_types.slug,
            node_types.plural_label,
            nodes.parent_node_id,
            parent_nodes.name,
            nodes.name
        "#,
    )
    .bind(node_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("node {node_id}")))?;

    let related_forms = sqlx::query(
        r#"
        SELECT
            forms.id AS form_id,
            forms.name AS form_name,
            forms.slug AS form_slug,
            COUNT(DISTINCT form_versions.id)
                FILTER (WHERE form_versions.status = 'published') AS published_version_count
        FROM forms
        JOIN form_versions ON form_versions.form_id = forms.id
        JOIN form_assignments ON form_assignments.form_version_id = form_versions.id
        WHERE form_assignments.node_id = $1
        GROUP BY forms.id, forms.name, forms.slug
        ORDER BY forms.name, forms.id
        "#,
    )
    .bind(node_id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(NodeFormLink {
            form_id: row.try_get("form_id")?,
            form_name: row.try_get("form_name")?,
            form_slug: row.try_get("form_slug")?,
            published_version_count: row.try_get("published_version_count")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let related_responses = sqlx::query(
        r#"
        SELECT
            submissions.id AS submission_id,
            forms.id AS form_id,
            forms.name AS form_name,
            submissions.form_version_id,
            form_versions.version_label,
            submissions.status::text AS status,
            submissions.created_at,
            submissions.submitted_at
        FROM submissions
        JOIN form_versions ON form_versions.id = submissions.form_version_id
        JOIN forms ON forms.id = form_versions.form_id
        WHERE submissions.node_id = $1
        ORDER BY submissions.created_at DESC, submissions.id DESC
        LIMIT 10
        "#,
    )
    .bind(node_id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(NodeSubmissionLink {
            submission_id: row.try_get("submission_id")?,
            form_id: row.try_get("form_id")?,
            form_name: row.try_get("form_name")?,
            form_version_id: row.try_get("form_version_id")?,
            version_label: row.try_get("version_label")?,
            status: row.try_get("status")?,
            created_at: row.try_get("created_at")?,
            submitted_at: row.try_get("submitted_at")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let related_dashboards = sqlx::query(
        r#"
        SELECT
            dashboards.id AS dashboard_id,
            dashboards.name AS dashboard_name,
            COUNT(all_components.id) AS component_count
        FROM dashboards
        LEFT JOIN dashboard_components AS all_components
            ON all_components.dashboard_id = dashboards.id
        WHERE EXISTS (
            SELECT 1
            FROM dashboard_components
            JOIN charts ON charts.id = dashboard_components.chart_id
            LEFT JOIN reports ON reports.id = charts.report_id
            LEFT JOIN aggregations ON aggregations.id = charts.aggregation_id
            LEFT JOIN reports AS aggregation_reports
                ON aggregation_reports.id = aggregations.report_id
            LEFT JOIN forms AS direct_forms ON direct_forms.id = reports.form_id
            LEFT JOIN form_versions AS direct_form_versions
                ON direct_form_versions.form_id = direct_forms.id
               AND direct_form_versions.status = 'published'::form_version_status
            LEFT JOIN forms AS aggregation_forms
                ON aggregation_forms.id = aggregation_reports.form_id
            LEFT JOIN form_versions AS aggregation_form_versions
                ON aggregation_form_versions.form_id = aggregation_forms.id
               AND aggregation_form_versions.status = 'published'::form_version_status
            LEFT JOIN form_assignments AS direct_assignments
                ON direct_assignments.form_version_id = direct_form_versions.id
            LEFT JOIN form_assignments AS aggregation_assignments
                ON aggregation_assignments.form_version_id = aggregation_form_versions.id
            WHERE dashboard_components.dashboard_id = dashboards.id
              AND (
                  direct_assignments.node_id = $1
                  OR aggregation_assignments.node_id = $1
              )
        )
        GROUP BY dashboards.id, dashboards.name
        ORDER BY dashboards.name, dashboards.id
        "#,
    )
    .bind(node_id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(NodeDashboardLink {
            dashboard_id: row.try_get("dashboard_id")?,
            dashboard_name: row.try_get("dashboard_name")?,
            component_count: row.try_get("component_count")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(NodeDetail {
        id: node.try_get("id")?,
        node_type_id: node.try_get("node_type_id")?,
        node_type_name: node.try_get("node_type_name")?,
        node_type_slug: node.try_get("node_type_slug")?,
        node_type_singular_label: derive_node_type_label(
            &node.try_get::<String, _>("node_type_name")?,
        ),
        node_type_plural_label: derive_node_type_plural_label(
            &node.try_get::<String, _>("node_type_name")?,
            node.try_get::<Option<String>, _>("plural_label")?,
        ),
        parent_node_id: node.try_get("parent_node_id")?,
        parent_node_name: node.try_get("parent_node_name")?,
        name: node.try_get("name")?,
        metadata: node.try_get("metadata")?,
        related_forms,
        related_responses,
        related_dashboards,
    }))
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
    node_id: Option<Uuid>,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
) -> ApiResult<()> {
    let allowed_parent_type_ids = allowed_parent_type_ids(pool, node_type_id).await?;

    let Some(parent_node_id) = parent_node_id else {
        return if allowed_parent_type_ids.is_empty() {
            Ok(())
        } else {
            Err(ApiError::BadRequest(
                "node parent is required for this child type".into(),
            ))
        };
    };

    if Some(parent_node_id) == node_id {
        return Err(ApiError::BadRequest("node cannot be its own parent".into()));
    }

    let parent_type_id: Uuid = sqlx::query_scalar("SELECT node_type_id FROM nodes WHERE id = $1")
        .bind(parent_node_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("parent node {parent_node_id}")))?;

    if allowed_parent_type_ids.contains(&parent_type_id) {
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

async fn load_node_type_catalog(pool: &sqlx::PgPool) -> ApiResult<Vec<NodeTypeCatalogEntry>> {
    let rows = sqlx::query(
        r#"
        SELECT
            node_types.id,
            node_types.name,
            node_types.slug,
            node_types.plural_label,
            COUNT(nodes.id) AS node_count
        FROM node_types
        LEFT JOIN nodes ON nodes.node_type_id = node_types.id
        GROUP BY
            node_types.id,
            node_types.name,
            node_types.slug,
            node_types.created_at,
            node_types.plural_label
        ORDER BY node_types.created_at, node_types.name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let relationships = sqlx::query(
        r#"
        SELECT
            parent_node_types.id AS parent_node_type_id,
            parent_node_types.name AS parent_node_type_name,
            parent_node_types.slug AS parent_node_type_slug,
            parent_node_types.plural_label AS parent_plural_label,
            child_node_types.id AS child_node_type_id,
            child_node_types.name AS child_node_type_name,
            child_node_types.slug AS child_node_type_slug,
            child_node_types.plural_label AS child_plural_label
        FROM node_type_relationships
        JOIN node_types AS parent_node_types
            ON parent_node_types.id = node_type_relationships.parent_node_type_id
        JOIN node_types AS child_node_types
            ON child_node_types.id = node_type_relationships.child_node_type_id
        ORDER BY
            parent_node_types.name,
            parent_node_types.id,
            child_node_types.name,
            child_node_types.id
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut parent_relationships: HashMap<Uuid, Vec<NodeTypePeerLink>> = HashMap::new();
    let mut child_relationships: HashMap<Uuid, Vec<NodeTypePeerLink>> = HashMap::new();
    for row in relationships {
        let parent_name: String = row.try_get("parent_node_type_name")?;
        let child_name: String = row.try_get("child_node_type_name")?;
        let parent_id: Uuid = row.try_get("parent_node_type_id")?;
        let child_id: Uuid = row.try_get("child_node_type_id")?;
        let parent_link = NodeTypePeerLink {
            node_type_id: parent_id,
            node_type_name: parent_name.clone(),
            node_type_slug: row.try_get("parent_node_type_slug")?,
            singular_label: derive_node_type_label(&parent_name),
            plural_label: derive_node_type_plural_label(
                &parent_name,
                row.try_get("parent_plural_label")?,
            ),
        };
        let child_link = NodeTypePeerLink {
            node_type_id: child_id,
            node_type_name: child_name.clone(),
            node_type_slug: row.try_get("child_node_type_slug")?,
            singular_label: derive_node_type_label(&child_name),
            plural_label: derive_node_type_plural_label(
                &child_name,
                row.try_get("child_plural_label")?,
            ),
        };
        parent_relationships
            .entry(child_id)
            .or_default()
            .push(parent_link);
        child_relationships
            .entry(parent_id)
            .or_default()
            .push(child_link);
    }

    let catalog = rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.try_get("id")?;
            let name: String = row.try_get("name")?;
            let parent_links = parent_relationships.remove(&id).unwrap_or_default();
            let child_links = child_relationships.remove(&id).unwrap_or_default();
            let singular_label = derive_node_type_label(&name);
            let plural_label = derive_node_type_plural_label(
                &name,
                row.try_get::<Option<String>, _>("plural_label")?,
            );
            Ok(NodeTypeCatalogEntry {
                id,
                name,
                slug: row.try_get("slug")?,
                singular_label,
                plural_label,
                is_root_type: parent_links.is_empty(),
                node_count: row.try_get("node_count")?,
                parent_relationships: parent_links,
                child_relationships: child_links,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(catalog)
}

async fn load_node_type_catalog_entry(
    pool: &sqlx::PgPool,
    node_type_id: Uuid,
) -> ApiResult<NodeTypeCatalogEntry> {
    load_node_type_catalog(pool)
        .await?
        .into_iter()
        .find(|entry| entry.id == node_type_id)
        .ok_or_else(|| ApiError::NotFound(format!("node type {node_type_id}")))
}

async fn allowed_parent_type_ids(pool: &sqlx::PgPool, node_type_id: Uuid) -> ApiResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar(
        r#"
        SELECT parent_node_type_id
        FROM node_type_relationships
        WHERE child_node_type_id = $1
        ORDER BY parent_node_type_id
        "#,
    )
    .bind(node_type_id)
    .fetch_all(pool)
    .await?)
}

fn derive_node_type_label(node_type_name: &str) -> String {
    node_type_name.to_string()
}

fn derive_node_type_plural_label(node_type_name: &str, provided: Option<String>) -> String {
    if let Some(label) = provided
        && !label.trim().is_empty()
    {
        return label;
    }
    infer_plural_node_type_label(node_type_name)
}

fn infer_plural_node_type_label(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "Organizations".to_string();
    }
    if trimmed.ends_with('s') {
        return format!("{trimmed}es");
    }
    if trimmed.ends_with('y') {
        if trimmed.len() >= 2 {
            let chars: Vec<char> = trimmed.chars().collect();
            let stem: String = chars[..chars.len() - 1].iter().collect();
            return format!("{stem}ies");
        }
    }
    if trimmed.ends_with("fe") {
        let stem = &trimmed[..trimmed.len() - 2];
        return format!("{stem}ves");
    }
    if trimmed.ends_with("f") {
        let stem = &trimmed[..trimmed.len() - 1];
        return format!("{stem}ves");
    }
    format!("{trimmed}s")
}

fn normalize_uuid_items(mut value: Vec<Uuid>) -> Vec<Uuid> {
    value.sort_unstable();
    value.dedup();
    value
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

async fn current_node_type_relationship_selection(
    pool: &sqlx::PgPool,
    node_type_id: Uuid,
) -> ApiResult<NodeTypeRelationshipSelection> {
    let parent_node_type_ids = sqlx::query_scalar(
        r#"
        SELECT parent_node_type_id
        FROM node_type_relationships
        WHERE child_node_type_id = $1
        ORDER BY parent_node_type_id
        "#,
    )
    .bind(node_type_id)
    .fetch_all(pool)
    .await?;

    let child_node_type_ids = sqlx::query_scalar(
        r#"
        SELECT child_node_type_id
        FROM node_type_relationships
        WHERE parent_node_type_id = $1
        ORDER BY child_node_type_id
        "#,
    )
    .bind(node_type_id)
    .fetch_all(pool)
    .await?;

    Ok(NodeTypeRelationshipSelection {
        parent_node_type_ids,
        child_node_type_ids,
    })
}

async fn resolve_node_type_relationship_selection(
    pool: &sqlx::PgPool,
    node_type_id: Uuid,
    parent_node_type_ids: Option<&[Uuid]>,
    child_node_type_ids: Option<&[Uuid]>,
) -> ApiResult<NodeTypeRelationshipSelection> {
    let current_selection = current_node_type_relationship_selection(pool, node_type_id).await?;

    Ok(NodeTypeRelationshipSelection {
        parent_node_type_ids: parent_node_type_ids
            .map(|value| normalize_uuid_items(value.to_vec()))
            .unwrap_or(current_selection.parent_node_type_ids),
        child_node_type_ids: child_node_type_ids
            .map(|value| normalize_uuid_items(value.to_vec()))
            .unwrap_or(current_selection.child_node_type_ids),
    })
}

fn planned_relationships_for_node(
    node_type_id: Uuid,
    selection: &NodeTypeRelationshipSelection,
) -> Vec<(Uuid, Uuid)> {
    let mut planned_relationships = selection
        .parent_node_type_ids
        .iter()
        .map(|parent_node_type_id| (*parent_node_type_id, node_type_id))
        .collect::<Vec<_>>();
    planned_relationships.extend(
        selection
            .child_node_type_ids
            .iter()
            .map(|child_node_type_id| (node_type_id, *child_node_type_id)),
    );
    planned_relationships
}

async fn validate_node_type_relationship_selection(
    pool: &sqlx::PgPool,
    node_type_id: Uuid,
    selection: &NodeTypeRelationshipSelection,
) -> ApiResult<()> {
    for parent_node_type_id in &selection.parent_node_type_ids {
        if *parent_node_type_id == node_type_id {
            return Err(ApiError::BadRequest(
                "node type cannot be its own parent".into(),
            ));
        }
        require_node_type_exists(pool, *parent_node_type_id).await?;
    }

    for child_node_type_id in &selection.child_node_type_ids {
        if *child_node_type_id == node_type_id {
            return Err(ApiError::BadRequest(
                "node type cannot be its own child".into(),
            ));
        }
        require_node_type_exists(pool, *child_node_type_id).await?;
    }

    if selection
        .parent_node_type_ids
        .iter()
        .any(|parent_node_type_id| selection.child_node_type_ids.contains(parent_node_type_id))
    {
        return Err(ApiError::BadRequest(
            "node type cannot be both a parent and child of the same node type".into(),
        ));
    }

    let existing_relationships = sqlx::query_as::<_, (Uuid, Uuid)>(
        "SELECT parent_node_type_id, child_node_type_id FROM node_type_relationships",
    )
    .fetch_all(pool)
    .await?;

    let mut planned_relationships = existing_relationships
        .into_iter()
        .filter(|(parent_node_type_id, child_node_type_id)| {
            *parent_node_type_id != node_type_id && *child_node_type_id != node_type_id
        })
        .collect::<Vec<_>>();

    for (parent_node_type_id, child_node_type_id) in
        planned_relationships_for_node(node_type_id, selection)
    {
        validate_node_type_relationship(
            parent_node_type_id,
            child_node_type_id,
            &planned_relationships,
        )
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        planned_relationships.push((parent_node_type_id, child_node_type_id));
    }

    Ok(())
}

async fn assert_removed_relationships_unused(
    pool: &sqlx::PgPool,
    node_type_id: Uuid,
    selection: &NodeTypeRelationshipSelection,
) -> ApiResult<()> {
    let current_selection = current_node_type_relationship_selection(pool, node_type_id).await?;

    for parent_node_type_id in current_selection.parent_node_type_ids {
        if !selection
            .parent_node_type_ids
            .contains(&parent_node_type_id)
        {
            assert_relationship_unused(pool, parent_node_type_id, node_type_id).await?;
        }
    }

    for child_node_type_id in current_selection.child_node_type_ids {
        if !selection.child_node_type_ids.contains(&child_node_type_id) {
            assert_relationship_unused(pool, node_type_id, child_node_type_id).await?;
        }
    }

    Ok(())
}

async fn sync_node_type_relationships(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    node_type_id: Uuid,
    selection: &NodeTypeRelationshipSelection,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        DELETE FROM node_type_relationships
        WHERE parent_node_type_id = $1 OR child_node_type_id = $1
        "#,
    )
    .bind(node_type_id)
    .execute(&mut **transaction)
    .await?;

    for (parent_node_type_id, child_node_type_id) in
        planned_relationships_for_node(node_type_id, selection)
    {
        sqlx::query(
            r#"
            INSERT INTO node_type_relationships (parent_node_type_id, child_node_type_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(parent_node_type_id)
        .bind(child_node_type_id)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
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
