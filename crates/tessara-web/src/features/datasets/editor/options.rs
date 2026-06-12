//! Dataset editor designer option sheet components.

use super::super::loaders::load_rendered_form;
use super::super::types::*;
use super::helpers::version_label;
use super::source_options::{
    add_fields_from_source, find_version, first_published_version, published_versions_for_form,
};
use super::{FieldOptionsPanel, OperationOptionsPanel};
use icons::X;
use leptos::portal::Portal;
use leptos::prelude::*;
use std::collections::BTreeMap;
#[component]
/// Renders the dataset designer options sheet view.
pub(crate) fn DatasetDesignerOptionsSheet(
    selection: RwSignal<DatasetDesignerSelection>,
    is_open: RwSignal<bool>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    composition_mode: RwSignal<String>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
) -> impl IntoView {
    view! {
        <Portal>
            <Show when=move || is_open.get()>
                <section class="sheet-overlay dataset-options-overlay" aria-label="Dataset designer options overlay">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close dataset designer options" on:click=move |_| is_open.set(false)></button>
                    <aside class="sheet-panel blurred-surface dataset-options-sheet" role="dialog" aria-modal="true" aria-label="Dataset designer options">
                        <div class="sheet-panel__actions">
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close dataset designer options" title="Close dataset designer options" on:click=move |_| is_open.set(false)>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || match selection.get() {
                            DatasetDesignerSelection::Operation => view! {
                                <OperationOptionsPanel
                                    sources
                                    forms
                                    rendered_forms
                                    composition_mode
                                    join_left_key
                                    join_right_key
                                />
                            }.into_any(),
                            DatasetDesignerSelection::Source(index) => view! {
                                <SourceOptionsPanel
                                    index
                                    sources
                                    forms
                                    datasets
                                    rendered_forms
                                    fields
                                    composition_mode
                                />
                            }.into_any(),
                            DatasetDesignerSelection::Field(index) => view! {
                                <FieldOptionsPanel
                                    index
                                    fields
                                    sources
                                    forms
                                    rendered_forms
                                />
                            }.into_any(),
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
/// Renders the source options panel view.
fn SourceOptionsPanel(
    index: usize,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    composition_mode: RwSignal<String>,
) -> impl IntoView {
    view! {
        {move || sources.get().get(index).cloned().map(|source| {
            view! {
                <div class="dataset-options-sheet__content">
                    <header class="dataset-options-sheet__header">
                        <span>"Source"</span>
                        <h4>{source.source_alias.clone()}</h4>
                    </header>
                    <div class="dataset-options-sheet__stack">
                        <label class="form-field">
                            <span>"Alias"</span>
                            <input prop:value=source.source_alias.clone() on:input=move |event| {
                                let value = event_target_value(&event);
                                sources.update(|items| if let Some(item) = items.get_mut(index) { item.source_alias = value; });
                            }/>
                        </label>
                        <label class="form-field">
                            <span>"Input Type"</span>
                            <select prop:value=source.input_kind.clone() on:change=move |event| {
                                let value = event_target_value(&event);
                                sources.update(|items| {
                                    if let Some(item) = items.get_mut(index) {
                                        item.input_kind = value.clone();
                                    }
                                });
                            }>
                                <option value="form">"Form"</option>
                                <option value="dataset">"Dataset"</option>
                            </select>
                        </label>
                        {if source.input_kind == "dataset" {
                            view! {
                                <label class="form-field">
                                    <span>"Dataset"</span>
                                    <select prop:value=source.dataset_id.clone() on:change=move |event| {
                                        let dataset_id = event_target_value(&event);
                                        let revision_id = datasets
                                            .get()
                                            .into_iter()
                                            .find(|dataset| dataset.id == dataset_id)
                                            .and_then(|dataset| dataset.current_revision_id)
                                            .unwrap_or_default();
                                        sources.update(|items| {
                                            if let Some(item) = items.get_mut(index) {
                                                item.dataset_id = dataset_id.clone();
                                                item.dataset_revision_id = revision_id.clone();
                                            }
                                        });
                                    }>
                                        <option value="">"Select dataset"</option>
                                        {datasets.get().into_iter().filter(|dataset| dataset.current_revision_id.is_some()).map(|dataset| {
                                            view! { <option value=dataset.id>{dataset.name}</option> }
                                        }).collect_view()}
                                    </select>
                                </label>
                                <label class="form-field">
                                    <span>"Revision"</span>
                                    <input readonly prop:value=source.dataset_revision_id.clone()/>
                                </label>
                            }.into_any()
                        } else {
                            view! {
                                <label class="form-field">
                                    <span>"Form"</span>
                                    <select prop:value=source.form_id.clone() on:change=move |event| {
                                        let form_id = event_target_value(&event);
                                        sources.update(|items| {
                                            if let Some(item) = items.get_mut(index) {
                                                item.form_id = form_id.clone();
                                                if let Some(version) = first_published_version(&forms.get(), &form_id) {
                                                    item.form_version_id = version.id.clone();
                                                    item.form_version_major = version.version_major;
                                                    load_rendered_form(version.id.clone(), rendered_forms);
                                                }
                                            }
                                        });
                                    }>
                                        <option value="">"Select form"</option>
                                        {forms.get().into_iter().map(|form| view! { <option value=form.id>{form.name}</option> }).collect_view()}
                                    </select>
                                </label>
                                <label class="form-field">
                                    <span>"Version"</span>
                                    <select prop:value=source.form_version_id.clone() on:change=move |event| {
                                        let version_id = event_target_value(&event);
                                        sources.update(|items| {
                                            if let Some(item) = items.get_mut(index) {
                                                item.form_version_id = version_id.clone();
                                                item.form_version_major = find_version(&forms.get(), &version_id).and_then(|version| version.version_major);
                                                load_rendered_form(version_id.clone(), rendered_forms);
                                            }
                                        });
                                    }>
                                        {published_versions_for_form(&forms.get(), &source.form_id).into_iter().map(|version| {
                                            view! { <option value=version.id>{version_label(&version)}</option> }
                                        }).collect_view()}
                                    </select>
                                </label>
                                <label class="form-field">
                                    <span>"Selection"</span>
                                    <select prop:value=source.selection_rule.clone() on:change=move |event| {
                                        let value = event_target_value(&event);
                                        sources.update(|items| if let Some(item) = items.get_mut(index) { item.selection_rule = value; });
                                    }>
                                        <option value="latest">"Latest"</option>
                                        <option value="earliest">"Earliest"</option>
                                        {move || if composition_mode.get() == "union" {
                                            view! { <option value="all">"All"</option> }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </select>
                                </label>
                                <button class="button button--secondary" type="button" on:click=move |_| add_fields_from_source(index, sources, forms, rendered_forms, fields)>"Add Fields From Source"</button>
                            }.into_any()
                        }}
                    </div>
                </div>
            }.into_any()
        }).unwrap_or_else(|| view! {
            <div class="dataset-options-sheet__content">
                <header class="dataset-options-sheet__header">
                    <span>"Source"</span>
                    <h4>"No Source Selected"</h4>
                </header>
            </div>
        }.into_any())}
    }
}
