//! Form builder field configuration sheet.
//!
//! Keep side-panel controls for editing field labels, types, validation, and layout here.

use super::field_config_controls::FieldConfigControls;
use leptos::portal::Portal;
use leptos::prelude::*;

use crate::features::forms::builder::{
    FormBuilderFieldDraft, FormBuilderSectionDraft, blank_form_builder_section,
};
use crate::features::forms::builder::{
    form_builder_section_layout, max_form_builder_field_height, max_form_builder_field_width,
};
use crate::ui::empty_view;
use icons::{Trash2, X};

#[component]
/// Renders the field config sheet view.
pub(crate) fn FieldConfigSheet(
    active_builder_field: RwSignal<Option<usize>>,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
) -> impl IntoView {
    view! {
        <Portal>
            <Show when=move || active_builder_field.get().is_some()>
                {move || {
                    let close = move |_| active_builder_field.set(None);
                    let field_id = active_builder_field.get().unwrap_or_default();
                    let field = builder_fields
                        .get()
                        .into_iter()
                        .find(|field| field.id == field_id);
                    field
                        .map(|field| {
                            let display_label = if field.label.trim().is_empty() {
                                format!("Field {}", field.id)
                            } else {
                                field.label.clone()
                            };
                            let section = builder_sections
                                .get()
                                .into_iter()
                                .find(|section| section.id == field.section_id)
                                .unwrap_or_else(|| blank_form_builder_section(field.section_id));
                            let all_fields = builder_fields.get();
                            let layout = form_builder_section_layout(&section, &all_fields);
                            let section_column_count = layout.column_count;
                            let section_fields_for_bounds = layout.fields;
                            let row_max = layout.row_count;
                            let width_max = max_form_builder_field_width(
                                &field,
                                &section_fields_for_bounds,
                            );
                            let height_max = max_form_builder_field_height(
                                &field,
                                &section_fields_for_bounds,
                            );
                            view! {
                                <section class="sheet-overlay form-field-config-overlay" aria-label="Field configuration">
                                    <button class="sheet-overlay__scrim" type="button" aria-label="Close field configuration" on:click=close></button>
                                    <aside class="sheet-panel blurred-surface form-field-config-sheet" role="dialog" aria-modal="true" aria-label="Field configuration">
                                        <div class="sheet-panel__actions">
                                            <button
                                                class="icon-button icon-button--danger"
                                                type="button"
                                                aria-label="Delete field"
                                                title="Delete field"
                                                on:click=move |_| {
                                                    builder_fields.update(|fields| {
                                                        fields.retain(|field| field.id != field_id);
                                                    });
                                                    active_builder_field.set(None);
                                                }
                                            >
                                                <Trash2/>
                                            </button>
                                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close field configuration" title="Close field configuration" on:click=close>
                                                <X/>
                                            </button>
                                        </div>

                                        <header class="sheet-panel__header">
                                            <p>"Field Configuration"</p>
                                            <h2>{display_label}</h2>
                                        </header>

                                        <section class="sheet-panel__section">
                                            <FieldConfigControls
                                                field
                                                field_id
                                                builder_fields
                                                section_column_count
                                                section_fields_for_bounds
                                                row_max
                                                width_max
                                                height_max
                                            />
                                        </section>
                                    </aside>
                                </section>
                            }
                            .into_any()
                        })
                        .unwrap_or_else(empty_view)
                }}
            </Show>
        </Portal>
    }
}
