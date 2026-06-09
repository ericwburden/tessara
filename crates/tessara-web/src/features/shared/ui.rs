use crate::features::form_builder::FORM_BUILDER_COLUMN_COUNT;
use crate::features::organization::{
    NodeTypeCatalogEntry, OrganizationNode, form_version_label, form_version_sort_label,
};
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
use std::collections::{BTreeMap, HashMap, HashSet};

type FormSummary = shared::FormSummary;
type FormVersionSummary = shared::FormVersionSummary;
type FormVersionAssignmentNodeSummary = shared::FormVersionAssignmentNodeSummary;
type WorkflowSummary = shared::WorkflowSummary;
type WorkflowAvailableNodeSummary = shared::WorkflowAvailableNodeSummary;
type WorkflowAssignedUserSummary = shared::WorkflowAssignedUserSummary;
type WorkflowDefinition = shared::WorkflowDefinition;
type WorkflowVersionSummary = shared::WorkflowVersionSummary;
type WorkflowStepSummary = shared::WorkflowStepSummary;
type FormDefinition = shared::FormDefinition;
type FormWorkflowLink = shared::FormWorkflowLink;
type FormDatasetSourceLink = shared::FormDatasetSourceLink;
type RenderedForm = shared::RenderedForm;
type RenderedSection = shared::RenderedSection;
type RenderedField = shared::RenderedField;
type FormAttachmentLink = shared::FormAttachmentLink;
type FormsAttachedNodesSheetData = shared::FormsAttachedNodesSheetData;
type WorkflowAssignedUsersSheetData = shared::WorkflowAssignedUsersSheetData;
type WorkflowAvailableNodesSheetData = shared::WorkflowAvailableNodesSheetData;

#[derive(Clone, Debug, PartialEq)]
pub struct FormNodeFilterOption {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) path: String,
    pub(crate) depth: usize,
}
pub(crate) fn form_version_desc_sort_key(version: &FormVersionSummary) -> (i32, i32, i32, String) {
    (
        version.version_major.unwrap_or(-1),
        version.version_minor.unwrap_or(-1),
        version.version_patch.unwrap_or(-1),
        version.published_at.clone().unwrap_or_default(),
    )
}

fn form_status_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| sentence_label(&version.status))
        .unwrap_or_else(|| "No versions".to_string())
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

