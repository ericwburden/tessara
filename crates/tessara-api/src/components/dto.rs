use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Payload for creating a component identity before versions exist.
#[derive(Deserialize)]
pub struct CreateComponentRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
}

/// Payload for creating a component version bound to one dataset revision.
#[derive(Deserialize)]
pub struct CreateComponentVersionRequest {
    pub(crate) dataset_revision_id: Uuid,
    pub(crate) component_type: String,
    pub(crate) config: Value,
    pub(crate) publish: Option<bool>,
}

/// Compact component row used by list surfaces.
#[derive(Serialize)]
pub struct ComponentSummary {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
    pub(crate) current_version_id: Option<Uuid>,
    pub(crate) current_component_type: Option<String>,
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
    pub(crate) dataset_revision_id: Uuid,
    pub(crate) component_type: String,
    pub(crate) status: String,
    pub(crate) version_label: String,
    pub(crate) config: Value,
}
