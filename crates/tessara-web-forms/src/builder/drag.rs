//! Form adapter for committing shared placement drag intent.

use crate::builder::FormBuilderFieldDraft;
use crate::builder::{
    FormBuilderDragPreview, clear_form_builder_drag_intent, form_builder_reflow_section_fields,
};
use leptos::prelude::*;

pub(crate) fn commit_form_builder_drag_preview(
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) {
    let preview = builder_drag_preview.get_untracked();

    if let Some(preview) = preview {
        builder_fields.update(|fields| {
            *fields = form_builder_reflow_section_fields(fields, preview);
        });
        suppress_builder_field_click.set(Some(preview.placement_id));
    }

    clear_form_builder_drag_intent(
        builder_drag_preview,
        pending_builder_drag_preview,
        builder_drag_preview_timeout,
    );
    dragged_builder_field.set(None);
}
