use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

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
use sqlx::{Row, postgres::PgRow};
use tessara_datasets::DatasetCompositionMode;
use tessara_reporting::{
    ReportFieldBindingDraft, ReportFieldBindingInput, parse_report_field_bindings,
};
use uuid::Uuid;

use crate::{
    auth,
    datasets::{DatasetTableRow, load_dataset_table_rows, require_executable_submission_dataset},
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
    source_alias: Option<String>,
    logical_key: Option<String>,
    field_value: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateAggregationRequest {
    name: String,
    report_id: Uuid,
    group_by_logical_key: Option<String>,
    metrics: Vec<CreateAggregationMetricRequest>,
}

#[derive(Deserialize)]
pub struct CreateAggregationMetricRequest {
    metric_key: String,
    source_logical_key: Option<String>,
    metric_kind: String,
}

#[derive(Serialize)]
pub struct AggregationSummary {
    id: Uuid,
    name: String,
    report_id: Uuid,
    report_name: String,
    group_by_logical_key: Option<String>,
    metric_count: i64,
}

#[derive(Serialize)]
pub struct AggregationDefinition {
    id: Uuid,
    name: String,
    report_id: Uuid,
    report_name: String,
    group_by_logical_key: Option<String>,
    metrics: Vec<AggregationMetricSummary>,
}

#[derive(Serialize)]
pub struct AggregationMetricSummary {
    metric_key: String,
    source_logical_key: Option<String>,
    metric_kind: String,
    position: i32,
}

#[derive(Serialize)]
pub struct AggregationResult {
    aggregation_id: Uuid,
    rows: Vec<AggregationResultRow>,
}

#[derive(Serialize)]
pub struct AggregationResultRow {
    group_key: String,
    metrics: BTreeMap<String, f64>,
}

#[derive(Default)]
struct AggregationMetricAccumulator {
    count: f64,
    total: f64,
    numeric_count: f64,
    min: Option<f64>,
    max: Option<f64>,
}

impl AggregationMetricAccumulator {
    fn increment_count(&mut self) {
        self.count += 1.0;
    }

    fn add_numeric(&mut self, value: f64) {
        self.total += value;
        self.numeric_count += 1.0;
        self.min = Some(self.min.map_or(value, |current| current.min(value)));
        self.max = Some(self.max.map_or(value, |current| current.max(value)));
    }

    fn finalize(&self, metric_kind: &str) -> ApiResult<f64> {
        match metric_kind {
            "count" => Ok(self.count),
            "sum" => Ok(self.total),
            "avg" => Ok(if self.numeric_count > 0.0 {
                self.total / self.numeric_count
            } else {
                0.0
            }),
            "min" => Ok(self.min.unwrap_or(0.0)),
            "max" => Ok(self.max.unwrap_or(0.0)),
            other => Err(ApiError::BadRequest(format!(
                "unsupported aggregation metric kind '{other}'"
            ))),
        }
    }
}

struct RuntimeAggregationMetric {
    metric_key: String,
    source_logical_key: Option<String>,
    metric_kind: String,
}

struct RuntimeReportBinding {
    logical_key: String,
    source_field_key: Option<String>,
    computed_expression: Option<String>,
    missing_policy: String,
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
    aggregations: Vec<ReportAggregationLink>,
    charts: Vec<ReportChartLink>,
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

#[derive(Serialize)]
pub struct ReportAggregationLink {
    id: Uuid,
    name: String,
    metric_count: i64,
}

#[derive(Serialize)]
pub struct ReportChartLink {
    id: Uuid,
    name: String,
    chart_type: String,
    aggregation_id: Option<Uuid>,
    aggregation_name: Option<String>,
}

/// Creates an aggregation definition over an existing report.
pub async fn create_aggregation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateAggregationRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "aggregations:write").await?;
    require_text("aggregation name", &payload.name)?;
    require_report_exists(&state.pool, payload.report_id).await?;
    validate_aggregation_metrics(&payload.metrics)?;

    let mut tx = state.pool.begin().await?;
    let aggregation_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO aggregations (name, report_id, group_by_logical_key)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(payload.name)
    .bind(payload.report_id)
    .bind(payload.group_by_logical_key)
    .fetch_one(&mut *tx)
    .await?;

    insert_aggregation_metrics(&mut tx, aggregation_id, payload.metrics).await?;

    tx.commit().await?;

    Ok(Json(IdResponse { id: aggregation_id }))
}

