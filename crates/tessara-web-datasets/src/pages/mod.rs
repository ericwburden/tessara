//! Content composition for the Datasets feature.
//!
//! Keep Leptos feature content here; route parameters and shell composition belong in the root route adapters.

use leptos::prelude::*;

use crate::text::text_matches;
use tessara_web_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, DataTable,
    EmptyState, PageHeader,
};

#[cfg(feature = "hydrate")]
use super::api;
use super::components::{DatasetDetailSurface, DatasetDirectoryTable, DatasetPreviewTable};
use super::editor::DatasetEditorSurface;
use super::loaders::*;
use super::permissions::can_manage_datasets;
use super::types::*;

#[component]
pub fn DatasetsIndexContent() -> impl IntoView {
    let datasets = RwSignal::new(Vec::<DatasetSummary>::new());
    let account = RwSignal::new(None::<SessionAccount>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let search = RwSignal::new(String::new());
    let page_index = RwSignal::new(0usize);
    let page_size = RwSignal::new(10usize);

    Effect::new(move |_| {
        load_account(account);
        load_datasets(datasets, is_loading, load_error);
    });

    let filtered = Memo::new(move |_| {
        let query = search.get();
        datasets
            .get()
            .into_iter()
            .filter(|dataset| {
                text_matches(
                    &query,
                    &[
                        dataset.name.as_str(),
                        dataset.slug.as_str(),
                        dataset.grain.as_str(),
                    ],
                )
            })
            .collect::<Vec<_>>()
    });
    let can_manage = move || {
        account
            .get()
            .is_some_and(|account| can_manage_datasets(&account))
    };

    view! {
        <section class="route-panel datasets-page">
            <PageHeader title="Datasets">
                {move || if can_manage() {
                    view! { <a class="button" href="/datasets/new">"Create Dataset"</a> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </PageHeader>

            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading datasets" message="Fetching visible datasets."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Datasets unavailable" message=message/> }.into_any()
                } else if datasets.get().is_empty() {
                    view! { <EmptyState title="No visible datasets" message="No datasets are visible for the current account."/> }.into_any()
                } else {
                    view! {
                        <DatasetDirectoryTable
                            datasets=filtered.get()
                            search
                            page_index
                            page_size
                        />
                    }.into_any()
                }
            }}
        </section>
    }
}

#[component]
pub fn DatasetDetailContent(dataset_id: String) -> impl IntoView {
    view! { <DatasetDetailSurface dataset_id edit=false/> }
}

#[component]
pub fn DatasetRevisionHistoryContent(dataset_id: String) -> impl IntoView {
    let dataset = RwSignal::new(None::<DatasetDefinition>);
    let revisions = RwSignal::new(Vec::<DatasetRevisionSummary>::new());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let dataset_is_loading = RwSignal::new(true);
    let dataset_load_error = RwSignal::new(None::<String>);
    let detail_href = format!("/datasets/{dataset_id}");
    let detail_href_for_breadcrumb = detail_href.clone();

    Effect::new({
        let dataset_id = dataset_id.clone();
        move |_| {
            load_dataset_detail(
                dataset_id.clone(),
                dataset,
                dataset_is_loading,
                dataset_load_error,
            );
            load_revision_history(dataset_id.clone(), revisions, is_loading, load_error);
        }
    });

    view! {
        <section class="route-panel datasets-page">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/datasets">"Datasets"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem>
                    <BreadcrumbLink href=detail_href_for_breadcrumb>"Dataset Detail"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem>
                    <BreadcrumbPage>"Revision History"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>
            {move || {
                let title = dataset
                    .get()
                    .map(|dataset| dataset.name)
                    .unwrap_or_else(|| "Dataset Revisions".to_string());
                view! { <PageHeader title/> }
            }}
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading revisions" message="Fetching dataset revision history."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Revision history unavailable" message=message/> }.into_any()
                } else if revisions.get().is_empty() {
                    view! { <EmptyState title="No revisions" message="This dataset does not have revisions yet."/> }.into_any()
                } else {
                    view! {
                        <section class="route-panel__section">
                            <DataTable>
                                <thead>
                                    <tr>
                                        <th>"Version"</th>
                                        <th>"Status"</th>
                                        <th>"Fields"</th>
                                        <th>"Compatibility"</th>
                                        <th>"Dependencies"</th>
                                        <th>"Published"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {revisions.get().into_iter().map(|revision| {
                                        let href = format!("/datasets/{}/revisions/{}", revision.dataset_id, revision.id);
                                        let label = revision_label_text(revision.version_number, &revision.version_label);
                                        let version = semantic_version_label(
                                            revision.version_major,
                                            revision.version_minor,
                                            revision.version_patch,
                                        ).unwrap_or_else(|| format!("Revision {}", revision.version_number));
                                        view! {
                                            <tr>
                                                <td class="data-table__stacked-label">
                                                    <a href=href>{version}</a>
                                                    {label.map(|label| view! { <span class="data-table__secondary-text">{label}</span> })}
                                                </td>
                                                <td>{revision_status_label(&revision.status, revision.is_current)}</td>
                                                <td>{revision.output_field_count}</td>
                                                <td>{compatibility_label(&revision.compatibility)}</td>
                                                <td>{dependency_label(&revision.dependencies)}</td>
                                                <td>{revision.published_at.unwrap_or_else(|| "Not published".into())}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </DataTable>
                        </section>
                    }.into_any()
                }
            }}
        </section>
    }
}

