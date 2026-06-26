use serde_json::{Value, json};
use sqlx::PgPool;
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::error::ApiResult;

use super::forms::current_form_version;

pub(super) struct DatasetFieldBinding<'a> {
    pub(super) label: &'a str,
    pub(super) source_field_key: &'a str,
    pub(super) field_type: &'a str,
}

struct ResolvedDatasetFieldBinding<'a> {
    key: String,
    label: &'a str,
    source_field_key: &'a str,
    field_type: &'a str,
    source_field_id: Option<Uuid>,
}

pub(super) async fn ensure_dataset(
    pool: &PgPool,
    form_id: Uuid,
    name: &str,
    slug: &str,
    source_alias: &str,
    visibility_node_ids: &[Uuid],
    bindings: &[DatasetFieldBinding<'_>],
) -> ApiResult<(Uuid, Uuid)> {
    let form_version_id = current_form_version(pool, form_id).await?.ok_or_else(|| {
        crate::error::ApiError::BadRequest(format!(
            "demo dataset '{slug}' requires a published form version"
        ))
    })?;
    let dataset_id = if let Some(id) = sqlx::query_scalar("SELECT id FROM datasets WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?
    {
        sqlx::query("UPDATE datasets SET name = $1, grain = 'submission' WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(pool)
            .await?;
        id
    } else {
        sqlx::query_scalar(
            "INSERT INTO datasets (name, slug, grain) VALUES ($1, $2, 'submission') RETURNING id",
        )
        .bind(name)
        .bind(slug)
        .fetch_one(pool)
        .await?
    };

    sqlx::query("DELETE FROM dataset_sources WHERE dataset_id = $1")
        .bind(dataset_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM dataset_fields WHERE dataset_id = $1")
        .bind(dataset_id)
        .execute(pool)
        .await?;
    replace_dataset_scope_nodes(pool, dataset_id, visibility_node_ids).await?;

    sqlx::query(
        r#"
        INSERT INTO dataset_sources
            (dataset_id, source_alias, form_id, form_version_id, position)
        VALUES ($1, $2, $3, $4, 0)
        "#,
    )
    .bind(dataset_id)
    .bind(source_alias)
    .bind(form_id)
    .bind(form_version_id)
    .execute(pool)
    .await?;

    let mut resolved_bindings = Vec::new();
    for binding in bindings {
        resolved_bindings.push(ResolvedDatasetFieldBinding {
            key: canonical_dataset_field_key(source_alias, binding.source_field_key),
            label: binding.label,
            source_field_key: binding.source_field_key,
            field_type: binding.field_type,
            source_field_id: source_field_id(pool, form_version_id, binding.source_field_key)
                .await?,
        });
    }

    for (position, resolved_binding) in resolved_bindings.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO dataset_fields
                (dataset_id, key, label, source_alias, source_field_key, source_field_id, field_type, position)
            VALUES ($1, $2, $3, $4, $5, $6, $7::field_type, $8)
            "#,
        )
        .bind(dataset_id)
        .bind(&resolved_binding.key)
        .bind(resolved_binding.label)
        .bind(source_alias)
        .bind(resolved_binding.source_field_key)
        .bind(resolved_binding.source_field_id)
        .bind(resolved_binding.field_type)
        .bind(position as i32)
        .execute(pool)
        .await?;
    }

    let version_number: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_number), 0) + 1 FROM dataset_revisions WHERE dataset_id = $1",
    )
    .bind(dataset_id)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        UPDATE dataset_revisions
        SET status = 'superseded'::dataset_revision_status
        WHERE dataset_id = $1
          AND status = 'published'::dataset_revision_status
        "#,
    )
    .bind(dataset_id)
    .execute(pool)
    .await?;
    let initial_source = json!({
        "kind": "form",
        "alias": source_alias,
        "form_id": form_id,
        "form_version_id": form_version_id
    });
    let operations = json!([{
        "kind": "projection",
        "position": 0,
        "fields": resolved_bindings.iter().enumerate().map(|(position, binding)| {
            json!({
                "key": binding.key,
                "label": binding.label,
                "input_field_key": binding.key,
                "position": position as i32
            })
        }).collect::<Vec<_>>()
    }]);
    let generated_sql = generated_dataset_sql(form_version_id, &resolved_bindings);
    let revision_id = sqlx::query_scalar(
        r#"
        INSERT INTO dataset_revisions
            (dataset_id, version_number, version_label, status, published_at, initial_source, operations, generated_sql)
        VALUES ($1, $2, $3, 'published'::dataset_revision_status, now(), $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(dataset_id)
    .bind(version_number)
    .bind(version_number.to_string())
    .bind(initial_source)
    .bind(operations)
    .bind(&generated_sql)
    .fetch_one(pool)
    .await?;
    materialize_dataset_revision(pool, revision_id, &generated_sql).await?;

    Ok((dataset_id, revision_id))
}

fn generated_dataset_sql(
    form_version_id: Uuid,
    bindings: &[ResolvedDatasetFieldBinding<'_>],
) -> String {
    let value_field_ids = bindings
        .iter()
        .filter_map(|binding| binding.source_field_id)
        .collect::<Vec<_>>();
    let needs_node_dim = bindings
        .iter()
        .any(|binding| binding.source_field_key == "__node_name");
    let mut group_by_expressions = BTreeSet::from([
        "submission_fact.form_version_id".to_string(),
        "submission_fact.submission_id".to_string(),
    ]);
    let select_columns = bindings
        .iter()
        .map(|binding| {
            let column = quote_identifier(&binding.key);
            if let Some(expression) = system_source_field_expression(binding.source_field_key) {
                group_by_expressions.insert(expression.to_string());
                format!("{expression} AS {column}")
            } else {
                let source_field_id = binding
                    .source_field_id
                    .expect("demo dataset field should resolve to a stable field id");
                format!(
                    "MAX(submission_value_fact.value_text) FILTER (WHERE submission_value_fact.field_id = {}::uuid) AS {column}",
                    sql_literal(&source_field_id.to_string())
                )
            }
        })
        .collect::<Vec<_>>()
        .join(",\n                ");
    let value_join = if value_field_ids.is_empty() {
        String::new()
    } else {
        let field_id_filter = value_field_ids
            .iter()
            .map(|field_id| format!("{}::uuid", sql_literal(&field_id.to_string())))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"
            LEFT JOIN analytics.submission_value_fact
              ON submission_value_fact.submission_id = submission_fact.submission_id
             AND submission_value_fact.form_version_id = submission_fact.form_version_id
             AND submission_value_fact.field_id IN ({field_id_filter})"#
        )
    };
    let group_by = if value_field_ids.is_empty() {
        String::new()
    } else {
        format!(
            r#"
            GROUP BY {}"#,
            group_by_expressions
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let node_join = if needs_node_dim {
        "\n            JOIN analytics.node_dim ON node_dim.node_id = submission_fact.node_id"
    } else {
        ""
    };

    format!(
        r#"WITH
        form_a_1 AS (
            SELECT
                md5(concat_ws('|', 'form', submission_fact.form_version_id::text, submission_fact.submission_id::text)) AS __row_id,
                {select_columns}
            FROM analytics.submission_fact{node_join}{value_join}
            WHERE submission_fact.status = 'submitted'
              AND submission_fact.form_version_id = {}::uuid{group_by}
        )
        SELECT
            __row_id,
            'public'::text AS "__restriction_tier",
            {}
        FROM form_a_1"#,
        sql_literal(&form_version_id.to_string()),
        bindings
            .iter()
            .map(|binding| quote_identifier(&binding.key))
            .collect::<Vec<_>>()
            .join(",\n            ")
    )
}

fn canonical_dataset_field_key(source_alias: &str, source_field_key: &str) -> String {
    format!(
        "{source_alias}__{}",
        source_field_key.trim_start_matches('_')
    )
}

async fn source_field_id(
    pool: &PgPool,
    form_version_id: Uuid,
    source_field_key: &str,
) -> ApiResult<Option<Uuid>> {
    if system_source_field_expression(source_field_key).is_some() {
        return Ok(None);
    }
    let field_id = sqlx::query_scalar(
        r#"
        SELECT form_fields.field_id
        FROM form_fields
        WHERE form_fields.form_version_id = $1
          AND form_fields.key = $2
        ORDER BY form_fields.position
        LIMIT 1
        "#,
    )
    .bind(form_version_id)
    .bind(source_field_key)
    .fetch_one(pool)
    .await?;
    Ok(Some(field_id))
}

fn system_source_field_expression(source_field_key: &str) -> Option<&'static str> {
    match source_field_key {
        "__submission_id" => Some("submission_fact.submission_id::text"),
        "__form_version_id" => Some("submission_fact.form_version_id::text"),
        "__node_id" => Some("submission_fact.node_id::text"),
        "__node_name" => Some("node_dim.node_name"),
        "__submission_status" => Some("submission_fact.status"),
        "__submitted_at" => Some("submission_fact.submitted_at::text"),
        "__submission_created_at" => Some("submission_fact.created_at::text"),
        "__last_updated_at" => Some("submission_fact.last_modified_at::text"),
        "__last_updated_by_user_name" => Some("submission_fact.last_modified_by_user_name"),
        _ => None,
    }
}

async fn materialize_dataset_revision(
    pool: &PgPool,
    revision_id: Uuid,
    generated_sql: &str,
) -> ApiResult<()> {
    let table_name = format!("dataset_{}", revision_id.simple());
    let full_name = format!(
        "{}.{}",
        quote_identifier("dataset_materialized"),
        quote_identifier(&table_name)
    );
    sqlx::query(&format!("DROP TABLE IF EXISTS {full_name}"))
        .execute(pool)
        .await?;
    sqlx::query(&format!("CREATE TABLE {full_name} AS {generated_sql}"))
        .execute(pool)
        .await?;
    sqlx::query(&format!("CREATE INDEX ON {full_name} (__row_id)"))
        .execute(pool)
        .await?;
    let row_count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {full_name}"))
        .fetch_one(pool)
        .await?;
    sqlx::query(
        r#"
        UPDATE dataset_revisions
        SET materialized_schema = 'dataset_materialized',
            materialized_table = $1,
            materialized_row_count = $2,
            materialized_at = now()
        WHERE id = $3
        "#,
    )
    .bind(&table_name)
    .bind(row_count)
    .bind(revision_id)
    .execute(pool)
    .await?;
    Ok(())
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

async fn replace_dataset_scope_nodes(
    pool: &PgPool,
    dataset_id: Uuid,
    node_ids: &[Uuid],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM dataset_scope_nodes WHERE dataset_id = $1")
        .bind(dataset_id)
        .execute(pool)
        .await?;
    for node_id in node_ids {
        sqlx::query(
            "INSERT INTO dataset_scope_nodes (dataset_id, node_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(dataset_id)
        .bind(node_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub(super) async fn ensure_component(
    pool: &PgPool,
    name: &str,
    slug: &str,
    dataset_revision_id: Uuid,
) -> ApiResult<(Uuid, Uuid)> {
    let component_id = if let Some(id) =
        sqlx::query_scalar("SELECT id FROM components WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?
    {
        sqlx::query("UPDATE components SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(pool)
            .await?;
        id
    } else {
        sqlx::query_scalar(
            "INSERT INTO components (name, slug, description) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(name)
        .bind(slug)
        .bind("Seeded demo component")
        .fetch_one(pool)
        .await?
    };

    let version_number: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_number), 0) + 1 FROM component_versions WHERE component_id = $1",
    )
    .bind(component_id)
    .fetch_one(pool)
    .await?;
    sqlx::query(
        r#"
        UPDATE component_versions
        SET status = 'superseded'::component_version_status
        WHERE component_id = $1
          AND status = 'published'::component_version_status
        "#,
    )
    .bind(component_id)
    .execute(pool)
    .await?;
    let component_version_id = sqlx::query_scalar(
        r#"
        INSERT INTO component_versions
            (component_id, dataset_revision_id, component_type, version_number, version_label, status, config, published_at)
        VALUES ($1, $2, 'detail_table'::component_type, $3, $4, 'published'::component_version_status, $5, now())
        RETURNING id
        "#,
    )
    .bind(component_id)
    .bind(dataset_revision_id)
    .bind(version_number)
    .bind(version_number.to_string())
    .bind(json!({"columns": "all"}))
    .fetch_one(pool)
    .await?;

    Ok((component_id, component_version_id))
}

pub(super) async fn ensure_dashboard(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
    visibility_node_ids: &[Uuid],
) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar("SELECT id FROM dashboards WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        sqlx::query("UPDATE dashboards SET description = $1 WHERE id = $2")
            .bind(description)
            .bind(id)
            .execute(pool)
            .await?;
        replace_dashboard_scope_nodes(pool, id, visibility_node_ids).await?;
        return Ok(id);
    }

    let id = sqlx::query_scalar(
        "INSERT INTO dashboards (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await?;
    replace_dashboard_scope_nodes(pool, id, visibility_node_ids).await?;
    Ok(id)
}

async fn replace_dashboard_scope_nodes(
    pool: &PgPool,
    dashboard_id: Uuid,
    node_ids: &[Uuid],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM dashboard_scope_nodes WHERE dashboard_id = $1")
        .bind(dashboard_id)
        .execute(pool)
        .await?;
    for node_id in node_ids {
        sqlx::query(
            "INSERT INTO dashboard_scope_nodes (dashboard_id, node_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(dashboard_id)
        .bind(node_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub(super) async fn replace_dashboard_components(
    pool: &PgPool,
    dashboard_id: Uuid,
    components: &[(Uuid, i32, Value)],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM dashboard_components WHERE dashboard_id = $1")
        .bind(dashboard_id)
        .execute(pool)
        .await?;

    for (component_version_id, position, config) in components {
        sqlx::query(
            r#"
            INSERT INTO dashboard_components (dashboard_id, component_version_id, position, config)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(dashboard_id)
        .bind(component_version_id)
        .bind(position)
        .bind(config)
        .execute(pool)
        .await?;
    }

    Ok(())
}
