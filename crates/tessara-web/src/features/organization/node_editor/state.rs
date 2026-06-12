//! Signal groups for organization node editor pages.

use crate::features::organization::types::{
    NodeMetadataFieldSummary, NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail,
};
use leptos::prelude::*;
use std::collections::HashMap;

pub(super) struct OrganizationNodeCreateState {
    pub(super) node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    pub(super) nodes: RwSignal<Vec<OrganizationNode>>,
    pub(super) selected_node_type_id: RwSignal<String>,
    pub(super) selected_parent_node_id: RwSignal<String>,
    pub(super) name: RwSignal<String>,
    pub(super) metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    pub(super) metadata_values: RwSignal<HashMap<String, String>>,
    pub(super) metadata_booleans: RwSignal<HashMap<String, bool>>,
    pub(super) is_loading: RwSignal<bool>,
    pub(super) is_saving: RwSignal<bool>,
    pub(super) message: RwSignal<Option<String>>,
}

impl OrganizationNodeCreateState {
    pub(super) fn new() -> Self {
        Self {
            node_types: RwSignal::new(Vec::new()),
            nodes: RwSignal::new(Vec::new()),
            selected_node_type_id: RwSignal::new(String::new()),
            selected_parent_node_id: RwSignal::new(String::new()),
            name: RwSignal::new(String::new()),
            metadata_fields: RwSignal::new(Vec::new()),
            metadata_values: RwSignal::new(HashMap::new()),
            metadata_booleans: RwSignal::new(HashMap::new()),
            is_loading: RwSignal::new(true),
            is_saving: RwSignal::new(false),
            message: RwSignal::new(None),
        }
    }
}

pub(super) struct OrganizationNodeEditState {
    pub(super) node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    pub(super) nodes: RwSignal<Vec<OrganizationNode>>,
    pub(super) detail: RwSignal<Option<OrganizationNodeDetail>>,
    pub(super) selected_parent_node_id: RwSignal<String>,
    pub(super) name: RwSignal<String>,
    pub(super) metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    pub(super) metadata_values: RwSignal<HashMap<String, String>>,
    pub(super) metadata_booleans: RwSignal<HashMap<String, bool>>,
    pub(super) is_loading: RwSignal<bool>,
    pub(super) is_saving: RwSignal<bool>,
    pub(super) message: RwSignal<Option<String>>,
}

impl OrganizationNodeEditState {
    pub(super) fn new() -> Self {
        Self {
            node_types: RwSignal::new(Vec::new()),
            nodes: RwSignal::new(Vec::new()),
            detail: RwSignal::new(None),
            selected_parent_node_id: RwSignal::new(String::new()),
            name: RwSignal::new(String::new()),
            metadata_fields: RwSignal::new(Vec::new()),
            metadata_values: RwSignal::new(HashMap::new()),
            metadata_booleans: RwSignal::new(HashMap::new()),
            is_loading: RwSignal::new(true),
            is_saving: RwSignal::new(false),
            message: RwSignal::new(None),
        }
    }
}
