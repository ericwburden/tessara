use super::*;


#[component]
pub(crate) fn FormsList(
    forms: Vec<FormSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    node_filter_query: RwSignal<String>,
    selected_node_id: RwSignal<Option<String>>,
    status_options: Vec<String>,
    node_filter_options: Vec<FormNodeFilterOption>,
) -> impl IntoView {
    let mut table_forms = forms.clone();
    table_forms.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then(left.id.cmp(&right.id))
    });
    let card_forms = table_forms.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_forms.len();
    let page_count = move || {
        if total_count == 0 {
            1
        } else {
            ((total_count + page_size.get() - 1) / page_size.get()).max(1)
        }
    };
    let current_page = move || page_index.get().min(page_count() - 1);
    let page_start = move || {
        if total_count == 0 {
            0
        } else {
            current_page() * page_size.get()
        }
    };
    let page_end = move || (page_start() + page_size.get()).min(total_count);
    let page_summary = move || {
        if total_count == 0 {
            "No forms to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} forms",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };
    let attached_nodes_sheet = RwSignal::new(None::<FormsAttachedNodesSheetData>);

    view! {
        <div class="forms-list forms-list-responsive-table">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search forms"</span>
                        <input
                            type="search"
                            placeholder="Search forms"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Form name"</th>
                        <th scope="col">
                            <div class="data-table-filter">
                                <span>"Attached To"</span>
                                <FormsNodeLineageFilter
                                    options=node_filter_options
                                    selected_node_id
                                    query=node_filter_query
                                />
                            </div>
                        </th>
                        <th class="data-table__cell--center" scope="col">"Active version"</th>
                        <th class="data-table__cell--center" scope="col">
                            <FilterHeader
                                label="Status"
                                all_label="All statuses"
                                filter=status_filter
                                options=status_options
                            />
                        </th>
                        <th class="data-table__cell--center" scope="col">"Fields"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || if table_forms.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="5">"No Forms to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        table_forms
                            .iter()
                            .skip(page_start())
                            .take(page_size.get())
                            .cloned()
                            .map(|form| {
                                let href = format!("/forms/{}", form.id);
                                let active_version = active_form_version(&form);
                                let status = active_version
                                    .map(|version| version.status.as_str())
                                    .unwrap_or("none");
                                let name = form.name.clone();
                                let attached_nodes = form_attached_nodes(active_version);
                                let attached_nodes_form_name = name.clone();
                                let version_label = form_version_label(active_version);
                                let status_label = form_status_label(active_version);
                                let field_count = form_field_count_label(active_version);
                                view! {
                                    <tr>
                                        <th scope="row">
                                            <a class="data-table__primary-link" href=href.clone()>{name}</a>
                                        </th>
                                        <td>
                                            <FormsAttachedNodesList
                                                nodes=attached_nodes
                                                form_name=attached_nodes_form_name
                                                form_href=href
                                                sheet=attached_nodes_sheet
                                            />
                                        </td>
                                        <td class="data-table__cell--center">{version_label}</td>
                                        <td class="data-table__cell--center"><span class=status_badge_class(status)>{status_label}</span></td>
                                        <td class="data-table__cell--center">{field_count}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </tbody>
                </DataTable>
                <div class="directory-table-pagination" aria-label="Forms table pagination">
                    <p>{move || page_summary()}</p>
                    <div class="directory-table-pagination__actions">
                        <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                            <span>"Rows"</span>
                            <select
                                prop:value=move || page_size.get().to_string()
                                on:change=move |event| {
                                    if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                        page_size.set(size);
                                        page_index.set(0);
                                    }
                                }
                            >
                                <option value="10">"10"</option>
                                <option value="25">"25"</option>
                                <option value="50">"50"</option>
                            </select>
                        </label>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || current_page() == 0
                            on:click=move |_| {
                                page_index.update(|page| *page = page.saturating_sub(1));
                            }
                        >
                            "Previous"
                        </button>
                        <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || { current_page() + 1 >= page_count() }
                            on:click=move |_| {
                                let last_page = page_count().saturating_sub(1);
                                page_index.update(|page| *page = (*page + 1).min(last_page));
                            }
                        >
                            "Next"
                        </button>
                    </div>
                </div>
            </div>
            <div class="forms-list-mobile-cards">
                {move || if card_forms.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Forms to Display"</p> }.into_any()
                } else {
                    card_forms
                        .iter()
                        .skip(page_start())
                        .take(page_size.get())
                        .cloned()
                        .map(|form| {
                            let href = format!("/forms/{}", form.id);
                            let active_version = active_form_version(&form);
                            let status = active_version
                                .map(|version| version.status.as_str())
                                .unwrap_or("none");
                            let name = form.name.clone();
                            let attached_nodes = form_attached_nodes(active_version);
                            let attached_nodes_form_name = name.clone();
                            let version_label = form_version_label(active_version);
                            let status_label = form_status_label(active_version);
                            let field_count = form_field_count_label(active_version);
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div>
                                            <h3><a href=href.clone()>{name}</a></h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Attached To"</dt>
                                            <dd>
                                                <FormsAttachedNodesList
                                                    nodes=attached_nodes
                                                    form_name=attached_nodes_form_name
                                                    form_href=href
                                                    sheet=attached_nodes_sheet
                                                />
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Active version"</dt>
                                            <dd>{version_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(status)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Fields"</dt>
                                            <dd>{field_count}</dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
            <FormsAttachedNodesSheet detail=attached_nodes_sheet/>
        </div>
    }
}

#[component]
fn FormsAttachedNodesList(
    nodes: Vec<FormAttachmentLink>,
    form_name: String,
    form_href: String,
    sheet: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let nodes_for_sheet = nodes.clone();
    let form_name_for_sheet = form_name.clone();
    let form_href_for_sheet = form_href.clone();

    view! {
        <div class="forms-attached-list">
            {if total_nodes == 0 {
                view! { <p>"Not attached"</p> }.into_any()
            } else if total_nodes > 0 {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View attached organization nodes for {form_name_for_sheet}")
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(FormsAttachedNodesSheetData {
                                form_name: form_name_for_sheet.clone(),
                                form_href: form_href_for_sheet.clone(),
                                nodes: nodes_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{node_count_label(total_nodes)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            } else {
                empty_view()
            }}
        </div>
    }
}

#[component]
fn FormsAttachedNodesSheet(detail: RwSignal<Option<FormsAttachedNodesSheetData>>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = move |_| {
        detail.set(None);
        search.set(String::new());
    };
    let filtered_nodes = move || {
        let query = search.get().trim().to_lowercase();
        detail
            .get()
            .map(|data| {
                data.nodes
                    .into_iter()
                    .filter(|node| {
                        query.is_empty()
                            || node.label.to_lowercase().contains(&query)
                            || node.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Attached organization nodes">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close attached nodes" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Attached organization nodes">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.form_href aria-label="Open form detail" title="Open form detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(|| empty_view())
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close attached nodes" title="Close attached nodes" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.nodes.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Attached Nodes"</p>
                                            <h2>{data.form_name}</h2>
                                            <span class="forms-attached-sheet__count">{format!("{total} nodes")}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search class="searchable-data-table__control-icon"/>
                                                <span class="sr-only">"Search attached nodes"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search attached nodes"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let nodes = filtered_nodes();
                                                    if nodes.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Attached Nodes to Display"</p> }.into_any()
                                                    } else {
                                                        nodes
                                                            .into_iter()
                                                            .map(|node| {
                                                                let node_title = node.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=node.href title=node_title>
                                                                        <span>{node.label}</span>
                                                                        <small>{node.title}</small>
                                                                    </a>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </section>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(|| empty_view())
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
fn FormBuilderSection(
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

#[component]
fn FormBuilderGrid(
    section_id: usize,
    layout: Memo<FormBuilderSectionLayout>,
    default_column_width: Memo<i32>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
    next_builder_field_id: RwSignal<usize>,
) -> impl IntoView {
    let grid_rows = Memo::new(move |_| layout.get().row_count);
    let grid_cells = Memo::new(move |_| {
        let row_count = grid_rows.get();
        (1..=row_count)
            .flat_map(|row| {
                (1..=FORM_BUILDER_COLUMN_COUNT)
                    .map(move |column| FormBuilderGridCell { row, column })
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div
            data-section-id=section_id
            class=move || {
                if dragged_builder_field.get().is_some() {
                    "form-builder-layout-grid is-dragging"
                } else {
                    "form-builder-layout-grid"
                }
            }
            style=move || {
                let row_count = grid_rows.get();
                format!(
                    "--form-builder-rows: {}; --form-builder-max-height: {}px;",
                    row_count,
                    row_count * 80,
                )
            }
            on:dragenter=move |event| {
                let Some(field_id) = dragged_builder_field.get_untracked() else {
                    return;
                };
                let Some((row, column, target_id)) = form_builder_grid_cell_from_drag_event(&event) else {
                    return;
                };
                event.prevent_default();
                schedule_form_builder_drag_preview(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                    FormBuilderDragPreview {
                        field_id,
                        section_id,
                        row,
                        column,
                    },
                    target_id,
                );
            }
            on:dragover=move |event| {
                let Some(field_id) = dragged_builder_field.get_untracked() else {
                    return;
                };
                event.prevent_default();
                let Some((row, column, target_id)) =
                    form_builder_grid_cell_from_pointer(&event, grid_rows.get_untracked())
                else {
                    return;
                };
                schedule_form_builder_drag_preview(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                    FormBuilderDragPreview {
                        field_id,
                        section_id,
                        row,
                        column,
                    },
                    target_id,
                );
            }
            on:drop=move |event| {
                event.prevent_default();
                if let Some(field_id) = dragged_builder_field.get_untracked() {
                    if let Some((row, column, _)) =
                        form_builder_grid_cell_from_pointer(&event, grid_rows.get_untracked())
                    {
                        set_form_builder_drag_preview(
                            builder_drag_preview,
                            FormBuilderDragPreview {
                                field_id,
                                section_id,
                                row,
                                column,
                            },
                        );
                    }
                }
                commit_form_builder_drag_preview(
                    builder_fields,
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                    dragged_builder_field,
                    suppress_builder_field_click,
                );
            }
            on:mouseleave=move |_| {
                if dragged_builder_field.get_untracked().is_some() {
                    clear_form_builder_drag_intent(
                        builder_drag_preview,
                        pending_builder_drag_preview,
                        builder_drag_preview_timeout,
                    );
                }
            }
            on:click=move |event| {
                let Some((row, column)) = form_builder_add_tile_from_click_event(&event) else {
                    return;
                };
                event.prevent_default();
                if suppress_builder_field_click.get_untracked().is_some() {
                    suppress_builder_field_click.set(None);
                    return;
                }
                let fields = builder_fields.get_untracked();
                let occupied_cells = {
                    let section_fields = form_builder_section_fields(section_id, &fields);
                    form_builder_occupancy_map(&section_fields)
                };
                if occupied_cells.contains(&(row, column)) {
                    return;
                }
                let field_id = next_builder_field_id.get_untracked();
                next_builder_field_id.set(field_id + 1);
                let default_width = default_column_width
                    .get_untracked()
                    .clamp(1, FORM_BUILDER_COLUMN_COUNT);
                let available_width =
                    max_form_builder_new_field_width_at(section_id, row, column, &fields);
                let new_field = blank_form_builder_field_at(
                    field_id,
                    section_id,
                    row,
                    column,
                    default_width.min(available_width),
                );
                builder_fields.update(|fields| fields.push(new_field));
                active_builder_field.set(Some(field_id));
            }
        >
            <div class="form-builder-grid-cells">
                <For
                    each=move || grid_cells.get()
                    key=|cell| (cell.row, cell.column)
                    children=move |cell| {
                        let cell_label =
                            format!("Add field at row {}, column {}", cell.row, cell.column);
                        view! {
                            <div
                                id=format!("form-builder-section-{section_id}-cell-r{}-c{}", cell.row, cell.column)
                                class="form-builder-grid-cell form-builder-grid-cell--empty"
                                data-row=cell.row
                                data-column=cell.column
                                data-empty=true
                                aria-label=cell_label
                                style=format!("grid-column: {}; grid-row: {};", cell.column, cell.row)
                            ></div>
                        }
                    }
                />
            </div>
            <For
                each=move || layout.get().fields
                key=|field| field.id
                children=move |field| {
                    view! {
                        <FormBuilderGridTile
                            field_id=field.id
                            section_id=section_id
                            builder_fields=builder_fields
                            active_builder_field=active_builder_field
                            dragged_builder_field=dragged_builder_field
                            builder_drag_preview=builder_drag_preview
                            pending_builder_drag_preview=pending_builder_drag_preview
                            builder_drag_preview_timeout=builder_drag_preview_timeout
                            suppress_builder_field_click=suppress_builder_field_click
                        />
                    }
                }
            />
        </div>
    }
}

#[component]
fn FormBuilderGridTile(
    field_id: usize,
    section_id: usize,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) -> impl IntoView {
    let field = Memo::new(move |_| {
        builder_fields
            .get()
            .into_iter()
            .find(|field| field.id == field_id)
    });
    let display_label = move || {
        field
            .get()
            .map(|field| {
                if field.label.trim().is_empty() {
                    form_builder_field_default_label(&field.field_type, field_id)
                } else {
                    field.label
                }
            })
            .unwrap_or_else(|| format!("Field {field_id}"))
    };
    view! {
        <div
            class=move || {
                let width_class = field
                    .get()
                    .map(|field| {
                        if field.grid_width <= 2 {
                            " form-builder-grid-tile--icon-only"
                        } else if field.grid_width >= 4 {
                            " form-builder-grid-tile--mobile-label"
                        } else {
                            ""
                        }
                    })
                    .unwrap_or("");
                if dragged_builder_field.get() == Some(field_id) {
                    format!(
                        "form-builder-grid-tile form-builder-grid-tile--field form-builder-grid-field form-builder-grid-field--summary is-dragging{width_class}"
                    )
                } else {
                    format!(
                        "form-builder-grid-tile form-builder-grid-tile--field form-builder-grid-field form-builder-grid-field--summary{width_class}"
                    )
                }
            }
            draggable="true"
            style=move || {
                field
                    .get()
                    .map(|field| form_builder_grid_tile_style(&field))
                    .unwrap_or_else(|| "display: none;".into())
            }
            on:dragstart=move |_event: leptos::ev::DragEvent| {
                #[cfg(feature = "hydrate")]
                {
                    if let Some(target) = _event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    {
                        if target.closest(".form-builder-resize-handle").ok().flatten().is_some() {
                            _event.prevent_default();
                            return;
                        }
                    }
                }
                clear_form_builder_drag_intent(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                );
                dragged_builder_field.set(Some(field_id));
            }
            on:dragenter=move |event| {
                if let Some(dragged_field_id) = dragged_builder_field.get_untracked() {
                    event.prevent_default();
                    let Some(field) = field.get_untracked() else {
                        return;
                    };
                    schedule_form_builder_drag_preview(
                        builder_drag_preview,
                        pending_builder_drag_preview,
                        builder_drag_preview_timeout,
                        FormBuilderDragPreview {
                            field_id: dragged_field_id,
                            section_id,
                            row: field.grid_row.max(1),
                            column: field.grid_column.max(1),
                        },
                        format!(
                            "form-builder-section-{section_id}-cell-r{}-c{}",
                            field.grid_row.max(1),
                            field.grid_column.max(1),
                        ),
                    );
                }
            }
            on:click=move |_| {
                if suppress_builder_field_click.get_untracked() == Some(field_id) {
                    suppress_builder_field_click.set(None);
                } else {
                    dragged_builder_field.set(None);
                    active_builder_field.set(Some(field_id));
                }
            }
            on:dragend=move |_| {
                clear_form_builder_drag_intent(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                );
                dragged_builder_field.set(None);
            }
        >
            <button
                class="form-builder-grid-field__summary"
                type="button"
                title=display_label
                aria-label=move || format!("Configure {}", display_label())
                on:click=move |event| {
                    event.stop_propagation();
                    if suppress_builder_field_click.get_untracked() == Some(field_id) {
                        suppress_builder_field_click.set(None);
                    } else {
                        dragged_builder_field.set(None);
                        active_builder_field.set(Some(field_id));
                    }
                }
            >
                <span class="form-builder-field-type-icon">
                    {move || {
                        field
                            .get()
                            .map(|field| form_builder_field_type_icon(&field.field_type))
                            .unwrap_or_else(|| form_builder_field_type_icon("text"))
                    }}
                </span>
                <div>
                    <h5>{display_label}</h5>
                </div>
            </button>
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
        </div>
    }
}

#[component]
fn FieldConfigSheet(
    active_builder_field: RwSignal<Option<usize>>,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
) -> impl IntoView {
    view! {
        <Portal>
            <Show when=move || active_builder_field.get().is_some()>
                {move || {
                    let close = move |_| active_builder_field.set(None);
                    let field_id = active_builder_field.get().unwrap_or_default();
                    let field = builder_fields
                        .get()
                        .into_iter()
                        .find(|field| field.id == field_id);
                    field
                        .map(|field| {
                            let display_label = if field.label.trim().is_empty() {
                                format!("Field {}", field.id)
                            } else {
                                field.label.clone()
                            };
                            let section = builder_sections
                                .get()
                                .into_iter()
                                .find(|section| section.id == field.section_id)
                                .unwrap_or_else(|| blank_form_builder_section(field.section_id));
                            let all_fields = builder_fields.get();
                            let layout = form_builder_section_layout(&section, &all_fields);
                            let section_column_count = layout.column_count;
                            let section_fields_for_bounds = layout.fields;
                            let row_max = layout.row_count;
                            let width_max = max_form_builder_field_width(
                                &field,
                                &section_fields_for_bounds,
                            );
                            let height_max = max_form_builder_field_height(
                                &field,
                                &section_fields_for_bounds,
                            );
                            view! {
                                <section class="sheet-overlay form-field-config-overlay" aria-label="Field configuration">
                                    <button class="sheet-overlay__scrim" type="button" aria-label="Close field configuration" on:click=close></button>
                                    <aside class="sheet-panel blurred-surface form-field-config-sheet" role="dialog" aria-modal="true" aria-label="Field configuration">
                                        <div class="sheet-panel__actions">
                                            <button
                                                class="icon-button icon-button--danger"
                                                type="button"
                                                aria-label="Delete field"
                                                title="Delete field"
                                                on:click=move |_| {
                                                    builder_fields.update(|fields| {
                                                        fields.retain(|field| field.id != field_id);
                                                    });
                                                    active_builder_field.set(None);
                                                }
                                            >
                                                <Trash2/>
                                            </button>
                                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close field configuration" title="Close field configuration" on:click=close>
                                                <X/>
                                            </button>
                                        </div>

                                        <header class="sheet-panel__header">
                                            <p>"Field Configuration"</p>
                                            <h2>{display_label}</h2>
                                        </header>

                                        <section class="sheet-panel__section">
                                            <div class="form-grid form-builder-field-sheet-controls">
                                                <label class="form-field" for=format!("sheet-form-field-label-{field_id}")>
                                                    <span>"Field Label"</span>
                                                    <input
                                                        id=format!("sheet-form-field-label-{field_id}")
                                                        type="text"
                                                        autocomplete="off"
                                                        prop:value=field.label.clone()
                                                        on:input=move |event| {
                                                            let next_label = event_target_value(&event);
                                                            builder_fields.update(|fields| {
                                                                if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
                                                                    field.label = next_label.clone();
                                                                    if !field.key_was_edited {
                                                                        field.key = slug_from_label(&next_label);
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    />
                                                </label>

                                                <label class="form-field" for=format!("sheet-form-field-key-{field_id}")>
                                                    <span>"Field Key"</span>
                                                    <input
                                                        id=format!("sheet-form-field-key-{field_id}")
                                                        type="text"
                                                        autocomplete="off"
                                                        prop:value=field.key.clone()
                                                        on:input=move |event| {
                                                            let next_key = slug_from_label(&event_target_value(&event));
                                                            builder_fields.update(|fields| {
                                                                if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
                                                                    field.key = next_key.clone();
                                                                    field.key_was_edited = true;
                                                                }
                                                            });
                                                        }
                                                    />
                                                </label>

                                                <label class="form-field" for=format!("sheet-form-field-type-{field_id}")>
                                                    <span>"Field Type"</span>
                                                    <select
                                                        id=format!("sheet-form-field-type-{field_id}")
                                                        prop:value=field.field_type.clone()
                                                        on:change=move |event| {
                                                            let next_type = event_target_value(&event);
                                                            builder_fields.update(|fields| {
                                                                if let Some(position) = fields.iter().position(|field| field.id == field_id) {
                                                                    let mut next_field = fields[position].clone();
                                                                    next_field.field_type = next_type.clone();
                                                                    if next_type == "static_text" {
                                                                        next_field.required = false;
                                                                        if next_field.label.trim().is_empty() {
                                                                            next_field.label = form_builder_field_default_label(&next_type, next_field.id);
                                                                        }
                                                                        if next_field.key.trim().is_empty() || !next_field.key_was_edited {
                                                                            next_field.key = slug_from_label(&next_field.label);
                                                                        }
                                                                        let mut candidate = next_field.clone();
                                                                        candidate.grid_width = candidate.grid_width.max(4);
                                                                        if candidate.grid_column + candidate.grid_width - 1 <= FORM_BUILDER_COLUMN_COUNT
                                                                            && !form_builder_field_has_collision(&candidate, fields)
                                                                        {
                                                                            next_field.grid_width = candidate.grid_width;
                                                                        }
                                                                    }
                                                                    fields[position] = next_field;
                                                                }
                                                            });
                                                        }
                                                    >
                                                        <option value="static_text">"Static text"</option>
                                                        <option value="text">"Text"</option>
                                                        <option value="number">"Number"</option>
                                                        <option value="date">"Date"</option>
                                                        <option value="boolean">"Checkbox"</option>
                                                        <option value="single_choice">"Single choice"</option>
                                                        <option value="multi_choice">"Multi choice"</option>
                                                    </select>
                                                </label>

                                                <label class="form-field form-field--checkbox form-builder-field__required">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=field.required
                                                        disabled=field.field_type == "static_text"
                                                        on:change=move |event| {
                                                            let checked = event_target_checked(&event);
                                                            builder_fields.update(|fields| {
                                                                if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
                                                                    if field.field_type != "static_text" {
                                                                        field.required = checked;
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    />
                                                    <span>"Required"</span>
                                                </label>

                                                {["Row", "Column", "Width", "Height"]
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(index, label)| {
                                                        let value = match index {
                                                            0 => field.grid_row,
                                                            1 => field.grid_column,
                                                            2 => field.grid_width,
                                                            _ => field.grid_height,
                                                        };
                                                        let max_value = match index {
                                                            0 => row_max,
                                                            1 => (section_column_count - field.grid_width.max(1) + 1)
                                                                .clamp(1, section_column_count.max(1)),
                                                            2 => width_max,
                                                            _ => height_max,
                                                        }
                                                        .max(1);
                                                        let value = value.clamp(1, max_value);
                                                        let valid_values = valid_form_builder_layout_values(
                                                            &field,
                                                            &section_fields_for_bounds,
                                                            index,
                                                            max_value,
                                                        );
                                                        let control_id = format!("sheet-form-field-layout-{index}-{field_id}");
                                                        let input_id = control_id.clone();
                                                        view! {
                                                            <label class="form-field" for=control_id>
                                                                <span>{label}</span>
                                                                <select
                                                                    id=input_id
                                                                    on:change=move |event| {
                                                                        let value = event_target_value(&event)
                                                                            .parse::<i32>()
                                                                            .unwrap_or(1)
                                                                            .clamp(1, max_value);
                                                                        builder_fields.update(|fields| {
                                                                            if let Some(position) = fields.iter().position(|field| field.id == field_id) {
                                                                                let candidate = form_builder_layout_candidate(
                                                                                    &fields[position],
                                                                                    index,
                                                                                    value,
                                                                                );

                                                                                if !form_builder_field_has_collision(&candidate, fields) {
                                                                                    fields[position] = candidate;
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                >
                                                                    {valid_values
                                                                        .into_iter()
                                                                        .map(|option_value| {
                                                                            view! {
                                                                                <option
                                                                                    value=option_value.to_string()
                                                                                    selected=option_value == value
                                                                                >
                                                                                    {option_value}
                                                                                </option>
                                                                            }
                                                                        })
                                                                        .collect_view()}
                                                                </select>
                                                            </label>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </div>
                                        </section>
                                    </aside>
                                </section>
                            }
                            .into_any()
                        })
                        .unwrap_or_else(|| empty_view())
                }}
            </Show>
        </Portal>
    }
}

#[component]
pub fn FormsNewPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let existing_forms = RwSignal::new(Vec::<FormSummary>::new());
    let name = RwSignal::new(String::new());
    let workflow_node_type_id = RwSignal::new(String::new());
    let builder_sections = RwSignal::new(vec![blank_form_builder_section(1)]);
    let active_builder_section = RwSignal::new("1".to_string());
    let next_builder_section_id = RwSignal::new(2usize);
    let builder_fields = RwSignal::new(Vec::<FormBuilderFieldDraft>::new());
    let active_builder_field = RwSignal::new(None::<usize>);
    let dragged_builder_field = RwSignal::new(None::<usize>);
    let builder_drag_preview = RwSignal::new(None::<FormBuilderDragPreview>);
    let pending_builder_drag_preview = RwSignal::new(None::<FormBuilderDragPreview>);
    let builder_drag_preview_timeout = RwSignal::new(None::<i32>);
    let suppress_builder_field_click = RwSignal::new(None::<usize>);
    let next_builder_field_id = RwSignal::new(1usize);
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let builder_field_count = Memo::new(move |_| builder_fields.get().len());

    Effect::new(move |_| {
        load_form_create_options(node_types, existing_forms, is_loading, message);
    });

    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <AppShell active_route="forms" title="Forms">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/forms">"Forms"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem>
                    <BreadcrumbPage>"Create Form"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>
            <section class="route-panel forms-page form-editor-panel">
                <PageHeader title="Create Form"/>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading form options"</h3>
                                <p>"Fetching available organization scopes."</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="form-create-workspace">
                            <form
                                class="native-form form-create-form"
                                on:submit=move |event| {
                                    event.prevent_default();
                                    submit_create_form(
                                        name,
                                        workflow_node_type_id,
                                        builder_sections,
                                        builder_fields,
                                        existing_forms,
                                        is_saving,
                                        message,
                                        false,
                                    );
                                }
                            >
                                <div class="form-grid">
                                    <label class="form-field form-field--wide" for="form-name">
                                        <span>"Form Name"</span>
                                        <input
                                            id="form-name"
                                            type="text"
                                            autocomplete="off"
                                            prop:value=move || name.get()
                                            on:input=move |event| name.set(event_target_value(&event))
                                            required
                                        />
                                    </label>

                                    <label class="form-field" for="form-scope-node-type">
                                        <span>"Scope"</span>
                                        <select
                                            id="form-scope-node-type"
                                            prop:value=move || workflow_node_type_id.get()
                                            on:change=move |event| workflow_node_type_id.set(event_target_value(&event))
                                        >
                                            <option value="">"No scope"</option>
                                            {move || {
                                                let mut options = node_types.get();
                                                options.sort_by(|left, right| {
                                                    left.singular_label
                                                        .cmp(&right.singular_label)
                                                        .then(left.name.cmp(&right.name))
                                                });
                                                options
                                                    .into_iter()
                                                    .map(|node_type| {
                                                        view! {
                                                            <option value=node_type.id>{node_type.singular_label}</option>
                                                        }
                                                    })
                                                    .collect_view()
                                            }}
                                        </select>
                                    </label>

                                </div>

                                <section class="form-section">
                                    <h3>"Initial Version"</h3>
                                    <InfoListTable>
                                        <InfoRow label="Status" value="Draft"/>
                                        <tr>
                                            <th scope="row">"Fields"</th>
                                            <td>
                                                {move || builder_field_count.get().to_string()}
                                            </td>
                                        </tr>
                                    </InfoListTable>
                                </section>

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
                                                    let section_id = next_builder_section_id.get_untracked();
                                                    next_builder_section_id.set(section_id + 1);
                                                    builder_sections.update(|sections| {
                                                        let mut section = blank_form_builder_section(section_id);
                                                        section.position = (sections.len() + 1) as i32;
                                                        sections.push(section);
                                                    });
                                                    active_builder_section.set(section_id.to_string());
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
                                                    .filter(|section| {
                                                        active_builder_section.get() == section.id.to_string()
                                                    })
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
                                </section>
                                {move || message.get().map(|message| view! {
                                    <p class="form-message" role="status">{message}</p>
                                })}

                                <div class="form-actions">
                                    <Button label="Cancel" href="/forms"/>
                                    <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                        {move || if is_saving.get() { "Saving..." } else { "Save as Draft" }}
                                    </button>
                                    <button
                                        class="button"
                                        type="button"
                                        disabled=move || !can_submit()
                                        on:click=move |_| {
                                            submit_create_form(
                                                name,
                                                workflow_node_type_id,
                                                builder_sections,
                                                builder_fields,
                                                existing_forms,
                                                is_saving,
                                                message,
                                                true,
                                            );
                                        }
                                    >
                                        {move || if is_saving.get() { "Publishing..." } else { "Create and Publish" }}
                                    </button>
                                </div>
                            </form>
                            <FieldConfigSheet
                                active_builder_field=active_builder_field
                                builder_sections=builder_sections
                                builder_fields=builder_fields
                            />
                            </div>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn FormsDetailPage() -> impl IntoView {
    let params = require_route_params::<FormRouteParams>();
    let form_id = params.form_id;
    let detail = RwSignal::new(None::<FormDefinition>);
    let rendered_form = RwSignal::new(None::<RenderedForm>);
    let is_loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_form_detail(form_id.clone(), detail, rendered_form, is_loading, error);
    });

    view! {
        <AppShell active_route="forms" title="Forms">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/forms">"Forms"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {move || {
                    detail.get().map(|form| {
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbPage>{form.name}</BreadcrumbPage>
                            </BreadcrumbItem>
                        }
                    })
                }}
                {move || {
                    if detail.get().is_none() {
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbPage>"Detail"</BreadcrumbPage>
                            </BreadcrumbItem>
                        }
                        .into_any()
                    } else {
                        empty_view()
                    }
                }}
            </Breadcrumb>

            <section class="route-panel forms-page form-detail-page">
                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading form"</h3>
                                <p>"Fetching form details."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Form detail unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(form) = detail.get() {
                        let edit_href = format!("/forms/{}/edit", form.id);
                        let create_workflow_href = format!("/workflows/new?form_id={}", form.id);
                        let assign_form_href = form
                            .workflows
                            .iter()
                            .find(|workflow| {
                                workflow.source == "generated_form"
                                    && workflow.current_version_label.is_some()
                            })
                            .map(|workflow| format!("/workflows/assignments?workflow_id={}", workflow.id));
                        view! {
                            <PageHeader title="Form Detail">
                                <a class="button button--secondary" href=create_workflow_href>"Create Workflow"</a>
                                {assign_form_href
                                    .map(|href| {
                                        view! { <a class="button button--secondary" href=href>"Assign Form"</a> }
                                    })
                                    .into_view()}
                                <a class="button" href=edit_href>"Edit Form"</a>
                            </PageHeader>
                            <FormDetailContent form rendered_form=rendered_form.get()/>
                        }
                        .into_any()
                    } else {
                        view! {
                            <EmptyState
                                title="Form detail unavailable"
                                message="The selected form could not be loaded."
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
fn FormDetailContent(form: FormDefinition, rendered_form: Option<RenderedForm>) -> impl IntoView {
    let fields_expanded = RwSignal::new(false);
    let active_version = active_form_definition_version(&form).cloned();
    let attached_nodes = form_attached_nodes(active_version.as_ref());
    let active_status = active_version
        .as_ref()
        .map(|version| version.status.clone())
        .unwrap_or_else(|| "none".to_string());
    let active_version_label = form_version_label(active_version.as_ref());
    let active_status_label = form_status_label(active_version.as_ref());
    let active_field_count = form_field_count_label(active_version.as_ref());
    let fields_toggle_count = active_field_count.clone();
    let published_at = active_version
        .as_ref()
        .and_then(|version| version.published_at.clone());
    let form_name = form.name.clone();
    let form_slug = form.slug.clone();
    let form_scope = form_definition_scope_label(&form);
    let version_count = form.versions.len().to_string();
    let versions = form.versions.clone();
    let workflows = form.workflows.clone();
    let dataset_sources = form.dataset_sources.clone();

    view! {
        <div class="organization-detail-content form-detail-content">
            <header class="organization-detail-content__header">
                <p>"Form Detail"</p>
                <h2>{form_name}</h2>
            </header>

            <div class="organization-detail-content__grid">
                <section class="organization-detail-card">
                    <h3>"Details"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Slug"</th>
                            <td>{form_slug}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Scope"</th>
                            <td>{form_scope}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Versions"</th>
                            <td>{version_count}</td>
                        </tr>
                    </InfoListTable>
                </section>

                <section class="organization-detail-card">
                    <h3>"Active Version"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Version"</th>
                            <td>{active_version_label}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Status"</th>
                            <td><span class=status_badge_class(&active_status)>{active_status_label}</span></td>
                        </tr>
                        <tr>
                            <th scope="row">"Fields"</th>
                            <td>{active_field_count}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Published"</th>
                            <td>
                                {published_at
                                    .map(|value| view! { <Timestamp value/> }.into_any())
                                    .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                            </td>
                        </tr>
                    </InfoListTable>
                </section>

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Fields"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || fields_expanded.get().to_string()
                            on:click=move |_| fields_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if fields_expanded.get() {
                                    "Hide Fields".to_string()
                                } else {
                                    format!("Show {fields_toggle_count} Fields")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if fields_expanded.get() {
                            view! { <RenderedFormSections rendered_form=rendered_form.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>

                <section class="organization-detail-card organization-detail-card--wide">
                    <h3>"Versions"</h3>
                    <FormVersionsTable versions=versions/>
                </section>

                <section class="organization-detail-card organization-detail-card--wide">
                    <h3>"Related Work"</h3>
                    <FormRelatedLinks
                        attached_nodes=attached_nodes
                        workflows=workflows
                        dataset_sources=dataset_sources
                    />
                </section>
            </div>
        </div>
    }
}

#[component]
fn FormAttachedNodesRelatedTable(nodes: Vec<FormAttachmentLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let nodes_for_filter = nodes;
    let filtered_nodes = Memo::new(move |_| {
        let query = search.get();
        nodes_for_filter
            .iter()
            .filter(|node| text_matches(&query, &[&node.label, &node.title]))
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_nodes.get().len());

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search attached nodes" placeholder="Search attached nodes" search>
                <thead>
                    <tr>
                        <th scope="col">"Node"</th>
                        <th scope="col">"Context"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_nodes.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="2">"No Attached Nodes to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            let total_count = rows.len();
                            let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                            rows
                                .iter()
                                .skip(start)
                                .take(page_size.get())
                                .cloned()
                                .map(|node| {
                                    let title = node.title.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=node.href title=title>{node.label}</a>
                                            </th>
                                            <td>{node.title}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </SearchableDataTable>
            <RelatedWorkPaginationFooter
                aria_label="Attached nodes table pagination"
                label="attached nodes"
                total_count=total_count
                page_size=page_size
                page_index=page_index
            />
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_nodes.get();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Attached Nodes to Display"</p> }.into_any()
                    } else {
                        let total_count = rows.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        rows
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|node| {
                                let title = node.title.clone();
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=node.href title=title>{node.label}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Context"</dt>
                                                <dd>{node.title}</dd>
                                            </div>
                                        </dl>
                                    </article>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn FormVersionsTable(versions: Vec<FormVersionSummary>) -> impl IntoView {
    const DEFAULT_VISIBLE_FORM_VERSIONS: usize = 5;

    let visible_count = RwSignal::new(DEFAULT_VISIBLE_FORM_VERSIONS);
    let mut sorted_versions = versions;
    sorted_versions.sort_by(|left, right| {
        form_version_desc_sort_key(right).cmp(&form_version_desc_sort_key(left))
    });
    let table_versions = sorted_versions.clone();
    let card_versions = sorted_versions.clone();
    let version_count = sorted_versions.len();

    view! {
        <div class="forms-list-responsive-table">
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Version"</th>
                        <th scope="col">"Status"</th>
                        <th scope="col">"Compatibility"</th>
                        <th scope="col">"Published"</th>
                        <th class="data-table__cell--center" scope="col">"Fields"</th>
                    </tr>
                </thead>
                <tbody>
                    {if table_versions.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="5">"No Versions to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        table_versions
                            .iter()
                            .take(visible_count.get())
                            .cloned()
                            .map(|version| {
                                let status = version.status.clone();
                                let published_at = version.published_at.clone();
                                view! {
                                    <tr>
                                        <th scope="row">{form_version_sort_label(&version)}</th>
                                        <td><span class=status_badge_class(&status)>{sentence_label(&status)}</span></td>
                                        <td>{nonempty_text(version.compatibility_group_name.as_deref(), "-")}</td>
                                        <td>
                                            {published_at
                                                .map(|value| view! { <Timestamp value/> }.into_any())
                                                .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                                        </td>
                                        <td class="data-table__cell--center">{version.field_count.to_string()}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </tbody>
            </DataTable>
            <div class="forms-list-mobile-cards">
                {if card_versions.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Versions to Display"</p> }.into_any()
                } else {
                    card_versions
                        .iter()
                        .take(visible_count.get())
                        .cloned()
                        .map(|version| {
                            let status = version.status.clone();
                            let published_at = version.published_at.clone();
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <h3>{form_version_sort_label(&version)}</h3>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(&status)>{sentence_label(&status)}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Compatibility"</dt>
                                            <dd>{nonempty_text(version.compatibility_group_name.as_deref(), "-")}</dd>
                                        </div>
                                        <div>
                                            <dt>"Published"</dt>
                                            <dd>
                                                {published_at
                                                    .map(|value| view! { <Timestamp value/> }.into_any())
                                                    .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Fields"</dt>
                                            <dd>{version.field_count.to_string()}</dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
            {move || {
                if version_count > visible_count.get() {
                    let remaining = version_count.saturating_sub(visible_count.get());
                    view! {
                        <button
                            class="button button--compact button--secondary form-versions-load-more"
                            type="button"
                            on:click=move |_| {
                                visible_count.update(|count| {
                                    *count = (*count + DEFAULT_VISIBLE_FORM_VERSIONS).min(version_count);
                                });
                            }
                        >
                            {format!("Load More ({remaining} older)")}
                        </button>
                    }
                    .into_any()
                } else {
                    empty_view()
                }
            }}
        </div>
    }
}

#[component]
fn RenderedFormSections(rendered_form: Option<RenderedForm>) -> impl IntoView {
    view! {
        <div class="form-detail-sections">
            {if let Some(rendered_form) = rendered_form {
                if rendered_form.sections.is_empty() {
                    view! { <p class="related-work-mobile-empty">"No Fields to Display"</p> }.into_any()
                } else {
                    rendered_form
                        .sections
                        .into_iter()
                        .map(|section| {
                            view! {
                                <article class="form-detail-section">
                                    <header>
                                        <div>
                                            <h4>{section.title}</h4>
                                            {if section.description.trim().is_empty() {
                                                empty_view()
                                            } else {
                                                view! { <p>{section.description}</p> }.into_any()
                                            }}
                                        </div>
                                    </header>
                                    <DataTable>
                                        <thead>
                                            <tr>
                                                <th scope="col">"Field"</th>
                                                <th scope="col">"Key"</th>
                                                <th scope="col">"Type"</th>
                                                <th scope="col">"Required"</th>
                                                <th scope="col">"Layout"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {if section.fields.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td class="data-table__empty" colspan="5">"No Fields to Display"</td>
                                                    </tr>
                                                }
                                                .into_any()
                                            } else {
                                                section
                                                    .fields
                                                    .into_iter()
                                                    .map(|field| {
                                                        let layout_label = rendered_field_layout_label(&field);
                                                        view! {
                                                            <tr>
                                                                <th scope="row">{field.label}</th>
                                                                <td>{field.key}</td>
                                                                <td>{rendered_field_type_label(&field.field_type)}</td>
                                                                <td>{if field.required { "Yes" } else { "No" }}</td>
                                                                <td>{layout_label}</td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect_view()
                                                    .into_any()
                                            }}
                                        </tbody>
                                    </DataTable>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }
            } else {
                view! { <p class="related-work-mobile-empty">"No Fields to Display"</p> }.into_any()
            }}
        </div>
    }
}

#[component]
fn FormRelatedLinks(
    attached_nodes: Vec<FormAttachmentLink>,
    workflows: Vec<FormWorkflowLink>,
    dataset_sources: Vec<FormDatasetSourceLink>,
) -> impl IntoView {
    let active_tab = RwSignal::new("attached".to_string());
    let attached_count = attached_nodes.len();
    let workflows_count = workflows.len();
    let dataset_sources_count = dataset_sources.len();

    view! {
        <div class="related-work-summary form-detail-related">
            <Tabs active=active_tab>
                <TabsList>
                    <TabsTrigger active=active_tab value="attached">
                        {format!("Attached To ({attached_count})")}
                    </TabsTrigger>
                    <TabsTrigger active=active_tab value="workflows">
                        {format!("Workflows ({workflows_count})")}
                    </TabsTrigger>
                    <TabsTrigger active=active_tab value="dataset-sources">
                        {format!("Dataset Sources ({dataset_sources_count})")}
                    </TabsTrigger>
                </TabsList>
                <TabsContent active=active_tab value="attached">
                    <FormAttachedNodesRelatedTable nodes=attached_nodes/>
                </TabsContent>
                <TabsContent active=active_tab value="workflows">
                    <FormRelatedWorkflowsTable workflows=workflows/>
                </TabsContent>
                <TabsContent active=active_tab value="dataset-sources">
                    <FormRelatedDatasetSourcesTable dataset_sources=dataset_sources/>
                </TabsContent>
            </Tabs>
        </div>
    }
}

#[component]
fn FormRelatedWorkflowsTable(workflows: Vec<FormWorkflowLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let workflows_for_filter = workflows;
    let filtered_workflows = Memo::new(move |_| {
        let query = search.get();
        workflows_for_filter
            .iter()
            .filter(|workflow| {
                text_matches(
                    &query,
                    &[
                        &workflow.name,
                        &workflow.slug,
                        workflow
                            .current_version_label
                            .as_deref()
                            .unwrap_or_default(),
                        workflow.current_status.as_deref().unwrap_or_default(),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_workflows.get().len());

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search workflows" placeholder="Search related workflows" search>
                <thead>
                    <tr>
                        <th scope="col">"Workflow"</th>
                        <th scope="col">"Revision"</th>
                        <th scope="col">"Status"</th>
                        <th class="data-table__cell--center" scope="col">"Assignments"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_workflows.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="4">"No Related Workflows to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            let total_count = rows.len();
                            let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                            rows
                                .iter()
                                .skip(start)
                                .take(page_size.get())
                                .cloned()
                                .map(|workflow| {
                                    let href = format!("/workflows/{}", workflow.id);
                                    let status = workflow.current_status.clone().unwrap_or_else(|| "none".to_string());
                                    let workflow_source = workflow.source.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=href>{workflow.name}</a>
                                                <WorkflowSourceMarker source=workflow_source/>
                                                <small class="workflow-assignment-step-meta">{workflow.slug}</small>
                                            </th>
                                            <td>{workflow_revision_label_from_option(workflow.current_version_label)}</td>
                                            <td><span class=status_badge_class(&status)>{sentence_label(&status)}</span></td>
                                            <td class="data-table__cell--center">{workflow.assignment_count.to_string()}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </SearchableDataTable>
            <RelatedWorkPaginationFooter
                aria_label="Related workflows table pagination"
                label="related workflows"
                total_count=total_count
                page_size=page_size
                page_index=page_index
            />
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_workflows.get();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Workflows to Display"</p> }.into_any()
                    } else {
                        let total_count = rows.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        rows
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|workflow| {
                                let href = format!("/workflows/{}", workflow.id);
                                let status = workflow.current_status.clone().unwrap_or_else(|| "none".to_string());
                                let workflow_source = workflow.source.clone();
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4>
                                                <a href=href>{workflow.name}</a>
                                                <WorkflowSourceMarker source=workflow_source/>
                                            </h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Slug"</dt>
                                                <dd>{workflow.slug}</dd>
                                            </div>
                                            <div>
                                                <dt>"Revision"</dt>
                                                <dd>{workflow_revision_label_from_option(workflow.current_version_label)}</dd>
                                            </div>
                                            <div>
                                                <dt>"Status"</dt>
                                                <dd><span class=status_badge_class(&status)>{sentence_label(&status)}</span></dd>
                                            </div>
                                            <div>
                                                <dt>"Assignments"</dt>
                                                <dd>{workflow.assignment_count.to_string()}</dd>
                                            </div>
                                        </dl>
                                    </article>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn FormRelatedDatasetSourcesTable(dataset_sources: Vec<FormDatasetSourceLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let sources_for_filter = dataset_sources;
    let filtered_sources = Memo::new(move |_| {
        let query = search.get();
        sources_for_filter
            .iter()
            .filter(|source| {
                text_matches(
                    &query,
                    &[
                        &source.dataset_name,
                        &source.source_alias,
                        &source.selection_rule,
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_sources.get().len());

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search dataset sources" placeholder="Search related dataset sources" search>
                <thead>
                    <tr>
                        <th scope="col">"Dataset"</th>
                        <th scope="col">"Alias"</th>
                        <th scope="col">"Selection rule"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_sources.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="3">"No Related Dataset Sources to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            let total_count = rows.len();
                            let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                            rows
                                .iter()
                                .skip(start)
                                .take(page_size.get())
                                .cloned()
                                .map(|source| {
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=format!("/datasets/{}", source.dataset_id)>{source.dataset_name}</a>
                                            </th>
                                            <td>{source.source_alias}</td>
                                            <td>{sentence_label(&source.selection_rule)}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </SearchableDataTable>
            <RelatedWorkPaginationFooter
                aria_label="Related dataset sources table pagination"
                label="related dataset sources"
                total_count=total_count
                page_size=page_size
                page_index=page_index
            />
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_sources.get();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Dataset Sources to Display"</p> }.into_any()
                    } else {
                        let total_count = rows.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        rows
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|source| {
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=format!("/datasets/{}", source.dataset_id)>{source.dataset_name}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Alias"</dt>
                                                <dd>{source.source_alias}</dd>
                                            </div>
                                            <div>
                                                <dt>"Selection rule"</dt>
                                                <dd>{sentence_label(&source.selection_rule)}</dd>
                                            </div>
                                        </dl>
                                    </article>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
pub fn FormsEditPage() -> impl IntoView {
    let params = require_route_params::<FormRouteParams>();
    let form_id = params.form_id;
    let form_id_for_load = form_id.clone();
    let form_id_for_submit = form_id.clone();
    let cancel_href = format!("/forms/{form_id}");
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let existing_forms = RwSignal::new(Vec::<FormSummary>::new());
    let detail = RwSignal::new(None::<FormDefinition>);
    let rendered_form = RwSignal::new(None::<RenderedForm>);
    let edit_version_id = RwSignal::new(None::<String>);
    let edit_version_status = RwSignal::new(None::<String>);
    let name = RwSignal::new(String::new());
    let workflow_node_type_id = RwSignal::new(String::new());
    let builder_sections = RwSignal::new(vec![blank_form_builder_section(1)]);
    let active_builder_section = RwSignal::new("1".to_string());
    let next_builder_section_id = RwSignal::new(2usize);
    let builder_fields = RwSignal::new(Vec::<FormBuilderFieldDraft>::new());
    let active_builder_field = RwSignal::new(None::<usize>);
    let dragged_builder_field = RwSignal::new(None::<usize>);
    let builder_drag_preview = RwSignal::new(None::<FormBuilderDragPreview>);
    let pending_builder_drag_preview = RwSignal::new(None::<FormBuilderDragPreview>);
    let builder_drag_preview_timeout = RwSignal::new(None::<i32>);
    let suppress_builder_field_click = RwSignal::new(None::<usize>);
    let next_builder_field_id = RwSignal::new(1usize);
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let builder_field_count = Memo::new(move |_| builder_fields.get().len());

    Effect::new(move |_| {
        load_form_edit_options(
            form_id_for_load.clone(),
            node_types,
            existing_forms,
            detail,
            rendered_form,
            edit_version_id,
            edit_version_status,
            name,
            workflow_node_type_id,
            builder_sections,
            builder_fields,
            active_builder_section,
            next_builder_section_id,
            next_builder_field_id,
            is_loading,
            message,
        );
    });

    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <AppShell active_route="forms" title="Forms">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/forms">"Forms"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {move || {
                    detail
                        .get()
                        .map(|form| {
                            let href = format!("/forms/{}", form.id);
                            view! {
                                <BreadcrumbItem>
                                    <BreadcrumbLink href=href>{form.name}</BreadcrumbLink>
                                </BreadcrumbItem>
                                <BreadcrumbSeparator/>
                            }
                            .into_any()
                        })
                        .unwrap_or_else(|| empty_view())
                }}
                <BreadcrumbItem>
                    <BreadcrumbPage>"Edit Form"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>

            <section class="route-panel forms-page form-editor-panel">
                <PageHeader title="Edit Form"/>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading form"</h3>
                                <p>"Fetching form definition and editable version."</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        let form_id_for_submit = form_id_for_submit.clone();
                        let form_id_for_draft_submit = form_id_for_submit.clone();
                        let form_id_for_publish_submit = form_id_for_submit.clone();
                        view! {
                            <div class="form-create-workspace">
                                <form
                                    class="native-form form-create-form"
                                    on:submit=move |event| {
                                        event.prevent_default();
                                        submit_update_form(
                                            form_id_for_draft_submit.clone(),
                                            name,
                                            workflow_node_type_id,
                                            builder_sections,
                                            builder_fields,
                                            existing_forms,
                                            edit_version_id,
                                            edit_version_status,
                                            rendered_form,
                                            is_saving,
                                            message,
                                            false,
                                        );
                                    }
                                >
                                    <div class="form-grid">
                                        <label class="form-field form-field--wide" for="form-name">
                                            <span>"Form Name"</span>
                                            <input
                                                id="form-name"
                                                type="text"
                                                autocomplete="off"
                                                prop:value=move || name.get()
                                                on:input=move |event| name.set(event_target_value(&event))
                                                required
                                            />
                                        </label>

                                        <label class="form-field" for="form-scope-node-type">
                                            <span>"Scope"</span>
                                            <select
                                                id="form-scope-node-type"
                                                prop:value=move || workflow_node_type_id.get()
                                                on:change=move |event| workflow_node_type_id.set(event_target_value(&event))
                                            >
                                                <option value="">"No scope"</option>
                                                {move || {
                                                    let mut options = node_types.get();
                                                    options.sort_by(|left, right| {
                                                        left.singular_label
                                                            .cmp(&right.singular_label)
                                                            .then(left.name.cmp(&right.name))
                                                    });
                                                    options
                                                        .into_iter()
                                                        .map(|node_type| {
                                                            view! {
                                                                <option value=node_type.id>{node_type.singular_label}</option>
                                                            }
                                                        })
                                                        .collect_view()
                                                }}
                                            </select>
                                        </label>
                                    </div>

                                    <section class="form-section">
                                        <h3>"Editable Version"</h3>
                                        <InfoListTable>
                                            <tr>
                                                <th scope="row">"Status"</th>
                                                <td>
                                                    {move || {
                                                        edit_version_status
                                                            .get()
                                                            .map(|status| {
                                                                view! {
                                                                    <span class=status_badge_class(&status)>
                                                                        {sentence_label(&status)}
                                                                    </span>
                                                                }
                                                                .into_any()
                                                            })
                                                            .unwrap_or_else(|| view! { <span>"Draft"</span> }.into_any())
                                                    }}
                                                </td>
                                            </tr>
                                            <tr>
                                                <th scope="row">"Fields"</th>
                                                <td>
                                                    {move || builder_field_count.get().to_string()}
                                                </td>
                                            </tr>
                                        </InfoListTable>
                                    </section>

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
                                                        let section_id = next_builder_section_id.get_untracked();
                                                        next_builder_section_id.set(section_id + 1);
                                                        builder_sections.update(|sections| {
                                                            let mut section = blank_form_builder_section(section_id);
                                                            section.position = (sections.len() + 1) as i32;
                                                            sections.push(section);
                                                        });
                                                        active_builder_section.set(section_id.to_string());
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
                                                        .filter(|section| {
                                                            active_builder_section.get() == section.id.to_string()
                                                        })
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
                                    </section>
                                    {move || message.get().map(|message| view! {
                                        <p class="form-message" role="status">{message}</p>
                                    })}

                                    <div class="form-actions">
                                        <a class="button" href=cancel_href.clone()>"Cancel"</a>
                                        <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                            {move || if is_saving.get() { "Saving..." } else { "Save as Draft" }}
                                        </button>
                                        <button
                                            class="button"
                                            type="button"
                                            disabled=move || !can_submit()
                                            on:click=move |_| {
                                                submit_update_form(
                                                    form_id_for_publish_submit.clone(),
                                                    name,
                                                    workflow_node_type_id,
                                                    builder_sections,
                                                    builder_fields,
                                                    existing_forms,
                                                    edit_version_id,
                                                    edit_version_status,
                                                    rendered_form,
                                                    is_saving,
                                                    message,
                                                    true,
                                                );
                                            }
                                        >
                                            {move || if is_saving.get() { "Publishing..." } else { "Save and Publish" }}
                                        </button>
                                    </div>
                                </form>
                                <FieldConfigSheet
                                    active_builder_field=active_builder_field
                                    builder_sections=builder_sections
                                    builder_fields=builder_fields
                                />
                            </div>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}


