use crate::features::form_builder::FORM_BUILDER_COLUMN_COUNT;
use crate::features::organization::OrganizationNode;
use crate::features::shared_data as shared;
use crate::features::workflows::submission::{
    AssignmentResponseStartOption, AssignmentResponseStartOptions, FormBuilderFieldDraft,
    FormBuilderSectionDraft, SubmissionDetail, SubmissionSummary, WorkflowAssigneeOption,
    WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};
use crate::ui::empty_view;
use crate::utils::metadata::metadata_label;
use crate::utils::text::{nonempty_text, sentence_label};
use icons::{
    CalendarDays, CircleDot, FileText, Hash, ListChecks, SquareCheckBig, TextCursorInput, TextQuote,
};
use leptos::prelude::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

type FormVersionSummary = shared::FormVersionSummary;
type WorkflowSummary = shared::WorkflowSummary;
type WorkflowAvailableNodeSummary = shared::WorkflowAvailableNodeSummary;
type WorkflowDefinition = shared::WorkflowDefinition;
type WorkflowVersionSummary = shared::WorkflowVersionSummary;
type FormDefinition = shared::FormDefinition;
type RenderedForm = shared::RenderedForm;
type RenderedField = shared::RenderedField;
type FormAttachmentLink = shared::FormAttachmentLink;
pub(crate) fn form_version_desc_sort_key(version: &FormVersionSummary) -> (i32, i32, i32, String) {
    (
        version.version_major.unwrap_or(-1),
        version.version_minor.unwrap_or(-1),
        version.version_patch.unwrap_or(-1),
        version.published_at.clone().unwrap_or_default(),
    )
}

pub(crate) fn form_field_count_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| version.field_count.to_string())
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn form_definition_scope_label(form: &FormDefinition) -> String {
    nonempty_text(form.scope_node_type_name.as_deref(), "All node types")
}

pub(crate) fn node_display_path(node: &OrganizationNode) -> String {
    node.parent_node_name
        .as_deref()
        .map(|parent| format!("{parent} / {}", node.name))
        .unwrap_or_else(|| node.name.clone())
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

pub(crate) fn workflow_version_label(workflow: &WorkflowSummary) -> String {
    workflow
        .current_version_label
        .as_deref()
        .map(workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn workflow_status_key(workflow: &WorkflowSummary) -> &str {
    workflow.current_status.as_deref().unwrap_or("none")
}

pub(crate) fn workflow_status_label(workflow: &WorkflowSummary) -> String {
    workflow
        .current_status
        .as_deref()
        .map(sentence_label)
        .unwrap_or_else(|| "No revisions".to_string())
}

pub(crate) fn workflow_description_label(workflow: &WorkflowSummary) -> String {
    nonempty_text(Some(workflow.description.as_str()), "No description")
}

pub(crate) fn workflow_available_nodes_label(nodes: &[WorkflowAvailableNodeSummary]) -> String {
    match nodes.len() {
        1 => nodes[0].name.clone(),
        2 => nodes
            .iter()
            .map(|node| node.name.clone())
            .collect::<Vec<_>>()
            .join(", "),
        count => format!("{count} nodes"),
    }
}

pub(crate) fn node_count_label(count: usize) -> String {
    if count == 1 {
        "1 Node".to_string()
    } else {
        format!("{count} Nodes")
    }
}

pub(crate) fn user_count_label(count: usize) -> String {
    if count == 1 {
        "1 User".to_string()
    } else {
        format!("{count} Users")
    }
}

pub(crate) fn assignment_count_label(count: usize) -> String {
    if count == 1 {
        "1 Assignment".to_string()
    } else {
        format!("{count} Assignments")
    }
}

pub(crate) fn workflow_source_label(source: &str) -> Option<&'static str> {
    if source == "generated_form" {
        Some("Generated single-form")
    } else {
        None
    }
}

#[component]
pub(crate) fn WorkflowSourceMarker(source: String) -> impl IntoView {
    if source == "generated_form" {
        view! {
            <span
                class="workflow-source-marker"
                title="Single-Form, Generated Workflow"
                aria-label="Single-Form, Generated Workflow"
            >
                <FileText class="workflow-source-marker__icon"/>
                <span>"Single-form"</span>
            </span>
        }
        .into_any()
    } else {
        empty_view()
    }
}

pub(crate) fn workflow_assigned_user_links(workflow: &WorkflowSummary) -> Vec<FormAttachmentLink> {
    workflow
        .assigned_users
        .iter()
        .map(|user| FormAttachmentLink {
            href: format!("/administration/users/{}", user.id),
            label: user.display_name.clone(),
            title: format!(
                "{} - {}",
                user.email,
                assignment_count_label(user.assignment_count.max(0) as usize)
            ),
        })
        .collect()
}

pub(crate) fn workflow_available_node_links(
    nodes: &[WorkflowAvailableNodeSummary],
) -> Vec<FormAttachmentLink> {
    nodes
        .iter()
        .map(|node| FormAttachmentLink {
            href: format!("/organization/{}", node.id),
            label: node.name.clone(),
            title: format!("{} - {}", node.node_type_name, node.path),
        })
        .collect()
}

pub(crate) fn workflow_assignment_state(assignment: &WorkflowAssignmentSummary) -> &'static str {
    if assignment.has_submitted {
        "submitted"
    } else if assignment.has_draft {
        "draft"
    } else {
        "pending"
    }
}

pub(crate) fn workflow_assignment_state_label(
    assignment: &WorkflowAssignmentSummary,
) -> &'static str {
    match workflow_assignment_state(assignment) {
        "submitted" => "Submitted",
        "draft" => "Draft Exists",
        _ => "Pending",
    }
}

