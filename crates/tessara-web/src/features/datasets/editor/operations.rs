//! Ordered dataset operation controls.

use super::SourceOptionsFields;
use super::aggregation::DatasetAggregationEditor;
use super::calculations::DatasetCalculationsEditor;
use super::fields::DatasetProjectionEditor;
use super::filters::DatasetFiltersEditor;
use super::pipeline_fields::{fields_after_aggregation, fields_after_calculations};
use super::source_field_actions::{canonical_field_key, rename_source_alias_references};
use super::source_options::source_field_options;
use crate::features::datasets::types::{
    DatasetAggregationDraft, DatasetCalculatedFieldDraft, DatasetFieldDraft, DatasetFormOption,
    DatasetOperationDraft, DatasetOperationDraftKind, DatasetRenderedForm, DatasetRowFilterDraft,
    DatasetSourceDraft, DatasetSummary, DatasetUserOption, NodeResponse,
};
use crate::ui::{
    DraggablePanelList, DraggablePanelListAnchor, DraggablePanelListDraggable,
    DraggablePanelListDropZone, DraggablePanelListItem, DraggablePanelListMove, SegmentedToggle,
    SegmentedToggleOption, empty_view,
};
use icons::{ArrowDown, ArrowUp, ChevronsDownUp, ChevronsUpDown, Plus, Trash2};
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[allow(clippy::too_many_arguments)]
#[component]
pub(crate) fn DatasetOperationSequence(
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    initial_source: RwSignal<DatasetSourceDraft>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    nodes: RwSignal<Vec<NodeResponse>>,
    users: RwSignal<Vec<DatasetUserOption>>,
) -> impl IntoView {
    let expanded_operation_ids = RwSignal::new(BTreeSet::<u64>::new());
    let open_insert_menu_anchor = RwSignal::new(None::<DraggablePanelListAnchor>);
    let projection_active_source_tabs = RwSignal::new(BTreeMap::<u64, String>::new());
    let operation_items = Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .map(|operation| DraggablePanelListItem {
                id: operation.id.to_string(),
            })
            .collect::<Vec<_>>()
    });

    view! {
        <DraggablePanelList
            list_id="dataset-operations"
            items=operation_items
            container_class="dataset-operation-sequence"
            list_class="dataset-operation-sequence__list"
            draggable_class="route-panel__section dataset-editor-section dataset-operation-panel"
            drop_zone_class="dataset-operation-insert"
            drag_handle_title="Drag operation to reorder"
            data_transfer_type="application/x-tessara-operation"
            render_draggable=Callback::new(move |draggable: DraggablePanelListDraggable| {
                let Some(operation_id) = draggable.id.parse::<u64>().ok() else {
                    return empty_view();
                };
                let Some(operation) = operation_order
                    .get()
                    .into_iter()
                    .find(|operation| operation.id == operation_id)
                else {
                    return empty_view();
                };

                operation_panel(
                    draggable.index,
                    operation,
                    operation_order,
                    initial_source,
                    forms,
                    datasets,
                    rendered_forms,
                    nodes,
                    users,
                    expanded_operation_ids,
                    projection_active_source_tabs,
                )
                .into_any()
            })
            render_drop_zone=Callback::new(move |drop_zone: DraggablePanelListDropZone| {
                operation_insert_control(
                    drop_zone.anchor,
                    operation_order,
                    initial_source,
                    open_insert_menu_anchor,
                )
                .into_any()
            })
            on_move=Callback::new(move |move_event: DraggablePanelListMove| {
                if let Some(dragged_id) = move_event.dragged_id.parse::<u64>().ok() {
                    move_operation_to_anchor(operation_order, dragged_id, move_event.anchor);
                }
            })
        />
    }
}

#[allow(clippy::too_many_arguments)]
fn operation_panel(
    fallback_index: usize,
    operation: DatasetOperationDraft,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    initial_source: RwSignal<DatasetSourceDraft>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    nodes: RwSignal<Vec<NodeResponse>>,
    users: RwSignal<Vec<DatasetUserOption>>,
    expanded_operation_ids: RwSignal<BTreeSet<u64>>,
    projection_active_source_tabs: RwSignal<BTreeMap<u64, String>>,
) -> impl IntoView {
    let operation_id = operation.id;
    let kind = operation.kind;
    let label = kind.label();
    let title = move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .map(|operation| operation.kind.label())
            .unwrap_or(label)
    };
    let position = move || {
        operation_order
            .get()
            .iter()
            .position(|operation| operation.id == operation_id)
            .map(|index| index + 1)
            .unwrap_or(fallback_index + 1)
    };
    let is_first = move || position() <= 1;
    let is_last = move || position() >= operation_order.get().len();

    view! {
        <>
            <div class="dataset-operation-panel__header">
                <button
                    class="dataset-operation-panel__toggle"
                    type="button"
                    aria-expanded=move || {
                        expanded_operation_ids
                            .get()
                            .contains(&operation_id)
                            .to_string()
                    }
                    on:click=move |_| {
                        expanded_operation_ids
                            .update(|ids| toggle_operation_expansion(ids, operation_id));
                    }
                >
                    <span class="dataset-operation-panel__collapse-icon" aria-hidden="true">
                        {move || {
                            if expanded_operation_ids.get().contains(&operation_id) {
                                view! { <ChevronsDownUp class="icon-button__icon"/> }.into_any()
                            } else {
                                view! { <ChevronsUpDown class="icon-button__icon"/> }.into_any()
                            }
                        }}
                    </span>
                    <span class="dataset-operation-panel__position">{move || position()}</span>
                    <span class="dataset-operation-panel__title">{move || title()}</span>
                </button>
                <div class="dataset-operation-panel__actions">
                    <button
                        class="icon-button icon-button--compact-control"
                        type="button"
                        title="Move operation up"
                        aria-label=format!("Move {label} up")
                        disabled=is_first
                        on:click=move |_| move_operation_by_delta(operation_order, operation_id, -1)
                    >
                        <ArrowUp class="icon-button__icon"/>
                    </button>
                    <button
                        class="icon-button icon-button--compact-control"
                        type="button"
                        title="Move operation down"
                        aria-label=format!("Move {label} down")
                        disabled=is_last
                        on:click=move |_| move_operation_by_delta(operation_order, operation_id, 1)
                    >
                        <ArrowDown class="icon-button__icon"/>
                    </button>
                    <button
                        class="icon-button icon-button--compact-control"
                        type="button"
                        title="Remove operation"
                        aria-label=format!("Remove {label}")
                        on:click=move |_| remove_operation(operation_order, operation_id)
                    >
                        <Trash2 class="icon-button__icon"/>
                    </button>
                </div>
            </div>
            <Show when=move || expanded_operation_ids.get().contains(&operation_id)>
                {move || view! {
                    <div class="dataset-operation-panel__body">
                        {operation_body(
                            operation_id,
                            kind,
                            operation_order,
                            initial_source,
                            forms,
                            datasets,
                            rendered_forms,
                            nodes,
                            users,
                            projection_active_source_tabs,
                        )}
                    </div>
                }}
            </Show>
        </>
    }
}

