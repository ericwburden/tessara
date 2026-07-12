//! Runtime execution, aggregation, filtering, and presentation transforms for Components.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::{DateTime, NaiveDate, NaiveDateTime};
use sqlx::{Column, Row};
use tessara_data_ops::{DataField, FieldType, FilterOperator};
use uuid::Uuid;

use super::{
    ComponentFilterConfig, ComponentRuntimeQuery, ComponentSortConfig, ComponentStatValue,
    ComponentTable, ComponentTableColumn, ComponentTablePagination, ComponentTableRow,
    ComponentVersionForTable, ComponentVisual, ComponentVisualPoint, ComponentVisualSlice,
    MajorLineMaterialization, TableComponentConfig, VisualComponentConfig, VisualSharedConfig,
    component_data_op_error, load_dataset_major_line_fields, require_component_field_ref,
    require_component_kind, validate_component_config, validate_component_type,
};
use crate::{
    auth,
    error::{ApiError, ApiResult},
};

pub(super) async fn execute_component_table(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    version: ComponentVersionForTable,
    query: ComponentRuntimeQuery,
) -> ApiResult<ComponentTable> {
    let fields =
        load_dataset_major_line_fields(pool, version.dataset_id, version.dataset_version_major)
            .await?;
    require_component_kind(&version.component_type, "table", "table endpoint")?;
    validate_component_config(&version.component_type, &version.config, &fields)?;
    let Some(materialization) =
        load_major_line_materialization(pool, version.dataset_id, version.dataset_version_major)
            .await?
    else {
        return Ok(empty_component_table(version, "pending", Vec::new()));
    };
    if materialization.state != "ready" {
        return Ok(empty_component_table(
            version,
            &materialization.state,
            Vec::new(),
        ));
    }
    execute_table_component(pool, account, version, materialization, &fields, query).await
}

async fn load_major_line_materialization(
    pool: &sqlx::PgPool,
    dataset_id: Uuid,
    version_major: i32,
) -> ApiResult<Option<MajorLineMaterialization>> {
    let row = sqlx::query(
        r#"
        SELECT materialized_schema, materialized_table, rebuild_status
        FROM dataset_major_materializations
        WHERE dataset_id = $1
          AND version_major = $2
        "#,
    )
    .bind(dataset_id)
    .bind(version_major)
    .fetch_optional(pool)
    .await?;
    if let Some(row) = row {
        Ok(Some(MajorLineMaterialization {
            schema: row.try_get("materialized_schema")?,
            table: row.try_get("materialized_table")?,
            state: row.try_get("rebuild_status")?,
        }))
    } else {
        Ok(None)
    }
}

pub(super) fn empty_component_table(
    version: ComponentVersionForTable,
    materialization_state: &str,
    columns: Vec<ComponentTableColumn>,
) -> ComponentTable {
    ComponentTable {
        component_id: version.component_id,
        component_version_id: version.id,
        dataset_id: version.dataset_id,
        dataset_version_major: version.dataset_version_major,
        component_type: version.component_type,
        materialization_state: materialization_state.into(),
        columns,
        rows: Vec::new(),
        pagination: ComponentTablePagination {
            page_size: 0,
            next_cursor: None,
            has_more: false,
        },
    }
}

async fn execute_table_component(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    version: ComponentVersionForTable,
    materialization: MajorLineMaterialization,
    fields: &[DataField],
    query: ComponentRuntimeQuery,
) -> ApiResult<ComponentTable> {
    validate_component_type(&version.component_type)?;
    let config: TableComponentConfig =
        serde_json::from_value(version.config.clone()).map_err(|error| {
            ApiError::BadRequest(format!("table component config is invalid: {error}"))
        })?;
    let field_refs = fields.iter().collect::<Vec<_>>();
    let configured_visible_columns = config
        .visible_columns
        .iter()
        .map(|column| column.field_key().to_string())
        .collect::<Vec<_>>();
    let component_contract_fields = visible_table_fields(&field_refs, &configured_visible_columns)?;
    let component_contract_refs = component_contract_fields.to_vec();
    let selected_fields = visible_table_fields(&component_contract_refs, &query.visible_columns)?;
    let columns = selected_fields
        .iter()
        .map(|field| component_table_column(field, &config.display_labels))
        .collect::<Vec<_>>();
    let select_columns = selected_fields
        .iter()
        .map(|field| quote_identifier(&field.key))
        .collect::<Vec<_>>();
    let mut predicates =
        vec![tier_access_predicate_for_materialization(pool, account, &materialization).await?];
    predicates.extend(component_filter_sql(&config.filters, &field_refs)?);
    predicates.extend(component_filter_sql(
        &query.filters,
        &component_contract_fields,
    )?);
    if let Some(search) = query.search.as_deref() {
        let search_fields = table_search_fields(&config, &component_contract_fields)?;
        if !search_fields.is_empty() {
            predicates.push(search_predicate_sql(&search_fields, search));
        }
    }
    let full_name = materialized_full_name(&materialization);
    let sort = query.sort.or(config.default_sort);
    let order_by = table_order_by_sql(sort.as_ref(), &component_contract_refs, "__row_id")?;
    let page_size = effective_component_page_size(query.page_size, config.page_size);
    let page = component_pagination_sql(query.offset, page_size);
    let sql = format!(
        "SELECT __row_id, {} FROM {full_name} WHERE {}{order_by}{page}",
        select_columns.join(", "),
        predicates.join(" AND ")
    );
    let (rows, pagination) = component_rows_from_query(pool, &sql, query.offset, page_size).await?;
    Ok(ComponentTable {
        component_id: version.component_id,
        component_version_id: version.id,
        dataset_id: version.dataset_id,
        dataset_version_major: version.dataset_version_major,
        component_type: version.component_type,
        materialization_state: "ready".into(),
        columns,
        rows,
        pagination,
    })
}

