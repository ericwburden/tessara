//! Shared data-operation contracts for Tessara datasets and components.
//!
//! This crate intentionally stays free of database and API framework concerns.
//! Dataset builders, component renderers, and future component kinds can share
//! the same operation grammar and validation while keeping SQL compilation in
//! their owning service layer.

use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

/// Rule violation returned by pure data-operation validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataOpError {
    code: &'static str,
    message: String,
}

impl DataOpError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Stable error code suitable for API validation findings.
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Human-readable validation message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for DataOpError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DataOpError {}

/// Field contract exposed by a dataset line or operation output.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DataField {
    pub key: String,
    pub label: String,
    pub field_type: FieldType,
    pub position: i32,
}

/// Stable operation-level field types.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    StaticText,
    Number,
    Boolean,
    Date,
    DateTime,
    Timestamp,
    SingleChoice,
    MultiChoice,
    Other(String),
}

impl FieldType {
    /// Parses the API/storage representation used by existing dataset fields.
    pub fn parse(value: &str) -> Self {
        match value.trim() {
            "text" => Self::Text,
            "static_text" => Self::StaticText,
            "number" => Self::Number,
            "boolean" => Self::Boolean,
            "date" => Self::Date,
            "datetime" => Self::DateTime,
            "timestamp" => Self::Timestamp,
            "single_choice" => Self::SingleChoice,
            "multi_choice" => Self::MultiChoice,
            other => Self::Other(other.to_string()),
        }
    }

    /// Returns the stable API/storage representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::StaticText => "static_text",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Date => "date",
            Self::DateTime => "datetime",
            Self::Timestamp => "timestamp",
            Self::SingleChoice => "single_choice",
            Self::MultiChoice => "multi_choice",
            Self::Other(value) => value.as_str(),
        }
    }

    const fn is_text_like(&self) -> bool {
        matches!(self, Self::Text | Self::StaticText)
    }

    const fn is_orderable(&self) -> bool {
        matches!(
            self,
            Self::Text
                | Self::StaticText
                | Self::Number
                | Self::Date
                | Self::DateTime
                | Self::Timestamp
                | Self::SingleChoice
                | Self::MultiChoice
        )
    }

    const fn is_comparable_range(&self) -> bool {
        matches!(
            self,
            Self::Number | Self::Date | Self::DateTime | Self::Timestamp
        )
    }
}

/// Operators shared by dataset filters and component table filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    Lt,
    Lte,
    Gt,
    Gte,
    Between,
    NotBetween,
    IsEmpty,
    IsNotEmpty,
    IsNull,
    IsNotNull,
}

impl FilterOperator {
    /// Parses the stable operator key used by APIs and component configs.
    pub fn parse(value: &str) -> Result<Self, DataOpError> {
        match value.trim() {
            "equals" => Ok(Self::Equals),
            "not_equals" => Ok(Self::NotEquals),
            "contains" => Ok(Self::Contains),
            "not_contains" => Ok(Self::NotContains),
            "lt" | "less_than" => Ok(Self::Lt),
            "lte" | "less_than_or_equal" => Ok(Self::Lte),
            "gt" | "greater_than" => Ok(Self::Gt),
            "gte" | "greater_than_or_equal" => Ok(Self::Gte),
            "between" => Ok(Self::Between),
            "not_between" => Ok(Self::NotBetween),
            "is_empty" => Ok(Self::IsEmpty),
            "is_not_empty" => Ok(Self::IsNotEmpty),
            "is_null" => Ok(Self::IsNull),
            "is_not_null" => Ok(Self::IsNotNull),
            other => Err(DataOpError::new(
                "UNSUPPORTED_FILTER_OPERATOR",
                format!("unsupported filter operator '{other}'"),
            )),
        }
    }

    /// Returns true when the operator needs one or more literal/field values.
    pub const fn requires_value(self) -> bool {
        matches!(
            self,
            Self::Equals
                | Self::NotEquals
                | Self::Contains
                | Self::NotContains
                | Self::Lt
                | Self::Lte
                | Self::Gt
                | Self::Gte
                | Self::Between
                | Self::NotBetween
        )
    }

