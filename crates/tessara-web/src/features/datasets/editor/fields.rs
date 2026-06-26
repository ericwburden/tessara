//! Dataset editor projected field picker.

use super::super::types::*;
use crate::ui::{
    DraggablePanelList, DraggablePanelListAnchor, DraggablePanelListDraggable,
    DraggablePanelListDropZone, DraggablePanelListItem, DraggablePanelListMove, empty_view,
};
use crate::utils::text::sentence_label;
use icons::{ArrowDown, ArrowUp, Search, Square, SquareCheckBig, Trash2};
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
struct ProjectionFieldGroup {
    label: String,
    fields: Vec<DatasetFieldDraft>,
}

#[component]
pub(crate) fn DatasetProjectionEditor(
    available_fields: Signal<Vec<DatasetFieldDraft>>,
    fields: Signal<Vec<DatasetFieldDraft>>,
    active_source_tab: Signal<Option<String>>,
    on_active_source_tab_change: Callback<Option<String>>,
    on_fields_change: Callback<Vec<DatasetFieldDraft>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let selected_field_keys = Memo::new(move |_| {
        fields.with(|items| {
            items
                .iter()
                .map(|field| field.key.clone())
                .collect::<BTreeSet<_>>()
        })
    });
    let previous_available_fields = RwSignal::new(Vec::<DatasetFieldDraft>::new());

    Effect::new(move |_| {
        let selected_fields = fields.get();
        let available_fields = available_fields.get();
        let reconciled_fields = reconcile_projection_fields(
            selected_fields.clone(),
            available_fields.clone(),
            &previous_available_fields.get_untracked(),
        );
        previous_available_fields.set(available_fields);
        if reconciled_fields != selected_fields {
            on_fields_change.run(reconciled_fields);
        }
    });

    let on_toggle_available_field = Callback::new(move |field_for_toggle: DatasetFieldDraft| {
        let mut items = fields.get();
        if let Some(index) = items
            .iter()
            .position(|item| item.key == field_for_toggle.key)
        {
            items.remove(index);
        } else {
            items.push(field_for_toggle);
        }
        on_fields_change.run(items);
    });
    let selected_field_items = Signal::derive(move || {
        fields
            .get()
            .into_iter()
            .map(|field| DraggablePanelListItem { id: field.key })
            .collect::<Vec<_>>()
    });

    view! {
        <section class="route-panel__section dataset-editor-section dataset-fields-section dataset-editor-section--embedded">
            <div class="dataset-projection-builder">
                <div class="dataset-projection-builder__toolbar">
                    <div class="dataset-projection-builder__search">
                        <Search class="dataset-projection-builder__search-icon"/>
                        <input
                            class="dataset-projection-builder__search-input"
                            type="search"
                            placeholder="Search available fields..."
                            aria-label="Search available fields"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </div>
                    <div class="dataset-projection-builder__toolbar-actions">
                        <button
                            class="button button--secondary"
                            type="button"
                            disabled=move || available_fields.get().is_empty()
                            on:click=move |_| {
                                on_fields_change.run(include_all_projection_fields(
                                    fields.get(),
                                    available_fields.get(),
                                ));
                            }
                        >
                            "Include All"
                        </button>
                        <button
                            class="button button--secondary"
                            type="button"
                            disabled=move || fields.get().is_empty()
                            on:click=move |_| {
                                on_fields_change.run(Vec::new());
                            }
                        >
                            "Clear All"
                        </button>
                    </div>
                </div>

                <ProjectionAvailableFields
                    available_fields=available_fields
                    selected_field_keys=selected_field_keys
                    search=search
                    active_source_tab=active_source_tab
                    on_active_source_tab_change=on_active_source_tab_change
                    on_toggle_field=on_toggle_available_field
                />

                <div class="dataset-projection-selected">
                    <div class="dataset-projection-selected__header">
                        <h5>"Selected Fields"</h5>
                        <small>{move || format!("{} fields", fields.get().len())}</small>
                    </div>
                    {move || {
                        let selected_fields = fields.get();
                        if selected_fields.is_empty() {
                            view! {
                                <p class="muted dataset-projection-builder__empty">
                                    "No fields selected."
                                </p>
                            }
                                .into_any()
                        } else {
                            view! {
                                <DraggablePanelList
                                    list_id="projection-selected-fields"
                                    items=selected_field_items
                                    container_class="dataset-projection-selected__list"
                                    list_class="dataset-projection-selected__list-items"
                                    draggable_class="dataset-projection-selected__item"
                                    drop_zone_class="dataset-projection-selected__drop-zone"
                                    drag_handle_title="Drag field to reorder"
                                    data_transfer_type="application/x-tessara-projection-field"
                                    render_drop_zone=Callback::new(move |_drop_zone: DraggablePanelListDropZone| {
                                        empty_view()
                                    })
                                    render_draggable=Callback::new(move |draggable: DraggablePanelListDraggable| {
                                        let Some(field) = fields
                                            .get()
                                            .into_iter()
                                            .find(|field| field.key == draggable.id)
                                        else {
                                            return empty_view();
                                        };
                                        let field_key = field.key.clone();
                                        let field_key_for_label = field_key.clone();
                                        let field_key_for_remove = field_key.clone();
                                        let field_key_for_up = field_key.clone();
                                        let field_key_for_down = field_key.clone();
                                        view! {
                                            <div class="dataset-projection-selected__row">
                                                    <div class="dataset-projection-selected__fields">
                                                        <label class="form-field dataset-projection-selected__label-field">
                                                            <span>"Display Label"</span>
                                                            <input
                                                                aria-label=format!("Display label for {}", field.label)
                                                                class="dataset-field-picker__label-input"
                                                                prop:value=field.label.clone()
                                                                on:change=move |event| {
                                                                    let value = event_target_value(&event);
                                                                    on_fields_change.run(update_projection_field_label(
                                                                        fields.get(),
                                                                        &field_key_for_label,
                                                                        value,
                                                                    ));
                                                                }
                                                            />
                                                        </label>
                                                        <div class="dataset-projection-selected__meta">
                                                            <span>
                                                                <small>"Field Name"</small>
                                                                <code>{field.key.clone()}</code>
                                                            </span>
                                                            <span>
                                                                <small>"Data Type"</small>
                                                                <strong>{projection_field_type_label(&field)}</strong>
                                                            </span>
                                                        </div>
                                                    </div>
                                                    <div class="dataset-projection-selected__actions">
                                                        <button
                                                            class="icon-button icon-button--compact-control"
                                                            type="button"
                                                            title="Move field up"
                                                            aria-label=format!("Move {} up", field.label)
                                                            on:click=move |_| {
                                                                on_fields_change.run(move_projection_field_by_delta(
                                                                    fields.get(),
                                                                    &field_key_for_up,
                                                                    -1,
                                                                ));
                                                            }
                                                        >
                                                            <ArrowUp class="icon-button__icon"/>
                                                        </button>
                                                        <button
                                                            class="icon-button icon-button--compact-control"
                                                            type="button"
                                                            title="Move field down"
                                                            aria-label=format!("Move {} down", field.label)
                                                            on:click=move |_| {
                                                                on_fields_change.run(move_projection_field_by_delta(
                                                                    fields.get(),
                                                                    &field_key_for_down,
                                                                    1,
                                                                ));
                                                            }
                                                        >
                                                            <ArrowDown class="icon-button__icon"/>
                                                        </button>
                                                        <button
                                                            class="icon-button icon-button--compact-control"
                                                            type="button"
                                                            title="Remove field"
                                                            aria-label=format!("Remove {}", field.label)
                                                            on:click=move |_| {
                                                                on_fields_change.run(remove_projection_field(
                                                                    fields.get(),
                                                                    &field_key_for_remove,
                                                                ));
                                                            }
                                                        >
                                                            <Trash2 class="icon-button__icon"/>
                                                        </button>
                                                    </div>
                                            </div>
                                        }
                                        .into_any()
                                    })
                                    on_move=Callback::new(move |move_event: DraggablePanelListMove| {
                                        let items = fields.get();
                                        let target_index = projection_insert_index_for_anchor(
                                            &items,
                                            &move_event.anchor,
                                        );
                                        on_fields_change.run(move_projection_field_to_index(
                                            items,
                                            &move_event.dragged_id,
                                            target_index,
                                        ));
                                    })
                                />
                            }
                                .into_any()
                        }
                    }}
                </div>
            </div>
        </section>
    }
}

