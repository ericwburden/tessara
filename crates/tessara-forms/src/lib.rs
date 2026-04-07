//! Form definition domain logic for Tessara.
//!
//! This crate owns pure rules for form families, versions, fields, sections,
//! choice lists, and compatibility groups. Database-backed orchestration stays
//! in `tessara-api`, but stable validation behavior should live here.

/// Error returned when a form lifecycle rule is violated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormRuleError {
    message: &'static str,
}

impl FormRuleError {
    const fn new(message: &'static str) -> Self {
        Self { message }
    }

    /// Human-readable rule violation message suitable for API diagnostics.
    pub const fn message(&self) -> &'static str {
        self.message
    }
}

impl std::fmt::Display for FormRuleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for FormRuleError {}

/// Ensures a form version can still be modified through the form builder.
pub fn ensure_form_version_editable(status: &str) -> Result<(), FormRuleError> {
    if status == "draft" {
        Ok(())
    } else {
        Err(FormRuleError::new(
            "published form versions cannot be modified",
        ))
    }
}

/// Ensures a form version has the minimum structure required for publication.
pub fn ensure_form_version_publishable(
    status: &str,
    section_count: i64,
    field_count: i64,
) -> Result<(), FormRuleError> {
    if status != "draft" {
        return Err(FormRuleError::new(
            "only draft form versions can be published",
        ));
    }

    if section_count == 0 {
        return Err(FormRuleError::new(
            "cannot publish a form version without sections",
        ));
    }

    if field_count == 0 {
        return Err(FormRuleError::new(
            "cannot publish a form version without fields",
        ));
    }

    Ok(())
}

/// Ensures a field is being attached to a section on the same form version.
pub fn ensure_section_belongs_to_form_version(matches: bool) -> Result<(), FormRuleError> {
    if matches {
        Ok(())
    } else {
        Err(FormRuleError::new(
            "field section must belong to the same form version",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_form_version_editable, ensure_form_version_publishable,
        ensure_section_belongs_to_form_version,
    };

    #[test]
    fn editable_form_versions_must_be_drafts() {
        assert!(ensure_form_version_editable("draft").is_ok());
        assert_eq!(
            ensure_form_version_editable("published")
                .expect_err("published versions should not be editable")
                .message(),
            "published form versions cannot be modified"
        );
    }

    #[test]
    fn publishable_form_versions_must_be_structured_drafts() {
        assert!(ensure_form_version_publishable("draft", 1, 1).is_ok());
        assert_eq!(
            ensure_form_version_publishable("published", 1, 1)
                .expect_err("only drafts should publish")
                .message(),
            "only draft form versions can be published"
        );
        assert_eq!(
            ensure_form_version_publishable("draft", 0, 1)
                .expect_err("sections should be required")
                .message(),
            "cannot publish a form version without sections"
        );
        assert_eq!(
            ensure_form_version_publishable("draft", 1, 0)
                .expect_err("fields should be required")
                .message(),
            "cannot publish a form version without fields"
        );
    }

    #[test]
    fn fields_must_use_sections_from_the_same_form_version() {
        assert!(ensure_section_belongs_to_form_version(true).is_ok());
        assert_eq!(
            ensure_section_belongs_to_form_version(false)
                .expect_err("cross-version sections should be rejected")
                .message(),
            "field section must belong to the same form version"
        );
    }
}
