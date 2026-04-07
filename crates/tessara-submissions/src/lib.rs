//! Submission workflow domain logic for Tessara.
//!
//! This crate owns pure rules for assignments, drafts, submit transitions,
//! audit events, and submission validation. Database-backed orchestration stays
//! in `tessara-api`, but stable workflow behavior should live here.

/// Error returned when a submission workflow rule is violated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionRuleError {
    message: String,
}

impl SubmissionRuleError {
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

impl std::fmt::Display for SubmissionRuleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SubmissionRuleError {}

/// Lightweight field state used to validate submission completeness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequiredFieldStatus<'a> {
    /// Stable form field key shown in validation diagnostics.
    pub key: &'a str,
    /// Whether the field is required by the published form version.
    pub required: bool,
    /// Whether the submission has a saved value for the field.
    pub has_value: bool,
}

/// Ensures a form version can be used to create a submission draft.
pub fn ensure_form_version_accepts_submission(status: &str) -> Result<(), SubmissionRuleError> {
    if status == "published" {
        Ok(())
    } else {
        Err(SubmissionRuleError::new(
            "submissions can only use published form versions",
        ))
    }
}

/// Ensures a submission can still be edited or submitted.
pub fn ensure_submission_is_draft(status: &str) -> Result<(), SubmissionRuleError> {
    if status == "draft" {
        Ok(())
    } else {
        Err(SubmissionRuleError::new(
            "submitted records are immutable in the initial workflow",
        ))
    }
}

/// Ensures all required fields have saved values before final submission.
pub fn ensure_required_values_present<'a>(
    fields: impl IntoIterator<Item = RequiredFieldStatus<'a>>,
) -> Result<(), SubmissionRuleError> {
    for field in fields {
        if field.required && !field.has_value {
            return Err(SubmissionRuleError::new(format!(
                "required field '{}' is missing",
                field.key
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        RequiredFieldStatus, ensure_form_version_accepts_submission,
        ensure_required_values_present, ensure_submission_is_draft,
    };

    #[test]
    fn drafts_can_only_target_published_form_versions() {
        assert!(ensure_form_version_accepts_submission("published").is_ok());
        assert_eq!(
            ensure_form_version_accepts_submission("draft")
                .expect_err("draft form versions should not accept submissions")
                .message(),
            "submissions can only use published form versions"
        );
    }

    #[test]
    fn submitted_records_are_not_editable() {
        assert!(ensure_submission_is_draft("draft").is_ok());
        assert_eq!(
            ensure_submission_is_draft("submitted")
                .expect_err("submitted records should be immutable")
                .message(),
            "submitted records are immutable in the initial workflow"
        );
    }

    #[test]
    fn required_values_must_be_present_before_submit() {
        assert!(
            ensure_required_values_present([
                RequiredFieldStatus {
                    key: "participants",
                    required: true,
                    has_value: true,
                },
                RequiredFieldStatus {
                    key: "notes",
                    required: false,
                    has_value: false,
                },
            ])
            .is_ok()
        );

        assert_eq!(
            ensure_required_values_present([RequiredFieldStatus {
                key: "participants",
                required: true,
                has_value: false,
            }])
            .expect_err("missing required values should be rejected")
            .message(),
            "required field 'participants' is missing"
        );
    }
}
