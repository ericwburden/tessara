//! Form builder field configuration sheet.
//!
//! Keep side-panel controls for editing field labels, types, validation, and layout here.

use super::field_config_controls::FieldConfigControls;
use leptos::prelude::*;

use crate::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft, blank_form_builder_section};
use crate::builder::{
    form_builder_section_layout, max_form_builder_field_height, max_form_builder_field_width,
};
use icons::Trash2;
use tessara_module_ui::{SideSheet, empty_view};

#[component]
pub(crate) fn FieldConfigSheet(
    active_builder_field: RwSignal<Option<usize>>,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
) -> impl IntoView {
    view! {
        <Show when=move || active_builder_field.get().is_some()>
            {move || {
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
                            <SideSheet
                                id=format!("form-field-config-{field_id}")
                                title=display_label
                                eyebrow="Field Configuration"
                                open=Signal::derive(move || active_builder_field.get() == Some(field_id))
                                on_close=Callback::new(move |_| active_builder_field.set(None))
                                close_label="Close field configuration"
                                class="form-field-config-sheet"
                                header_actions=move || view! {
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
                                }
                            >
                                <section class="sheet-panel__section">
                                    <FieldConfigControls
                                        field=field.clone()
                                        field_id
                                        builder_fields
                                        section_column_count
                                        section_fields_for_bounds=section_fields_for_bounds.clone()
                                        row_max
                                        width_max
                                        height_max
                                    />
                                </section>
                            </SideSheet>
                        }
                        .into_any()
                    })
                    .unwrap_or_else(empty_view)
            }}
        </Show>
    }
}
