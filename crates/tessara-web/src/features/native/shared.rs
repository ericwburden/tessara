use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct FormSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<String>,
    pub(crate) scope_node_type_name: Option<String>,
    #[serde(default)]
    pub(crate) versions: Vec<FormVersionSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct FormVersionSummary {
    pub(crate) id: String,
    pub(crate) version_label: Option<String>,
    pub(crate) status: String,
    pub(crate) version_major: Option<i32>,
    pub(crate) version_minor: Option<i32>,
    pub(crate) version_patch: Option<i32>,
    pub(crate) compatibility_group_name: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) field_count: i64,
    pub(crate) semantic_bump: Option<String>,
    pub(crate) started_new_major_line: Option<bool>,
    #[serde(default)]
    pub(crate) assignment_nodes: Vec<FormVersionAssignmentNodeSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct FormVersionAssignmentNodeSummary {
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) node_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowSummary {
    pub(crate) id: String,
    pub(crate) workflow_node_type_id: String,
    pub(crate) workflow_node_type_name: String,
    #[serde(default)]
    pub(crate) available_nodes: Vec<WorkflowAvailableNodeSummary>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) source: String,
    pub(crate) source_form_id: Option<String>,
    pub(crate) current_version_id: Option<String>,
    pub(crate) current_version_label: Option<String>,
    pub(crate) current_status: Option<String>,
    #[serde(default)]
    pub(crate) assigned_users: Vec<WorkflowAssignedUserSummary>,
    pub(crate) version_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAvailableNodeSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) node_type_name: String,
    pub(crate) path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAssignedUserSummary {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) email: String,
    pub(crate) assignment_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowDefinition {
    pub(crate) id: String,
    pub(crate) workflow_node_type_id: String,
    pub(crate) workflow_node_type_name: String,
    #[serde(default)]
    pub(crate) available_nodes: Vec<WorkflowAvailableNodeSummary>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) source: String,
    pub(crate) source_form_id: Option<String>,
    #[serde(default)]
    pub(crate) versions: Vec<WorkflowVersionSummary>,
    #[serde(default)]
    pub(crate) assignments: Vec<WorkflowAssignmentSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowVersionSummary {
    pub(crate) id: String,
    pub(crate) workflow_revision_label: Option<String>,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) published_at: Option<String>,
    pub(crate) created_at: String,
    pub(crate) step_count: i64,
    #[serde(default)]
    pub(crate) steps: Vec<WorkflowStepSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowStepSummary {
    pub(crate) id: String,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_version_id: String,
    pub(crate) form_version_label: Option<String>,
    pub(crate) title: String,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct FormDefinition {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<String>,
    pub(crate) scope_node_type_name: Option<String>,
    #[serde(default)]
    pub(crate) versions: Vec<FormVersionSummary>,
    #[serde(default)]
    pub(crate) workflows: Vec<FormWorkflowLink>,
    #[serde(default)]
    pub(crate) dataset_sources: Vec<FormDatasetSourceLink>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct FormWorkflowLink {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    #[serde(default)]
    pub(crate) source: String,
    pub(crate) current_version_label: Option<String>,
    pub(crate) current_status: Option<String>,
    pub(crate) assignment_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct FormDatasetSourceLink {
    pub(crate) dataset_id: String,
    pub(crate) dataset_name: String,
    pub(crate) source_alias: String,
    pub(crate) selection_rule: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct RenderedForm {
    pub(crate) form_version_id: String,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) version_label: Option<String>,
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) sections: Vec<RenderedSection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct RenderedSection {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) position: i32,
    #[serde(default)]
    pub(crate) fields: Vec<RenderedField>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct RenderedField {
    pub(crate) id: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
    pub(crate) position: i32,
    pub(crate) grid_row: i32,
    pub(crate) grid_column: i32,
    pub(crate) grid_width: i32,
    pub(crate) grid_height: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormAttachmentLink {
    pub(crate) href: String,
    pub(crate) label: String,
    pub(crate) title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormsAttachedNodesSheetData {
    pub(crate) form_name: String,
    pub(crate) form_href: String,
    pub(crate) nodes: Vec<FormAttachmentLink>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkflowAssignedUsersSheetData {
    pub(crate) workflow_name: String,
    pub(crate) workflow_href: String,
    pub(crate) users: Vec<FormAttachmentLink>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkflowAvailableNodesSheetData {
    pub(crate) workflow_name: String,
    pub(crate) workflow_href: String,
    pub(crate) nodes: Vec<FormAttachmentLink>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FormNodeFilterOption {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) path: String,
    pub(crate) depth: usize,
}
pub(crate) fn text_matches(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    values
        .iter()
        .any(|value| value.to_lowercase().contains(&query))
}

pub(crate) fn nonempty_text(value: Option<&str>, fallback: &'static str) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

pub(crate) fn sentence_label(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub(crate) fn metadata_rows(metadata: &Value) -> Vec<(String, String)> {
    match metadata {
        Value::Object(values) => values
            .iter()
            .map(|(key, value)| (metadata_label(key), metadata_value(value)))
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn metadata_label(key: &str) -> String {
    key.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn metadata_value(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::Bool(value) => {
            if *value {
                "Yes".to_string()
            } else {
                "No".to_string()
            }
        }
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .map(metadata_value)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(_) => value.to_string(),
    }
}

pub(crate) fn form_version_desc_sort_key(version: &FormVersionSummary) -> (i32, i32, i32, String) {
    (
        version.version_major.unwrap_or(-1),
        version.version_minor.unwrap_or(-1),
        version.version_patch.unwrap_or(-1),
        version.published_at.clone().unwrap_or_default(),
    )
}

pub(crate) fn form_status_label(version: Option<&FormVersionSummary>) -> String {
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

pub(crate) fn workflow_revision_label_from_option(label: Option<String>) -> String {
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

pub(crate) fn workflow_assignment_state_label(assignment: &WorkflowAssignmentSummary) -> &'static str {
    match workflow_assignment_state(assignment) {
        "submitted" => "Submitted",
        "draft" => "Draft Exists",
        _ => "Pending",
    }
}

pub(crate) fn workflow_assignment_status_key(assignment: &WorkflowAssignmentSummary) -> &'static str {
    if assignment.is_active {
        "active"
    } else {
        "inactive"
    }
}

pub(crate) fn workflow_assignment_status_label(assignment: &WorkflowAssignmentSummary) -> &'static str {
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

pub(crate) fn workflow_definition_version_label(version: Option<&WorkflowVersionSummary>) -> String {
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

pub(crate) fn form_attached_to_label(version: Option<&FormVersionSummary>) -> String {
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

pub(crate) fn form_node_filter_options(forms: &[FormSummary]) -> Vec<FormNodeFilterOption> {
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

pub(crate) fn form_node_filter_depth(
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

pub(crate) fn form_node_filter_path(
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

pub(crate) fn form_matches_node_filter(
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

pub(crate) fn form_node_is_descendant_of_selected(
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

pub(crate) fn visible_form_node_filter_options(
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

pub(crate) fn indented_node_label(option: &FormNodeFilterOption) -> String {
    format!("{}{}", " ".repeat(option.depth), option.name)
}

pub(crate) fn unique_filter_options(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut options = values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

pub(crate) fn slug_from_label(label: &str) -> String {
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
pub(crate) fn unique_slug_from_label(label: &str, existing_slugs: &[String]) -> String {
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
pub(crate) fn existing_form_slugs(forms: &[FormSummary]) -> Vec<String> {
    forms.iter().map(|form| form.slug.clone()).collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn existing_form_slugs_for_update(forms: &[FormSummary], current_form_id: &str) -> Vec<String> {
    forms
        .iter()
        .filter(|form| form.id != current_form_id)
        .map(|form| form.slug.clone())
        .collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn existing_workflow_slugs(workflows: &[WorkflowSummary]) -> Vec<String> {
    workflows
        .iter()
        .map(|workflow| workflow.slug.clone())
        .collect()
}

pub(crate) fn workflow_form_is_in_scope(
    form: &FormSummary,
    node_types: &[NodeTypeCatalogEntry],
    workflow_node_type_id: &str,
) -> bool {
    let _ = (form, node_types, workflow_node_type_id);
    true
}

pub(crate) fn workflow_form_version_options(
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
            form_version_sort_label(left).cmp(&form_version_sort_label(right))
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
pub(crate) fn workflow_step_form_label(forms: &[FormSummary], form_version_id: &str) -> String {
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

pub(crate) fn blank_form_builder_section(id: usize) -> FormBuilderSectionDraft {
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

pub(crate) fn form_builder_linear_grid_index(field: &FormBuilderFieldDraft, column_count: i32) -> i32 {
    let column_count = column_count.max(1);
    (field.grid_row.max(1) - 1) * column_count + field.grid_column.max(1) - 1
}

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

pub(crate) fn response_field_class(field_type: &str) -> String {
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

pub(crate) fn status_badge_class(status: &str) -> &'static str {
    match status {
        "published" | "done" | "active" | "submitted" => "status-badge is-success",
        "draft" | "in_progress" => "status-badge is-warning",
        "error" | "archived" => "status-badge is-danger",
        _ => "status-badge is-info",
    }
}

pub(crate) fn load_forms(
    forms: RwSignal<Vec<FormSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/forms").send().await {
                Ok(response) if response.status() == 401 => {
                    forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<Vec<FormSummary>>().await {
                    Ok(loaded_forms) => {
                        forms.set(loaded_forms);
                        is_loading.set(false);
                    }
                    Err(error) => {
                        forms.set(Vec::new());
                        load_error.set(Some(format!("Unable to parse forms: {error}")));
                        is_loading.set(false);
                    }
                },
                Ok(response) => {
                    forms.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load forms. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    forms.set(Vec::new());
                    load_error.set(Some(format!("Unable to load forms: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (forms, is_loading, load_error);
    }
}

pub(crate) fn load_workflow_assignments(
    assignments: RwSignal<Vec<WorkflowAssignmentSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/workflow-assignments").send().await {
                Ok(response) if response.status() == 401 => {
                    assignments.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<WorkflowAssignmentSummary>>().await {
                        Ok(loaded_assignments) => {
                            assignments.set(loaded_assignments);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            assignments.set(Vec::new());
                            load_error.set(Some(format!(
                                "Unable to parse workflow assignments: {error}"
                            )));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    assignments.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load workflow assignments. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    assignments.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load workflow assignments: {error}"
                    )));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (assignments, is_loading, load_error);
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn load_pending_work(
    pending_work: RwSignal<Vec<PendingWorkflowWork>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/workflow-assignments/pending")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    pending_work.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<PendingWorkflowWork>>().await {
                        Ok(loaded_work) => {
                            pending_work.set(loaded_work);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            pending_work.set(Vec::new());
                            load_error.set(Some(format!("Unable to parse assigned work: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    pending_work.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load assigned work. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    pending_work.set(Vec::new());
                    load_error.set(Some(format!("Unable to load assigned work: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (pending_work, is_loading, load_error);
    }
}

pub(crate) fn load_submissions(
    submissions: RwSignal<Vec<SubmissionSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/submissions")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    submissions.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<SubmissionSummary>>().await {
                        Ok(loaded_submissions) => {
                            submissions.set(loaded_submissions);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            submissions.set(Vec::new());
                            load_error.set(Some(format!("Unable to parse responses: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    submissions.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load responses. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    submissions.set(Vec::new());
                    load_error.set(Some(format!("Unable to load responses: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (submissions, is_loading, load_error);
    }
}

pub(crate) fn load_admin_users(
    users: RwSignal<Vec<AdminUserSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/admin/users")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    users.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<AdminUserSummary>>().await {
                        Ok(loaded_users) => {
                            users.set(loaded_users);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            users.set(Vec::new());
                            load_error.set(Some(format!("Unable to parse users: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    users.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load users. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    users.set(Vec::new());
                    load_error.set(Some(format!("Unable to load users: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (users, is_loading, load_error);
    }
}

pub(crate) fn load_admin_user_access(
    account_id: String,
    detail: RwSignal<Option<AdminUserAccessDetail>>,
    selected_scope_node_ids: RwSignal<Vec<String>>,
    selected_delegate_account_ids: RwSignal<Vec<String>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get(&format!("/api/admin/users/{account_id}/access"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<AdminUserAccessDetail>().await {
                        Ok(loaded_detail) => {
                            selected_scope_node_ids.set(
                                loaded_detail
                                    .scope_nodes
                                    .iter()
                                    .map(|node| node.node_id.clone())
                                    .collect(),
                            );
                            selected_delegate_account_ids.set(
                                loaded_detail
                                    .delegations
                                    .iter()
                                    .map(|delegation| delegation.account_id.clone())
                                    .collect(),
                            );
                            detail.set(Some(loaded_detail));
                            is_loading.set(false);
                        }
                        Err(error) => {
                            detail.set(None);
                            load_error
                                .set(Some(format!("Unable to parse user permissions: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load user permissions. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    load_error.set(Some(format!("Unable to load user permissions: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            account_id,
            detail,
            selected_scope_node_ids,
            selected_delegate_account_ids,
            is_loading,
            load_error,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn load_admin_user_edit_context(
    account_id: String,
    detail: RwSignal<Option<AdminUserDetail>>,
    roles: RwSignal<Vec<AdminRoleSummary>>,
    email: RwSignal<String>,
    display_name: RwSignal<String>,
    is_active: RwSignal<bool>,
    selected_role_ids: RwSignal<Vec<String>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            let user_response =
                gloo_net::http::Request::get(&format!("/api/admin/users/{account_id}"))
                    .send()
                    .await;
            let roles_response = gloo_net::http::Request::get("/api/admin/roles")
                .send()
                .await;

            match (user_response, roles_response) {
                (Ok(user_response), _) if user_response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(user_response), Ok(roles_response))
                    if user_response.ok() && roles_response.ok() =>
                {
                    let loaded_user = user_response.json::<AdminUserDetail>().await;
                    let loaded_roles = roles_response.json::<Vec<AdminRoleSummary>>().await;
                    match (loaded_user, loaded_roles) {
                        (Ok(user), Ok(available_roles)) => {
                            email.set(user.email.clone());
                            display_name.set(user.display_name.clone());
                            is_active.set(user.is_active);
                            selected_role_ids
                                .set(user.roles.iter().map(|role| role.id.clone()).collect());
                            detail.set(Some(user));
                            roles.set(available_roles);
                            is_loading.set(false);
                        }
                        (Err(error), _) => {
                            load_error.set(Some(format!("Unable to parse user: {error}")));
                            is_loading.set(false);
                        }
                        (_, Err(error)) => {
                            load_error.set(Some(format!("Unable to parse roles: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                (Ok(user_response), _) if !user_response.ok() => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load user. Server returned {}.",
                        user_response.status()
                    )));
                    is_loading.set(false);
                }
                (_, Ok(roles_response)) if !roles_response.ok() => {
                    roles.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load roles. Server returned {}.",
                        roles_response.status()
                    )));
                    is_loading.set(false);
                }
                (Err(error), _) => {
                    load_error.set(Some(format!("Unable to load user: {error}")));
                    is_loading.set(false);
                }
                (_, Err(error)) => {
                    load_error.set(Some(format!("Unable to load roles: {error}")));
                    is_loading.set(false);
                }
                _ => {
                    load_error.set(Some("Unable to load user edit context.".into()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            account_id,
            detail,
            roles,
            email,
            display_name,
            is_active,
            selected_role_ids,
            is_loading,
            load_error,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn submit_update_admin_user(
    account_id: String,
    email: RwSignal<String>,
    display_name: RwSignal<String>,
    password: RwSignal<String>,
    is_active: RwSignal<bool>,
    selected_role_ids: RwSignal<Vec<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);
            let password_value = password.get().trim().to_string();
            let payload = UpdateAdminUserPayload {
                email: email.get().trim().to_string(),
                display_name: display_name.get().trim().to_string(),
                password: (!password_value.is_empty()).then_some(password_value),
                is_active: is_active.get(),
                role_ids: selected_role_ids.get(),
            };

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("User update could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_id_request(
                gloo_net::http::Request::put(&format!("/api/admin/users/{account_id}")),
                Some(body),
                "Update user",
            )
            .await
            {
                Ok(_) => navigate_to_href(&format!("/administration/users/{account_id}")),
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            account_id,
            email,
            display_name,
            password,
            is_active,
            selected_role_ids,
            is_saving,
            message,
        );
    }
}

pub(crate) fn submit_update_admin_user_access(
    account_id: String,
    selected_scope_node_ids: RwSignal<Vec<String>>,
    selected_delegate_account_ids: RwSignal<Vec<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);
            let payload = UpdateAdminUserAccessPayload {
                scope_node_ids: selected_scope_node_ids.get(),
                delegate_account_ids: selected_delegate_account_ids.get(),
            };

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Permission update could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_id_request(
                gloo_net::http::Request::put(&format!("/api/admin/users/{account_id}/access")),
                Some(body),
                "Update permissions",
            )
            .await
            {
                Ok(_) => navigate_to_href(&format!("/administration/users/{account_id}")),
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            account_id,
            selected_scope_node_ids,
            selected_delegate_account_ids,
            is_saving,
            message,
        );
    }
}

pub(crate) fn load_submission_detail(
    submission_id: String,
    detail: RwSignal<Option<SubmissionDetail>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get(&format!("/api/submissions/{submission_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<SubmissionDetail>().await {
                    Ok(loaded_detail) => {
                        detail.set(Some(loaded_detail));
                        is_loading.set(false);
                    }
                    Err(error) => {
                        detail.set(None);
                        load_error.set(Some(format!("Unable to parse response: {error}")));
                        is_loading.set(false);
                    }
                },
                Ok(response) => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load response. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    load_error.set(Some(format!("Unable to load response: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (submission_id, detail, is_loading, load_error);
    }
}

pub(crate) fn load_submission_edit_context(
    submission_id: String,
    detail: RwSignal<Option<SubmissionDetail>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    text_values: RwSignal<HashMap<String, String>>,
    boolean_values: RwSignal<HashMap<String, bool>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            let loaded_detail =
                match gloo_net::http::Request::get(&format!("/api/submissions/{submission_id}"))
                    .send()
                    .await
                {
                    Ok(response) if response.status() == 401 => {
                        is_loading.set(false);
                        redirect_to_login();
                        return;
                    }
                    Ok(response) if response.ok() => {
                        match response.json::<SubmissionDetail>().await {
                            Ok(detail) => detail,
                            Err(error) => {
                                load_error.set(Some(format!("Unable to parse response: {error}")));
                                is_loading.set(false);
                                return;
                            }
                        }
                    }
                    Ok(response) => {
                        load_error.set(Some(format!(
                            "Unable to load response. Server returned {}.",
                            response.status()
                        )));
                        is_loading.set(false);
                        return;
                    }
                    Err(error) => {
                        load_error.set(Some(format!("Unable to load response: {error}")));
                        is_loading.set(false);
                        return;
                    }
                };

            if loaded_detail.status != "draft" {
                let (loaded_text_values, loaded_boolean_values) =
                    submission_value_maps(&loaded_detail);
                text_values.set(loaded_text_values);
                boolean_values.set(loaded_boolean_values);
                detail.set(Some(loaded_detail));
                rendered_form.set(None);
                is_loading.set(false);
                return;
            }

            let loaded_rendered = match gloo_net::http::Request::get(&format!(
                "/api/form-versions/{}/render",
                loaded_detail.form_version_id
            ))
            .send()
            .await
            {
                Ok(response) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                    return;
                }
                Ok(response) if response.ok() => match response.json::<RenderedForm>().await {
                    Ok(rendered) => rendered,
                    Err(error) => {
                        load_error.set(Some(format!("Unable to parse response form: {error}")));
                        is_loading.set(false);
                        return;
                    }
                },
                Ok(response) => {
                    load_error.set(Some(format!(
                        "Unable to load response form. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                    return;
                }
                Err(error) => {
                    load_error.set(Some(format!("Unable to load response form: {error}")));
                    is_loading.set(false);
                    return;
                }
            };

            let (loaded_text_values, loaded_boolean_values) = submission_value_maps(&loaded_detail);
            text_values.set(loaded_text_values);
            boolean_values.set(loaded_boolean_values);
            detail.set(Some(loaded_detail));
            rendered_form.set(Some(loaded_rendered));
            is_loading.set(false);
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            submission_id,
            detail,
            rendered_form,
            text_values,
            boolean_values,
            is_loading,
            load_error,
        );
    }
}

pub(crate) fn load_response_start_options(
    options: RwSignal<Option<AssignmentResponseStartOptions>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    delegate_account_id: Option<String>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let path = delegate_account_id
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .map(|value| format!("/api/responses/options?delegate_account_id={value}"))
                .unwrap_or_else(|| "/api/responses/options".to_string());

            match gloo_net::http::Request::get(&path).send().await {
                Ok(response) if response.status() == 401 => {
                    options.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<AssignmentResponseStartOptions>().await {
                        Ok(loaded_options) => {
                            options.set(Some(loaded_options));
                            is_loading.set(false);
                        }
                        Err(error) => {
                            options.set(None);
                            message.set(Some(format!(
                                "Unable to parse assigned response start options: {error}"
                            )));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    options.set(None);
                    message.set(Some(format!(
                        "Unable to load assigned response start options. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    options.set(None);
                    message.set(Some(format!(
                        "Unable to load assigned response start options: {error}"
                    )));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (options, is_loading, message, delegate_account_id);
    }
}

pub(crate) fn start_workflow_assignment_response(
    workflow_assignment_id: String,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(Some("Starting assigned response...".into()));

            match send_json_id_request(
                gloo_net::http::Request::post(&format!(
                    "/api/workflow-assignments/{workflow_assignment_id}/start"
                )),
                Some("{}".into()),
                "Start assigned response",
            )
            .await
            {
                Ok(response) => navigate_to_href(&format!("/responses/{}/edit", response.id)),
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (workflow_assignment_id, is_saving, message);
    }
}

pub(crate) fn save_submission_values(
    submission_id: String,
    rendered_form: RenderedForm,
    text_values: HashMap<String, String>,
    boolean_values: HashMap<String, bool>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let values =
                match collect_response_values(&rendered_form, &text_values, &boolean_values) {
                    Ok(values) => values,
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                };

            let body = match serde_json::to_string(&SaveSubmissionValuesPayload { values }) {
                Ok(body) => body,
                Err(error) => {
                    message.set(Some(format!(
                        "Response values could not be prepared: {error}"
                    )));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_id_request(
                gloo_net::http::Request::put(&format!("/api/submissions/{submission_id}/values")),
                Some(body),
                "Save response draft",
            )
            .await
            {
                Ok(_) => {
                    message.set(Some("Draft saved.".into()));
                    is_saving.set(false);
                }
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            submission_id,
            rendered_form,
            text_values,
            boolean_values,
            is_saving,
            message,
        );
    }
}

pub(crate) fn submit_response_values(
    submission_id: String,
    rendered_form: RenderedForm,
    text_values: HashMap<String, String>,
    boolean_values: HashMap<String, bool>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let values =
                match collect_response_values(&rendered_form, &text_values, &boolean_values) {
                    Ok(values) => values,
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                };

            let body = match serde_json::to_string(&SaveSubmissionValuesPayload { values }) {
                Ok(body) => body,
                Err(error) => {
                    message.set(Some(format!(
                        "Response values could not be prepared: {error}"
                    )));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_id_request(
                gloo_net::http::Request::put(&format!("/api/submissions/{submission_id}/values")),
                Some(body),
                "Save response draft",
            )
            .await
            {
                Ok(_) => match send_json_id_request(
                    gloo_net::http::Request::post(&format!(
                        "/api/submissions/{submission_id}/submit"
                    )),
                    Some("{}".into()),
                    "Submit response",
                )
                .await
                {
                    Ok(response) => navigate_to_href(&format!("/responses/{}", response.id)),
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                    }
                },
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            submission_id,
            rendered_form,
            text_values,
            boolean_values,
            is_saving,
            message,
        );
    }
}

pub(crate) fn load_workflow_assignment_candidates(
    candidates: RwSignal<Vec<WorkflowAssignmentCandidate>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/workflow-assignment-candidates")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    candidates.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<WorkflowAssignmentCandidate>>().await {
                        Ok(loaded_candidates) => {
                            candidates.set(loaded_candidates);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            candidates.set(Vec::new());
                            load_error.set(Some(format!(
                                "Unable to parse assignment candidates: {error}"
                            )));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    candidates.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load assignment candidates. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    candidates.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load assignment candidates: {error}"
                    )));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (candidates, is_loading, load_error);
    }
}

pub(crate) fn load_workflow_assignment_assignees(
    workflow_version_id: String,
    node_id: String,
    assignees: RwSignal<Vec<WorkflowAssigneeOption>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if workflow_version_id.trim().is_empty() || node_id.trim().is_empty() {
                assignees.set(Vec::new());
                is_loading.set(false);
                load_error.set(None);
                return;
            }

            is_loading.set(true);
            load_error.set(None);
            let url = format!(
                "/api/workflow-assignment-candidates/assignees?workflow_version_id={workflow_version_id}&node_id={node_id}"
            );

            match gloo_net::http::Request::get(&url).send().await {
                Ok(response) if response.status() == 401 => {
                    assignees.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<WorkflowAssigneeOption>>().await {
                        Ok(loaded_assignees) => {
                            assignees.set(loaded_assignees);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            assignees.set(Vec::new());
                            load_error
                                .set(Some(format!("Unable to parse eligible assignees: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    assignees.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load eligible assignees. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    assignees.set(Vec::new());
                    load_error.set(Some(format!("Unable to load eligible assignees: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            workflow_version_id,
            node_id,
            assignees,
            is_loading,
            load_error,
        );
    }
}

pub(crate) fn load_form_detail(
    form_id: String,
    detail: RwSignal<Option<FormDefinition>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);
            rendered_form.set(None);

            match gloo_net::http::Request::get(&format!("/api/forms/{form_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail.set(None);
                    rendered_form.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<FormDefinition>().await {
                    Ok(form) => {
                        let active_version_id =
                            active_form_definition_version(&form).map(|version| version.id.clone());
                        detail.set(Some(form));
                        if let Some(version_id) = active_version_id {
                            load_rendered_form_version(version_id, rendered_form);
                        }
                        is_loading.set(false);
                    }
                    Err(error) => {
                        detail.set(None);
                        load_error.set(Some(format!("Unable to parse form detail: {error}")));
                        is_loading.set(false);
                    }
                },
                Ok(response) => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load form detail. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    load_error.set(Some(format!("Unable to load form detail: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (form_id, detail, rendered_form, is_loading, load_error);
    }
}

pub(crate) fn load_workflow_detail(
    workflow_id: String,
    detail: RwSignal<Option<WorkflowDefinition>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get(&format!("/api/workflows/{workflow_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<WorkflowDefinition>().await {
                        Ok(workflow) => {
                            detail.set(Some(workflow));
                            is_loading.set(false);
                        }
                        Err(error) => {
                            detail.set(None);
                            load_error
                                .set(Some(format!("Unable to parse workflow detail: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load workflow detail. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    load_error.set(Some(format!("Unable to load workflow detail: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (workflow_id, detail, is_loading, load_error);
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn load_rendered_form_version(
    form_version_id: String,
    rendered_form: RwSignal<Option<RenderedForm>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match gloo_net::http::Request::get(&format!(
                "/api/form-versions/{form_version_id}/render"
            ))
            .send()
            .await
            {
                Ok(response) if response.ok() => {
                    if let Ok(rendered) = response.json::<RenderedForm>().await {
                        rendered_form.set(Some(rendered));
                    }
                }
                _ => rendered_form.set(None),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (form_version_id, rendered_form);
    }
}

#[cfg(feature = "hydrate")]
async fn send_json_id_request(
    builder: gloo_net::http::RequestBuilder,
    body: Option<String>,
    action: &str,
) -> Result<IdResponse, String> {
    send_json_request(builder, body, action).await
}

pub(crate) fn load_form_create_options(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let forms_response = gloo_net::http::Request::get("/api/forms").send().await;

            match (node_types_response, forms_response) {
                (Ok(response), _) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response)) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_types_response), Ok(forms_response))
                    if node_types_response.ok() && forms_response.ok() =>
                {
                    let loaded_node_types = node_types_response
                        .json::<Vec<NodeTypeCatalogEntry>>()
                        .await;
                    let loaded_forms = forms_response.json::<Vec<FormSummary>>().await;

                    match (loaded_node_types, loaded_forms) {
                        (Ok(loaded_node_types), Ok(loaded_forms)) => {
                            node_types.set(loaded_node_types);
                            existing_forms.set(loaded_forms);
                            is_loading.set(false);
                        }
                        _ => {
                            node_types.set(Vec::new());
                            existing_forms.set(Vec::new());
                            message.set(Some("Form options could not be read.".into()));
                            is_loading.set(false);
                        }
                    }
                }
                (Ok(node_types_response), Ok(forms_response)) => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    message.set(Some(format!(
                        "Form options failed with status {} / {}.",
                        node_types_response.status(),
                        forms_response.status()
                    )));
                    is_loading.set(false);
                }
                _ => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    message.set(Some("Could not reach the form option APIs.".into()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (node_types, existing_forms, is_loading, message);
    }
}

pub(crate) fn load_workflow_create_options(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    organization_nodes: RwSignal<Vec<OrganizationNode>>,
    forms: RwSignal<Vec<FormSummary>>,
    workflows: RwSignal<Vec<WorkflowSummary>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let nodes_response = gloo_net::http::Request::get("/api/nodes").send().await;
            let forms_response = gloo_net::http::Request::get("/api/forms").send().await;
            let workflows_response = gloo_net::http::Request::get("/api/workflows").send().await;

            match (
                node_types_response,
                nodes_response,
                forms_response,
                workflows_response,
            ) {
                (Ok(response), _, _, _) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response), _, _) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, Ok(response), _) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, _, Ok(response)) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (
                    Ok(node_types_response),
                    Ok(nodes_response),
                    Ok(forms_response),
                    Ok(workflows_response),
                ) if node_types_response.ok()
                    && nodes_response.ok()
                    && forms_response.ok()
                    && workflows_response.ok() =>
                {
                    let loaded_node_types = node_types_response
                        .json::<Vec<NodeTypeCatalogEntry>>()
                        .await;
                    let loaded_nodes = nodes_response.json::<Vec<OrganizationNode>>().await;
                    let loaded_forms = forms_response.json::<Vec<FormSummary>>().await;
                    let loaded_workflows = workflows_response.json::<Vec<WorkflowSummary>>().await;

                    match (
                        loaded_node_types,
                        loaded_nodes,
                        loaded_forms,
                        loaded_workflows,
                    ) {
                        (
                            Ok(mut loaded_node_types),
                            Ok(mut loaded_nodes),
                            Ok(mut loaded_forms),
                            Ok(mut loaded_workflows),
                        ) => {
                            loaded_node_types.sort_by(|left, right| {
                                left.singular_label
                                    .cmp(&right.singular_label)
                                    .then(left.name.cmp(&right.name))
                            });
                            loaded_forms.sort_by(|left, right| {
                                left.name.cmp(&right.name).then(left.slug.cmp(&right.slug))
                            });
                            loaded_nodes.sort_by(|left, right| {
                                node_display_path(left)
                                    .cmp(&node_display_path(right))
                                    .then(left.name.cmp(&right.name))
                            });
                            loaded_workflows.sort_by(|left, right| {
                                left.name.cmp(&right.name).then(left.slug.cmp(&right.slug))
                            });

                            node_types.set(loaded_node_types);
                            organization_nodes.set(loaded_nodes);
                            forms.set(loaded_forms);
                            workflows.set(loaded_workflows);
                            is_loading.set(false);
                        }
                        _ => {
                            node_types.set(Vec::new());
                            organization_nodes.set(Vec::new());
                            forms.set(Vec::new());
                            workflows.set(Vec::new());
                            message.set(Some("Workflow options could not be read.".into()));
                            is_loading.set(false);
                        }
                    }
                }
                (
                    Ok(node_types_response),
                    Ok(nodes_response),
                    Ok(forms_response),
                    Ok(workflows_response),
                ) => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    message.set(Some(format!(
                        "Workflow options failed with status {} / {} / {} / {}.",
                        node_types_response.status(),
                        nodes_response.status(),
                        forms_response.status(),
                        workflows_response.status()
                    )));
                    is_loading.set(false);
                }
                _ => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    message.set(Some("Could not reach the workflow option APIs.".into()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_types,
            organization_nodes,
            forms,
            workflows,
            is_loading,
            message,
        );
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn hydrate_form_builder_from_rendered(
    rendered_form: Option<&RenderedForm>,
) -> (
    Vec<FormBuilderSectionDraft>,
    Vec<FormBuilderFieldDraft>,
    usize,
    usize,
) {
    let Some(rendered_form) = rendered_form else {
        return (vec![blank_form_builder_section(1)], Vec::new(), 2, 1);
    };

    let mut sections = rendered_form.sections.clone();
    sections.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then(left.title.cmp(&right.title))
    });

    if sections.is_empty() {
        return (vec![blank_form_builder_section(1)], Vec::new(), 2, 1);
    }

    let mut section_id_by_remote = HashMap::new();
    let mut builder_sections = Vec::new();
    let mut builder_fields = Vec::new();
    let mut next_section_id = 1usize;
    let mut next_field_id = 1usize;

    for section in &sections {
        let local_section_id = next_section_id;
        next_section_id += 1;
        section_id_by_remote.insert(section.id.clone(), local_section_id);

        builder_sections.push(FormBuilderSectionDraft {
            id: local_section_id,
            remote_id: Some(section.id.clone()),
            title: nonempty_text(Some(section.title.as_str()), "Main"),
            description: section.description.clone(),
            default_column_width: 6,
            position: section.position,
        });
    }

    for section in &sections {
        let Some(section_id) = section_id_by_remote.get(&section.id).copied() else {
            continue;
        };
        let mut fields = section.fields.clone();
        fields.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then(left.label.cmp(&right.label))
        });

        for field in fields {
            let local_field_id = next_field_id;
            next_field_id += 1;
            builder_fields.push(FormBuilderFieldDraft {
                id: local_field_id,
                remote_id: Some(field.id),
                section_id,
                label: field.label,
                key: field.key,
                field_type: field.field_type,
                required: field.required,
                grid_row: field.grid_row.max(1),
                grid_column: field.grid_column.clamp(1, FORM_BUILDER_COLUMN_COUNT),
                grid_width: field.grid_width.clamp(1, FORM_BUILDER_COLUMN_COUNT),
                grid_height: field.grid_height.clamp(1, 6),
                key_was_edited: true,
            });
        }
    }

    (
        builder_sections,
        builder_fields,
        next_section_id,
        next_field_id,
    )
}

pub(crate) fn load_form_edit_options(
    form_id: String,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    detail: RwSignal<Option<FormDefinition>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    edit_version_id: RwSignal<Option<String>>,
    edit_version_status: RwSignal<Option<String>>,
    name: RwSignal<String>,
    workflow_node_type_id: RwSignal<String>,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_section: RwSignal<String>,
    next_builder_section_id: RwSignal<usize>,
    next_builder_field_id: RwSignal<usize>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);
            detail.set(None);
            rendered_form.set(None);
            edit_version_id.set(None);
            edit_version_status.set(None);

            let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let forms_response = gloo_net::http::Request::get("/api/forms").send().await;
            let detail_response =
                gloo_net::http::Request::get(&format!("/api/admin/forms/{form_id}"))
                    .send()
                    .await;

            match (node_types_response, forms_response, detail_response) {
                (Ok(response), _, _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response), _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, Ok(response)) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_types_response), Ok(forms_response), Ok(detail_response))
                    if node_types_response.ok() && forms_response.ok() && detail_response.ok() =>
                {
                    let loaded_node_types = node_types_response
                        .json::<Vec<NodeTypeCatalogEntry>>()
                        .await;
                    let loaded_forms = forms_response.json::<Vec<FormSummary>>().await;
                    let loaded_detail = detail_response.json::<FormDefinition>().await;

                    match (loaded_node_types, loaded_forms, loaded_detail) {
                        (Ok(loaded_node_types), Ok(loaded_forms), Ok(form)) => {
                            let selected_version = editable_form_definition_version(&form).cloned();
                            let mut loaded_rendered_form = None;

                            if let Some(version) = selected_version.as_ref() {
                                match gloo_net::http::Request::get(&format!(
                                    "/api/form-versions/{}/render",
                                    version.id
                                ))
                                .send()
                                .await
                                {
                                    Ok(response) if response.ok() => {
                                        loaded_rendered_form =
                                            response.json::<RenderedForm>().await.ok();
                                    }
                                    Ok(response) if response.status() == 401 => {
                                        is_loading.set(false);
                                        redirect_to_login();
                                        return;
                                    }
                                    _ => {
                                        loaded_rendered_form = None;
                                    }
                                }
                            }

                            let (sections, fields, next_section, next_field) =
                                hydrate_form_builder_from_rendered(loaded_rendered_form.as_ref());
                            let active_section = sections
                                .first()
                                .map(|section| section.id.to_string())
                                .unwrap_or_else(|| "1".to_string());

                            name.set(form.name.clone());
                            workflow_node_type_id
                                .set(form.scope_node_type_id.clone().unwrap_or_default());
                            edit_version_id
                                .set(selected_version.as_ref().map(|version| version.id.clone()));
                            edit_version_status.set(
                                selected_version
                                    .as_ref()
                                    .map(|version| version.status.clone()),
                            );
                            active_builder_section.set(active_section);
                            next_builder_section_id.set(next_section);
                            next_builder_field_id.set(next_field);
                            builder_sections.set(sections);
                            builder_fields.set(fields);
                            rendered_form.set(loaded_rendered_form);
                            detail.set(Some(form));
                            node_types.set(loaded_node_types);
                            existing_forms.set(loaded_forms);
                            is_loading.set(false);
                        }
                        _ => {
                            is_loading.set(false);
                            message.set(Some("Form edit options could not be read.".into()));
                        }
                    }
                }
                (Ok(node_types_response), Ok(forms_response), Ok(detail_response)) => {
                    is_loading.set(false);
                    message.set(Some(format!(
                        "Form edit options failed with status {} / {} / {}.",
                        node_types_response.status(),
                        forms_response.status(),
                        detail_response.status()
                    )));
                }
                _ => {
                    is_loading.set(false);
                    message.set(Some("Could not reach the form edit APIs.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            form_id,
            node_types,
            existing_forms,
            detail,
            rendered_form,
            edit_version_id,
            edit_version_status,
            name,
            workflow_node_type_id,
            builder_sections,
            builder_fields,
            active_builder_section,
            next_builder_section_id,
            next_builder_field_id,
            is_loading,
            message,
        );
    }
}

