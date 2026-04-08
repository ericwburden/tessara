use std::sync::Arc;

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
use tessara_reporting::{
    ReportFieldBindingDraft, ReportFieldBindingInput, parse_report_field_bindings,
};
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, require_text},
};

#[derive(Deserialize)]
pub struct CreateReportRequest {
    name: String,
    form_id: Option<Uuid>,
    dataset_id: Option<Uuid>,
    fields: Vec<CreateReportFieldBindingRequest>,
}

#[derive(Deserialize)]
pub struct CreateReportFieldBindingRequest {
    logical_key: String,
    source_field_key: Option<String>,
    computed_expression: Option<String>,
    missing_policy: Option<String>,
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

#[derive(Serialize)]
pub struct ReportSummary {
    id: Uuid,
    name: String,
    form_id: Option<Uuid>,
    form_name: Option<String>,
    dataset_id: Option<Uuid>,
    dataset_name: Option<String>,
}

#[derive(Serialize)]
pub struct ReportDefinition {
    id: Uuid,
    name: String,
    form_id: Option<Uuid>,
    form_name: Option<String>,
    dataset_id: Option<Uuid>,
    dataset_name: Option<String>,
    bindings: Vec<ReportFieldBindingSummary>,
}

#[derive(Serialize)]
pub struct ReportFieldBindingSummary {
    id: Uuid,
    logical_key: String,
    source_field_key: Option<String>,
    computed_expression: Option<String>,
    missing_policy: String,
    position: i32,
}

pub async fn create_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateReportRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_text("report name", &payload.name)?;

    if payload.fields.is_empty() {
        return Err(ApiError::BadRequest(
            "a report requires at least one field binding".into(),
        ));
    }

    let fields = validate_report_field_bindings(
        &state.pool,
        payload.form_id,
        payload.dataset_id,
        payload.fields,
    )
    .await?;

    let mut tx = state.pool.begin().await?;
    let report_id: Uuid = sqlx::query_scalar(
        "INSERT INTO reports (name, form_id, dataset_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(payload.name)
    .bind(payload.form_id)
    .bind(payload.dataset_id)
    .fetch_one(&mut *tx)
    .await?;

    insert_report_field_bindings(&mut tx, report_id, fields).await?;

    tx.commit().await?;

    Ok(Json(IdResponse { id: report_id }))
}

/// Updates an existing report definition and replaces its field bindings.
pub async fn update_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(report_id): Path<Uuid>,
    Json(payload): Json<CreateReportRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_report_exists(&state.pool, report_id).await?;
    require_text("report name", &payload.name)?;

    if payload.fields.is_empty() {
        return Err(ApiError::BadRequest(
            "a report requires at least one field binding".into(),
        ));
    }

    let fields = validate_report_field_bindings(
        &state.pool,
        payload.form_id,
        payload.dataset_id,
        payload.fields,
    )
    .await?;

    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE reports SET name = $1, form_id = $2, dataset_id = $3 WHERE id = $4")
        .bind(payload.name)
        .bind(payload.form_id)
        .bind(payload.dataset_id)
        .bind(report_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM report_field_bindings WHERE report_id = $1")
        .bind(report_id)
        .execute(&mut *tx)
        .await?;
    insert_report_field_bindings(&mut tx, report_id, fields).await?;

    tx.commit().await?;

    Ok(Json(IdResponse { id: report_id }))
}

/// Deletes an existing report definition and its field bindings.
pub async fn delete_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(report_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "reports:write").await?;
    require_report_exists(&state.pool, report_id).await?;

    sqlx::query("DELETE FROM reports WHERE id = $1")
        .bind(report_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: report_id }))
}

