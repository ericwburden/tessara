//! Resize handles for form builder field tiles.

use crate::builder::{FormBuilderFieldDraft, start_form_builder_field_resize};
use leptos::prelude::*;
use tessara_web_ui::placement_editor::{PlacementResizeHandles, PlacementResizeStart};

#[component]
pub(super) fn FormBuilderFieldResizeHandles(
    field_id: usize,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        <PlacementResizeHandles
            on_resize_start=Callback::new(move |request: PlacementResizeStart| {
                start_form_builder_field_resize(
                    request.event,
                    request.axis,
                    field_id,
                    builder_fields,
                    suppress_builder_field_click,
                );
            })
            width_title="Resize field width"
            height_title="Resize field height"
            handle_class="form-builder-resize-handle"
            width_class="form-builder-resize-handle--width"
            height_class="form-builder-resize-handle--height"
        />
    }
}
