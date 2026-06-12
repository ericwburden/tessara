//! Dataset editor projected field options panel.

use super::super::types::*;
use super::helpers::join_key_option_label;
use super::source_options::source_field_options_with_selected;
use leptos::prelude::*;
use std::collections::BTreeMap;

#[component]
pub(crate) fn FieldOptionsPanel(
    index: usize,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
) -> impl IntoView {
    view! {
        {move || fields.get().get(index).cloned().map(|field| {
            view! {
                <div class="dataset-options-sheet__content">
                    <header class="dataset-options-sheet__header">
                        <span>"Projected Field"</span>
                        <h4>{field.label.clone()}</h4>
                    </header>
                    <div class="dataset-options-sheet__stack">
                        <label class="form-field">
                            <span>"Key"</span>
                            <input prop:value=field.key.clone() on:input=move |event| {
                                let value = event_target_value(&event);
                                fields.update(|items| if let Some(item) = items.get_mut(index) { item.key = value; });
                            }/>
                        </label>
                        <label class="form-field">
                            <span>"Label"</span>
                            <input prop:value=field.label.clone() on:input=move |event| {
                                let value = event_target_value(&event);
                                fields.update(|items| if let Some(item) = items.get_mut(index) { item.label = value; });
                            }/>
                        </label>
                        <label class="form-field">
                            <span>"Source"</span>
                            <select prop:value=field.source_alias.clone() on:change=move |event| {
                                let value = event_target_value(&event);
                                fields.update(|items| if let Some(item) = items.get_mut(index) { item.source_alias = value; });
                            }>
                                {sources.get().into_iter().map(|source| view! { <option value=source.source_alias.clone()>{source.source_alias.clone()}</option> }).collect_view()}
                            </select>
                        </label>
                        <label class="form-field">
                            <span>"Source Field"</span>
                            <select prop:value=field.source_field_key.clone() on:change=move |event| {
                                let value = event_target_value(&event);
                                fields.update(|items| if let Some(item) = items.get_mut(index) { item.source_field_key = value; });
                            }>
                                {source_field_options_with_selected(
                                    &sources.get(),
                                    &forms.get(),
                                    &rendered_forms.get(),
                                    &field.source_alias,
                                    &field.source_field_key,
                                ).into_iter().map(|option| {
                                    view! { <option value=option.key.clone()>{join_key_option_label(&option)}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                    </div>
                </div>
            }.into_any()
        }).unwrap_or_else(|| view! {
            <div class="dataset-options-sheet__content">
                <header class="dataset-options-sheet__header">
                    <span>"Projected Field"</span>
                    <h4>"No Field Selected"</h4>
                </header>
            </div>
        }.into_any())}
    }
}
