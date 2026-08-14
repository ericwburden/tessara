//! Dataset-owned materialized revision and major-line storage.

use std::collections::BTreeSet;

use sqlx::{Postgres, Row, Transaction};
use tessara_datasets_contract::DatasetProductFieldV1;
use uuid::Uuid;

use crate::DatasetModuleError;

pub(crate) async fn materialize_revision(
    tx: &mut Transaction<'_, Postgres>,
    revision_id: Uuid,
    generated_sql: &str,
) -> Result<(), DatasetModuleError> {
    let table_name = format!("dataset_{}", revision_id.simple());
    let full_name = format!(
        "{}.{}",
        quote_identifier("dataset_materialized"),
        quote_identifier(&table_name)
    );
    sqlx::query(&format!("DROP TABLE IF EXISTS {full_name}"))
        .execute(&mut **tx)
        .await?;
    sqlx::query(&format!("CREATE TABLE {full_name} AS {generated_sql}"))
        .execute(&mut **tx)
        .await?;
    sqlx::query(&format!("CREATE UNIQUE INDEX ON {full_name} (__row_id)"))
        .execute(&mut **tx)
        .await?;
    sqlx::query(&format!(
        "ALTER TABLE {full_name}
         ALTER COLUMN __restriction_tier SET NOT NULL,
         ALTER COLUMN __scope_node_ids SET NOT NULL,
         ADD CHECK (__restriction_tier IN ('public','internal','restricted','confidential')),
         ADD CHECK (cardinality(__scope_node_ids) > 0)"
    ))
    .execute(&mut **tx)
    .await?;
    sqlx::query(&format!(
        "CREATE INDEX ON {full_name} USING GIN (__scope_node_ids)"
    ))
    .execute(&mut **tx)
    .await?;
    let row_count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {full_name}"))
        .fetch_one(&mut **tx)
        .await?;
    sqlx::query(
        "UPDATE dataset_revisions
         SET materialized_schema='dataset_materialized',materialized_table=$1,
             materialized_row_count=$2,materialized_at=now(),
             resource_revision=resource_revision+1,updated_at=now()
         WHERE id=$3",
    )
    .bind(&table_name)
    .bind(row_count)
    .bind(revision_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn rebuild_major_line(
    tx: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    version_major: i32,
) -> Result<(), DatasetModuleError> {
    let rows = sqlx::query(
        "SELECT id,version_major,version_minor,version_patch,version_number,
                materialized_schema,materialized_table,output_fields
         FROM dataset_revisions
         WHERE dataset_id=$1 AND version_major=$2
           AND status IN ('published','superseded')
           AND materialized_schema IS NOT NULL AND materialized_table IS NOT NULL
         ORDER BY version_minor,version_patch,version_number",
    )
    .bind(dataset_id)
    .bind(version_major)
    .fetch_all(&mut **tx)
    .await?;
    if rows.is_empty() {
        return Err(DatasetModuleError::Conflict(format!(
            "Dataset version {version_major} has no materialized published revisions"
        )));
    }
    let output_fields = parse_fields(
        rows.last()
            .expect("non-empty rows")
            .try_get("output_fields")?,
    )?;
    let table_name = format!("dataset_major_{}_v{version_major}", dataset_id.simple());
    let full_name = format!(
        "{}.{}",
        quote_identifier("dataset_materialized"),
        quote_identifier(&table_name)
    );
    sqlx::query(&format!("DROP TABLE IF EXISTS {full_name}"))
        .execute(&mut **tx)
        .await?;
    let mut selects = Vec::new();
    for row in rows {
        let revision_id: Uuid = row.try_get("id")?;
        let major = row
            .try_get::<Option<i32>, _>("version_major")?
            .unwrap_or(version_major);
        let minor = row.try_get::<Option<i32>, _>("version_minor")?.unwrap_or(0);
        let patch = row.try_get::<Option<i32>, _>("version_patch")?.unwrap_or(0);
        let schema: String = row.try_get("materialized_schema")?;
        let table: String = row.try_get("materialized_table")?;
        let available = parse_fields(row.try_get("output_fields")?)?
            .into_iter()
            .map(|field| field.key)
            .collect::<BTreeSet<_>>();
        let mut columns = vec![
            format!("concat('{}:',__row_id)::text AS __row_id", revision_id),
            "__restriction_tier".into(),
            "__scope_node_ids".into(),
            format!("'{revision_id}'::uuid AS __source_dataset_revision_id"),
            format!("{major}::integer AS __source_dataset_version_major"),
            format!("{minor}::integer AS __source_dataset_version_minor"),
            format!("{patch}::integer AS __source_dataset_version_patch"),
            format!("'v{major}.{minor}.{patch}'::text AS __source_dataset_semantic_version"),
        ];
        columns.extend(output_fields.iter().map(|field| {
            let column = quote_identifier(&field.key);
            if available.contains(&field.key) {
                format!("{column}::text AS {column}")
            } else {
                format!("NULL::text AS {column}")
            }
        }));
        selects.push(format!(
            "SELECT {} FROM {}.{}",
            columns.join(", "),
            quote_identifier(&schema),
            quote_identifier(&table)
        ));
    }
    sqlx::query(&format!(
        "CREATE TABLE {full_name} AS {}",
        selects.join("\nUNION ALL\n")
    ))
    .execute(&mut **tx)
    .await?;
    sqlx::query(&format!("CREATE UNIQUE INDEX ON {full_name} (__row_id)"))
        .execute(&mut **tx)
        .await?;
    sqlx::query(&format!(
        "ALTER TABLE {full_name}
         ALTER COLUMN __restriction_tier SET NOT NULL,
         ALTER COLUMN __scope_node_ids SET NOT NULL,
         ADD CHECK (__restriction_tier IN ('public','internal','restricted','confidential')),
         ADD CHECK (cardinality(__scope_node_ids) > 0)"
    ))
    .execute(&mut **tx)
    .await?;
    sqlx::query(&format!(
        "CREATE INDEX ON {full_name} USING GIN (__scope_node_ids)"
    ))
    .execute(&mut **tx)
    .await?;
    sqlx::query(&format!(
        "CREATE INDEX ON {full_name} (__source_dataset_revision_id)"
    ))
    .execute(&mut **tx)
    .await?;
    let row_count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {full_name}"))
        .fetch_one(&mut **tx)
        .await?;
    sqlx::query(
        "INSERT INTO dataset_major_materializations
         (dataset_id,version_major,materialized_schema,materialized_table,
          materialized_row_count,materialized_at,rebuild_status,updated_at)
         VALUES($1,$2,'dataset_materialized',$3,$4,now(),'ready',now())
         ON CONFLICT(dataset_id,version_major) DO UPDATE SET
          materialized_schema=EXCLUDED.materialized_schema,
          materialized_table=EXCLUDED.materialized_table,
          materialized_row_count=EXCLUDED.materialized_row_count,
          materialized_at=EXCLUDED.materialized_at,rebuild_status='ready',updated_at=now()",
    )
    .bind(dataset_id)
    .bind(version_major)
    .bind(table_name)
    .bind(row_count)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "UPDATE datasets
         SET resource_revision=resource_revision+1,updated_at=now()
         WHERE id=$1",
    )
    .bind(dataset_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn parse_fields(
    value: serde_json::Value,
) -> Result<Vec<DatasetProductFieldV1>, DatasetModuleError> {
    serde_json::from_value(value).map_err(|error| {
        DatasetModuleError::Internal(format!("Stored Dataset output fields are invalid: {error}"))
    })
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