pub(super) async fn execute_component_visual(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    version: ComponentVersionForTable,
    visual_kind: &'static str,
    source_row_limit: Option<usize>,
) -> ApiResult<ComponentVisual> {
    require_component_kind(&version.component_type, visual_kind, "visual endpoint")?;
    let fields =
        load_dataset_major_line_fields(pool, version.dataset_id, version.dataset_version_major)
            .await?;
    validate_component_config(&version.component_type, &version.config, &fields)?;
    let config = VisualComponentConfig::parse(&version.component_type, &version.config)?;
    let Some(materialization) =
        load_major_line_materialization(pool, version.dataset_id, version.dataset_version_major)
            .await?
    else {
        return Ok(empty_component_visual(
            version,
            config.value_format(),
            "pending",
        ));
    };
    if materialization.state != "ready" {
        let state = materialization.state.clone();
        return Ok(empty_component_visual(
            version,
            config.value_format(),
            &state,
        ));
    }
    let rows = component_visual_aggregated_rows(
        pool,
        account,
        &materialization,
        &config,
        &fields,
        source_row_limit,
    )
    .await?;
    visual_from_aggregated_rows(version, config, rows, &fields)
}

fn empty_component_visual(
    version: ComponentVersionForTable,
    value_format: String,
    materialization_state: &str,
) -> ComponentVisual {
    ComponentVisual {
        component_id: version.component_id,
        component_version_id: version.id,
        dataset_id: version.dataset_id,
        dataset_version_major: version.dataset_version_major,
        component_type: version.component_type,
        materialization_state: materialization_state.into(),
        value_format,
        legend_title: None,
        bar_orientation: None,
        bar_comparison_layout: None,
        x_axis_label: None,
        y_axis_label: None,
        line_smoothing: None,
        stat: None,
        points: Vec::new(),
        slices: Vec::new(),
    }
}

async fn component_visual_aggregated_rows(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    materialization: &MajorLineMaterialization,
    config: &VisualComponentConfig,
    fields: &[DataField],
    source_row_limit: Option<usize>,
) -> ApiResult<Vec<BTreeMap<String, Option<String>>>> {
    let mut keys = config.referenced_fields();
    keys.sort();
    keys.dedup();
    let mut select_columns = keys
        .iter()
        .map(|field| quote_identifier(field))
        .collect::<Vec<_>>();
    select_columns.push(quote_identifier("__row_id"));
    select_columns.sort();
    select_columns.dedup();
    let field_refs = fields.iter().collect::<Vec<_>>();
    let mut predicates =
        vec![tier_access_predicate_for_materialization(pool, account, materialization).await?];
    predicates.extend(component_filter_sql(&config.shared().filters, &field_refs)?);
    let full_name = materialized_full_name(materialization);
    let limit_clause = component_visual_source_limit_clause(source_row_limit);
    let source_sql = format!(
        "SELECT {} FROM {full_name} WHERE {} ORDER BY {}{limit_clause}",
        select_columns.join(", "),
        predicates.join(" AND "),
        quote_identifier("__row_id")
    );
    let grouping = visual_grouping_contract(config);
    let aggregate = visual_aggregate_sql(config.shared());
    let (sql, dimension_field, comparison_field) = if let Some(grouping) = grouping {
        let dimension =
            visual_dimension_sql(grouping.dimension_field, grouping.dimension_missing_policy);
        let comparison = grouping
            .comparison_field
            .map(|(field, missing_policy)| (field, visual_dimension_sql(field, missing_policy)));
        let mut select = vec![format!("{} AS __component_dimension", dimension.expression)];
        let mut group_by = vec!["1"];
        let mut group_filters = dimension.filters;
        if let Some((_, comparison)) = &comparison {
            select.push(format!(
                "{} AS __component_comparison",
                comparison.expression
            ));
            group_by.push("2");
            group_filters.extend(comparison.filters.clone());
        }
        select.push(format!("{aggregate} AS __component_value"));
        select.push("COUNT(*)::bigint AS __component_source_count".into());
        let group_filter_sql = if group_filters.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", group_filters.join(" AND "))
        };
        let having = visual_aggregate_having_sql(config.shared(), &aggregate);
        (
            format!(
                "SELECT {} FROM ({source_sql}) AS source_rows{group_filter_sql} GROUP BY {}{having} ORDER BY {}",
                select.join(", "),
                group_by.join(", "),
                group_by.join(", ")
            ),
            Some(grouping.dimension_field),
            comparison.map(|(field, _)| field),
        )
    } else {
        (
            format!(
                "SELECT {aggregate} AS __component_value, COUNT(*)::bigint AS __component_source_count FROM ({source_sql}) AS source_rows"
            ),
            None,
            None,
        )
    };
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    rows.into_iter()
        .map(|row| {
            let source_count: i64 = row.try_get("__component_source_count")?;
            if config.shared().summary_type == "none" && source_count != 1 {
                let context = dimension_field
                    .and_then(|_| row.try_get::<String, _>("__component_dimension").ok())
                    .map(|dimension| format!("category '{dimension}'"))
                    .unwrap_or_else(|| "stat card".into());
                return Err(ApiError::BadRequest(format!(
                    "Do not summarize requires exactly one row for {context}; found {source_count} rows"
                )));
            }
            let mut values = BTreeMap::new();
            if let Some(field) = dimension_field {
                values.insert(
                    field.to_string(),
                    Some(row.try_get::<String, _>("__component_dimension")?),
                );
            }
            if let Some(field) = comparison_field {
                values.insert(
                    field.to_string(),
                    Some(row.try_get::<String, _>("__component_comparison")?),
                );
            }
            let value = row
                .try_get::<Option<f64>, _>("__component_value")?
                .map(|value| value.to_string());
            values.insert("__component_value".into(), value);
            Ok(values)
        })
        .collect()
}

struct VisualGroupingContract<'a> {
    dimension_field: &'a str,
    dimension_missing_policy: &'a str,
    comparison_field: Option<(&'a str, &'a str)>,
}

struct VisualDimensionSql {
    expression: String,
    filters: Vec<String>,
}