#[allow(clippy::too_many_arguments)]
fn operation_body(
    operation_id: u64,
    kind: DatasetOperationDraftKind,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    initial_source: RwSignal<DatasetSourceDraft>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    nodes: RwSignal<Vec<NodeResponse>>,
    users: RwSignal<Vec<DatasetUserOption>>,
    projection_active_source_tabs: RwSignal<BTreeMap<u64, String>>,
) -> AnyView {
    match kind {
        DatasetOperationDraftKind::JoinSource => source_join_body(
            operation_id,
            operation_order,
            initial_source,
            forms,
            datasets,
            rendered_forms,
        ),
        DatasetOperationDraftKind::UnionSource | DatasetOperationDraftKind::UnionAllSource => {
            source_union_body(
                operation_id,
                operation_order,
                forms,
                datasets,
                rendered_forms,
            )
        }
        DatasetOperationDraftKind::Projection => {
            let operation_fields = Memo::new(move |_| {
                catalog_before_operation_id(
                    initial_source.get(),
                    forms.get(),
                    rendered_forms.get(),
                    operation_order.get(),
                    operation_id,
                )
            });
            view! {
                <DatasetProjectionEditor
                    available_fields=Signal::derive(move || operation_fields.get())
                    fields=operation_projection_fields(operation_order, operation_id)
                    active_source_tab=Signal::derive(move || {
                        projection_active_source_tabs.get().get(&operation_id).cloned()
                    })
                    on_active_source_tab_change=Callback::new(move |source_tab| {
                        projection_active_source_tabs.update(|source_tabs| {
                            if let Some(source_tab) = source_tab {
                                source_tabs.insert(operation_id, source_tab);
                            } else {
                                source_tabs.remove(&operation_id);
                            }
                        });
                    })
                    on_fields_change=Callback::new(move |fields| {
                        update_operation(operation_order, operation_id, |operation| {
                            operation.projection_fields = fields;
                        });
                    })
                />
            }
        }
        .into_any(),
        DatasetOperationDraftKind::Aggregation => {
            let operation_fields = Signal::derive(move || {
                catalog_before_operation_id(
                    initial_source.get(),
                    forms.get(),
                    rendered_forms.get(),
                    operation_order.get(),
                    operation_id,
                )
            });
            view! {
                <DatasetAggregationEditor
                    fields=operation_fields
                    aggregation=operation_aggregation(operation_order, operation_id)
                    on_aggregation_change=Callback::new(move |aggregation| {
                        update_operation(operation_order, operation_id, |operation| {
                            operation.aggregation = aggregation;
                        });
                    })
                    embedded=true
                />
            }
            .into_any()
        }
        DatasetOperationDraftKind::CalculatedFields => {
            let operation_fields = Signal::derive(move || {
                catalog_before_operation_id(
                    initial_source.get(),
                    forms.get(),
                    rendered_forms.get(),
                    operation_order.get(),
                    operation_id,
                )
            });
            view! {
                <DatasetCalculationsEditor
                    fields=operation_fields
                    calculated_fields=operation_calculated_fields(operation_order, operation_id)
                    on_calculated_fields_change=Callback::new(move |calculated_fields| {
                        update_operation(operation_order, operation_id, |operation| {
                            operation.calculated_fields = calculated_fields;
                        });
                    })
                    embedded=true
                />
            }
            .into_any()
        }
        DatasetOperationDraftKind::Filter => {
            let operation_fields = Signal::derive(move || {
                catalog_before_operation_id(
                    initial_source.get(),
                    forms.get(),
                    rendered_forms.get(),
                    operation_order.get(),
                    operation_id,
                )
            });
            view! {
                <DatasetFiltersEditor
                    fields=operation_fields
                    initial_source=initial_source
                    forms=forms
                    rendered_forms=rendered_forms
                    nodes=nodes
                    users=users
                    row_filters=operation_filters(operation_order, operation_id)
                    on_row_filters_change=Callback::new(move |row_filters| {
                        update_operation(operation_order, operation_id, |operation| {
                            operation.row_filters = row_filters;
                        });
                    })
                    embedded=true
                />
            }
            .into_any()
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn source_union_body(
    operation_id: u64,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
) -> AnyView {
    let operation_source = operation_source(operation_order, operation_id);
    let union_mode = Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .map(|operation| {
                if operation.kind == DatasetOperationDraftKind::UnionAllSource {
                    "union_all".to_string()
                } else {
                    "union".to_string()
                }
            })
            .unwrap_or_else(|| "union".into())
    });

    view! {
        <div class="dataset-operation-panel__source">
            <div class="dataset-operation-toggle-row">
                <span class="dataset-operation-toggle-row__label">"Union mode"</span>
                <SegmentedToggle
                    active=union_mode
                    aria_label="Union mode"
                    class="segmented-toggle--binary"
                    options=vec![
                        SegmentedToggleOption { value: "union", label: "Union" },
                        SegmentedToggleOption { value: "union_all", label: "Union All" },
                    ]
                    on_select=Callback::new(move |mode: String| {
                        operation_order.update(|operations| {
                            if let Some(operation) = operations
                                .iter_mut()
                                .find(|operation| operation.id == operation_id)
                            {
                                operation.kind = if mode == "union_all" {
                                    DatasetOperationDraftKind::UnionAllSource
                                } else {
                                    DatasetOperationDraftKind::UnionSource
                                };
                            }
                        });
                    })
                />
            </div>
            <SourceOptionsFields
                source_signal=operation_source
                on_source_change=Callback::new(move |source: DatasetSourceDraft| {
                    let previous_alias = operation_order
                        .get()
                        .into_iter()
                        .find(|operation| operation.id == operation_id)
                        .and_then(|operation| operation.source)
                        .map(|source| source.source_alias)
                        .unwrap_or_default();
                    let next_alias = source.source_alias.clone();
                    update_operation(operation_order, operation_id, |operation| {
                        operation.source = Some(source);
                    });
                    rename_source_alias_references(&previous_alias, &next_alias, operation_order);
                })
                forms=forms
                datasets=datasets
                rendered_forms=rendered_forms
            />
        </div>
    }
    .into_any()
}

#[allow(clippy::too_many_arguments)]
fn source_join_body(
    operation_id: u64,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    initial_source: RwSignal<DatasetSourceDraft>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
) -> AnyView {
    let operation_source = operation_source(operation_order, operation_id);
    let join_type = Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .map(|operation| operation.join_type)
            .unwrap_or_else(|| "left_join".into())
    });
    let current_fields = Signal::derive(move || {
        catalog_before_operation_id(
            initial_source.get(),
            forms.get(),
            rendered_forms.get(),
            operation_order.get(),
            operation_id,
        )
    });
    let left_field_key = Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .map(|operation| operation.left_field_key)
            .unwrap_or_default()
    });
    let right_field_key = Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .map(|operation| operation.right_field_key)
            .unwrap_or_default()
    });
    view! {
        <div class="dataset-operation-panel__source">
            <div class="dataset-operation-toggle-row">
                <span class="dataset-operation-toggle-row__label">"Join type"</span>
                <SegmentedToggle
                    active=join_type
                    aria_label="Join type"
                    options=vec![
                        SegmentedToggleOption { value: "left_join", label: "Left" },
                        SegmentedToggleOption { value: "inner_join", label: "Inner" },
                        SegmentedToggleOption { value: "outer_join", label: "Outer" },
                    ]
                    on_select=Callback::new(move |value: String| {
                        operation_order.update(|operations| {
                            if let Some(operation) = operations
                                .iter_mut()
                                .find(|operation| operation.id == operation_id)
                            {
                                operation.join_type = value;
                            }
                        });
                    })
                />
            </div>
            <SourceOptionsFields
                source_signal=operation_source
                on_source_change=Callback::new(move |source: DatasetSourceDraft| {
                    let previous_alias = operation_order
                        .get()
                        .into_iter()
                        .find(|operation| operation.id == operation_id)
                        .and_then(|operation| operation.source)
                        .map(|source| source.source_alias)
                        .unwrap_or_default();
                    let next_alias = source.source_alias.clone();
                    update_operation(operation_order, operation_id, |operation| {
                        operation.source = Some(source);
                    });
                    rename_source_alias_references(&previous_alias, &next_alias, operation_order);
                })
                forms=forms
                datasets=datasets
                rendered_forms=rendered_forms
            />
            <div class="dataset-operation-panel__join-grid">
                <label class="form-field">
                    <span>"Current Field"</span>
                    <select
                        prop:value=move || left_field_key.get()
                        on:change=move |event| {
                            let value = event_target_value(&event);
                            operation_order.update(|operations| {
                                if let Some(operation) = operations
                                    .iter_mut()
                                    .find(|operation| operation.id == operation_id)
                                {
                                    operation.left_field_key = value.clone();
                                }
                            });
                        }
                    >
                        <option value="">"Select field"</option>
                        {move || sorted_fields(current_fields.get()).into_iter().map(|field| {
                            view! {
                                <option value=field.key.clone()>
                                    {format!("{} ({})", field.label, field.key)}
                                </option>
                            }
                        }).collect_view()}
                    </select>
                </label>
                <label class="form-field">
                    <span>"Source Field"</span>
                    <select
                        prop:value=move || right_field_key.get()
                        on:change=move |event| {
                            let value = event_target_value(&event);
                            operation_order.update(|operations| {
                                if let Some(operation) = operations
                                    .iter_mut()
                                    .find(|operation| operation.id == operation_id)
                                {
                                    operation.right_field_key = value.clone();
                                }
                            });
                        }
                    >
                        <option value="">"Select field"</option>
                        {move || {
                            source_fields_for_source(
                                &operation_source.get(),
                                &forms.get(),
                                &rendered_forms.get(),
                            ).into_iter().map(|field| {
                            view! {
                                <option value=field.key.clone()>
                                    {format!("{} ({})", field.label, field.key)}
                                </option>
                            }
                        }).collect_view()
                        }}
                    </select>
                </label>
            </div>
        </div>
    }
    .into_any()
}

