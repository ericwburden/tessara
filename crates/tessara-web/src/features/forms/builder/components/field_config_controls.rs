//! Field configuration controls for the form builder sheet.

use leptos::prelude::*;

use crate::features::forms::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderFieldDraft, form_builder_field_default_label,
    form_builder_field_has_collision, form_builder_layout_candidate,
    valid_form_builder_layout_values,
};
use crate::features::shared::slug_from_label;

#[component]
pub(crate) fn FieldConfigControls(
    field: FormBuilderFieldDraft,
    field_id: usize,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    section_column_count: i32,
    section_fields_for_bounds: Vec<FormBuilderFieldDraft>,
    row_max: i32,
    width_max: i32,
    height_max: i32,
) -> impl IntoView {
    view! {
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
    }
}
