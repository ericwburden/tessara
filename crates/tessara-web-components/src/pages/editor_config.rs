//! Typed Component editor drafts, saved-config parsing, and query serialization.

use leptos::prelude::{RwSignal, Set};
use serde_json::{Value, json};
use tessara_web_data_ops::{
    DatasetFieldDraft as DataOpsDatasetFieldDraft, DatasetRowFilterDraft as DataOpsRowFilterDraft,
};

use super::{
    ComponentDefinition, ComponentFormValues, ComponentVersionSummary, category_colors_map,
    category_labels_map, csv_field_keys,
};

#[cfg(feature = "hydrate")]
pub(super) fn build_component_config(values: &ComponentFormValues) -> Value {
    ComponentConfigDraft::from_form(values).into_json()
}

pub(super) struct TableConfigDraft {
    pub(super) columns: Vec<DataOpsDatasetFieldDraft>,
    pub(super) filters: Vec<DataOpsRowFilterDraft>,
    pub(super) sort_field: String,
    pub(super) sort_direction: String,
    pub(super) page_size: String,
}

#[derive(Clone)]
pub(super) struct VisualSharedDraft {
    pub(super) summary_field: String,
    pub(super) summary_type: String,
    pub(super) value_format: String,
    pub(super) value_missing_policy: String,
    pub(super) sort_field: String,
    pub(super) sort_direction: String,
    pub(super) filters: Vec<DataOpsRowFilterDraft>,
    pub(super) limit: usize,
}

pub(super) struct BarConfigDraft {
    pub(super) shared: VisualSharedDraft,
    pub(super) category_field: String,
    pub(super) category_missing_policy: String,
    pub(super) comparison_field: String,
    pub(super) comparison_missing_policy: String,
    pub(super) comparison_layout: String,
    pub(super) orientation: String,
    pub(super) x_axis_label: String,
    pub(super) y_axis_label: String,
    pub(super) category_labels: String,
    pub(super) category_colors: String,
    pub(super) legend_title: String,
}

pub(super) struct LineConfigDraft {
    pub(super) shared: VisualSharedDraft,
    pub(super) x_field: String,
    pub(super) x_missing_policy: String,
    pub(super) smoothing: bool,
}

pub(super) struct PieDonutConfigDraft {
    pub(super) shared: VisualSharedDraft,
    pub(super) category_field: String,
    pub(super) category_missing_policy: String,
    pub(super) category_labels: String,
    pub(super) category_colors: String,
    pub(super) legend_title: String,
}

pub(super) struct StatCardConfigDraft {
    pub(super) shared: VisualSharedDraft,
    pub(super) label: String,
    pub(super) supporting_text: String,
    pub(super) panel_style: String,
}

pub(super) enum ComponentConfigDraft {
    Table(TableConfigDraft),
    Bar(BarConfigDraft),
    Line(LineConfigDraft),
    Pie(PieDonutConfigDraft),
    Donut(PieDonutConfigDraft),
    StatCard(StatCardConfigDraft),
}

impl ComponentConfigDraft {
    #[cfg(feature = "hydrate")]
    fn from_form(values: &ComponentFormValues) -> Self {
        if values.component_type == "table" {
            return Self::Table(TableConfigDraft {
                columns: values.columns.clone(),
                filters: values.filters.clone(),
                sort_field: values.sort_field.clone(),
                sort_direction: values.sort_direction.clone(),
                page_size: values.page_size.clone(),
            });
        }
        let shared = VisualSharedDraft::from_form(values);
        match values.component_type.as_str() {
            "bar" => Self::Bar(BarConfigDraft {
                shared,
                category_field: values.visual_category_field.clone(),
                category_missing_policy: values.visual_category_missing_policy.clone(),
                comparison_field: values.visual_comparison_field.clone(),
                comparison_missing_policy: values.visual_comparison_missing_policy.clone(),
                comparison_layout: values.visual_bar_comparison_layout.clone(),
                orientation: values.visual_bar_orientation.clone(),
                x_axis_label: values.visual_x_axis_label.clone(),
                y_axis_label: values.visual_y_axis_label.clone(),
                category_labels: values.visual_category_labels.clone(),
                category_colors: values.visual_category_colors.clone(),
                legend_title: values.visual_legend_title.clone(),
            }),
            "line" => Self::Line(LineConfigDraft {
                shared,
                x_field: values.visual_x_field.clone(),
                x_missing_policy: values.visual_category_missing_policy.clone(),
                smoothing: values.visual_line_smoothing,
            }),
            "pie" | "donut" => {
                let draft = PieDonutConfigDraft {
                    shared,
                    category_field: values.visual_category_field.clone(),
                    category_missing_policy: values.visual_category_missing_policy.clone(),
                    category_labels: values.visual_category_labels.clone(),
                    category_colors: values.visual_category_colors.clone(),
                    legend_title: values.visual_legend_title.clone(),
                };
                if values.component_type == "pie" {
                    Self::Pie(draft)
                } else {
                    Self::Donut(draft)
                }
            }
            _ => Self::StatCard(StatCardConfigDraft {
                shared,
                label: values.stat_label.clone(),
                supporting_text: values.stat_supporting_text.clone(),
                panel_style: values.stat_panel_style.clone(),
            }),
        }
    }

