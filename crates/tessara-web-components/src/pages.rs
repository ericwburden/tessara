//! Route-level page composition for the Components feature.

use std::collections::BTreeMap;

#[cfg(feature = "hydrate")]
use super::types::{CreateComponentRequest, CreateComponentVersionRequest, UpdateComponentRequest};
use icons::{ChevronDown, History, ListFilter, Pencil, Search, X};
use leptos::prelude::*;
use serde_json::{Value, json};
use tessara_web_data_ops::{
    DataOpsFiltersEditor, DataOpsProjectionEditor, DatasetFieldDraft as DataOpsDatasetFieldDraft,
    DatasetRowFilterDraft as DataOpsRowFilterDraft,
};
use tessara_web_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, DataTable,
    EmptyState, InteractiveDataTable, InteractiveTableColumn, InteractiveTableRow, PageHeader,
    TableFilterHeader, TablePaginationFooter,
};

#[cfg(feature = "hydrate")]
use super::api;
use super::types::{
    ComponentDefinition, ComponentSummary, ComponentTable, ComponentValidationFinding,
    ComponentVersionSummary, DatasetFieldDefinition, DatasetSummary,
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
pub fn ComponentVersionsContent(component_ref: String) -> impl IntoView {
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
                    let view_href = format!("/components/{}", component.slug);
                    view! {
                        <ComponentNestedBreadcrumb
                            component_href=view_href.clone()
                            component_label=component.name.clone()
                            current="Versions"
                        />
                        <PageHeader title=component.name.clone()>
                            <a class="button button--secondary" href=edit_href>"Edit"</a>
                            <a class="button" href=view_href>"View"</a>
                        </PageHeader>
                        <ComponentVersionsSection versions=component.versions/>
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
    let columns = RwSignal::new(Vec::<DataOpsDatasetFieldDraft>::new());
    let filters = RwSignal::new(Vec::<DataOpsRowFilterDraft>::new());
    let projection_active_source_tab = RwSignal::new(None::<String>);
    let sort_field = RwSignal::new(String::new());
    let sort_direction = RwSignal::new(String::from("asc"));
    let page_size = RwSignal::new(String::from("50"));
    let editing_component_id = RwSignal::new(None::<String>);
    let editing_version_id = RwSignal::new(None::<String>);
    let current_published_version_id = RwSignal::new(None::<String>);
    let publish_menu_open = RwSignal::new(false);
    let consumer_modal_open = RwSignal::new(false);
    let consumer_search = RwSignal::new(String::new());
    let new_version_note = RwSignal::new(String::new());

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
                    current_published_version_id,
                    name,
                    slug,
                    description,
                    dataset_id,
                    dataset_major,
                    component_type,
                    columns,
                    filters,
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
            <form class="route-panel__section form-grid component-editor-form" on:submit=move |event| {
                event.prevent_default();
                create_component_from_form(
                    editing_component_id.get_untracked(),
                    editing_version_id.get_untracked(),
                    current_published_version_id.get_untracked(),
                    ComponentPublishAction::SaveDraft,
                    None,
                    ComponentFormValues {
                        name: name.get_untracked(),
                        slug: slug.get_untracked(),
                        description: description.get_untracked(),
                        dataset_id: dataset_id.get_untracked(),
                        dataset_major: dataset_major.get_untracked(),
                        columns: columns.get_untracked(),
                        filters: filters.get_untracked(),
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
                    <input
                        prop:value=move || name.get()
                        on:change=move |event| commit_component_name(name, slug, event_target_value(&event))
                        on:blur=move |event| commit_component_name(name, slug, event_target_value(&event))
                    />
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
                            columns.set(Vec::new());
                            filters.set(Vec::new());
                            sort_field.set(String::new());
                        }
                    }>
                        <option value="" prop:selected=move || selected_dataset_major_value(&dataset_id.get(), &dataset_major.get()).is_empty()>"Select a Dataset version"</option>
                        {move || datasets.get().into_iter().flat_map(|dataset| {
                            dataset_picker_majors(&dataset).into_iter().map(move |major| {
                                let value = format!("{}|{}", dataset.id, major);
                                let selected_value = value.clone();
                                let label = dataset_catalog_option_label(&dataset, major);
                                view! {
                                    <option
                                        value=value
                                        prop:selected=move || selected_dataset_major_value(&dataset_id.get(), &dataset_major.get()) == selected_value
                                    >
                                        {label}
                                    </option>
                                }
                            }).collect::<Vec<_>>()
                        }).collect_view()}
                    </select>
                </label>
                <div class="component-editor__dataset-subpanels">
                    <DatasetCatalogContext dataset=Signal::derive(move || selected_dataset.get())/>
                    <TableDefaultsControls
                        fields=Signal::derive(move || selected_fields.get())
                        sort_field
                        sort_direction
                        page_size
                    />
                </div>
                <DataOpsProjectionEditor
                    available_fields=Signal::derive(move || {
                        selected_fields
                            .get()
                            .into_iter()
                            .map(|field| component_data_ops_field(&field))
                            .collect::<Vec<_>>()
                    })
                    fields=Signal::derive(move || columns.get())
                    active_source_tab=Signal::derive(move || projection_active_source_tab.get())
                    on_active_source_tab_change=Callback::new(move |tab| projection_active_source_tab.set(tab))
                    on_fields_change=Callback::new(move |fields| columns.set(fields))
                    title="Displayed Fields"
                    collapsible=true
                    initially_open=false
                />
                <DataOpsFiltersEditor
                    fields=Signal::derive(move || {
                        selected_fields
                            .get()
                            .into_iter()
                            .map(|field| component_data_ops_field(&field))
                            .collect::<Vec<_>>()
                    })
                    row_filters=Signal::derive(move || filters.get())
                    on_row_filters_change=Callback::new(move |row_filters| filters.set(row_filters))
                    title="Default Filters"
                    collapsible=true
                    initially_open=false
                />
                <div class="form-actions">
                    {move || editing_version_id.get().map(|version_id| {
                        let component_id = editing_component_id.get_untracked().unwrap_or_default();
                        view! {
                            <button class="button button--danger" type="button" on:click=move |_| {
                                delete_component_draft(
                                    component_id.clone(),
                                    version_id.clone(),
                                    message,
                                    error,
                                    validation_findings,
                                );
                            }>"Delete Draft"</button>
                        }
                    })}
                    <button class="button button--secondary" type="submit">"Save Draft"</button>
                    <div class=move || if publish_menu_open.get() {
                        "dropdown-menu component-editor__publish-menu is-open"
                    } else {
                        "dropdown-menu component-editor__publish-menu"
                    }>
                        <button
                            class="button component-editor__publish-button"
                            type="button"
                            aria-expanded=move || publish_menu_open.get().to_string()
                            on:click=move |_| publish_menu_open.update(|open| *open = !*open)
                        >
                            "Save and Publish"
                            <ChevronDown class="button__icon"/>
                        </button>
                        <button
                            class="dropdown-menu__scrim"
                            type="button"
                            aria-label="Close publish options"
                            on:click=move |_| publish_menu_open.set(false)
                        ></button>
                        <div class="dropdown-menu__content component-editor__publish-options" role="menu">
                            <button
                                class="dropdown-menu__item"
                                type="button"
                                role="menuitem"
                                disabled=move || current_published_version_id.get().is_none()
                                on:click=move |_| {
                                publish_menu_open.set(false);
                                create_component_from_form(
                                    editing_component_id.get_untracked(),
                                    editing_version_id.get_untracked(),
                                    current_published_version_id.get_untracked(),
                                    ComponentPublishAction::UpdateExistingVersion,
                                    None,
                                    ComponentFormValues {
                                        name: name.get_untracked(),
                                        slug: slug.get_untracked(),
                                        description: description.get_untracked(),
                                        dataset_id: dataset_id.get_untracked(),
                                        dataset_major: dataset_major.get_untracked(),
                                        columns: columns.get_untracked(),
                                        filters: filters.get_untracked(),
                                        sort_field: sort_field.get_untracked(),
                                        sort_direction: sort_direction.get_untracked(),
                                        page_size: page_size.get_untracked(),
                                    },
                                    message,
                                    error,
                                    validation_findings,
                                );
                            }>
                                "Update Existing Version"
                            </button>
                            <button class="dropdown-menu__item" type="button" role="menuitem" on:click=move |_| {
                                publish_menu_open.set(false);
                                consumer_search.set(String::new());
                                new_version_note.set(String::new());
                                consumer_modal_open.set(true);
                            }>
                                "Create New Version"
                            </button>
                        </div>
                    </div>
                </div>
            </form>
            {move || consumer_modal_open.get().then(|| {
                view! {
                    <section class="component-consumers-modal" aria-label="Review component consumers">
                        <button
                            class="component-consumers-modal__scrim"
                            type="button"
                            aria-label="Close consumer review"
                            on:click=move |_| consumer_modal_open.set(false)
                        ></button>
                        <aside class="component-consumers-modal__panel blurred-surface" role="dialog" aria-modal="true" aria-label="Review component consumers">
                            <header class="component-consumers-modal__header">
                                <div>
                                    <p class="eyebrow">"Create New Version"</p>
                                    <h2>"Review Consumers"</h2>
                                </div>
                                <button
                                    class="icon-button"
                                    type="button"
                                    aria-label="Close consumer review"
                                    on:click=move |_| consumer_modal_open.set(false)
                                >
                                    <X class="icon-button__icon"/>
                                </button>
                            </header>
                            <p class="component-consumers-modal__intro">
                                "Consumers pinned to the current version can be repinned to the new version. Deselect any consumer that should remain on the current version."
                            </p>
                            <label class="form-field component-consumers-modal__search">
                                <span>"Search consumers"</span>
                                <input
                                    type="search"
                                    placeholder="Search by dashboard, report, or placement"
                                    prop:value=move || consumer_search.get()
                                    on:input=move |event| consumer_search.set(event_target_value(&event))
                                />
                            </label>
                            <div class="component-consumers-modal__list" role="list">
                                <EmptyState
                                    title="No consumers found"
                                    message="No dashboards or reports are currently pinned to this component version. Creating a new version will not repin any consumers yet."
                                />
                            </div>
                            <label class="form-field component-consumers-modal__note">
                                <span>"New Version Note"</span>
                                <textarea
                                    placeholder="Summarize what changed in this version"
                                    prop:value=move || new_version_note.get()
                                    on:input=move |event| new_version_note.set(event_target_value(&event))
                                ></textarea>
                            </label>
                            <footer class="component-consumers-modal__footer">
                                <button
                                    class="button button--secondary"
                                    type="button"
                                    on:click=move |_| consumer_modal_open.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button class="button" type="button" on:click=move |_| {
                                    consumer_modal_open.set(false);
                                    create_component_from_form(
                                        editing_component_id.get_untracked(),
                                        editing_version_id.get_untracked(),
                                        current_published_version_id.get_untracked(),
                                        ComponentPublishAction::CreateNewVersion,
                                        Some(new_version_note.get_untracked()),
                                        ComponentFormValues {
                                            name: name.get_untracked(),
                                            slug: slug.get_untracked(),
                                            description: description.get_untracked(),
                                            dataset_id: dataset_id.get_untracked(),
                                            dataset_major: dataset_major.get_untracked(),
                                            columns: columns.get_untracked(),
                                            filters: filters.get_untracked(),
                                            sort_field: sort_field.get_untracked(),
                                            sort_direction: sort_direction.get_untracked(),
                                            page_size: page_size.get_untracked(),
                                        },
                                        message,
                                        error,
                                        validation_findings,
                                    );
                                }>
                                    "Create New Version"
                                </button>
                            </footer>
                        </aside>
                    </section>
                }
            })}
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
    let component = RwSignal::new(None::<ComponentDefinition>);
    let component_loading = RwSignal::new(true);
    let component_error = RwSignal::new(None::<String>);
    let table = RwSignal::new(None::<ComponentTable>);
    let error = RwSignal::new(None::<String>);
    let component_ref_for_title = component_ref.clone();

    Effect::new({
        let component_ref = component_ref.clone();
        move |_| {
            load_component(
                component_ref.clone(),
                component,
                component_loading,
                component_error,
            )
        }
    });
    Effect::new({
        let component_ref = component_ref.clone();
        move |_| load_component_table(component_ref.clone(), String::new(), table, error)
    });

    view! {
        <section class="route-panel components-page">
            <ComponentViewerBreadcrumb component_ref=component_ref.clone() component=component/>
            <header class="page-header">
                <div>
                    <h1>{move || {
                        component
                            .get()
                            .map(|component| component.name)
                            .unwrap_or_else(|| component_ref_for_title.clone())
                    }}</h1>
                </div>
            </header>
            {move || {
                if component_loading.get() {
                    view! { <EmptyState title="Loading configuration" message="Fetching component configuration."/> }.into_any()
                } else if let Some(message) = component_error.get() {
                    view! { <EmptyState title="Configuration unavailable" message=message/> }.into_any()
                } else if let Some(component) = component.get() {
                    if component.versions.iter().any(|version| version.status == "published") {
                        view! { <ComponentTablePreviewSection table=table.get() table_error=error.get()/> }.into_any()
                    } else {
                        view! { <EmptyState title="No published version" message="This component does not have a published table yet."/> }.into_any()
                    }
                } else {
                    view! { <EmptyState title="Component unavailable" message="Component data could not be loaded."/> }.into_any()
                }
            }}
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
        <section class="route-panel__section component-editor__subpanel">
            {move || if let Some(dataset) = dataset.get() {
                let tags = dataset_tag_label(&dataset.tags);
                let provenance = dataset_provenance_label(&dataset.provenance);
                view! {
                    <h2>"Dataset Context"</h2>
                    <table class="info-list-table component-editor__context-table">
                        <tbody>
                            <tr><th scope="row">"Grain"</th><td>{dataset.grain}</td></tr>
                            <tr><th scope="row">"Tags"</th><td>{tags}</td></tr>
                            <tr><th scope="row">"Provenance"</th><td>{provenance}</td></tr>
                        </tbody>
                    </table>
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
fn TableDefaultsControls(
    fields: Signal<Vec<DatasetFieldDefinition>>,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    page_size: RwSignal<String>,
) -> impl IntoView {
    view! {
        <fieldset class="route-panel__section component-editor__subpanel component-editor__table-defaults">
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
fn ComponentViewerBreadcrumb(
    #[prop(into)] component_ref: String,
    component: RwSignal<Option<ComponentDefinition>>,
) -> impl IntoView {
    view! {
        <Breadcrumb>
            <BreadcrumbItem>
                <BreadcrumbLink href="/components">"Components"</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator/>
            <BreadcrumbItem>
                <BreadcrumbPage>
                    {move || {
                        component
                            .get()
                            .map(|component| component.name)
                            .unwrap_or_else(|| component_ref.clone())
                    }}
                </BreadcrumbPage>
            </BreadcrumbItem>
        </Breadcrumb>
    }
}

#[component]
fn ComponentNestedBreadcrumb(
    #[prop(into)] component_href: String,
    #[prop(into)] component_label: String,
    #[prop(into)] current: String,
) -> impl IntoView {
    view! {
        <Breadcrumb>
            <BreadcrumbItem>
                <BreadcrumbLink href="/components">"Components"</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator/>
            <BreadcrumbItem>
                <BreadcrumbLink href=component_href>{component_label}</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator/>
            <BreadcrumbItem>
                <BreadcrumbPage>{current}</BreadcrumbPage>
            </BreadcrumbItem>
        </Breadcrumb>
    }
}

#[component]
fn ComponentTablePreviewSection(
    table: Option<ComponentTable>,
    table_error: Option<String>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section component-table-preview">
            {if let Some(message) = table_error {
                view! { <EmptyState title="Preview unavailable" message=message/> }.into_any()
            } else if let Some(table) = table {
                view! { <ComponentTablePreview table/> }.into_any()
            } else {
                view! { <EmptyState title="Loading preview" message="Fetching the published table preview."/> }.into_any()
            }}
        </section>
    }
}

#[component]
fn ComponentTablePreview(table: ComponentTable) -> impl IntoView {
    let columns = table.columns.clone();
    let rows = table.rows.clone();
    let column_count = columns.len();
    let row_count = rows.len();
    let table_columns = columns
        .iter()
        .map(|column| {
            InteractiveTableColumn::new(
                column.key.clone(),
                column.label.clone(),
                sentence_label(&column.field_type),
            )
        })
        .collect::<Vec<_>>();
    let table_rows = rows
        .into_iter()
        .map(|row| {
            let values = columns
                .iter()
                .map(|column| {
                    let value = row
                        .values
                        .get(&column.key)
                        .cloned()
                        .flatten()
                        .unwrap_or_default();
                    (column.key.clone(), value)
                })
                .collect::<BTreeMap<_, _>>();
            InteractiveTableRow::new(row.row_id, values)
        })
        .collect::<Vec<_>>();

    view! {
        <div class="component-table-preview__header">
            <div>
                <h2>"Preview"</h2>
                <p>{format!("Showing {row_count} rows across {column_count} visible columns.")}</p>
            </div>
        </div>
        {if columns.is_empty() {
            view! { <EmptyState title="No visible columns" message="This component does not currently expose any table columns."/> }.into_any()
        } else if row_count == 0 {
            view! { <EmptyState title="No rows to display" message="The published table returned no rows for its current configuration."/> }.into_any()
        } else {
            view! {
                <InteractiveDataTable
                    columns=table_columns
                    rows=table_rows
                    search_label="Search component rows"
                    search_placeholder="Search table"
                    item_label="table rows"
                    empty_message="No table rows match the current controls."
                />
            }.into_any()
        }}
    }
}

#[component]
fn ComponentVersionsSection(versions: Vec<ComponentVersionSummary>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h2>"Versions"</h2>
            <DataTable>
                <thead>
                    <tr>
                        <th>"Version"</th>
                        <th>"Status"</th>
                        <th>"Kind"</th>
                        <th>"Dataset Version"</th>
                        <th>"Note"</th>
                    </tr>
                </thead>
                <tbody>
                    {versions.into_iter().map(|version| {
                        let version_note = if version.version_note.trim().is_empty() {
                            "-".to_string()
                        } else {
                            version.version_note.clone()
                        };
                        view! {
                            <tr>
                                <td>{version.version_label}</td>
                                <td>{component_status_label(&version.status)}</td>
                                <td>{component_type_label(&version.component_type)}</td>
                                <td>{format!("v{}", version.dataset_version_major)}</td>
                                <td>{version_note}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </DataTable>
        </section>
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
                                <th class="data-table__cell--center" scope="col">"Revision"</th>
                                <th class="data-table__cell--center" scope="col">
                                    <TableFilterHeader
                                        label="Status"
                                        all_label="All statuses"
                                        filter=status_filter
                                        options=table_status_options.clone()
                                    />
                                </th>
                                <th class="data-table__cell--center" scope="col">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let components = filtered_components.get();
                                if components.is_empty() {
                                    view! {
                                        <tr>
                                            <td class="data-table__empty" colspan="4">"No Components to Display"</td>
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
                                            let edit_href = format!("/components/{}/edit", component.slug);
                                            let versions_href = format!("/components/{}/versions", component.slug);
                                            let kind_label = component_summary_kind_label(&component);
                                            let status_label = component_summary_status_label(&component);
                                            let revision_label = component_summary_revision_label(&component);
                                            view! {
                                                <tr>
                                                    <th scope="row">
                                                        <a class="data-table__primary-link" href=href>{component.name}</a>
                                                    </th>
                                                    <td class="data-table__cell--center">{kind_label}</td>
                                                    <td class="data-table__cell--center">{revision_label}</td>
                                                    <td class="data-table__cell--center">{status_label}</td>
                                                    <td class="data-table__cell--center">
                                                        <div class="components-list-actions">
                                                            <a class="icon-button" href=edit_href aria-label="Edit component" title="Edit component">
                                                                <Pencil class="icon-button__icon"/>
                                                            </a>
                                                            <a class="icon-button" href=versions_href aria-label="View component versions" title="View component versions">
                                                                <History class="icon-button__icon"/>
                                                            </a>
                                                        </div>
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
                            let edit_href = format!("/components/{}/edit", component.slug);
                            let versions_href = format!("/components/{}/versions", component.slug);
                            let kind_label = component_summary_kind_label(&component);
                            let status_label = component_summary_status_label(&component);
                            let revision_label = component_summary_revision_label(&component);
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
                                        <div>
                                            <dt>"Revision"</dt>
                                            <dd>{revision_label}</dd>
                                        </div>
                                    </dl>
                                    <div class="forms-list-mobile-card__actions components-list-mobile-card__actions">
                                        <a class="icon-button" href=edit_href aria-label="Edit component" title="Edit component">
                                            <Pencil class="icon-button__icon"/>
                                        </a>
                                        <a class="icon-button" href=versions_href aria-label="View component versions" title="View component versions">
                                            <History class="icon-button__icon"/>
                                        </a>
                                    </div>
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

#[cfg_attr(not(test), allow(dead_code))]
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
        .unwrap_or("Table")
}

fn component_summary_status_label(component: &ComponentSummary) -> &'static str {
    if component.current_version_id.is_some() && component.draft_version_id.is_some() {
        "Updating"
    } else if component.current_version_id.is_some() {
        "Published"
    } else {
        "Draft"
    }
}

fn component_summary_revision_label(component: &ComponentSummary) -> String {
    component
        .current_version_label
        .as_deref()
        .or(component.draft_version_label.as_deref())
        .map(|label| format!("v{label}"))
        .unwrap_or_else(|| "Draft".into())
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

fn component_data_ops_field(field: &DatasetFieldDefinition) -> DataOpsDatasetFieldDraft {
    DataOpsDatasetFieldDraft {
        key: field.key.clone(),
        label: field.label.clone(),
        source_alias: "dataset".into(),
        source_field_key: field.key.clone(),
        field_type: field.field_type.clone(),
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

fn sentence_label(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

#[cfg_attr(not(test), allow(dead_code))]
fn toggle_csv_key(value: &mut String, key: &str) {
    let mut keys = csv_field_keys(value);
    if keys.iter().any(|existing| existing == key) {
        keys.retain(|existing| existing != key);
    } else {
        keys.push(key.to_string());
    }
    *value = keys.join(", ");
}

#[cfg_attr(not(test), allow(dead_code))]
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
    columns: &[DataOpsDatasetFieldDraft],
    filters: &[DataOpsRowFilterDraft],
    sort_field: &str,
    sort_direction: &str,
    page_size: &str,
) -> Value {
    let defaults = table_defaults_config(sort_field, sort_direction, page_size);
    let display_labels = columns
        .iter()
        .map(|field| (field.key.clone(), Value::String(field.label.clone())))
        .collect::<serde_json::Map<_, _>>();
    let filters = filters
        .iter()
        .filter(|filter| !filter.field_key.trim().is_empty())
        .map(table_filter_config)
        .collect::<Vec<_>>();
    let mut config = json!({
        "visible_columns": columns
            .iter()
            .map(|field| field.key.clone())
            .collect::<Vec<_>>(),
        "display_labels": display_labels,
        "filters": filters
    });
    merge_table_defaults(&mut config, defaults);
    config
}

fn table_filter_config(filter: &DataOpsRowFilterDraft) -> Value {
    let mut filter_config = serde_json::Map::new();
    filter_config.insert("field_key".into(), Value::String(filter.field_key.clone()));
    filter_config.insert("operator".into(), Value::String(filter.operator.clone()));
    if filter.value_mode == "field" {
        filter_config.insert(
            "value_field_key".into(),
            Value::String(filter.value_field_key.clone()),
        );
    } else if !filter.value.trim().is_empty() {
        filter_config.insert("value".into(), Value::String(filter.value.clone()));
    }
    Value::Object(filter_config)
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
fn table_projection_fields_from_config_keys(config: &Value) -> Vec<DataOpsDatasetFieldDraft> {
    let labels = config
        .get("display_labels")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    csv_field_keys(&table_visible_columns_from_config(config))
        .into_iter()
        .map(|key| DataOpsDatasetFieldDraft {
            label: labels
                .get(&key)
                .and_then(Value::as_str)
                .unwrap_or(&key)
                .into(),
            source_alias: "dataset".into(),
            source_field_key: key.clone(),
            field_type: String::new(),
            key,
        })
        .collect()
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
fn table_filter_drafts_from_config(config: &Value) -> Vec<DataOpsRowFilterDraft> {
    config
        .get("filters")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(index, filter)| {
            let field_key = filter.get("field_key").and_then(Value::as_str)?.trim();
            let operator = filter.get("operator").and_then(Value::as_str)?.trim();
            if field_key.is_empty() || operator.is_empty() {
                return None;
            }
            let value_field_key = filter
                .get("value_field_key")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            Some(DataOpsRowFilterDraft {
                id: (index as u64) + 1,
                field_key: field_key.into(),
                operator: operator.into(),
                value: filter
                    .get("value")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .into(),
                value_mode: if value_field_key.is_empty() {
                    "value".into()
                } else {
                    "field".into()
                },
                value_field_key,
            })
        })
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
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

#[allow(dead_code)]
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

#[cfg_attr(not(test), allow(dead_code))]
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

#[cfg_attr(not(test), allow(dead_code))]
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
        match api::fetch_admin_components().await {
            Ok(Some(response)) => components.set(response),
            Ok(None) => components.set(Vec::new()),
            Err(_) => match api::fetch_components().await {
                Ok(Some(response)) => components.set(response),
                Ok(None) => components.set(Vec::new()),
                Err(message) => load_error.set(Some(message)),
            },
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
    current_published_version_id: RwSignal<Option<String>>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    description: RwSignal<String>,
    dataset_id: RwSignal<String>,
    dataset_major: RwSignal<String>,
    component_type: RwSignal<String>,
    columns: RwSignal<Vec<DataOpsDatasetFieldDraft>>,
    filters: RwSignal<Vec<DataOpsRowFilterDraft>>,
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
                current_published_version_id.set(
                    component
                        .versions
                        .iter()
                        .find(|version| version.status == "published")
                        .map(|version| version.id.clone()),
                );
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
                    columns.set(table_projection_fields_from_config_keys(&version.config));
                    filters.set(table_filter_drafts_from_config(&version.config));
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
    _: RwSignal<Option<String>>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<Vec<DataOpsDatasetFieldDraft>>,
    _: RwSignal<Vec<DataOpsRowFilterDraft>>,
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
    columns: Vec<DataOpsDatasetFieldDraft>,
    filters: Vec<DataOpsRowFilterDraft>,
    sort_field: String,
    sort_direction: String,
    page_size: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComponentPublishAction {
    SaveDraft,
    UpdateExistingVersion,
    CreateNewVersion,
}

#[cfg(feature = "hydrate")]
fn create_component_from_form(
    editing_component_id: Option<String>,
    editing_version_id: Option<String>,
    current_published_version_id: Option<String>,
    publish_action: ComponentPublishAction,
    version_note: Option<String>,
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
            &values.filters,
            &values.sort_field,
            &values.sort_direction,
            &values.page_size,
        );
        let version = CreateComponentVersionRequest {
            dataset_id: Some(values.dataset_id),
            dataset_version_major: Some(major),
            component_type: "table".into(),
            config,
            version_note: normalized_component_version_note(version_note),
        };
        match api::validate_component_version(version.clone()).await {
            Ok(response) if response.valid => {}
            Ok(response) => {
                findings.set(response.findings);
                return;
            }
            Err(message) => {
                error.set(Some(message));
                return;
            }
        }
        let description = if values.description.trim().is_empty() {
            None
        } else {
            Some(values.description.trim().to_string())
        };
        let redirect_ref = component_redirect_ref(&values.slug);
        if publish_action == ComponentPublishAction::UpdateExistingVersion {
            let Some(component_id) = editing_component_id.clone() else {
                error.set(Some(
                    "Save the component draft before updating an existing version.".into(),
                ));
                return;
            };
            let Some(version_id) = current_published_version_id.clone() else {
                error.set(Some(
                    "This component does not have an existing published version to update.".into(),
                ));
                return;
            };
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
                Ok(_) => match api::update_published_component_version(
                    &component_id,
                    &version_id,
                    version,
                )
                .await
                {
                    Ok(_) => {
                        message.set(Some("Existing component version updated.".into()));
                        if let Some(window) = web_sys::window() {
                            let _ = window
                                .location()
                                .set_href(&format!("/components/{redirect_ref}"));
                        }
                    }
                    Err(message) => error.set(Some(message)),
                },
                Err(message) => error.set(Some(message)),
            }
            return;
        }

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
                    let version_result = if let Some(version_id) = editing_version_id {
                        api::update_component_version(&component_id, &version_id, version).await
                    } else {
                        api::save_component_version(&component_id, version).await
                    };
                    version_result.map(|response| (component_id, response.id))
                }
                Err(message) => Err(message),
            }
        } else {
            match api::create_component(CreateComponentRequest {
                name: values.name,
                slug: values.slug,
                description,
                version: Some(version),
            })
            .await
            {
                Ok(response) => {
                    let component_id = response.id;
                    if publish_action == ComponentPublishAction::CreateNewVersion {
                        match api::fetch_admin_component(&redirect_ref).await {
                            Ok(Some(component)) => component
                                .versions
                                .iter()
                                .find(|version| version.status == "draft")
                                .map(|version| (component_id, version.id.clone()))
                                .ok_or_else(|| {
                                    "Component draft could not be found after saving.".to_string()
                                }),
                            Ok(None) => {
                                Err("Component draft could not be loaded after saving.".to_string())
                            }
                            Err(message) => Err(message),
                        }
                    } else {
                        Ok((component_id, String::new()))
                    }
                }
                Err(message) => Err(message),
            }
        };
        match result {
            Ok((component_id, version_id)) => {
                if publish_action == ComponentPublishAction::CreateNewVersion {
                    match api::publish_component_version(&component_id, &version_id).await {
                        Ok(_) => {
                            message.set(Some("Component saved and published.".into()));
                            if let Some(window) = web_sys::window() {
                                let _ = window
                                    .location()
                                    .set_href(&format!("/components/{redirect_ref}"));
                            }
                        }
                        Err(message) => error.set(Some(message)),
                    }
                } else {
                    message.set(Some("Component draft saved.".into()));
                    if let Some(window) = web_sys::window() {
                        let _ = window
                            .location()
                            .set_href(&format!("/components/{redirect_ref}/edit"));
                    }
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

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn normalized_component_version_note(note: Option<String>) -> Option<String> {
    note.map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn commit_component_name(name: RwSignal<String>, slug: RwSignal<String>, value: String) {
    let derived_slug = snake_case_component_slug(&value);
    name.set(value);

    if slug.get_untracked().trim().is_empty() && !derived_slug.is_empty() {
        slug.set(derived_slug);
    }
}

fn snake_case_component_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !slug.is_empty() {
            slug.push('_');
            previous_was_separator = true;
        }
    }

    slug.trim_end_matches('_').to_string()
}

#[cfg(not(feature = "hydrate"))]
fn create_component_from_form(
    _: Option<String>,
    _: Option<String>,
    _: Option<String>,
    _: ComponentPublishAction,
    _: Option<String>,
    _: ComponentFormValues,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Vec<ComponentValidationFinding>>,
) {
}

#[cfg(feature = "hydrate")]
fn delete_component_draft(
    component_id: String,
    version_id: String,
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    findings: RwSignal<Vec<ComponentValidationFinding>>,
) {
    let confirmed = web_sys::window()
        .and_then(|window| {
            window
                .confirm_with_message(
                    "Delete this component draft? Published versions will remain available.",
                )
                .ok()
        })
        .unwrap_or(false);
    if !confirmed {
        return;
    }

    leptos::task::spawn_local(async move {
        message.set(None);
        error.set(None);
        findings.set(Vec::new());
        match api::delete_component_version(&component_id, &version_id).await {
            Ok(_) => {
                message.set(Some("Component draft deleted.".into()));
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/components");
                }
            }
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn delete_component_draft(
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
            version_note: None,
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
        snake_case_component_slug, table_page_size_from_config, table_sort_from_config,
        table_visible_columns_from_config,
    };
    use crate::types::{ComponentSummary, DatasetProvenanceItem, DatasetProvenanceSummary};
    use tessara_web_data_ops::{
        DatasetFieldDraft as DataOpsDatasetFieldDraft,
        DatasetRowFilterDraft as DataOpsRowFilterDraft,
    };

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
            current_version_label: published.then(|| "1".into()),
            current_component_type: component_type.map(str::to_string),
            draft_version_id: (!published).then(|| format!("{name}-draft")),
            draft_version_label: (!published).then(|| "1".into()),
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
        let mut updating_component = component_summary("Program Update", Some("table"), true);
        updating_component.draft_version_id = Some("Program Update-draft".into());
        updating_component.draft_version_label = Some("2".into());
        let components = vec![
            published_table.clone(),
            draft_component.clone(),
            updating_component.clone(),
        ];

        assert_eq!(component_kind_filter_options(&components), vec!["Table"]);
        assert_eq!(
            component_status_filter_options(&components),
            vec!["Draft", "Published", "Updating"]
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
            "Table",
            "Draft"
        ));
        assert!(component_matches_filters(
            &updating_component,
            "program",
            "Table",
            "Updating"
        ));
    }

    #[test]
    fn table_config_uses_visible_columns_and_defaults() {
        let config = build_component_config(
            &projection_fields(&["program", "amount"]),
            &[filter_draft(1, "program", "equals", "Afterschool")],
            "program",
            "desc",
            "25",
        );

        assert_eq!(
            config["visible_columns"],
            serde_json::json!(["program", "amount"])
        );
        assert_eq!(config["display_labels"]["program"], "program");
        assert_eq!(config["default_sort"]["field_key"], "program");
        assert_eq!(config["default_sort"]["direction"], "desc");
        assert_eq!(config["page_size"], 25);
        assert_eq!(config["filters"][0]["field_key"], "program");
        assert_eq!(config["filters"][0]["operator"], "equals");
        assert_eq!(config["filters"][0]["value"], "Afterschool");
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
    fn snake_case_component_slug_normalizes_component_names() {
        assert_eq!(
            snake_case_component_slug("UAT Table Component"),
            "uat_table_component"
        );
        assert_eq!(
            snake_case_component_slug(" Demo Partner: Snapshot 2026 "),
            "demo_partner_snapshot_2026"
        );
        assert_eq!(snake_case_component_slug("Already_snake"), "already_snake");
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
            version_note: String::new(),
            config: serde_json::json!({ "visible_columns": ["program"] }),
        }
    }

    fn projection_fields(keys: &[&str]) -> Vec<DataOpsDatasetFieldDraft> {
        keys.iter()
            .map(|key| DataOpsDatasetFieldDraft {
                key: (*key).into(),
                label: (*key).into(),
                source_alias: "dataset".into(),
                source_field_key: (*key).into(),
                field_type: "text".into(),
            })
            .collect()
    }

    fn filter_draft(
        id: u64,
        field_key: &str,
        operator: &str,
        value: &str,
    ) -> DataOpsRowFilterDraft {
        DataOpsRowFilterDraft {
            id,
            field_key: field_key.into(),
            operator: operator.into(),
            value: value.into(),
            value_mode: "value".into(),
            value_field_key: String::new(),
        }
    }
}
