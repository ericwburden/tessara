//! Request and response shapes for hierarchy endpoints.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub(crate) struct CreateNodeTypeRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) plural_label: Option<String>,
    #[serde(default)]
    pub(crate) parent_node_type_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub(crate) child_node_type_ids: Option<Vec<Uuid>>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateNodeTypeRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) plural_label: Option<String>,
    #[serde(default)]
    pub(crate) parent_node_type_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub(crate) child_node_type_ids: Option<Vec<Uuid>>,
}

#[derive(Deserialize)]
pub(crate) struct CreateNodeTypeRelationshipRequest {
    pub(crate) parent_node_type_id: Uuid,
    pub(crate) child_node_type_id: Uuid,
}

#[derive(Deserialize)]
pub(crate) struct CreateNodeMetadataFieldRequest {
    pub(crate) node_type_id: Uuid,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
}

#[derive(Deserialize)]
pub(crate) struct UpdateNodeMetadataFieldRequest {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
}

#[derive(Deserialize)]
pub(crate) struct CreateNodeRequest {
    pub(crate) node_type_id: Uuid,
    pub(crate) parent_node_id: Option<Uuid>,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) metadata: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub(crate) struct UpdateNodeRequest {
    pub(crate) parent_node_id: Option<Uuid>,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) metadata: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub(crate) struct ListNodesQuery {
    pub(crate) q: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct IdResponse {
    pub id: Uuid,
}

#[derive(Serialize)]
pub(crate) struct NodeResponse {
    pub(crate) id: Uuid,
    pub(crate) node_type_id: Uuid,
    pub(crate) node_type_name: String,
    pub(crate) node_type_slug: String,
    pub(crate) node_type_singular_label: String,
    pub(crate) node_type_plural_label: String,
    pub(crate) parent_node_id: Option<Uuid>,
    pub(crate) parent_node_name: Option<String>,
    pub(crate) name: String,
    pub(crate) metadata: Value,
}

#[derive(Serialize)]
pub(crate) struct NodeDetail {
    pub(crate) id: Uuid,
    pub(crate) node_type_id: Uuid,
    pub(crate) node_type_name: String,
    pub(crate) node_type_slug: String,
    pub(crate) node_type_singular_label: String,
    pub(crate) node_type_plural_label: String,
    pub(crate) parent_node_id: Option<Uuid>,
    pub(crate) parent_node_name: Option<String>,
    pub(crate) name: String,
    pub(crate) metadata: Value,
    pub(crate) related_forms: Vec<NodeFormLink>,
    pub(crate) related_responses: Vec<NodeSubmissionLink>,
    pub(crate) related_dashboards: Vec<NodeDashboardLink>,
}

#[derive(Serialize)]
pub(crate) struct NodeFormLink {
    pub(crate) form_id: Uuid,
    pub(crate) form_name: String,
    pub(crate) form_slug: String,
    pub(crate) published_version_count: i64,
    pub(crate) active_version_label: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct NodeSubmissionLink {
    pub(crate) submission_id: Uuid,
    pub(crate) form_id: Uuid,
    pub(crate) form_name: String,
    pub(crate) form_version_id: Uuid,
    pub(crate) version_label: String,
    pub(crate) status: String,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) submitted_by: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct NodeDashboardLink {
    pub(crate) dashboard_id: Uuid,
    pub(crate) dashboard_name: String,
    pub(crate) component_count: i64,
    pub(crate) description: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct NodeTypeSummary {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
    pub(crate) is_root_type: bool,
    pub(crate) node_count: i64,
}

#[derive(Serialize, Clone)]
pub(crate) struct NodeTypePeerLink {
    pub(crate) node_type_id: Uuid,
    pub(crate) node_type_name: String,
    pub(crate) node_type_slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
}

#[derive(Serialize)]
pub(crate) struct NodeTypeCatalogEntry {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
    pub(crate) is_root_type: bool,
    pub(crate) node_count: i64,
    pub(crate) parent_relationships: Vec<NodeTypePeerLink>,
    pub(crate) child_relationships: Vec<NodeTypePeerLink>,
}

#[derive(Serialize)]
pub(crate) struct NodeTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
    pub(crate) is_root_type: bool,
    pub(crate) node_count: i64,
    pub(crate) parent_relationships: Vec<NodeTypePeerLink>,
    pub(crate) child_relationships: Vec<NodeTypePeerLink>,
    pub(crate) metadata_fields: Vec<NodeMetadataFieldSummary>,
    pub(crate) scoped_forms: Vec<NodeTypeFormLink>,
}

#[derive(Serialize)]
pub(crate) struct NodeTypeRelationshipSummary {
    pub(crate) parent_node_type_id: Uuid,
    pub(crate) parent_name: String,
    pub(crate) child_node_type_id: Uuid,
    pub(crate) child_name: String,
}

#[derive(Serialize)]
pub(crate) struct NodeMetadataFieldSummary {
    pub(crate) id: Uuid,
    pub(crate) node_type_id: Uuid,
    pub(crate) node_type_name: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
}

#[derive(Serialize)]
pub(crate) struct NodeTypeFormLink {
    pub(crate) form_id: Uuid,
    pub(crate) form_name: String,
    pub(crate) form_slug: String,
}
