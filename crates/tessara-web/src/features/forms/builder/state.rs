//! Owns the features::forms::builder::state module behavior.

use crate::features::forms::builder::types::FormBuilderDragPreview;
use crate::features::forms::builder::{
    FormBuilderFieldDraft, FormBuilderSectionDraft, blank_form_builder_section,
};
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct FormBuilderEditorState {
    pub(crate) builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    pub(crate) active_builder_section: RwSignal<String>,
    pub(crate) next_builder_section_id: RwSignal<usize>,
    pub(crate) builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    pub(crate) active_builder_field: RwSignal<Option<usize>>,
    pub(crate) dragged_builder_field: RwSignal<Option<usize>>,
    pub(crate) builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pub(crate) pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pub(crate) builder_drag_preview_timeout: RwSignal<Option<i32>>,
    pub(crate) suppress_builder_field_click: RwSignal<Option<usize>>,
    pub(crate) next_builder_field_id: RwSignal<usize>,
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the new form builder editor state behavior.
pub(crate) fn new_form_builder_editor_state() -> FormBuilderEditorState {
    FormBuilderEditorState {
        builder_sections: RwSignal::new(vec![blank_form_builder_section(1)]),
        active_builder_section: RwSignal::new("1".to_string()),
        next_builder_section_id: RwSignal::new(2usize),
        builder_fields: RwSignal::new(Vec::<FormBuilderFieldDraft>::new()),
        active_builder_field: RwSignal::new(None::<usize>),
        dragged_builder_field: RwSignal::new(None::<usize>),
        builder_drag_preview: RwSignal::new(None::<FormBuilderDragPreview>),
        pending_builder_drag_preview: RwSignal::new(None::<FormBuilderDragPreview>),
        builder_drag_preview_timeout: RwSignal::new(None::<i32>),
        suppress_builder_field_click: RwSignal::new(None::<usize>),
        next_builder_field_id: RwSignal::new(1usize),
    }
}

/// Handles the add form builder section to editor behavior.
pub(crate) fn add_form_builder_section_to_editor(
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    next_builder_section_id: RwSignal<usize>,
    active_builder_section: RwSignal<String>,
) {
    let section_id = next_builder_section_id.get_untracked();
    next_builder_section_id.set(section_id + 1);
    builder_sections.update(|sections| {
        let mut section = blank_form_builder_section(section_id);
        section.position = (sections.len() + 1) as i32;
        sections.push(section);
    });
    active_builder_section.set(section_id.to_string());
}
