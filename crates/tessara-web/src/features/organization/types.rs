//! Data contracts for the Organization feature.
//!
//! Keep API response shapes, request payloads, and feature-local value objects here when they are owned by Organization.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct OrganizationNode {
    pub(crate) id: String,
    pub(crate) node_type_name: String,
    pub(crate) node_type_singular_label: String,
    pub(crate) node_type_plural_label: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) parent_node_name: Option<String>,
    pub(crate) node_type_id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct OrganizationNodeDetail {
    pub(crate) id: String,
    pub(crate) node_type_id: String,
    pub(crate) node_type_name: String,
    pub(crate) node_type_singular_label: String,
    pub(crate) node_type_plural_label: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) parent_node_name: Option<String>,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) metadata: Value,
    #[serde(default)]
    pub(crate) related_forms: Vec<NodeFormLink>,
    #[serde(default)]
    pub(crate) related_responses: Vec<NodeSubmissionLink>,
    #[serde(default)]
    pub(crate) related_dashboards: Vec<NodeDashboardLink>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct NodeFormLink {
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_slug: String,
    pub(crate) published_version_count: i64,
    pub(crate) active_version_label: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct NodeSubmissionLink {
    pub(crate) submission_id: String,
    pub(crate) form_name: String,
    pub(crate) version_label: String,
    pub(crate) status: String,
    pub(crate) created_at: String,
    pub(crate) submitted_at: Option<String>,
    pub(crate) submitted_by: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct NodeDashboardLink {
    pub(crate) dashboard_id: String,
    pub(crate) dashboard_name: String,
    pub(crate) component_count: i64,
    pub(crate) description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct NodeTypeCatalogEntry {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
    pub(crate) is_root_type: bool,
    pub(crate) node_count: i64,
    #[serde(default)]
    pub(crate) parent_relationships: Vec<NodeTypePeerLink>,
    #[serde(default)]
    pub(crate) child_relationships: Vec<NodeTypePeerLink>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct NodeTypePeerLink {
    pub(crate) node_type_id: String,
    pub(crate) node_type_name: String,
    pub(crate) node_type_slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct NodeTypeDefinition {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
    pub(crate) is_root_type: bool,
    pub(crate) node_count: i64,
    #[serde(default)]
    pub(crate) parent_relationships: Vec<NodeTypePeerLink>,
    #[serde(default)]
    pub(crate) child_relationships: Vec<NodeTypePeerLink>,
    #[serde(default)]
    pub(crate) metadata_fields: Vec<NodeMetadataFieldSummary>,
    #[serde(default)]
    pub(crate) scoped_forms: Vec<NodeTypeFormLink>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct NodeTypeFormLink {
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_slug: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct NodeMetadataFieldSummary {
    pub(crate) id: String,
    pub(crate) node_type_id: String,
    pub(crate) node_type_name: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct NodeTypeUpsertRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) plural_label: Option<String>,
    pub(crate) parent_node_type_ids: Vec<String>,
    pub(crate) child_node_type_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateNodeMetadataFieldRequest {
    pub(crate) node_type_id: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateNodeMetadataFieldRequest {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminRoleSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) capability_count: i64,
    pub(crate) account_count: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrganizationTreeNode {
    pub(crate) node: OrganizationNode,
    pub(crate) children: Vec<OrganizationTreeNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CreateChildLink {
    pub(crate) href: String,
    pub(crate) label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ParentNodeOption {
    pub(crate) id: String,
    pub(crate) label: String,
}