#[allow(clippy::too_many_arguments)]
fn operation_insert_control(
    anchor: DraggablePanelListAnchor,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    initial_source: RwSignal<DatasetSourceDraft>,
    open_insert_menu_anchor: RwSignal<Option<DraggablePanelListAnchor>>,
) -> impl IntoView {
    let anchor_for_expanded = anchor.clone();
    let anchor_for_click = anchor.clone();
    let anchor_for_show = anchor.clone();
    view! {
        <>
            <span class="dataset-operation-insert__line" aria-hidden="true"></span>
            <div class="dataset-operation-insert__control">
                <button
                    class="icon-button icon-button--compact-control dataset-operation-insert__button"
                    type="button"
                    aria-label="Add operation"
                    title="Add operation"
                    aria-expanded=move || {
                        (open_insert_menu_anchor.get() == Some(anchor_for_expanded.clone())).to_string()
                    }
                    on:click=move |_| {
                        open_insert_menu_anchor.update(|open_anchor| {
                            *open_anchor = if *open_anchor == Some(anchor_for_click.clone()) {
                                None
                            } else {
                                Some(anchor_for_click.clone())
                            };
                        });
                    }
                >
                    <Plus class="icon-button__icon"/>
                </button>
                <Show when=move || open_insert_menu_anchor.get() == Some(anchor_for_show.clone())>
                    <div class="dataset-operation-insert__menu">
                        {operation_add_menu_button(
                            "Join Source",
                            "join_source",
                            anchor.clone(),
                            operation_order,
                            initial_source,
                            open_insert_menu_anchor,
                        )}
                        {operation_add_menu_button(
                            "Union Source",
                            "union_source",
                            anchor.clone(),
                            operation_order,
                            initial_source,
                            open_insert_menu_anchor,
                        )}
                        {operation_add_menu_button(
                            "Projection",
                            "projection",
                            anchor.clone(),
                            operation_order,
                            initial_source,
                            open_insert_menu_anchor,
                        )}
                        {operation_add_menu_button(
                            "Aggregation",
                            "aggregation",
                            anchor.clone(),
                            operation_order,
                            initial_source,
                            open_insert_menu_anchor,
                        )}
                        {operation_add_menu_button(
                            "Calculated Fields",
                            "calculated_fields",
                            anchor.clone(),
                            operation_order,
                            initial_source,
                            open_insert_menu_anchor,
                        )}
                        {operation_add_menu_button(
                            "Filter",
                            "filter",
                            anchor.clone(),
                            operation_order,
                            initial_source,
                            open_insert_menu_anchor,
                        )}
                    </div>
                </Show>
            </div>
            <span class="dataset-operation-insert__line" aria-hidden="true"></span>
        </>
    }
}

