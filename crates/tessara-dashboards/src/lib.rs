//! Dashboard and chart domain logic for Tessara.
//!
//! This crate owns pure dashboard concepts that are useful outside the HTTP
//! layer. Database-backed dashboard persistence still lives in `tessara-api`
//! until the repository seams stabilize.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Chart presentation type supported by the current dashboard slice.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    /// Tabular report output.
    Table,
    /// Bar chart visualization.
    Bar,
    /// Single summary metric visualization.
    Summary,
}

impl ChartType {
    /// Returns the canonical database/API string for this chart type.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Bar => "bar",
            Self::Summary => "summary",
        }
    }
}

impl fmt::Display for ChartType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ChartType {
    type Err = ChartTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "table" => Ok(Self::Table),
            "bar" => Ok(Self::Bar),
            "summary" => Ok(Self::Summary),
            other => Err(ChartTypeError::Unsupported(other.to_string())),
        }
    }
}

/// Error returned when parsing a [`ChartType`].
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum ChartTypeError {
    /// The provided chart type string is not supported.
    #[error("unsupported chart type '{0}'")]
    Unsupported(String),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::ChartType;

    #[test]
    fn parses_supported_chart_types() {
        for (raw, expected) in [
            ("table", ChartType::Table),
            ("bar", ChartType::Bar),
            ("summary", ChartType::Summary),
        ] {
            assert_eq!(ChartType::from_str(raw), Ok(expected));
            assert_eq!(expected.as_str(), raw);
        }
    }

    #[test]
    fn rejects_unknown_chart_types() {
        assert!(ChartType::from_str("scatter").is_err());
    }
}