#[component]
pub fn DatasetRevisionDetailContent(dataset_id: String, revision_id: String) -> impl IntoView {
    let revision = RwSignal::new(None::<DatasetRevisionDetail>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let publish_error = RwSignal::new(None::<String>);
    let publish_message = RwSignal::new(None::<String>);
    let label_draft = RwSignal::new(String::new());
    let notes_draft = RwSignal::new(String::new());
    let label_loaded_revision_id = RwSignal::new(None::<String>);
    let label_error = RwSignal::new(None::<String>);
    let label_message = RwSignal::new(None::<String>);
    let options_error = RwSignal::new(None::<String>);
    let delete_error = RwSignal::new(None::<String>);
    let delete_message = RwSignal::new(None::<String>);

    Effect::new({
        let dataset_id = dataset_id.clone();
        let revision_id = revision_id.clone();
        move |_| {
            load_revision_detail(
                dataset_id.clone(),
                revision_id.clone(),
                revision,
                is_loading,
                load_error,
            );
        }
    });

    Effect::new(move |_| {
        if let Some(loaded) = revision.get() {
            let revision_id = loaded.id.clone();
            if label_loaded_revision_id.get_untracked().as_deref() != Some(revision_id.as_str()) {
                label_draft.set(
                    revision_label_text(loaded.version_number, &loaded.version_label)
                        .unwrap_or_default(),
                );
                notes_draft.set(loaded.revision_notes);
                label_loaded_revision_id.set(Some(revision_id));
            }
        }
    });

    view! {
        <section class="route-panel datasets-page">
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading revision" message="Fetching dataset revision detail."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Revision unavailable" message=message/> }.into_any()
                } else if let Some(loaded) = revision.get() {
                    let detail_href = format!("/datasets/{}", loaded.dataset_id);
                    let history_href_for_breadcrumb = format!("/datasets/{}/revisions", loaded.dataset_id);
                    let status_label = revision_status_label(&loaded.status, loaded.is_current);
                    let compatibility_state = compatibility_state_label(&loaded.compatibility);
                    let dependency_summary = dependency_label(&loaded.dependencies);
                    let dependency_state = carry_forward_label(&loaded.dependencies.carry_forward_state);
                    let row_count = loaded
                        .materialized_row_count
                        .map(|count| count.to_string())
                        .unwrap_or_else(|| "Not materialized".into());
                    let major_count = loaded.compatibility.major_count;
                    let minor_count = loaded.compatibility.minor_count;
                    let patch_count = loaded.compatibility.patch_count;
                    let revision_label = revision_label_text(loaded.version_number, &loaded.version_label)
                        .unwrap_or_else(|| "No label".into());
                    let revision_notes = loaded.revision_notes.clone();
                    let label_dataset_id = loaded.dataset_id.clone();
                    let label_revision_id = loaded.id.clone();
                    let is_draft = loaded.status == DatasetRevisionStatus::Draft;
                    let publish_dataset_id = loaded.dataset_id.clone();
                    let publish_revision_id = loaded.id.clone();
                    let delete_dataset_id = loaded.dataset_id.clone();
                    let delete_revision_id = loaded.id.clone();
                    let edit_href = format!("/datasets/{}/revisions/{}/edit", loaded.dataset_id, loaded.id);
                    let version = semantic_version_label(
                        loaded.version_major,
                        loaded.version_minor,
                        loaded.version_patch,
                    ).unwrap_or_else(|| format!("Revision {}", loaded.version_number));
                    view! {
                        <Breadcrumb>
                            <BreadcrumbItem>
                                <BreadcrumbLink href="/datasets">"Datasets"</BreadcrumbLink>
                            </BreadcrumbItem>
                            <BreadcrumbSeparator/>
                            <BreadcrumbItem>
                                <BreadcrumbLink href=detail_href>"Dataset Detail"</BreadcrumbLink>
                            </BreadcrumbItem>
                            <BreadcrumbSeparator/>
                            <BreadcrumbItem>
                                <BreadcrumbLink href=history_href_for_breadcrumb>"Revision History"</BreadcrumbLink>
                            </BreadcrumbItem>
                            <BreadcrumbSeparator/>
                            <BreadcrumbItem>
                                <BreadcrumbPage>"Dataset Revision"</BreadcrumbPage>
                            </BreadcrumbItem>
                        </Breadcrumb>
                        <PageHeader title=loaded.metadata.name.clone()/>
                        {move || publish_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
                        {move || publish_message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
                        {move || options_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
                        {move || delete_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
                        {move || delete_message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
                        {if is_draft {
                            view! {
                                <section class="route-panel__section">
                                    <div class="button-row dataset-revision-actions">
                                        <a class="button button--secondary" href=edit_href>"Edit Revision"</a>
                                        <button class="button button--danger" type="button" on:click=move |_| {
                                            delete_dataset_revision(
                                                delete_dataset_id.clone(),
                                                delete_revision_id.clone(),
                                                delete_error,
                                                delete_message,
                                            );
                                        }>"Delete Revision"</button>
                                        <PublishRevisionMenu
                                            dataset_id=publish_dataset_id
                                            revision_id=publish_revision_id
                                            revision
                                            options_error
                                            publish_error
                                            publish_message
                                        />
                                    </div>
                                </section>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                        <section class="route-panel__section">
                            <form class="dataset-revision-notes" on:submit=move |event| {
                                event.prevent_default();
                                update_revision_label(
                                    label_dataset_id.clone(),
                                    label_revision_id.clone(),
                                    label_draft.get_untracked(),
                                    notes_draft.get_untracked(),
                                    revision,
                                    label_error,
                                    label_message,
                                );
                            }>
                                <label class="form-field">
                                    <span>"Revision Label"</span>
                                    <input
                                        maxlength="80"
                                        aria-label="Revision label"
                                        placeholder=revision_label
                                        prop:value=move || label_draft.get()
                                        on:change=move |event| label_draft.set(event_target_value(&event))
                                        on:input=move |event| label_draft.set(event_target_value(&event))
                                    />
                                </label>
                                <label class="form-field form-field--wide">
                                    <span>"Revision Notes"</span>
                                    <textarea
                                        maxlength="2000"
                                        aria-label="Revision notes"
                                        placeholder="Add notes for this revision"
                                        prop:value=move || notes_draft.get()
                                        on:change=move |event| notes_draft.set(event_target_value(&event))
                                        on:input=move |event| notes_draft.set(event_target_value(&event))
                                    >{revision_notes}</textarea>
                                </label>
                                <div class="form-actions form-actions--compact">
                                    <button class="button button--secondary" type="submit">"Save Revision Details"</button>
                                </div>
                            </form>
                            {move || label_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
                            {move || label_message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
                        </section>
                        <section class="route-panel__section">
                            <DataTable>
                                <tbody>
                                    <tr>
                                        <th scope="row">"Version"</th>
                                        <td>{version}</td>
                                    </tr>
                                    <tr>
                                        <th scope="row">"Status"</th>
                                        <td>{status_label}</td>
                                    </tr>
                                    <tr>
                                        <th scope="row">"Compatibility"</th>
                                        <td>{compatibility_state}</td>
                                    </tr>
                                    <tr>
                                        <th scope="row">"Changelog"</th>
                                        <td>{format!("{major_count} major · {minor_count} minor · {patch_count} patch")}</td>
                                    </tr>
                                    <tr>
                                        <th scope="row">"Dependency Review"</th>
                                        <td>{dependency_state}</td>
                                    </tr>
                                    <tr>
                                        <th scope="row">"Downstream Dependencies"</th>
                                        <td>{dependency_summary}</td>
                                    </tr>
                                    <tr>
                                        <th scope="row">"Rows"</th>
                                        <td>{row_count}</td>
                                    </tr>
                                </tbody>
                            </DataTable>
                        </section>
                        <RevisionFindings findings=loaded.compatibility_findings.clone()/ >
                        <RevisionDependencies impacts=loaded.dependency_impacts.clone()/ >
                        <section class="route-panel__section">
                            <h3>"Output Fields"</h3>
                            <DataTable>
                                <thead>
                                    <tr>
                                        <th>"Key"</th>
                                        <th>"Label"</th>
                                        <th>"Type"</th>
                                        <th>"Source"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {loaded.output_fields.into_iter().map(|field| view! {
                                        <tr>
                                            <td>{field.key}</td>
                                            <td>{field.label}</td>
                                            <td>{field.field_type}</td>
                                            <td>{field.source_alias}</td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </DataTable>
                        </section>
                        <section class="route-panel__section">
                            <h3>"Generated SQL"</h3>
                            <pre class="dataset-sql-panel">{loaded.generated_sql.unwrap_or_else(|| "SQL unavailable".into())}</pre>
                        </section>
                    }.into_any()
                } else {
                    view! { <EmptyState title="Revision unavailable" message="Dataset revision could not be loaded."/> }.into_any()
                }
            }}
        </section>
    }
}

#[component]
fn PublishRevisionMenu(
    dataset_id: String,
    revision_id: String,
    revision: RwSignal<Option<DatasetRevisionDetail>>,
    options_error: RwSignal<Option<String>>,
    publish_error: RwSignal<Option<String>>,
    publish_message: RwSignal<Option<String>>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let menu_class = move || {
        if is_open.get() {
            "dropdown-menu is-open"
        } else {
            "dropdown-menu"
        }
    };
    let revision_dataset_id = dataset_id.clone();
    let revision_revision_id = revision_id.clone();
    let major_dataset_id = dataset_id.clone();
    let major_revision_id = revision_id.clone();

    view! {
        <div class=menu_class>
            <button
                class="button"
                type="button"
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                "Publish"
            </button>
            <button
                class="dropdown-menu__scrim"
                type="button"
                aria-label="Close publish menu"
                on:click=move |_| is_open.set(false)
            ></button>
            <div class="dropdown-menu__content blurred-surface" role="menu" on:click=move |_| is_open.set(false)>
                <button class="dropdown-menu__item" type="button" role="menuitem" on:click=move |_| {
                    publish_dataset_revision(
                        revision_dataset_id.clone(),
                        revision_revision_id.clone(),
                        false,
                        revision,
                        options_error,
                        publish_error,
                        publish_message,
                    );
                }>
                    <span>"Revision"</span>
                </button>
                <button class="dropdown-menu__item" type="button" role="menuitem" on:click=move |_| {
                    publish_dataset_revision(
                        major_dataset_id.clone(),
                        major_revision_id.clone(),
                        true,
                        revision,
                        options_error,
                        publish_error,
                        publish_message,
                    );
                }>
                    <span>"New Major Version"</span>
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn DatasetEditorContent(dataset_id: Option<String>) -> impl IntoView {
    view! { <DatasetEditorSurface dataset_id revision_id=None/> }
}

#[component]
pub fn DatasetRevisionEditorContent(dataset_id: String, revision_id: String) -> impl IntoView {
    view! { <DatasetEditorSurface dataset_id=Some(dataset_id) revision_id=Some(revision_id)/> }
}

#[component]
pub fn DatasetPreviewContent(dataset_id: String) -> impl IntoView {
    let dataset = RwSignal::new(None::<DatasetDefinition>);
    let table = RwSignal::new(None::<DatasetTable>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let table_error = RwSignal::new(None::<String>);

    Effect::new({
        let dataset_id = dataset_id.clone();
        move |_| {
            load_dataset_detail(dataset_id.clone(), dataset, is_loading, load_error);
            load_dataset_table(dataset_id.clone(), table, table_error);
        }
    });

    view! {
        <main class="dataset-preview-page">
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading preview" message="Fetching dataset preview rows."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Preview unavailable" message=message/> }.into_any()
                } else if let Some(loaded) = dataset.get() {
                    view! {
                        <section class="dataset-preview-page__content">
                            <header class="dataset-preview-page__header">
                                <p>"Dataset Preview"</p>
                                <h1>{loaded.name.clone()}</h1>
                            </header>
                            <DatasetPreviewTable dataset=loaded table=table.get() error=table_error.get()/ >
                        </section>
                    }.into_any()
                } else {
                    view! { <EmptyState title="Preview unavailable" message="Dataset details could not be loaded."/> }.into_any()
                }
            }}
        </main>
    }
}

#[component]
fn RevisionFindings(findings: Vec<DatasetCompatibilityFinding>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h3>"Changelog"</h3>
            {if findings.is_empty() {
                view! { <EmptyState title="No changelog entries" message="No dataset changes were detected against the current published revision."/> }.into_any()
            } else {
                view! {
                    <DataTable>
                        <thead>
                            <tr>
                                <th>"Version Impact"</th>
                                <th>"Change"</th>
                                <th>"Field"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {findings.into_iter().map(|finding| view! {
                                <tr>
                                    <td>
                                        <span class=version_impact_class(&finding.version_impact)>
                                            {version_impact_label(&finding.version_impact)}
                                        </span>
                                    </td>
                                    <td>{finding.message}</td>
                                    <td>{finding.field_key.unwrap_or_else(|| "Dataset".into())}</td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </DataTable>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn RevisionDependencies(impacts: Vec<DatasetDependencyImpact>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h3>"Downstream Dependencies"</h3>
            {if impacts.is_empty() {
                view! { <EmptyState title="No downstream dependencies" message="No datasets, components, or dashboards are pinned to this revision."/> }.into_any()
            } else {
                view! {
                    <DataTable>
                        <thead>
                            <tr>
                                <th>"Kind"</th>
                                <th>"Name"</th>
                                <th>"Binding"</th>
                                <th>"Carry Forward"</th>
                                <th>"Pinned Target"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {impacts.into_iter().map(|impact| {
                                let kind = dependency_kind_label(&impact.kind);
                                let binding = dependency_binding_label(&impact);
                                let carry_forward = carry_forward_label(&impact.carry_forward_state);
                                let pinned_target = dependency_pinned_target(&impact);
                                view! {
                                    <tr>
                                        <td>{kind}</td>
                                        <td>{impact.name}</td>
                                        <td>{binding}</td>
                                        <td>{carry_forward}</td>
                                        <td>{pinned_target}</td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </DataTable>
                }.into_any()
            }}
        </section>
    }
}

fn revision_status_label(status: &DatasetRevisionStatus, is_current: bool) -> String {
    let label = match status {
        DatasetRevisionStatus::Draft => "Draft",
        DatasetRevisionStatus::Published => "Published",
        DatasetRevisionStatus::Superseded => "Superseded",
    };
    if is_current {
        format!("{label} current")
    } else {
        label.into()
    }
}

fn revision_label_text(version_number: i32, version_label: &str) -> Option<String> {
    let label = version_label.trim();
    (!label.is_empty() && label != version_number.to_string()).then(|| label.to_string())
}

fn semantic_version_label(
    major: Option<i32>,
    minor: Option<i32>,
    patch: Option<i32>,
) -> Option<String> {
    Some(format!("v{}.{}.{}", major?, minor?, patch?))
}

fn compatibility_label(summary: &DatasetCompatibilitySummary) -> String {
    let state = compatibility_state_label(summary);
    format!(
        "{state} · {} major · {} minor · {} patch",
        summary.major_count, summary.minor_count, summary.patch_count
    )
}

fn compatibility_state_label(summary: &DatasetCompatibilitySummary) -> &'static str {
    match summary.state {
        DatasetCompatibilityState::Compatible => "Compatible",
        DatasetCompatibilityState::Review => "Review",
        DatasetCompatibilityState::Breaking => "Breaking",
    }
}

fn dependency_label(summary: &DatasetDependencySummary) -> String {
    format!(
        "{} total · {} datasets · {} components · {} dashboards",
        summary.dependency_count,
        summary.dataset_count,
        summary.component_version_count,
        summary.dashboard_count
    )
}

fn version_impact_label(impact: &DatasetVersionImpact) -> &'static str {
    match impact {
        DatasetVersionImpact::Patch => "Patch",
        DatasetVersionImpact::Minor => "Minor",
        DatasetVersionImpact::Major => "Major",
    }
}

fn version_impact_class(impact: &DatasetVersionImpact) -> &'static str {
    match impact {
        DatasetVersionImpact::Patch => "status-badge status-badge--patch",
        DatasetVersionImpact::Minor => "status-badge status-badge--minor",
        DatasetVersionImpact::Major => "status-badge status-badge--major",
    }
}

fn dependency_kind_label(kind: &DatasetDependencyKind) -> &'static str {
    match kind {
        DatasetDependencyKind::Dataset => "Dataset",
        DatasetDependencyKind::ComponentVersion => "Component Version",
        DatasetDependencyKind::Dashboard => "Dashboard",
    }
}

fn dependency_binding_label(impact: &DatasetDependencyImpact) -> String {
    match impact.binding_mode {
        DatasetDependencyBindingMode::MajorLine => impact
            .pinned_version_major
            .map(|major| format!("Version {major}"))
            .unwrap_or_else(|| "Version".to_string()),
        DatasetDependencyBindingMode::ExactRevision => "Exact Revision".to_string(),
    }
}

fn dependency_pinned_target(impact: &DatasetDependencyImpact) -> String {
    impact
        .pinned_version_major
        .map(|major| format!("Version {major}"))
        .unwrap_or_else(|| impact.pinned_revision_id.clone())
}

fn carry_forward_label(state: &DatasetCarryForwardState) -> &'static str {
    match state {
        DatasetCarryForwardState::Safe => "Safe",
        DatasetCarryForwardState::ManualReview => "Manual Review",
        DatasetCarryForwardState::Blocked => "Blocked",
    }
}

#[cfg(feature = "hydrate")]
fn load_revision_history(
    dataset_id: String,
    revisions: RwSignal<Vec<DatasetRevisionSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        load_error.set(None);
        match api::fetch_dataset_revisions(&dataset_id).await {
            Ok(Some(response)) => revisions.set(response),
            Ok(None) => revisions.set(Vec::new()),
            Err(message) => load_error.set(Some(message)),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_revision_history(
    _: String,
    _: RwSignal<Vec<DatasetRevisionSummary>>,
    is_loading: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
    is_loading.set(false);
}

#[cfg(feature = "hydrate")]
fn load_revision_detail(
    dataset_id: String,
    revision_id: String,
    revision: RwSignal<Option<DatasetRevisionDetail>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        load_error.set(None);
        match api::fetch_dataset_revision(&dataset_id, &revision_id).await {
            Ok(Some(response)) => revision.set(Some(response)),
            Ok(None) => revision.set(None),
            Err(message) => load_error.set(Some(message)),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_revision_detail(
    _: String,
    _: String,
    _: RwSignal<Option<DatasetRevisionDetail>>,
    is_loading: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
    is_loading.set(false);
}

#[cfg(feature = "hydrate")]
fn publish_dataset_revision(
    dataset_id: String,
    revision_id: String,
    force_new_major_version: bool,
    revision: RwSignal<Option<DatasetRevisionDetail>>,
    options_error: RwSignal<Option<String>>,
    publish_error: RwSignal<Option<String>>,
    publish_message: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        let confirmation = if force_new_major_version {
            "Publish this draft as a new major version?"
        } else {
            "Publish this draft revision?"
        };
        let confirmed = web_sys::window()
            .and_then(|window| window.confirm_with_message(confirmation).ok())
            .unwrap_or(false);
        if !confirmed {
            return;
        }

        options_error.set(None);
        publish_error.set(None);
        publish_message.set(None);
        match api::update_dataset_revision_options(
            &dataset_id,
            &revision_id,
            force_new_major_version,
        )
        .await
        {
            Ok(updated) => revision.set(Some(updated)),
            Err(message) => {
                options_error.set(Some(message));
                return;
            }
        }
        match api::publish_dataset_revision(&dataset_id, &revision_id).await {
            Ok(response) => {
                publish_message.set(Some("Revision published.".into()));
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href(&format!(
                        "/datasets/{}/revisions/{}",
                        response.dataset_id, response.revision_id
                    ));
                }
            }
            Err(message) => publish_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn publish_dataset_revision(
    _: String,
    _: String,
    _: bool,
    _: RwSignal<Option<DatasetRevisionDetail>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
fn delete_dataset_revision(
    dataset_id: String,
    revision_id: String,
    delete_error: RwSignal<Option<String>>,
    delete_message: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        let confirmed = web_sys::window()
            .and_then(|window| {
                window
                    .confirm_with_message("Are you sure you want to delete this draft revision? This cannot be undone.")
                    .ok()
            })
            .unwrap_or(false);
        if !confirmed {
            return;
        }

        delete_error.set(None);
        delete_message.set(None);
        match api::delete_dataset_revision(&dataset_id, &revision_id).await {
            Ok(_) => {
                delete_message.set(Some("Draft revision deleted.".into()));
                if let Some(window) = web_sys::window() {
                    let _ = window
                        .location()
                        .set_href(&format!("/datasets/{dataset_id}/revisions"));
                }
            }
            Err(message) => delete_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn delete_dataset_revision(
    _: String,
    _: String,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
fn update_revision_label(
    dataset_id: String,
    revision_id: String,
    version_label: String,
    revision_notes: String,
    revision: RwSignal<Option<DatasetRevisionDetail>>,
    label_error: RwSignal<Option<String>>,
    label_message: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        label_error.set(None);
        label_message.set(None);
        match api::update_dataset_revision_label(
            &dataset_id,
            &revision_id,
            version_label,
            revision_notes,
        )
        .await
        {
            Ok(response) => {
                let saved_label = response.version_label;
                let saved_notes = response.revision_notes;
                revision.update(|revision| {
                    if let Some(revision) = revision.as_mut()
                        && revision.id == response.revision_id
                    {
                        revision.version_label = saved_label.clone();
                        revision.revision_notes = saved_notes.clone();
                    }
                });
                label_message.set(Some("Revision details saved.".into()));
            }
            Err(message) => label_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn update_revision_label(
    _: String,
    _: String,
    _: String,
    _: String,
    _: RwSignal<Option<DatasetRevisionDetail>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}
