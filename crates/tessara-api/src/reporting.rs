use std::{collections::HashSet, str::FromStr, sync::Arc};

use arrow::{
    array::{Array, ArrayRef, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use datafusion::{datasource::MemTable, prelude::SessionContext};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tessara_reporting::MissingDataPolicy;
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::IdResponse,
};

#[derive(Deserialize)]
pub struct CreateReportRequest {
    name: String,
    form_id: Option<Uuid>,
    fields: Vec<CreateReportFieldBindingRequest>,
}

#[derive(Deserialize)]
pub struct CreateReportFieldBindingRequest {
    logical_key: String,
    source_field_key: String,
    missing_policy: Option<String>,
}

struct ParsedReportFieldBinding {
    logical_key: String,
    source_field_key: String,
    missing_policy: MissingDataPolicy,
}

#[derive(Serialize)]
pub struct ReportTable {
    report_id: Uuid,
    rows: Vec<ReportTableRow>,
}

#[derive(Serialize)]
pub struct ReportTableRow {
    submission_id: Option<String>,
    node_name: Option<String>,
    logical_key: Option<String>,
    field_value: Option<String>,
}

pub async fn create_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateReportRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;

    if payload.fields.is_empty() {
        return Err(ApiError::BadRequest(
            "a report requires at least one field binding".into(),
        ));
    }

    let fields =
        validate_report_field_bindings(&state.pool, payload.form_id, payload.fields).await?;

    let mut tx = state.pool.begin().await?;
    let report_id: Uuid =
        sqlx::query_scalar("INSERT INTO reports (name, form_id) VALUES ($1, $2) RETURNING id")
            .bind(payload.name)
            .bind(payload.form_id)
            .fetch_one(&mut *tx)
            .await?;

    for (position, field) in fields.into_iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO report_field_bindings
                (report_id, logical_key, source_field_key, missing_policy, position)
            VALUES ($1, $2, $3, $4::missing_data_policy, $5)
            "#,
        )
        .bind(report_id)
        .bind(field.logical_key)
        .bind(field.source_field_key)
        .bind(field.missing_policy.as_str())
        .bind(position as i32)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(IdResponse { id: report_id }))
}

pub async fn run_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(report_id): Path<Uuid>,
) -> ApiResult<Json<ReportTable>> {
    auth::require_capability(&state.pool, &headers, "reports:read").await?;
    require_report_exists(&state.pool, report_id).await?;

    let source_rows = sqlx::query(
        r#"
        SELECT
            submission_fact.submission_id::text AS submission_id,
            node_dim.node_name,
            report_field_bindings.logical_key,
            CASE
                WHEN submission_value_fact.value_text IS NULL
                     AND report_field_bindings.missing_policy::text = 'bucket_unknown'
                    THEN 'Unknown'
                ELSE submission_value_fact.value_text
            END AS field_value
        FROM reports
        JOIN report_field_bindings
            ON report_field_bindings.report_id = reports.id
        JOIN analytics.form_version_dim
            ON reports.form_id IS NULL OR analytics.form_version_dim.form_id = reports.form_id
        JOIN analytics.submission_fact
            ON submission_fact.form_version_id = analytics.form_version_dim.form_version_id
        JOIN analytics.node_dim
            ON node_dim.node_id = submission_fact.node_id
        LEFT JOIN analytics.submission_value_fact
            ON submission_value_fact.submission_id = submission_fact.submission_id
           AND submission_value_fact.field_key = report_field_bindings.source_field_key
        WHERE reports.id = $1
          AND submission_fact.status = 'submitted'
          AND (
            submission_value_fact.value_text IS NOT NULL
            OR report_field_bindings.missing_policy::text <> 'exclude_row'
          )
        ORDER BY submission_fact.submission_id, report_field_bindings.position
        "#,
    )
    .bind(report_id)
    .fetch_all(&state.pool)
    .await?;

    let mut submission_ids = Vec::with_capacity(source_rows.len());
    let mut node_names = Vec::with_capacity(source_rows.len());
    let mut logical_keys = Vec::with_capacity(source_rows.len());
    let mut field_values = Vec::with_capacity(source_rows.len());

    for row in source_rows {
        submission_ids.push(row.try_get::<String, _>("submission_id").ok());
        node_names.push(row.try_get::<String, _>("node_name").ok());
        logical_keys.push(row.try_get::<String, _>("logical_key").ok());
        field_values.push(row.try_get::<Option<String>, _>("field_value")?);
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("submission_id", DataType::Utf8, true),
        Field::new("node_name", DataType::Utf8, true),
        Field::new("logical_key", DataType::Utf8, true),
        Field::new("field_value", DataType::Utf8, true),
    ]));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(submission_ids)) as ArrayRef,
            Arc::new(StringArray::from(node_names)) as ArrayRef,
            Arc::new(StringArray::from(logical_keys)) as ArrayRef,
            Arc::new(StringArray::from(field_values)) as ArrayRef,
        ],
    )
    .map_err(|err| ApiError::Internal(err.into()))?;

    let table = MemTable::try_new(schema, vec![vec![batch]])
        .map_err(|err| ApiError::Internal(err.into()))?;
    let context = SessionContext::new();
    context
        .register_table("report_values", Arc::new(table))
        .map_err(|err| ApiError::Internal(err.into()))?;

    let frame = context
        .sql(
            r#"
            SELECT submission_id, node_name, logical_key, field_value
            FROM report_values
            ORDER BY submission_id, logical_key
            "#,
        )
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;
    let batches = frame
        .collect()
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;

    let mut rows = Vec::new();
    for batch in batches {
        let submission_ids = as_string_array(&batch, 0)?;
        let node_names = as_string_array(&batch, 1)?;
        let logical_keys = as_string_array(&batch, 2)?;
        let field_values = as_string_array(&batch, 3)?;

        for index in 0..batch.num_rows() {
            rows.push(ReportTableRow {
                submission_id: string_value(submission_ids, index),
                node_name: string_value(node_names, index),
                logical_key: string_value(logical_keys, index),
                field_value: string_value(field_values, index),
            });
        }
    }

    Ok(Json(ReportTable { report_id, rows }))
}

