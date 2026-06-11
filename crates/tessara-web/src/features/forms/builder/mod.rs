//! Form builder module boundary.
//!
//! Re-export the builder canvas, state, drag, layout, validation, display, and type helpers needed by forms and organization form workflows.

mod components;
mod display;
mod drag;
mod hydrate;
mod layout;
mod state;
mod types;
mod validation;

pub(crate) use components::FormBuilderCanvas;
pub(crate) use display::{form_builder_field_default_label, form_builder_field_type_icon};
pub(crate) use drag::{
    clear_form_builder_drag_intent, commit_form_builder_drag_preview,
    form_builder_add_tile_from_click_event, form_builder_grid_cell_from_drag_event,
    form_builder_grid_cell_from_pointer, schedule_form_builder_drag_preview,
    set_form_builder_drag_preview,
};
#[cfg(feature = "hydrate")]
pub(crate) use hydrate::hydrate_form_builder_from_rendered;
pub(crate) use layout::{
    FormBuilderGridCell, FormBuilderSectionLayout, blank_form_builder_field_at,
    form_builder_field_has_collision, form_builder_layout_candidate, form_builder_occupancy_map,
    form_builder_reflow_section_fields, form_builder_section_fields, form_builder_section_layout,
    max_form_builder_field_height, max_form_builder_field_width,
    max_form_builder_new_field_width_at, start_form_builder_field_resize,
    valid_form_builder_layout_values,
};
pub(crate) use state::{FormBuilderEditorState, new_form_builder_editor_state};
pub(crate) use types::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderDragPreview, FormBuilderFieldDraft,
    FormBuilderResizeAxis, FormBuilderSectionDraft, blank_form_builder_section,
};
#[cfg(feature = "hydrate")]
pub(crate) use validation::{prepared_form_builder_fields, prepared_form_builder_sections};