pub async fn list_reports(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ReportSummary>>> {
    auth::require_capability(&state.pool, &headers, "reports:read").await?;

    let rows = sqlx::query(
        r#"
        SELECT
            reports.id,
            reports.name,
            reports.form_id,
            forms.name AS form_name,
            reports.dataset_id,
            datasets.name AS dataset_name
        FROM reports
        LEFT JOIN forms ON forms.id = reports.form_id
        LEFT JOIN datasets ON datasets.id = reports.dataset_id
        ORDER BY reports.name, reports.created_at
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let reports = rows
        .into_iter()
        .map(|row| {
            Ok(ReportSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                form_id: row.try_get("form_id")?,
                form_name: row.try_get("form_name")?,
                dataset_id: row.try_get("dataset_id")?,
                dataset_name: row.try_get("dataset_name")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(reports))
}

/// Returns a report definition with its configured field bindings.
pub async fn get_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(report_id): Path<Uuid>,
) -> ApiResult<Json<ReportDefinition>> {
    auth::require_capability(&state.pool, &headers, "reports:read").await?;

    let report = sqlx::query(
        r#"
        SELECT
            reports.id,
            reports.name,
            reports.form_id,
            forms.name AS form_name,
            reports.dataset_id,
            datasets.name AS dataset_name
        FROM reports
        LEFT JOIN forms ON forms.id = reports.form_id
        LEFT JOIN datasets ON datasets.id = reports.dataset_id
        WHERE reports.id = $1
        "#,
    )
    .bind(report_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("report {report_id}")))?;

    let rows = sqlx::query(
        r#"
        SELECT
            id,
            logical_key,
            source_field_key,
            computed_expression,
            missing_policy::text AS missing_policy,
            position
        FROM report_field_bindings
        WHERE report_id = $1
        ORDER BY position, logical_key
        "#,
    )
    .bind(report_id)
    .fetch_all(&state.pool)
    .await?;

    let bindings = rows
        .into_iter()
        .map(|row| {
            Ok(ReportFieldBindingSummary {
                id: row.try_get("id")?,
                logical_key: row.try_get("logical_key")?,
                source_field_key: row.try_get("source_field_key")?,
                computed_expression: row.try_get("computed_expression")?,
                missing_policy: row.try_get("missing_policy")?,
                position: row.try_get("position")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(ReportDefinition {
        id: report.try_get("id")?,
        name: report.try_get("name")?,
        form_id: report.try_get("form_id")?,
        form_name: report.try_get("form_name")?,
        dataset_id: report.try_get("dataset_id")?,
        dataset_name: report.try_get("dataset_name")?,
        bindings,
    }))
}

pub async fn run_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(report_id): Path<Uuid>,
) -> ApiResult<Json<ReportTable>> {
    auth::require_capability(&state.pool, &headers, "reports:read").await?;
    require_report_exists(&state.pool, report_id).await?;

    let report_dataset_id: Option<Uuid> =
        sqlx::query_scalar("SELECT dataset_id FROM reports WHERE id = $1")
            .bind(report_id)
            .fetch_one(&state.pool)
            .await?;

    let source_rows = if let Some(dataset_id) = report_dataset_id {
        assert_report_dataset_is_executable(&state.pool, dataset_id).await?;
        sqlx::query(
            r#"
            SELECT
                submission_fact.submission_id::text AS submission_id,
                node_dim.node_name,
                report_field_bindings.logical_key,
                CASE
                    WHEN report_field_bindings.computed_expression IS NOT NULL
                        THEN substring(report_field_bindings.computed_expression from 9)
                    WHEN submission_value_fact.value_text IS NULL
                         AND report_field_bindings.missing_policy::text = 'bucket_unknown'
                        THEN 'Unknown'
                    ELSE submission_value_fact.value_text
                END AS field_value
            FROM reports
            JOIN dataset_sources
                ON dataset_sources.dataset_id = reports.dataset_id
            JOIN report_field_bindings
                ON report_field_bindings.report_id = reports.id
            LEFT JOIN dataset_fields
                ON dataset_fields.dataset_id = reports.dataset_id
               AND dataset_fields.key = report_field_bindings.source_field_key
               AND dataset_fields.source_alias = dataset_sources.source_alias
            JOIN analytics.form_version_dim
                ON form_version_dim.form_id = dataset_sources.form_id
            JOIN analytics.submission_fact
                ON submission_fact.form_version_id = form_version_dim.form_version_id
            JOIN analytics.node_dim
                ON node_dim.node_id = submission_fact.node_id
            LEFT JOIN analytics.submission_value_fact
                ON submission_value_fact.submission_id = submission_fact.submission_id
               AND submission_value_fact.field_key = dataset_fields.source_field_key
            WHERE reports.id = $1
              AND dataset_sources.form_id IS NOT NULL
              AND dataset_sources.compatibility_group_id IS NULL
              AND dataset_sources.selection_rule = 'all'
              AND submission_fact.status = 'submitted'
              AND (
                report_field_bindings.computed_expression IS NOT NULL
                OR
                submission_value_fact.value_text IS NOT NULL
                OR report_field_bindings.missing_policy::text <> 'exclude_row'
              )
            ORDER BY submission_fact.submission_id, report_field_bindings.position
            "#,
        )
        .bind(report_id)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                submission_fact.submission_id::text AS submission_id,
                node_dim.node_name,
                report_field_bindings.logical_key,
                CASE
                    WHEN report_field_bindings.computed_expression IS NOT NULL
                        THEN substring(report_field_bindings.computed_expression from 9)
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
                report_field_bindings.computed_expression IS NOT NULL
                OR
                submission_value_fact.value_text IS NOT NULL
                OR report_field_bindings.missing_policy::text <> 'exclude_row'
              )
            ORDER BY submission_fact.submission_id, report_field_bindings.position
            "#,
        )
        .bind(report_id)
        .fetch_all(&state.pool)
        .await?
    };

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
    dataset_id: Option<Uuid>,
    fields: Vec<CreateReportFieldBindingRequest>,
) -> ApiResult<Vec<ReportFieldBindingDraft>> {
    if form_id.is_some() && dataset_id.is_some() {
        return Err(ApiError::BadRequest(
            "a report can bind to either a form or a dataset, not both".into(),
        ));
    }

    let parsed_fields =
        parse_report_field_bindings(fields.iter().map(|field| ReportFieldBindingInput {
            logical_key: &field.logical_key,
            source_field_key: field.source_field_key.as_deref(),
            computed_expression: field.computed_expression.as_deref(),
            missing_policy: field.missing_policy.as_deref(),
        }))
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;

    if let Some(form_id) = form_id {
        require_form_exists(pool, form_id).await?;
        assert_report_source_fields_exist(pool, form_id, &parsed_fields).await?;
    } else if let Some(dataset_id) = dataset_id {
        require_dataset_exists(pool, dataset_id).await?;
        assert_report_dataset_is_executable(pool, dataset_id).await?;
        assert_report_dataset_fields_exist(pool, dataset_id, &parsed_fields).await?;
    }

    Ok(parsed_fields)
}

async fn insert_report_field_bindings(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    report_id: Uuid,
    fields: Vec<ReportFieldBindingDraft>,
) -> ApiResult<()> {
    for (position, field) in fields.into_iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO report_field_bindings
                (report_id, logical_key, source_field_key, computed_expression, missing_policy, position)
            VALUES ($1, $2, $3, $4, $5::missing_data_policy, $6)
            "#,
        )
        .bind(report_id)
        .bind(field.logical_key)
        .bind(field.source_field_key)
        .bind(field.computed_expression)
        .bind(field.missing_policy.as_str())
        .bind(position as i32)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
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

async fn require_dataset_exists(pool: &sqlx::PgPool, dataset_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM datasets WHERE id = $1)")
        .bind(dataset_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("dataset {dataset_id}")))
    }
}