#[allow(clippy::too_many_arguments)]
fn operation_add_menu_button(
    label: &'static str,
    kind: &'static str,
    anchor: DraggablePanelListAnchor,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    initial_source: RwSignal<DatasetSourceDraft>,
    open_insert_menu_anchor: RwSignal<Option<DraggablePanelListAnchor>>,
) -> impl IntoView {
    let anchor_for_click = anchor.clone();
    view! {
        <button
            type="button"
            on:click=move |_| {
                add_operation_at(
                    kind,
                    anchor_for_click.clone(),
                    operation_order,
                    initial_source,
                );
                open_insert_menu_anchor.set(None);
            }
        >
            {label}
        </button>
    }
}

#[allow(clippy::too_many_arguments)]
fn add_operation_at(
    kind: &str,
    anchor: DraggablePanelListAnchor,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    initial_source: RwSignal<DatasetSourceDraft>,
) {
    let next_id = operation_order
        .get()
        .iter()
        .map(|operation| operation.id)
        .max()
        .unwrap_or(0)
        + 1;
    let insert_operation = |operation_order: RwSignal<Vec<DatasetOperationDraft>>,
                            operation: DatasetOperationDraft| {
        operation_order.update(|operations| {
            let index = insert_index_for_anchor(operations, anchor);
            operations.insert(index, operation);
        });
    };

    match kind {
        "join_source" => {
            let mut operation =
                DatasetOperationDraft::new(next_id, DatasetOperationDraftKind::JoinSource);
            operation.source = Some(new_operation_source_draft(
                &initial_source.get(),
                &operation_order.get(),
            ));
            operation.join_type = "left_join".into();
            insert_operation(operation_order, operation);
        }
        "union_source" => {
            let mut operation =
                DatasetOperationDraft::new(next_id, DatasetOperationDraftKind::UnionSource);
            operation.source = Some(new_operation_source_draft(
                &initial_source.get(),
                &operation_order.get(),
            ));
            insert_operation(operation_order, operation);
        }
        "union_all_source" => {
            let mut operation =
                DatasetOperationDraft::new(next_id, DatasetOperationDraftKind::UnionAllSource);
            operation.source = Some(new_operation_source_draft(
                &initial_source.get(),
                &operation_order.get(),
            ));
            insert_operation(operation_order, operation);
        }
        "projection" => insert_operation(
            operation_order,
            DatasetOperationDraft::new(next_id, DatasetOperationDraftKind::Projection),
        ),
        "aggregation" => insert_operation(
            operation_order,
            DatasetOperationDraft::new(next_id, DatasetOperationDraftKind::Aggregation),
        ),
        "calculated_fields" => insert_operation(
            operation_order,
            DatasetOperationDraft::new(next_id, DatasetOperationDraftKind::CalculatedFields),
        ),
        "filter" => insert_operation(
            operation_order,
            DatasetOperationDraft::new(next_id, DatasetOperationDraftKind::Filter),
        ),
        _ => {}
    }
}

fn remove_operation(operation_order: RwSignal<Vec<DatasetOperationDraft>>, operation_id: u64) {
    operation_order.update(|operations| {
        operations.retain(|operation| operation.id != operation_id);
    });
}

fn move_operation_by_delta(
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    operation_id: u64,
    delta: isize,
) {
    operation_order.update(|operations| {
        move_operation_by_delta_in_place(operations, operation_id, delta);
    });
}

fn move_operation_by_delta_in_place(
    operations: &mut [DatasetOperationDraft],
    operation_id: u64,
    delta: isize,
) {
    let Some(index) = operations
        .iter()
        .position(|operation| operation.id == operation_id)
    else {
        return;
    };
    let target_index = index.saturating_add_signed(delta);
    if target_index < operations.len() {
        operations.swap(index, target_index);
    }
}

fn move_operation_to_anchor(
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    dragged_id: u64,
    anchor: DraggablePanelListAnchor,
) {
    operation_order.update(|operations| {
        let target_index = insert_index_for_anchor(operations, anchor);
        move_operation_to_index_in_place(operations, dragged_id, target_index);
    });
}

fn insert_index_for_anchor(
    operations: &[DatasetOperationDraft],
    anchor: DraggablePanelListAnchor,
) -> usize {
    match anchor {
        DraggablePanelListAnchor::Start => 0,
        DraggablePanelListAnchor::After(operation_id) => operations
            .iter()
            .position(|operation| operation.id.to_string() == operation_id)
            .map(|index| index + 1)
            .unwrap_or(operations.len()),
    }
}

