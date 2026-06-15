//! Signal state for the dataset editor surface.

use crate::features::datasets::types::{
    DatasetAggregationDraft, DatasetDesignerSelection, DatasetExpressionDraft, DatasetFieldDraft,
    DatasetFormOption, DatasetRenderedForm, DatasetSourceDraft, DatasetSummary, NodeResponse,
};
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
pub(crate) struct DatasetEditorState {
    pub(crate) name: RwSignal<String>,
    pub(crate) slug: RwSignal<String>,
    pub(crate) composition_mode: RwSignal<String>,
    pub(crate) visibility_node_ids: RwSignal<BTreeSet<String>>,
    pub(crate) sources: RwSignal<Vec<DatasetSourceDraft>>,
    pub(crate) expression: RwSignal<DatasetExpressionDraft>,
    pub(crate) fields: RwSignal<Vec<DatasetFieldDraft>>,
    pub(crate) aggregation: RwSignal<DatasetAggregationDraft>,
    pub(crate) join_left_key: RwSignal<String>,
    pub(crate) join_right_key: RwSignal<String>,
    pub(crate) forms: RwSignal<Vec<DatasetFormOption>>,
    pub(crate) datasets: RwSignal<Vec<DatasetSummary>>,
    pub(crate) nodes: RwSignal<Vec<NodeResponse>>,
    pub(crate) rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    pub(crate) load_error: RwSignal<Option<String>>,
    pub(crate) save_error: RwSignal<Option<String>>,
    pub(crate) save_message: RwSignal<Option<String>>,
    pub(crate) sql_preview: RwSignal<Option<String>>,
    pub(crate) sql_preview_error: RwSignal<Option<String>>,
    pub(crate) sql_preview_expanded: RwSignal<bool>,
    pub(crate) visibility_search: RwSignal<String>,
    pub(crate) visibility_expanded_node_ids: RwSignal<BTreeSet<String>>,
    pub(crate) designer_selection: RwSignal<DatasetDesignerSelection>,
    pub(crate) designer_sheet_open: RwSignal<bool>,
    pub(crate) auto_seeded_sources: RwSignal<BTreeSet<String>>,
}

impl DatasetEditorState {
    pub(crate) fn new() -> Self {
        Self {
            name: RwSignal::new(String::new()),
            slug: RwSignal::new(String::new()),
            composition_mode: RwSignal::new("union".to_string()),
            visibility_node_ids: RwSignal::new(BTreeSet::<String>::new()),
            sources: RwSignal::new(vec![DatasetSourceDraft::default()]),
            expression: RwSignal::new(DatasetExpressionDraft::default()),
            fields: RwSignal::new(Vec::<DatasetFieldDraft>::new()),
            aggregation: RwSignal::new(DatasetAggregationDraft::default()),
            join_left_key: RwSignal::new(String::new()),
            join_right_key: RwSignal::new(String::new()),
            forms: RwSignal::new(Vec::<DatasetFormOption>::new()),
            datasets: RwSignal::new(Vec::<DatasetSummary>::new()),
            nodes: RwSignal::new(Vec::<NodeResponse>::new()),
            rendered_forms: RwSignal::new(BTreeMap::<String, DatasetRenderedForm>::new()),
            load_error: RwSignal::new(None::<String>),
            save_error: RwSignal::new(None::<String>),
            save_message: RwSignal::new(None::<String>),
            sql_preview: RwSignal::new(None::<String>),
            sql_preview_error: RwSignal::new(None::<String>),
            sql_preview_expanded: RwSignal::new(false),
            visibility_search: RwSignal::new(String::new()),
            visibility_expanded_node_ids: RwSignal::new(BTreeSet::<String>::new()),
            designer_selection: RwSignal::new(DatasetDesignerSelection::Operation(Vec::new())),
            designer_sheet_open: RwSignal::new(false),
            auto_seeded_sources: RwSignal::new(BTreeSet::<String>::new()),
        }
    }
}