#[component]
fn ProjectionAvailableFields(
    available_fields: Signal<Vec<DatasetFieldDraft>>,
    selected_field_keys: Memo<BTreeSet<String>>,
    search: RwSignal<String>,
    active_source_tab: Signal<Option<String>>,
    on_active_source_tab_change: Callback<Option<String>>,
    on_toggle_field: Callback<DatasetFieldDraft>,
) -> impl IntoView {
    let option_groups = Memo::new(move |_| {
        projection_option_groups(
            sorted_projection_fields(available_fields.get()),
            &search.get(),
        )
    });

    view! {
        <div class="dataset-projection-builder__available" role="listbox" aria-label="Available fields">
            {move || {
                let groups = option_groups.get();
                if groups.is_empty() {
                    return view! {
                        <p class="muted dataset-projection-builder__empty">
                            "No available fields match the current search."
                        </p>
                    }
                        .into_any();
                }

                let active_label = active_source_tab
                    .get()
                    .filter(|label| groups.iter().any(|group| &group.label == label))
                    .unwrap_or_else(|| {
                        groups
                            .first()
                            .map(|group| group.label.clone())
                            .unwrap_or_default()
                    });
                let active_fields = groups
                    .iter()
                    .find(|group| group.label == active_label)
                    .map(|group| group.fields.clone())
                    .unwrap_or_default();

                view! {
                    <div class="dataset-projection-builder__source-tabs" role="tablist" aria-label="Available field sources">
                        {groups
                            .into_iter()
                            .map(|group| {
                                let tab_label = group.label.clone();
                                let is_active = tab_label == active_label;
                                view! {
                                    <button
                                        class="dataset-projection-builder__source-tab"
                                        class:is-active=is_active
                                        type="button"
                                        role="tab"
                                        aria-selected=is_active
                                        on:click=move |_| {
                                            on_active_source_tab_change.run(Some(tab_label.clone()));
                                        }
                                    >
                                        {group.label}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                    <div class="dataset-projection-builder__options">
                        <For
                            each=move || active_fields.clone()
                            key=|field| field.key.clone()
                            children=move |field| {
                                view! {
                                    <ProjectionAvailableFieldOption
                                        field=field
                                        selected_field_keys=selected_field_keys
                                        on_toggle_field=on_toggle_field
                                    />
                                }
                            }
                        />
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
}

#[component]
fn ProjectionAvailableFieldOption(
    field: DatasetFieldDraft,
    selected_field_keys: Memo<BTreeSet<String>>,
    on_toggle_field: Callback<DatasetFieldDraft>,
) -> impl IntoView {
    let label = field.label.clone();
    let key = field.key.clone();
    let type_label = projection_field_type_label(&field);
    let selected_key = key.clone();
    let label_for_aria = label.clone();
    let is_selected =
        Memo::new(move |_| selected_field_keys.with(|keys| keys.contains(&selected_key)));
    let field_for_toggle = field.clone();

    view! {
    <button
        class="dataset-projection-builder__option"
        class:is-selected=move || is_selected.get()
        type="button"
        role="option"
        aria-label=move || {
            if is_selected.get() {
                format!("Remove {label_for_aria}")
            } else {
                format!("Add {label_for_aria}")
            }
        }
        aria-selected=move || is_selected.get()
        on:mousedown=move |event| event.prevent_default()
        on:click=move |_| {
            on_toggle_field.run(field_for_toggle.clone());
        }
    >
            <span class="dataset-projection-builder__option-main">
                <span class="dataset-projection-builder__option-row">
                    <strong>{label}</strong>
                    <small>{type_label}</small>
                </span>
                <span class="dataset-projection-builder__option-meta-row">
                    <code>{key}</code>
                    <span class="dataset-projection-builder__option-check" aria-hidden="true">
                        {move || {
                            if is_selected.get() {
                                view! { <SquareCheckBig class="icon-button__icon"/> }.into_any()
                            } else {
                                view! { <Square class="icon-button__icon"/> }.into_any()
                            }
                        }}
                    </span>
                </span>
            </span>
        </button>
    }
}

fn projection_option_groups(
    available_fields: Vec<DatasetFieldDraft>,
    query: &str,
) -> Vec<ProjectionFieldGroup> {
    let query = query.trim().to_lowercase();
    let mut groups = BTreeMap::<String, Vec<DatasetFieldDraft>>::new();
    for field in available_fields {
        let searchable =
            format!("{} {} {}", field.label, field.key, field.source_alias).to_lowercase();
        if !query.is_empty() && !searchable.contains(&query) {
            continue;
        }
        groups
            .entry(field.source_alias.clone())
            .or_default()
            .push(field);
    }
    let mut groups = groups
        .into_iter()
        .map(|(label, fields)| ProjectionFieldGroup { label, fields })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        projection_source_group_sort_key(&left.label)
            .cmp(&projection_source_group_sort_key(&right.label))
    });
    groups
}

fn projection_source_group_sort_key(label: &str) -> (u8, &str) {
    match label {
        "calculated" | "aggregation" => (1, label),
        _ => (0, label),
    }
}

fn include_all_projection_fields(
    mut selected_fields: Vec<DatasetFieldDraft>,
    available_fields: Vec<DatasetFieldDraft>,
) -> Vec<DatasetFieldDraft> {
    let mut selected_keys = selected_fields
        .iter()
        .map(|field| field.key.clone())
        .collect::<BTreeSet<_>>();
    for field in sorted_projection_fields(available_fields) {
        if selected_keys.insert(field.key.clone()) {
            selected_fields.push(field);
        }
    }
    selected_fields
}

fn reconcile_projection_fields(
    selected_fields: Vec<DatasetFieldDraft>,
    available_fields: Vec<DatasetFieldDraft>,
    previous_available_fields: &[DatasetFieldDraft],
) -> Vec<DatasetFieldDraft> {
    let mut available_by_key = BTreeMap::<String, DatasetFieldDraft>::new();
    let mut available_by_input = BTreeMap::<String, Vec<DatasetFieldDraft>>::new();
    for field in available_fields {
        available_by_input
            .entry(projection_source_key(&field))
            .or_default()
            .push(field.clone());
        available_by_key.insert(field.key.clone(), field);
    }
    let previous_by_key = previous_available_fields
        .iter()
        .map(|field| (field.key.clone(), field.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut previous_by_input = BTreeMap::<String, Vec<DatasetFieldDraft>>::new();
    for field in previous_available_fields {
        previous_by_input
            .entry(projection_source_key(field))
            .or_default()
            .push(field.clone());
    }

    let mut selected_keys = BTreeSet::<String>::new();
    selected_fields
        .into_iter()
        .filter_map(|selected_field| {
            let mut available_field =
                projection_catalog_match(&selected_field, &available_by_key, &available_by_input)?;

            if !selected_keys.insert(available_field.key.clone()) {
                return None;
            }

            if projection_catalog_match(&selected_field, &previous_by_key, &previous_by_input)
                .is_none_or(|previous_field| selected_field.label != previous_field.label)
            {
                available_field.label = selected_field.label;
            }
            Some(available_field)
        })
        .collect()
}

fn projection_catalog_match(
    field: &DatasetFieldDraft,
    fields_by_key: &BTreeMap<String, DatasetFieldDraft>,
    fields_by_input: &BTreeMap<String, Vec<DatasetFieldDraft>>,
) -> Option<DatasetFieldDraft> {
    fields_by_key.get(&field.key).cloned().or_else(|| {
        let input_key = projection_source_key(field);
        fields_by_input
            .get(&input_key)
            .and_then(|matches| (matches.len() == 1).then(|| matches[0].clone()))
    })
}

fn remove_projection_field(
    mut fields: Vec<DatasetFieldDraft>,
    field_key: &str,
) -> Vec<DatasetFieldDraft> {
    fields.retain(|field| field.key != field_key);
    fields
}

fn update_projection_field_label(
    mut fields: Vec<DatasetFieldDraft>,
    field_key: &str,
    label: String,
) -> Vec<DatasetFieldDraft> {
    if let Some(field) = fields.iter_mut().find(|field| field.key == field_key) {
        field.label = label;
    }
    fields
}

fn move_projection_field_by_delta(
    mut fields: Vec<DatasetFieldDraft>,
    field_key: &str,
    delta: isize,
) -> Vec<DatasetFieldDraft> {
    let Some(index) = fields.iter().position(|field| field.key == field_key) else {
        return fields;
    };
    let next_index =
        (index as isize + delta).clamp(0, fields.len().saturating_sub(1) as isize) as usize;
    if index != next_index {
        fields.swap(index, next_index);
    }
    fields
}

fn move_projection_field_to_index(
    mut fields: Vec<DatasetFieldDraft>,
    dragged_key: &str,
    target_index: usize,
) -> Vec<DatasetFieldDraft> {
    let Some(dragged_index) = fields.iter().position(|field| field.key == dragged_key) else {
        return fields;
    };
    let dragged_field = fields.remove(dragged_index);
    let target_index = if dragged_index < target_index {
        target_index.saturating_sub(1)
    } else {
        target_index
    }
    .min(fields.len());
    fields.insert(target_index, dragged_field);
    fields
}

fn projection_insert_index_for_anchor(
    fields: &[DatasetFieldDraft],
    anchor: &DraggablePanelListAnchor,
) -> usize {
    match anchor {
        DraggablePanelListAnchor::Start => 0,
        DraggablePanelListAnchor::After(field_key) => fields
            .iter()
            .position(|field| &field.key == field_key)
            .map(|index| index + 1)
            .unwrap_or(fields.len()),
    }
}

fn sorted_projection_fields(mut fields: Vec<DatasetFieldDraft>) -> Vec<DatasetFieldDraft> {
    fields.sort_by(|left, right| {
        projection_field_group(left)
            .cmp(&projection_field_group(right))
            .then_with(|| {
                projection_source_group_sort_key(&left.source_alias)
                    .cmp(&projection_source_group_sort_key(&right.source_alias))
            })
            .then_with(|| left.key.cmp(&right.key))
    });
    fields
}

fn projection_field_group(field: &DatasetFieldDraft) -> u8 {
    if projection_source_key(field).starts_with("__") {
        0
    } else {
        1
    }
}

fn projection_field_type_label(field: &DatasetFieldDraft) -> String {
    match projection_source_key(field).as_str() {
        "__submission_id" | "__form_version_id" | "__node_id" => "Key".into(),
        "__submission_status" => "Status".into(),
        "__node_name" | "__last_updated_by_user_name" => "Lookup".into(),
        _ => sentence_label(&field.field_type),
    }
}

fn projection_source_key(field: &DatasetFieldDraft) -> String {
    if field.source_field_key.starts_with("__") {
        return field.source_field_key.clone();
    }
    let source_prefix = format!("{}__", field.source_alias);
    let suffix = field
        .key
        .strip_prefix(&source_prefix)
        .unwrap_or(&field.source_field_key);
    match suffix.trim_start_matches('_') {
        "submission_id"
        | "form_version_id"
        | "node_id"
        | "node_name"
        | "submission_status"
        | "submitted_at"
        | "submission_created_at"
        | "last_updated_at"
        | "last_updated_by_user_name" => format!("__{}", suffix.trim_start_matches('_')),
        _ => field.source_field_key.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconcile_projection_fields_removes_unavailable_fields() {
        let selected_fields = vec![
            projection_field("source_1__included", "included", "text"),
            projection_field("source_1__removed", "removed", "number"),
        ];
        let available_fields = vec![projection_field("source_1__included", "included", "text")];

        let reconciled = reconcile_projection_fields(selected_fields, available_fields, &[]);

        assert_eq!(
            reconciled,
            vec![projection_field("source_1__included", "included", "text")]
        );
    }

    #[test]
    fn reconcile_projection_fields_updates_alias_keys_and_preserves_custom_display_labels() {
        let selected_fields = vec![DatasetFieldDraft {
            label: "My Display Label".into(),
            ..projection_field("source_1__field_a", "field_a", "number")
        }];
        let available_fields = vec![projection_field(
            "renamed_source__field_a",
            "field_a",
            "number",
        )];
        let previous_available_fields =
            vec![projection_field("source_1__field_a", "field_a", "number")];

        let reconciled = reconcile_projection_fields(
            selected_fields,
            available_fields,
            &previous_available_fields,
        );

        assert_eq!(reconciled.len(), 1);
        assert_eq!(reconciled[0].key, "renamed_source__field_a");
        assert_eq!(reconciled[0].source_alias, "renamed_source");
        assert_eq!(reconciled[0].source_field_key, "field_a");
        assert_eq!(reconciled[0].label, "My Display Label");
    }

    #[test]
    fn reconcile_projection_fields_updates_inherited_display_labels() {
        let selected_fields = vec![DatasetFieldDraft {
            label: "Focus Tags".into(),
            ..projection_field("source_1__focus_tags", "focus_tags", "multi_choice")
        }];
        let previous_available_fields = vec![DatasetFieldDraft {
            label: "Focus Tags".into(),
            ..projection_field("source_1__focus_tags", "focus_tags", "multi_choice")
        }];
        let available_fields = vec![DatasetFieldDraft {
            label: "Focus Tags Updated".into(),
            ..projection_field("source_1__focus_tags", "focus_tags", "multi_choice")
        }];

        let reconciled = reconcile_projection_fields(
            selected_fields,
            available_fields,
            &previous_available_fields,
        );

        assert_eq!(reconciled.len(), 1);
        assert_eq!(reconciled[0].label, "Focus Tags Updated");
    }

    #[test]
    fn reconcile_projection_fields_drops_ambiguous_alias_fallbacks() {
        let selected_fields = vec![projection_field("source_1__field_a", "field_a", "number")];
        let available_fields = vec![
            projection_field("left__field_a", "field_a", "number"),
            projection_field("right__field_a", "field_a", "number"),
        ];

        let reconciled = reconcile_projection_fields(selected_fields, available_fields, &[]);

        assert!(reconciled.is_empty());
    }

    fn projection_field(key: &str, source_field_key: &str, field_type: &str) -> DatasetFieldDraft {
        let source_alias = key
            .split_once("__")
            .map(|(source_alias, _)| source_alias)
            .unwrap_or_default();

        DatasetFieldDraft {
            key: key.into(),
            label: key.into(),
            source_alias: source_alias.into(),
            source_field_key: source_field_key.into(),
            field_type: field_type.into(),
        }
    }
}
