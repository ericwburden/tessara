//! Shared Tessara primitives.
//!
//! This crate owns concepts that are intentionally shared across bounded
//! contexts. Keep this crate small: add types here only when multiple contexts
//! need the same semantic contract.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Validates that user-configurable text is present after trimming.
///
/// Tessara builders use this at API boundaries for names, slugs, keys, and
/// labels that would otherwise persist as ambiguous blank configuration.
pub fn validate_required_text(
    field_name: &'static str,
    value: &str,
) -> Result<(), RequiredTextError> {
    if value.trim().is_empty() {
        Err(RequiredTextError { field_name })
    } else {
        Ok(())
    }
}

/// Error returned when required builder text is empty or whitespace.
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
#[error("{field_name} is required")]
pub struct RequiredTextError {
    /// User-facing field name that failed validation.
    pub field_name: &'static str,
}

/// Typed value category used by configurable metadata and form fields.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    /// Free-form text stored as a JSON string.
    Text,
    /// Numeric value stored as a JSON number.
    Number,
    /// Boolean value stored as a JSON boolean.
    Boolean,
    /// Calendar date stored as an ISO-like JSON string for the current slice.
    Date,
    /// Single selected choice stored as a JSON string.
    SingleChoice,
    /// Multiple selected choices stored as a JSON array of strings.
    MultiChoice,
}

impl FieldType {
    /// Returns the canonical database/API string for this field type.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Date => "date",
            Self::SingleChoice => "single_choice",
            Self::MultiChoice => "multi_choice",
        }
    }

    /// Validates that a JSON value is compatible with this field type.
    ///
    /// Draft workflows can decide whether a value is required, but when a value
    /// is present it must match the configured type.
    pub fn validate_json_value(self, value: &Value) -> Result<(), FieldTypeError> {
        let valid = match self {
            Self::Text | Self::Date | Self::SingleChoice => value.is_string(),
            Self::Number => value.is_number(),
            Self::Boolean => value.is_boolean(),
            Self::MultiChoice => value
                .as_array()
                .map(|items| items.iter().all(Value::is_string))
                .unwrap_or(false),
        };

        if valid {
            Ok(())
        } else {
            Err(FieldTypeError::InvalidValue { field_type: self })
        }
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for FieldType {
    type Err = FieldTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "text" => Ok(Self::Text),
            "number" => Ok(Self::Number),
            "boolean" => Ok(Self::Boolean),
            "date" => Ok(Self::Date),
            "single_choice" => Ok(Self::SingleChoice),
            "multi_choice" => Ok(Self::MultiChoice),
            other => Err(FieldTypeError::Unsupported(other.to_string())),
        }
    }
}

/// Error returned when parsing or validating a [`FieldType`].
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum FieldTypeError {
    /// The provided field type string is not part of Tessara's current type set.
    #[error("unsupported field type '{0}'")]
    Unsupported(String),
    /// The JSON value is not compatible with the field type.
    #[error("value does not match field type '{field_type}'")]
    InvalidValue {
        /// Field type used for validation.
        field_type: FieldType,
    },
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use serde_json::json;

    use super::{FieldType, validate_required_text};

    #[test]
    fn validates_required_text() {
        assert!(validate_required_text("name", "Quarterly Report").is_ok());
        assert_eq!(
            validate_required_text("name", "   ")
                .expect_err("blank text should fail")
                .to_string(),
            "name is required"
        );
    }

    #[test]
    fn parses_supported_field_types() {
        for (raw, expected) in [
            ("text", FieldType::Text),
            ("number", FieldType::Number),
            ("boolean", FieldType::Boolean),
            ("date", FieldType::Date),
            ("single_choice", FieldType::SingleChoice),
            ("multi_choice", FieldType::MultiChoice),
        ] {
            assert_eq!(FieldType::from_str(raw), Ok(expected));
            assert_eq!(expected.as_str(), raw);
        }

        assert!(FieldType::from_str("file_upload").is_err());
    }

    #[test]
    fn validates_json_values_against_field_types() {
        assert!(FieldType::Text.validate_json_value(&json!("hello")).is_ok());
        assert!(
            FieldType::Date
                .validate_json_value(&json!("2026-04-06"))
                .is_ok()
        );
        assert!(
            FieldType::SingleChoice
                .validate_json_value(&json!("yes"))
                .is_ok()
        );
        assert!(FieldType::Number.validate_json_value(&json!(42)).is_ok());
        assert!(FieldType::Boolean.validate_json_value(&json!(true)).is_ok());
        assert!(
            FieldType::MultiChoice
                .validate_json_value(&json!(["a", "b"]))
                .is_ok()
        );
    }

    #[test]
    fn rejects_json_values_that_do_not_match_field_types() {
        assert!(FieldType::Text.validate_json_value(&json!(42)).is_err());
        assert!(FieldType::Number.validate_json_value(&json!("42")).is_err());
        assert!(
            FieldType::Boolean
                .validate_json_value(&json!("true"))
                .is_err()
        );
        assert!(
            FieldType::MultiChoice
                .validate_json_value(&json!(["a", 2]))
                .is_err()
        );
    }
}
