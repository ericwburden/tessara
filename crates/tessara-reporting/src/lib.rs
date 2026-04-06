//! Reporting domain logic for Tessara.
//!
//! This crate owns pure reporting concepts that are useful outside the HTTP
//! layer. Database-backed report execution still lives in `tessara-api` until
//! the query planner and repository seams stabilize.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::MissingDataPolicy;

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
}
