//! Signal state for the dataset editor surface.

use crate::features::datasets::types::{
    DatasetFormOption, DatasetOperationDraft, DatasetRenderedForm, DatasetSourceDraft,
    DatasetSummary, DatasetUserOption, NodeResponse,
};
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
pub(crate) struct DatasetEditorState {
    pub(crate) name: RwSignal<String>,
    pub(crate) slug: RwSignal<String>,
    pub(crate) visibility_node_ids: RwSignal<BTreeSet<String>>,
    pub(crate) initial_source: RwSignal<DatasetSourceDraft>,
    pub(crate) operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    pub(crate) restriction_internal_field_key: RwSignal<String>,
    pub(crate) restriction_restricted_field_key: RwSignal<String>,
    pub(crate) restriction_confidential_field_key: RwSignal<String>,
    pub(crate) forms: RwSignal<Vec<DatasetFormOption>>,
    pub(crate) datasets: RwSignal<Vec<DatasetSummary>>,
    pub(crate) nodes: RwSignal<Vec<NodeResponse>>,
    pub(crate) users: RwSignal<Vec<DatasetUserOption>>,
    pub(crate) rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    pub(crate) load_error: RwSignal<Option<String>>,
    pub(crate) save_error: RwSignal<Option<String>>,
    pub(crate) save_message: RwSignal<Option<String>>,
    pub(crate) sql_preview: RwSignal<Option<String>>,
    pub(crate) sql_preview_error: RwSignal<Option<String>>,
    pub(crate) sql_preview_expanded: RwSignal<bool>,
    pub(crate) visibility_search: RwSignal<String>,
    pub(crate) visibility_expanded_node_ids: RwSignal<BTreeSet<String>>,
}

impl DatasetEditorState {
    pub(crate) fn new() -> Self {
        Self {
            name: RwSignal::new(String::new()),
            slug: RwSignal::new(String::new()),
            visibility_node_ids: RwSignal::new(BTreeSet::<String>::new()),
            initial_source: RwSignal::new(DatasetSourceDraft::default()),
            operation_order: RwSignal::new(Vec::<DatasetOperationDraft>::new()),
            restriction_internal_field_key: RwSignal::new(String::new()),
            restriction_restricted_field_key: RwSignal::new(String::new()),
            restriction_confidential_field_key: RwSignal::new(String::new()),
            forms: RwSignal::new(Vec::<DatasetFormOption>::new()),
            datasets: RwSignal::new(Vec::<DatasetSummary>::new()),
            nodes: RwSignal::new(Vec::<NodeResponse>::new()),
            users: RwSignal::new(Vec::<DatasetUserOption>::new()),
            rendered_forms: RwSignal::new(BTreeMap::<String, DatasetRenderedForm>::new()),
            load_error: RwSignal::new(None::<String>),
            save_error: RwSignal::new(None::<String>),
            save_message: RwSignal::new(None::<String>),
            sql_preview: RwSignal::new(None::<String>),
            sql_preview_error: RwSignal::new(None::<String>),
            sql_preview_expanded: RwSignal::new(false),
            visibility_search: RwSignal::new(String::new()),
            visibility_expanded_node_ids: RwSignal::new(BTreeSet::<String>::new()),
        }
    }
}