    pub(super) fn into_json(self) -> Value {
        match self {
            Self::Table(draft) => build_table_component_config(
                &draft.columns,
                &draft.filters,
                &draft.sort_field,
                &draft.sort_direction,
                &draft.page_size,
            ),
            Self::Bar(draft) => draft.into_json(),
            Self::Line(draft) => draft.into_json(),
            Self::Pie(draft) | Self::Donut(draft) => draft.into_json(),
            Self::StatCard(draft) => draft.into_json(),
        }
    }
}

impl VisualSharedDraft {
    #[cfg(feature = "hydrate")]
    fn from_form(values: &ComponentFormValues) -> Self {
        Self {
            summary_field: values.visual_summary_field.trim().into(),
            summary_type: values.visual_summary_type.trim().into(),
            value_format: values.visual_value_format.trim().into(),
            value_missing_policy: values.visual_missing_policy.trim().into(),
            sort_field: values.visual_sort_field.trim().into(),
            sort_direction: if values.visual_sort_direction.trim() == "desc" {
                "desc"
            } else {
                "asc"
            }
            .into(),
            filters: values.filters.clone(),
            limit: values
                .visual_limit
                .trim()
                .parse::<usize>()
                .ok()
                .map(|value| value.clamp(1, 100))
                .unwrap_or(20),
        }
    }

    fn into_map(self, include_sort: bool) -> serde_json::Map<String, Value> {
        let mut config = serde_json::Map::new();
        config.insert("summary_field".into(), Value::String(self.summary_field));
        config.insert("summary_type".into(), Value::String(self.summary_type));
        config.insert("value_format".into(), Value::String(self.value_format));
        config.insert("missing_policy".into(), Value::String("omit".into()));
        config.insert(
            "value_missing_policy".into(),
            Value::String(self.value_missing_policy),
        );
        if include_sort && !self.sort_field.is_empty() {
            config.insert("sort_field".into(), Value::String(self.sort_field));
        }
        config.insert("sort_direction".into(), Value::String(self.sort_direction));
        config.insert(
            "filters".into(),
            Value::Array(
                self.filters
                    .iter()
                    .filter(|filter| !filter.field_key.trim().is_empty())
                    .map(table_filter_config)
                    .collect(),
            ),
        );
        config
    }
}

impl BarConfigDraft {
    fn into_json(self) -> Value {
        let limit = self.shared.limit;
        let mut config = self.shared.into_map(true);
        config.insert(
            "mode".into(),
            Value::String(
                if self.comparison_field.trim().is_empty() {
                    "summary"
                } else {
                    "comparison"
                }
                .into(),
            ),
        );
        config.insert(
            "category_field".into(),
            Value::String(self.category_field.trim().into()),
        );
        config.insert(
            "category_missing_policy".into(),
            Value::String(self.category_missing_policy.trim().into()),
        );
        if !self.comparison_field.trim().is_empty() {
            config.insert(
                "comparison_field".into(),
                Value::String(self.comparison_field.trim().into()),
            );
            config.insert(
                "comparison_missing_policy".into(),
                Value::String(self.comparison_missing_policy.trim().into()),
            );
            config.insert(
                "comparison_layout".into(),
                Value::String(
                    if self.comparison_layout.trim() == "stacked" {
                        "stacked"
                    } else {
                        "grouped"
                    }
                    .into(),
                ),
            );
            insert_visual_display_overrides(
                &mut config,
                &self.category_labels,
                &self.category_colors,
                &self.legend_title,
            );
        }
        config.insert(
            "orientation".into(),
            Value::String(
                if self.orientation.trim() == "vertical" {
                    "vertical"
                } else {
                    "horizontal"
                }
                .into(),
            ),
        );
        insert_nonempty_string(&mut config, "x_axis_label", &self.x_axis_label);
        insert_nonempty_string(&mut config, "y_axis_label", &self.y_axis_label);
        config.insert("number_of_points".into(), json!(limit));
        Value::Object(config)
    }
}

