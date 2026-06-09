#[cfg(feature = "hydrate")]
use crate::api::client::{redirect_to_login, send_json_request};
use crate::features::administration::*;
use crate::features::form_builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderDragPreview, FormBuilderEditorState,
    add_form_builder_section_to_editor, new_form_builder_editor_state,
};
use crate::features::forms::*;
use crate::features::organization::*;
use crate::features::shared::{
    FormNodeFilterOption, existing_form_slugs, existing_form_slugs_for_update,
    existing_workflow_slugs, form_matches_node_filter, form_node_filter_depth,
    form_node_filter_options, form_node_filter_path, form_node_is_descendant_of_selected,
    indented_node_label, unique_filter_options, unique_slug_from_label,
    visible_form_node_filter_options, workflow_form_is_in_scope, workflow_form_version_options,
    workflow_step_form_label,
};
use crate::features::shared::{
    WorkflowSourceMarker, active_workflow_definition_version, assignment_count_label,
    blank_form_builder_field_at, collect_response_values, form_attached_nodes,
    form_builder_field_default_label, form_builder_field_has_collision,
    form_builder_field_type_icon, form_builder_fields_overlap, form_builder_linear_grid_index,
    form_builder_occupancy_map, form_builder_section_fields, form_builder_section_layout,
    form_definition_scope_label, form_field_count_label, form_version_desc_sort_key,
    node_count_label, node_display_path, prepared_form_builder_fields,
    prepared_form_builder_sections, rendered_field_layout_label, rendered_field_type_label,
    response_input_value, response_selected_assignment, response_start_can_submit,
    submission_assignee_label, submission_progress_label, submission_status_key,
    submission_status_label, submission_step_label, submission_value_maps,
    submission_workflow_label, workflow_assigned_user_links, workflow_assignee_label,
    workflow_assignment_assignee_label, workflow_assignment_candidate_key,
    workflow_assignment_revision_label, workflow_assignment_state, workflow_assignment_state_label,
    workflow_assignment_status_key, workflow_assignment_status_label,
    workflow_available_node_links, workflow_available_nodes_label,
    workflow_definition_status_label, workflow_definition_version_label,
    workflow_description_label, workflow_revision_label_from_raw, workflow_source_label,
    workflow_status_key, workflow_status_label, workflow_version_label,
};
use crate::features::workflows::submission::*;
use crate::features::workflows::submission::{FormBuilderFieldDraft, FormBuilderSectionDraft};
use crate::types::route_params::{
    AccountRouteParams, DashboardRouteParams, FormRouteParams, NodeRouteParams,
    NodeTypeRouteParams, ReportRouteParams, RoleRouteParams, SubmissionRouteParams,
    WorkflowRouteParams, WorkflowRouteParams as WorkflowRouteParamsForShared, require_route_params,
};
use crate::ui::components::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, DataTable, DropdownMenu, EmptyState, InfoListTable, InfoRow, PageHeader,
    SearchableDataTable, StatusBadge, Tabs, TabsContent, TabsList, TabsTrigger, Timestamp,
};
use crate::utils::pagination::{
    pagination_current_page, pagination_page_count, pagination_page_end, pagination_page_start,
};
use crate::utils::text;
use crate::utils::text::text_matches;

use crate::ui::empty_view;
use icons::{
    ArrowDown, ArrowUp, CalendarDays, ChevronDown, ChevronRight, CircleDot, ExternalLink, FileText,
    Hash, ListChecks, ListFilter, LockKeyhole, Mail, PanelRight, Pencil, Plus, Search,
    SquareCheckBig, TextCursorInput, TextQuote, Trash2, X,
};
use leptos::portal::Portal;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
#[cfg(feature = "hydrate")]
use std::{cell::Cell, cell::RefCell, rc::Rc};
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

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

pub(crate) fn form_status_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| sentence_label(&version.status))
        .unwrap_or_else(|| "No versions".to_string())
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

pub(crate) fn workflow_revision_label_from_option(label: Option<String>) -> String {
    label
        .as_deref()
        .map(crate::features::workflows::submission::workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
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

pub(crate) fn status_badge_class(status: &str) -> &'static str {
    match status {
        "published" | "done" | "active" | "submitted" => "status-badge is-success",
        "draft" | "in_progress" => "status-badge is-warning",
        "error" | "archived" => "status-badge is-danger",
        _ => "status-badge is-info",
    }
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

            match gloo_net::http::Request::get("/api/workflow-assignments")
                .send()
                .await
            {
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
fn load_pending_work(
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

fn start_workflow_assignment_response(
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
pub(crate) async fn send_json_id_request(
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
