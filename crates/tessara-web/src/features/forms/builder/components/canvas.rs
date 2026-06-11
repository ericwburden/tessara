//! Owns the features::forms::builder::components::canvas module behavior.

use leptos::prelude::*;

use crate::features::forms::builder::components::{FieldConfigSheet, FormBuilderSection};
use crate::features::forms::builder::state::{
    FormBuilderEditorState, add_form_builder_section_to_editor,
};
use crate::ui::{Tabs, TabsList};
use icons::Plus;

#[component]
/// Renders the form builder canvas view.
pub(crate) fn FormBuilderCanvas(state: FormBuilderEditorState) -> impl IntoView {
    let FormBuilderEditorState {
        builder_sections,
        active_builder_section,
        next_builder_section_id,
        builder_fields,
        active_builder_field,
        dragged_builder_field,
        builder_drag_preview,
        pending_builder_drag_preview,
        builder_drag_preview_timeout,
        suppress_builder_field_click,
        next_builder_field_id,
    } = state;

    view! {
        <section class="form-builder form-section">
            <div class="form-builder__header">
                <h3>"Form Builder"</h3>
            </div>

            <Tabs active=active_builder_section>
                <TabsList>
                    {move || {
                        builder_sections
                            .get()
                            .into_iter()
                            .map(|section| {
                                let section_value = section.id.to_string();
                                let section_tab_value = section_value.clone();
                                view! {
                                    <button
                                        class=move || {
                                            if active_builder_section.get() == section_tab_value {
                                                "tabs-trigger is-active"
                                            } else {
                                                "tabs-trigger"
                                            }
                                        }
                                        type="button"
                                        role="tab"
                                        aria-selected=move || (active_builder_section.get() == section_value).to_string()
                                        on:click=move |_| active_builder_section.set(section.id.to_string())
                                    >
                                        {section.title}
                                    </button>
                                }
                            })
                            .collect_view()
                    }}
                    <button
                        class="tabs-trigger form-builder__add-section-tab"
                        type="button"
                        on:click=move |_| {
                            add_form_builder_section_to_editor(
                                builder_sections,
                                next_builder_section_id,
                                active_builder_section,
                            );
                        }
                    >
                        <Plus/>
                        "Section"
                    </button>
                </TabsList>
            </Tabs>

            <div class="form-builder__sections">
                <For
                    each=move || {
                        builder_sections
                            .get()
                            .into_iter()
                            .filter(|section| active_builder_section.get() == section.id.to_string())
                            .map(|section| section.id)
                            .collect::<Vec<_>>()
                    }
                    key=|section_id| *section_id
                    children=move |section_id| {
                        view! {
                            <FormBuilderSection
                                section_id=section_id
                                builder_sections=builder_sections
                                builder_fields=builder_fields
                                active_builder_field=active_builder_field
                                dragged_builder_field=dragged_builder_field
                                builder_drag_preview=builder_drag_preview
                                pending_builder_drag_preview=pending_builder_drag_preview
                                builder_drag_preview_timeout=builder_drag_preview_timeout
                                suppress_builder_field_click=suppress_builder_field_click
                                next_builder_field_id=next_builder_field_id
                            />
                        }
                    }
                />
            </div>

            <FieldConfigSheet
                active_builder_field=active_builder_field
                builder_sections=builder_sections
                builder_fields=builder_fields
            />
        </section>
    }
}