#[cfg(test)]
fn move_operation_before_in_place(
    operations: &mut Vec<DatasetOperationDraft>,
    dragged_id: u64,
    target_id: u64,
) {
    let Some(target_index) = operations
        .iter()
        .position(|operation| operation.id == target_id)
    else {
        return;
    };
    move_operation_to_index_in_place(operations, dragged_id, target_index);
}

fn move_operation_to_index_in_place(
    operations: &mut Vec<DatasetOperationDraft>,
    dragged_id: u64,
    target_index: usize,
) {
    let Some(from_index) = operations
        .iter()
        .position(|operation| operation.id == dragged_id)
    else {
        return;
    };
    let operation = operations.remove(from_index);
    let target_index = if from_index < target_index {
        target_index.saturating_sub(1)
    } else {
        target_index
    }
    .min(operations.len());
    operations.insert(target_index, operation);
}

fn new_operation_source_draft(
    initial_source: &DatasetSourceDraft,
    operations: &[DatasetOperationDraft],
) -> DatasetSourceDraft {
    DatasetSourceDraft {
        source_alias: unique_source_alias(initial_source, operations),
        ..DatasetSourceDraft::default()
    }
}

fn unique_source_alias(
    initial_source: &DatasetSourceDraft,
    operations: &[DatasetOperationDraft],
) -> String {
    for index in 2.. {
        let alias = format!("source_{index}");
        let exists_in_initial_source = initial_source.source_alias == alias;
        let exists_in_operations = operations
            .iter()
            .filter_map(|operation| operation.source.as_ref())
            .any(|source| source.source_alias == alias);
        if !exists_in_initial_source && !exists_in_operations {
            return alias;
        }
    }
    "source".into()
}

fn operation_source(
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    operation_id: u64,
) -> Signal<DatasetSourceDraft> {
    Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .and_then(|operation| operation.source)
            .unwrap_or_default()
    })
}

fn operation_projection_fields(
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    operation_id: u64,
) -> Signal<Vec<DatasetFieldDraft>> {
    Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .map(|operation| operation.projection_fields)
            .unwrap_or_default()
    })
}

fn operation_aggregation(
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    operation_id: u64,
) -> Signal<DatasetAggregationDraft> {
    Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .map(|operation| operation.aggregation)
            .unwrap_or_default()
    })
}

fn operation_calculated_fields(
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    operation_id: u64,
) -> Signal<Vec<DatasetCalculatedFieldDraft>> {
    Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .map(|operation| operation.calculated_fields)
            .unwrap_or_default()
    })
}

fn operation_filters(
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    operation_id: u64,
) -> Signal<Vec<DatasetRowFilterDraft>> {
    Signal::derive(move || {
        operation_order
            .get()
            .into_iter()
            .find(|operation| operation.id == operation_id)
            .map(|operation| operation.row_filters)
            .unwrap_or_default()
    })
}

fn update_operation(
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    operation_id: u64,
    update: impl FnOnce(&mut DatasetOperationDraft),
) {
    operation_order.update(|operations| {
        if let Some(operation) = operations
            .iter_mut()
            .find(|operation| operation.id == operation_id)
        {
            update(operation);
        }
    });
}

fn toggle_operation_expansion(expanded_ids: &mut BTreeSet<u64>, operation_id: u64) {
    if !expanded_ids.insert(operation_id) {
        expanded_ids.remove(&operation_id);
    }
}

fn sorted_fields(mut fields: Vec<DatasetFieldDraft>) -> Vec<DatasetFieldDraft> {
    fields.sort_by(|left, right| {
        field_sort_group(left)
            .cmp(&field_sort_group(right))
            .then_with(|| left.key.cmp(&right.key))
    });
    fields
}

fn field_sort_group(field: &DatasetFieldDraft) -> u8 {
    if field.source_field_key.starts_with("__") {
        0
    } else {
        1
    }
}

fn catalog_before_operation_id(
    initial_source: DatasetSourceDraft,
    forms: Vec<DatasetFormOption>,
    rendered_forms: BTreeMap<String, DatasetRenderedForm>,
    operation_order: Vec<DatasetOperationDraft>,
    target_id: u64,
) -> Vec<DatasetFieldDraft> {
    let mut current_fields = source_catalog_for_initial(&initial_source, &forms, &rendered_forms);

    for operation in operation_order {
        if operation.id == target_id {
            return sorted_fields(current_fields);
        }

        current_fields =
            apply_operation_to_catalog(current_fields, operation, &forms, &rendered_forms);
    }

    sorted_fields(current_fields)
}

pub(super) fn catalog_after_operations(
    initial_source: DatasetSourceDraft,
    forms: Vec<DatasetFormOption>,
    rendered_forms: BTreeMap<String, DatasetRenderedForm>,
    operation_order: Vec<DatasetOperationDraft>,
) -> Vec<DatasetFieldDraft> {
    let mut current_fields = source_catalog_for_initial(&initial_source, &forms, &rendered_forms);
    for operation in operation_order {
        current_fields =
            apply_operation_to_catalog(current_fields, operation, &forms, &rendered_forms);
    }
    sorted_fields(current_fields)
}

fn apply_operation_to_catalog(
    mut current_fields: Vec<DatasetFieldDraft>,
    operation: DatasetOperationDraft,
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
) -> Vec<DatasetFieldDraft> {
    match operation.kind {
        DatasetOperationDraftKind::JoinSource
        | DatasetOperationDraftKind::UnionSource
        | DatasetOperationDraftKind::UnionAllSource => {
            let source_fields = source_catalog_for_operation(&operation, forms, rendered_forms);
            extend_catalog(&mut current_fields, source_fields);
            current_fields
        }
        DatasetOperationDraftKind::Projection => {
            apply_projection_to_catalog(current_fields, &operation.projection_fields)
        }
        DatasetOperationDraftKind::Aggregation => {
            fields_after_aggregation(current_fields, operation.aggregation.clone())
        }
        DatasetOperationDraftKind::CalculatedFields => {
            fields_after_calculations(current_fields, operation.calculated_fields.clone())
        }
        DatasetOperationDraftKind::Filter => current_fields,
    }
}

