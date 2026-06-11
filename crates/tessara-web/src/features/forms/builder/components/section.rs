//! Owns the features::forms::builder::components::section module behavior.

use leptos::prelude::*;

use crate::features::forms::builder::components::grid::FormBuilderGrid;
use crate::features::forms::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderDragPreview, form_builder_section_layout,
};
use crate::features::forms::builder::{
    FormBuilderFieldDraft, FormBuilderSectionDraft, blank_form_builder_section,
};

#[component]
/// Renders the form builder section view.
pub(crate) fn FormBuilderSection(
    section_id: usize,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
    next_builder_field_id: RwSignal<usize>,
) -> impl IntoView {
    let section = Memo::new(move |_| {
        builder_sections
            .get()
            .into_iter()
            .find(|section| section.id == section_id)
            .unwrap_or_else(|| blank_form_builder_section(section_id))
    });
    let layout = Memo::new(move |_| {
        let section = section.get();
        let fields = builder_fields.get();
        form_builder_section_layout(&section, &fields)
    });
    let default_column_width = Memo::new(move |_| section.get().default_column_width);

    view! {
        <article class="form-builder-section-card">
            <div class="form-builder-section-card__header">
                <h4>{move || section.get().title}</h4>
            </div>

            <div class="form-grid form-builder-section-card__settings">
                <label class="form-field" for=format!("form-section-title-{section_id}")>
                    <span>"Section Title"</span>
                    <input
                        id=format!("form-section-title-{section_id}")
                        type="text"
                        autocomplete="off"
                        prop:value=move || section.get().title
                        on:input=move |event| {
                            let next_title = event_target_value(&event);
                            builder_sections.update(|sections| {
                                if let Some(section) = sections.iter_mut().find(|section| section.id == section_id) {
                                    section.title = next_title.clone();
                                }
                            });
                        }
                    />
                </label>

                <label class="form-field" for=format!("form-section-default-width-{section_id}")>
                    <span>"Default Column Width"</span>
                    <select
                        id=format!("form-section-default-width-{section_id}")
                        prop:value=move || section.get().default_column_width.to_string()
                        on:change=move |event| {
                            let next_width = event_target_value(&event)
                                .parse::<i32>()
                                .unwrap_or(6)
                                .clamp(1, FORM_BUILDER_COLUMN_COUNT);
                            builder_sections.update(|sections| {
                                if let Some(section) = sections.iter_mut().find(|section| section.id == section_id) {
                                    section.default_column_width = next_width;
                                }
                            });
                        }
                    >
                        {(1..=FORM_BUILDER_COLUMN_COUNT)
                            .map(|width| view! { <option value=width.to_string()>{width}</option> })
                            .collect_view()}
                    </select>
                </label>

                <label class="form-field form-field--wide" for=format!("form-section-description-{section_id}")>
                    <span>"Description"</span>
                    <textarea
                        id=format!("form-section-description-{section_id}")
                        prop:value=move || section.get().description
                        on:input=move |event| {
                            let next_description = event_target_value(&event);
                            builder_sections.update(|sections| {
                                if let Some(section) = sections.iter_mut().find(|section| section.id == section_id) {
                                    section.description = next_description.clone();
                                }
                            });
                        }
                    ></textarea>
                </label>
            </div>

            <FormBuilderGrid
                section_id=section_id
                layout=layout
                default_column_width=default_column_width
                builder_fields=builder_fields
                active_builder_field=active_builder_field
                dragged_builder_field=dragged_builder_field
                builder_drag_preview=builder_drag_preview
                pending_builder_drag_preview=pending_builder_drag_preview
                builder_drag_preview_timeout=builder_drag_preview_timeout
                suppress_builder_field_click=suppress_builder_field_click
                next_builder_field_id=next_builder_field_id
            />
        </article>
    }
}
