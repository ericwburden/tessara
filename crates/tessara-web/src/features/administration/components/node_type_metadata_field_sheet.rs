//! Node type metadata field editor sheet.

use super::super::api::save_node_type_metadata_field;
use icons::X;
use leptos::portal::Portal;
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
/// Renders the node type metadata field editor sheet.
#[component]
pub(super) fn NodeTypeMetadataFieldSheet(
    node_type_id: RwSignal<String>,
    editing_field_id: RwSignal<Option<String>>,
    field_label: RwSignal<String>,
    field_key: RwSignal<String>,
    field_type: RwSignal<String>,
    field_required: RwSignal<bool>,
    is_saving_field: RwSignal<bool>,
    field_message: RwSignal<Option<String>>,
    sheet_open: RwSignal<bool>,
    clear_field_editor: impl Fn() + 'static + Copy + Send + Sync,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let close_field_sheet = move |_| {
        sheet_open.set(false);
        clear_field_editor();
    };

    view! {
        <Portal>
            <Show when=move || sheet_open.get()>
                <section class="sheet-overlay node-type-metadata-overlay" aria-label="Metadata field editor">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close metadata field editor" on:click=close_field_sheet></button>
                    <aside class="sheet-panel blurred-surface node-type-metadata-sheet" role="dialog" aria-modal="true" aria-label="Metadata field editor">
                        <div class="sheet-panel__actions">
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close metadata field editor" title="Close metadata field editor" on:click=close_field_sheet>
                                <X/>
                            </button>
                        </div>

                        <header class="sheet-panel__header">
                            <p>"Node Type Metadata"</p>
                            <h2>{move || if editing_field_id.get().is_some() { "Edit Metadata Field" } else { "Add Metadata Field" }}</h2>
                        </header>

                        <section class="sheet-panel__section">
                            <div class="form-grid node-type-metadata-sheet__fields">
                                <label class="form-field">
                                    <span>"Label"</span>
                                    <input
                                        type="text"
                                        placeholder="Display label"
                                        prop:value=move || field_label.get()
                                        on:input=move |event| field_label.set(event_target_value(&event))
                                    />
                                </label>
                                <label class="form-field">
                                    <span>"Key"</span>
                                    <input
                                        type="text"
                                        placeholder="metadata_key"
                                        prop:value=move || field_key.get()
                                        on:input=move |event| field_key.set(event_target_value(&event))
                                    />
                                </label>
                                <label class="form-field">
                                    <span>"Type"</span>
                                    <select
                                        prop:value=move || field_type.get()
                                        on:change=move |event| field_type.set(event_target_value(&event))
                                    >
                                        <option value="text">"Text"</option>
                                        <option value="number">"Number"</option>
                                        <option value="boolean">"Boolean"</option>
                                        <option value="date">"Date"</option>
                                        <option value="single_choice">"Single Choice"</option>
                                        <option value="multi_choice">"Multi Choice"</option>
                                    </select>
                                </label>
                                <label class="toggle-row toggle-row--compact metadata-field-editor__required">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || field_required.get()
                                        on:change=move |event| field_required.set(event_target_checked(&event))
                                    />
                                    <span>"Required"</span>
                                </label>
                            </div>
                            <Show when=move || field_message.get().is_some()>
                                <p class="form-message">{move || field_message.get().unwrap_or_default()}</p>
                            </Show>
                        </section>

                        <div class="form-actions">
                            <button
                                class="button button--secondary"
                                type="button"
                                on:click=close_field_sheet
                            >
                                "Cancel"
                            </button>
                            <button
                                class="button"
                                type="button"
                                disabled=move || is_saving_field.get()
                                on:click=move |_| {
                                    save_node_type_metadata_field(
                                        node_type_id.get_untracked(),
                                        editing_field_id,
                                        field_label,
                                        field_key,
                                        field_type,
                                        field_required,
                                        is_saving_field,
                                        field_message,
                                        sheet_open,
                                        clear_field_editor,
                                        on_metadata_changed,
                                    );
                                }
                            >
                                {move || {
                                    if is_saving_field.get() {
                                        "Saving"
                                    } else if editing_field_id.get().is_some() {
                                        "Save Field"
                                    } else {
                                        "Create Field"
                                    }
                                }}
                            </button>
                        </div>
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}