async fn validate_report_field_bindings(
    pool: &sqlx::PgPool,
    form_id: Option<Uuid>,
    fields: Vec<CreateReportFieldBindingRequest>,
) -> ApiResult<Vec<ParsedReportFieldBinding>> {
    if let Some(form_id) = form_id {
        require_form_exists(pool, form_id).await?;
    }

    let mut logical_keys = HashSet::new();
    let mut parsed_fields = Vec::with_capacity(fields.len());
    for field in fields {
        if !logical_keys.insert(field.logical_key.clone()) {
            return Err(ApiError::BadRequest(format!(
                "report logical field '{}' is duplicated",
                field.logical_key
            )));
        }

        let missing_policy = field
            .missing_policy
            .as_deref()
            .map(MissingDataPolicy::from_str)
            .transpose()
            .map_err(|error| ApiError::BadRequest(error.to_string()))?
            .unwrap_or(MissingDataPolicy::Null);

        parsed_fields.push(ParsedReportFieldBinding {
            logical_key: field.logical_key,
            source_field_key: field.source_field_key,
            missing_policy,
        });
    }

    if let Some(form_id) = form_id {
        assert_report_source_fields_exist(pool, form_id, &parsed_fields).await?;
    }

    Ok(parsed_fields)
}

async fn require_form_exists(pool: &sqlx::PgPool, form_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM forms WHERE id = $1)")
        .bind(form_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("form {form_id}")))
    }
}

async fn require_report_exists(pool: &sqlx::PgPool, report_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM reports WHERE id = $1)")
        .bind(report_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("report {report_id}")))
    }
}

async fn assert_report_source_fields_exist(
    pool: &sqlx::PgPool,
    form_id: Uuid,
    fields: &[ParsedReportFieldBinding],
) -> ApiResult<()> {
    for field in fields {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM form_fields
                JOIN form_versions ON form_versions.id = form_fields.form_version_id
                WHERE form_versions.form_id = $1 AND form_fields.key = $2
            )
            "#,
        )
        .bind(form_id)
        .bind(&field.source_field_key)
        .fetch_one(pool)
        .await?;

        if !exists {
            return Err(ApiError::BadRequest(format!(
                "report source field '{}' is not available on form {form_id}",
                field.source_field_key
            )));
        }
    }

    Ok(())
}

fn as_string_array(batch: &RecordBatch, column: usize) -> ApiResult<&StringArray> {
    batch
        .column(column)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("report column was not Utf8")))
}

fn string_value(array: &StringArray, index: usize) -> Option<String> {
    if array.is_null(index) {
        None
    } else {
        Some(array.value(index).to_string())
    }
}

#[cfg(test)]
mod tests {
    use arrow::array::StringArray;

    use super::string_value;

    #[test]
    fn string_value_preserves_nulls_from_arrow_arrays() {
        let values = StringArray::from(vec![Some("North"), None]);

        assert_eq!(string_value(&values, 0), Some("North".to_string()));
        assert_eq!(string_value(&values, 1), None);
    }
}
