//! Owns the features::forms::builder::components::field_config_sheet module behavior.

use leptos::portal::Portal;
use leptos::prelude::*;

use crate::features::forms::builder::{
    FORM_BUILDER_COLUMN_COUNT, form_builder_field_default_label, form_builder_field_has_collision,
    form_builder_layout_candidate, form_builder_section_layout, max_form_builder_field_height,
    max_form_builder_field_width, valid_form_builder_layout_values,
};
use crate::features::forms::builder::{
    FormBuilderFieldDraft, FormBuilderSectionDraft, blank_form_builder_section,
};
use crate::features::shared::slug_from_label;
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
                                            <div class="form-grid form-builder-field-sheet-controls">
                                                <label class="form-field" for=format!("sheet-form-field-label-{field_id}")>
                                                    <span>"Field Label"</span>
                                                    <input
                                                        id=format!("sheet-form-field-label-{field_id}")
                                                        type="text"
                                                        autocomplete="off"
                                                        prop:value=field.label.clone()
                                                        on:input=move |event| {
                                                            let next_label = event_target_value(&event);
                                                            builder_fields.update(|fields| {
                                                                if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
                                                                    field.label = next_label.clone();
                                                                    if !field.key_was_edited {
                                                                        field.key = slug_from_label(&next_label);
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    />
                                                </label>

                                                <label class="form-field" for=format!("sheet-form-field-key-{field_id}")>
                                                    <span>"Field Key"</span>
                                                    <input
                                                        id=format!("sheet-form-field-key-{field_id}")
                                                        type="text"
                                                        autocomplete="off"
                                                        prop:value=field.key.clone()
                                                        on:input=move |event| {
                                                            let next_key = slug_from_label(&event_target_value(&event));
                                                            builder_fields.update(|fields| {
                                                                if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
                                                                    field.key = next_key.clone();
                                                                    field.key_was_edited = true;
                                                                }
                                                            });
                                                        }
                                                    />
                                                </label>

                                                <label class="form-field" for=format!("sheet-form-field-type-{field_id}")>
                                                    <span>"Field Type"</span>
                                                    <select
                                                        id=format!("sheet-form-field-type-{field_id}")
                                                        prop:value=field.field_type.clone()
                                                        on:change=move |event| {
                                                            let next_type = event_target_value(&event);
                                                            builder_fields.update(|fields| {
                                                                if let Some(position) = fields.iter().position(|field| field.id == field_id) {
                                                                    let mut next_field = fields[position].clone();
                                                                    next_field.field_type = next_type.clone();
                                                                    if next_type == "static_text" {
                                                                        next_field.required = false;
                                                                        if next_field.label.trim().is_empty() {
                                                                            next_field.label = form_builder_field_default_label(&next_type, next_field.id);
                                                                        }
                                                                        if next_field.key.trim().is_empty() || !next_field.key_was_edited {
                                                                            next_field.key = slug_from_label(&next_field.label);
                                                                        }
                                                                        let mut candidate = next_field.clone();
                                                                        candidate.grid_width = candidate.grid_width.max(4);
                                                                        if candidate.grid_column + candidate.grid_width - 1 <= FORM_BUILDER_COLUMN_COUNT
                                                                            && !form_builder_field_has_collision(&candidate, fields)
                                                                        {
                                                                            next_field.grid_width = candidate.grid_width;
                                                                        }
                                                                    }
                                                                    fields[position] = next_field;
                                                                }
                                                            });
                                                        }
                                                    >
                                                        <option value="static_text">"Static text"</option>
                                                        <option value="text">"Text"</option>
                                                        <option value="number">"Number"</option>
                                                        <option value="date">"Date"</option>
                                                        <option value="boolean">"Checkbox"</option>
                                                        <option value="single_choice">"Single choice"</option>
                                                        <option value="multi_choice">"Multi choice"</option>
                                                    </select>
                                                </label>

                                                <label class="form-field form-field--checkbox form-builder-field__required">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=field.required
                                                        disabled=field.field_type == "static_text"
                                                        on:change=move |event| {
                                                            let checked = event_target_checked(&event);
                                                            builder_fields.update(|fields| {
                                                                if let Some(field) = fields.iter_mut().find(|field| field.id == field_id)
                                                                    && field.field_type != "static_text" {
                                                                        field.required = checked;
                                                                    }
                                                            });
                                                        }
                                                    />
                                                    <span>"Required"</span>
                                                </label>

                                                {["Row", "Column", "Width", "Height"]
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(index, label)| {
                                                        let value = match index {
                                                            0 => field.grid_row,
                                                            1 => field.grid_column,
                                                            2 => field.grid_width,
                                                            _ => field.grid_height,
                                                        };
                                                        let max_value = match index {
                                                            0 => row_max,
                                                            1 => (section_column_count - field.grid_width.max(1) + 1)
                                                                .clamp(1, section_column_count.max(1)),
                                                            2 => width_max,
                                                            _ => height_max,
                                                        }
                                                        .max(1);
                                                        let value = value.clamp(1, max_value);
                                                        let valid_values = valid_form_builder_layout_values(
                                                            &field,
                                                            &section_fields_for_bounds,
                                                            index,
                                                            max_value,
                                                        );
                                                        let control_id = format!("sheet-form-field-layout-{index}-{field_id}");
                                                        let input_id = control_id.clone();
                                                        view! {
                                                            <label class="form-field" for=control_id>
                                                                <span>{label}</span>
                                                                <select
                                                                    id=input_id
                                                                    on:change=move |event| {
                                                                        let value = event_target_value(&event)
                                                                            .parse::<i32>()
                                                                            .unwrap_or(1)
                                                                            .clamp(1, max_value);
                                                                        builder_fields.update(|fields| {
                                                                            if let Some(position) = fields.iter().position(|field| field.id == field_id) {
                                                                                let candidate = form_builder_layout_candidate(
                                                                                    &fields[position],
                                                                                    index,
                                                                                    value,
                                                                                );

                                                                                if !form_builder_field_has_collision(&candidate, fields) {
                                                                                    fields[position] = candidate;
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                >
                                                                    {valid_values
                                                                        .into_iter()
                                                                        .map(|option_value| {
                                                                            view! {
                                                                                <option
                                                                                    value=option_value.to_string()
                                                                                    selected=option_value == value
                                                                                >
                                                                                    {option_value}
                                                                                </option>
                                                                            }
                                                                        })
                                                                        .collect_view()}
                                                                </select>
                                                            </label>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </div>
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