/// Updates an aggregation definition and replaces its metrics.
pub async fn update_aggregation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(aggregation_id): Path<Uuid>,
    Json(payload): Json<CreateAggregationRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "aggregations:write").await?;
    require_aggregation_exists(&state.pool, aggregation_id).await?;
    require_text("aggregation name", &payload.name)?;
    require_report_exists(&state.pool, payload.report_id).await?;
    validate_aggregation_metrics(&payload.metrics)?;

    let mut tx = state.pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE aggregations
        SET name = $1, report_id = $2, group_by_logical_key = $3
        WHERE id = $4
        "#,
    )
    .bind(payload.name)
    .bind(payload.report_id)
    .bind(payload.group_by_logical_key)
    .bind(aggregation_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM aggregation_metrics WHERE aggregation_id = $1")
        .bind(aggregation_id)
        .execute(&mut *tx)
        .await?;
    insert_aggregation_metrics(&mut tx, aggregation_id, payload.metrics).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: aggregation_id }))
}

/// Deletes an aggregation definition.
pub async fn delete_aggregation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(aggregation_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "aggregations:write").await?;
    require_aggregation_exists(&state.pool, aggregation_id).await?;

    sqlx::query("DELETE FROM aggregations WHERE id = $1")
        .bind(aggregation_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(IdResponse { id: aggregation_id }))
}

