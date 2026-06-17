//! Data contracts for the Forms feature.
//!
//! Keep API response shapes, request payloads, and feature-local value objects here when they are owned by Forms.

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateFormPayload {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateFormPayload {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateFormSectionPayload {
    pub(crate) title: String,
    pub(crate) position: i32,
    pub(crate) description: String,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateFormFieldPayload {
    pub(crate) section_id: String,
    pub(crate) field_id: Option<String>,
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
    pub(crate) field_id: String,
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
