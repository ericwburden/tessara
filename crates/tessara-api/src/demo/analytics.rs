use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiResult;

use super::forms::current_form_major;

pub(super) struct DatasetFieldBinding<'a> {
    pub(super) logical_key: &'a str,
    pub(super) label: &'a str,
    pub(super) source_field_key: &'a str,
    pub(super) field_type: &'a str,
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
    let form_version_major = current_form_major(pool, form_id).await?.ok_or_else(|| {
        crate::error::ApiError::BadRequest(format!(
            "demo dataset '{slug}' requires a published form version"
        ))
    })?;
    let dataset_id = if let Some(id) = sqlx::query_scalar("SELECT id FROM datasets WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?
    {
        sqlx::query(
            "UPDATE datasets SET name = $1, grain = 'submission', composition_mode = 'union' WHERE id = $2",
        )
            .bind(name)
            .bind(id)
            .execute(pool)
            .await?;
        id
    } else {
        sqlx::query_scalar(
            "INSERT INTO datasets (name, slug, grain, composition_mode) VALUES ($1, $2, 'submission', 'union') RETURNING id",
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
            (dataset_id, source_alias, form_id, form_version_major, selection_rule, position)
        VALUES ($1, $2, $3, $4, 'latest', 0)
        "#,
    )
    .bind(dataset_id)
    .bind(source_alias)
    .bind(form_id)
    .bind(form_version_major)
    .execute(pool)
    .await?;

    for (position, binding) in bindings.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO dataset_fields
                (dataset_id, key, label, source_alias, source_field_key, field_type, position)
            VALUES ($1, $2, $3, $4, $5, $6::field_type, $7)
            "#,
        )
        .bind(dataset_id)
        .bind(binding.logical_key)
        .bind(binding.label)
        .bind(source_alias)
        .bind(binding.source_field_key)
        .bind(binding.field_type)
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
    let definition_ast = json!({
        "kind": "form",
        "alias": source_alias,
        "form_id": form_id,
        "form_version_major": form_version_major,
        "selection_rule": "latest"
    });
    let generated_sql = generated_dataset_sql(form_id, form_version_major, bindings);
    let revision_id = sqlx::query_scalar(
        r#"
        INSERT INTO dataset_revisions
            (dataset_id, version_number, version_label, status, published_at, definition_ast, generated_sql)
        VALUES ($1, $2, $3, 'published'::dataset_revision_status, now(), $4, $5)
        RETURNING id
        "#,
    )
    .bind(dataset_id)
    .bind(version_number)
    .bind(version_number.to_string())
    .bind(definition_ast)
    .bind(&generated_sql)
    .fetch_one(pool)
    .await?;
    materialize_dataset_revision(pool, revision_id, &generated_sql).await?;

    Ok((dataset_id, revision_id))
}

fn generated_dataset_sql(
    form_id: Uuid,
    form_version_major: i32,
    bindings: &[DatasetFieldBinding<'_>],
) -> String {
    let select_columns = bindings
        .iter()
        .map(|binding| {
            format!(
                "MAX(submission_value_fact.value_text) FILTER (WHERE submission_value_fact.field_key = {}) AS {}",
                sql_literal(binding.source_field_key),
                quote_identifier(binding.logical_key)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n                ");

    format!(
        r#"WITH
        form_a_1 AS (
            WITH ranked AS (
                SELECT
                    submission_fact.submission_id,
                    submission_fact.node_id,
                    node_dim.node_name,
                    ROW_NUMBER() OVER (
                        PARTITION BY submission_fact.node_id
                        ORDER BY submission_fact.submitted_at DESC NULLS LAST, submission_fact.submission_id
                    ) AS selection_rank
                FROM analytics.submission_fact
                JOIN form_versions ON form_versions.id = submission_fact.form_version_id
                JOIN analytics.node_dim ON node_dim.node_id = submission_fact.node_id
                WHERE submission_fact.status = 'submitted'
                  AND form_versions.form_id = {}::uuid
                  AND form_versions.version_major = {form_version_major}
            )
            SELECT
                ranked.submission_id::text AS __row_id,
                ranked.node_id AS __node_id,
                ranked.node_name AS __node_name,
                {select_columns}
            FROM ranked
            LEFT JOIN analytics.submission_value_fact
              ON submission_value_fact.submission_id = ranked.submission_id
            WHERE selection_rank = 1
            GROUP BY ranked.submission_id, ranked.node_id, ranked.node_name
        )
        SELECT
            __row_id,
            __node_id,
            __node_name,
            {}
        FROM form_a_1"#,
        sql_literal(&form_id.to_string()),
        bindings
            .iter()
            .map(|binding| quote_identifier(binding.logical_key))
            .collect::<Vec<_>>()
            .join(",\n            ")
    )
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
    sqlx::query(&format!("CREATE INDEX ON {full_name} (__node_id)"))
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
