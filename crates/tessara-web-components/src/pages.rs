//! Route-level page composition for the Components feature.

#[cfg(feature = "hydrate")]
use super::types::{CreateComponentRequest, CreateComponentVersionRequest, UpdateComponentRequest};
use icons::{ListFilter, Search, X};
use leptos::prelude::*;
use serde_json::{Value, json};
use tessara_web_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, DataTable,
    EmptyState, PageHeader, TableFilterHeader, TablePaginationFooter,
};

#[cfg(feature = "hydrate")]
use super::api;
use super::types::{
    ComponentDefinition, ComponentSummary, ComponentTable, ComponentTableColumn,
    ComponentValidationFinding, ComponentVersionSummary, DatasetFieldDefinition, DatasetSummary,
};

#[component]
pub fn ComponentsIndexContent() -> impl IntoView {
    let components = RwSignal::new(Vec::<ComponentSummary>::new());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_components(components, is_loading, load_error);
    });

    view! {
        <section class="route-panel components-page">
            <div class="page-header">
                <div></div>
                <div class="page-header__actions">
                    <a class="button" href="/components/new">"Create Component"</a>
                </div>
            </div>
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading components" message="Fetching visible components."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Components unavailable" message=message/> }.into_any()
                } else if components.get().is_empty() {
                    view! { <EmptyState title="No visible components" message="No components are visible for the current account."/> }.into_any()
                } else {
                    view! { <ComponentsTable components=components.get()/> }.into_any()
                }
            }}
        </section>
    }
}

#[component]
pub fn ComponentDetailContent(component_ref: String) -> impl IntoView {
    let component = RwSignal::new(None::<ComponentDefinition>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new({
        let component_ref = component_ref.clone();
        move |_| load_component(component_ref.clone(), component, is_loading, load_error)
    });

    view! {
        <section class="route-panel components-page">
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading component" message="Fetching component detail."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Component unavailable" message=message/> }.into_any()
                } else if let Some(component) = component.get() {
                    let edit_href = format!("/components/{}/edit", component.slug);
                    let publish_href = format!("/components/{}/publish", component.slug);
                    let view_href = format!("/components/{}/view", component.slug);
                    view! {
                        <ComponentsBreadcrumb current=component.name.clone()/>
                        <PageHeader title=component.name.clone()>
                            <a class="button button--secondary" href=edit_href>"Edit"</a>
                            <a class="button button--secondary" href=publish_href>"Publish"</a>
                            <a class="button" href=view_href>"View"</a>
                        </PageHeader>
                        <section class="route-panel__section">
                            <DataTable>
                                <thead>
                                    <tr>
                                        <th>"Version"</th>
                                        <th>"Status"</th>
                                        <th>"Kind"</th>
                                        <th>"Dataset Version"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {component.versions.into_iter().map(|version| view! {
                                        <tr>
                                            <td>{version.version_label}</td>
                                            <td>{component_status_label(&version.status)}</td>
                                            <td>{component_type_label(&version.component_type)}</td>
                                            <td>{format!("v{}", version.dataset_version_major)}</td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </DataTable>
                        </section>
                    }.into_any()
                } else {
                    view! { <EmptyState title="Component unavailable" message="Component detail could not be loaded."/> }.into_any()
                }
            }}
        </section>
    }
}