    /// Validates operator compatibility with a field type.
    pub fn validate_for_field(self, field: &DataField) -> Result<(), DataOpError> {
        let valid = match self {
            Self::Contains | Self::NotContains | Self::IsEmpty | Self::IsNotEmpty => {
                field.field_type.is_text_like()
            }
            Self::Lt | Self::Lte | Self::Gt | Self::Gte | Self::Between | Self::NotBetween => {
                field.field_type.is_comparable_range()
            }
            Self::Equals | Self::NotEquals | Self::IsNull | Self::IsNotNull => true,
        };
        if valid {
            Ok(())
        } else {
            Err(DataOpError::new(
                "FILTER_OPERATOR_FIELD_TYPE_MISMATCH",
                format!(
                    "filter operator '{}' is not supported for field '{}' with type '{}'",
                    self.as_str(),
                    field.key,
                    field.field_type.as_str()
                ),
            ))
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::NotEquals => "not_equals",
            Self::Contains => "contains",
            Self::NotContains => "not_contains",
            Self::Lt => "lt",
            Self::Lte => "lte",
            Self::Gt => "gt",
            Self::Gte => "gte",
            Self::Between => "between",
            Self::NotBetween => "not_between",
            Self::IsEmpty => "is_empty",
            Self::IsNotEmpty => "is_not_empty",
            Self::IsNull => "is_null",
            Self::IsNotNull => "is_not_null",
        }
    }
}

/// Aggregate functions exposed to component authoring and shared operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregateFunction {
    Count,
    CountValues,
    CountDistinct,
    Sum,
    Avg,
    Min,
    Max,
}

impl AggregateFunction {
    /// Parses component-facing and legacy dataset function keys.
    pub fn parse(value: &str) -> Result<Self, DataOpError> {
        match value.trim() {
            "count" | "count_rows" => Ok(Self::Count),
            "count_values" => Ok(Self::CountValues),
            "count_distinct" => Ok(Self::CountDistinct),
            "sum" => Ok(Self::Sum),
            "avg" | "average" => Ok(Self::Avg),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            other => Err(DataOpError::new(
                "UNSUPPORTED_AGGREGATE_FUNCTION",
                format!("unsupported aggregate function '{other}'"),
            )),
        }
    }

    pub const fn requires_source_field(self) -> bool {
        !matches!(self, Self::Count)
    }

    pub fn validate_source_field(
        self,
        metric_key: &str,
        field: &DataField,
    ) -> Result<(), DataOpError> {
        let valid = match self {
            Self::Count => true,
            Self::CountValues => true,
            Self::CountDistinct => true,
            Self::Sum | Self::Avg => field.field_type == FieldType::Number,
            Self::Min | Self::Max => field.field_type.is_orderable(),
        };
        if valid {
            Ok(())
        } else {
            Err(DataOpError::new(
                "AGGREGATE_FUNCTION_FIELD_TYPE_MISMATCH",
                format!(
                    "aggregation metric '{metric_key}' cannot use field type '{}'",
                    field.field_type.as_str()
                ),
            ))
        }
    }

    pub fn output_field_type(self, source: Option<&DataField>) -> FieldType {
        match self {
            Self::Count | Self::CountValues | Self::CountDistinct | Self::Sum | Self::Avg => {
                FieldType::Number
            }
            Self::Min | Self::Max => source
                .map(|field| field.field_type.clone())
                .unwrap_or(FieldType::Text),
        }
    }
}

/// One aggregate metric in a shared aggregation plan.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AggregateMetric {
    pub key: String,
    pub label: String,
    pub function: AggregateFunction,
    pub source_field_key: Option<String>,
    pub position: i32,
}

/// Validated aggregate metric with resolved output type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedAggregateMetric {
    pub key: String,
    pub label: String,
    pub function: AggregateFunction,
    pub source_field_key: Option<String>,
    pub output_field_type: FieldType,
    pub position: i32,
}

/// Minimal shared aggregation plan.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct AggregationPlan {
    pub group_fields: Vec<String>,
    pub metrics: Vec<AggregateMetric>,
}

