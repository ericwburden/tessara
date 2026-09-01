//! Display formatting helpers for the Responses feature.
//!
//! Keep label, class, and summary formatting here when it depends on Responses domain values but not on route state.

use crate::metadata::metadata_label;
use crate::text::nonempty_text;
use crate::types::{
    RESPONSE_FORM_GRID_COLUMN_COUNT, ResponseFormField, ResponseStartOption, ResponseStartOptions,
    ResponseSummary,
};
use leptos::prelude::*;
use serde_json::Value;

pub(crate) fn response_status_key(response: &ResponseSummary) -> String {
    response.status.trim().to_lowercase()
}

pub(crate) fn response_status_label(response: &ResponseSummary) -> String {
    metadata_label(&response.status)
}

pub(crate) fn workflow_revision_label_from_raw(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }

    if let Ok(revision) = trimmed.parse::<u64>() {
        return revision.to_string();
    }

    trimmed
        .split('.')
        .next()
        .and_then(|part| part.trim().parse::<u64>().ok())
        .map(|revision| revision.to_string())
        .unwrap_or_else(|| trimmed.to_string())
}

pub(crate) fn workflow_revision_label_from_option(label: Option<String>) -> String {
    label
        .as_deref()
        .map(workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn response_workflow_label(response: &ResponseSummary) -> String {
    nonempty_text(response.workflow_name.as_deref(), "Standalone Response")
}

pub(crate) fn response_assignee_label(response: &ResponseSummary) -> String {
    nonempty_text(response.assigned_to_display_name.as_deref(), "Unassigned")
}

pub(crate) fn response_step_label(response: &ResponseSummary) -> String {
    if response_status_key(response) == "submitted" {
        return "No active step".to_string();
    }
    let title = nonempty_text(
        response.current_workflow_step_title.as_deref(),
        "No active step",
    );
    match (
        response.workflow_step_position,
        response.workflow_step_count,
    ) {
        (Some(position), Some(count)) if count > 0 => {
            format!("Step {} of {count}: {title}", position + 1)
        }
        _ => title,
    }
}

pub(crate) fn response_progress_label(response: &ResponseSummary) -> String {
    if response_status_key(response) == "submitted"
        && let Some(count) = response.workflow_step_count.filter(|count| *count > 0)
    {
        return format!("{count} of {count} completed");
    }
    match (
        response.workflow_steps_completed,
        response.workflow_step_count,
    ) {
        (Some(completed), Some(count)) if count > 0 => format!("{completed} of {count} completed"),
        _ => format!("{} saved values", response.value_count),
    }
}

pub(crate) fn response_selected_assignment(
    options: RwSignal<Option<ResponseStartOptions>>,
    selected_assignment_index: RwSignal<String>,
) -> Option<ResponseStartOption> {
    let index = selected_assignment_index.get().parse::<usize>().ok()?;
    options
        .get()
        .and_then(|options| options.assignments.get(index).cloned())
}

pub(crate) fn response_start_can_submit(
    options: RwSignal<Option<ResponseStartOptions>>,
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

pub(crate) fn rendered_form_field_layout_style(field: &ResponseFormField) -> String {
    let width = field.grid_width.clamp(1, RESPONSE_FORM_GRID_COLUMN_COUNT);
    let max_column = (RESPONSE_FORM_GRID_COLUMN_COUNT - width + 1).max(1);
    let column = field.grid_column.clamp(1, max_column);
    let row = field.grid_row.max(1);
    let height = field.grid_height.max(1);
    let control_min_height = 2.65 + ((height - 1) as f32 * 1.0);

    format!(
        "--response-field-column: {column}; --response-field-width: {width}; --response-field-row: {row}; --response-field-height: {height}; --response-control-min-height: {control_min_height:.2}rem;",
    )
}

pub(crate) fn response_field_class(field_type: &str) -> String {
    format!(
        "form-field response-form-field response-form-field--{}",
        field_type.replace('_', "-")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(status: &str) -> ResponseSummary {
        ResponseSummary {
            id: uuid::Uuid::from_u128(1),
            form_id: uuid::Uuid::from_u128(2),
            form_version_id: uuid::Uuid::from_u128(3),
            form_name: "Primary Responses".into(),
            workflow_name: Some("Primary Responses Workflow".into()),
            workflow_description: None,
            workflow_step_position: Some(0),
            workflow_step_count: Some(1),
            workflow_steps_completed: Some(0),
            current_workflow_step_title: Some("Primary Responses Response".into()),
            next_workflow_step_title: None,
            next_workflow_step_form_name: None,
            assigned_to_display_name: Some("Response Owner".into()),
            version_label: "1.0.0".into(),
            node_id: uuid::Uuid::from_u128(4),
            node_name: "Reference Organization".into(),
            status: status.into(),
            value_count: 3,
            created_at: "2026-08-31T00:00:00Z".into(),
            last_modified_at: "2026-08-31T00:00:00Z".into(),
            submitted_at: None,
        }
    }

    #[test]
    fn submitted_response_presents_the_completed_assignment() {
        let response = response("submitted");

        assert_eq!(response_step_label(&response), "No active step");
        assert_eq!(response_progress_label(&response), "1 of 1 completed");
    }

    #[test]
    fn draft_response_preserves_current_step_progress() {
        let response = response("draft");

        assert_eq!(
            response_step_label(&response),
            "Step 1 of 1: Primary Responses Response"
        );
        assert_eq!(response_progress_label(&response), "0 of 1 completed");
    }
}