#[component]
pub fn ComponentEditorContent(component_ref: Option<String>) -> impl IntoView {
    let title = if component_ref.is_some() {
        "Edit Component"
    } else {
        "Create Component"
    };
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let validation_findings = RwSignal::new(Vec::<ComponentValidationFinding>::new());
    let datasets = RwSignal::new(Vec::<DatasetSummary>::new());
    let dataset_error = RwSignal::new(None::<String>);
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let dataset_id = RwSignal::new(String::new());
    let dataset_major = RwSignal::new(String::from("1"));
    let component_type = RwSignal::new(String::from("table"));
    let columns = RwSignal::new(String::new());
    let sort_field = RwSignal::new(String::new());
    let sort_direction = RwSignal::new(String::from("asc"));
    let page_size = RwSignal::new(String::from("50"));
    let editing_component_id = RwSignal::new(None::<String>);
    let editing_version_id = RwSignal::new(None::<String>);

    let selected_fields = Memo::new(move |_| {
        datasets
            .get()
            .into_iter()
            .find(|dataset| dataset.id == dataset_id.get())
            .map(|dataset| dataset.output_fields)
            .unwrap_or_default()
    });
    let selected_dataset = Memo::new(move |_| {
        datasets
            .get()
            .into_iter()
            .find(|dataset| dataset.id == dataset_id.get())
    });

    Effect::new(move |_| load_datasets(datasets, dataset_error));
    Effect::new({
        let component_ref = component_ref.clone();
        move |_| {
            if let Some(component_ref) = component_ref.clone() {
                load_component_for_edit(
                    component_ref,
                    editing_component_id,
                    editing_version_id,
                    name,
                    slug,
                    description,
                    dataset_id,
                    dataset_major,
                    component_type,
                    columns,
                    sort_field,
                    sort_direction,
                    page_size,
                    error,
                );
            }
        }
    });

    view! {
        <section class="route-panel components-page">
            <ComponentsBreadcrumb current=title/>
            <PageHeader title/>
            <form class="route-panel__section form-grid" on:submit=move |event| {
                event.prevent_default();
                create_component_from_form(
                    editing_component_id.get_untracked(),
                    editing_version_id.get_untracked(),
                    ComponentFormValues {
                        name: name.get_untracked(),
                        slug: slug.get_untracked(),
                        description: description.get_untracked(),
                        dataset_id: dataset_id.get_untracked(),
                        dataset_major: dataset_major.get_untracked(),
                        columns: columns.get_untracked(),
                        sort_field: sort_field.get_untracked(),
                        sort_direction: sort_direction.get_untracked(),
                        page_size: page_size.get_untracked(),
                    },
                    message,
                    error,
                    validation_findings,
                );
            }>
                <label class="form-field">
                    <span>"Name"</span>
                    <input prop:value=move || name.get() on:input=move |event| name.set(event_target_value(&event))/>
                </label>
                <label class="form-field">
                    <span>"Slug"</span>
                    <input prop:value=move || slug.get() on:input=move |event| slug.set(event_target_value(&event))/>
                </label>
                <label class="form-field form-field--wide">
                    <span>"Description"</span>
                    <textarea prop:value=move || description.get() on:input=move |event| description.set(event_target_value(&event))></textarea>
                </label>
                <label class="form-field form-field--wide">
                    <span>"Dataset Version"</span>
                    <select prop:value=move || selected_dataset_major_value(&dataset_id.get(), &dataset_major.get()) on:change=move |event| {
                        let value = event_target_value(&event);
                        if let Some((selected_dataset_id, selected_major)) = value.split_once('|') {
                            dataset_id.set(selected_dataset_id.to_string());
                            dataset_major.set(selected_major.to_string());
                            if let Some(dataset) = datasets
                                .get_untracked()
                                .into_iter()
                                .find(|dataset| dataset.id == selected_dataset_id)
                            {
                                let keys = dataset
                                    .output_fields
                                    .iter()
                                    .map(|field| field.key.clone())
                                    .take(6)
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                columns.set(keys);
                                sort_field.set(
                                    dataset
                                        .output_fields
                                        .first()
                                        .map(|field| field.key.clone())
                                        .unwrap_or_default(),
                                );
                            }
                        }
                    }>
                        <option value="">"Select a Dataset version"</option>
                        {move || datasets.get().into_iter().flat_map(|dataset| {
                            dataset_picker_majors(&dataset).into_iter().map(move |major| {
                                let value = format!("{}|{}", dataset.id, major);
                                let label = dataset_catalog_option_label(&dataset, major);
                                view! { <option value=value>{label}</option> }
                            }).collect::<Vec<_>>()
                        }).collect_view()}
                    </select>
                </label>
                <DatasetCatalogContext dataset=Signal::derive(move || selected_dataset.get())/>
                <DetailAuthoringControls
                    fields=Signal::derive(move || selected_fields.get())
                    columns
                />
                <TableDefaultsControls
                    fields=Signal::derive(move || selected_fields.get())
                    sort_field
                    sort_direction
                    page_size
                />
                <div class="form-actions">
                    <button class="button button--secondary" type="button" on:click=move |_| {
                        validate_component_form(
                            ComponentValidationValues {
                                dataset_id: dataset_id.get_untracked(),
                                dataset_major: dataset_major.get_untracked(),
                                columns: columns.get_untracked(),
                                sort_field: sort_field.get_untracked(),
                                sort_direction: sort_direction.get_untracked(),
                                page_size: page_size.get_untracked(),
                            },
                            message,
                            error,
                            validation_findings,
                        );
                    }>"Validate Draft"</button>
                    <button class="button" type="submit">"Save Draft"</button>
                </div>
            </form>
            {move || dataset_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
            {move || message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
            {move || error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
            {move || {
                let findings = validation_findings.get();
                (!findings.is_empty()).then(|| view! { <ValidationFindingsPanel findings/> })
            }}
        </section>
    }
}

#[component]
pub fn ComponentPublishContent(component_ref: String) -> impl IntoView {
    let component = RwSignal::new(None::<ComponentDefinition>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let publish_error = RwSignal::new(None::<String>);
    let publish_message = RwSignal::new(None::<String>);
    let validation_error = RwSignal::new(None::<String>);
    let validation_message = RwSignal::new(None::<String>);
    let validation_findings = RwSignal::new(Vec::<ComponentValidationFinding>::new());

    Effect::new({
        let component_ref = component_ref.clone();
        move |_| load_admin_component(component_ref.clone(), component, is_loading, load_error)
    });

    view! {
        <section class="route-panel components-page">
            <ComponentsBreadcrumb current="Publish Component"/>
            <PageHeader title="Publish Component"/>
            {move || publish_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
            {move || publish_message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
            {move || validation_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
            {move || validation_message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
            {move || {
                let findings = validation_findings.get();
                (!findings.is_empty()).then(|| view! { <ValidationFindingsPanel findings/> })
            }}
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading draft" message="Fetching component versions."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Component unavailable" message=message/> }.into_any()
                } else if let Some(component) = component.get() {
                    let draft = component.versions.iter().find(|version| version.status == "draft").cloned();
                    if let Some(draft) = draft {
                        let component_id = component.id.clone();
                        let version_id = draft.id.clone();
                        let draft_for_validation = draft.clone();
                        view! {
                            <section class="route-panel__section">
                                <p>{format!("Draft {} is ready to publish.", draft.version_label)}</p>
                                <button class="button button--secondary" type="button" on:click=move |_| {
                                    validate_component_draft(draft_for_validation.clone(), validation_message, validation_error, validation_findings);
                                }>"Validate Draft"</button>
                                <button class="button" type="button" on:click=move |_| {
                                    publish_component(component_id.clone(), version_id.clone(), publish_message, publish_error);
                                }>"Publish Draft"</button>
                            </section>
                        }.into_any()
                    } else {
                        view! { <EmptyState title="No draft" message="This component does not have a draft version to publish."/> }.into_any()
                    }
                } else {
                    view! { <EmptyState title="Component unavailable" message="Component detail could not be loaded."/> }.into_any()
                }
            }}
        </section>
    }
}

#[component]
pub fn ComponentViewerContent(component_ref: String) -> impl IntoView {
    let table = RwSignal::new(None::<ComponentTable>);
    let all_columns = RwSignal::new(Vec::<ComponentTableColumn>::new());
    let error = RwSignal::new(None::<String>);
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(String::from("50"));
    let cursor = RwSignal::new(String::new());
    let sort_field = RwSignal::new(String::new());
    let sort_direction = RwSignal::new(String::from("asc"));
    let filter_field = RwSignal::new(String::new());
    let filter_operator = RwSignal::new(String::from("equals"));
    let filter_value = RwSignal::new(String::new());
    let visible_columns = RwSignal::new(String::new());

    Effect::new({
        let component_ref = component_ref.clone();
        move |_| {
            load_component_table(
                component_ref.clone(),
                build_component_table_query(ComponentTableQueryInput {
                    search: &search.get(),
                    page_size: &page_size.get(),
                    cursor: &cursor.get(),
                    sort_field: &sort_field.get(),
                    sort_direction: &sort_direction.get(),
                    filter_field: &filter_field.get(),
                    filter_operator: &filter_operator.get(),
                    filter_value: &filter_value.get(),
                    visible_columns: &visible_columns.get(),
                }),
                table,
                error,
            )
        }
    });

    Effect::new(move |_| {
        if let Some(table) = table.get()
            && (visible_columns.get().trim().is_empty() || all_columns.get_untracked().is_empty())
        {
            all_columns.set(table.columns);
        }
    });

    view! {
        <section class="dataset-preview-page">
            <section class="dataset-preview-page__content">
                <ComponentsBreadcrumb current="Component Viewer"/>
                <header class="dataset-preview-page__header">
                    <p>"Component Viewer"</p>
                    <h1>{component_ref.clone()}</h1>
                </header>
                <section class="route-panel__section form-grid">
                    <label class="form-field">
                        <span>"Search"</span>
                        <input prop:value=move || search.get() on:input=move |event| {
                            cursor.set(String::new());
                            search.set(event_target_value(&event));
                        }/>
                    </label>
                    <label class="form-field">
                        <span>"Page Size"</span>
                        <input type="number" min="1" max="200" prop:value=move || page_size.get() on:input=move |event| {
                            cursor.set(String::new());
                            page_size.set(event_target_value(&event));
                        }/>
                    </label>
                    <label class="form-field">
                        <span>"Sort Field"</span>
                        <select prop:value=move || sort_field.get() on:change=move |event| {
                            cursor.set(String::new());
                            sort_field.set(event_target_value(&event));
                        }>
                            <option value="">"Default row order"</option>
                            {move || {
                                let columns = all_columns.get();
                                if !columns.is_empty() {
                                    view! {
                                        <>
                                            {columns.into_iter().map(|column| {
                                                view! { <option value=column.key>{column.label}</option> }
                                            }).collect_view()}
                                        </>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }
                            }}
                        </select>
                    </label>
                    <label class="form-field">
                        <span>"Sort Direction"</span>
                        <select prop:value=move || sort_direction.get() on:change=move |event| {
                            cursor.set(String::new());
                            sort_direction.set(event_target_value(&event));
                        }>
                            <option value="asc">"Ascending"</option>
                            <option value="desc">"Descending"</option>
                        </select>
                    </label>
                    <label class="form-field">
                        <span>"Filter Field"</span>
                        <select prop:value=move || filter_field.get() on:change=move |event| {
                            cursor.set(String::new());
                            filter_field.set(event_target_value(&event));
                        }>
                            <option value="">"No filter"</option>
                            {move || {
                                let columns = all_columns.get();
                                if !columns.is_empty() {
                                    view! {
                                        <>
                                            {columns.into_iter().map(|column| {
                                                let label = format!("{} ({})", column.label, column.field_type);
                                                view! { <option value=column.key>{label}</option> }
                                            }).collect_view()}
                                        </>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }
                            }}
                        </select>
                    </label>
                    <label class="form-field">
                        <span>"Filter Operator"</span>
                        <select prop:value=move || filter_operator.get() on:change=move |event| {
                            cursor.set(String::new());
                            filter_operator.set(event_target_value(&event));
                        }>
                            <option value="equals">"Equals"</option>
                            <option value="not_equals">"Not Equals"</option>
                            <option value="contains">"Contains"</option>
                            <option value="not_contains">"Not Contains"</option>
                            <option value="lt">"Less Than"</option>
                            <option value="lte">"Less Than Or Equal"</option>
                            <option value="gt">"Greater Than"</option>
                            <option value="gte">"Greater Than Or Equal"</option>
                            <option value="between">"Between"</option>
                            <option value="not_between">"Not Between"</option>
                            <option value="is_empty">"Is Empty"</option>
                            <option value="is_not_empty">"Is Not Empty"</option>
                            <option value="is_null">"Is Null"</option>
                            <option value="is_not_null">"Is Not Null"</option>
                        </select>
                    </label>
                    <label class="form-field">
                        <span>"Filter Value"</span>
                        <input prop:value=move || filter_value.get() on:input=move |event| {
                            cursor.set(String::new());
                            filter_value.set(event_target_value(&event));
                        }/>
                    </label>
                    <fieldset class="form-field form-field--wide component-field-picker">
                        <legend>"Visible Columns"</legend>
                        {move || {
                            let columns = all_columns.get();
                            if columns.is_empty() {
                                view! { <p class="muted">"Columns load with the table."</p> }.into_any()
                            } else {
                                let all_keys = columns.iter().map(|column| column.key.clone()).collect::<Vec<_>>();
                                view! {
                                    <>
                                        <button
                                            class="button button--secondary"
                                            type="button"
                                            on:click=move |_| {
                                                cursor.set(String::new());
                                                visible_columns.set(String::new());
                                            }
                                        >"Show All"</button>
                                        {columns.into_iter().map(|column| {
                                            let key = column.key.clone();
                                            let key_for_toggle = column.key.clone();
                                            let keys_for_toggle = all_keys.clone();
                                            let label = format!("{} ({})", column.label, column.field_type);
                                            view! {
                                                <label class="checkbox-row">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || {
                                                            let selected = visible_columns.get();
                                                            selected.trim().is_empty() || csv_contains(&selected, &key)
                                                        }
                                                        on:change=move |_| {
                                                            cursor.set(String::new());
                                                            visible_columns.update(|value| {
                                                                toggle_visible_column(value, &key_for_toggle, &keys_for_toggle);
                                                            });
                                                        }
                                                    />
                                                    <span>{label}</span>
                                                </label>
                                            }
                                        }).collect_view()}
                                    </>
                                }.into_any()
                            }
                        }}
                    </fieldset>
                </section>
                {move || {
                    if let Some(message) = error.get() {
                        view! { <EmptyState title="Component table unavailable" message=message/> }.into_any()
                    } else if let Some(table) = table.get() {
                        let next_cursor = table.pagination.next_cursor.clone();
                        let next_cursor_for_disabled = next_cursor.clone();
                        let next_cursor_for_click = next_cursor.clone();
                        view! {
                            <ComponentTableView table/>
                            <section class="route-panel__section form-actions">
                                <button
                                    class="button button--secondary"
                                    type="button"
                                    disabled=move || cursor.get().is_empty()
                                    on:click=move |_| cursor.set(String::new())
                                >"First Page"</button>
                                <button
                                    class="button"
                                    type="button"
                                    disabled=move || next_cursor_for_disabled.is_none()
                                    on:click=move |_| {
                                        if let Some(next) = next_cursor_for_click.clone() {
                                            cursor.set(next);
                                        }
                                    }
                                >"Next Page"</button>
                            </section>
                        }.into_any()
                    } else {
                        view! { <EmptyState title="Loading table" message="Fetching component rows."/> }.into_any()
                    }
                }}
            </section>
        </section>
    }
}

#[component]
fn ValidationFindingsPanel(findings: Vec<ComponentValidationFinding>) -> impl IntoView {
    view! {
        <section class="route-panel__section validation-findings" aria-label="Validation Findings">
            <h2>"Validation Findings"</h2>
            <ul>
                {findings.into_iter().map(|finding| {
                    let field_path = finding.field_path.unwrap_or_else(|| "component".into());
                    let field_path_attr = field_path.clone();
                    let severity = finding.severity;
                    let code = finding.code;
                    let message = finding.message;
                    view! {
                        <li class="validation-finding" data-field-path=field_path_attr>
                            <strong>{field_path}</strong>
                            <span>{format!("{severity}: {code}")}</span>
                            <p>{message}</p>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </section>
    }
}

#[component]
fn DatasetCatalogContext(dataset: Signal<Option<DatasetSummary>>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            {move || if let Some(dataset) = dataset.get() {
                let tags = dataset_tag_label(&dataset.tags);
                let provenance = dataset_provenance_label(&dataset.provenance);
                let fields = dataset
                    .output_fields
                    .iter()
                    .map(|field| format!("{} ({})", field.label, field.field_type))
                    .collect::<Vec<_>>()
                    .join(", ");
                view! {
                    <h2>"Dataset Context"</h2>
                    <dl class="definition-list">
                        <dt>"Grain"</dt><dd>{dataset.grain}</dd>
                        <dt>"Tags"</dt><dd>{tags}</dd>
                        <dt>"Provenance"</dt><dd>{provenance}</dd>
                        <dt>"Fields"</dt><dd>{if fields.is_empty() { "No output fields".into() } else { fields }}</dd>
                    </dl>
                }.into_any()
            } else {
                view! {
                    <h2>"Dataset Context"</h2>
                    <p class="muted">"Select a Dataset version to review tags, provenance, and output fields."</p>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn DetailAuthoringControls(
    fields: Signal<Vec<DatasetFieldDefinition>>,
    columns: RwSignal<String>,
) -> impl IntoView {
    view! {
        <fieldset class="route-panel__section">
            <legend>"Columns"</legend>
            {move || if fields.get().is_empty() {
                view! { <p class="muted">"Select a Dataset version to choose columns."</p> }.into_any()
            } else {
                let column_fields = fields.get();
                view! {
                    <div class="component-field-picker">
                        {column_fields.into_iter().map(|field| {
                            let field_key = field.key.clone();
                            let field_key_for_toggle = field.key.clone();
                            let label = format!("{} ({})", field.label, field.field_type);
                            view! {
                                <label class="checkbox-row">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || csv_contains(&columns.get(), &field_key)
                                        on:change=move |_| {
                                            columns.update(|value| toggle_csv_key(value, &field_key_for_toggle));
                                        }
                                    />
                                    <span>{label}</span>
                                </label>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </fieldset>
    }
}

#[component]
fn TableDefaultsControls(
    fields: Signal<Vec<DatasetFieldDefinition>>,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    page_size: RwSignal<String>,
) -> impl IntoView {
    view! {
        <fieldset class="route-panel__section">
            <legend>"Table Defaults"</legend>
            <label class="form-field">
                <span>"Sort Field"</span>
                <select prop:value=move || sort_field.get() on:change=move |event| sort_field.set(event_target_value(&event))>
                    <option value="">"Default row order"</option>
                    {move || fields.get().into_iter().map(|field| {
                        let label = format!("{} ({})", field.label, field.field_type);
                        view! { <option value=field.key>{label}</option> }
                    }).collect_view()}
                </select>
            </label>
            <label class="form-field">
                <span>"Sort Direction"</span>
                <select prop:value=move || sort_direction.get() on:change=move |event| sort_direction.set(event_target_value(&event))>
                    <option value="asc">"Ascending"</option>
                    <option value="desc">"Descending"</option>
                </select>
            </label>
            <label class="form-field">
                <span>"Page Size"</span>
                <input
                    type="number"
                    min="1"
                    max="200"
                    prop:value=move || page_size.get()
                    on:input=move |event| page_size.set(event_target_value(&event))
                />
            </label>
        </fieldset>
    }
}

#[component]
fn ComponentsBreadcrumb(#[prop(into)] current: String) -> impl IntoView {
    view! {
        <Breadcrumb>
            <BreadcrumbItem>
                <BreadcrumbLink href="/components">"Components"</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator/>
            <BreadcrumbItem>
                <BreadcrumbPage>{current}</BreadcrumbPage>
            </BreadcrumbItem>
        </Breadcrumb>
    }
}

#[component]
fn ComponentsTable(components: Vec<ComponentSummary>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let kind_filter = RwSignal::new(String::from("all"));
    let status_filter = RwSignal::new(String::from("all"));
    let mobile_filter_open = RwSignal::new(false);
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let kind_options = component_kind_filter_options(&components);
    let status_options = component_status_filter_options(&components);
    let table_kind_options = kind_options.clone();
    let table_status_options = status_options.clone();
    let mobile_kind_options = kind_options.clone();
    let mobile_status_options = status_options.clone();
    let filtered_components = Memo::new(move |_| {
        let query = search.get();
        let kind = kind_filter.get();
        let status = status_filter.get();
        components
            .iter()
            .filter(|component| component_matches_filters(component, &query, &kind, &status))
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_components.get().len());
    let mobile_components = Memo::new(move |_| filtered_components.get());

    Effect::new(move |_| {
        search.get();
        kind_filter.get();
        status_filter.get();
        page_index.set(0);
    });

    view! {
        <section class="route-panel__section">
            <div class="forms-list-responsive-table components-list-responsive-table">
                <div class="searchable-data-table">
                    <div class="searchable-data-table__toolbar components-list__toolbar">
                        <label class="searchable-data-table__search searchable-data-table__control">
                            <Search class="searchable-data-table__control-icon"/>
                            <span class="sr-only">"Search components by name"</span>
                            <input
                                type="search"
                                placeholder="Search components"
                                prop:value=move || search.get()
                                on:input=move |event| search.set(event_target_value(&event))
                            />
                        </label>
                        <button
                            class="icon-button components-list__mobile-filter-button"
                            type="button"
                            aria-label=move || component_mobile_filter_button_label(
                                &kind_filter.get(),
                                &status_filter.get(),
                            )
                            title=move || component_mobile_filter_button_label(
                                &kind_filter.get(),
                                &status_filter.get(),
                            )
                            on:click=move |_| mobile_filter_open.set(true)
                        >
                            <ListFilter/>
                        </button>
                    </div>
                    <DataTable>
                        <thead>
                            <tr>
                                <th scope="col">"Name"</th>
                                <th class="data-table__cell--center" scope="col">
                                    <TableFilterHeader
                                        label="Kind"
                                        all_label="All kinds"
                                        filter=kind_filter
                                        options=table_kind_options.clone()
                                    />
                                </th>
                                <th class="data-table__cell--center" scope="col">
                                    <TableFilterHeader
                                        label="Status"
                                        all_label="All statuses"
                                        filter=status_filter
                                        options=table_status_options.clone()
                                    />
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let components = filtered_components.get();
                                if components.is_empty() {
                                    view! {
                                        <tr>
                                            <td class="data-table__empty" colspan="3">"No Components to Display"</td>
                                        </tr>
                                    }
                                    .into_any()
                                } else {
                                    components
                                        .into_iter()
                                        .skip(component_pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                                        .take(page_size.get())
                                        .map(|component| {
                                            let href = format!("/components/{}", component.slug);
                                            let kind_label = component_summary_kind_label(&component);
                                            let status_label = component_summary_status_label(&component);
                                            view! {
                                                <tr>
                                                    <th scope="row">
                                                        <a class="data-table__primary-link" href=href>{component.name}</a>
                                                    </th>
                                                    <td class="data-table__cell--center">{kind_label}</td>
                                                    <td class="data-table__cell--center">{status_label}</td>
                                                </tr>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                            }}
                        </tbody>
                    </DataTable>
                </div>
                <ComponentsMobileFilterSheet
                    is_open=mobile_filter_open
                    kind_filter=kind_filter
                    status_filter=status_filter
                    kind_options=mobile_kind_options
                    status_options=mobile_status_options
                />
                <ComponentsMobileCards
                    components=mobile_components
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                />
                <TablePaginationFooter
                    aria_label="Components table pagination"
                    item_label="components"
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                />
            </div>
        </section>
    }
}

#[component]
fn ComponentsMobileCards(
    components: Memo<Vec<ComponentSummary>>,
    total_count: Memo<usize>,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="forms-list-mobile-cards components-list-mobile-cards">
            {move || {
                let components = components.get();
                if components.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Components to Display"</p> }
                        .into_any()
                } else {
                    components
                        .into_iter()
                        .skip(component_pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                        .take(page_size.get())
                        .map(|component| {
                            let href = format!("/components/{}", component.slug);
                            let kind_label = component_summary_kind_label(&component);
                            let status_label = component_summary_status_label(&component);
                            view! {
                                <article class="forms-list-mobile-card components-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <h3><a href=href>{component.name}</a></h3>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Kind"</dt>
                                            <dd>{kind_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd>{status_label}</dd>
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
    }
}

#[component]
fn ComponentsMobileFilterSheet(
    is_open: RwSignal<bool>,
    kind_filter: RwSignal<String>,
    status_filter: RwSignal<String>,
    kind_options: Vec<String>,
    status_options: Vec<String>,
) -> impl IntoView {
    view! {
        <Show when=move || is_open.get()>
            <section class="sheet-overlay components-filter-overlay" aria-label="Component filters">
                <button
                    class="sheet-overlay__scrim"
                    type="button"
                    aria-label="Close component filters"
                    on:click=move |_| is_open.set(false)
                ></button>
                <aside class="sheet-panel blurred-surface components-filter-sheet" role="dialog" aria-modal="true" aria-label="Component filters">
                    <div class="sheet-panel__actions">
                        <button
                            class="icon-button sheet-panel__close"
                            type="button"
                            aria-label="Close component filters"
                            title="Close component filters"
                            on:click=move |_| is_open.set(false)
                        >
                            <X/>
                        </button>
                    </div>
                    <header class="sheet-panel__header">
                        <span>"Filters"</span>
                        <h2>"Components"</h2>
                    </header>
                    <section class="sheet-panel__section components-filter-sheet__controls">
                        <label class="form-field">
                            <span>"Kind"</span>
                            <select
                                aria-label="Filter components by kind"
                                prop:value=move || kind_filter.get()
                                on:change=move |event| kind_filter.set(event_target_value(&event))
                            >
                                <option value="all">"All kinds"</option>
                                {kind_options.clone().into_iter().map(|option| {
                                    view! { <option value=option.clone()>{option.clone()}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                        <label class="form-field">
                            <span>"Status"</span>
                            <select
                                aria-label="Filter components by status"
                                prop:value=move || status_filter.get()
                                on:change=move |event| status_filter.set(event_target_value(&event))
                            >
                                <option value="all">"All statuses"</option>
                                {status_options.clone().into_iter().map(|option| {
                                    view! { <option value=option.clone()>{option.clone()}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                    </section>
                    <div class="sheet-panel__footer components-filter-sheet__footer">
                        <button
                            class="button button--secondary"
                            type="button"
                            on:click=move |_| {
                                kind_filter.set("all".to_string());
                                status_filter.set("all".to_string());
                            }
                        >
                            "Clear All"
                        </button>
                    </div>
                </aside>
            </section>
        </Show>
    }
}

#[component]
fn ComponentTableView(table: ComponentTable) -> impl IntoView {
    if table.materialization_state != "ready" {
        let (title, message) = materialization_empty_state(&table.materialization_state);
        return view! {
            <EmptyState title=title message=message/>
        }
        .into_any();
    }
    view! {
        <DataTable>
            <thead>
                <tr>
                    {table.columns.iter().map(|column| view! { <th>{column.label.clone()}</th> }).collect_view()}
                </tr>
            </thead>
            <tbody>
                {table.rows.into_iter().map(|row| {
                    let columns = table.columns.clone();
                    view! {
                        <tr>
                            {columns.into_iter().map(|column| {
                                let value = row.values.get(&column.key).and_then(Clone::clone).unwrap_or_default();
                                view! { <td>{value}</td> }
                            }).collect_view()}
                        </tr>
                    }
                }).collect_view()}
            </tbody>
        </DataTable>
    }.into_any()
}

fn materialization_empty_state(state: &str) -> (&'static str, String) {
    match state {
        "failed" | "error" => (
            "Table materialization failed",
            "The component configuration is valid, but the bound Dataset major-line table could not be materialized. Retry after the Dataset materialization is rebuilt.".into(),
        ),
        "pending" => (
            "Table materializing",
            "The component configuration is valid, but the bound Dataset major-line table is still being prepared.".into(),
        ),
        other => (
            "Table materializing",
            format!("The component configuration is valid, but the bound Dataset major-line table is not ready yet. Materialization state: {other}"),
        ),
    }
}

fn component_type_label(component_type: &str) -> &'static str {
    match component_type {
        "table" => "Table",
        _ => "Component",
    }
}

fn component_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Draft",
        "published" => "Published",
        "superseded" => "Superseded",
        _ => "Unknown",
    }
}

fn component_summary_kind_label(component: &ComponentSummary) -> &'static str {
    component
        .current_component_type
        .as_deref()
        .map(component_type_label)
        .unwrap_or("Draft")
}

fn component_summary_status_label(component: &ComponentSummary) -> &'static str {
    if component.current_version_id.is_some() {
        "Published"
    } else {
        "Draft"
    }
}

fn component_kind_filter_options(components: &[ComponentSummary]) -> Vec<String> {
    let mut options = components
        .iter()
        .map(|component| component_summary_kind_label(component).to_string())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

fn component_status_filter_options(components: &[ComponentSummary]) -> Vec<String> {
    let mut options = components
        .iter()
        .map(|component| component_summary_status_label(component).to_string())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

fn component_matches_filters(
    component: &ComponentSummary,
    search: &str,
    kind_filter: &str,
    status_filter: &str,
) -> bool {
    component_text_matches(search, &[&component.name])
        && (kind_filter == "all" || component_summary_kind_label(component) == kind_filter)
        && (status_filter == "all" || component_summary_status_label(component) == status_filter)
}

fn component_mobile_filter_button_label(kind_filter: &str, status_filter: &str) -> String {
    let active_count = [kind_filter, status_filter]
        .into_iter()
        .filter(|value| *value != "all")
        .count();
    if active_count == 0 {
        "Open component filters".into()
    } else {
        format!("Open component filters ({active_count} active)")
    }
}

fn component_text_matches(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || values
            .iter()
            .any(|value| value.to_lowercase().contains(&query))
}

fn component_pagination_page_start(
    total_count: usize,
    page_size: usize,
    page_index: usize,
) -> usize {
    if total_count == 0 {
        0
    } else {
        let page_count = total_count.div_ceil(page_size);
        page_index.min(page_count.saturating_sub(1)) * page_size
    }
}

fn dataset_catalog_option_label(dataset: &DatasetSummary, major: i32) -> String {
    let mut parts = vec![format!("{} · v{}", dataset.name, major)];
    if !dataset.tags.is_empty() {
        parts.push(dataset.tags.join(", "));
    }
    let provenance = dataset_provenance_label(&dataset.provenance);
    if provenance != "No direct sources" {
        parts.push(provenance);
    }
    parts.join(" · ")
}

fn dataset_tag_label(tags: &[String]) -> String {
    if tags.is_empty() {
        "No tags".into()
    } else {
        tags.join(", ")
    }
}

fn dataset_provenance_label(provenance: &super::types::DatasetProvenanceSummary) -> String {
    let forms = provenance.forms.iter().map(|item| item.name.clone());
    let datasets = provenance
        .datasets
        .iter()
        .map(|item| format!("Dataset: {}", item.name));
    let sources = forms.chain(datasets).collect::<Vec<_>>();
    if sources.is_empty() {
        "No direct sources".into()
    } else {
        sources.join(", ")
    }
}

fn dataset_picker_majors(dataset: &DatasetSummary) -> Vec<i32> {
    if dataset.major_versions.is_empty() {
        dataset.current_version_major.into_iter().collect()
    } else {
        dataset.major_versions.clone()
    }
}

fn csv_field_keys(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn csv_contains(value: &str, key: &str) -> bool {
    csv_field_keys(value).iter().any(|existing| existing == key)
}

fn toggle_csv_key(value: &mut String, key: &str) {
    let mut keys = csv_field_keys(value);
    if keys.iter().any(|existing| existing == key) {
        keys.retain(|existing| existing != key);
    } else {
        keys.push(key.to_string());
    }
    *value = keys.join(", ");
}

fn toggle_visible_column(value: &mut String, key: &str, all_keys: &[String]) {
    let mut keys = if value.trim().is_empty() {
        all_keys.to_vec()
    } else {
        csv_field_keys(value)
    };
    if keys.iter().any(|existing| existing == key) {
        keys.retain(|existing| existing != key);
    } else {
        keys.push(key.to_string());
    }
    if keys.len() == all_keys.len() && all_keys.iter().all(|key| keys.contains(key)) {
        value.clear();
    } else {
        *value = all_keys
            .iter()
            .filter(|key| keys.contains(key))
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
    }
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
fn build_component_config(
    columns: &str,
    sort_field: &str,
    sort_direction: &str,
    page_size: &str,
) -> Value {
    let fields = csv_field_keys(columns);
    let defaults = table_defaults_config(sort_field, sort_direction, page_size);
    let mut config = json!({
        "visible_columns": fields
    });
    merge_table_defaults(&mut config, defaults);
    config
}

fn table_defaults_config(sort_field: &str, sort_direction: &str, page_size: &str) -> Value {
    let parsed_page_size = page_size
        .trim()
        .parse::<usize>()
        .ok()
        .map(|value| value.clamp(1, 200))
        .unwrap_or(50);
    let direction = if sort_direction.trim() == "desc" {
        "desc"
    } else {
        "asc"
    };
    let default_sort = if sort_field.trim().is_empty() {
        Value::Null
    } else {
        json!({
            "field_key": sort_field.trim(),
            "direction": direction
        })
    };
    json!({
        "default_sort": default_sort,
        "page_size": parsed_page_size
    })
}

fn merge_table_defaults(config: &mut Value, defaults: Value) {
    if let (Some(target), Some(defaults)) = (config.as_object_mut(), defaults.as_object()) {
        for (key, value) in defaults {
            target.insert(key.clone(), value.clone());
        }
    }
}

fn selected_dataset_major_value(dataset_id: &str, dataset_major: &str) -> String {
    if dataset_id.trim().is_empty() || dataset_major.trim().is_empty() {
        String::new()
    } else {
        format!("{}|{}", dataset_id.trim(), dataset_major.trim())
    }
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
fn editable_component_version(component: &ComponentDefinition) -> Option<ComponentVersionSummary> {
    component
        .versions
        .iter()
        .find(|version| version.status == "draft")
        .or_else(|| {
            component
                .versions
                .iter()
                .find(|version| version.status == "published")
        })
        .cloned()
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
fn table_visible_columns_from_config(config: &Value) -> String {
    config
        .get("visible_columns")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| {
            value.as_str().map(str::to_string).or_else(|| {
                value
                    .get("field_key")
                    .or_else(|| value.get("key"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
fn table_sort_from_config(config: &Value) -> (String, String) {
    let Some(sort) = config.get("default_sort") else {
        return (String::new(), "asc".into());
    };
    let field = sort
        .get("field_key")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let direction = match sort.get("direction").and_then(Value::as_str) {
        Some("desc") => "desc",
        _ => "asc",
    };
    (field, direction.into())
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
fn table_page_size_from_config(config: &Value) -> String {
    config
        .get("page_size")
        .and_then(Value::as_u64)
        .map(|value| value.clamp(1, 200).to_string())
        .unwrap_or_else(|| "50".into())
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
struct ComponentTableQueryInput<'a> {
    search: &'a str,
    page_size: &'a str,
    cursor: &'a str,
    sort_field: &'a str,
    sort_direction: &'a str,
    filter_field: &'a str,
    filter_operator: &'a str,
    filter_value: &'a str,
    visible_columns: &'a str,
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
fn build_component_table_query(input: ComponentTableQueryInput<'_>) -> String {
    let mut params = Vec::new();
    push_query_param(&mut params, "q", input.search);
    push_query_param(&mut params, "page_size", input.page_size);
    push_query_param(&mut params, "cursor", input.cursor);
    if !input.sort_field.trim().is_empty() {
        let direction = if input.sort_direction.trim() == "desc" {
            "desc"
        } else {
            "asc"
        };
        push_query_param(
            &mut params,
            "sort",
            &format!("{}:{direction}", input.sort_field.trim()),
        );
    }
    let filter_field = input.filter_field.trim();
    if !filter_field.is_empty() {
        push_query_param(
            &mut params,
            &format!("filter[{filter_field}][operator]"),
            input.filter_operator,
        );
        push_query_param(
            &mut params,
            &format!("filter[{filter_field}][value]"),
            input.filter_value,
        );
    }
    push_query_param(&mut params, "visible_columns", input.visible_columns);
    params.join("&")
}

fn push_query_param(params: &mut Vec<String>, key: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        params.push(format!(
            "{}={}",
            percent_encode_query_component(key),
            percent_encode_query_component(value)
        ));
    }
}

fn percent_encode_query_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            byte => format!("%{byte:02X}").chars().collect::<Vec<_>>(),
        })
        .collect()
}

#[cfg(feature = "hydrate")]
fn load_components(
    components: RwSignal<Vec<ComponentSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        load_error.set(None);
        match api::fetch_components().await {
            Ok(Some(response)) => components.set(response),
            Ok(None) => components.set(Vec::new()),
            Err(message) => load_error.set(Some(message)),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_components(
    _: RwSignal<Vec<ComponentSummary>>,
    is_loading: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
    is_loading.set(false);
}

#[cfg(feature = "hydrate")]
fn load_component(
    component_ref: String,
    component: RwSignal<Option<ComponentDefinition>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        load_error.set(None);
        match fetch_authoring_or_reader_component(&component_ref).await {
            Ok(Some(response)) => component.set(Some(response)),
            Ok(None) => component.set(None),
            Err(message) => load_error.set(Some(message)),
        }
        is_loading.set(false);
    });
}

#[cfg(feature = "hydrate")]
async fn fetch_authoring_or_reader_component(
    component_ref: &str,
) -> Result<Option<ComponentDefinition>, String> {
    match api::fetch_admin_component(component_ref).await {
        Ok(response) => Ok(response),
        Err(_) => api::fetch_component(component_ref).await,
    }
}

#[cfg(feature = "hydrate")]
fn load_admin_component(
    component_ref: String,
    component: RwSignal<Option<ComponentDefinition>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        load_error.set(None);
        match api::fetch_admin_component(&component_ref).await {
            Ok(Some(response)) => component.set(Some(response)),
            Ok(None) => component.set(None),
            Err(message) => load_error.set(Some(message)),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_admin_component(
    _: String,
    _: RwSignal<Option<ComponentDefinition>>,
    is_loading: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
    is_loading.set(false);
}

#[cfg(feature = "hydrate")]
#[allow(clippy::too_many_arguments)]
fn load_component_for_edit(
    component_ref: String,
    editing_component_id: RwSignal<Option<String>>,
    editing_version_id: RwSignal<Option<String>>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    description: RwSignal<String>,
    dataset_id: RwSignal<String>,
    dataset_major: RwSignal<String>,
    component_type: RwSignal<String>,
    columns: RwSignal<String>,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    page_size: RwSignal<String>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        error.set(None);
        match api::fetch_admin_component(&component_ref).await {
            Ok(Some(component)) => {
                editing_component_id.set(Some(component.id.clone()));
                name.set(component.name.clone());
                slug.set(component.slug.clone());
                description.set(component.description.clone().unwrap_or_default());
                if let Some(version) = editable_component_version(&component) {
                    editing_version_id.set((version.status == "draft").then(|| version.id.clone()));
                    dataset_id.set(version.dataset_id);
                    dataset_major.set(version.dataset_version_major.to_string());
                    component_type.set(version.component_type.clone());
                    let (loaded_sort_field, loaded_sort_direction) =
                        table_sort_from_config(&version.config);
                    sort_field.set(loaded_sort_field);
                    sort_direction.set(loaded_sort_direction);
                    page_size.set(table_page_size_from_config(&version.config));
                    columns.set(table_visible_columns_from_config(&version.config));
                }
            }
            Ok(None) => error.set(Some("Component could not be loaded.".into())),
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(clippy::too_many_arguments)]
fn load_component_for_edit(
    _: String,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(not(feature = "hydrate"))]
fn load_component(
    _: String,
    _: RwSignal<Option<ComponentDefinition>>,
    is_loading: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
    is_loading.set(false);
}

#[cfg(feature = "hydrate")]
fn load_datasets(datasets: RwSignal<Vec<DatasetSummary>>, error: RwSignal<Option<String>>) {
    leptos::task::spawn_local(async move {
        error.set(None);
        match api::fetch_datasets().await {
            Ok(Some(response)) => datasets.set(response),
            Ok(None) => datasets.set(Vec::new()),
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_datasets(_: RwSignal<Vec<DatasetSummary>>, _: RwSignal<Option<String>>) {}

#[cfg(feature = "hydrate")]
fn load_component_table(
    component_ref: String,
    query: String,
    table: RwSignal<Option<ComponentTable>>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        error.set(None);
        match api::fetch_component_table(&component_ref, &query).await {
            Ok(Some(response)) => table.set(Some(response)),
            Ok(None) => table.set(None),
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_component_table(
    _: String,
    _: String,
    _: RwSignal<Option<ComponentTable>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct ComponentFormValues {
    name: String,
    slug: String,
    description: String,
    dataset_id: String,
    dataset_major: String,
    columns: String,
    sort_field: String,
    sort_direction: String,
    page_size: String,
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct ComponentValidationValues {
    dataset_id: String,
    dataset_major: String,
    columns: String,
    sort_field: String,
    sort_direction: String,
    page_size: String,
}

#[cfg(feature = "hydrate")]
fn create_component_from_form(
    editing_component_id: Option<String>,
    editing_version_id: Option<String>,
    values: ComponentFormValues,
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    findings: RwSignal<Vec<ComponentValidationFinding>>,
) {
    leptos::task::spawn_local(async move {
        message.set(None);
        error.set(None);
        findings.set(Vec::new());
        let major = values.dataset_major.trim().parse::<i32>().unwrap_or(1);
        let config = build_component_config(
            &values.columns,
            &values.sort_field,
            &values.sort_direction,
            &values.page_size,
        );
        let version = CreateComponentVersionRequest {
            dataset_id: Some(values.dataset_id),
            dataset_version_major: Some(major),
            component_type: "table".into(),
            config,
        };
        let description = if values.description.trim().is_empty() {
            None
        } else {
            Some(values.description.trim().to_string())
        };
        let redirect_ref = component_redirect_ref(&values.slug);
        let result = if let Some(component_id) = editing_component_id {
            match api::update_component(
                &component_id,
                UpdateComponentRequest {
                    name: values.name,
                    slug: values.slug,
                    description,
                },
            )
            .await
            {
                Ok(_) => {
                    if let Some(version_id) = editing_version_id {
                        api::update_component_version(&component_id, &version_id, version).await
                    } else {
                        api::save_component_version(&component_id, version).await
                    }
                }
                Err(message) => Err(message),
            }
        } else {
            api::create_component(CreateComponentRequest {
                name: values.name,
                slug: values.slug,
                description,
                version: Some(version),
            })
            .await
        };
        match result {
            Ok(_) => {
                message.set(Some("Component draft saved.".into()));
                if let Some(window) = web_sys::window() {
                    let _ = window
                        .location()
                        .set_href(&format!("/components/{redirect_ref}"));
                }
            }
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
fn component_redirect_ref(slug: &str) -> String {
    slug.trim().to_string()
}

#[cfg(not(feature = "hydrate"))]
fn create_component_from_form(
    _: Option<String>,
    _: Option<String>,
    _: ComponentFormValues,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Vec<ComponentValidationFinding>>,
) {
}

#[cfg(feature = "hydrate")]
fn validate_component_form(
    values: ComponentValidationValues,
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    findings: RwSignal<Vec<ComponentValidationFinding>>,
) {
    leptos::task::spawn_local(async move {
        message.set(None);
        error.set(None);
        findings.set(Vec::new());
        let major = values.dataset_major.trim().parse::<i32>().unwrap_or(1);
        let config = build_component_config(
            &values.columns,
            &values.sort_field,
            &values.sort_direction,
            &values.page_size,
        );
        let payload = CreateComponentVersionRequest {
            dataset_id: Some(values.dataset_id),
            dataset_version_major: Some(major),
            component_type: "table".into(),
            config,
        };
        match api::validate_component_version(payload).await {
            Ok(response) if response.valid => {
                message.set(Some("Component draft is valid.".into()));
            }
            Ok(response) => {
                findings.set(response.findings);
            }
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn validate_component_form(
    _: ComponentValidationValues,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Vec<ComponentValidationFinding>>,
) {
}

#[cfg(feature = "hydrate")]
fn validate_component_draft(
    draft: ComponentVersionSummary,
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    findings: RwSignal<Vec<ComponentValidationFinding>>,
) {
    leptos::task::spawn_local(async move {
        message.set(None);
        error.set(None);
        findings.set(Vec::new());
        let payload = CreateComponentVersionRequest {
            dataset_id: Some(draft.dataset_id),
            dataset_version_major: Some(draft.dataset_version_major),
            component_type: draft.component_type,
            config: draft.config,
        };
        match api::validate_component_version(payload).await {
            Ok(response) if response.valid => {
                message.set(Some("Component draft is valid.".into()));
            }
            Ok(response) => {
                findings.set(response.findings);
            }
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn validate_component_draft(
    _: ComponentVersionSummary,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Vec<ComponentValidationFinding>>,
) {
}

#[cfg(feature = "hydrate")]
fn publish_component(
    component_id: String,
    version_id: String,
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        message.set(None);
        error.set(None);
        match api::publish_component_version(&component_id, &version_id).await {
            Ok(_) => message.set(Some("Component published.".into())),
            Err(error_message) => error.set(Some(error_message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn publish_component(
    _: String,
    _: String,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentDefinition, ComponentVersionSummary, DatasetSummary, build_component_config,
        dataset_picker_majors, toggle_csv_key, toggle_visible_column,
    };
    use super::{
        ComponentTableQueryInput, build_component_table_query, percent_encode_query_component,
    };
    use super::{
        component_kind_filter_options, component_matches_filters, component_status_filter_options,
    };
    use super::{
        component_redirect_ref, dataset_catalog_option_label, dataset_provenance_label,
        editable_component_version, materialization_empty_state, selected_dataset_major_value,
        table_page_size_from_config, table_sort_from_config, table_visible_columns_from_config,
    };
    use crate::types::{ComponentSummary, DatasetProvenanceItem, DatasetProvenanceSummary};

    fn dataset(major_versions: Vec<i32>, current_version_major: Option<i32>) -> DatasetSummary {
        DatasetSummary {
            id: "dataset-1".into(),
            current_version_major,
            major_versions,
            name: "Dataset".into(),
            slug: "dataset".into(),
            grain: "submission".into(),
            tags: Vec::new(),
            provenance: Default::default(),
            output_fields: Vec::new(),
        }
    }

    fn component_summary(
        name: &str,
        component_type: Option<&str>,
        published: bool,
    ) -> ComponentSummary {
        ComponentSummary {
            id: format!("{name}-id"),
            name: name.into(),
            slug: name.to_lowercase().replace(' ', "-"),
            description: None,
            current_version_id: published.then(|| format!("{name}-version")),
            current_component_type: component_type.map(str::to_string),
        }
    }

    #[test]
    fn dataset_picker_prefers_major_versions_from_list_response() {
        assert_eq!(
            dataset_picker_majors(&dataset(vec![1, 2], Some(3))),
            vec![1, 2]
        );
    }

    #[test]
    fn dataset_picker_falls_back_to_current_major() {
        assert_eq!(
            dataset_picker_majors(&dataset(Vec::new(), Some(4))),
            vec![4]
        );
    }

    #[test]
    fn dataset_catalog_option_includes_tags_and_provenance() {
        let mut dataset = dataset(vec![1], Some(1));
        dataset.tags = vec!["finance".into(), "display".into()];
        dataset.provenance = DatasetProvenanceSummary {
            forms: vec![DatasetProvenanceItem {
                id: "form-1".into(),
                name: "Intake Form".into(),
                slug: Some("intake".into()),
            }],
            datasets: vec![DatasetProvenanceItem {
                id: "dataset-2".into(),
                name: "Analytical Source".into(),
                slug: Some("analytical-source".into()),
            }],
        };

        assert_eq!(
            dataset_catalog_option_label(&dataset, 1),
            "Dataset · v1 · finance, display · Intake Form, Dataset: Analytical Source"
        );
        assert_eq!(
            dataset_provenance_label(&dataset.provenance),
            "Intake Form, Dataset: Analytical Source"
        );
    }

    #[test]
    fn component_list_filters_match_name_kind_and_status() {
        let published_table = component_summary("Program Snapshot", Some("table"), true);
        let draft_component = component_summary("Program Draft", None, false);
        let components = vec![published_table.clone(), draft_component.clone()];

        assert_eq!(
            component_kind_filter_options(&components),
            vec!["Draft", "Table"]
        );
        assert_eq!(
            component_status_filter_options(&components),
            vec!["Draft", "Published"]
        );
        assert!(component_matches_filters(
            &published_table,
            "snapshot",
            "Table",
            "Published"
        ));
        assert!(!component_matches_filters(
            &published_table,
            "snapshot",
            "Draft",
            "Published"
        ));
        assert!(component_matches_filters(
            &draft_component,
            "program",
            "Draft",
            "Draft"
        ));
    }

    #[test]
    fn table_config_uses_visible_columns_and_defaults() {
        let config = build_component_config("program, amount", "program", "desc", "25");

        assert_eq!(
            config["visible_columns"],
            serde_json::json!(["program", "amount"])
        );
        assert_eq!(config["default_sort"]["field_key"], "program");
        assert_eq!(config["default_sort"]["direction"], "desc");
        assert_eq!(config["page_size"], 25);
    }

    #[test]
    fn toggle_csv_key_adds_and_removes_keys() {
        let mut value = "program".to_string();
        toggle_csv_key(&mut value, "amount");
        assert_eq!(value, "program, amount");
        toggle_csv_key(&mut value, "program");
        assert_eq!(value, "amount");
    }

    #[test]
    fn toggle_visible_column_treats_blank_as_all_selected() {
        let all_keys = vec!["program".into(), "amount".into(), "status".into()];
        let mut value = String::new();

        toggle_visible_column(&mut value, "amount", &all_keys);
        assert_eq!(value, "program, status");

        toggle_visible_column(&mut value, "amount", &all_keys);
        assert_eq!(value, "");
    }

    #[test]
    fn table_visible_columns_parse_string_and_object_configs() {
        let config = serde_json::json!({
            "visible_columns": [
                "program",
                { "field_key": "amount" },
                { "key": "status" }
            ]
        });

        assert_eq!(
            table_visible_columns_from_config(&config),
            "program, amount, status"
        );
    }

    #[test]
    fn table_config_extracts_sort_and_page_size() {
        let config = serde_json::json!({
            "default_sort": {
                "field_key": "program",
                "direction": "desc"
            },
            "page_size": 25
        });

        assert_eq!(
            table_sort_from_config(&config),
            ("program".into(), "desc".into())
        );
        assert_eq!(table_page_size_from_config(&config), "25");
    }

    #[test]
    fn component_table_query_encodes_server_driven_view_state() {
        let query = build_component_table_query(ComponentTableQueryInput {
            search: "family outreach",
            page_size: "25",
            cursor: "offset:25",
            sort_field: "program",
            sort_direction: "desc",
            filter_field: "row_count",
            filter_operator: "between",
            filter_value: "1,10",
            visible_columns: "program, row_count",
        });

        assert_eq!(
            query,
            "q=family%20outreach&page_size=25&cursor=offset%3A25&sort=program%3Adesc&filter%5Brow_count%5D%5Boperator%5D=between&filter%5Brow_count%5D%5Bvalue%5D=1%2C10&visible_columns=program%2C%20row_count"
        );
    }

    #[test]
    fn component_table_query_omits_blank_optional_params() {
        assert_eq!(
            build_component_table_query(ComponentTableQueryInput {
                search: "",
                page_size: "50",
                cursor: "",
                sort_field: "",
                sort_direction: "asc",
                filter_field: "",
                filter_operator: "equals",
                filter_value: "",
                visible_columns: "",
            }),
            "page_size=50"
        );
        assert_eq!(percent_encode_query_component("a/b?c"), "a%2Fb%3Fc");
    }

    #[test]
    fn materialization_empty_state_distinguishes_failed_from_pending() {
        let (pending_title, pending_message) = materialization_empty_state("pending");
        assert_eq!(pending_title, "Table materializing");
        assert!(pending_message.contains("still being prepared"));

        let (failed_title, failed_message) = materialization_empty_state("failed");
        assert_eq!(failed_title, "Table materialization failed");
        assert!(failed_message.contains("configuration is valid"));

        let (retry_title, retry_message) = materialization_empty_state("retry");
        assert_eq!(retry_title, "Table materializing");
        assert!(retry_message.contains("retry"));
    }

    #[test]
    fn selected_dataset_major_value_is_empty_until_complete() {
        assert_eq!(selected_dataset_major_value("", "1"), "");
        assert_eq!(selected_dataset_major_value("dataset-1", ""), "");
        assert_eq!(
            selected_dataset_major_value("dataset-1", "2"),
            "dataset-1|2"
        );
    }

    #[test]
    fn component_redirect_ref_uses_trimmed_slug() {
        assert_eq!(
            component_redirect_ref("  family-outreach-table  "),
            "family-outreach-table"
        );
    }

    #[test]
    fn editable_component_version_prefers_draft() {
        let component = ComponentDefinition {
            id: "component-1".into(),
            name: "Component".into(),
            slug: "component".into(),
            description: None,
            versions: vec![
                component_version("published", "published-version"),
                component_version("draft", "draft-version"),
            ],
        };

        assert_eq!(
            editable_component_version(&component)
                .expect("editable version")
                .id,
            "draft-version"
        );
    }

    fn component_version(status: &str, id: &str) -> ComponentVersionSummary {
        ComponentVersionSummary {
            id: id.into(),
            component_id: "component-1".into(),
            dataset_id: "dataset-1".into(),
            dataset_version_major: 1,
            binding_mode: "major_line".into(),
            component_type: "table".into(),
            status: status.into(),
            version_label: "1".into(),
            config: serde_json::json!({ "visible_columns": ["program"] }),
        }
    }
}
