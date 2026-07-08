//! Dataset editor filter controls.

use super::source_options::source_field_options;
use crate::types::{
    DatasetFieldDraft, DatasetFormOption, DatasetRenderedForm, DatasetRowFilterDraft,
    DatasetSourceDraft, DatasetUserOption, NodeResponse,
};
use leptos::prelude::*;
use std::collections::BTreeMap;
use tessara_web_data_ops::DataOpsFiltersEditor;

#[component]
pub(crate) fn DatasetFiltersEditor(
    fields: Signal<Vec<DatasetFieldDraft>>,
    initial_source: RwSignal<DatasetSourceDraft>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    nodes: RwSignal<Vec<NodeResponse>>,
    users: RwSignal<Vec<DatasetUserOption>>,
    row_filters: Signal<Vec<DatasetRowFilterDraft>>,
    on_row_filters_change: Callback<Vec<DatasetRowFilterDraft>>,
    #[prop(optional)] embedded: bool,
) -> impl IntoView {
    let value_options = Callback::new(move |field: DatasetFieldDraft| {
        filter_value_options(
            &field,
            &initial_source.get(),
            &forms.get(),
            &rendered_forms.get(),
            &nodes.get(),
            &users.get(),
        )
    });
    view! {
        <DataOpsFiltersEditor
            fields=fields
            row_filters=row_filters
            on_row_filters_change=on_row_filters_change
            value_options_provider=value_options
            allow_field_comparison=true
            embedded=embedded
        />
    }
}

fn filter_value_options(
    field: &DatasetFieldDraft,
    initial_source: &DatasetSourceDraft,
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    nodes: &[NodeResponse],
    users: &[DatasetUserOption],
) -> Vec<String> {
    if field.field_type == "boolean" {
        return vec!["true".into(), "false".into()];
    }
    if field.source_field_key == "__node_name" {
        return sorted_unique_options(nodes.iter().map(|node| node.name.clone()).collect());
    }
    if field.source_field_key == "__node_id" {
        return sorted_unique_options(nodes.iter().map(|node| node.id.clone()).collect());
    }
    if field.source_field_key == "__last_updated_by_user_name" {
        return sorted_unique_options(users.iter().map(|user| user.display_name.clone()).collect());
    }
    let sources = [initial_source.clone()];
    let mut options =
        source_field_options(&sources, &[], forms, rendered_forms, &field.source_alias)
            .into_iter()
            .find(|option| option.key == field.source_field_key)
            .map(|option| option.value_options)
            .unwrap_or_default();
    options.sort();
    options.dedup();
    options
}

fn sorted_unique_options(mut options: Vec<String>) -> Vec<String> {
    options.retain(|option| !option.trim().is_empty());
    options.sort();
    options.dedup();
    options
}