fn visual_grouping_contract(config: &VisualComponentConfig) -> Option<VisualGroupingContract<'_>> {
    match config {
        VisualComponentConfig::StatCard(_) => None,
        VisualComponentConfig::Bar(config) => Some(VisualGroupingContract {
            dimension_field: &config.category_field,
            dimension_missing_policy: config
                .category_missing_policy
                .as_deref()
                .unwrap_or(&config.shared.missing_policy),
            comparison_field: config.comparison_field.as_deref().map(|field| {
                (
                    field,
                    config
                        .comparison_missing_policy
                        .as_deref()
                        .unwrap_or(&config.shared.missing_policy),
                )
            }),
        }),
        VisualComponentConfig::Line(config) => Some(VisualGroupingContract {
            dimension_field: &config.x_field,
            dimension_missing_policy: config
                .x_missing_policy
                .as_deref()
                .unwrap_or(&config.shared.missing_policy),
            comparison_field: None,
        }),
        VisualComponentConfig::Pie(config) | VisualComponentConfig::Donut(config) => {
            Some(VisualGroupingContract {
                dimension_field: &config.category_field,
                dimension_missing_policy: config
                    .category_missing_policy
                    .as_deref()
                    .unwrap_or(&config.shared.missing_policy),
                comparison_field: None,
            })
        }
    }
}

fn visual_dimension_sql(field: &str, missing_policy: &str) -> VisualDimensionSql {
    let field = quote_identifier(field);
    let normalized = format!("NULLIF(BTRIM({field}), '')");
    if missing_policy == "explicit_missing" {
        VisualDimensionSql {
            expression: format!("COALESCE({normalized}, '(Missing)')"),
            filters: Vec::new(),
        }
    } else {
        VisualDimensionSql {
            expression: normalized.clone(),
            filters: vec![format!("{normalized} IS NOT NULL")],
        }
    }
}

fn visual_aggregate_sql(shared: &VisualSharedConfig) -> String {
    if shared.summary_type == "row_count" {
        return "COUNT(*)::double precision".into();
    }
    let field = quote_identifier(&shared.summary_field);
    let normalized = format!("NULLIF(BTRIM({field}), '')");
    let missing_policy = shared
        .value_missing_policy
        .as_deref()
        .unwrap_or(&shared.missing_policy);
    match shared.summary_type.as_str() {
        "count" if missing_policy == "omit" => {
            format!("COUNT({normalized})::double precision")
        }
        "count" => "COUNT(*)::double precision".into(),
        "unique_count" if missing_policy == "explicit_missing" => format!(
            "COUNT(DISTINCT CASE WHEN {normalized} IS NULL THEN JSONB_BUILD_ARRAY('missing') ELSE JSONB_BUILD_ARRAY('value', {normalized}) END)::double precision"
        ),
        "unique_count" => format!("COUNT(DISTINCT {normalized})::double precision"),
        summary_type => {
            let numeric = format!("({normalized})::double precision");
            let numeric = if missing_policy == "zero" {
                format!("COALESCE({numeric}, 0::double precision)")
            } else {
                numeric
            };
            match summary_type {
                "sum" => format!("SUM({numeric})"),
                "average" => format!("AVG({numeric})"),
                "median" => {
                    format!("PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY {numeric})")
                }
                "none" => format!("MAX({numeric})"),
                _ => "NULL::double precision".into(),
            }
        }
    }
}

fn visual_aggregate_having_sql(shared: &VisualSharedConfig, aggregate: &str) -> String {
    let missing_policy = shared
        .value_missing_policy
        .as_deref()
        .unwrap_or(&shared.missing_policy);
    if matches!(shared.summary_type.as_str(), "count" | "unique_count") && missing_policy == "omit"
    {
        format!(" HAVING {aggregate} > 0")
    } else if matches!(shared.summary_type.as_str(), "sum" | "average" | "median")
        && missing_policy != "zero"
    {
        format!(" HAVING {aggregate} IS NOT NULL")
    } else {
        String::new()
    }
}