/// Validated aggregation plan over a known field contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedAggregationPlan {
    pub group_fields: Vec<String>,
    pub metrics: Vec<ValidatedAggregateMetric>,
}

/// Validates group fields and metric definitions against a field contract.
pub fn validate_aggregation_plan(
    plan: AggregationPlan,
    fields: &[DataField],
) -> Result<ValidatedAggregationPlan, DataOpError> {
    let field_by_key = fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<HashMap<_, _>>();

    let mut group_fields = Vec::new();
    let mut seen_groups = BTreeSet::new();
    for key in plan.group_fields {
        require_identifier("aggregation group field", &key)?;
        if !field_by_key.contains_key(key.as_str()) {
            return Err(DataOpError::new(
                "AGGREGATE_GROUP_FIELD_NOT_FOUND",
                format!("aggregation group field '{key}' is not in the field contract"),
            ));
        }
        if seen_groups.insert(key.clone()) {
            group_fields.push(key);
        }
    }

    let mut seen_metric_keys = BTreeSet::new();
    let mut metrics = Vec::new();
    for metric in plan.metrics {
        require_identifier("aggregation metric key", &metric.key)?;
        require_text("aggregation metric label", &metric.label)?;
        if !seen_metric_keys.insert(metric.key.clone()) {
            return Err(DataOpError::new(
                "AGGREGATE_DUPLICATE_METRIC_KEY",
                format!("aggregation metric key '{}' is duplicated", metric.key),
            ));
        }
        if field_by_key.contains_key(metric.key.as_str()) || seen_groups.contains(&metric.key) {
            return Err(DataOpError::new(
                "AGGREGATE_METRIC_KEY_CONFLICT",
                format!(
                    "aggregation metric key '{}' conflicts with an output field",
                    metric.key
                ),
            ));
        }

        let source_field = metric
            .source_field_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let source = match (metric.function.requires_source_field(), source_field) {
            (true, Some(key)) => Some(field_by_key.get(key).ok_or_else(|| {
                DataOpError::new(
                    "AGGREGATE_METRIC_FIELD_NOT_FOUND",
                    format!(
                        "aggregation metric '{}' references field '{}' outside the field contract",
                        metric.key, key
                    ),
                )
            })?),
            (true, None) => {
                return Err(DataOpError::new(
                    "AGGREGATE_METRIC_SOURCE_REQUIRED",
                    format!(
                        "aggregation metric '{}' requires a source field",
                        metric.key
                    ),
                ));
            }
            (false, Some(_)) => {
                return Err(DataOpError::new(
                    "AGGREGATE_METRIC_SOURCE_NOT_USED",
                    format!(
                        "aggregation metric '{}' does not use a source field",
                        metric.key
                    ),
                ));
            }
            (false, None) => None,
        };

        if let Some(field) = source {
            metric.function.validate_source_field(&metric.key, field)?;
        }

        metrics.push(ValidatedAggregateMetric {
            key: metric.key,
            label: metric.label,
            function: metric.function,
            source_field_key: source.map(|field| field.key.clone()),
            output_field_type: metric.function.output_field_type(source.copied()),
            position: metric.position,
        });
    }

    Ok(ValidatedAggregationPlan {
        group_fields,
        metrics,
    })
}

fn require_text(label: &str, value: &str) -> Result<(), DataOpError> {
    if value.trim().is_empty() {
        Err(DataOpError::new(
            "DATA_OP_REQUIRED_TEXT",
            format!("{label} is required"),
        ))
    } else {
        Ok(())
    }
}

