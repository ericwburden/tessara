//! Signal state for the node-type administration page.

use crate::features::organization::{NodeTypeCatalogEntry, NodeTypeDefinition};
use leptos::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) struct AdministrationNodeTypesPageState {
    pub(super) node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    pub(super) selected_node_type_id: RwSignal<Option<String>>,
    pub(super) selected_detail: RwSignal<Option<NodeTypeDefinition>>,
    pub(super) search: RwSignal<String>,
    pub(super) is_loading: RwSignal<bool>,
    pub(super) detail_loading: RwSignal<bool>,
    pub(super) is_saving: RwSignal<bool>,
    pub(super) is_creating: RwSignal<bool>,
    pub(super) message: RwSignal<Option<String>>,
    pub(super) name: RwSignal<String>,
    pub(super) slug: RwSignal<String>,
    pub(super) plural_label: RwSignal<String>,
    pub(super) parent_node_type_ids: RwSignal<HashSet<String>>,
    pub(super) child_node_type_ids: RwSignal<HashSet<String>>,
}

impl AdministrationNodeTypesPageState {
    pub(super) fn new() -> Self {
        Self {
            node_types: RwSignal::new(Vec::new()),
            selected_node_type_id: RwSignal::new(None),
            selected_detail: RwSignal::new(None),
            search: RwSignal::new(String::new()),
            is_loading: RwSignal::new(true),
            detail_loading: RwSignal::new(false),
            is_saving: RwSignal::new(false),
            is_creating: RwSignal::new(false),
            message: RwSignal::new(None),
            name: RwSignal::new(String::new()),
            slug: RwSignal::new(String::new()),
            plural_label: RwSignal::new(String::new()),
            parent_node_type_ids: RwSignal::new(HashSet::new()),
            child_node_type_ids: RwSignal::new(HashSet::new()),
        }
    }
}
