//! Dataset editor source composition components.

use super::super::loaders::load_rendered_form;
use super::super::types::*;
use super::SourceOptionsFields;
use super::source_field_actions::rename_source_alias_references;
use icons::ChevronsUpDown;
use leptos::prelude::*;
use std::collections::BTreeMap;

#[component]
pub(crate) fn DatasetSourcesEditor(
    initial_source: RwSignal<DatasetSourceDraft>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);

    Effect::new(move |_| {
        let source = initial_source.get();
        if source.input_kind == "form" && !source.form_version_id.is_empty() {
            load_rendered_form(source.form_version_id, rendered_forms);
        }
    });

    let on_source_change = Callback::new(move |source: DatasetSourceDraft| {
        let previous_alias = initial_source.get().source_alias;
        let next_alias = source.source_alias.clone();
        initial_source.set(source);
        if previous_alias != next_alias {
            rename_source_alias_references(&previous_alias, &next_alias, operation_order);
        }
    });

    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <button
                    class="dataset-editor-section__header dataset-sql-header dataset-editor-section__collapse"
                    type="button"
                    aria-expanded=move || is_open.get().to_string()
                    on:click=move |_| is_open.update(|open| *open = !*open)
                >
                    <h3>"Initial Data Source"</h3>
                    <ChevronsUpDown class="dataset-sql-header__icon"/>
                </button>
            </div>
            {move || if is_open.get() {
                view! {
                    <div class="dataset-initial-source">
                        <SourceOptionsFields
                            source_signal=Signal::derive(move || initial_source.get())
                            on_source_change
                            forms
                            datasets
                            rendered_forms
                        />
                    </div>
                }.into_any()
            } else {
                view! { <span class="dataset-editor-section__collapsed-spacer"></span> }.into_any()
            }}
        </section>
    }
}