pub(super) fn component_visual_source_limit_clause(source_row_limit: Option<usize>) -> String {
    source_row_limit
        .map(|limit| format!(" LIMIT {}", limit.max(1)))
        .unwrap_or_default()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn visual_from_rows(
    version: ComponentVersionForTable,
    config: VisualComponentConfig,
    rows: Vec<BTreeMap<String, Option<String>>>,
    fields: &[DataField],
) -> ApiResult<ComponentVisual> {
    visual_from_rows_with_mode(version, config, rows, fields, false)
}

fn visual_from_aggregated_rows(
    version: ComponentVersionForTable,
    config: VisualComponentConfig,
    rows: Vec<BTreeMap<String, Option<String>>>,
    fields: &[DataField],
) -> ApiResult<ComponentVisual> {
    visual_from_rows_with_mode(version, config, rows, fields, true)
}

fn visual_from_rows_with_mode(
    version: ComponentVersionForTable,
    config: VisualComponentConfig,
    rows: Vec<BTreeMap<String, Option<String>>>,
    fields: &[DataField],
    preaggregated: bool,
) -> ApiResult<ComponentVisual> {
    let value_format = config.value_format();
    let mut visual = empty_component_visual(version, value_format.clone(), "ready");
    match config {
        VisualComponentConfig::StatCard(config) => {
            let calculation = visual_calculation_shared(&config.shared, preaggregated);
            let values = summary_values(&rows, &calculation, None, None);
            let value = summarize_values_checked(&values, &calculation.summary_type, "stat card")?;
            visual.stat = Some(ComponentStatValue {
                label: config.label.unwrap_or_else(|| {
                    if config.shared.summary_type == "row_count" {
                        "Row count".into()
                    } else {
                        sentence_label(&config.shared.summary_field)
                    }
                }),
                display_value: value.map(|value| format_visual_value(value, &value_format)),
                value,
                supporting_text: config.supporting_text,
                panel_style: config.panel_style,
            });
        }
        VisualComponentConfig::Bar(config) => {
            let calculation = visual_calculation_shared(&config.shared, preaggregated);
            let category_field_type = visual_field_type(fields, &config.category_field)?;
            let comparison_field_type = config
                .comparison_field
                .as_deref()
                .map(|field| visual_field_type(fields, field))
                .transpose()?;
            visual.legend_title = config.legend_title.clone();
            visual.bar_orientation = Some(config.orientation.clone());
            visual.bar_comparison_layout = Some(config.comparison_layout.clone());
            visual.x_axis_label = config.x_axis_label.clone();
            visual.y_axis_label = config.y_axis_label.clone();
            visual.points = if config.mode == "comparison" {
                comparison_display_points(
                    grouped_points(
                        &rows,
                        &calculation,
                        VisualGrouping {
                            dimension_field: &config.category_field,
                            dimension_field_type: category_field_type,
                            dimension_missing_policy: config
                                .category_missing_policy
                                .as_deref()
                                .unwrap_or(&config.shared.missing_policy),
                            comparison_field: config.comparison_field.as_deref(),
                            comparison_field_type,
                            comparison_missing_policy: config
                                .comparison_missing_policy
                                .as_deref()
                                .unwrap_or(&config.shared.missing_policy),
                            limit: config.number_of_points,
                            limit_by_category: true,
                            category_labels: &BTreeMap::new(),
                            category_colors: &BTreeMap::new(),
                        },
                    )?,
                    &config.category_labels,
                    &config.category_colors,
                )
            } else {
                grouped_points(
                    &rows,
                    &calculation,
                    VisualGrouping {
                        dimension_field: &config.category_field,
                        dimension_field_type: category_field_type,
                        dimension_missing_policy: config
                            .category_missing_policy
                            .as_deref()
                            .unwrap_or(&config.shared.missing_policy),
                        comparison_field: None,
                        comparison_field_type: None,
                        comparison_missing_policy: &config.shared.missing_policy,
                        limit: config.number_of_points,
                        limit_by_category: false,
                        category_labels: &BTreeMap::new(),
                        category_colors: &BTreeMap::new(),
                    },
                )?
            };
        }
        VisualComponentConfig::Line(config) => {
            let calculation = visual_calculation_shared(&config.shared, preaggregated);
            let x_field_type = visual_field_type(fields, &config.x_field)?;
            visual.line_smoothing = Some(config.smoothing);
            visual.points = grouped_points(
                &rows,
                &calculation,
                VisualGrouping {
                    dimension_field: &config.x_field,
                    dimension_field_type: x_field_type,
                    dimension_missing_policy: config
                        .x_missing_policy
                        .as_deref()
                        .unwrap_or(&config.shared.missing_policy),
                    comparison_field: None,
                    comparison_field_type: None,
                    comparison_missing_policy: &config.shared.missing_policy,
                    limit: config.number_of_points,
                    limit_by_category: false,
                    category_labels: &BTreeMap::new(),
                    category_colors: &BTreeMap::new(),
                },
            )?;
        }
        VisualComponentConfig::Pie(config) | VisualComponentConfig::Donut(config) => {
            let calculation = visual_calculation_shared(&config.shared, preaggregated);
            let category_field_type = visual_field_type(fields, &config.category_field)?;
            visual.legend_title = config.legend_title.clone();
            let points = grouped_points(
                &rows,
                &calculation,
                VisualGrouping {
                    dimension_field: &config.category_field,
                    dimension_field_type: category_field_type,
                    dimension_missing_policy: config
                        .category_missing_policy
                        .as_deref()
                        .unwrap_or(&config.shared.missing_policy),
                    comparison_field: None,
                    comparison_field_type: None,
                    comparison_missing_policy: &config.shared.missing_policy,
                    limit: config.max_slices,
                    limit_by_category: false,
                    category_labels: &config.category_labels,
                    category_colors: &config.category_colors,
                },
            )?;
            if points.iter().any(|point| point.value < 0.0) {
                return Err(ApiError::BadRequest(
                    "pie and donut charts do not support negative summarized values".into(),
                ));
            }
            visual.slices = points
                .into_iter()
                .map(|point| ComponentVisualSlice {
                    category: point.x,
                    value: point.value,
                    display_value: point.display_value,
                    color: point.color,
                })
                .collect();
        }
    }
    Ok(visual)
}

fn visual_calculation_shared(
    shared: &VisualSharedConfig,
    preaggregated: bool,
) -> VisualSharedConfig {
    let mut calculation = shared.clone();
    if preaggregated {
        calculation.summary_field = "__component_value".into();
        calculation.summary_type = "none".into();
        calculation.value_missing_policy = Some("omit".into());
    }
    calculation
}

struct VisualGrouping<'a> {
    dimension_field: &'a str,
    dimension_field_type: &'a FieldType,
    dimension_missing_policy: &'a str,
    comparison_field: Option<&'a str>,
    comparison_field_type: Option<&'a FieldType>,
    comparison_missing_policy: &'a str,
    limit: usize,
    limit_by_category: bool,
    category_labels: &'a BTreeMap<String, String>,
    category_colors: &'a BTreeMap<String, String>,
}

fn grouped_points(
    rows: &[BTreeMap<String, Option<String>>],
    shared: &VisualSharedConfig,
    grouping: VisualGrouping<'_>,
) -> ApiResult<Vec<ComponentVisualPoint>> {
    let mut groups = BTreeMap::<(String, Option<String>), Vec<VisualSummaryValue>>::new();
    for row in rows {
        let Some(dimension) = visual_dimension_value(
            row,
            grouping.dimension_field,
            grouping.dimension_missing_policy,
        ) else {
            continue;
        };
        let comparison = match grouping.comparison_field {
            Some(field) => {
                match visual_dimension_value(row, field, grouping.comparison_missing_policy) {
                    Some(value) => Some(value),
                    None => continue,
                }
            }
            None => None,
        };
        let values = summary_values(
            std::slice::from_ref(row),
            shared,
            Some(&dimension),
            comparison.as_deref(),
        );
        if values.is_empty() {
            continue;
        }
        groups
            .entry((dimension, comparison))
            .or_default()
            .extend(values);
    }
    let mut points = Vec::new();
    for ((x, comparison), values) in groups {
        let context = comparison
            .as_deref()
            .map(|series| format!("category '{x}' and series '{series}'"))
            .unwrap_or_else(|| format!("category '{x}'"));
        if let Some(value) = summarize_values_checked(&values, &shared.summary_type, &context)? {
            points.push(ComponentVisualPoint {
                color: grouping.category_colors.get(&x).cloned(),
                x: grouping
                    .category_labels
                    .get(&x)
                    .cloned()
                    .unwrap_or_else(|| x.clone()),
                sort_x: x,
                value,
                display_value: format_visual_value(value, &shared.value_format),
                sort_comparison: comparison.clone(),
                comparison,
            });
        }
    }
    if grouping.limit_by_category {
        sort_and_limit_comparison_points(
            &mut points,
            shared,
            grouping.limit,
            grouping.dimension_field_type,
            grouping.comparison_field_type,
        );
    } else {
        sort_visual_points(&mut points, shared, grouping.dimension_field_type);
        points.truncate(grouping.limit);
    }
    Ok(points)
}