async fn assert_report_dataset_is_executable(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
) -> ApiResult<()> {
    let grain: String = sqlx::query_scalar("SELECT grain FROM datasets WHERE id = $1")
        .bind(dataset_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dataset {dataset_id}")))?;

    if grain != "submission" {
        return Err(ApiError::BadRequest(
            "dataset-backed reports currently support only submission grain".into(),
        ));
    }

    let (source_count, executable_source_count): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) AS source_count,
            COUNT(*) FILTER (
                WHERE form_id IS NOT NULL
                  AND compatibility_group_id IS NULL
                  AND selection_rule = 'all'
            ) AS executable_source_count
        FROM dataset_sources
        WHERE dataset_id = $1
        "#,
    )
    .bind(dataset_id)
    .fetch_one(pool)
    .await?;

    if source_count == 1 && executable_source_count == 1 {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "dataset-backed reports currently require one form source with all records".into(),
        ))
    }
}

async fn assert_report_source_fields_exist(
    pool: &sqlx::PgPool,
    form_id: Uuid,
    fields: &[ReportFieldBindingDraft],
) -> ApiResult<()> {
    for field in fields {
        let Some(source_field_key) = &field.source_field_key else {
            continue;
        };
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
        .bind(source_field_key)
        .fetch_one(pool)
        .await?;

        if !exists {
            return Err(ApiError::BadRequest(format!(
                "report source field '{}' is not available on form {form_id}",
                source_field_key
            )));
        }
    }

    Ok(())
}

async fn assert_report_dataset_fields_exist(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
    fields: &[ReportFieldBindingDraft],
) -> ApiResult<()> {
    for field in fields {
        let Some(source_field_key) = &field.source_field_key else {
            continue;
        };
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM dataset_fields
                WHERE dataset_id = $1 AND key = $2
            )
            "#,
        )
        .bind(dataset_id)
        .bind(source_field_key)
        .fetch_one(pool)
        .await?;

        if !exists {
            return Err(ApiError::BadRequest(format!(
                "report dataset field '{}' is not available on dataset {dataset_id}",
                source_field_key
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
