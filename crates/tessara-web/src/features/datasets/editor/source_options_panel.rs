//! Dataset editor source option panel.

use super::super::loaders::load_rendered_form;
use super::super::types::*;
use super::helpers::version_label;
use super::source_options::{first_published_version, published_versions_for_form};
use leptos::prelude::*;
use std::collections::BTreeMap;

#[allow(clippy::too_many_arguments)]
#[component]
pub(crate) fn SourceOptionsFields(
    source_signal: Signal<DatasetSourceDraft>,
    on_source_change: Callback<DatasetSourceDraft>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
) -> impl IntoView {
    view! {
        {move || {
            let source = source_signal.get();
            view! {
                <div class="dataset-options-sheet__content">
                    <div class="dataset-options-sheet__stack">
                        <label class="form-field">
                            <span>"Alias"</span>
                            <input prop:value=source.source_alias.clone() on:change=move |event| {
                                let value = event_target_value(&event).trim().to_string();
                                if value.is_empty() {
                                    return;
                                }
                                let mut next_source = source_signal.get();
                                next_source.source_alias = value;
                                on_source_change.run(next_source);
                            }/>
                        </label>
                        <label class="form-field">
                            <span>"Input Type"</span>
                            <select prop:value=source.input_kind.clone() on:change=move |event| {
                                let mut next_source = source_signal.get();
                                next_source.input_kind = event_target_value(&event);
                                on_source_change.run(next_source);
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
                                        let mut next_source = source_signal.get();
                                        next_source.dataset_id = dataset_id;
                                        next_source.dataset_revision_id = revision_id;
                                        on_source_change.run(next_source);
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
                                        let mut next_source = source_signal.get();
                                        next_source.form_id = form_id.clone();
                                        if let Some(version) = first_published_version(&forms.get(), &form_id) {
                                            next_source.form_version_id = version.id.clone();
                                            load_rendered_form(version.id.clone(), rendered_forms);
                                        }
                                        on_source_change.run(next_source);
                                    }>
                                        <option value="">"Select form"</option>
                                        {forms.get().into_iter().map(|form| view! { <option value=form.id>{form.name}</option> }).collect_view()}
                                    </select>
                                </label>
                                <label class="form-field">
                                    <span>"Version"</span>
                                    <select prop:value=source.form_version_id.clone() on:change=move |event| {
                                        let version_id = event_target_value(&event);
                                        let mut next_source = source_signal.get();
                                        next_source.form_version_id = version_id.clone();
                                        load_rendered_form(version_id.clone(), rendered_forms);
                                        on_source_change.run(next_source);
                                    }>
                                        {published_versions_for_form(&forms.get(), &source.form_id).into_iter().map(|version| {
                                            view! { <option value=version.id>{version_label(&version)}</option> }
                                        }).collect_view()}
                                    </select>
                                </label>
                            }.into_any()
                        }}
                    </div>
                </div>
            }
        }}
    }
}
