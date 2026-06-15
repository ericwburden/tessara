//! Signal-aware field actions for dataset editor sources.

use super::source_options::source_field_options;
use crate::features::datasets::types::{
    DatasetFieldDraft, DatasetFormOption, DatasetRenderedForm, DatasetSourceDraft,
};
use leptos::prelude::*;
use std::collections::BTreeMap;

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
