//! Hydration helpers for seeding builder drafts from rendered forms.

use super::types::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderFieldDraft, FormBuilderSectionDraft,
    blank_form_builder_section,
};
use crate::features::forms::RenderedForm;
use crate::utils::text::nonempty_text;
use std::collections::HashMap;

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn hydrate_form_builder_from_rendered(
    rendered_form: Option<&RenderedForm>,
) -> (
    Vec<FormBuilderSectionDraft>,
    Vec<FormBuilderFieldDraft>,
    usize,
    usize,
) {
    let Some(rendered_form) = rendered_form else {
        return (vec![blank_form_builder_section(1)], Vec::new(), 2, 1);
    };

    let mut sections = rendered_form.sections.clone();
    sections.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then(left.title.cmp(&right.title))
    });

    if sections.is_empty() {
        return (vec![blank_form_builder_section(1)], Vec::new(), 2, 1);
    }

    let mut section_id_by_remote = HashMap::new();
    let mut builder_sections = Vec::new();
    let mut builder_fields = Vec::new();
    let mut next_section_id = 1usize;
    let mut next_field_id = 1usize;

    for section in &sections {
        let local_section_id = next_section_id;
        next_section_id += 1;
        section_id_by_remote.insert(section.id.clone(), local_section_id);

        builder_sections.push(FormBuilderSectionDraft {
            id: local_section_id,
            remote_id: Some(section.id.clone()),
            title: nonempty_text(Some(section.title.as_str()), "Main"),
            description: section.description.clone(),
            default_column_width: 6,
            position: section.position,
        });
    }

    for section in &sections {
        let Some(section_id) = section_id_by_remote.get(&section.id).copied() else {
            continue;
        };
        let mut fields = section.fields.clone();
        fields.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then(left.label.cmp(&right.label))
        });

        for field in fields {
            let local_field_id = next_field_id;
            next_field_id += 1;
            builder_fields.push(FormBuilderFieldDraft {
                id: local_field_id,
                remote_id: Some(field.id),
                section_id,
                label: field.label,
                key: field.key,
                field_type: field.field_type,
                required: field.required,
                grid_row: field.grid_row.max(1),
                grid_column: field.grid_column.clamp(1, FORM_BUILDER_COLUMN_COUNT),
                grid_width: field.grid_width.clamp(1, FORM_BUILDER_COLUMN_COUNT),
                grid_height: field.grid_height.clamp(1, 6),
                key_was_edited: true,
            });
        }
    }

    (
        builder_sections,
        builder_fields,
        next_section_id,
        next_field_id,
    )
}
