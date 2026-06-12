//! Response form field input component.

use crate::features::forms::RenderedField;
use crate::features::responses::display::{rendered_form_field_layout_style, response_field_class};
use crate::ui::empty_view;
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
/// Renders the response field input view.
pub(crate) fn ResponseFieldInput(
    field: RenderedField,
    text_values: RwSignal<HashMap<String, String>>,
    boolean_values: RwSignal<HashMap<String, bool>>,
) -> impl IntoView {
    let field_key = field.key.clone();
    let field_key_for_input = field.key.clone();
    let field_key_for_bool = field.key.clone();
    let input_id = format!("response-field-{}", field.id);
    let required_label = if field.required { " *" } else { "" };
    let layout_style = rendered_form_field_layout_style(&field);
    let field_height = field.grid_height;
    let field_class = response_field_class(&field.field_type);

    view! {
        <div class=field_class style=layout_style>
            {if field.field_type == "static_text" {
                empty_view()
            } else {
                view! { <span>{format!("{}{}", field.label, required_label)}</span> }.into_any()
            }}
            {if field.field_type == "static_text" {
                view! {
                    <p class="response-form-field__static-text">{field.label.clone()}</p>
                }
                .into_any()
            } else if field.field_type == "boolean" {
                let input_id_for_label = input_id.clone();
                view! {
                    <label class="form-field--checkbox" for=input_id_for_label>
                        <input
                            id=input_id
                            type="checkbox"
                            prop:checked=move || {
                                boolean_values
                                    .get()
                                    .get(&field_key_for_bool)
                                    .copied()
                                    .unwrap_or(false)
                            }
                            on:change=move |event| {
                                let checked = event_target_checked(&event);
                                boolean_values.update(|values| {
                                    values.insert(field_key.clone(), checked);
                                });
                            }
                        />
                        <span>"Yes"</span>
                    </label>
                }
                .into_any()
            } else {
                let input_type = if field.field_type == "number" {
                    "number"
                } else if field.field_type == "date" {
                    "date"
                } else {
                    "text"
                };
                if input_type == "text" && field_height > 1 {
                    view! {
                        <textarea
                            id=input_id
                            required=field.required
                            prop:value=move || {
                                text_values
                                    .get()
                                    .get(&field_key_for_input)
                                    .cloned()
                                    .unwrap_or_default()
                            }
                            on:input=move |event| {
                                let value = event_target_value(&event);
                                text_values.update(|values| {
                                    values.insert(field.key.clone(), value);
                                });
                            }
                        ></textarea>
                    }
                    .into_any()
                } else {
                    view! {
                        <input
                            id=input_id
                            type=input_type
                            required=field.required
                            prop:value=move || {
                                text_values
                                    .get()
                                    .get(&field_key_for_input)
                                    .cloned()
                                    .unwrap_or_default()
                            }
                            on:input=move |event| {
                                let value = event_target_value(&event);
                                text_values.update(|values| {
                                    values.insert(field.key.clone(), value);
                                });
                            }
                        />
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