fn sort_and_limit_comparison_points(
    points: &mut Vec<ComponentVisualPoint>,
    shared: &VisualSharedConfig,
    limit: usize,
    dimension_field_type: &FieldType,
    comparison_field_type: Option<&FieldType>,
) {
    let totals = points
        .iter()
        .fold(BTreeMap::<String, f64>::new(), |mut totals, point| {
            *totals.entry(point.sort_x.clone()).or_default() += point.value;
            totals
        });
    let sort_field = shared.sort_field.as_deref().unwrap_or("category");
    let descending = shared.sort_direction == "desc";
    let mut categories = totals.keys().cloned().collect::<Vec<_>>();
    categories.sort_by(|left, right| {
        let order = match sort_field {
            "summary_value" => totals
                .get(left)
                .copied()
                .unwrap_or_default()
                .total_cmp(&totals.get(right).copied().unwrap_or_default()),
            _ => compare_dimension_values(left, right, dimension_field_type),
        };
        if descending && sort_field != "comparison" {
            order.reverse()
        } else {
            order
        }
    });
    categories.truncate(limit);
    let category_rank = categories
        .iter()
        .enumerate()
        .map(|(index, category)| (category.clone(), index))
        .collect::<BTreeMap<_, _>>();
    points.retain(|point| category_rank.contains_key(&point.sort_x));
    points.sort_by(|left, right| {
        category_rank[&left.sort_x]
            .cmp(&category_rank[&right.sort_x])
            .then_with(|| {
                let order = compare_optional_dimension_values(
                    left.sort_comparison.as_deref(),
                    right.sort_comparison.as_deref(),
                    comparison_field_type,
                );
                if descending && sort_field == "comparison" {
                    order.reverse()
                } else {
                    order
                }
            })
    });
}

fn comparison_display_points(
    points: Vec<ComponentVisualPoint>,
    comparison_labels: &BTreeMap<String, String>,
    comparison_colors: &BTreeMap<String, String>,
) -> Vec<ComponentVisualPoint> {
    points
        .into_iter()
        .map(|mut point| {
            if let Some(comparison) = point.comparison.clone() {
                point.color = comparison_colors.get(&comparison).cloned();
                point.comparison = Some(
                    comparison_labels
                        .get(&comparison)
                        .cloned()
                        .unwrap_or(comparison),
                );
            }
            point
        })
        .collect()
}

