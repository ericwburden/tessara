use std::collections::HashMap;

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiResult;

pub(super) struct MetadataFieldDef<'a> {
    pub(super) key: &'a str,
    pub(super) label: &'a str,
    pub(super) field_type: &'a str,
    pub(super) required: bool,
}

pub(super) struct DemoNodeSpec<'a> {
    pub(super) name: &'a str,
    pub(super) metadata: Vec<(&'a str, Value)>,
}

pub(super) async fn ensure_node_type(pool: &PgPool, name: &str, slug: &str) -> ApiResult<Uuid> {
    sqlx::query_scalar(
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
    .await
    .map_err(Into::into)
}

pub(super) async fn ensure_node_type_relationship(
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

pub(super) async fn ensure_metadata_fields(
    pool: &PgPool,
    node_type_id: Uuid,
    definitions: &[MetadataFieldDef<'_>],
) -> ApiResult<HashMap<String, Uuid>> {
    let mut field_ids = HashMap::new();
    for definition in definitions {
        let id = ensure_node_metadata_field(
            pool,
            node_type_id,
            definition.key,
            definition.label,
            definition.field_type,
            definition.required,
        )
        .await?;
        field_ids.insert(definition.key.to_string(), id);
    }
    Ok(field_ids)
}

async fn ensure_node_metadata_field(
    pool: &PgPool,
    node_type_id: Uuid,
    key: &str,
    label: &str,
    field_type: &str,
    required: bool,
) -> ApiResult<Uuid> {
    sqlx::query_scalar(
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
    .await
    .map_err(Into::into)
}

pub(super) async fn ensure_demo_node(
    pool: &PgPool,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    metadata_fields: &HashMap<String, Uuid>,
    spec: DemoNodeSpec<'_>,
) -> ApiResult<Uuid> {
    let node_id = ensure_node(pool, node_type_id, parent_node_id, spec.name).await?;
    for (key, value) in spec.metadata {
        if let Some(field_definition_id) = metadata_fields.get(key) {
            upsert_metadata_value(pool, node_id, *field_definition_id, value).await?;
        }
    }
    Ok(node_id)
}

async fn ensure_node(
    pool: &PgPool,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    name: &str,
) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT id
        FROM nodes
        WHERE node_type_id = $1
          AND parent_node_id IS NOT DISTINCT FROM $2
          AND name = $3
        "#,
    )
    .bind(node_type_id)
    .bind(parent_node_id)
    .bind(name)
    .fetch_optional(pool)
    .await?
    {
        return Ok(id);
    }

    sqlx::query_scalar(
        "INSERT INTO nodes (node_type_id, parent_node_id, name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(node_type_id)
    .bind(parent_node_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

async fn upsert_metadata_value(
    pool: &PgPool,
    node_id: Uuid,
    field_definition_id: Uuid,
    value: Value,
) -> ApiResult<()> {
    if value.is_null() {
        sqlx::query(
            "DELETE FROM node_metadata_values WHERE node_id = $1 AND field_definition_id = $2",
        )
        .bind(node_id)
        .bind(field_definition_id)
        .execute(pool)
        .await?;
    } else {
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
    Ok(())
}