fn source_catalog_for_initial(
    initial_source: &DatasetSourceDraft,
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
) -> Vec<DatasetFieldDraft> {
    source_fields_for_source(initial_source, forms, rendered_forms)
}

fn source_catalog_for_operation(
    operation: &DatasetOperationDraft,
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
) -> Vec<DatasetFieldDraft> {
    operation
        .source
        .as_ref()
        .map(|source| source_fields_for_source(source, forms, rendered_forms))
        .unwrap_or_default()
}

fn extend_catalog(
    current_fields: &mut Vec<DatasetFieldDraft>,
    next_fields: Vec<DatasetFieldDraft>,
) {
    let mut seen = current_fields
        .iter()
        .map(|field| field.key.clone())
        .collect::<BTreeSet<_>>();
    for field in next_fields {
        if seen.insert(field.key.clone()) {
            current_fields.push(field);
        }
    }
}

fn apply_projection_to_catalog(
    current_fields: Vec<DatasetFieldDraft>,
    selected_fields: &[DatasetFieldDraft],
) -> Vec<DatasetFieldDraft> {
    current_fields
        .into_iter()
        .filter_map(|field| {
            selected_fields
                .iter()
                .find(|selected| {
                    selected.key == field.key || projection_input_key(selected) == field.key
                })
                .cloned()
        })
        .collect()
}

fn projection_input_key(field: &DatasetFieldDraft) -> String {
    if field.source_alias.trim().is_empty() || field.source_field_key.trim().is_empty() {
        return field.key.clone();
    }
    let canonical_key = canonical_field_key(&field.source_alias, &field.source_field_key);
    if canonical_key.starts_with("aggregation__")
        || canonical_key.starts_with("calculated__")
        || canonical_key.starts_with("projection__")
    {
        field.key.clone()
    } else {
        canonical_key
    }
}

