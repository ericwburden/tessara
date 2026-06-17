//! Signal-aware field actions for dataset editor sources.

use super::source_options::source_field_options;
use crate::features::datasets::types::{
    DatasetAggregationDraft, DatasetFieldDraft, DatasetFormOption, DatasetRenderedForm,
    DatasetSourceDraft,
};
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn add_fields_from_source(
    index: usize,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
) {
    let source = sources.get().get(index).cloned();
    if let Some(source) = source {
        let options = source_field_options(
            &sources.get(),
            &forms.get(),
            &rendered_forms.get(),
            &source.source_alias,
        );
        fields.update(|items| {
            for option in options {
                let key = canonical_field_key(&source.source_alias, &option.key);
                if items.iter().any(|item| {
                    item.key == key
                        || (item.source_alias == source.source_alias
                            && item.source_field_key == option.key)
                }) {
                    continue;
                }
                items.push(DatasetFieldDraft {
                    key,
                    label: option.label,
                    source_alias: source.source_alias.clone(),
                    source_field_key: option.key,
                    field_type: option.field_type,
                });
            }
        });
    }
}

pub(crate) fn canonical_field_key(source_alias: &str, source_field_key: &str) -> String {
    let field_key = source_field_key.trim_start_matches('_');
    if field_key.is_empty() {
        source_alias.into()
    } else {
        format!("{source_alias}__{field_key}")
    }
}

pub(crate) fn rename_source_alias_references(
    old_alias: &str,
    new_alias: &str,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    aggregation: RwSignal<DatasetAggregationDraft>,
) {
    if old_alias == new_alias {
        return;
    }

    let mut renamed_field_keys = BTreeMap::new();
    fields.update(|items| {
        for field in items
            .iter_mut()
            .filter(|field| field.source_alias == old_alias)
        {
            let previous_key = field.key.clone();
            field.source_alias = new_alias.to_string();
            field.key = canonical_field_key(new_alias, &field.source_field_key);
            renamed_field_keys.insert(previous_key, field.key.clone());
        }
        let mut seen = BTreeSet::new();
        items.retain(|field| {
            seen.insert((field.source_alias.clone(), field.source_field_key.clone()))
        });
    });

    if renamed_field_keys.is_empty() {
        return;
    }

    aggregation.update(|draft| {
        for group_field in &mut draft.group_fields {
            if let Some(renamed) = renamed_field_keys.get(group_field) {
                *group_field = renamed.clone();
            }
        }
        for metric in &mut draft.metrics {
            if let Some(renamed) = renamed_field_keys.get(&metric.source_field_key) {
                metric.source_field_key = renamed.clone();
            }
        }
        if let Some(row_picker) = &mut draft.row_picker {
            for sort_field in &mut row_picker.sort_fields {
                if let Some(renamed) = renamed_field_keys.get(&sort_field.field_key) {
                    sort_field.field_key = renamed.clone();
                }
            }
        }
    });
}