fn require_identifier(label: &str, value: &str) -> Result<(), DataOpError> {
    require_text(label, value)?;
    let valid = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_');
    if valid {
        Ok(())
    } else {
        Err(DataOpError::new(
            "DATA_OP_INVALID_IDENTIFIER",
            format!("{label} '{value}' must contain only letters, numbers, and underscores"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AggregateFunction, AggregateMetric, AggregationPlan, DataField, FieldType, FilterOperator,
        validate_aggregation_plan,
    };

    fn field(key: &str, field_type: FieldType) -> DataField {
        DataField {
            key: key.into(),
            label: key.into(),
            field_type,
            position: 0,
        }
    }

    #[test]
    fn parses_component_and_legacy_dataset_aggregate_names() {
        assert_eq!(
            AggregateFunction::parse("count").expect("count should parse"),
            AggregateFunction::Count
        );
        assert_eq!(
            AggregateFunction::parse("count_values").expect("legacy count values should parse"),
            AggregateFunction::CountValues
        );
        assert_eq!(
            AggregateFunction::parse("count_rows").expect("legacy count should parse"),
            AggregateFunction::Count
        );
        assert_eq!(
            AggregateFunction::parse("average").expect("legacy average should parse"),
            AggregateFunction::Avg
        );
        assert_eq!(
            AggregateFunction::parse("count_distinct").expect("count distinct should parse"),
            AggregateFunction::CountDistinct
        );
    }

    #[test]
    fn validates_aggregation_group_and_metric_fields() {
        let fields = vec![
            field("program", FieldType::Text),
            field("amount", FieldType::Number),
        ];
        let plan = AggregationPlan {
            group_fields: vec!["program".into()],
            metrics: vec![AggregateMetric {
                key: "total_amount".into(),
                label: "Total amount".into(),
                function: AggregateFunction::Sum,
                source_field_key: Some("amount".into()),
                position: 0,
            }],
        };

        let validated =
            validate_aggregation_plan(plan, &fields).expect("valid aggregation should pass");
        assert_eq!(validated.group_fields, vec!["program"]);
        assert_eq!(validated.metrics[0].output_field_type, FieldType::Number);
    }

    #[test]
    fn rejects_duplicate_metric_keys() {
        let fields = vec![field("amount", FieldType::Number)];
        let plan = AggregationPlan {
            group_fields: Vec::new(),
            metrics: vec![
                AggregateMetric {
                    key: "total".into(),
                    label: "Total".into(),
                    function: AggregateFunction::Sum,
                    source_field_key: Some("amount".into()),
                    position: 0,
                },
                AggregateMetric {
                    key: "total".into(),
                    label: "Total again".into(),
                    function: AggregateFunction::Sum,
                    source_field_key: Some("amount".into()),
                    position: 1,
                },
            ],
        };

        let error =
            validate_aggregation_plan(plan, &fields).expect_err("duplicate metrics should fail");
        assert_eq!(error.code(), "AGGREGATE_DUPLICATE_METRIC_KEY");
    }

    #[test]
    fn rejects_sum_over_text_fields() {
        let fields = vec![field("name", FieldType::Text)];
        let plan = AggregationPlan {
            group_fields: Vec::new(),
            metrics: vec![AggregateMetric {
                key: "total_name".into(),
                label: "Total name".into(),
                function: AggregateFunction::Sum,
                source_field_key: Some("name".into()),
                position: 0,
            }],
        };

        let error =
            validate_aggregation_plan(plan, &fields).expect_err("sum over text should fail");
        assert_eq!(error.code(), "AGGREGATE_FUNCTION_FIELD_TYPE_MISMATCH");
    }

    #[test]
    fn validates_min_and_max_over_text_fields() {
        let fields = vec![field("name", FieldType::Text)];
        for function in [AggregateFunction::Min, AggregateFunction::Max] {
            let plan = AggregationPlan {
                group_fields: Vec::new(),
                metrics: vec![AggregateMetric {
                    key: format!("{function:?}_name").to_ascii_lowercase(),
                    label: format!("{function:?} name"),
                    function,
                    source_field_key: Some("name".into()),
                    position: 0,
                }],
            };

            let validated = validate_aggregation_plan(plan, &fields)
                .unwrap_or_else(|error| panic!("{function:?} over text should pass: {error}"));
            assert_eq!(validated.metrics[0].output_field_type, FieldType::Text);
        }
    }

    #[test]
    fn validates_filter_operator_field_compatibility() {
        let text = field("name", FieldType::Text);
        let number = field("amount", FieldType::Number);
        assert!(FilterOperator::Contains.validate_for_field(&text).is_ok());
        assert!(
            FilterOperator::Contains
                .validate_for_field(&number)
                .is_err()
        );
        assert!(FilterOperator::Between.validate_for_field(&number).is_ok());
    }
}