fn visual_dimension_value(
    row: &BTreeMap<String, Option<String>>,
    field: &str,
    missing_policy: &str,
) -> Option<String> {
    match row.get(field).and_then(|value| value.as_deref()) {
        Some(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
        _ if missing_policy == "explicit_missing" => Some("(Missing)".into()),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
enum VisualSummaryValue {
    Numeric(Option<f64>),
    Distinct(Option<String>),
}

impl VisualSummaryValue {
    fn numeric(&self) -> Option<f64> {
        match self {
            Self::Numeric(value) => *value,
            Self::Distinct(_) => None,
        }
    }
}

fn summary_values(
    rows: &[BTreeMap<String, Option<String>>],
    shared: &VisualSharedConfig,
    _dimension: Option<&str>,
    _comparison: Option<&str>,
) -> Vec<VisualSummaryValue> {
    let missing_policy = shared
        .value_missing_policy
        .as_deref()
        .unwrap_or(&shared.missing_policy);
    rows.iter()
        .filter_map(|row| {
            let raw = row
                .get(&shared.summary_field)
                .and_then(|value| value.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty());
            match shared.summary_type.as_str() {
                "row_count" => Some(VisualSummaryValue::Numeric(Some(1.0))),
                "count" => match raw {
                    Some(_) => Some(VisualSummaryValue::Numeric(Some(1.0))),
                    None if missing_policy == "omit" => None,
                    None => Some(VisualSummaryValue::Numeric(Some(1.0))),
                },
                "unique_count" => match raw {
                    Some(value) => Some(VisualSummaryValue::Distinct(Some(value.to_string()))),
                    None if missing_policy == "explicit_missing" => {
                        Some(VisualSummaryValue::Distinct(None))
                    }
                    None => None,
                },
                _ => match raw.and_then(|value| value.parse::<f64>().ok()) {
                    Some(value) => Some(VisualSummaryValue::Numeric(Some(value))),
                    None if missing_policy == "zero" => {
                        Some(VisualSummaryValue::Numeric(Some(0.0)))
                    }
                    None => Some(VisualSummaryValue::Numeric(None)),
                },
            }
        })
        .collect()
}

fn summarize_values(values: &[VisualSummaryValue], summary_type: &str) -> Option<f64> {
    match summary_type {
        "row_count" => Some(values.len() as f64),
        "count" => Some(values.len() as f64),
        "unique_count" => {
            let unique = values
                .iter()
                .filter_map(|value| match value {
                    VisualSummaryValue::Distinct(value) => Some(value.clone()),
                    VisualSummaryValue::Numeric(_) => None,
                })
                .collect::<BTreeSet<_>>();
            Some(unique.len() as f64)
        }
        "sum" => {
            let numeric = values
                .iter()
                .filter_map(VisualSummaryValue::numeric)
                .collect::<Vec<_>>();
            (!numeric.is_empty()).then(|| numeric.iter().sum())
        }
        "average" => {
            let numeric = values
                .iter()
                .filter_map(VisualSummaryValue::numeric)
                .collect::<Vec<_>>();
            (!numeric.is_empty()).then(|| numeric.iter().sum::<f64>() / numeric.len() as f64)
        }
        "median" => {
            let mut numeric = values
                .iter()
                .filter_map(VisualSummaryValue::numeric)
                .collect::<Vec<_>>();
            if numeric.is_empty() {
                return None;
            }
            numeric.sort_by(|left, right| left.total_cmp(right));
            let middle = numeric.len() / 2;
            if numeric.len() % 2 == 0 {
                Some((numeric[middle - 1] + numeric[middle]) / 2.0)
            } else {
                Some(numeric[middle])
            }
        }
        _ => None,
    }
}

fn summarize_values_checked(
    values: &[VisualSummaryValue],
    summary_type: &str,
    context: &str,
) -> ApiResult<Option<f64>> {
    if summary_type == "none" && values.len() != 1 {
        return Err(ApiError::BadRequest(format!(
            "Do not summarize requires exactly one row for {context}; found {} rows",
            values.len()
        )));
    }
    if summary_type == "none" {
        return Ok(values.first().and_then(VisualSummaryValue::numeric));
    }
    Ok(summarize_values(values, summary_type))
}

fn sort_visual_points(
    points: &mut [ComponentVisualPoint],
    shared: &VisualSharedConfig,
    dimension_field_type: &FieldType,
) {
    let descending = shared.sort_direction == "desc";
    match shared.sort_field.as_deref() {
        Some("summary_value") => points.sort_by(|left, right| left.value.total_cmp(&right.value)),
        _ => points.sort_by(|left, right| {
            compare_dimension_values(&left.sort_x, &right.sort_x, dimension_field_type)
        }),
    }
    if descending {
        points.reverse();
    }
}

fn compare_optional_dimension_values(
    left: Option<&str>,
    right: Option<&str>,
    field_type: Option<&FieldType>,
) -> std::cmp::Ordering {
    match (left, right) {
        (Some(left), Some(right)) => {
            compare_dimension_values(left, right, field_type.unwrap_or(&FieldType::Text))
        }
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn compare_dimension_values(left: &str, right: &str, field_type: &FieldType) -> std::cmp::Ordering {
    if *field_type == FieldType::Number {
        match (left.parse::<f64>(), right.parse::<f64>()) {
            (Ok(left), Ok(right)) => return left.total_cmp(&right),
            (Ok(_), Err(_)) => return std::cmp::Ordering::Less,
            (Err(_), Ok(_)) => return std::cmp::Ordering::Greater,
            (Err(_), Err(_)) => {}
        }
    }
    left.cmp(right)
}

fn visual_field_type<'a>(fields: &'a [DataField], field_key: &str) -> ApiResult<&'a FieldType> {
    fields
        .iter()
        .find(|field| field.key == field_key)
        .map(|field| &field.field_type)
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "component field '{field_key}' is not in the selected Dataset major line"
            ))
        })
}

fn format_visual_value(value: f64, value_format: &str) -> String {
    match value_format {
        "integer" => format!("{value:.0}"),
        "decimal" => format!("{value:.2}"),
        "percent" => format!("{:.1}%", value * 100.0),
        _ => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                format!("{value:.2}")
            }
        }
    }
}

fn sentence_label(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

async fn component_rows_from_query(
    pool: &sqlx::PgPool,
    sql: &str,
    offset: usize,
    page_size: usize,
) -> ApiResult<(Vec<ComponentTableRow>, ComponentTablePagination)> {
    let mut rows = sqlx::query(sql).fetch_all(pool).await?;
    let has_more = rows.len() > page_size;
    if has_more {
        rows.truncate(page_size);
    }
    let next_cursor = has_more.then(|| format!("offset:{}", offset + page_size));
    let rows = rows
        .into_iter()
        .map(|row| {
            let row_id: String = row.try_get("__row_id")?;
            let mut values = BTreeMap::new();
            for column in row.columns() {
                let name = column.name();
                if name.starts_with("__") {
                    continue;
                }
                values.insert(name.to_string(), row.try_get(name)?);
            }
            Ok(ComponentTableRow { row_id, values })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(ApiError::from)?;
    Ok((
        rows,
        ComponentTablePagination {
            page_size,
            next_cursor,
            has_more,
        },
    ))
}

fn component_table_column(
    field: &DataField,
    display_labels: &BTreeMap<String, String>,
) -> ComponentTableColumn {
    ComponentTableColumn {
        key: field.key.clone(),
        label: display_labels
            .get(&field.key)
            .cloned()
            .unwrap_or_else(|| field.label.clone()),
        field_type: field.field_type.as_str().to_string(),
    }
}

pub(super) fn visible_table_fields<'a>(
    fields: &[&'a DataField],
    visible_columns: &[String],
) -> ApiResult<Vec<&'a DataField>> {
    if visible_columns.is_empty() {
        return Ok(fields.to_vec());
    }
    let mut selected = Vec::new();
    for key in visible_columns {
        let field = fields
            .iter()
            .find(|field| field.key == *key)
            .copied()
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "visible column '{key}' is outside the component table contract"
                ))
            })?;
        selected.push(field);
    }
    Ok(selected)
}

pub(super) fn table_search_fields<'a>(
    config: &TableComponentConfig,
    fields: &[&'a DataField],
) -> ApiResult<Vec<&'a DataField>> {
    // Blank search fields means search every projected component field using
    // text coercion, matching the shared interactive table viewer behavior.
    if config.search_fields.is_empty() {
        return Ok(fields.to_vec());
    }
    config
        .search_fields
        .iter()
        .map(|key| require_component_field_ref(fields, key, "table search field"))
        .collect()
}

fn search_predicate_sql(fields: &[&DataField], search: &str) -> String {
    let value = sql_literal(search);
    let predicates = fields
        .iter()
        .map(|field| {
            format!(
                "POSITION(LOWER({value}) IN LOWER(COALESCE({}::text, ''))) > 0",
                quote_identifier(&field.key)
            )
        })
        .collect::<Vec<_>>();
    format!("({})", predicates.join(" OR "))
}