fn workflow_revision_label_from_option(label: Option<String>) -> String {
    label
        .as_deref()
        .map(workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
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

fn response_value_label(value: Option<&Value>) -> String {
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

fn form_attached_to_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| {
            version
                .assignment_nodes
                .iter()
                .map(|node| node.node_name.as_str())
                .filter(|name| !name.trim().is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Not attached".to_string())
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

fn form_node_filter_options(forms: &[FormSummary]) -> Vec<FormNodeFilterOption> {
    let mut options_by_id = BTreeMap::<String, FormNodeFilterOption>::new();

    for form in forms {
        for version in &form.versions {
            for node in &version.assignment_nodes {
                if node.node_id.trim().is_empty() || node.node_name.trim().is_empty() {
                    continue;
                }

                let path = if node.node_path.trim().is_empty() {
                    node.node_name.clone()
                } else {
                    node.node_path.clone()
                };

                options_by_id
                    .entry(node.node_id.clone())
                    .or_insert_with(|| FormNodeFilterOption {
                        id: node.node_id.clone(),
                        name: node.node_name.clone(),
                        parent_node_id: node.parent_node_id.clone(),
                        path,
                        depth: 0,
                    });
            }
        }
    }

    let options_map = options_by_id.clone();
    let mut options = options_by_id
        .into_values()
        .map(|mut option| {
            option.depth = form_node_filter_depth(&option.id, &options_map, &mut HashSet::new());
            option.path = form_node_filter_path(&option.id, &options_map, &mut HashSet::new());
            option
        })
        .collect::<Vec<_>>();
    options.sort_by(|left, right| left.path.cmp(&right.path).then(left.name.cmp(&right.name)));
    options
}

fn form_node_filter_depth(
    node_id: &str,
    options_by_id: &BTreeMap<String, FormNodeFilterOption>,
    visited: &mut HashSet<String>,
) -> usize {
    if !visited.insert(node_id.to_string()) {
        return 0;
    }

    options_by_id
        .get(node_id)
        .and_then(|option| option.parent_node_id.as_deref())
        .and_then(|parent_id| {
            options_by_id
                .contains_key(parent_id)
                .then(|| 1 + form_node_filter_depth(parent_id, options_by_id, visited))
        })
        .unwrap_or(0)
}

fn form_node_filter_path(
    node_id: &str,
    options_by_id: &BTreeMap<String, FormNodeFilterOption>,
    visited: &mut HashSet<String>,
) -> String {
    if !visited.insert(node_id.to_string()) {
        return options_by_id
            .get(node_id)
            .map(|option| option.name.clone())
            .unwrap_or_else(|| node_id.to_string());
    }

    let Some(option) = options_by_id.get(node_id) else {
        return node_id.to_string();
    };

    option
        .parent_node_id
        .as_deref()
        .filter(|parent_id| options_by_id.contains_key(*parent_id))
        .map(|parent_id| {
            format!(
                "{} / {}",
                form_node_filter_path(parent_id, options_by_id, visited),
                option.name
            )
        })
        .unwrap_or_else(|| option.name.clone())
}

fn form_matches_node_filter(
    form: &FormSummary,
    selected_node_id: Option<&str>,
    options: &[FormNodeFilterOption],
) -> bool {
    let Some(selected_node_id) = selected_node_id else {
        return true;
    };

    form.versions.iter().any(|version| {
        version.assignment_nodes.iter().any(|node| {
            node.node_id == selected_node_id
                || form_node_is_descendant_of_selected(&node.node_id, selected_node_id, options)
        })
    })
}

fn form_node_is_descendant_of_selected(
    node_id: &str,
    selected_node_id: &str,
    options: &[FormNodeFilterOption],
) -> bool {
    let by_id = options
        .iter()
        .map(|option| (option.id.as_str(), option))
        .collect::<HashMap<_, _>>();
    let mut current_parent = by_id
        .get(node_id)
        .and_then(|option| option.parent_node_id.as_deref());
    let mut visited = HashSet::<String>::new();

    while let Some(parent_id) = current_parent {
        if parent_id == selected_node_id {
            return true;
        }
        if !visited.insert(parent_id.to_string()) {
            return false;
        }
        current_parent = by_id
            .get(parent_id)
            .and_then(|option| option.parent_node_id.as_deref());
    }

    false
}

fn visible_form_node_filter_options(
    options: &[FormNodeFilterOption],
    selected_node_id: Option<&str>,
    query: &str,
) -> Vec<FormNodeFilterOption> {
    let query = query.trim().to_lowercase();

    options
        .iter()
        .filter(|option| {
            if selected_node_id == Some(option.id.as_str()) {
                return false;
            }

            let Some(selected_node_id) = selected_node_id else {
                return true;
            };

            form_node_is_descendant_of_selected(&option.id, selected_node_id, options)
        })
        .filter(|option| {
            query.is_empty()
                || option.name.to_lowercase().contains(&query)
                || option.path.to_lowercase().contains(&query)
        })
        .cloned()
        .collect()
}

fn indented_node_label(option: &FormNodeFilterOption) -> String {
    format!("{}{}", " ".repeat(option.depth), option.name)
}

fn unique_filter_options(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut options = values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
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

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn unique_slug_from_label(label: &str, existing_slugs: &[String]) -> String {
    let base = slug_from_label(label);
    if base.is_empty() {
        return String::new();
    }

    let existing = existing_slugs.iter().cloned().collect::<HashSet<_>>();
    if !existing.contains(&base) {
        return base;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !existing.contains(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn existing_form_slugs(forms: &[FormSummary]) -> Vec<String> {
    forms.iter().map(|form| form.slug.clone()).collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn existing_form_slugs_for_update(forms: &[FormSummary], current_form_id: &str) -> Vec<String> {
    forms
        .iter()
        .filter(|form| form.id != current_form_id)
        .map(|form| form.slug.clone())
        .collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn existing_workflow_slugs(workflows: &[WorkflowSummary]) -> Vec<String> {
    workflows
        .iter()
        .map(|workflow| workflow.slug.clone())
        .collect()
}

fn workflow_form_is_in_scope(
    form: &FormSummary,
    node_types: &[NodeTypeCatalogEntry],
    workflow_node_type_id: &str,
) -> bool {
    let _ = (form, node_types, workflow_node_type_id);
    true
}

fn workflow_form_version_options(
    forms: &[FormSummary],
    node_types: &[NodeTypeCatalogEntry],
    workflow_node_type_id: &str,
) -> Vec<(String, String, String)> {
    let mut options = Vec::new();

    for form in forms {
        if !workflow_form_is_in_scope(form, node_types, workflow_node_type_id) {
            continue;
        }
        let mut versions = form
            .versions
            .iter()
            .filter(|version| version.status == "published")
            .collect::<Vec<_>>();
        versions.sort_by(|left, right| {
            form_version_sort_label(*left).cmp(&form_version_sort_label(*right))
        });

        for version in versions {
            let version_label = form_version_label(Some(version));
            options.push((
                version.id.clone(),
                format!("{} ({version_label})", form.name),
                form.name.clone(),
            ));
        }
    }

    options.sort_by(|left, right| left.1.cmp(&right.1));
    options
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn workflow_step_form_label(forms: &[FormSummary], form_version_id: &str) -> String {
    forms
        .iter()
        .flat_map(|form| {
            form.versions.iter().map(move |version| {
                (
                    version.id.as_str(),
                    format!("{} ({})", form.name, form_version_label(Some(version))),
                )
            })
        })
        .find(|(id, _)| *id == form_version_id)
        .map(|(_, label)| label)
        .unwrap_or_else(|| "Select form version".to_string())
}

fn blank_form_builder_section(id: usize) -> FormBuilderSectionDraft {
    FormBuilderSectionDraft {
        id,
        remote_id: None,
        title: if id == 1 {
            "Main".into()
        } else {
            format!("Section {id}")
        },
        description: String::new(),
        default_column_width: 6,
        position: id as i32,
    }
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

pub(crate) fn form_builder_fields_overlap(
    left: &FormBuilderFieldDraft,
    right: &FormBuilderFieldDraft,
) -> bool {
    if left.section_id != right.section_id || left.id == right.id {
        return false;
    }

    let left_row_start = left.grid_row.max(1);
    let left_row_end = left_row_start + left.grid_height.max(1) - 1;
    let left_column_start = left.grid_column.max(1);
    let left_column_end = left_column_start + left.grid_width.max(1) - 1;

    let right_row_start = right.grid_row.max(1);
    let right_row_end = right_row_start + right.grid_height.max(1) - 1;
    let right_column_start = right.grid_column.max(1);
    let right_column_end = right_column_start + right.grid_width.max(1) - 1;

    left_row_start <= right_row_end
        && left_row_end >= right_row_start
        && left_column_start <= right_column_end
        && left_column_end >= right_column_start
}

pub(crate) fn form_builder_field_has_collision(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
) -> bool {
    fields
        .iter()
        .any(|candidate| candidate.id != field.id && form_builder_fields_overlap(field, candidate))
}

pub(crate) fn form_builder_linear_grid_index(
    field: &FormBuilderFieldDraft,
    column_count: i32,
) -> i32 {
    let column_count = column_count.max(1);
    (field.grid_row.max(1) - 1) * column_count + field.grid_column.max(1) - 1
}

fn rendered_form_field_layout_style(field: &RenderedField) -> String {
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

fn response_field_class(field_type: &str) -> String {
    format!(
        "form-field response-form-field response-form-field--{}",
        field_type.replace('_', "-")
    )
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

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "published" | "done" | "active" | "submitted" => "status-badge is-success",
        "draft" | "in_progress" => "status-badge is-warning",
        "error" | "archived" => "status-badge is-danger",
        _ => "status-badge is-info",
    }
}
