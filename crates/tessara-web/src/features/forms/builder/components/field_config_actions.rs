//! Signal-aware actions for form builder field configuration.

use crate::features::forms::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderFieldDraft, form_builder_field_default_label,
    form_builder_field_has_collision, form_builder_layout_candidate,
};
use crate::utils::slug::slug_from_label;
use leptos::prelude::*;

pub(in crate::features::forms::builder::components) fn update_field_label(
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    field_id: usize,
    next_label: String,
) {
    builder_fields.update(|fields| {
        if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
            field.label = next_label.clone();
            if !field.key_was_edited {
                field.key = slug_from_label(&next_label);
            }
        }
    });
}

pub(in crate::features::forms::builder::components) fn update_field_key(
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    field_id: usize,
    next_key: String,
) {
    builder_fields.update(|fields| {
        if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
            field.key = slug_from_label(&next_key);
            field.key_was_edited = true;
        }
    });
}

pub(in crate::features::forms::builder::components) fn update_field_type(
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    field_id: usize,
    next_type: String,
) {
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

pub(in crate::features::forms::builder::components) fn update_field_required(
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    field_id: usize,
    checked: bool,
) {
    builder_fields.update(|fields| {
        if let Some(field) = fields.iter_mut().find(|field| field.id == field_id)
            && field.field_type != "static_text"
        {
            field.required = checked;
        }
    });
}

pub(in crate::features::forms::builder::components) fn update_field_layout_value(
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    field_id: usize,
    index: usize,
    value: i32,
) {
    builder_fields.update(|fields| {
        if let Some(position) = fields.iter().position(|field| field.id == field_id) {
            let candidate = form_builder_layout_candidate(&fields[position], index, value);

            if !form_builder_field_has_collision(&candidate, fields) {
                fields[position] = candidate;
            }
        }
    });
}
