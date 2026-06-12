//! Resize handles for form builder field tiles.

use crate::features::forms::builder::{
    FormBuilderFieldDraft, FormBuilderResizeAxis, start_form_builder_field_resize,
};
use leptos::prelude::*;

#[component]
pub(super) fn FormBuilderFieldResizeHandles(
    field_id: usize,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        <span
            class="form-builder-resize-handle form-builder-resize-handle--width"
            title="Resize field width"
            aria-hidden="true"
            on:mousedown=move |event| {
                start_form_builder_field_resize(
                    event,
                    FormBuilderResizeAxis::Width,
                    field_id,
                    builder_fields,
                    suppress_builder_field_click,
                );
            }
        ></span>
        <span
            class="form-builder-resize-handle form-builder-resize-handle--height"
            title="Resize field height"
            aria-hidden="true"
            on:mousedown=move |event| {
                start_form_builder_field_resize(
                    event,
                    FormBuilderResizeAxis::Height,
                    field_id,
                    builder_fields,
                    suppress_builder_field_click,
                );
            }
        ></span>
    }
}
