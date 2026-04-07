//! Reporting domain logic for Tessara.
//!
//! This crate owns pure reporting concepts that are useful outside the HTTP
//! layer. Database-backed report execution still lives in `tessara-api` until
//! the query planner and repository seams stabilize.

use std::{collections::HashSet, fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use tessara_core::{RequiredTextError, validate_required_text};

/// Policy used when a logical report field has no value for a row.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingDataPolicy {
    /// Keep the row and expose the missing field as `null`.
    Null,
    /// Drop the row from the report output.
    ExcludeRow,
    /// Keep the row and place the missing value in an `Unknown` bucket.
    BucketUnknown,
}

impl MissingDataPolicy {
    /// Returns the canonical database/API string for this missing-data policy.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::ExcludeRow => "exclude_row",
            Self::BucketUnknown => "bucket_unknown",
        }
    }
}

impl fmt::Display for MissingDataPolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for MissingDataPolicy {
    type Err = MissingDataPolicyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "null" => Ok(Self::Null),
            "exclude_row" => Ok(Self::ExcludeRow),
            "bucket_unknown" => Ok(Self::BucketUnknown),
            other => Err(MissingDataPolicyError::Unsupported(other.to_string())),
        }
    }
}

/// Error returned when parsing a [`MissingDataPolicy`].
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum MissingDataPolicyError {
    /// The provided policy string is not supported.
    #[error("unsupported missing-data policy '{0}'")]
    Unsupported(String),
}

/// Borrowed field binding input from an API or import boundary.
#[derive(Debug, Clone, Copy)]
pub struct ReportFieldBindingInput<'a> {
    /// Logical report field name exposed to users.
    pub logical_key: &'a str,
    /// Physical source field key on compatible form versions.
    pub source_field_key: &'a str,
    /// Missing-data policy string, defaulting to [`MissingDataPolicy::Null`].
    pub missing_policy: Option<&'a str>,
}

/// Parsed report field binding safe to persist after database reference checks.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReportFieldBindingDraft {
    /// Logical report field name exposed to users.
    pub logical_key: String,
    /// Physical source field key on compatible form versions.
    pub source_field_key: String,
    /// Missing-data behavior for this binding.
    pub missing_policy: MissingDataPolicy,
}

/// Parses and validates report field bindings independent of database state.
///
/// This covers text presence, missing-data policy parsing, and duplicate
/// logical keys. The API layer remains responsible for checking whether source
/// field keys exist on the selected form.
pub fn parse_report_field_bindings<'a>(
    fields: impl IntoIterator<Item = ReportFieldBindingInput<'a>>,
) -> Result<Vec<ReportFieldBindingDraft>, ReportBindingError> {
    let mut logical_keys = HashSet::new();
    let mut parsed_fields = Vec::new();

    for field in fields {
        validate_required_text("report logical key", field.logical_key)?;
        validate_required_text("report source field key", field.source_field_key)?;
        if !logical_keys.insert(field.logical_key.to_string()) {
            return Err(ReportBindingError::DuplicateLogicalKey(
                field.logical_key.to_string(),
            ));
        }

        let missing_policy = field
            .missing_policy
            .map(MissingDataPolicy::from_str)
            .transpose()?
            .unwrap_or(MissingDataPolicy::Null);

        parsed_fields.push(ReportFieldBindingDraft {
            logical_key: field.logical_key.to_string(),
            source_field_key: field.source_field_key.to_string(),
            missing_policy,
        });
    }

    Ok(parsed_fields)
}

/// Error returned when validating report field binding definitions.
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum ReportBindingError {
    /// Required text was empty or whitespace.
    #[error(transparent)]
    RequiredText(#[from] RequiredTextError),
    /// Missing-data policy was unsupported.
    #[error(transparent)]
    MissingPolicy(#[from] MissingDataPolicyError),
    /// The same logical key appeared more than once in the report definition.
    #[error("report logical field '{0}' is duplicated")]
    DuplicateLogicalKey(String),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{MissingDataPolicy, ReportFieldBindingInput, parse_report_field_bindings};

    #[test]
    fn parses_supported_missing_data_policies() {
        for (raw, expected) in [
            ("null", MissingDataPolicy::Null),
            ("exclude_row", MissingDataPolicy::ExcludeRow),
            ("bucket_unknown", MissingDataPolicy::BucketUnknown),
        ] {
            assert_eq!(MissingDataPolicy::from_str(raw), Ok(expected));
            assert_eq!(expected.as_str(), raw);
        }
    }

    #[test]
    fn rejects_unknown_missing_data_policies() {
        assert!(MissingDataPolicy::from_str("drop_column").is_err());
    }

    #[test]
    fn parses_report_field_bindings() {
        let bindings = parse_report_field_bindings([ReportFieldBindingInput {
            logical_key: "participants",
            source_field_key: "participants_count",
            missing_policy: Some("bucket_unknown"),
        }])
        .expect("binding should parse");

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].logical_key, "participants");
        assert_eq!(bindings[0].source_field_key, "participants_count");
        assert_eq!(bindings[0].missing_policy, MissingDataPolicy::BucketUnknown);
    }

    #[test]
    fn rejects_invalid_report_field_bindings() {
        let blank = parse_report_field_bindings([ReportFieldBindingInput {
            logical_key: " ",
            source_field_key: "participants",
            missing_policy: None,
        }])
        .expect_err("blank logical key should fail");
        assert_eq!(blank.to_string(), "report logical key is required");

        let duplicate = parse_report_field_bindings([
            ReportFieldBindingInput {
                logical_key: "participants",
                source_field_key: "participants",
                missing_policy: None,
            },
            ReportFieldBindingInput {
                logical_key: "participants",
                source_field_key: "renamed_participants",
                missing_policy: None,
            },
        ])
        .expect_err("duplicate logical key should fail");
        assert_eq!(
            duplicate.to_string(),
            "report logical field 'participants' is duplicated"
        );
    }
}