/// Lists aggregation definitions for the reporting workbench.
pub async fn list_aggregations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<AggregationSummary>>> {
    auth::require_capability(&state.pool, &headers, "aggregations:read").await?;

    let rows = sqlx::query(
        r#"
        SELECT
            aggregations.id,
            aggregations.name,
            aggregations.report_id,
            reports.name AS report_name,
            aggregations.group_by_logical_key,
            COUNT(aggregation_metrics.id) AS metric_count
        FROM aggregations
        JOIN reports ON reports.id = aggregations.report_id
        LEFT JOIN aggregation_metrics
            ON aggregation_metrics.aggregation_id = aggregations.id
        GROUP BY aggregations.id, reports.name
        ORDER BY aggregations.created_at, aggregations.name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let aggregations = rows
        .into_iter()
        .map(|row| {
            Ok(AggregationSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                report_id: row.try_get("report_id")?,
                report_name: row.try_get("report_name")?,
                group_by_logical_key: row.try_get("group_by_logical_key")?,
                metric_count: row.try_get("metric_count")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(aggregations))
}

/// Returns an aggregation definition with its configured metrics.
pub async fn get_aggregation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(aggregation_id): Path<Uuid>,
) -> ApiResult<Json<AggregationDefinition>> {
    auth::require_capability(&state.pool, &headers, "aggregations:read").await?;

    let aggregation = sqlx::query(
        r#"
        SELECT
            aggregations.id,
            aggregations.name,
            aggregations.report_id,
            reports.name AS report_name,
            aggregations.group_by_logical_key
        FROM aggregations
        JOIN reports ON reports.id = aggregations.report_id
        WHERE aggregations.id = $1
        "#,
    )
    .bind(aggregation_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("aggregation {aggregation_id}")))?;

    let metric_rows = sqlx::query(
        r#"
        SELECT metric_key, source_logical_key, metric_kind, position
        FROM aggregation_metrics
        WHERE aggregation_id = $1
        ORDER BY position, metric_key
        "#,
    )
    .bind(aggregation_id)
    .fetch_all(&state.pool)
    .await?;

    let metrics = metric_rows
        .into_iter()
        .map(|row| {
            Ok(AggregationMetricSummary {
                metric_key: row.try_get("metric_key")?,
                source_logical_key: row.try_get("source_logical_key")?,
                metric_kind: row.try_get("metric_kind")?,
                position: row.try_get("position")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(AggregationDefinition {
        id: aggregation.try_get("id")?,
        name: aggregation.try_get("name")?,
        report_id: aggregation.try_get("report_id")?,
        report_name: aggregation.try_get("report_name")?,
        group_by_logical_key: aggregation.try_get("group_by_logical_key")?,
        metrics,
    }))
}

/// Runs an aggregation over report result rows.
pub async fn run_aggregation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(aggregation_id): Path<Uuid>,
) -> ApiResult<Json<AggregationResult>> {
    auth::require_capability(&state.pool, &headers, "aggregations:read").await?;

    let aggregation = sqlx::query(
        r#"
        SELECT id, report_id, group_by_logical_key
        FROM aggregations
        WHERE id = $1
        "#,
    )
    .bind(aggregation_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("aggregation {aggregation_id}")))?;
    let report_id: Uuid = aggregation.try_get("report_id")?;
    let group_by_logical_key: Option<String> = aggregation.try_get("group_by_logical_key")?;

    let metric_rows = sqlx::query(
        r#"
        SELECT metric_key, source_logical_key, metric_kind
        FROM aggregation_metrics
        WHERE aggregation_id = $1
        ORDER BY position, metric_key
        "#,
    )
    .bind(aggregation_id)
    .fetch_all(&state.pool)
    .await?;

    let report_rows = load_report_rows(&state.pool, report_id).await?;
    let mut submissions = BTreeMap::<String, HashMap<String, String>>::new();
    for row in report_rows {
        let (Some(submission_id), Some(logical_key), Some(field_value)) =
            (row.submission_id, row.logical_key, row.field_value)
        else {
            continue;
        };
        submissions
            .entry(submission_id)
            .or_default()
            .insert(logical_key, field_value);
    }

    let runtime_metrics = metric_rows
        .into_iter()
        .map(|metric| -> Result<RuntimeAggregationMetric, sqlx::Error> {
            Ok(RuntimeAggregationMetric {
                metric_key: metric.try_get("metric_key")?,
                source_logical_key: metric.try_get("source_logical_key")?,
                metric_kind: metric.try_get("metric_kind")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    for metric in &runtime_metrics {
        require_supported_aggregation_metric(
            &metric.metric_kind,
            metric.source_logical_key.as_deref(),
        )?;
    }

    let mut groups = BTreeMap::<String, BTreeMap<String, AggregationMetricAccumulator>>::new();
    for values in submissions.values() {
        let group_key = group_by_logical_key
            .as_ref()
            .and_then(|key| values.get(key))
            .cloned()
            .unwrap_or_else(|| "All".to_string());
        let metrics = groups.entry(group_key).or_default();

        for metric in &runtime_metrics {
            let accumulator = metrics.entry(metric.metric_key.clone()).or_default();

            match metric.metric_kind.as_str() {
                "count" => accumulator.increment_count(),
                "sum" | "avg" | "min" | "max" => {
                    let source_logical_key =
                        metric.source_logical_key.as_deref().ok_or_else(|| {
                            ApiError::BadRequest(format!(
                                "{} metrics require a source logical key",
                                metric.metric_kind
                            ))
                        })?;
                    if let Some(raw) = values.get(source_logical_key) {
                        accumulator.add_numeric(raw.parse::<f64>().map_err(|_| {
                            ApiError::BadRequest(format!(
                                "aggregation metric '{source_logical_key}' expected numeric values"
                            ))
                        })?);
                    }
                }
                other => {
                    return Err(ApiError::BadRequest(format!(
                        "unsupported aggregation metric kind '{other}'"
                    )));
                }
            }
        }
    }

    Ok(Json(AggregationResult {
        aggregation_id,
        rows: groups
            .into_iter()
            .map(|(group_key, accumulators)| {
                let mut metrics = BTreeMap::new();
                for metric in &runtime_metrics {
                    let value = if let Some(accumulator) = accumulators.get(&metric.metric_key) {
                        accumulator.finalize(&metric.metric_kind)?
                    } else {
                        AggregationMetricAccumulator::default().finalize(&metric.metric_kind)?
                    };
                    metrics.insert(metric.metric_key.clone(), value);
                }
                Ok(AggregationResultRow { group_key, metrics })
            })
            .collect::<ApiResult<Vec<_>>>()?,
    }))
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

    let aggregations = sqlx::query(
        r#"
        SELECT
            aggregations.id,
            aggregations.name,
            COUNT(aggregation_metrics.id) AS metric_count
        FROM aggregations
        LEFT JOIN aggregation_metrics
            ON aggregation_metrics.aggregation_id = aggregations.id
        WHERE aggregations.report_id = $1
        GROUP BY aggregations.id, aggregations.name
        ORDER BY aggregations.name, aggregations.id
        "#,
    )
    .bind(report_id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ReportAggregationLink {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            metric_count: row.try_get("metric_count")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let charts = sqlx::query(
        r#"
        SELECT
            charts.id,
            charts.name,
            charts.chart_type::text AS chart_type,
            charts.aggregation_id,
            aggregations.name AS aggregation_name
        FROM charts
        LEFT JOIN aggregations ON aggregations.id = charts.aggregation_id
        WHERE charts.report_id = $1
           OR charts.aggregation_id IN (
                SELECT id FROM aggregations WHERE report_id = $1
           )
        ORDER BY charts.name, charts.id
        "#,
    )
    .bind(report_id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ReportChartLink {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            chart_type: row.try_get("chart_type")?,
            aggregation_id: row.try_get("aggregation_id")?,
            aggregation_name: row.try_get("aggregation_name")?,
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
        aggregations,
        charts,
    }))
}

pub async fn run_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(report_id): Path<Uuid>,
) -> ApiResult<Json<ReportTable>> {
    auth::require_capability(&state.pool, &headers, "reports:read").await?;
    let rows = load_report_rows(&state.pool, report_id).await?;
    Ok(Json(ReportTable { report_id, rows }))
}

async fn load_report_rows(pool: &sqlx::PgPool, report_id: Uuid) -> ApiResult<Vec<ReportTableRow>> {
    require_report_exists(pool, report_id).await?;
    let report_dataset_id: Option<Uuid> =
        sqlx::query_scalar("SELECT dataset_id FROM reports WHERE id = $1")
            .bind(report_id)
            .fetch_one(pool)
            .await?;

    if let Some(dataset_id) = report_dataset_id {
        let composition_mode = require_executable_submission_dataset(pool, dataset_id).await?;
        if composition_mode == DatasetCompositionMode::Join {
            return load_join_dataset_report_rows(pool, report_id, dataset_id).await;
        }
    }

    let source_rows = load_report_source_rows(pool, report_id, report_dataset_id).await?;
    finalize_report_rows(source_rows).await
}

async fn load_report_source_rows(
    pool: &sqlx::PgPool,
    report_id: Uuid,
    report_dataset_id: Option<Uuid>,
) -> ApiResult<Vec<PgRow>> {
    if let Some(dataset_id) = report_dataset_id {
        assert_report_dataset_is_executable(pool, dataset_id).await?;
        sqlx::query(
            r#"
            WITH ranked_submissions AS (
                SELECT
                    dataset_sources.dataset_id,
                    dataset_sources.source_alias,
                    dataset_sources.selection_rule,
                    submission_fact.submission_id,
                    submission_fact.node_id,
                    node_dim.node_name,
                    ROW_NUMBER() OVER (
                        PARTITION BY dataset_sources.dataset_id, dataset_sources.source_alias, submission_fact.node_id
                        ORDER BY
                            CASE
                                WHEN dataset_sources.selection_rule = 'earliest' THEN submission_fact.submitted_at
                            END ASC NULLS LAST,
                            CASE
                                WHEN dataset_sources.selection_rule <> 'earliest' THEN submission_fact.submitted_at
                            END DESC NULLS LAST,
                            submission_fact.submission_id
                    ) AS selection_rank
                FROM reports
                JOIN dataset_sources
                    ON dataset_sources.dataset_id = reports.dataset_id
                JOIN analytics.form_version_dim
                    ON (
                        dataset_sources.form_id IS NOT NULL
                        AND form_version_dim.form_id = dataset_sources.form_id
                    )
                    OR (
                        dataset_sources.compatibility_group_id IS NOT NULL
                        AND form_version_dim.compatibility_group_id = dataset_sources.compatibility_group_id
                    )
                JOIN analytics.submission_fact
                    ON submission_fact.form_version_id = form_version_dim.form_version_id
                JOIN analytics.node_dim
                    ON node_dim.node_id = submission_fact.node_id
                WHERE reports.id = $1
                  AND submission_fact.status = 'submitted'
            )
            SELECT
                ranked_submissions.submission_id::text AS submission_id,
                ranked_submissions.node_name,
                ranked_submissions.source_alias,
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
            JOIN ranked_submissions
                ON ranked_submissions.dataset_id = reports.dataset_id
            JOIN report_field_bindings
                ON report_field_bindings.report_id = reports.id
            LEFT JOIN dataset_fields
                ON dataset_fields.dataset_id = reports.dataset_id
               AND dataset_fields.key = report_field_bindings.source_field_key
               AND dataset_fields.source_alias = ranked_submissions.source_alias
            LEFT JOIN analytics.submission_value_fact
                ON submission_value_fact.submission_id = ranked_submissions.submission_id
               AND submission_value_fact.field_key = dataset_fields.source_field_key
            WHERE reports.id = $1
              AND (
                ranked_submissions.selection_rule = 'all'
                OR ranked_submissions.selection_rank = 1
              )
              AND (
                report_field_bindings.computed_expression IS NOT NULL
                OR dataset_fields.id IS NOT NULL
              )
              AND (
                report_field_bindings.computed_expression IS NOT NULL
                OR
                submission_value_fact.value_text IS NOT NULL
                OR report_field_bindings.missing_policy::text <> 'exclude_row'
              )
            ORDER BY ranked_submissions.submission_id, report_field_bindings.position
            "#,
        )
        .bind(report_id)
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)
    } else {
        sqlx::query(
            r#"
            SELECT
                submission_fact.submission_id::text AS submission_id,
                node_dim.node_name,
                NULL::text AS source_alias,
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
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)
    }
}

async fn finalize_report_rows(source_rows: Vec<PgRow>) -> ApiResult<Vec<ReportTableRow>> {
    let mut submission_ids = Vec::with_capacity(source_rows.len());
    let mut node_names = Vec::with_capacity(source_rows.len());
    let mut source_aliases = Vec::with_capacity(source_rows.len());
    let mut logical_keys = Vec::with_capacity(source_rows.len());
    let mut field_values = Vec::with_capacity(source_rows.len());

    for row in source_rows {
        submission_ids.push(row.try_get::<String, _>("submission_id").ok());
        node_names.push(row.try_get::<String, _>("node_name").ok());
        source_aliases.push(row.try_get::<Option<String>, _>("source_alias")?);
        logical_keys.push(row.try_get::<String, _>("logical_key").ok());
        field_values.push(row.try_get::<Option<String>, _>("field_value")?);
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("submission_id", DataType::Utf8, true),
        Field::new("node_name", DataType::Utf8, true),
        Field::new("source_alias", DataType::Utf8, true),
        Field::new("logical_key", DataType::Utf8, true),
        Field::new("field_value", DataType::Utf8, true),
    ]));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(submission_ids)) as ArrayRef,
            Arc::new(StringArray::from(node_names)) as ArrayRef,
            Arc::new(StringArray::from(source_aliases)) as ArrayRef,
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
            SELECT submission_id, node_name, source_alias, logical_key, field_value
            FROM report_values
            ORDER BY submission_id, source_alias, logical_key
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
        let source_aliases = as_string_array(&batch, 2)?;
        let logical_keys = as_string_array(&batch, 3)?;
        let field_values = as_string_array(&batch, 4)?;

        for index in 0..batch.num_rows() {
            rows.push(ReportTableRow {
                submission_id: string_value(submission_ids, index),
                node_name: string_value(node_names, index),
                source_alias: string_value(source_aliases, index),
                logical_key: string_value(logical_keys, index),
                field_value: string_value(field_values, index),
            });
        }
    }

    Ok(rows)
}

async fn load_join_dataset_report_rows(
    pool: &sqlx::PgPool,
    report_id: Uuid,
    dataset_id: Uuid,
) -> ApiResult<Vec<ReportTableRow>> {
    let dataset_rows = load_dataset_table_rows(pool, dataset_id).await?;
    let binding_rows = sqlx::query(
        r#"
        SELECT logical_key, source_field_key, computed_expression, missing_policy::text AS missing_policy
        FROM report_field_bindings
        WHERE report_id = $1
        ORDER BY position, logical_key
        "#,
    )
    .bind(report_id)
    .fetch_all(pool)
    .await?;

    let bindings = binding_rows
        .into_iter()
        .map(|row| -> Result<RuntimeReportBinding, sqlx::Error> {
            Ok(RuntimeReportBinding {
                logical_key: row.try_get("logical_key")?,
                source_field_key: row.try_get("source_field_key")?,
                computed_expression: row.try_get("computed_expression")?,
                missing_policy: row.try_get("missing_policy")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let mut rows = Vec::new();
    for dataset_row in dataset_rows {
        append_join_dataset_report_rows(&mut rows, &dataset_row, &bindings);
    }

    Ok(rows)
}

fn append_join_dataset_report_rows(
    rows: &mut Vec<ReportTableRow>,
    dataset_row: &DatasetTableRow,
    bindings: &[RuntimeReportBinding],
) {
    for binding in bindings {
        let field_value = resolve_join_dataset_binding_value(dataset_row, binding);
        if field_value.is_none()
            && binding.computed_expression.is_none()
            && binding.missing_policy == "exclude_row"
        {
            continue;
        }

        rows.push(ReportTableRow {
            submission_id: Some(dataset_row.submission_id.clone()),
            node_name: Some(dataset_row.node_name.clone()),
            source_alias: Some(dataset_row.source_alias.clone()),
            logical_key: Some(binding.logical_key.clone()),
            field_value,
        });
    }
}

fn resolve_join_dataset_binding_value(
    dataset_row: &DatasetTableRow,
    binding: &RuntimeReportBinding,
) -> Option<String> {
    if let Some(computed_expression) = &binding.computed_expression {
        return literal_expression_value(computed_expression);
    }

    match binding
        .source_field_key
        .as_ref()
        .and_then(|key| dataset_row.values.get(key).cloned().flatten())
    {
        Some(value) => Some(value),
        None if binding.missing_policy == "bucket_unknown" => Some("Unknown".to_string()),
        None => None,
    }
}

fn literal_expression_value(expression: &str) -> Option<String> {
    expression.strip_prefix("literal:").map(ToString::to_string)
}

async fn insert_aggregation_metrics(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    aggregation_id: Uuid,
    metrics: Vec<CreateAggregationMetricRequest>,
) -> ApiResult<()> {
    for (position, metric) in metrics.into_iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO aggregation_metrics
                (aggregation_id, metric_key, source_logical_key, metric_kind, position)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(aggregation_id)
        .bind(metric.metric_key)
        .bind(metric.source_logical_key)
        .bind(metric.metric_kind)
        .bind(position as i32)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn require_aggregation_exists(pool: &sqlx::PgPool, aggregation_id: Uuid) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM aggregations WHERE id = $1)")
            .bind(aggregation_id)
            .fetch_one(pool)
            .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("aggregation {aggregation_id}")))
    }
}

fn validate_aggregation_metrics(metrics: &[CreateAggregationMetricRequest]) -> ApiResult<()> {
    if metrics.is_empty() {
        return Err(ApiError::BadRequest(
            "an aggregation requires at least one metric".into(),
        ));
    }

    for metric in metrics {
        require_text("aggregation metric key", &metric.metric_key)?;
        require_supported_aggregation_metric(
            &metric.metric_kind,
            metric.source_logical_key.as_deref(),
        )?;
    }

    Ok(())
}

fn require_supported_aggregation_metric(
    metric_kind: &str,
    source_logical_key: Option<&str>,
) -> ApiResult<()> {
    match metric_kind {
        "count" => Ok(()),
        "sum" | "avg" | "min" | "max" => {
            if source_logical_key
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
            {
                Err(ApiError::BadRequest(format!(
                    "{metric_kind} metrics require a source logical key"
                )))
            } else {
                Ok(())
            }
        }
        other => Err(ApiError::BadRequest(format!(
            "unsupported aggregation metric kind '{other}'"
        ))),
    }
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
    require_executable_submission_dataset(pool, dataset_id)
        .await
        .map(|_| ())
        .map_err(|error| match error {
            ApiError::BadRequest(message) if message.contains("submission grain") => {
                ApiError::BadRequest(
                    "dataset-backed reports currently support only submission grain".into(),
                )
            }
            ApiError::BadRequest(message)
                if message.contains("form or compatibility-group sources")
                    || message.contains("selection rules")
                    || message.contains("at least two sources") =>
            {
                ApiError::BadRequest(
                    "dataset-backed reports currently require executable dataset sources with supported selection rules".into(),
                )
            }
            other => other,
        })
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
    use std::collections::BTreeMap;

    use arrow::array::StringArray;

    use crate::datasets::DatasetTableRow;

    use super::{
        RuntimeReportBinding, literal_expression_value, resolve_join_dataset_binding_value,
        string_value,
    };

    #[test]
    fn string_value_preserves_nulls_from_arrow_arrays() {
        let values = StringArray::from(vec![Some("North"), None]);

        assert_eq!(string_value(&values, 0), Some("North".to_string()));
        assert_eq!(string_value(&values, 1), None);
    }

    #[test]
    fn literal_expression_value_extracts_literal_payloads() {
        assert_eq!(
            literal_expression_value("literal:Submitted"),
            Some("Submitted".to_string())
        );
        assert_eq!(literal_expression_value("sum:value"), None);
    }

    #[test]
    fn join_dataset_binding_value_uses_dataset_rows_and_missing_policies() {
        let dataset_row = DatasetTableRow {
            submission_id: "check_in:1 | follow_up:2".to_string(),
            node_name: "Demo Organization".to_string(),
            source_alias: "join".to_string(),
            values: BTreeMap::from([
                ("participant_count".to_string(), Some("42".to_string())),
                ("attendee_count".to_string(), None),
            ]),
        };

        let direct_binding = RuntimeReportBinding {
            logical_key: "participants".to_string(),
            source_field_key: Some("participant_count".to_string()),
            computed_expression: None,
            missing_policy: "null".to_string(),
        };
        let missing_binding = RuntimeReportBinding {
            logical_key: "attendees".to_string(),
            source_field_key: Some("attendee_count".to_string()),
            computed_expression: None,
            missing_policy: "bucket_unknown".to_string(),
        };
        let computed_binding = RuntimeReportBinding {
            logical_key: "status".to_string(),
            source_field_key: None,
            computed_expression: Some("literal:Submitted".to_string()),
            missing_policy: "null".to_string(),
        };

        assert_eq!(
            resolve_join_dataset_binding_value(&dataset_row, &direct_binding),
            Some("42".to_string())
        );
        assert_eq!(
            resolve_join_dataset_binding_value(&dataset_row, &missing_binding),
            Some("Unknown".to_string())
        );
        assert_eq!(
            resolve_join_dataset_binding_value(&dataset_row, &computed_binding),
            Some("Submitted".to_string())
        );
    }
}
