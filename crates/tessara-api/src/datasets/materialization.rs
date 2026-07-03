use super::*;

pub(super) async fn materialize_dataset_revision(
    tx: &mut Transaction<'_, Postgres>,
    revision_id: Uuid,
    compiled: &CompiledDataset,
) -> ApiResult<()> {
    materialize_dataset_revision_sql(tx, revision_id, &compiled.generated_sql).await
}

async fn materialize_dataset_revision_sql(
    tx: &mut Transaction<'_, Postgres>,
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
        .execute(&mut **tx)
        .await?;
    sqlx::query(&format!("CREATE TABLE {full_name} AS {generated_sql}"))
        .execute(&mut **tx)
        .await?;
    sqlx::query(&format!("CREATE INDEX ON {full_name} (__row_id)"))
        .execute(&mut **tx)
        .await?;
    let row_count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {full_name}"))
        .fetch_one(&mut **tx)
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
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn rebuild_dataset_major_materialization(
    tx: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    version_major: i32,
) -> ApiResult<()> {
    let revision_rows = sqlx::query(
        r#"
        SELECT id, version_major, version_minor, version_patch, version_number,
               materialized_schema, materialized_table, output_fields
        FROM dataset_revisions
        WHERE dataset_id = $1
          AND version_major = $2
          AND status IN ('published'::dataset_revision_status, 'superseded'::dataset_revision_status)
          AND materialized_schema IS NOT NULL
          AND materialized_table IS NOT NULL
        ORDER BY version_major,
                 COALESCE(version_minor, 0),
                 COALESCE(version_patch, 0),
                 version_number
        "#,
    )
    .bind(dataset_id)
    .bind(version_major)
    .fetch_all(&mut **tx)
    .await?;
    if revision_rows.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "dataset version {version_major} has no materialized published revisions"
        )));
    }

    let latest_revision = revision_rows
        .last()
        .expect("revision rows should not be empty after guard");
    let output_fields =
        parse_revision_output_field_value(latest_revision.try_get("output_fields")?)?;
    let table_name = format!("dataset_major_{}_v{}", dataset_id.simple(), version_major);
    let full_name = format!(
        "{}.{}",
        quote_identifier("dataset_materialized"),
        quote_identifier(&table_name)
    );
    sqlx::query(&format!("DROP TABLE IF EXISTS {full_name}"))
        .execute(&mut **tx)
        .await?;

    let mut selects = Vec::new();
    for row in revision_rows {
        let revision_id: Uuid = row.try_get("id")?;
        let source_major: Option<i32> = row.try_get("version_major")?;
        let source_minor: Option<i32> = row.try_get("version_minor")?;
        let source_patch: Option<i32> = row.try_get("version_patch")?;
        let schema: String = row.try_get("materialized_schema")?;
        let table: String = row.try_get("materialized_table")?;
        let fields = parse_revision_output_field_value(row.try_get("output_fields")?)?
            .into_iter()
            .map(|field| field.key)
            .collect::<BTreeSet<_>>();
        let mut columns = vec![
            format!("concat('{}:', __row_id)::text AS __row_id", revision_id),
            "__restriction_tier".to_string(),
            format!("'{}'::uuid AS __source_dataset_revision_id", revision_id),
            format!(
                "{}::integer AS __source_dataset_version_major",
                source_major.unwrap_or(version_major)
            ),
            format!(
                "{}::integer AS __source_dataset_version_minor",
                source_minor.unwrap_or(0)
            ),
            format!(
                "{}::integer AS __source_dataset_version_patch",
                source_patch.unwrap_or(0)
            ),
            format!(
                "'v{}.{}.{}'::text AS __source_dataset_semantic_version",
                source_major.unwrap_or(version_major),
                source_minor.unwrap_or(0),
                source_patch.unwrap_or(0)
            ),
        ];
        columns.extend(output_fields.iter().map(|field| {
            let column = quote_identifier(&field.key);
            if fields.contains(&field.key) {
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
    sqlx::query(&format!("CREATE INDEX ON {full_name} (__row_id)"))
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
        r#"
        INSERT INTO dataset_major_materializations
            (dataset_id, version_major, materialized_schema, materialized_table, materialized_row_count, materialized_at, rebuild_status, updated_at)
        VALUES ($1, $2, 'dataset_materialized', $3, $4, now(), 'ready', now())
        ON CONFLICT (dataset_id, version_major)
        DO UPDATE SET materialized_schema = EXCLUDED.materialized_schema,
                      materialized_table = EXCLUDED.materialized_table,
                      materialized_row_count = EXCLUDED.materialized_row_count,
                      materialized_at = EXCLUDED.materialized_at,
                      rebuild_status = 'ready',
                      updated_at = now()
        "#,
    )
    .bind(dataset_id)
    .bind(version_major)
    .bind(table_name)
    .bind(row_count)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn refresh_major_line_consumers(
    tx: &mut Transaction<'_, Postgres>,
    source_dataset_id: Uuid,
    source_version_major: i32,
) -> ApiResult<()> {
    let mut pending = vec![(source_dataset_id, source_version_major)];
    let mut visited = BTreeSet::<(Uuid, i32)>::new();

    while let Some((upstream_dataset_id, upstream_major)) = pending.pop() {
        if !visited.insert((upstream_dataset_id, upstream_major)) {
            continue;
        }

        let consumer_rows = sqlx::query(
            r#"
            SELECT DISTINCT
                dataset_sources.dataset_id,
                current_revisions.id AS revision_id,
                current_revisions.version_major,
                current_revisions.generated_sql
            FROM dataset_sources
            JOIN dataset_revisions AS current_revisions
              ON current_revisions.dataset_id = dataset_sources.dataset_id
             AND current_revisions.status = 'published'::dataset_revision_status
            WHERE dataset_sources.source_dataset_id = $1
              AND dataset_sources.dataset_version_major = $2
              AND current_revisions.generated_sql IS NOT NULL
            "#,
        )
        .bind(upstream_dataset_id)
        .bind(upstream_major)
        .fetch_all(&mut **tx)
        .await?;

        for row in consumer_rows {
            let consumer_dataset_id: Uuid = row.try_get("dataset_id")?;
            let consumer_revision_id: Uuid = row.try_get("revision_id")?;
            let consumer_major: Option<i32> = row.try_get("version_major")?;
            let generated_sql: String = row.try_get("generated_sql")?;
            materialize_dataset_revision_sql(tx, consumer_revision_id, &generated_sql).await?;
            if let Some(consumer_major) = consumer_major {
                rebuild_dataset_major_materialization(tx, consumer_dataset_id, consumer_major)
                    .await?;
                pending.push((consumer_dataset_id, consumer_major));
            }
        }
    }

    Ok(())
}

pub(super) async fn load_dataset_revision_output_fields(
    pool: &sqlx::PgPool,
    dataset_revision_id: Uuid,
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let revision = sqlx::query(
        r#"
        SELECT dataset_id, output_fields
        FROM dataset_revisions
        WHERE id = $1
        "#,
    )
    .bind(dataset_revision_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ApiError::BadRequest(format!(
            "dataset revision {dataset_revision_id} is not available"
        ))
    })?;
    if let Some(output_fields) =
        revision.try_get::<Option<serde_json::Value>, _>("output_fields")?
    {
        let fields = parse_revision_output_field_value(Some(output_fields))?;
        return Ok(fields
            .into_iter()
            .map(|field| ValidatedDatasetField {
                id: Some(field.id),
                key: field.key,
                label: field.label,
                source_alias: field.source_alias,
                source_field_key: field.source_field_key,
                source_field_id: None,
                field_type: field.field_type,
                position: field.position,
            })
            .collect());
    }
    let dataset_id: Uuid = revision.try_get("dataset_id")?;
    let fields = sqlx::query(
        r#"
        SELECT id, key, label, source_alias, source_field_key, source_field_id, field_type::text AS field_type, position
        FROM dataset_fields
        WHERE dataset_id = $1
        ORDER BY position, key
        "#,
    )
    .bind(dataset_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ValidatedDatasetField {
            id: row.try_get("id")?,
            key: row.try_get("key")?,
            label: row.try_get("label")?,
            source_alias: row.try_get("source_alias")?,
            source_field_key: row.try_get("source_field_key")?,
            source_field_id: row.try_get("source_field_id")?,
            field_type: row.try_get("field_type")?,
            position: row.try_get("position")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;
    Ok(fields)
}

fn parse_revision_output_field_value(
    value: Option<serde_json::Value>,
) -> ApiResult<Vec<DatasetFieldDefinition>> {
    value
        .map(serde_json::from_value::<Vec<DatasetFieldDefinition>>)
        .transpose()
        .map_err(|error| {
            ApiError::Internal(anyhow::anyhow!(
                "stored dataset revision output fields are invalid: {error}"
            ))
        })
        .map(Option::unwrap_or_default)
}

pub(super) async fn load_dataset_major_source_catalog(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
    version_major: i32,
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let revision_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT id
        FROM dataset_revisions
        WHERE dataset_id = $1
          AND version_major = $2
          AND status IN ('published'::dataset_revision_status, 'superseded'::dataset_revision_status)
        ORDER BY COALESCE(version_minor, 0) DESC,
                 COALESCE(version_patch, 0) DESC,
                 version_number DESC
        LIMIT 1
        "#,
    )
    .bind(dataset_id)
    .bind(version_major)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ApiError::BadRequest(format!("dataset version {version_major} is not available"))
    })?;
    load_dataset_revision_output_fields(pool, revision_id).await
}
