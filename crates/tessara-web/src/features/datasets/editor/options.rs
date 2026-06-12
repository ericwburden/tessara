//! Dataset editor designer option sheet components.

use super::super::types::*;
use super::{FieldOptionsPanel, OperationOptionsPanel, SourceOptionsPanel};
use icons::X;
use leptos::portal::Portal;
use leptos::prelude::*;
use std::collections::BTreeMap;
#[component]
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
