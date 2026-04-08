//! Dataset domain rules for Tessara reporting.
//!
//! Datasets define the semantic row model that reports query. This crate keeps
//! pure dataset validation separate from API/database orchestration.

use std::collections::HashSet;

/// Error returned when a dataset rule is violated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetRuleError {
    message: String,
}

impl DatasetRuleError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Human-readable rule violation message suitable for API diagnostics.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for DatasetRuleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for DatasetRuleError {}

/// Dataset row grain supported by the current implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetGrain {
    /// One row per submitted form submission.
    Submission,
    /// One row per runtime hierarchy node.
    Node,
}

impl DatasetGrain {
    /// Parses a dataset grain from the API/storage representation.
    pub fn parse(value: &str) -> Result<Self, DatasetRuleError> {
        match value.trim() {
            "submission" => Ok(Self::Submission),
            "node" => Ok(Self::Node),
            other => Err(DatasetRuleError::new(format!(
                "unsupported dataset grain '{other}'"
            ))),
        }
    }

    /// Returns the stable storage representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Submission => "submission",
            Self::Node => "node",
        }
    }
}

/// Dataset source composition mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetCompositionMode {
    /// Union rows from all configured sources into one dataset row stream.
    Union,
    /// Join rows from multiple sources into one semantic row.
    Join,
}

impl DatasetCompositionMode {
    /// Parses a dataset composition mode from the API/storage representation.
    pub fn parse(value: &str) -> Result<Self, DatasetRuleError> {
        match value.trim() {
            "union" => Ok(Self::Union),
            "join" => Ok(Self::Join),
            other => Err(DatasetRuleError::new(format!(
                "unsupported dataset composition mode '{other}'"
            ))),
        }
    }

    /// Returns the stable storage representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Union => "union",
            Self::Join => "join",
        }
    }
}

/// Dataset source record-selection rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetSelectionRule {
    /// Include every matching record from the source.
    All,
    /// Select the latest matching record per dataset grain.
    Latest,
    /// Select the earliest matching record per dataset grain.
    Earliest,
}

impl DatasetSelectionRule {
    /// Parses a dataset source selection rule from the API/storage representation.
    pub fn parse(value: &str) -> Result<Self, DatasetRuleError> {
        match value.trim() {
            "all" => Ok(Self::All),
            "latest" => Ok(Self::Latest),
            "earliest" => Ok(Self::Earliest),
            other => Err(DatasetRuleError::new(format!(
                "unsupported dataset selection rule '{other}'"
            ))),
        }
    }

    /// Returns the stable storage representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Latest => "latest",
            Self::Earliest => "earliest",
        }
    }
}

/// Validates the shape of a dataset definition before database-specific checks.
pub fn validate_dataset_shape<'a>(
    source_aliases: impl IntoIterator<Item = &'a str>,
    field_keys: impl IntoIterator<Item = &'a str>,
) -> Result<(), DatasetRuleError> {
    let mut aliases = HashSet::new();
    for alias in source_aliases {
        let trimmed = alias.trim();
        if trimmed.is_empty() {
            return Err(DatasetRuleError::new(
                "dataset sources require a non-empty alias",
            ));
        }
        if !aliases.insert(trimmed.to_owned()) {
            return Err(DatasetRuleError::new(format!(
                "dataset source alias '{trimmed}' is duplicated"
            )));
        }
    }

    if aliases.is_empty() {
        return Err(DatasetRuleError::new(
            "a dataset requires at least one source",
        ));
    }

    let mut keys = HashSet::new();
    for key in field_keys {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(DatasetRuleError::new(
                "dataset fields require a non-empty key",
            ));
        }
        if !keys.insert(trimmed.to_owned()) {
            return Err(DatasetRuleError::new(format!(
                "dataset field key '{trimmed}' is duplicated"
            )));
        }
    }

    if keys.is_empty() {
        return Err(DatasetRuleError::new(
            "a dataset requires at least one exposed field",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        DatasetCompositionMode, DatasetGrain, DatasetSelectionRule, validate_dataset_shape,
    };

    #[test]
    fn parses_supported_dataset_grains() {
        assert_eq!(
            DatasetGrain::parse("submission").expect("submission should parse"),
            DatasetGrain::Submission
        );
        assert_eq!(
            DatasetGrain::parse("node").expect("node should parse"),
            DatasetGrain::Node
        );
        assert_eq!(
            DatasetGrain::parse("client")
                .expect_err("unsupported grains should fail")
                .message(),
            "unsupported dataset grain 'client'"
        );
    }

    #[test]
    fn parses_supported_selection_rules() {
        assert_eq!(
            DatasetSelectionRule::parse("all").expect("all should parse"),
            DatasetSelectionRule::All
        );
        assert_eq!(
            DatasetSelectionRule::parse("latest").expect("latest should parse"),
            DatasetSelectionRule::Latest
        );
        assert_eq!(
            DatasetSelectionRule::parse("earliest").expect("earliest should parse"),
            DatasetSelectionRule::Earliest
        );
    }

    #[test]
    fn parses_supported_composition_modes() {
        assert_eq!(
            DatasetCompositionMode::parse("union").expect("union should parse"),
            DatasetCompositionMode::Union
        );
        assert_eq!(
            DatasetCompositionMode::parse("join").expect("join should parse"),
            DatasetCompositionMode::Join
        );
        assert_eq!(
            DatasetCompositionMode::parse("merge")
                .expect_err("unsupported composition modes should fail")
                .message(),
            "unsupported dataset composition mode 'merge'"
        );
    }

    #[test]
    fn dataset_shape_rejects_empty_and_duplicate_keys() {
        assert!(validate_dataset_shape(["intake"], ["participant_count"]).is_ok());
        assert_eq!(
            validate_dataset_shape(["intake", "intake"], ["participant_count"])
                .expect_err("duplicate aliases should fail")
                .message(),
            "dataset source alias 'intake' is duplicated"
        );
        assert_eq!(
            validate_dataset_shape(["intake"], ["participant_count", "participant_count"])
                .expect_err("duplicate fields should fail")
                .message(),
            "dataset field key 'participant_count' is duplicated"
        );
    }
}