impl LineConfigDraft {
    fn into_json(self) -> Value {
        let limit = self.shared.limit;
        let mut config = self.shared.into_map(true);
        config.insert("x_field".into(), Value::String(self.x_field.trim().into()));
        config.insert(
            "x_missing_policy".into(),
            Value::String(self.x_missing_policy.trim().into()),
        );
        config.insert("smoothing".into(), Value::Bool(self.smoothing));
        config.insert("number_of_points".into(), json!(limit));
        Value::Object(config)
    }
}

impl PieDonutConfigDraft {
    fn into_json(self) -> Value {
        let limit = self.shared.limit;
        let mut config = self.shared.into_map(true);
        config.insert(
            "category_field".into(),
            Value::String(self.category_field.trim().into()),
        );
        config.insert(
            "category_missing_policy".into(),
            Value::String(self.category_missing_policy.trim().into()),
        );
        config.insert("max_slices".into(), json!(limit));
        insert_visual_display_overrides(
            &mut config,
            &self.category_labels,
            &self.category_colors,
            &self.legend_title,
        );
        Value::Object(config)
    }
}

impl StatCardConfigDraft {
    fn into_json(self) -> Value {
        let mut config = self.shared.into_map(false);
        insert_nonempty_string(&mut config, "label", &self.label);
        insert_nonempty_string(&mut config, "supporting_text", &self.supporting_text);
        config.insert(
            "panel_style".into(),
            Value::String(self.panel_style.trim().into()),
        );
        Value::Object(config)
    }
}

pub(super) fn insert_nonempty_string(
    config: &mut serde_json::Map<String, Value>,
    key: &str,
    value: &str,
) {
    if !value.trim().is_empty() {
        config.insert(key.into(), Value::String(value.trim().into()));
    }
}

