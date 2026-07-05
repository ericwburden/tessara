//! Route-level page composition for the Components feature.

#[cfg(feature = "hydrate")]
use super::types::{CreateComponentRequest, CreateComponentVersionRequest, UpdateComponentRequest};
use leptos::prelude::*;
use serde_json::{Value, json};
use tessara_web_ui::{DataTable, EmptyState, PageHeader};

#[cfg(feature = "hydrate")]
use super::api;
use super::types::{
    ComponentDefinition, ComponentSummary, ComponentTable, ComponentTableColumn,
    ComponentValidationFinding, ComponentVersionSummary, DatasetFieldDefinition, DatasetSummary,
};
use tessara_web_data_ops::{
    DataOpsAggregationEditor, DatasetAggregationDraft, DatasetAggregationMetricDraft,
    DatasetFieldDraft,
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
            <PageHeader title="Components">
                <a class="button" href="/components/new">"Create Component"</a>
            </PageHeader>
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
    let component_type = RwSignal::new(String::from("detail_table"));
    let columns = RwSignal::new(String::new());
    let aggregation = RwSignal::new(default_component_aggregation());
    let detail_filter_field = RwSignal::new(String::new());
    let detail_filter_operator = RwSignal::new(String::from("equals"));
    let detail_filter_value = RwSignal::new(String::new());
    let pre_filter_field = RwSignal::new(String::new());
    let pre_filter_operator = RwSignal::new(String::from("equals"));
    let pre_filter_value = RwSignal::new(String::new());
    let post_filter_field = RwSignal::new(String::new());
    let post_filter_operator = RwSignal::new(String::from("equals"));
    let post_filter_value = RwSignal::new(String::new());
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
                    aggregation,
                    detail_filter_field,
                    detail_filter_operator,
                    detail_filter_value,
                    pre_filter_field,
                    pre_filter_operator,
                    pre_filter_value,
                    post_filter_field,
                    post_filter_operator,
                    post_filter_value,
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
            <PageHeader title/>
            <form class="route-panel__section form-grid" on:submit=move |event| {
                event.prevent_default();
                create_component_from_form(
                    editing_component_id.get_untracked(),
                    editing_version_id.get_untracked(),
                    name.get_untracked(),
                    slug.get_untracked(),
                    description.get_untracked(),
                    dataset_id.get_untracked(),
                    dataset_major.get_untracked(),
                    component_type.get_untracked(),
                    columns.get_untracked(),
                    aggregation.get_untracked(),
                    detail_filter_field.get_untracked(),
                    detail_filter_operator.get_untracked(),
                    detail_filter_value.get_untracked(),
                    pre_filter_field.get_untracked(),
                    pre_filter_operator.get_untracked(),
                    pre_filter_value.get_untracked(),
                    post_filter_field.get_untracked(),
                    post_filter_operator.get_untracked(),
                    post_filter_value.get_untracked(),
                    sort_field.get_untracked(),
                    sort_direction.get_untracked(),
                    page_size.get_untracked(),
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
                                let default_group = dataset
                                    .output_fields
                                    .iter()
                                    .find(|field| field.field_type != "number")
                                    .or_else(|| dataset.output_fields.first())
                                    .map(|field| field.key.clone());
                                aggregation.set(default_component_aggregation_with_group(default_group));
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
                                let label = format!("{} · v{}", dataset.name, major);
                                view! { <option value=value>{label}</option> }
                            }).collect::<Vec<_>>()
                        }).collect_view()}
                    </select>
                </label>
                <label class="form-field">
                    <span>"Kind"</span>
                    <select prop:value=move || component_type.get() on:change=move |event| component_type.set(event_target_value(&event))>
                        <option value="detail_table">"Detail Table"</option>
                        <option value="aggregate_table">"Aggregate Table"</option>
                    </select>
                </label>
                {move || if component_type.get() == "aggregate_table" {
                    view! {
                        <AggregateAuthoringControls
                            fields=selected_fields.get()
                            aggregation
                            pre_filter_field
                            pre_filter_operator
                            pre_filter_value
                            post_filter_field
                            post_filter_operator
                            post_filter_value
                        />
                    }.into_any()
                } else {
                    view! {
                        <DetailAuthoringControls
                            fields=selected_fields.get()
                            columns
                            filter_field=detail_filter_field
                            filter_operator=detail_filter_operator
                            filter_value=detail_filter_value
                        />
                    }.into_any()
                }}
                {move || if component_type.get() == "aggregate_table" {
                    view! {
                        <AggregateTableDefaultsControls
                            fields=selected_fields.get()
                            aggregation
                            sort_field
                            sort_direction
                            page_size
                        />
                    }.into_any()
                } else {
                    view! {
                        <TableDefaultsControls
                            fields=selected_fields.get()
                            sort_field
                            sort_direction
                            page_size
                        />
                    }.into_any()
                }}
                <div class="form-actions">
                    <button class="button button--secondary" type="button" on:click=move |_| {
                        validate_component_form(
                            dataset_id.get_untracked(),
                            dataset_major.get_untracked(),
                            component_type.get_untracked(),
                            columns.get_untracked(),
                            aggregation.get_untracked(),
                            detail_filter_field.get_untracked(),
                            detail_filter_operator.get_untracked(),
                            detail_filter_value.get_untracked(),
                            pre_filter_field.get_untracked(),
                            pre_filter_operator.get_untracked(),
                            pre_filter_value.get_untracked(),
                            post_filter_field.get_untracked(),
                            post_filter_operator.get_untracked(),
                            post_filter_value.get_untracked(),
                            sort_field.get_untracked(),
                            sort_direction.get_untracked(),
                            page_size.get_untracked(),
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
                build_component_table_query(
                    &search.get(),
                    &page_size.get(),
                    &cursor.get(),
                    &sort_field.get(),
                    &sort_direction.get(),
                    &filter_field.get(),
                    &filter_operator.get(),
                    &filter_value.get(),
                    &visible_columns.get(),
                ),
                table,
                error,
            )
        }
    });

    Effect::new(move |_| {
        if let Some(table) = table.get() {
            if visible_columns.get().trim().is_empty() || all_columns.get_untracked().is_empty() {
                all_columns.set(table.columns);
            }
        }
    });

    view! {
        <section class="dataset-preview-page">
            <section class="dataset-preview-page__content">
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
                                    view! { <></> }.into_any()
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
                                    view! { <></> }.into_any()
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
fn DetailAuthoringControls(
    fields: Vec<DatasetFieldDefinition>,
    columns: RwSignal<String>,
    filter_field: RwSignal<String>,
    filter_operator: RwSignal<String>,
    filter_value: RwSignal<String>,
) -> impl IntoView {
    let column_fields = fields.clone();
    view! {
        <fieldset class="route-panel__section">
            <legend>"Columns"</legend>
            {if fields.is_empty() {
                view! { <p class="muted">"Select a Dataset version to choose columns."</p> }.into_any()
            } else {
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
        <FilterAuthoringControls
            title="Default Filter"
            fields=fields
            filter_field
            filter_operator
            filter_value
        />
    }
}

#[component]
fn AggregateAuthoringControls(
    fields: Vec<DatasetFieldDefinition>,
    aggregation: RwSignal<DatasetAggregationDraft>,
    pre_filter_field: RwSignal<String>,
    pre_filter_operator: RwSignal<String>,
    pre_filter_value: RwSignal<String>,
    post_filter_field: RwSignal<String>,
    post_filter_operator: RwSignal<String>,
    post_filter_value: RwSignal<String>,
) -> impl IntoView {
    let dataset_fields = fields
        .iter()
        .map(dataset_field_draft_from_component_field)
        .collect::<Vec<_>>();
    let fields_signal = Signal::derive(move || dataset_fields.clone());
    let aggregation_signal = Signal::derive(move || aggregation.get());
    let on_aggregation_change = Callback::new(move |draft: DatasetAggregationDraft| {
        aggregation.set(draft);
    });
    view! {
        {if fields.is_empty() {
            view! {
                <fieldset class="route-panel__section">
                    <legend>"Aggregation"</legend>
                    <p class="muted">"Select a Dataset version to configure grouping and metrics."</p>
                </fieldset>
            }.into_any()
        } else {
            view! {
                <DataOpsAggregationEditor
                    fields=fields_signal
                    aggregation=aggregation_signal
                    on_aggregation_change
                    embedded=true
                    metrics_only=true
                />
            }.into_any()
        }}
        <FilterAuthoringControls
            title="Pre-Aggregation Filter"
            fields=fields.clone()
            filter_field=pre_filter_field
            filter_operator=pre_filter_operator
            filter_value=pre_filter_value
        />
        <AggregatePostFilterControls
            fields
            aggregation
            post_filter_field
            post_filter_operator
            post_filter_value
        />
    }
}

#[component]
fn AggregatePostFilterControls(
    fields: Vec<DatasetFieldDefinition>,
    aggregation: RwSignal<DatasetAggregationDraft>,
    post_filter_field: RwSignal<String>,
    post_filter_operator: RwSignal<String>,
    post_filter_value: RwSignal<String>,
) -> impl IntoView {
    view! {
        <fieldset class="route-panel__section">
            <legend>"Post-Aggregation Filter"</legend>
            <label class="form-field">
                <span>"Field"</span>
                <select prop:value=move || post_filter_field.get() on:change=move |event| post_filter_field.set(event_target_value(&event))>
                    <option value="">"No filter"</option>
                    {move || aggregate_output_fields(
                        &fields,
                        &aggregation.get(),
                    ).into_iter().map(|field| {
                        let label = format!("{} ({})", field.label, field.field_type);
                        view! { <option value=field.key>{label}</option> }
                    }).collect_view()}
                </select>
            </label>
            <label class="form-field">
                <span>"Operator"</span>
                <select prop:value=move || post_filter_operator.get() on:change=move |event| post_filter_operator.set(event_target_value(&event))>
                    <option value="equals">"Equals"</option>
                    <option value="not_equals">"Not Equals"</option>
                    <option value="contains">"Contains"</option>
                    <option value="not_contains">"Not Contains"</option>
                    <option value="is_null">"Is Null"</option>
                    <option value="is_not_null">"Is Not Null"</option>
                </select>
            </label>
            <label class="form-field">
                <span>"Value"</span>
                <input prop:value=move || post_filter_value.get() on:input=move |event| post_filter_value.set(event_target_value(&event))/>
            </label>
        </fieldset>
    }
}

#[component]
fn AggregateTableDefaultsControls(
    fields: Vec<DatasetFieldDefinition>,
    aggregation: RwSignal<DatasetAggregationDraft>,
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
                    {move || aggregate_output_fields(
                        &fields,
                        &aggregation.get(),
                    ).into_iter().map(|field| {
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
fn FilterAuthoringControls(
    title: &'static str,
    fields: Vec<DatasetFieldDefinition>,
    filter_field: RwSignal<String>,
    filter_operator: RwSignal<String>,
    filter_value: RwSignal<String>,
) -> impl IntoView {
    view! {
        <fieldset class="route-panel__section">
            <legend>{title}</legend>
            <label class="form-field">
                <span>"Field"</span>
                <select prop:value=move || filter_field.get() on:change=move |event| filter_field.set(event_target_value(&event))>
                    <option value="">"No filter"</option>
                    {fields.into_iter().map(|field| {
                        let label = format!("{} ({})", field.label, field.field_type);
                        view! { <option value=field.key>{label}</option> }
                    }).collect_view()}
                </select>
            </label>
            <label class="form-field">
                <span>"Operator"</span>
                <select prop:value=move || filter_operator.get() on:change=move |event| filter_operator.set(event_target_value(&event))>
                    <option value="equals">"Equals"</option>
                    <option value="not_equals">"Not Equals"</option>
                    <option value="contains">"Contains"</option>
                    <option value="not_contains">"Not Contains"</option>
                    <option value="is_null">"Is Null"</option>
                    <option value="is_not_null">"Is Not Null"</option>
                </select>
            </label>
            <label class="form-field">
                <span>"Value"</span>
                <input prop:value=move || filter_value.get() on:input=move |event| filter_value.set(event_target_value(&event))/>
            </label>
        </fieldset>
    }
}

#[component]
fn TableDefaultsControls(
    fields: Vec<DatasetFieldDefinition>,
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
                    {fields.into_iter().map(|field| {
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
fn ComponentsTable(components: Vec<ComponentSummary>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <DataTable>
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Kind"</th>
                        <th>"Status"</th>
                    </tr>
                </thead>
                <tbody>
                    {components.into_iter().map(|component| {
                        let href = format!("/components/{}", component.slug);
                        view! {
                            <tr>
                                <td><a href=href>{component.name}</a></td>
                                <td>{component.current_component_type.as_deref().map(component_type_label).unwrap_or("Draft")}</td>
                                <td>{if component.current_version_id.is_some() { "Published" } else { "Draft" }}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </DataTable>
        </section>
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
        "detail_table" => "Detail Table",
        "aggregate_table" => "Aggregate Table",
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

fn aggregate_output_fields(
    fields: &[DatasetFieldDefinition],
    aggregation: &DatasetAggregationDraft,
) -> Vec<DatasetFieldDefinition> {
    let mut output = Vec::new();
    for group_field in &aggregation.group_fields {
        if let Some(group) = fields.iter().find(|field| field.key == group_field.trim()) {
            output.push(group.clone());
        }
    }
    output.extend(aggregation.metrics.iter().filter_map(|metric| {
        if metric.key.trim().is_empty() || metric.label.trim().is_empty() {
            return None;
        }
        let metric_type = fields
            .iter()
            .find(|field| field.key == metric.source_field_key.trim())
            .filter(|_| matches!(metric.function.as_str(), "min" | "max"))
            .map(|field| field.field_type.clone())
            .unwrap_or_else(|| "number".into());
        Some(DatasetFieldDefinition {
            key: metric.key.trim().into(),
            label: metric.label.trim().into(),
            field_type: metric_type,
        })
    }));
    output
}

fn dataset_field_draft_from_component_field(field: &DatasetFieldDefinition) -> DatasetFieldDraft {
    DatasetFieldDraft {
        key: field.key.clone(),
        label: field.label.clone(),
        source_alias: "component".into(),
        source_field_key: field.key.clone(),
        field_type: field.field_type.clone(),
    }
}

fn default_component_aggregation() -> DatasetAggregationDraft {
    default_component_aggregation_with_group(None)
}

fn default_component_aggregation_with_group(
    group_field: Option<String>,
) -> DatasetAggregationDraft {
    DatasetAggregationDraft {
        enabled: true,
        group_fields: group_field.into_iter().collect(),
        metrics: vec![DatasetAggregationMetricDraft {
            id: 1,
            key: "row_count".into(),
            label: "Rows".into(),
            function: "count_rows".into(),
            source_field_key: String::new(),
        }],
        row_picker: None,
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
    component_type: &str,
    columns: &str,
    aggregation: &DatasetAggregationDraft,
    detail_filter_field: &str,
    detail_filter_operator: &str,
    detail_filter_value: &str,
    pre_filter_field: &str,
    pre_filter_operator: &str,
    pre_filter_value: &str,
    post_filter_field: &str,
    post_filter_operator: &str,
    post_filter_value: &str,
    sort_field: &str,
    sort_direction: &str,
    page_size: &str,
) -> Value {
    let fields = csv_field_keys(columns);
    let defaults = table_defaults_config(sort_field, sort_direction, page_size);
    if component_type == "aggregate_table" {
        let metrics = aggregation
            .metrics
            .iter()
            .filter(|metric| {
                !metric.key.trim().is_empty()
                    && !metric.label.trim().is_empty()
                    && !metric.function.trim().is_empty()
            })
            .map(|metric| {
                let source_field_key = (metric.function != "count_rows")
                    .then(|| metric.source_field_key.trim().to_string())
                    .filter(|value| !value.is_empty());
                json!({
                    "key": metric.key.trim(),
                    "label": metric.label.trim(),
                    "function": metric.function.trim(),
                    "source_field_key": source_field_key
                })
            })
            .collect::<Vec<_>>();
        let mut config = json!({
            "group_fields": aggregation.group_fields.clone(),
            "metrics": metrics,
            "pre_filters": filter_array_config(pre_filter_field, pre_filter_operator, pre_filter_value),
            "post_filters": filter_array_config(post_filter_field, post_filter_operator, post_filter_value)
        });
        merge_table_defaults(&mut config, defaults);
        config
    } else {
        let mut config = json!({
            "columns": fields,
            "default_filters": filter_array_config(detail_filter_field, detail_filter_operator, detail_filter_value)
        });
        merge_table_defaults(&mut config, defaults);
        config
    }
}

fn filter_array_config(field: &str, operator: &str, value: &str) -> Value {
    if field.trim().is_empty() {
        return json!([]);
    }
    let operator = if operator.trim().is_empty() {
        "equals"
    } else {
        operator.trim()
    };
    let value = value
        .trim()
        .is_empty()
        .then(|| Value::Null)
        .unwrap_or_else(|| json!(value.trim()));
    json!([{
        "field_key": field.trim(),
        "operator": operator,
        "value": value
    }])
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
    json!({
        "default_sort": sort_field.trim().is_empty().then(|| Value::Null).unwrap_or_else(|| json!({
            "field_key": sort_field.trim(),
            "direction": direction
        })),
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
fn detail_columns_from_config(config: &Value) -> String {
    config
        .get("columns")
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
fn aggregate_from_config(config: &Value) -> DatasetAggregationDraft {
    let group_fields = config
        .get("group_fields")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let metrics = config
        .get("metrics")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, metric)| DatasetAggregationMetricDraft {
            id: index as u64 + 1,
            key: metric
                .get("key")
                .and_then(Value::as_str)
                .unwrap_or("metric")
                .to_string(),
            label: metric
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("Metric")
                .to_string(),
            function: metric
                .get("function")
                .and_then(Value::as_str)
                .unwrap_or("count_rows")
                .to_string(),
            source_field_key: metric
                .get("source_field_key")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        })
        .collect::<Vec<_>>();
    DatasetAggregationDraft {
        enabled: true,
        group_fields,
        metrics: if metrics.is_empty() {
            default_component_aggregation().metrics
        } else {
            metrics
        },
        row_picker: None,
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn first_filter_from_config(config: &Value, key: &str) -> (String, String, String) {
    let Some(filter) = config
        .get(key)
        .and_then(Value::as_array)
        .and_then(|filters| filters.first())
    else {
        return (String::new(), "equals".into(), String::new());
    };
    let field = filter
        .get("field_key")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let operator = filter
        .get("operator")
        .and_then(Value::as_str)
        .unwrap_or("equals")
        .to_string();
    let value = filter
        .get("value")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    (field, operator, value)
}

#[cfg(feature = "hydrate")]
fn clear_filter_controls(
    field: RwSignal<String>,
    operator: RwSignal<String>,
    value: RwSignal<String>,
) {
    field.set(String::new());
    operator.set(String::from("equals"));
    value.set(String::new());
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
fn build_component_table_query(
    search: &str,
    page_size: &str,
    cursor: &str,
    sort_field: &str,
    sort_direction: &str,
    filter_field: &str,
    filter_operator: &str,
    filter_value: &str,
    visible_columns: &str,
) -> String {
    let mut params = Vec::new();
    push_query_param(&mut params, "q", search);
    push_query_param(&mut params, "page_size", page_size);
    push_query_param(&mut params, "cursor", cursor);
    if !sort_field.trim().is_empty() {
        let direction = if sort_direction.trim() == "desc" {
            "desc"
        } else {
            "asc"
        };
        push_query_param(
            &mut params,
            "sort",
            &format!("{}:{direction}", sort_field.trim()),
        );
    }
    let filter_field = filter_field.trim();
    if !filter_field.is_empty() {
        push_query_param(
            &mut params,
            &format!("filter[{filter_field}][operator]"),
            filter_operator,
        );
        push_query_param(
            &mut params,
            &format!("filter[{filter_field}][value]"),
            filter_value,
        );
    }
    push_query_param(&mut params, "visible_columns", visible_columns);
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
    aggregation: RwSignal<DatasetAggregationDraft>,
    detail_filter_field: RwSignal<String>,
    detail_filter_operator: RwSignal<String>,
    detail_filter_value: RwSignal<String>,
    pre_filter_field: RwSignal<String>,
    pre_filter_operator: RwSignal<String>,
    pre_filter_value: RwSignal<String>,
    post_filter_field: RwSignal<String>,
    post_filter_operator: RwSignal<String>,
    post_filter_value: RwSignal<String>,
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
                    if version.component_type == "aggregate_table" {
                        columns.set(String::new());
                        clear_filter_controls(
                            detail_filter_field,
                            detail_filter_operator,
                            detail_filter_value,
                        );
                        let (field, operator, value) =
                            first_filter_from_config(&version.config, "pre_filters");
                        pre_filter_field.set(field);
                        pre_filter_operator.set(operator);
                        pre_filter_value.set(value);
                        let (field, operator, value) =
                            first_filter_from_config(&version.config, "post_filters");
                        post_filter_field.set(field);
                        post_filter_operator.set(operator);
                        post_filter_value.set(value);
                        aggregation.set(aggregate_from_config(&version.config));
                    } else {
                        columns.set(detail_columns_from_config(&version.config));
                        let (field, operator, value) =
                            first_filter_from_config(&version.config, "default_filters");
                        detail_filter_field.set(field);
                        detail_filter_operator.set(operator);
                        detail_filter_value.set(value);
                        clear_filter_controls(
                            pre_filter_field,
                            pre_filter_operator,
                            pre_filter_value,
                        );
                        clear_filter_controls(
                            post_filter_field,
                            post_filter_operator,
                            post_filter_value,
                        );
                        aggregation.set(default_component_aggregation());
                    }
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
    _: RwSignal<DatasetAggregationDraft>,
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

#[cfg(feature = "hydrate")]
fn create_component_from_form(
    editing_component_id: Option<String>,
    editing_version_id: Option<String>,
    name: String,
    slug: String,
    description: String,
    dataset_id: String,
    dataset_major: String,
    component_type: String,
    columns: String,
    aggregation: DatasetAggregationDraft,
    detail_filter_field: String,
    detail_filter_operator: String,
    detail_filter_value: String,
    pre_filter_field: String,
    pre_filter_operator: String,
    pre_filter_value: String,
    post_filter_field: String,
    post_filter_operator: String,
    post_filter_value: String,
    sort_field: String,
    sort_direction: String,
    page_size: String,
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    findings: RwSignal<Vec<ComponentValidationFinding>>,
) {
    leptos::task::spawn_local(async move {
        message.set(None);
        error.set(None);
        findings.set(Vec::new());
        let major = dataset_major.trim().parse::<i32>().unwrap_or(1);
        let config = build_component_config(
            &component_type,
            &columns,
            &aggregation,
            &detail_filter_field,
            &detail_filter_operator,
            &detail_filter_value,
            &pre_filter_field,
            &pre_filter_operator,
            &pre_filter_value,
            &post_filter_field,
            &post_filter_operator,
            &post_filter_value,
            &sort_field,
            &sort_direction,
            &page_size,
        );
        let version = CreateComponentVersionRequest {
            dataset_id: Some(dataset_id),
            dataset_version_major: Some(major),
            component_type,
            config,
        };
        let description = description
            .trim()
            .is_empty()
            .then(|| None)
            .unwrap_or_else(|| Some(description.trim().to_string()));
        let redirect_ref = component_redirect_ref(&slug);
        let result = if let Some(component_id) = editing_component_id {
            match api::update_component(
                &component_id,
                UpdateComponentRequest {
                    name,
                    slug,
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
                name,
                slug,
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
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: DatasetAggregationDraft,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Vec<ComponentValidationFinding>>,
) {
}

#[cfg(feature = "hydrate")]
fn validate_component_form(
    dataset_id: String,
    dataset_major: String,
    component_type: String,
    columns: String,
    aggregation: DatasetAggregationDraft,
    detail_filter_field: String,
    detail_filter_operator: String,
    detail_filter_value: String,
    pre_filter_field: String,
    pre_filter_operator: String,
    pre_filter_value: String,
    post_filter_field: String,
    post_filter_operator: String,
    post_filter_value: String,
    sort_field: String,
    sort_direction: String,
    page_size: String,
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    findings: RwSignal<Vec<ComponentValidationFinding>>,
) {
    leptos::task::spawn_local(async move {
        message.set(None);
        error.set(None);
        findings.set(Vec::new());
        let major = dataset_major.trim().parse::<i32>().unwrap_or(1);
        let config = build_component_config(
            &component_type,
            &columns,
            &aggregation,
            &detail_filter_field,
            &detail_filter_operator,
            &detail_filter_value,
            &pre_filter_field,
            &pre_filter_operator,
            &pre_filter_value,
            &post_filter_field,
            &post_filter_operator,
            &post_filter_value,
            &sort_field,
            &sort_direction,
            &page_size,
        );
        let payload = CreateComponentVersionRequest {
            dataset_id: Some(dataset_id),
            dataset_version_major: Some(major),
            component_type,
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
    _: String,
    _: String,
    _: String,
    _: String,
    _: DatasetAggregationDraft,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
    _: String,
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
        aggregate_from_config, aggregate_output_fields, component_redirect_ref,
        default_component_aggregation, detail_columns_from_config, editable_component_version,
        materialization_empty_state, selected_dataset_major_value, table_page_size_from_config,
        table_sort_from_config,
    };
    use super::{build_component_table_query, percent_encode_query_component};
    use crate::types::DatasetFieldDefinition;
    use tessara_web_data_ops::{DatasetAggregationDraft, DatasetAggregationMetricDraft};

    fn dataset(major_versions: Vec<i32>, current_version_major: Option<i32>) -> DatasetSummary {
        DatasetSummary {
            id: "dataset-1".into(),
            current_version_major,
            major_versions,
            name: "Dataset".into(),
            slug: "dataset".into(),
            output_fields: Vec::new(),
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

    fn aggregation_with_metrics(
        group_fields: Vec<&str>,
        metrics: Vec<(&str, &str, &str, &str)>,
    ) -> DatasetAggregationDraft {
        DatasetAggregationDraft {
            enabled: true,
            group_fields: group_fields.into_iter().map(str::to_string).collect(),
            metrics: metrics
                .into_iter()
                .enumerate()
                .map(|(index, (key, label, function, source_field_key))| {
                    DatasetAggregationMetricDraft {
                        id: index as u64 + 1,
                        key: key.into(),
                        label: label.into(),
                        function: function.into(),
                        source_field_key: source_field_key.into(),
                    }
                })
                .collect(),
            row_picker: None,
        }
    }

    #[test]
    fn detail_table_config_uses_selected_columns() {
        let config = build_component_config(
            "detail_table",
            "program, amount",
            &default_component_aggregation(),
            "program",
            "not_equals",
            "archived",
            "",
            "equals",
            "",
            "",
            "equals",
            "",
            "program",
            "desc",
            "25",
        );

        assert_eq!(config["columns"], serde_json::json!(["program", "amount"]));
        assert_eq!(config["default_filters"][0]["field_key"], "program");
        assert_eq!(config["default_filters"][0]["operator"], "not_equals");
        assert_eq!(config["default_filters"][0]["value"], "archived");
        assert_eq!(config["default_sort"]["field_key"], "program");
        assert_eq!(config["default_sort"]["direction"], "desc");
        assert_eq!(config["page_size"], 25);
    }

    #[test]
    fn aggregate_table_config_uses_group_and_metric_source() {
        let aggregation =
            aggregation_with_metrics(vec!["program"], vec![("total", "Total", "sum", "amount")]);
        let config = build_component_config(
            "aggregate_table",
            "",
            &aggregation,
            "",
            "equals",
            "",
            "program",
            "equals",
            "active",
            "total",
            "gt",
            "0",
            "program",
            "asc",
            "500",
        );

        assert_eq!(config["group_fields"], serde_json::json!(["program"]));
        assert_eq!(config["metrics"][0]["function"], "sum");
        assert_eq!(config["metrics"][0]["source_field_key"], "amount");
        assert_eq!(config["pre_filters"][0]["field_key"], "program");
        assert_eq!(config["post_filters"][0]["field_key"], "total");
        assert_eq!(config["page_size"], 200);
    }

    #[test]
    fn aggregate_count_metric_omits_source_field() {
        let aggregation = default_component_aggregation();
        let config = build_component_config(
            "aggregate_table",
            "",
            &aggregation,
            "",
            "equals",
            "",
            "",
            "equals",
            "",
            "",
            "equals",
            "",
            "",
            "asc",
            "50",
        );

        assert_eq!(config["metrics"][0]["function"], "count_rows");
        assert!(config["metrics"][0]["source_field_key"].is_null());
        assert!(config["default_sort"].is_null());
    }

    #[test]
    fn aggregate_config_supports_multiple_simultaneous_metrics() {
        let aggregation = aggregation_with_metrics(
            vec!["program"],
            vec![
                ("total_a", "Total A", "sum", "field_a"),
                ("average_b", "Average B", "average", "field_b"),
                (
                    "present_a",
                    "Count values present A",
                    "count_values",
                    "field_a",
                ),
                (
                    "present_b",
                    "Count values present B",
                    "count_values",
                    "field_b",
                ),
            ],
        );
        let config = build_component_config(
            "aggregate_table",
            "",
            &aggregation,
            "",
            "equals",
            "",
            "",
            "equals",
            "",
            "",
            "equals",
            "",
            "",
            "asc",
            "50",
        );

        assert_eq!(config["metrics"].as_array().unwrap().len(), 4);
        assert_eq!(config["metrics"][0]["function"], "sum");
        assert_eq!(config["metrics"][0]["source_field_key"], "field_a");
        assert_eq!(config["metrics"][1]["function"], "average");
        assert_eq!(config["metrics"][1]["source_field_key"], "field_b");
        assert_eq!(config["metrics"][2]["function"], "count_values");
        assert_eq!(config["metrics"][2]["source_field_key"], "field_a");
        assert_eq!(config["metrics"][3]["function"], "count_values");
        assert_eq!(config["metrics"][3]["source_field_key"], "field_b");
    }

    #[test]
    fn aggregate_output_fields_use_group_and_metric_key() {
        let fields = vec![
            DatasetFieldDefinition {
                key: "program".into(),
                label: "Program".into(),
                field_type: "text".into(),
            },
            DatasetFieldDefinition {
                key: "amount".into(),
                label: "Amount".into(),
                field_type: "number".into(),
            },
        ];

        let aggregation =
            aggregation_with_metrics(vec!["program"], vec![("total", "Total", "sum", "amount")]);
        let output = aggregate_output_fields(&fields, &aggregation);

        assert_eq!(output.len(), 2);
        assert_eq!(output[0].key, "program");
        assert_eq!(output[1].key, "total");
        assert_eq!(output[1].label, "Total");
        assert_eq!(output[1].field_type, "number");
    }

    #[test]
    fn aggregate_output_fields_keep_min_max_source_type() {
        let fields = vec![DatasetFieldDefinition {
            key: "ended_at".into(),
            label: "Ended At".into(),
            field_type: "date".into(),
        }];

        let aggregation =
            aggregation_with_metrics(Vec::new(), vec![("maximum", "Maximum", "max", "ended_at")]);
        let output = aggregate_output_fields(&fields, &aggregation);

        assert_eq!(output.len(), 1);
        assert_eq!(output[0].key, "maximum");
        assert_eq!(output[0].field_type, "date");
    }

    #[test]
    fn aggregate_output_fields_count_values_is_numeric() {
        let fields = vec![DatasetFieldDefinition {
            key: "status".into(),
            label: "Status".into(),
            field_type: "text".into(),
        }];

        let aggregation = aggregation_with_metrics(
            Vec::new(),
            vec![(
                "values_present_count",
                "Count values present",
                "count_values",
                "status",
            )],
        );
        let output = aggregate_output_fields(&fields, &aggregation);

        assert_eq!(output.len(), 1);
        assert_eq!(output[0].key, "values_present_count");
        assert_eq!(output[0].label, "Count values present");
        assert_eq!(output[0].field_type, "number");
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
    fn detail_columns_parse_string_and_object_configs() {
        let config = serde_json::json!({
            "columns": [
                "program",
                { "field_key": "amount" },
                { "key": "status" }
            ]
        });

        assert_eq!(
            detail_columns_from_config(&config),
            "program, amount, status"
        );
    }

    #[test]
    fn aggregate_config_extracts_groups_and_metrics() {
        let config = serde_json::json!({
            "group_fields": ["program", "status"],
            "metrics": [
                {
                    "key": "total",
                    "label": "Total",
                    "function": "sum",
                    "source_field_key": "amount"
                },
                {
                    "key": "present_status",
                    "label": "Count values present",
                    "function": "count_values",
                    "source_field_key": "status"
                }
            ],
            "default_sort": {
                "field_key": "program",
                "direction": "desc"
            },
            "page_size": 25
        });

        let aggregation = aggregate_from_config(&config);
        assert_eq!(aggregation.group_fields, vec!["program", "status"]);
        assert_eq!(aggregation.metrics.len(), 2);
        assert_eq!(aggregation.metrics[0].function, "sum");
        assert_eq!(aggregation.metrics[0].source_field_key, "amount");
        assert_eq!(aggregation.metrics[1].function, "count_values");
        assert_eq!(aggregation.metrics[1].source_field_key, "status");
        assert_eq!(
            table_sort_from_config(&config),
            ("program".into(), "desc".into())
        );
        assert_eq!(table_page_size_from_config(&config), "25");
    }

    #[test]
    fn component_table_query_encodes_server_driven_view_state() {
        let query = build_component_table_query(
            "family outreach",
            "25",
            "offset:25",
            "program",
            "desc",
            "row_count",
            "between",
            "1,10",
            "program, row_count",
        );

        assert_eq!(
            query,
            "q=family%20outreach&page_size=25&cursor=offset%3A25&sort=program%3Adesc&filter%5Brow_count%5D%5Boperator%5D=between&filter%5Brow_count%5D%5Bvalue%5D=1%2C10&visible_columns=program%2C%20row_count"
        );
    }

    #[test]
    fn component_table_query_omits_blank_optional_params() {
        assert_eq!(
            build_component_table_query("", "50", "", "", "asc", "", "equals", "", ""),
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
            component_type: "detail_table".into(),
            status: status.into(),
            version_label: "1".into(),
            config: serde_json::json!({ "columns": ["program"] }),
        }
    }
}
