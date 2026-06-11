//! Node type detail collection components.

use super::super::api::save_node_type_metadata_field;
use crate::features::organization::{
    NodeMetadataFieldSummary, NodeTypeDefinition, NodeTypeFormLink, RelatedWorkPaginationFooter,
};
use crate::features::shared::status_badge_class;
#[cfg(feature = "hydrate")]
use crate::http::send_json_id_request;
use crate::ui::{DataTable, DropdownMenu};
use crate::utils::metadata::metadata_label;
use crate::utils::pagination::pagination_page_start;
use icons::{Pencil, Plus, Search, Trash2, X};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
/// Renders the node type scoped forms list view.
pub(crate) fn NodeTypeScopedFormsList(forms: Vec<NodeTypeFormLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let has_forms = !forms.is_empty();
    let searchable_forms = forms;
    let filtered_forms = Memo::new(move |_| {
        let query = search.get().trim().to_lowercase();
        searchable_forms
            .iter()
            .filter(|form| {
                query.is_empty()
                    || form.form_name.to_lowercase().contains(&query)
                    || form.form_slug.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_forms.get().len());
    view! {
        <section class="organization-detail-card node-type-detail-list node-type-detail-list--wide">
            <div class="node-type-detail-list__header">
                <h3>"Scoped Forms"</h3>
                <label class="searchable-data-table__search searchable-data-table__control node-type-detail-list__search">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search scoped forms"</span>
                    <input
                        type="search"
                        placeholder="Search forms"
                        prop:value=move || search.get()
                        on:input=move |event| {
                            search.set(event_target_value(&event));
                            page_index.set(0);
                        }
                    />
                </label>
            </div>
            {if !has_forms {
                view! { <p class="muted">"No forms are scoped to this node type."</p> }.into_any()
            } else {
                view! {
                    <div class="capability-list node-type-scoped-forms-list">
                        {move || {
                            let visible_forms = filtered_forms.get();
                            if visible_forms.is_empty() {
                                view! { <div class="capability-list__item">"No scoped forms match this search."</div> }.into_any()
                            } else {
                                let total_count = visible_forms.len();
                                let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                                visible_forms
                                    .iter()
                                    .skip(start)
                                    .take(page_size.get())
                                    .cloned()
                                    .map(|form| view! {
                                        <div class="capability-list__item">
                                            <strong>{form.form_name}</strong>
                                            <small>{form.form_slug}</small>
                                        </div>
                                    })
                                    .collect_view()
                                    .into_any()
                            }
                        }}
                    </div>
                    <RelatedWorkPaginationFooter
                        aria_label="Scoped forms list pagination"
                        label="scoped forms"
                        total_count=total_count
                        page_size=page_size
                        page_index=page_index
                    />
                }
                .into_any()
            }}
        </section>
    }
}

#[component]
/// Renders the node type detail collections view.
pub(crate) fn NodeTypeDetailCollections(
    detail: Option<NodeTypeDefinition>,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    if let Some(detail) = detail {
        view! {
            <div class="administration-node-type-collections">
                <NodeTypeMetadataList
                    node_type_id=detail.id
                    fields=detail.metadata_fields
                    on_metadata_changed
                />
                <NodeTypeScopedFormsList forms=detail.scoped_forms/>
            </div>
        }
        .into_any()
    } else {
        view! { <div></div> }.into_any()
    }
}

#[component]
/// Renders the node type metadata list view.
fn NodeTypeMetadataList(
    node_type_id: String,
    fields: Vec<NodeMetadataFieldSummary>,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    let _ = (&node_type_id, &on_metadata_changed);
    let node_type_id_value = RwSignal::new(node_type_id);
    let search = RwSignal::new(String::new());
    let editing_field_id = RwSignal::new(None::<String>);
    let field_label = RwSignal::new(String::new());
    let field_key = RwSignal::new(String::new());
    let field_type = RwSignal::new("text".to_string());
    let field_required = RwSignal::new(false);
    let is_saving_field = RwSignal::new(false);
    let field_message = RwSignal::new(None::<String>);
    let sheet_open = RwSignal::new(false);
    let has_fields = !fields.is_empty();
    let table_searchable_fields = fields.clone();
    let card_searchable_fields = fields;
    let table_fields = move || {
        let query = search.get().trim().to_lowercase();
        table_searchable_fields
            .iter()
            .filter(|field| {
                query.is_empty()
                    || field.label.to_lowercase().contains(&query)
                    || field.key.to_lowercase().contains(&query)
                    || field.field_type.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let card_fields = move || {
        let query = search.get().trim().to_lowercase();
        card_searchable_fields
            .iter()
            .filter(|field| {
                query.is_empty()
                    || field.label.to_lowercase().contains(&query)
                    || field.key.to_lowercase().contains(&query)
                    || field.field_type.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let clear_field_editor = move || {
        editing_field_id.set(None);
        field_label.set(String::new());
        field_key.set(String::new());
        field_type.set("text".to_string());
        field_required.set(false);
        field_message.set(None);
    };
    let close_field_sheet = move |_| {
        sheet_open.set(false);
        clear_field_editor();
    };
    let open_new_field_sheet = move |_| {
        clear_field_editor();
        sheet_open.set(true);
    };

    view! {
        <section class="organization-detail-card node-type-detail-list node-type-detail-list--wide">
            <div class="node-type-detail-list__header">
                <h3>"Metadata Fields"</h3>
            </div>
            <div class="forms-list forms-list-responsive-table node-type-metadata-list">
                <div class="searchable-data-table">
                    <div class="searchable-data-table__toolbar forms-list__toolbar">
                        <label class="searchable-data-table__search searchable-data-table__control">
                            <Search class="searchable-data-table__control-icon"/>
                            <span class="sr-only">"Search metadata fields"</span>
                            <input
                                type="search"
                                placeholder="Search metadata"
                                prop:value=move || search.get()
                                on:input=move |event| search.set(event_target_value(&event))
                            />
                        </label>
                        <button
                            class="button button--compact button--secondary node-type-add-field-button"
                            type="button"
                            on:click=open_new_field_sheet
                        >
                            <Plus class="button__icon"/>
                            "Add Field"
                        </button>
                    </div>
                    <Show when=move || field_message.get().is_some() && !sheet_open.get()>
                        <p class="form-message">{move || field_message.get().unwrap_or_default()}</p>
                    </Show>
                    {if !has_fields {
                        view! { <p class="muted">"No metadata fields configured."</p> }.into_any()
                    } else {
                        view! {
                            <DataTable>
                                <thead>
                                    <tr>
                                        <th scope="col">"Field"</th>
                                        <th scope="col">"Key"</th>
                                        <th scope="col">"Type"</th>
                                        <th scope="col">"Required"</th>
                                        <th class="data-table__cell--center" scope="col">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        let visible_fields = table_fields();
                                        if visible_fields.is_empty() {
                                            view! {
                                                <tr>
                                                    <td class="data-table__empty" colspan="5">"No Metadata Fields to Display"</td>
                                                </tr>
                                            }
                                            .into_any()
                                        } else {
                                            visible_fields
                                                .into_iter()
                                                .map(|field| {
                                                    let edit_field = field.clone();
                                                    let delete_field = field.clone();
                                                    let row_label = field.label.clone();
                                                    view! {
                                                        <tr>
                                                            <th scope="row">{field.label}</th>
                                                            <td>{field.key}</td>
                                                            <td>{metadata_label(&field.field_type)}</td>
                                                            <td>{if field.required { "Yes" } else { "No" }}</td>
                                                            <td class="data-table__cell--center">
                                                                <DropdownMenu label=format!("Open actions for {row_label}")>
                                                                    <button
                                                                        class="dropdown-menu__item"
                                                                        type="button"
                                                                        role="menuitem"
                                                                        on:click=move |_| {
                                                                            editing_field_id.set(Some(edit_field.id.clone()));
                                                                            field_label.set(edit_field.label.clone());
                                                                            field_key.set(edit_field.key.clone());
                                                                            field_type.set(edit_field.field_type.clone());
                                                                            field_required.set(edit_field.required);
                                                                            field_message.set(None);
                                                                            sheet_open.set(true);
                                                                        }
                                                                    >
                                                                        <Pencil class="dropdown-menu__item-icon"/>
                                                                        <span>"Edit Field"</span>
                                                                    </button>
                                                                    <button
                                                                        class="dropdown-menu__item"
                                                                        type="button"
                                                                        role="menuitem"
                                                                        on:click=move |_| {
                                                                            #[cfg(feature = "hydrate")]
                                                                            {
                                                                                let field_id = delete_field.id.clone();
                                                                                leptos::task::spawn_local(async move {
                                                                                    field_message.set(None);
                                                                                    match send_json_id_request(
                                                                                        gloo_net::http::Request::delete(&format!(
                                                                                            "/api/admin/node-metadata-fields/{field_id}"
                                                                                        )),
                                                                                        None,
                                                                                        "Delete metadata field",
                                                                                    )
                                                                                    .await
                                                                                    {
                                                                                        Ok(_) => {
                                                                                            sheet_open.set(false);
                                                                                            clear_field_editor();
                                                                                            on_metadata_changed();
                                                                                        }
                                                                                        Err(error) => field_message.set(Some(error)),
                                                                                    }
                                                                                });
                                                                            }
                                                                            #[cfg(not(feature = "hydrate"))]
                                                                            let _ = &delete_field;
                                                                        }
                                                                    >
                                                                        <Trash2 class="dropdown-menu__item-icon"/>
                                                                        <span>"Delete Field"</span>
                                                                    </button>
                                                                </DropdownMenu>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()
                                                .into_any()
                                        }
                                    }}
                                </tbody>
                            </DataTable>
                        }
                        .into_any()
                    }}
                </div>
                <div class="forms-list-mobile-cards node-type-metadata-mobile-cards">
                    {if !has_fields {
                        view! { <p class="forms-list-mobile-empty">"No Metadata Fields to Display"</p> }.into_any()
                    } else {
                        view! {
                            {move || {
                                let visible_fields = card_fields();
                                if visible_fields.is_empty() {
                                    view! { <p class="forms-list-mobile-empty">"No Metadata Fields to Display"</p> }.into_any()
                                } else {
                                    visible_fields
                                        .into_iter()
                                        .map(|field| {
                                            let edit_field = field.clone();
                                            let delete_field = field.clone();
                                            view! {
                                                <article class="forms-list-mobile-card node-type-metadata-mobile-card">
                                                    <div class="forms-list-mobile-card__header">
                                                        <div>
                                                            <h3>{field.label}</h3>
                                                            <span>{field.key}</span>
                                                        </div>
                                                        <span class=status_badge_class(if field.required { "active" } else { "inactive" })>
                                                            {if field.required { "Required" } else { "Optional" }}
                                                        </span>
                                                    </div>
                                                    <dl>
                                                        <div>
                                                            <dt>"Type"</dt>
                                                            <dd>{metadata_label(&field.field_type)}</dd>
                                                        </div>
                                                    </dl>
                                                    <div class="workflow-assignment-mobile-card__actions">
                                                        <button
                                                            class="button button--compact button--secondary"
                                                            type="button"
                                                            on:click=move |_| {
                                                                editing_field_id.set(Some(edit_field.id.clone()));
                                                                field_label.set(edit_field.label.clone());
                                                                field_key.set(edit_field.key.clone());
                                                                field_type.set(edit_field.field_type.clone());
                                                                field_required.set(edit_field.required);
                                                                field_message.set(None);
                                                                sheet_open.set(true);
                                                            }
                                                        >
                                                            "Edit Field"
                                                        </button>
                                                        <button
                                                            class="button button--compact button--secondary"
                                                            type="button"
                                                            on:click=move |_| {
                                                                #[cfg(feature = "hydrate")]
                                                                {
                                                                    let field_id = delete_field.id.clone();
                                                                    leptos::task::spawn_local(async move {
                                                                        field_message.set(None);
                                                                        match send_json_id_request(
                                                                            gloo_net::http::Request::delete(&format!(
                                                                                "/api/admin/node-metadata-fields/{field_id}"
                                                                            )),
                                                                            None,
                                                                            "Delete metadata field",
                                                                        )
                                                                        .await
                                                                        {
                                                                            Ok(_) => {
                                                                                sheet_open.set(false);
                                                                                clear_field_editor();
                                                                                on_metadata_changed();
                                                                            }
                                                                            Err(error) => field_message.set(Some(error)),
                                                                        }
                                                                    });
                                                                }
                                                                #[cfg(not(feature = "hydrate"))]
                                                                let _ = &delete_field;
                                                            }
                                                        >
                                                            "Delete Field"
                                                        </button>
                                                    </div>
                                                </article>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                            }}
                        }
                        .into_any()
                    }}
                </div>
            </div>
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
                                            node_type_id_value.get_untracked(),
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
        </section>
    }
}