pub(super) fn insert_visual_display_overrides(
    config: &mut serde_json::Map<String, Value>,
    labels: &str,
    colors: &str,
    legend_title: &str,
) {
    let labels = category_labels_config(labels);
    if !labels.is_empty() {
        config.insert("category_labels".into(), Value::Object(labels));
    }
    let colors = category_colors_config(colors);
    if !colors.is_empty() {
        config.insert("category_colors".into(), Value::Object(colors));
    }
    insert_nonempty_string(config, "legend_title", legend_title);
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
pub(super) fn build_table_component_config(
    columns: &[DataOpsDatasetFieldDraft],
    filters: &[DataOpsRowFilterDraft],
    sort_field: &str,
    sort_direction: &str,
    page_size: &str,
) -> Value {
    let defaults = table_defaults_config(sort_field, sort_direction, page_size);
    let display_labels = columns
        .iter()
        .map(|field| (field.key.clone(), Value::String(field.label.clone())))
        .collect::<serde_json::Map<_, _>>();
    let filters = filters
        .iter()
        .filter(|filter| !filter.field_key.trim().is_empty())
        .map(table_filter_config)
        .collect::<Vec<_>>();
    let mut config = json!({
        "visible_columns": columns
            .iter()
            .map(|field| field.key.clone())
            .collect::<Vec<_>>(),
        "display_labels": display_labels,
        "filters": filters
    });
    merge_table_defaults(&mut config, defaults);
    config
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(super) fn category_labels_config(value: &str) -> serde_json::Map<String, Value> {
    category_labels_map(value)
        .into_iter()
        .map(|(raw, display)| (raw, Value::String(display)))
        .collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(super) fn category_labels_text(config: &Value) -> String {
    config
        .get("category_labels")
        .and_then(Value::as_object)
        .map(|labels| {
            labels
                .iter()
                .filter_map(|(raw, display)| {
                    display.as_str().map(|display| format!("{raw} = {display}"))
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(super) fn category_colors_config(value: &str) -> serde_json::Map<String, Value> {
    category_colors_map(value)
        .into_iter()
        .map(|(raw, color)| (raw, Value::String(color)))
        .collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(super) fn category_colors_text(config: &Value) -> String {
    config
        .get("category_colors")
        .and_then(Value::as_object)
        .map(|colors| {
            colors
                .iter()
                .filter_map(|(raw, color)| color.as_str().map(|color| format!("{raw} = {color}")))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

pub(super) fn table_filter_config(filter: &DataOpsRowFilterDraft) -> Value {
    let mut filter_config = serde_json::Map::new();
    filter_config.insert("field_key".into(), Value::String(filter.field_key.clone()));
    filter_config.insert("operator".into(), Value::String(filter.operator.clone()));
    if !filter.value.trim().is_empty() {
        filter_config.insert("value".into(), Value::String(filter.value.clone()));
    }
    Value::Object(filter_config)
}

pub(super) fn table_defaults_config(
    sort_field: &str,
    sort_direction: &str,
    page_size: &str,
) -> Value {
    let parsed_page_size = page_size
        .trim()
        .parse::<usize>()
        .ok()
        .map(|value| value.clamp(1, 200))
        .unwrap_or(50);
    let direction = if sort_direction.trim() == "desc" {
        "desc"
    } else {
        "asc"
    };
    let default_sort = if sort_field.trim().is_empty() {
        Value::Null
    } else {
        json!({
            "field_key": sort_field.trim(),
            "direction": direction
        })
    };
    json!({
        "default_sort": default_sort,
        "page_size": parsed_page_size
    })
}

pub(super) fn merge_table_defaults(config: &mut Value, defaults: Value) {
    if let (Some(target), Some(defaults)) = (config.as_object_mut(), defaults.as_object()) {
        for (key, value) in defaults {
            target.insert(key.clone(), value.clone());
        }
    }
}

#[cfg(test)]
pub(super) fn selected_dataset_major_value(dataset_id: &str, dataset_major: &str) -> String {
    if dataset_id.trim().is_empty() || dataset_major.trim().is_empty() {
        String::new()
    } else {
        format!("{}|{}", dataset_id.trim(), dataset_major.trim())
    }
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
pub(super) fn editable_component_version(
    component: &ComponentDefinition,
) -> Option<ComponentVersionSummary> {
    component
        .versions
        .iter()
        .find(|version| version.status == "draft")
        .or_else(|| {
            component
                .versions
                .iter()
                .find(|version| version.status == "published")
        })
        .cloned()
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
pub(super) fn table_visible_columns_from_config(config: &Value) -> String {
    config
        .get("visible_columns")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| {
            value.as_str().map(str::to_string).or_else(|| {
                value
                    .get("field_key")
                    .or_else(|| value.get("key"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(super) fn table_projection_fields_from_config_keys(
    config: &Value,
) -> Vec<DataOpsDatasetFieldDraft> {
    let labels = config
        .get("display_labels")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    csv_field_keys(&table_visible_columns_from_config(config))
        .into_iter()
        .map(|key| DataOpsDatasetFieldDraft {
            label: labels
                .get(&key)
                .and_then(Value::as_str)
                .unwrap_or(&key)
                .into(),
            source_alias: "dataset".into(),
            source_field_key: key.clone(),
            field_type: String::new(),
            key,
        })
        .collect()
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
pub(super) fn table_sort_from_config(config: &Value) -> (String, String) {
    let Some(sort) = config.get("default_sort") else {
        return (String::new(), "asc".into());
    };
    let field = sort
        .get("field_key")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let direction = match sort.get("direction").and_then(Value::as_str) {
        Some("desc") => "desc",
        _ => "asc",
    };
    (field, direction.into())
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
pub(super) fn table_page_size_from_config(config: &Value) -> String {
    config
        .get("page_size")
        .and_then(Value::as_u64)
        .map(|value| value.clamp(1, 200).to_string())
        .unwrap_or_else(|| "50".into())
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(super) fn table_filter_drafts_from_config(config: &Value) -> Vec<DataOpsRowFilterDraft> {
    config
        .get("filters")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(index, filter)| {
            let field_key = filter.get("field_key").and_then(Value::as_str)?.trim();
            let operator = filter.get("operator").and_then(Value::as_str)?.trim();
            if field_key.is_empty() || operator.is_empty() {
                return None;
            }
            Some(DataOpsRowFilterDraft {
                id: (index as u64) + 1,
                field_key: field_key.into(),
                operator: operator.into(),
                value: filter
                    .get("value")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .into(),
                value_mode: "value".into(),
                value_field_key: String::new(),
            })
        })
        .collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
#[allow(clippy::too_many_arguments)]
pub(super) fn load_visual_config_signals(
    component_type: &str,
    config: &Value,
    summary_field: RwSignal<String>,
    summary_type: RwSignal<String>,
    category_field: RwSignal<String>,
    category_labels: RwSignal<String>,
    category_colors: RwSignal<String>,
    legend_title: RwSignal<String>,
    comparison_field: RwSignal<String>,
    bar_orientation: RwSignal<String>,
    bar_comparison_layout: RwSignal<String>,
    x_axis_label: RwSignal<String>,
    y_axis_label: RwSignal<String>,
    x_field: RwSignal<String>,
    line_smoothing: RwSignal<bool>,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    limit: RwSignal<String>,
    value_format: RwSignal<String>,
    category_missing_policy: RwSignal<String>,
    comparison_missing_policy: RwSignal<String>,
    missing_policy: RwSignal<String>,
    stat_label: RwSignal<String>,
    stat_supporting_text: RwSignal<String>,
    stat_panel_style: RwSignal<String>,
) {
    summary_field.set(config_string(config, "summary_field"));
    summary_type.set(config_string_or(config, "summary_type", "count"));
    category_field.set(config_string(config, "category_field"));
    category_labels.set(category_labels_text(config));
    category_colors.set(category_colors_text(config));
    legend_title.set(config_string(config, "legend_title"));
    comparison_field.set(config_string(config, "comparison_field"));
    bar_orientation.set(config_string_or(config, "orientation", "horizontal"));
    bar_comparison_layout.set(config_string_or(config, "comparison_layout", "grouped"));
    x_axis_label.set(config_string(config, "x_axis_label"));
    y_axis_label.set(config_string(config, "y_axis_label"));
    x_field.set(config_string(config, "x_field"));
    line_smoothing.set(
        config
            .get("smoothing")
            .and_then(Value::as_bool)
            .unwrap_or(true),
    );
    sort_field.set(config_string(config, "sort_field"));
    sort_direction.set(config_string_or(config, "sort_direction", "asc"));
    value_format.set(config_string_or(config, "value_format", "plain"));
    let legacy_missing_policy = config_string_or(config, "missing_policy", "omit");
    category_missing_policy.set(config_string_or(
        config,
        if component_type == "line" {
            "x_missing_policy"
        } else {
            "category_missing_policy"
        },
        &legacy_missing_policy,
    ));
    comparison_missing_policy.set(config_string_or(
        config,
        "comparison_missing_policy",
        &legacy_missing_policy,
    ));
    missing_policy.set(config_string_or(
        config,
        "value_missing_policy",
        &legacy_missing_policy,
    ));
    stat_label.set(config_string(config, "label"));
    stat_supporting_text.set(config_string(config, "supporting_text"));
    stat_panel_style.set(config_string_or(config, "panel_style", "default"));
    let limit_key = if matches!(component_type, "pie" | "donut") {
        "max_slices"
    } else {
        "number_of_points"
    };
    limit.set(
        config
            .get(limit_key)
            .and_then(Value::as_u64)
            .map(|value| value.clamp(1, 100).to_string())
            .unwrap_or_else(|| "20".into()),
    );
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(super) fn config_string(config: &Value, key: &str) -> String {
    config
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .into()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(super) fn config_string_or(config: &Value, key: &str, fallback: &str) -> String {
    config
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .into()
}
