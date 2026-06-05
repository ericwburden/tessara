//! Request and response shapes for form endpoints.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub(crate) struct CreateFormRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<Uuid>,
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateFormRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<Uuid>,
    #[serde(default)]
    pub(crate) visibility_node_ids: Vec<Uuid>,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct CreateFormVersionRequest {}

#[derive(Deserialize)]
pub(crate) struct CreateFormSectionRequest {
    pub(crate) title: String,
    pub(crate) position: i32,
    #[serde(default = "default_form_section_description")]
    pub(crate) description: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateFormFieldRequest {
    pub(crate) section_id: Uuid,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
    pub(crate) position: i32,
    #[serde(default = "default_form_field_grid_row")]
    pub(crate) grid_row: i32,
    #[serde(default = "default_form_field_grid_column")]
    pub(crate) grid_column: i32,
    #[serde(default = "default_form_field_grid_width")]
    pub(crate) grid_width: i32,
    #[serde(default = "default_form_field_grid_height")]
    pub(crate) grid_height: i32,
}

#[derive(Deserialize)]
pub(crate) struct UpdateFormSectionRequest {
    pub(crate) title: String,
    pub(crate) position: i32,
    #[serde(default = "default_form_section_description")]
    pub(crate) description: String,
}

#[derive(Deserialize)]
pub(crate) struct UpdateFormFieldRequest {
    pub(crate) section_id: Uuid,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
    pub(crate) position: i32,
    #[serde(default = "default_form_field_grid_row")]
    pub(crate) grid_row: i32,
    #[serde(default = "default_form_field_grid_column")]
    pub(crate) grid_column: i32,
    #[serde(default = "default_form_field_grid_width")]
    pub(crate) grid_width: i32,
    #[serde(default = "default_form_field_grid_height")]
    pub(crate) grid_height: i32,
}

#[derive(Serialize)]
pub(crate) struct RenderedForm {
    pub(crate) form_version_id: Uuid,
    pub(crate) form_id: Uuid,
    pub(crate) form_name: String,
    pub(crate) version_label: Option<String>,
    pub(crate) status: String,
    pub(crate) sections: Vec<RenderedSection>,
}

#[derive(Serialize)]
pub(crate) struct RenderedSection {
    pub(crate) id: Uuid,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) position: i32,
    pub(crate) fields: Vec<RenderedField>,
}

fn default_form_section_description() -> String {
    String::new()
}

fn default_form_field_grid_row() -> i32 {
    1
}

fn default_form_field_grid_column() -> i32 {
    1
}

fn default_form_field_grid_width() -> i32 {
    1
}

fn default_form_field_grid_height() -> i32 {
    1
}

#[derive(Serialize)]
pub(crate) struct RenderedField {
    pub(crate) id: Uuid,
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

#[derive(Serialize)]
pub(crate) struct FormSummary {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<Uuid>,
    pub(crate) scope_node_type_name: Option<String>,
    pub(crate) visibility_nodes: Vec<FormVisibilityNodeSummary>,
    pub(crate) versions: Vec<FormVersionSummary>,
}

#[derive(Serialize)]
pub(crate) struct FormDefinition {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<Uuid>,
    pub(crate) scope_node_type_name: Option<String>,
    pub(crate) visibility_nodes: Vec<FormVisibilityNodeSummary>,
    pub(crate) versions: Vec<FormVersionSummary>,
    pub(crate) workflows: Vec<FormWorkflowLink>,
    pub(crate) dataset_sources: Vec<FormDatasetSourceLink>,
}

#[derive(Serialize)]
pub(crate) struct FormVersionSummary {
    pub(crate) id: Uuid,
    pub(crate) version_label: Option<String>,
    pub(crate) status: String,
    pub(crate) version_major: Option<i32>,
    pub(crate) version_minor: Option<i32>,
    pub(crate) version_patch: Option<i32>,
    pub(crate) compatibility_group_id: Option<Uuid>,
    pub(crate) compatibility_group_name: Option<String>,
    pub(crate) published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) field_count: i64,
    pub(crate) semantic_bump: Option<String>,
    pub(crate) started_new_major_line: Option<bool>,
    pub(crate) publish_preview: Option<FormPublishPreview>,
    pub(crate) assignment_nodes: Vec<FormVersionAssignmentNodeSummary>,
}

#[derive(Clone, Serialize)]
pub(crate) struct FormVersionAssignmentNodeSummary {
    pub(crate) node_id: Uuid,
    pub(crate) node_name: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<Uuid>,
    pub(crate) node_path: String,
}

#[derive(Clone, Serialize)]
pub(crate) struct FormVisibilityNodeSummary {
    pub(crate) node_id: Uuid,
    pub(crate) node_name: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<Uuid>,
    pub(crate) node_path: String,
}

#[derive(Serialize)]
pub(crate) struct FormPublishPreview {
    pub(crate) version_label: String,
    pub(crate) version_major: i32,
    pub(crate) version_minor: i32,
    pub(crate) version_patch: i32,
    pub(crate) semantic_bump: String,
    pub(crate) compatibility_label: String,
    pub(crate) starts_new_major_line: bool,
    pub(crate) dependency_warnings: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct PublishFormVersionResponse {
    pub(crate) id: Uuid,
    pub(crate) version_label: String,
    pub(crate) version_major: i32,
    pub(crate) version_minor: i32,
    pub(crate) version_patch: i32,
    pub(crate) semantic_bump: String,
    pub(crate) compatibility_label: String,
    pub(crate) status: String,
    pub(crate) published_at: chrono::DateTime<chrono::Utc>,
    pub(crate) dependency_warnings: Vec<String>,
    pub(crate) starts_new_major_line: bool,
}

#[derive(Serialize)]
pub(crate) struct FormDatasetSourceLink {
    pub(crate) dataset_id: Uuid,
    pub(crate) dataset_name: String,
    pub(crate) source_alias: String,
    pub(crate) selection_rule: String,
}

#[derive(Serialize)]
pub(crate) struct FormWorkflowLink {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) source: String,
    pub(crate) current_version_id: Option<Uuid>,
    pub(crate) current_version_label: Option<String>,
    pub(crate) current_status: Option<String>,
    pub(crate) assignment_count: i64,
}

#[derive(Serialize)]
pub(crate) struct PublishedFormVersionSummary {
    pub(crate) form_id: Uuid,
    pub(crate) form_name: String,
    pub(crate) form_slug: String,
    pub(crate) form_version_id: Uuid,
    pub(crate) version_label: String,
    pub(crate) published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) field_count: i64,
}
