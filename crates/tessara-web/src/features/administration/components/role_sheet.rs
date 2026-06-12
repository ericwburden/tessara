//! Role editor sheet.

use super::super::state::toggle_string_selection;
use crate::features::administration::models::AdminCapabilitySummary;
use crate::utils::text::text_matches;
use icons::{Search, X};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
pub(crate) fn AdminRoleSheet(
    is_open: RwSignal<bool>,
    editing_role_id: RwSignal<Option<String>>,
    role_name: RwSignal<String>,
    capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_capability_ids: RwSignal<Vec<String>>,
    capability_search: RwSignal<String>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    on_close: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
    on_save: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <Portal>
            <Show when=move || is_open.get()>
                <section class="sheet-overlay administration-role-overlay" aria-label="Role editor">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close role editor" on:click=on_close></button>
                    <aside class="sheet-panel blurred-surface administration-role-sheet" role="dialog" aria-modal="true" aria-label="Role editor">
                        <div class="sheet-panel__actions">
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close role editor" title="Close role editor" on:click=on_close>
                                <X/>
                            </button>
                        </div>
                        <header class="sheet-panel__header">
                            <p>"Role Template"</p>
                            <h2>{move || if editing_role_id.get().is_some() { "Edit Role Capabilities" } else { "New Role" }}</h2>
                        </header>
                        <section class="sheet-panel__section">
                            <Show when=move || editing_role_id.get().is_none()>
                                <label class="form-field">
                                    <span>"Role Name"</span>
                                    <input
                                        type="text"
                                        placeholder="coordinator"
                                        prop:value=move || role_name.get()
                                        on:input=move |event| role_name.set(event_target_value(&event))
                                    />
                                </label>
                            </Show>
                            <label class="searchable-data-table__search searchable-data-table__control administration-role-sheet__search">
                                <Search class="searchable-data-table__control-icon"/>
                                <span class="sr-only">"Search capabilities"</span>
                                <input
                                    type="search"
                                    placeholder="Search capabilities"
                                    prop:value=move || capability_search.get()
                                    on:input=move |event| capability_search.set(event_target_value(&event))
                                />
                            </label>
                            <div class="checkbox-list permission-picker__list administration-role-capability-picker">
                                {move || {
                                    let query = capability_search.get();
                                    let selected = selected_capability_ids.get();
                                    let visible = capabilities
                                        .get()
                                        .into_iter()
                                        .filter(|capability| {
                                            text_matches(&query, &[capability.key.as_str(), capability.description.as_str()])
                                        })
                                        .collect::<Vec<_>>();
                                    if visible.is_empty() {
                                        view! { <p class="forms-list-mobile-empty">"No Capabilities to Display"</p> }.into_any()
                                    } else {
                                        visible
                                            .into_iter()
                                            .map(|capability| {
                                                let capability_id = capability.id.clone();
                                                let checked = selected.iter().any(|id| id == &capability.id);
                                                view! {
                                                    <label class="checkbox-list__item permission-picker__item">
                                                        <input
                                                            type="checkbox"
                                                            prop:checked=checked
                                                            on:change=move |event| {
                                                                toggle_string_selection(
                                                                    selected_capability_ids,
                                                                    capability_id.clone(),
                                                                    event_target_checked(&event),
                                                                );
                                                            }
                                                        />
                                                        <span>
                                                            <strong>{capability.key}</strong>
                                                            <small>{capability.description}</small>
                                                        </span>
                                                    </label>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }
                                }}
                            </div>
                            <Show when=move || message.get().is_some()>
                                <p class="form-message" role="status">{move || message.get().unwrap_or_default()}</p>
                            </Show>
                        </section>
                        <div class="form-actions">
                            <button class="button button--secondary" type="button" on:click=on_close>
                                "Cancel"
                            </button>
                            <button class="button" type="button" disabled=move || is_saving.get() on:click=on_save>
                                {move || if is_saving.get() { "Saving..." } else { "Save Role" }}
                            </button>
                        </div>
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}
