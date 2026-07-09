use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

/// Payload for creating a component identity before versions exist.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateComponentRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) version: Option<CreateComponentVersionRequest>,
}

/// Mutable shell metadata for a component identity.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateComponentRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
}

/// Atomic edit-screen command for component shell metadata plus version action.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveComponentEditRequest {
    #[serde(default)]
    pub(crate) component_id: Option<Uuid>,
    #[serde(default)]
    pub(crate) draft_version_id: Option<Uuid>,
    #[serde(default)]
    pub(crate) published_version_id: Option<Uuid>,
    pub(crate) action: SaveComponentEditAction,
    pub(crate) component: UpdateComponentRequest,
    pub(crate) version: CreateComponentVersionRequest,
}

/// Author-selected edit-screen version action.
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SaveComponentEditAction {
    SaveDraft,
    UpdateExistingVersion,
    CreateNewVersion,
}

/// Payload for creating a component version bound to a dataset major line.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateComponentVersionRequest {
    #[serde(default)]
    pub(crate) dataset_id: Option<Uuid>,
    #[serde(default)]
    pub(crate) dataset_version_major: Option<i32>,
    pub(crate) component_type: String,
    pub(crate) config: Value,
    #[serde(default)]
    pub(crate) version_note: Option<String>,
}

/// Component validation response used by authoring and pre-publish flows.
#[derive(Serialize)]
pub struct ComponentValidationResponse {
    pub(crate) valid: bool,
    pub(crate) findings: Vec<ComponentValidationFinding>,
}

/// One stable component validation finding.
#[derive(Serialize)]
pub struct ComponentValidationFinding {
    pub(crate) code: String,
    pub(crate) severity: String,
    pub(crate) field_path: Option<String>,
    pub(crate) message: String,
}

/// Compact component row used by list surfaces.
#[derive(Serialize)]
pub struct ComponentSummary {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
    pub(crate) current_version_id: Option<Uuid>,
    pub(crate) current_version_label: Option<String>,
    pub(crate) current_component_type: Option<String>,
    pub(crate) draft_version_id: Option<Uuid>,
    pub(crate) draft_version_label: Option<String>,
}

/// Component detail with its revision history.
#[derive(Serialize)]
pub struct ComponentDefinition {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
    pub(crate) versions: Vec<ComponentVersionSummary>,
}

/// One version of a component presentation configuration.
#[derive(Serialize)]
pub struct ComponentVersionSummary {
    pub(crate) id: Uuid,
    pub(crate) component_id: Uuid,
    pub(crate) dataset_id: Uuid,
    pub(crate) dataset_version_major: i32,
    pub(crate) binding_mode: String,
    pub(crate) component_type: String,
    pub(crate) status: String,
    pub(crate) version_label: String,
    pub(crate) version_note: String,
    pub(crate) config: Value,
}

/// Executed component table response.
#[derive(Serialize)]
pub struct ComponentTable {
    pub(crate) component_id: Uuid,
    pub(crate) component_version_id: Uuid,
    pub(crate) dataset_id: Uuid,
    pub(crate) dataset_version_major: i32,
    pub(crate) component_type: String,
    pub(crate) materialization_state: String,
    pub(crate) columns: Vec<ComponentTableColumn>,
    pub(crate) rows: Vec<ComponentTableRow>,
    pub(crate) pagination: ComponentTablePagination,
}

/// Server-derived pagination state for component table viewers.
#[derive(Serialize)]
pub struct ComponentTablePagination {
    pub(crate) page_size: usize,
    pub(crate) next_cursor: Option<String>,
    pub(crate) has_more: bool,
}

/// One rendered component table column.
#[derive(Serialize)]
pub struct ComponentTableColumn {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
}

/// One rendered component table row.
#[derive(Serialize)]
pub struct ComponentTableRow {
    pub(crate) row_id: String,
    pub(crate) values: BTreeMap<String, Option<String>>,
}