pub(super) fn table_order_by_sql(
    sort: Option<&ComponentSortConfig>,
    fields: &[&DataField],
    fallback: &str,
) -> ApiResult<String> {
    let Some(sort) = sort else {
        return Ok(format!(" ORDER BY {}", quote_identifier(fallback)));
    };
    let field = fields
        .iter()
        .find(|field| field.key == sort.field_key)
        .copied()
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "component table sort references field '{}' outside the table contract",
                sort.field_key
            ))
        })?;
    let direction = if sort.direction.eq_ignore_ascii_case("desc") {
        "DESC"
    } else {
        "ASC"
    };
    Ok(format!(
        " ORDER BY {} {direction}, {}",
        typed_orderable_sql(&quote_identifier(&field.key), field.field_type.as_str()),
        quote_identifier(fallback)
    ))
}

pub(super) fn component_pagination_sql(offset: usize, page_size: usize) -> String {
    format!(" LIMIT {} OFFSET {}", page_size + 1, offset)
}

pub(super) fn effective_component_page_size(
    query_page_size: Option<usize>,
    config_page_size: Option<usize>,
) -> usize {
    query_page_size
        .or(config_page_size)
        .unwrap_or(50)
        .clamp(1, 200)
}

fn filter_to_sql(
    filter: &ComponentFilterConfig,
    fields: &[&DataField],
    label: &str,
) -> ApiResult<String> {
    let field = require_component_field_ref(fields, &filter.field_key, label)?;
    let operator = FilterOperator::parse(&filter.operator).map_err(component_data_op_error)?;
    operator
        .validate_for_field(field)
        .map_err(component_data_op_error)?;
    validate_component_filter_value(field, operator, filter.value.as_deref(), label)?;
    Ok(filter_predicate_sql(
        field,
        operator,
        filter.value.as_deref(),
    ))
}

fn validate_component_filter_value(
    field: &DataField,
    operator: FilterOperator,
    value: Option<&str>,
    label: &str,
) -> ApiResult<()> {
    if !operator.requires_value() {
        return Ok(());
    }
    if matches!(
        operator,
        FilterOperator::Between | FilterOperator::NotBetween
    ) {
        let value = required_filter_value(field, value, operator, label)?;
        let (lower, upper) = parse_filter_range(value).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "{label} for field '{}' requires two values for operator '{}'",
                field.key,
                operator.as_str()
            ))
        })?;
        validate_filter_scalar(field, lower, operator, label)?;
        validate_filter_scalar(field, upper, operator, label)?;
        return Ok(());
    }
    let value = required_filter_value(field, value, operator, label)?;
    validate_filter_scalar(field, value, operator, label)
}

fn required_filter_value<'a>(
    field: &DataField,
    value: Option<&'a str>,
    operator: FilterOperator,
    label: &str,
) -> ApiResult<&'a str> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "{label} for field '{}' requires a value for operator '{}'",
                field.key,
                operator.as_str()
            ))
        })
}

fn parse_filter_range(value: &str) -> Option<(&str, &str)> {
    value
        .split_once("..")
        .or_else(|| value.split_once(','))
        .and_then(|(lower, upper)| {
            let lower = lower.trim();
            let upper = upper.trim();
            if lower.is_empty() || upper.is_empty() {
                None
            } else {
                Some((lower, upper))
            }
        })
}

fn validate_filter_scalar(
    field: &DataField,
    value: &str,
    operator: FilterOperator,
    label: &str,
) -> ApiResult<()> {
    let valid = match &field.field_type {
        FieldType::Number => value.parse::<f64>().is_ok(),
        FieldType::Boolean => matches!(
            value.to_ascii_lowercase().as_str(),
            "true" | "t" | "1" | "yes" | "y" | "false" | "f" | "0" | "no" | "n"
        ),
        FieldType::Date => NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok(),
        FieldType::DateTime | FieldType::Timestamp => parse_filter_timestamp(value),
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} for field '{}' has invalid value '{}' for operator '{}' and type '{}'",
            field.key,
            value,
            operator.as_str(),
            field.field_type.as_str()
        )))
    }
}

fn parse_filter_timestamp(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value).is_ok()
        || DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f%#z").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f").is_ok()
        || NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}

pub(super) fn parse_component_cursor(cursor: Option<&str>) -> ApiResult<usize> {
    let Some(cursor) = cursor.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(0);
    };
    let value = cursor.strip_prefix("offset:").unwrap_or(cursor);
    value
        .parse::<usize>()
        .map_err(|_| ApiError::BadRequest("component table cursor is invalid".into()))
}

pub(super) fn parse_component_sort(value: &str) -> ApiResult<ComponentSortConfig> {
    let (field_key, direction) = value.split_once(':').unwrap_or((value, "asc"));
    let direction = match direction.trim().to_ascii_lowercase().as_str() {
        "asc" | "" => "asc",
        "desc" => "desc",
        _ => {
            return Err(ApiError::BadRequest(
                "component table sort direction must be asc or desc".into(),
            ));
        }
    };
    Ok(ComponentSortConfig {
        field_key: field_key.trim().to_string(),
        direction: direction.into(),
    })
}

pub(super) fn parse_component_query_filters(
    values: &HashMap<String, String>,
) -> ApiResult<Vec<ComponentFilterConfig>> {
    let mut by_field = BTreeMap::<String, (Option<String>, Option<String>)>::new();
    for (key, value) in values {
        let Some(remainder) = key.strip_prefix("filter[") else {
            continue;
        };
        let Some((field_key, suffix)) = remainder.split_once("][") else {
            continue;
        };
        let Some(kind) = suffix.strip_suffix(']') else {
            continue;
        };
        let entry = by_field.entry(field_key.to_string()).or_default();
        match kind {
            "operator" => entry.0 = Some(value.clone()),
            "value" => entry.1 = Some(value.clone()),
            _ => {}
        }
    }
    by_field
        .into_iter()
        .map(|(field_key, (operator, value))| {
            let operator = operator.ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "component table filter for '{field_key}' is missing an operator"
                ))
            })?;
            Ok(ComponentFilterConfig {
                field_key,
                operator,
                value,
            })
        })
        .collect()
}

