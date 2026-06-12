//! Dataset editor source composition components.

use super::super::loaders::load_rendered_form;
use super::super::types::*;
use super::helpers::source_seed_key;
use super::source_options::{
    add_fields_from_source, first_published_version, published_versions_for_form,
    resolved_form_version_id,
};
use super::{DatasetDesignerOptionsSheet, DatasetExpressionChain, ExpressionPreview};
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[component]
/// Renders the dataset sources editor view.
pub(crate) fn DatasetSourcesEditor(
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    composition_mode: RwSignal<String>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
    auto_seeded_sources: RwSignal<BTreeSet<String>>,
) -> impl IntoView {
    Effect::new(move |_| {
        let form_options = forms.get();
        sources.update(|items| {
            for source in items {
                if source.input_kind != "form"
                    || source.form_id.is_empty()
                    || !source.form_version_id.is_empty()
                {
                    continue;
                }
                let version = source
                    .form_version_major
                    .and_then(|major| {
                        published_versions_for_form(&form_options, &source.form_id)
                            .into_iter()
                            .find(|version| version.version_major == Some(major))
                    })
                    .or_else(|| first_published_version(&form_options, &source.form_id));
                if let Some(version) = version {
                    source.form_version_id = version.id;
                    source.form_version_major = version.version_major;
                }
            }
        });
    });

    Effect::new(move |_| {
        let form_options = forms.get();
        for (index, source) in sources.get().into_iter().enumerate() {
            if source.input_kind == "form"
                && let Some(version_id) = resolved_form_version_id(&source, &form_options)
            {
                load_rendered_form(version_id.clone(), rendered_forms);
                if rendered_forms.get().contains_key(&version_id) {
                    let seed_key = source_seed_key(index, &version_id);
                    if !auto_seeded_sources.get().contains(&seed_key) {
                        add_fields_from_source(index, sources, forms, rendered_forms, fields);
                        auto_seeded_sources.update(|keys| {
                            keys.insert(seed_key);
                        });
                    }
                }
            }
        }
    });

    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <h3>"Operation Designer"</h3>
                <button class="button button--secondary button--compact" type="button" on:click=move |_| {
                    let next = sources.get().len() + 1;
                    sources.update(|items| items.push(DatasetSourceDraft { source_alias: format!("source_{next}"), ..DatasetSourceDraft::default() }));
                    designer_selection.set(DatasetDesignerSelection::Source(next - 1));
                    designer_sheet_open.set(true);
                }>"Add Input"</button>
            </div>
            <div class="dataset-expression-workspace">
                <div class="dataset-expression-canvas">
                    <ExpressionPreview sources=sources composition_mode/>
                    <DatasetExpressionChain
                        sources
                        fields
                        composition_mode
                        designer_selection
                        designer_sheet_open
                    />
                </div>
                <DatasetDesignerOptionsSheet
                    selection=designer_selection
                    is_open=designer_sheet_open
                    sources
                    forms
                    datasets
                    rendered_forms
                    composition_mode
                    fields
                    join_left_key
                    join_right_key
                />
            </div>
        </section>
    }
}
