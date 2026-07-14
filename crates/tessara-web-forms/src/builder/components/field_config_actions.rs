//! Signal-aware actions for form builder field configuration.

use crate::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderFieldDraft, form_builder_field_default_label,
    form_builder_field_has_collision, form_builder_layout_candidate,
};
use crate::support::slug::slug_from_label;
use leptos::prelude::*;

pub(crate) fn update_field_label(
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

pub(crate) fn update_field_key(
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

pub(crate) fn update_field_type(
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

pub(crate) fn update_field_required(
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

pub(crate) fn update_field_layout_value(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::blank_form_builder_field_at;

    fn field(id: usize, column: i32) -> FormBuilderFieldDraft {
        blank_form_builder_field_at(id, 1, 1, column, 6)
    }

    #[test]
    fn label_configuration_still_generates_a_key_until_manually_edited() {
        Owner::new().with(|| {
            let fields = RwSignal::new(vec![field(1, 1)]);
            update_field_label(fields, 1, "Customer Name".into());
            assert_eq!(fields.get()[0].key, "customer-name");

            update_field_key(fields, 1, "account owner".into());
            update_field_label(fields, 1, "Primary Contact".into());
            let configured = fields.get();
            assert_eq!(configured[0].label, "Primary Contact");
            assert_eq!(configured[0].key, "account-owner");
        });
    }

    #[test]
    fn direct_form_layout_edits_still_reject_collision_without_reflow() {
        Owner::new().with(|| {
            let fields = RwSignal::new(vec![field(1, 1), field(2, 7)]);

            update_field_layout_value(fields, 2, 1, 1);
            assert_eq!(fields.get()[1].grid_column, 7);

            update_field_layout_value(fields, 2, 0, 2);
            assert_eq!(fields.get()[1].grid_row, 2);
            assert_eq!(fields.get()[1].grid_column, 7);
        });
    }
}
