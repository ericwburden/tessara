//! Field configuration controls for the form builder sheet.

use leptos::prelude::*;

use crate::features::forms::builder::components::field_config_actions::{
    update_field_key, update_field_label, update_field_layout_value, update_field_required,
    update_field_type,
};
use crate::features::forms::builder::{FormBuilderFieldDraft, valid_form_builder_layout_values};

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
                        update_field_label(builder_fields, field_id, event_target_value(&event));
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
                        update_field_key(builder_fields, field_id, event_target_value(&event));
                    }
                />
            </label>

            <label class="form-field" for=format!("sheet-form-field-type-{field_id}")>
                <span>"Field Type"</span>
                <select
                    id=format!("sheet-form-field-type-{field_id}")
                    prop:value=field.field_type.clone()
                    on:change=move |event| {
                        update_field_type(builder_fields, field_id, event_target_value(&event));
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
                        update_field_required(builder_fields, field_id, event_target_checked(&event));
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
                                    update_field_layout_value(builder_fields, field_id, index, value);
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
