//! Owns the features::responses::display module behavior.

use crate::features::forms::RenderedField;
use crate::features::forms::builder::FORM_BUILDER_COLUMN_COUNT;
use crate::features::responses::types::{
    AssignmentResponseStartOption, AssignmentResponseStartOptions, SubmissionSummary,
};
use crate::utils::metadata::metadata_label;
use crate::utils::text::nonempty_text;
use leptos::prelude::*;
use serde_json::Value;
/// Handles the submission status key behavior.
pub(crate) fn submission_status_key(submission: &SubmissionSummary) -> String {
    submission.status.trim().to_lowercase()
}

/// Handles the submission status label behavior.
pub(crate) fn submission_status_label(submission: &SubmissionSummary) -> String {
    metadata_label(&submission.status)
}

/// Handles the submission workflow label behavior.
pub(crate) fn submission_workflow_label(submission: &SubmissionSummary) -> String {
    nonempty_text(submission.workflow_name.as_deref(), "Standalone Response")
}

/// Handles the submission assignee label behavior.
pub(crate) fn submission_assignee_label(submission: &SubmissionSummary) -> String {
    nonempty_text(submission.assigned_to_display_name.as_deref(), "Unassigned")
}

/// Handles the submission step label behavior.
pub(crate) fn submission_step_label(submission: &SubmissionSummary) -> String {
    let title = nonempty_text(
        submission.current_workflow_step_title.as_deref(),
        "No active step",
    );
    match (
        submission.workflow_step_position,
        submission.workflow_step_count,
    ) {
        (Some(position), Some(count)) if count > 0 => {
            format!("Step {} of {count}: {title}", position + 1)
        }
        _ => title,
    }
}

/// Handles the submission progress label behavior.
pub(crate) fn submission_progress_label(submission: &SubmissionSummary) -> String {
    match (
        submission.workflow_steps_completed,
        submission.workflow_step_count,
    ) {
        (Some(completed), Some(count)) if count > 0 => format!("{completed} of {count} completed"),
        _ => format!("{} saved values", submission.value_count),
    }
}

/// Handles the response selected assignment behavior.
pub(crate) fn response_selected_assignment(
    options: RwSignal<Option<AssignmentResponseStartOptions>>,
    selected_assignment_index: RwSignal<String>,
) -> Option<AssignmentResponseStartOption> {
    let index = selected_assignment_index.get().parse::<usize>().ok()?;
    options
        .get()
        .and_then(|options| options.assignments.get(index).cloned())
}

/// Handles the response start can submit behavior.
pub(crate) fn response_start_can_submit(
    options: RwSignal<Option<AssignmentResponseStartOptions>>,
    is_loading: RwSignal<bool>,
    is_saving: RwSignal<bool>,
    selected_assignment_index: RwSignal<String>,
) -> bool {
    if is_loading.get() || is_saving.get() {
        return false;
    }

    if let Some(loaded_options) = options.get() {
        !loaded_options.assignments.is_empty()
            && response_selected_assignment(options, selected_assignment_index).is_some()
    } else {
        false
    }
}

/// Handles the response value label behavior.
pub(crate) fn response_value_label(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "Missing".into(),
        Some(Value::String(value)) if value.trim().is_empty() => "Missing".into(),
        Some(Value::String(value)) => value.clone(),
        Some(Value::Bool(value)) => {
            if *value {
                "Yes".into()
            } else {
                "No".into()
            }
        }
        Some(Value::Array(values)) if values.is_empty() => "Missing".into(),
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>()
            .join(", "),
        Some(value) => value.to_string(),
    }
}

/// Handles the rendered form field layout style behavior.
pub(crate) fn rendered_form_field_layout_style(field: &RenderedField) -> String {
    let width = field.grid_width.clamp(1, FORM_BUILDER_COLUMN_COUNT);
    let max_column = (FORM_BUILDER_COLUMN_COUNT - width + 1).max(1);
    let column = field.grid_column.clamp(1, max_column);
    let row = field.grid_row.max(1);
    let height = field.grid_height.max(1);
    let control_min_height = 2.65 + ((height - 1) as f32 * 1.0);

    format!(
        "--response-field-column: {column}; --response-field-width: {width}; --response-field-row: {row}; --response-field-height: {height}; --response-control-min-height: {control_min_height:.2}rem;",
    )
}

/// Handles the response field class behavior.
pub(crate) fn response_field_class(field_type: &str) -> String {
    format!(
        "form-field response-form-field response-form-field--{}",
        field_type.replace('_', "-")
    )
}
