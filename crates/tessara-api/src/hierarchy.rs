use std::collections::HashMap;

use axum::{Json, extract::State, http::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
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

pub async fn create_node_type_relationship(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNodeTypeRelationshipRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "hierarchy:write").await?;

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
    validate_field_type(&payload.field_type)?;

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
    .bind(payload.field_type)
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
        let field_type: String = row.try_get("field_type")?;
        match payload.metadata.get(&key) {
            Some(value) => validate_json_value(&field_type, value)?,
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

pub fn validate_field_type(field_type: &str) -> ApiResult<()> {
    match field_type {
        "text" | "number" | "boolean" | "date" | "single_choice" | "multi_choice" => Ok(()),
        other => Err(ApiError::BadRequest(format!(
            "unsupported field type '{other}'"
        ))),
    }
}

pub fn validate_json_value(field_type: &str, value: &Value) -> ApiResult<()> {
    let valid = match field_type {
        "text" | "date" | "single_choice" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "multi_choice" => value
            .as_array()
            .map(|items| items.iter().all(Value::is_string))
            .unwrap_or(false),
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "value does not match field type '{field_type}'"
        )))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{validate_field_type, validate_json_value};

    #[test]
    fn validates_supported_field_types() {
        for field_type in [
            "text",
            "number",
            "boolean",
            "date",
            "single_choice",
            "multi_choice",
        ] {
            assert!(
                validate_field_type(field_type).is_ok(),
                "{field_type} should be accepted"
            );
        }

        assert!(validate_field_type("file_upload").is_err());
    }

    #[test]
    fn validates_json_values_against_field_types() {
        assert!(validate_json_value("text", &json!("hello")).is_ok());
        assert!(validate_json_value("date", &json!("2026-04-06")).is_ok());
        assert!(validate_json_value("single_choice", &json!("yes")).is_ok());
        assert!(validate_json_value("number", &json!(42)).is_ok());
        assert!(validate_json_value("boolean", &json!(true)).is_ok());
        assert!(validate_json_value("multi_choice", &json!(["a", "b"])).is_ok());
    }

    #[test]
    fn rejects_json_values_that_do_not_match_field_types() {
        assert!(validate_json_value("text", &json!(42)).is_err());
        assert!(validate_json_value("number", &json!("42")).is_err());
        assert!(validate_json_value("boolean", &json!("true")).is_err());
        assert!(validate_json_value("multi_choice", &json!(["a", 2])).is_err());
        assert!(validate_json_value("unknown", &json!("value")).is_err());
    }
}