pub(crate) fn workflow_assignment_status_key(
    assignment: &WorkflowAssignmentSummary,
) -> &'static str {
    if assignment.is_active {
        "active"
    } else {
        "inactive"
    }
}

pub(crate) fn workflow_assignment_status_label(
    assignment: &WorkflowAssignmentSummary,
) -> &'static str {
    if assignment.is_active {
        "Active"
    } else {
        "Inactive"
    }
}

pub(crate) fn submission_status_key(submission: &SubmissionSummary) -> String {
    submission.status.trim().to_lowercase()
}

pub(crate) fn submission_status_label(submission: &SubmissionSummary) -> String {
    metadata_label(&submission.status)
}

pub(crate) fn submission_workflow_label(submission: &SubmissionSummary) -> String {
    nonempty_text(submission.workflow_name.as_deref(), "Standalone Response")
}

pub(crate) fn submission_assignee_label(submission: &SubmissionSummary) -> String {
    nonempty_text(submission.assigned_to_display_name.as_deref(), "Unassigned")
}

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

pub(crate) fn submission_progress_label(submission: &SubmissionSummary) -> String {
    match (
        submission.workflow_steps_completed,
        submission.workflow_step_count,
    ) {
        (Some(completed), Some(count)) if count > 0 => format!("{completed} of {count} completed"),
        _ => format!("{} saved values", submission.value_count),
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn response_input_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        Some(Value::Bool(value)) => value.to_string(),
        Some(value) if !value.is_null() => value.to_string(),
        _ => String::new(),
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn submission_value_maps(
    detail: &SubmissionDetail,
) -> (HashMap<String, String>, HashMap<String, bool>) {
    let mut text_values = HashMap::new();
    let mut boolean_values = HashMap::new();

    for value in &detail.values {
        if value.field_type == "boolean" {
            boolean_values.insert(
                value.key.clone(),
                value
                    .value
                    .as_ref()
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            );
        } else {
            text_values.insert(
                value.key.clone(),
                response_input_value(value.value.as_ref()),
            );
        }
    }

    (text_values, boolean_values)
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn collect_response_values(
    rendered_form: &RenderedForm,
    text_values: &HashMap<String, String>,
    boolean_values: &HashMap<String, bool>,
) -> Result<HashMap<String, Value>, String> {
    let mut values = HashMap::new();

    for section in &rendered_form.sections {
        for field in &section.fields {
            if field.field_type == "boolean" {
                values.insert(
                    field.key.clone(),
                    Value::Bool(*boolean_values.get(&field.key).unwrap_or(&false)),
                );
                continue;
            }

            let raw = text_values
                .get(&field.key)
                .map(String::as_str)
                .unwrap_or_default()
                .trim();
            if raw.is_empty() {
                if field.required {
                    return Err(format!("Required fields missing: {}", field.label));
                }
                continue;
            }

            let value = match field.field_type.as_str() {
                "number" => {
                    let parsed = raw
                        .parse::<f64>()
                        .map_err(|_| format!("{} must be a number.", field.label))?;
                    Value::Number(
                        serde_json::Number::from_f64(parsed)
                            .ok_or_else(|| format!("{} must be a finite number.", field.label))?,
                    )
                }
                "multi_choice" => Value::Array(
                    raw.split(',')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(|value| Value::String(value.to_string()))
                        .collect(),
                ),
                _ => Value::String(raw.to_string()),
            };

            values.insert(field.key.clone(), value);
        }
    }

    Ok(values)
}

pub(crate) fn response_selected_assignment(
    options: RwSignal<Option<AssignmentResponseStartOptions>>,
    selected_assignment_index: RwSignal<String>,
) -> Option<AssignmentResponseStartOption> {
    let index = selected_assignment_index.get().parse::<usize>().ok()?;
    options
        .get()
        .and_then(|options| options.assignments.get(index).cloned())
}

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

pub(crate) fn active_workflow_definition_version(
    workflow: &WorkflowDefinition,
) -> Option<&WorkflowVersionSummary> {
    workflow
        .versions
        .iter()
        .find(|version| version.status.eq_ignore_ascii_case("published"))
        .or_else(|| workflow.versions.first())
}

pub(crate) fn workflow_definition_version_label(
    version: Option<&WorkflowVersionSummary>,
) -> String {
    version
        .and_then(|version| version.workflow_revision_label.as_deref())
        .map(workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn workflow_definition_status_label(version: Option<&WorkflowVersionSummary>) -> String {
    version
        .map(|version| sentence_label(&version.status))
        .unwrap_or_else(|| "No revisions".to_string())
}

pub(crate) fn workflow_assignment_revision_label(label: Option<&str>) -> String {
    label
        .map(workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn workflow_assignment_candidate_key(candidate: &WorkflowAssignmentCandidate) -> String {
    format!("{}|{}", candidate.workflow_version_id, candidate.node_id)
}

pub(crate) fn workflow_assignee_label(assignee: &WorkflowAssigneeOption) -> String {
    if assignee.display_name.trim().is_empty() {
        assignee.email.clone()
    } else {
        format!("{} ({})", assignee.display_name, assignee.email)
    }
}

pub(crate) fn workflow_assignment_assignee_label(assignment: &WorkflowAssignmentSummary) -> String {
    if assignment.account_display_name.trim().is_empty() {
        assignment.account_email.clone()
    } else {
        format!(
            "{} ({})",
            assignment.account_display_name, assignment.account_email
        )
    }
}

pub(crate) fn form_attached_nodes(version: Option<&FormVersionSummary>) -> Vec<FormAttachmentLink> {
    version
        .map(|version| {
            version
                .assignment_nodes
                .iter()
                .filter(|node| !node.node_name.trim().is_empty())
                .map(|node| FormAttachmentLink {
                    href: format!("/organization/{}", node.node_id),
                    label: node.node_name.clone(),
                    title: if node.node_path.trim().is_empty() {
                        node.node_name.clone()
                    } else {
                        node.node_path.replace(" / ", " > ")
                    },
                })
                .collect::<Vec<_>>()
        })
        .filter(|nodes| !nodes.is_empty())
        .unwrap_or_default()
}

pub(crate) fn rendered_field_type_label(field_type: &str) -> String {
    match field_type {
        "static_text" => "Static text".to_string(),
        "single_choice" => "Single choice".to_string(),
        "multi_choice" => "Multi choice".to_string(),
        "boolean" => "Checkbox".to_string(),
        _ => sentence_label(field_type),
    }
}

pub(crate) fn rendered_field_layout_label(field: &RenderedField) -> String {
    format!(
        "Row {}, Column {} · {} wide × {} tall",
        field.grid_row, field.grid_column, field.grid_width, field.grid_height
    )
}

fn slug_from_label(label: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for character in label
        .trim()
        .chars()
        .flat_map(|character| character.to_lowercase())
    {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    slug
}

pub(crate) fn blank_form_builder_field_at(
    id: usize,
    section_id: usize,
    grid_row: i32,
    grid_column: i32,
    grid_width: i32,
) -> FormBuilderFieldDraft {
    FormBuilderFieldDraft {
        id,
        remote_id: None,
        section_id,
        label: String::new(),
        key: String::new(),
        field_type: "text".into(),
        required: false,
        grid_row,
        grid_column,
        grid_width: grid_width.clamp(1, FORM_BUILDER_COLUMN_COUNT),
        grid_height: 1,
        key_was_edited: false,
    }
}

pub(crate) fn form_builder_field_default_label(field_type: &str, id: usize) -> String {
    if field_type == "static_text" {
        "Static text".into()
    } else {
        format!("Field {id}")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormBuilderGridCell {
    pub(crate) row: i32,
    pub(crate) column: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormBuilderSectionLayout {
    pub(crate) fields: Vec<FormBuilderFieldDraft>,
    pub(crate) occupied_cells: HashSet<(i32, i32)>,
    pub(crate) column_count: i32,
    pub(crate) row_count: i32,
}

pub(crate) fn form_builder_section_fields(
    section_id: usize,
    fields: &[FormBuilderFieldDraft],
) -> Vec<FormBuilderFieldDraft> {
    fields
        .iter()
        .filter(|field| field.section_id == section_id)
        .cloned()
        .collect()
}

pub(crate) fn form_builder_occupancy_map(fields: &[FormBuilderFieldDraft]) -> HashSet<(i32, i32)> {
    let mut occupied = HashSet::new();

    for field in fields {
        let row_start = field.grid_row.max(1);
        let row_end = row_start + field.grid_height.max(1) - 1;
        let column_start = field.grid_column.max(1);
        let column_end = column_start + field.grid_width.max(1) - 1;

        for row in row_start..=row_end {
            for column in column_start..=column_end {
                occupied.insert((row, column));
            }
        }
    }

    occupied
}

pub(crate) fn form_builder_section_layout(
    section: &FormBuilderSectionDraft,
    fields: &[FormBuilderFieldDraft],
) -> FormBuilderSectionLayout {
    let section_fields = form_builder_section_fields(section.id, fields);
    let occupied_cells = form_builder_occupancy_map(&section_fields);
    let column_count = FORM_BUILDER_COLUMN_COUNT;
    let bottom_occupied_row = section_fields
        .iter()
        .map(|field| field.grid_row.max(1) + field.grid_height.max(1) - 1)
        .max()
        .unwrap_or(0);
    let row_count = (bottom_occupied_row + 1).max(2);

    FormBuilderSectionLayout {
        fields: section_fields,
        occupied_cells,
        column_count,
        row_count,
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn prepared_form_builder_sections(
    sections: &[FormBuilderSectionDraft],
) -> Result<Vec<FormBuilderSectionDraft>, String> {
    let mut prepared = Vec::new();

    for (index, section) in sections.iter().enumerate() {
        let title = section.title.trim();
        if title.is_empty() {
            return Err("Every section needs a title.".into());
        }
        let mut section = section.clone();
        section.title = title.to_string();
        section.description = section.description.trim().to_string();
        section.position = (index + 1) as i32;
        prepared.push(section);
    }

    Ok(prepared)
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn prepared_form_builder_fields(
    fields: &[FormBuilderFieldDraft],
) -> Result<Vec<FormBuilderFieldDraft>, String> {
    let mut prepared = Vec::new();
    let mut keys = HashSet::new();

    for field in fields {
        let label = field.label.trim();
        let key = field.key.trim();
        if label.is_empty() && key.is_empty() {
            continue;
        }
        if label.is_empty() {
            return Err("Every builder field needs a label.".into());
        }
        if key.is_empty() {
            return Err(format!("{label} needs a field key."));
        }

        let normalized_key = slug_from_label(key);
        if normalized_key.is_empty() {
            return Err(format!("{label} needs a valid field key."));
        }
        if !keys.insert(normalized_key.clone()) {
            return Err(format!("Field key {normalized_key} is already used."));
        }
        if field.grid_row < 1 {
            return Err(format!("{label} must start on row 1 or later."));
        }
        if field.grid_column < 1 {
            return Err(format!("{label} must start on column 1 or later."));
        }
        if field.grid_width < 1 {
            return Err(format!("{label} must span at least 1 column."));
        }
        if field.grid_height < 1 {
            return Err(format!("{label} must span at least 1 row."));
        }

        let mut field = field.clone();
        field.label = label.to_string();
        field.key = normalized_key;
        prepared.push(field);
    }

    Ok(prepared)
}

pub(crate) fn form_builder_field_type_icon(field_type: &str) -> AnyView {
    match field_type {
        "static_text" => view! { <TextQuote /> }.into_any(),
        "number" => view! { <Hash /> }.into_any(),
        "date" => view! { <CalendarDays /> }.into_any(),
        "boolean" => view! { <SquareCheckBig /> }.into_any(),
        "single_choice" => view! { <CircleDot /> }.into_any(),
        "multi_choice" => view! { <ListChecks /> }.into_any(),
        _ => view! { <TextCursorInput /> }.into_any(),
    }
}
