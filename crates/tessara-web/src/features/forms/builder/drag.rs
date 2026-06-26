//! Form builder drag interaction state.
//!
//! Keep drag previews, drop intent, and pointer-driven field movement helpers here.

#[cfg(feature = "hydrate")]
use super::drag_dom::{clear_form_builder_drag_target_dom, set_form_builder_drag_target_dom};
use crate::features::forms::builder::FormBuilderFieldDraft;
use crate::features::forms::builder::{FormBuilderDragPreview, form_builder_reflow_section_fields};
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;

pub(crate) fn set_form_builder_drag_preview(
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    next_preview: FormBuilderDragPreview,
) {
    if builder_drag_preview.get_untracked() != Some(next_preview) {
        builder_drag_preview.set(Some(next_preview));
    }
}

pub(crate) fn clear_form_builder_drag_intent(
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
) {
    pending_builder_drag_preview.set(None);
    builder_drag_preview.set(None);

    #[cfg(feature = "hydrate")]
    {
        if let (Some(window), Some(timeout_handle)) = (
            web_sys::window(),
            builder_drag_preview_timeout.get_untracked(),
        ) {
            window.clear_timeout_with_handle(timeout_handle);
        }
        clear_form_builder_drag_target_dom();
    }

    builder_drag_preview_timeout.set(None);
}

pub(crate) fn schedule_form_builder_drag_preview(
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    _builder_drag_preview_timeout: RwSignal<Option<i32>>,
    next_preview: FormBuilderDragPreview,
    target_id: String,
) {
    if builder_drag_preview.get_untracked() == Some(next_preview) {
        return;
    }

    pending_builder_drag_preview.set(Some(next_preview));

    #[cfg(feature = "hydrate")]
    {
        if let (Some(window), Some(timeout_handle)) = (
            web_sys::window(),
            _builder_drag_preview_timeout.get_untracked(),
        ) {
            window.clear_timeout_with_handle(timeout_handle);
        }

        let pending_preview = pending_builder_drag_preview;
        let preview_signal = builder_drag_preview;
        let timeout_signal = _builder_drag_preview_timeout;
        let callback = Closure::wrap(Box::new(move || {
            if pending_preview.get_untracked() == Some(next_preview) {
                set_form_builder_drag_preview(preview_signal, next_preview);
                set_form_builder_drag_target_dom(&target_id);
            }
            timeout_signal.set(None);
        }) as Box<dyn FnMut()>);

        if let Some(window) = web_sys::window()
            && let Ok(timeout_handle) = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    1_000,
                )
        {
            _builder_drag_preview_timeout.set(Some(timeout_handle));
            callback.forget();
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        set_form_builder_drag_preview(builder_drag_preview, next_preview);
        let _ = target_id;
    }
}

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
        suppress_builder_field_click.set(Some(preview.field_id));
    }

    clear_form_builder_drag_intent(
        builder_drag_preview,
        pending_builder_drag_preview,
        builder_drag_preview_timeout,
    );
    dragged_builder_field.set(None);
}