pub(super) fn csv_keys(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn materialized_full_name(materialization: &MajorLineMaterialization) -> String {
    format!(
        "{}.{}",
        quote_identifier(&materialization.schema),
        quote_identifier(&materialization.table)
    )
}

pub(super) fn component_filter_sql(
    filters: &[ComponentFilterConfig],
    fields: &[&DataField],
) -> ApiResult<Vec<String>> {
    filters
        .iter()
        .map(|filter| filter_to_sql(filter, fields, "component filter"))
        .collect()
}

fn filter_predicate_sql(
    field: &DataField,
    operator: FilterOperator,
    value: Option<&str>,
) -> String {
    let field_sql = quote_identifier(&field.key);
    let value_sql = sql_literal(value.unwrap_or_default());
    match operator {
        FilterOperator::Equals => equality_sql(&field_sql, &value_sql, field.field_type.as_str()),
        FilterOperator::NotEquals => {
            let equality = equality_sql(&field_sql, &value_sql, field.field_type.as_str());
            format!("NOT ({equality})")
        }
        FilterOperator::Contains => {
            format!("POSITION(LOWER({value_sql}) IN LOWER(COALESCE({field_sql}, ''))) > 0")
        }
        FilterOperator::NotContains => {
            format!("POSITION(LOWER({value_sql}) IN LOWER(COALESCE({field_sql}, ''))) = 0")
        }
        FilterOperator::Lt => {
            comparison_sql(&field_sql, "<", &value_sql, field.field_type.as_str())
        }
        FilterOperator::Lte => {
            comparison_sql(&field_sql, "<=", &value_sql, field.field_type.as_str())
        }
        FilterOperator::Gt => {
            comparison_sql(&field_sql, ">", &value_sql, field.field_type.as_str())
        }
        FilterOperator::Gte => {
            comparison_sql(&field_sql, ">=", &value_sql, field.field_type.as_str())
        }
        FilterOperator::Between | FilterOperator::NotBetween => {
            between_filter_sql(&field_sql, field.field_type.as_str(), value, operator)
        }
        FilterOperator::IsEmpty => format!("NULLIF({field_sql}, '') IS NULL"),
        FilterOperator::IsNotEmpty => format!("NULLIF({field_sql}, '') IS NOT NULL"),
        FilterOperator::IsNull => format!("{field_sql} IS NULL"),
        FilterOperator::IsNotNull => format!("{field_sql} IS NOT NULL"),
    }
}

fn between_filter_sql(
    field_sql: &str,
    field_type: &str,
    value: Option<&str>,
    operator: FilterOperator,
) -> String {
    let (lower, upper) = value
        .unwrap_or_default()
        .split_once("..")
        .or_else(|| value.unwrap_or_default().split_once(','))
        .unwrap_or(("", ""));
    let lower = sql_literal(lower.trim());
    let upper = sql_literal(upper.trim());
    let predicate = format!(
        "({} AND {})",
        comparison_sql(field_sql, ">=", &lower, field_type),
        comparison_sql(field_sql, "<=", &upper, field_type)
    );
    if operator == FilterOperator::NotBetween {
        format!("NOT {predicate}")
    } else {
        predicate
    }
}

fn comparison_sql(left: &str, operator: &str, right: &str, field_type: &str) -> String {
    typed_comparable_sql(left, field_type)
        .zip(typed_comparable_sql(right, field_type))
        .map(|(left, right)| format!("{left} {operator} {right}"))
        .unwrap_or_else(|| "FALSE /* unsupported comparison */".to_string())
}

fn equality_sql(left: &str, right: &str, field_type: &str) -> String {
    typed_comparable_sql(left, field_type)
        .zip(typed_comparable_sql(right, field_type))
        .map(|(left, right)| {
            format!("COALESCE({left} = {right}, {left} IS NULL AND {right} IS NULL)")
        })
        .unwrap_or_else(|| format!("COALESCE({left}, '') = COALESCE({right}, '')"))
}

fn typed_comparable_sql(expression: &str, field_type: &str) -> Option<String> {
    match field_type {
        "number" => Some(format!("NULLIF({expression}, '')::numeric")),
        "date" => Some(format!("NULLIF({expression}, '')::date")),
        "datetime" | "timestamp" => Some(format!("NULLIF({expression}, '')::timestamptz")),
        "boolean" => Some(nullable_boolean_expression_sql(expression)),
        _ => None,
    }
}

fn typed_orderable_sql(expression: &str, field_type: &str) -> String {
    typed_comparable_sql(expression, field_type)
        .unwrap_or_else(|| format!("NULLIF({expression}, '')"))
}

fn boolean_expression_sql(expression: &str) -> String {
    format!("LOWER(COALESCE({expression}, '')) IN ('true', 't', '1', 'yes', 'y')")
}

fn nullable_boolean_expression_sql(expression: &str) -> String {
    format!(
        "CASE WHEN NULLIF({expression}, '') IS NULL THEN NULL ELSE {} END",
        boolean_expression_sql(expression)
    )
}

fn tier_access_predicate(account: &auth::AccountContext) -> &'static str {
    if account.has_capability("admin:all") || account.has_capability("datasets:read_confidential") {
        "TRUE"
    } else if account.has_capability("datasets:read_restricted") {
        "COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal', 'restricted')"
    } else {
        "COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal')"
    }
}

async fn tier_access_predicate_for_materialization(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    materialization: &MajorLineMaterialization,
) -> ApiResult<String> {
    let has_restriction_tier: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM information_schema.columns
            WHERE table_schema = $1
              AND table_name = $2
              AND column_name = '__restriction_tier'
        )
        "#,
    )
    .bind(&materialization.schema)
    .bind(&materialization.table)
    .fetch_one(pool)
    .await?;

    if has_restriction_tier {
        Ok(tier_access_predicate(account).to_string())
    } else {
        Ok("TRUE".into())
    }
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