fn source_fields_for_source(
    source: &DatasetSourceDraft,
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
) -> Vec<DatasetFieldDraft> {
    let sources = vec![source.clone()];
    let fields = source_field_options(&sources, forms, rendered_forms, &source.source_alias)
        .into_iter()
        .map(|field| DatasetFieldDraft {
            key: canonical_field_key(&source.source_alias, &field.key),
            label: field.label,
            source_alias: source.source_alias.clone(),
            source_field_key: field.key,
            field_type: field.field_type,
        })
        .collect::<Vec<_>>();
    sorted_fields(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::datasets::types::{
        DatasetAggregationMetricDraft, DatasetCalculationFunctionDraft, DatasetFormVersionOption,
        DatasetRenderedField, DatasetRenderedSection, DatasetRowPickerDraft,
        DatasetRowPickerSortDraft,
    };

    fn form_option(form_id: &str, version_id: &str) -> DatasetFormOption {
        DatasetFormOption {
            id: form_id.into(),
            name: form_id.into(),
            versions: vec![DatasetFormVersionOption {
                id: version_id.into(),
                version_label: Some("1.0.0".into()),
                status: "published".into(),
                version_major: Some(1),
                field_count: 2,
            }],
        }
    }

    fn rendered_form(
        form_id: &str,
        version_id: &str,
        fields: Vec<(&str, &str, &str)>,
    ) -> DatasetRenderedForm {
        DatasetRenderedForm {
            form_version_id: version_id.into(),
            form_id: form_id.into(),
            form_name: form_id.into(),
            sections: vec![DatasetRenderedSection {
                fields: fields
                    .into_iter()
                    .map(|(key, label, field_type)| DatasetRenderedField {
                        key: key.into(),
                        label: label.into(),
                        field_type: field_type.into(),
                        value_options: Vec::new(),
                    })
                    .collect(),
            }],
        }
    }

    fn source(alias: &str, form_id: &str, version_id: &str) -> DatasetSourceDraft {
        DatasetSourceDraft {
            input_kind: "form".into(),
            source_alias: alias.into(),
            form_id: form_id.into(),
            form_version_id: version_id.into(),
            dataset_id: String::new(),
            dataset_revision_id: String::new(),
        }
    }

    fn catalog_fixture() -> (
        DatasetSourceDraft,
        Vec<DatasetFormOption>,
        BTreeMap<String, DatasetRenderedForm>,
    ) {
        let initial_source = source("program", "program_form", "program_v1");
        let forms = vec![
            form_option("program_form", "program_v1"),
            form_option("partner_form", "partner_v1"),
        ];
        let rendered_forms = BTreeMap::from([
            (
                "program_v1".into(),
                rendered_form(
                    "program_form",
                    "program_v1",
                    vec![
                        ("participant_target", "Participant Target", "number"),
                        ("submission_status", "Submission Status", "single_choice"),
                    ],
                ),
            ),
            (
                "partner_v1".into(),
                rendered_form(
                    "partner_form",
                    "partner_v1",
                    vec![
                        ("contact_name", "Contact Name", "text"),
                        ("compliance_confirmed", "Compliance Confirmed", "boolean"),
                    ],
                ),
            ),
        ]);

        (initial_source, forms, rendered_forms)
    }

    fn field_keys(fields: Vec<DatasetFieldDraft>) -> Vec<String> {
        fields.into_iter().map(|field| field.key).collect()
    }

    #[test]
    fn incomplete_initial_source_has_empty_catalog() {
        let (_, forms, rendered_forms) = catalog_fixture();
        let initial_source = DatasetSourceDraft::default();
        let projection = DatasetOperationDraft::new(1, DatasetOperationDraftKind::Projection);

        let before_projection =
            catalog_before_operation_id(initial_source, forms, rendered_forms, vec![projection], 1);

        assert!(before_projection.is_empty());
    }

    fn projection_field(key: &str, source_field_key: &str, field_type: &str) -> DatasetFieldDraft {
        DatasetFieldDraft {
            key: key.into(),
            label: key.into(),
            source_alias: "program".into(),
            source_field_key: source_field_key.into(),
            field_type: field_type.into(),
        }
    }

    fn metric(id: u64, key: &str, source_field_key: &str) -> DatasetAggregationMetricDraft {
        DatasetAggregationMetricDraft {
            id,
            key: key.into(),
            label: key.into(),
            function: "sum".into(),
            source_field_key: source_field_key.into(),
        }
    }

    fn filter(id: u64, field_key: &str, value: &str) -> DatasetRowFilterDraft {
        DatasetRowFilterDraft {
            id,
            field_key: field_key.into(),
            operator: "equals".into(),
            value: value.into(),
            value_mode: "value".into(),
            value_field_key: String::new(),
        }
    }

    fn calculated_field(
        id: u64,
        key: &str,
        base_field_key: &str,
        function: &str,
        argument: &str,
    ) -> DatasetCalculatedFieldDraft {
        DatasetCalculatedFieldDraft {
            id,
            key: key.into(),
            label: key.into(),
            base_field_key: base_field_key.into(),
            functions: vec![DatasetCalculationFunctionDraft {
                id,
                function: function.into(),
                argument: argument.into(),
                argument_mode: "value".into(),
                argument_field_key: String::new(),
            }],
        }
    }

    #[test]
    fn catalog_before_each_operation_is_folded_from_prior_operations() {
        let (initial_source, forms, rendered_forms) = catalog_fixture();

        let mut projection = DatasetOperationDraft::new(1, DatasetOperationDraftKind::Projection);
        projection.projection_fields = vec![DatasetFieldDraft {
            key: "target".into(),
            label: "Target".into(),
            source_alias: "program".into(),
            source_field_key: "participant_target".into(),
            field_type: "number".into(),
        }];

        let mut union = DatasetOperationDraft::new(2, DatasetOperationDraftKind::UnionSource);
        union.source = Some(source("partner", "partner_form", "partner_v1"));

        let filter = DatasetOperationDraft::new(3, DatasetOperationDraftKind::Filter);
        let operations = vec![projection, union, filter];

        let before_projection = field_keys(catalog_before_operation_id(
            initial_source.clone(),
            forms.clone(),
            rendered_forms.clone(),
            operations.clone(),
            1,
        ));
        assert!(before_projection.contains(&"program__participant_target".into()));
        assert!(!before_projection.contains(&"partner__contact_name".into()));

        let before_union = field_keys(catalog_before_operation_id(
            initial_source.clone(),
            forms.clone(),
            rendered_forms.clone(),
            operations.clone(),
            2,
        ));
        assert_eq!(before_union, vec!["target".to_string()]);

        let before_filter = field_keys(catalog_before_operation_id(
            initial_source,
            forms,
            rendered_forms,
            operations,
            3,
        ));
        assert!(before_filter.contains(&"target".into()));
        assert!(before_filter.contains(&"partner__contact_name".into()));
        assert!(!before_filter.contains(&"program__submission_status".into()));
    }

    #[test]
    fn final_catalog_includes_calculated_outputs_for_restriction_options() {
        let (initial_source, forms, rendered_forms) = catalog_fixture();
        let mut calculation =
            DatasetOperationDraft::new(1, DatasetOperationDraftKind::CalculatedFields);
        calculation.calculated_fields = vec![DatasetCalculatedFieldDraft {
            id: 1,
            key: "is_confidential".into(),
            label: "Is Confidential".into(),
            base_field_key: "program__participant_target".into(),
            functions: vec![DatasetCalculationFunctionDraft {
                id: 1,
                function: "greater_than_or_equal".into(),
                argument: "100".into(),
                argument_mode: "value".into(),
                argument_field_key: String::new(),
            }],
        }];

        let keys = field_keys(catalog_after_operations(
            initial_source,
            forms,
            rendered_forms,
            vec![calculation],
        ));
        assert!(keys.contains(&"is_confidential".into()));
    }

    #[test]
    fn repeated_aggregation_operations_keep_independent_configs() {
        let mut first = DatasetOperationDraft::new(1, DatasetOperationDraftKind::Aggregation);
        first.aggregation = DatasetAggregationDraft {
            enabled: true,
            group_fields: vec!["program__submission_status".into()],
            metrics: vec![metric(1, "target_sum", "program__participant_target")],
            row_picker: None,
        };

        let mut second = DatasetOperationDraft::new(2, DatasetOperationDraftKind::Aggregation);
        second.aggregation = DatasetAggregationDraft {
            enabled: true,
            group_fields: vec!["aggregation__target_sum".into()],
            metrics: vec![metric(2, "target_sum_again", "aggregation__target_sum")],
            row_picker: Some(DatasetRowPickerDraft {
                sort_fields: vec![DatasetRowPickerSortDraft {
                    field_key: "aggregation__target_sum".into(),
                }],
                direction: "desc".into(),
            }),
        };

        let operations = vec![first, second];

        assert_eq!(
            operations[0].aggregation.group_fields,
            vec!["program__submission_status"]
        );
        assert_eq!(operations[0].aggregation.metrics[0].key, "target_sum");
        assert!(operations[0].aggregation.row_picker.is_none());

        assert_eq!(
            operations[1].aggregation.group_fields,
            vec!["aggregation__target_sum"]
        );
        assert_eq!(operations[1].aggregation.metrics[0].key, "target_sum_again");
        assert_eq!(
            operations[1]
                .aggregation
                .row_picker
                .as_ref()
                .and_then(|row_picker| row_picker.sort_fields.first())
                .map(|sort| sort.field_key.as_str()),
            Some("aggregation__target_sum")
        );
    }

    #[test]
    fn repeated_filter_operations_keep_independent_rows() {
        let mut first = DatasetOperationDraft::new(1, DatasetOperationDraftKind::Filter);
        first.row_filters = vec![filter(1, "program__submission_status", "submitted")];

        let mut second = DatasetOperationDraft::new(2, DatasetOperationDraftKind::Filter);
        second.row_filters = vec![filter(2, "program__participant_target", "100")];
        second.row_filters[0].operator = "greater_than_or_equal".into();

        let operations = vec![first, second];

        assert_eq!(
            operations[0].row_filters[0].field_key,
            "program__submission_status"
        );
        assert_eq!(operations[0].row_filters[0].operator, "equals");
        assert_eq!(operations[0].row_filters[0].value, "submitted");

        assert_eq!(
            operations[1].row_filters[0].field_key,
            "program__participant_target"
        );
        assert_eq!(
            operations[1].row_filters[0].operator,
            "greater_than_or_equal"
        );
        assert_eq!(operations[1].row_filters[0].value, "100");
    }

    #[test]
    fn repeated_calculated_field_operations_keep_independent_chains() {
        let mut first = DatasetOperationDraft::new(1, DatasetOperationDraftKind::CalculatedFields);
        first.calculated_fields = vec![calculated_field(
            1,
            "status_upper",
            "program__submission_status",
            "uppercase",
            "",
        )];

        let mut second = DatasetOperationDraft::new(2, DatasetOperationDraftKind::CalculatedFields);
        second.calculated_fields = vec![calculated_field(
            2,
            "target_flag",
            "program__participant_target",
            "greater_than_or_equal",
            "100",
        )];

        let operations = vec![first, second];

        assert_eq!(operations[0].calculated_fields[0].key, "status_upper");
        assert_eq!(
            operations[0].calculated_fields[0].functions[0].function,
            "uppercase"
        );
        assert_eq!(operations[0].calculated_fields[0].functions[0].argument, "");

        assert_eq!(operations[1].calculated_fields[0].key, "target_flag");
        assert_eq!(
            operations[1].calculated_fields[0].functions[0].function,
            "greater_than_or_equal"
        );
        assert_eq!(
            operations[1].calculated_fields[0].functions[0].argument,
            "100"
        );
    }

    #[test]
    fn reordering_operations_moves_complete_operation_config() {
        let mut projection = DatasetOperationDraft::new(1, DatasetOperationDraftKind::Projection);
        projection.projection_fields =
            vec![projection_field("target", "participant_target", "number")];

        let mut filter_operation = DatasetOperationDraft::new(2, DatasetOperationDraftKind::Filter);
        filter_operation.row_filters = vec![filter(1, "target", "100")];

        let mut calculation =
            DatasetOperationDraft::new(3, DatasetOperationDraftKind::CalculatedFields);
        calculation.calculated_fields =
            vec![calculated_field(1, "target_label", "target", "to_text", "")];

        let mut operations = vec![projection, filter_operation, calculation];

        move_operation_by_delta_in_place(&mut operations, 3, -1);
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.id)
                .collect::<Vec<_>>(),
            vec![1, 3, 2]
        );
        assert_eq!(operations[1].calculated_fields[0].key, "target_label");
        assert_eq!(operations[2].row_filters[0].field_key, "target");

        move_operation_before_in_place(&mut operations, 2, 1);
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.id)
                .collect::<Vec<_>>(),
            vec![2, 1, 3]
        );
        assert_eq!(operations[0].row_filters[0].value, "100");
        assert_eq!(operations[1].projection_fields[0].key, "target");
        assert_eq!(operations[2].calculated_fields[0].base_field_key, "target");
    }

    #[test]
    fn insert_anchors_resolve_against_current_operation_order() {
        let mut operations = vec![
            DatasetOperationDraft::new(1, DatasetOperationDraftKind::Projection),
            DatasetOperationDraft::new(2, DatasetOperationDraftKind::Filter),
            DatasetOperationDraft::new(3, DatasetOperationDraftKind::CalculatedFields),
        ];

        assert_eq!(
            insert_index_for_anchor(&operations, DraggablePanelListAnchor::Start),
            0
        );
        assert_eq!(
            insert_index_for_anchor(&operations, DraggablePanelListAnchor::After("1".into())),
            1
        );
        assert_eq!(
            insert_index_for_anchor(&operations, DraggablePanelListAnchor::After("2".into())),
            2
        );
        assert_eq!(
            insert_index_for_anchor(&operations, DraggablePanelListAnchor::After("3".into())),
            3
        );

        move_operation_to_index_in_place(&mut operations, 3, 0);
        assert_eq!(
            operations
                .iter()
                .map(|operation| operation.id)
                .collect::<Vec<_>>(),
            vec![3, 1, 2]
        );

        assert_eq!(
            insert_index_for_anchor(&operations, DraggablePanelListAnchor::Start),
            0
        );
        assert_eq!(
            insert_index_for_anchor(&operations, DraggablePanelListAnchor::After("3".into())),
            1
        );
        assert_eq!(
            insert_index_for_anchor(&operations, DraggablePanelListAnchor::After("1".into())),
            2
        );
        assert_eq!(
            insert_index_for_anchor(&operations, DraggablePanelListAnchor::After("2".into())),
            3
        );
        assert_eq!(
            insert_index_for_anchor(&operations, DraggablePanelListAnchor::After("999".into())),
            operations.len()
        );
    }

    #[test]
    fn expanding_and_collapsing_panels_only_mutates_ui_state() {
        let mut expanded_ids = BTreeSet::new();
        let operations = vec![
            DatasetOperationDraft::new(1, DatasetOperationDraftKind::Aggregation),
            DatasetOperationDraft::new(2, DatasetOperationDraftKind::Filter),
        ];
        let operations_before = operation_signature(&operations);

        toggle_operation_expansion(&mut expanded_ids, 1);
        assert!(expanded_ids.contains(&1));
        assert_eq!(operation_signature(&operations), operations_before);

        toggle_operation_expansion(&mut expanded_ids, 1);
        assert!(!expanded_ids.contains(&1));
        assert_eq!(operation_signature(&operations), operations_before);
    }

    fn operation_signature(
        operations: &[DatasetOperationDraft],
    ) -> Vec<(u64, DatasetOperationDraftKind)> {
        operations
            .iter()
            .map(|operation| (operation.id, operation.kind))
            .collect()
    }
}
