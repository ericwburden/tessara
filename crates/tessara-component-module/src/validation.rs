use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::Value;
use tessara_datasets_contract::{DatasetFieldContract, DatasetFieldRequirement};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ConfigFinding {
    pub code: &'static str,
    pub field_path: Option<String>,
    pub message: String,
}

impl ConfigFinding {
    fn new(
        code: &'static str,
        field_path: impl Into<Option<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            field_path: field_path.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentFilterConfig {
    #[serde(alias = "field")]
    field_key: String,
    operator: String,
    #[serde(default)]
    value: Option<Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentSortConfig {
    field_key: String,
    #[serde(default = "default_sort_direction")]
    direction: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ComponentFieldRef {
    Key(String),
    FieldKey { field_key: String },
    ObjectKey { key: String },
}

impl ComponentFieldRef {
    fn field_key(&self) -> &str {
        match self {
            Self::Key(key) => key,
            Self::FieldKey { field_key } => field_key,
            Self::ObjectKey { key } => key,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TableComponentConfig {
    #[serde(default)]
    visible_columns: Vec<ComponentFieldRef>,
    #[serde(default)]
    filters: Vec<ComponentFilterConfig>,
    #[serde(default)]
    search_fields: Vec<String>,
    #[serde(default)]
    default_sort: Option<ComponentSortConfig>,
    #[serde(default)]
    page_size: Option<usize>,
    #[serde(default)]
    display_labels: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VisualSharedConfig {
    summary_field: String,
    summary_type: String,
    #[serde(default = "default_value_format")]
    value_format: String,
    #[serde(default = "default_missing_policy")]
    missing_policy: String,
    #[serde(default)]
    value_missing_policy: Option<String>,
    #[serde(default)]
    sort_field: Option<String>,
    #[serde(default = "default_sort_direction")]
    sort_direction: String,
    #[serde(default)]
    filters: Vec<ComponentFilterConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StatCardComponentConfig {
    #[serde(flatten)]
    shared: VisualSharedConfig,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    supporting_text: Option<String>,
    #[serde(default = "default_panel_style")]
    panel_style: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BarComponentConfig {
    #[serde(flatten)]
    shared: VisualSharedConfig,
    mode: String,
    category_field: String,
    #[serde(default)]
    category_missing_policy: Option<String>,
    #[serde(default)]
    comparison_field: Option<String>,
    #[serde(default)]
    comparison_missing_policy: Option<String>,
    #[serde(default = "default_bar_orientation")]
    orientation: String,
    #[serde(default = "default_bar_comparison_layout")]
    comparison_layout: String,
    #[serde(default = "default_visual_limit")]
    number_of_points: usize,
    #[serde(default)]
    category_labels: BTreeMap<String, String>,
    #[serde(default)]
    category_colors: BTreeMap<String, String>,
    #[serde(default)]
    legend_title: Option<String>,
    #[serde(default)]
    x_axis_label: Option<String>,
    #[serde(default)]
    y_axis_label: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LineComponentConfig {
    #[serde(flatten)]
    shared: VisualSharedConfig,
    x_field: String,
    #[serde(default)]
    x_missing_policy: Option<String>,
    #[serde(default = "default_line_smoothing")]
    smoothing: bool,
    #[serde(default = "default_visual_limit")]
    number_of_points: usize,
    #[serde(default)]
    x_axis_label: Option<String>,
    #[serde(default)]
    y_axis_label: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PieComponentConfig {
    #[serde(flatten)]
    shared: VisualSharedConfig,
    category_field: String,
    #[serde(default)]
    category_missing_policy: Option<String>,
    #[serde(default = "default_visual_limit")]
    max_slices: usize,
    #[serde(default)]
    category_labels: BTreeMap<String, String>,
    #[serde(default)]
    category_colors: BTreeMap<String, String>,
    #[serde(default)]
    legend_title: Option<String>,
}

enum VisualComponentConfig {
    StatCard(StatCardComponentConfig),
    Bar(BarComponentConfig),
    Line(LineComponentConfig),
    Pie(PieComponentConfig),
    Donut(PieComponentConfig),
}

impl VisualComponentConfig {
    fn parse(component_type: &str, value: &Value) -> Result<Self, String> {
        let parse = |label: &str| format!("{label} Component configuration is invalid");
        match component_type {
            "stat_card" => serde_json::from_value(value.clone())
                .map(Self::StatCard)
                .map_err(|_| parse("Stat card")),
            "bar" => serde_json::from_value(value.clone())
                .map(Self::Bar)
                .map_err(|_| parse("Bar")),
            "line" => serde_json::from_value(value.clone())
                .map(Self::Line)
                .map_err(|_| parse("Line")),
            "pie" => serde_json::from_value(value.clone())
                .map(Self::Pie)
                .map_err(|_| parse("Pie")),
            "donut" => serde_json::from_value(value.clone())
                .map(Self::Donut)
                .map_err(|_| parse("Donut")),
            _ => Err("Component kind is unsupported".into()),
        }
    }

    fn shared(&self) -> &VisualSharedConfig {
        match self {
            Self::StatCard(config) => &config.shared,
            Self::Bar(config) => &config.shared,
            Self::Line(config) => &config.shared,
            Self::Pie(config) | Self::Donut(config) => &config.shared,
        }
    }
}

fn default_value_format() -> String {
    "plain".into()
}
fn default_missing_policy() -> String {
    "omit".into()
}
fn default_sort_direction() -> String {
    "asc".into()
}
fn default_panel_style() -> String {
    "default".into()
}
fn default_bar_orientation() -> String {
    "horizontal".into()
}
fn default_bar_comparison_layout() -> String {
    "grouped".into()
}
fn default_visual_limit() -> usize {
    20
}
fn default_line_smoothing() -> bool {
    true
}

pub(super) fn validate_component_config(
    component_type: &str,
    config: &Value,
    fields: &[DatasetFieldContract],
) -> Vec<ConfigFinding> {
    match component_type {
        "table" => validate_table(config, fields),
        "bar" | "line" | "pie" | "donut" | "stat_card" => {
            validate_visual(component_type, config, fields)
        }
        _ => vec![ConfigFinding::new(
            "component_type.unsupported",
            Some("component_type".into()),
            "Component kind is unsupported",
        )],
    }
}

pub(super) fn required_field_keys(component_type: &str, config: &Value) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    match component_type {
        "table" => {
            let Ok(config) = serde_json::from_value::<TableComponentConfig>(config.clone()) else {
                return keys;
            };
            keys.extend(
                config
                    .visible_columns
                    .iter()
                    .map(|field| field.field_key().to_string()),
            );
            keys.extend(config.filters.iter().map(|filter| filter.field_key.clone()));
            keys.extend(config.search_fields);
            if let Some(sort) = config.default_sort {
                keys.insert(sort.field_key);
            }
        }
        "bar" | "line" | "pie" | "donut" | "stat_card" => {
            let Ok(config) = VisualComponentConfig::parse(component_type, config) else {
                return keys;
            };
            let shared = config.shared();
            if shared.summary_type != "row_count" {
                keys.insert(shared.summary_field.clone());
            }
            keys.extend(shared.filters.iter().map(|filter| filter.field_key.clone()));
            match config {
                VisualComponentConfig::Bar(config) => {
                    keys.insert(config.category_field);
                    keys.extend(config.comparison_field);
                }
                VisualComponentConfig::Line(config) => {
                    keys.insert(config.x_field);
                }
                VisualComponentConfig::Pie(config) | VisualComponentConfig::Donut(config) => {
                    keys.insert(config.category_field);
                }
                VisualComponentConfig::StatCard(_) => {}
            }
        }
        _ => {}
    }
    keys.retain(|key| !key.trim().is_empty());
    keys
}

pub(super) fn required_field_requirements(
    component_type: &str,
    config: &Value,
) -> Vec<DatasetFieldRequirement> {
    let numeric_summary_field = match component_type {
        "bar" | "line" | "pie" | "donut" | "stat_card" => {
            VisualComponentConfig::parse(component_type, config)
                .ok()
                .and_then(|config| {
                    let shared = config.shared();
                    matches!(
                        shared.summary_type.as_str(),
                        "sum" | "average" | "median" | "none"
                    )
                    .then(|| shared.summary_field.trim().to_string())
                    .filter(|field| !field.is_empty())
                })
        }
        _ => None,
    };
    let supported_existence_types = [
        "boolean",
        "date",
        "multi_choice",
        "number",
        "single_choice",
        "text",
    ];
    required_field_keys(component_type, config)
        .into_iter()
        .map(|field_key| DatasetFieldRequirement {
            accepted_types: if numeric_summary_field.as_deref() == Some(field_key.as_str()) {
                vec!["number".into()]
            } else {
                supported_existence_types
                    .iter()
                    .map(|field_type| (*field_type).into())
                    .collect()
            },
            field_key,
        })
        .collect()
}

pub(super) fn validate_version_note(note: &str) -> Vec<ConfigFinding> {
    if note.trim().chars().count() > 2_000 {
        vec![ConfigFinding::new(
            "version_note.too_long",
            Some("version_note".into()),
            "Component version note must be 2,000 characters or fewer",
        )]
    } else {
        Vec::new()
    }
}

fn validate_table(value: &Value, fields: &[DatasetFieldContract]) -> Vec<ConfigFinding> {
    let config: TableComponentConfig = match serde_json::from_value(value.clone()) {
        Ok(config) => config,
        Err(_) => {
            return vec![ConfigFinding::new(
                "config.invalid",
                Some("config".into()),
                "Table Component configuration is invalid",
            )];
        }
    };
    let mut findings = Vec::new();
    let known = field_map(fields);
    let mut visible = BTreeSet::new();
    for column in &config.visible_columns {
        require_field(
            &known,
            column.field_key(),
            "config.visible_columns",
            &mut findings,
        );
        if !visible.insert(column.field_key()) {
            findings.push(ConfigFinding::new(
                "config.field.duplicate",
                Some("config.visible_columns".into()),
                format!(
                    "Dataset field '{}' is selected more than once",
                    column.field_key()
                ),
            ));
        }
    }
    for key in &config.search_fields {
        require_visible(&visible, key, "config.search_fields", &mut findings);
    }
    for key in config.display_labels.keys() {
        require_visible(&visible, key, "config.display_labels", &mut findings);
    }
    if let Some(sort) = &config.default_sort {
        require_visible(
            &visible,
            &sort.field_key,
            "config.default_sort.field_key",
            &mut findings,
        );
        require_enum(
            &sort.direction,
            &["asc", "desc"],
            "config.default_sort.direction",
            &mut findings,
        );
    }
    if config
        .page_size
        .is_some_and(|value| !(1..=200).contains(&value))
    {
        findings.push(ConfigFinding::new(
            "config.page_size.out_of_range",
            Some("config.page_size".into()),
            "Table page size must be between 1 and 200",
        ));
    }
    validate_filters(&config.filters, &known, &mut findings);
    findings
}

fn validate_visual(
    component_type: &str,
    value: &Value,
    fields: &[DatasetFieldContract],
) -> Vec<ConfigFinding> {
    let config = match VisualComponentConfig::parse(component_type, value) {
        Ok(config) => config,
        Err(message) => {
            return vec![ConfigFinding::new(
                "config.invalid",
                Some("config".into()),
                message,
            )];
        }
    };
    let mut findings = Vec::new();
    let known = field_map(fields);
    let shared = config.shared();
    require_enum(
        &shared.summary_type,
        &[
            "row_count",
            "count",
            "unique_count",
            "sum",
            "average",
            "median",
            "none",
        ],
        "config.summary_type",
        &mut findings,
    );
    require_enum(
        &shared.value_format,
        &["plain", "integer", "decimal", "percent"],
        "config.value_format",
        &mut findings,
    );
    require_enum(
        &shared.missing_policy,
        &["omit", "zero", "explicit_missing"],
        "config.missing_policy",
        &mut findings,
    );
    if let Some(value) = &shared.value_missing_policy {
        require_enum(
            value,
            &["omit", "zero", "explicit_missing"],
            "config.value_missing_policy",
            &mut findings,
        );
    }
    require_enum(
        &shared.sort_direction,
        &["asc", "desc"],
        "config.sort_direction",
        &mut findings,
    );
    if shared.summary_type != "row_count" {
        require_field(
            &known,
            &shared.summary_field,
            "config.summary_field",
            &mut findings,
        );
    }
    if matches!(
        shared.summary_type.as_str(),
        "sum" | "average" | "median" | "none"
    ) && known
        .get(shared.summary_field.as_str())
        .is_some_and(|field| field.field_type != "number")
    {
        findings.push(ConfigFinding::new(
            "config.summary_field.type",
            Some("config.summary_field".into()),
            "The selected calculation requires a numeric summary field",
        ));
    }
    validate_filters(&shared.filters, &known, &mut findings);
    match &config {
        VisualComponentConfig::StatCard(config) => {
            let _ = (&config.label, &config.supporting_text);
            require_enum(
                &config.panel_style,
                &["default", "muted", "accent"],
                "config.panel_style",
                &mut findings,
            );
            if config.shared.sort_field.is_some() {
                findings.push(ConfigFinding::new(
                    "config.sort_field.unsupported",
                    Some("config.sort_field".into()),
                    "Stat card Components do not support a sort field",
                ));
            }
        }
        VisualComponentConfig::Bar(config) => {
            let _ = (
                &config.category_labels,
                &config.category_colors,
                &config.legend_title,
                &config.x_axis_label,
                &config.y_axis_label,
            );
            require_field(
                &known,
                &config.category_field,
                "config.category_field",
                &mut findings,
            );
            optional_missing_policy(
                config.category_missing_policy.as_deref(),
                "config.category_missing_policy",
                &mut findings,
            );
            optional_missing_policy(
                config.comparison_missing_policy.as_deref(),
                "config.comparison_missing_policy",
                &mut findings,
            );
            require_enum(
                &config.mode,
                &["summary", "comparison"],
                "config.mode",
                &mut findings,
            );
            require_enum(
                &config.orientation,
                &["vertical", "horizontal"],
                "config.orientation",
                &mut findings,
            );
            require_enum(
                &config.comparison_layout,
                &["grouped", "stacked"],
                "config.comparison_layout",
                &mut findings,
            );
            visual_limit(
                config.number_of_points,
                "config.number_of_points",
                &mut findings,
            );
            if config.mode == "summary" && config.comparison_field.is_some() {
                findings.push(ConfigFinding::new(
                    "config.comparison_field.unexpected",
                    Some("config.comparison_field".into()),
                    "Summary bars cannot declare a comparison field",
                ));
            }
            if config.mode == "comparison" {
                if let Some(field) = &config.comparison_field {
                    require_field(&known, field, "config.comparison_field", &mut findings);
                } else {
                    findings.push(ConfigFinding::new(
                        "config.comparison_field.required",
                        Some("config.comparison_field".into()),
                        "Comparison bars require a comparison field",
                    ));
                }
            }
            if config.mode == "comparison"
                && config.comparison_layout == "stacked"
                && !matches!(
                    config.shared.summary_type.as_str(),
                    "row_count" | "count" | "sum"
                )
            {
                findings.push(ConfigFinding::new(
                    "config.comparison_layout.incompatible",
                    Some("config.comparison_layout".into()),
                    "Stacked comparison bars require row count, count, or sum",
                ));
            }
            visual_sort(
                config.shared.sort_field.as_deref(),
                if config.mode == "comparison" {
                    &["category", "comparison", "summary_value"]
                } else {
                    &["category", "summary_value"]
                },
                &mut findings,
            );
        }
        VisualComponentConfig::Line(config) => {
            let _ = (config.smoothing, &config.x_axis_label, &config.y_axis_label);
            require_field(&known, &config.x_field, "config.x_field", &mut findings);
            optional_missing_policy(
                config.x_missing_policy.as_deref(),
                "config.x_missing_policy",
                &mut findings,
            );
            visual_limit(
                config.number_of_points,
                "config.number_of_points",
                &mut findings,
            );
            visual_sort(
                config.shared.sort_field.as_deref(),
                &["x", "summary_value"],
                &mut findings,
            );
        }
        VisualComponentConfig::Pie(config) | VisualComponentConfig::Donut(config) => {
            let _ = (
                &config.category_labels,
                &config.category_colors,
                &config.legend_title,
            );
            require_field(
                &known,
                &config.category_field,
                "config.category_field",
                &mut findings,
            );
            optional_missing_policy(
                config.category_missing_policy.as_deref(),
                "config.category_missing_policy",
                &mut findings,
            );
            visual_limit(config.max_slices, "config.max_slices", &mut findings);
            visual_sort(
                config.shared.sort_field.as_deref(),
                &["category", "summary_value"],
                &mut findings,
            );
        }
    }
    findings
}

fn field_map(fields: &[DatasetFieldContract]) -> BTreeMap<&str, &DatasetFieldContract> {
    fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect()
}

fn require_field(
    known: &BTreeMap<&str, &DatasetFieldContract>,
    key: &str,
    path: &str,
    findings: &mut Vec<ConfigFinding>,
) {
    if key.trim().is_empty() || !known.contains_key(key) {
        findings.push(ConfigFinding::new(
            "config.field_unavailable",
            Some(path.into()),
            format!("Dataset field '{key}' is unavailable"),
        ));
    }
}

fn require_visible(
    visible: &BTreeSet<&str>,
    key: &str,
    path: &str,
    findings: &mut Vec<ConfigFinding>,
) {
    if !visible.contains(key) {
        findings.push(ConfigFinding::new(
            "config.field_outside_projection",
            Some(path.into()),
            format!("Dataset field '{key}' is outside the Component projection"),
        ));
    }
}

fn require_enum(value: &str, allowed: &[&str], path: &str, findings: &mut Vec<ConfigFinding>) {
    if !allowed.contains(&value) {
        findings.push(ConfigFinding::new(
            "config.value.unsupported",
            Some(path.into()),
            format!("'{value}' is not supported for {path}"),
        ));
    }
}

fn optional_missing_policy(value: Option<&str>, path: &str, findings: &mut Vec<ConfigFinding>) {
    if let Some(value) = value {
        require_enum(value, &["omit", "explicit_missing"], path, findings);
    }
}

fn visual_limit(value: usize, path: &str, findings: &mut Vec<ConfigFinding>) {
    if !(1..=100).contains(&value) {
        findings.push(ConfigFinding::new(
            "config.limit.out_of_range",
            Some(path.into()),
            "Visual limits must be between 1 and 100",
        ));
    }
}

fn visual_sort(value: Option<&str>, allowed: &[&str], findings: &mut Vec<ConfigFinding>) {
    if let Some(value) = value {
        require_enum(value, allowed, "config.sort_field", findings);
    }
}

fn validate_filters(
    filters: &[ComponentFilterConfig],
    fields: &BTreeMap<&str, &DatasetFieldContract>,
    findings: &mut Vec<ConfigFinding>,
) {
    for filter in filters {
        let Some(field) = fields.get(filter.field_key.as_str()) else {
            require_field(fields, &filter.field_key, "config.filters", findings);
            continue;
        };
        let operator = filter.operator.as_str();
        let value_required = !matches!(
            operator,
            "is_empty" | "is_not_empty" | "is_null" | "is_not_null"
        );
        if value_required && filter.value.as_ref().is_none_or(Value::is_null) {
            findings.push(ConfigFinding::new(
                "config.filter.value_required",
                Some("config.filters".into()),
                format!("Filter '{}' requires a value", filter.field_key),
            ));
        }
        let allowed = match field.field_type.as_str() {
            "text" | "string" => &[
                "equals",
                "not_equals",
                "contains",
                "not_contains",
                "starts_with",
                "ends_with",
                "is_empty",
                "is_not_empty",
                "is_null",
                "is_not_null",
            ][..],
            "number" | "integer" | "decimal" | "date" | "datetime" => &[
                "equals",
                "not_equals",
                "lt",
                "lte",
                "gt",
                "gte",
                "less_than",
                "less_than_or_equal",
                "greater_than",
                "greater_than_or_equal",
                "between",
                "not_between",
                "is_null",
                "is_not_null",
            ][..],
            _ => &["equals", "not_equals", "is_null", "is_not_null"][..],
        };
        if !allowed.contains(&operator) {
            findings.push(ConfigFinding::new(
                "config.filter.operator_unsupported",
                Some("config.filters".into()),
                format!(
                    "Filter operator '{operator}' is not supported for field '{}'",
                    filter.field_key
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{required_field_requirements, validate_component_config, validate_version_note};
    use tessara_datasets_contract::DatasetFieldContract;

    fn fields() -> Vec<DatasetFieldContract> {
        vec![
            DatasetFieldContract {
                key: "label".into(),
                label: "Label".into(),
                field_type: "text".into(),
                restriction_tier: "public".into(),
            },
            DatasetFieldContract {
                key: "amount".into(),
                label: "Amount".into(),
                field_type: "number".into(),
                restriction_tier: "public".into(),
            },
        ]
    }

    #[test]
    fn table_contract_rejects_unknown_keys_fields_and_invalid_limits() {
        assert!(
            validate_component_config(
                "table",
                &json!({"visible_columns":["label"],"metrics":[]}),
                &fields()
            )
            .iter()
            .any(|finding| finding.code == "config.invalid")
        );
        assert!(
            validate_component_config("table", &json!({"visible_columns":["missing"]}), &fields())
                .iter()
                .any(|finding| finding.code == "config.field_unavailable")
        );
        assert!(
            validate_component_config(
                "table",
                &json!({"visible_columns":["label"],"page_size":500}),
                &fields()
            )
            .iter()
            .any(|finding| finding.code == "config.page_size.out_of_range")
        );
    }

    #[test]
    fn visual_contract_rejects_invalid_type_combinations_and_limits() {
        let invalid = json!({"mode":"comparison","summary_field":"label","summary_type":"average","category_field":"label","comparison_layout":"stacked","number_of_points":0});
        let findings = validate_component_config("bar", &invalid, &fields());
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "config.summary_field.type")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "config.comparison_field.required")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "config.comparison_layout.incompatible")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "config.limit.out_of_range")
        );
    }

    #[test]
    fn line_contract_retains_axis_titles_and_smoothing() {
        let config = json!({
            "summary_field":"label",
            "summary_type":"count",
            "x_field":"label",
            "sort_field":"x",
            "sort_direction":"asc",
            "number_of_points":20,
            "smoothing":true,
            "x_axis_label":"Program",
            "y_axis_label":"Responses"
        });
        assert!(validate_component_config("line", &config, &fields()).is_empty());
    }

    #[test]
    fn version_notes_retain_the_closed_baseline_bound() {
        assert!(validate_version_note(&"x".repeat(2_000)).is_empty());
        assert_eq!(
            validate_version_note(&"x".repeat(2_001))[0].code,
            "version_note.too_long"
        );
    }

    #[test]
    fn compatibility_requirements_come_from_component_semantics() {
        let requirements = required_field_requirements(
            "bar",
            &json!({
                "mode": "summary",
                "summary_field": "amount",
                "summary_type": "average",
                "category_field": "label"
            }),
        );
        let by_field = requirements
            .into_iter()
            .map(|requirement| (requirement.field_key, requirement.accepted_types))
            .collect::<std::collections::BTreeMap<_, _>>();

        assert_eq!(by_field["amount"], ["number"]);
        assert_eq!(
            by_field["label"],
            [
                "boolean",
                "date",
                "multi_choice",
                "number",
                "single_choice",
                "text",
            ]
        );
    }
}
