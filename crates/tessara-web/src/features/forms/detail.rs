//! Detail view components for the Forms feature.
//!
//! Keep read-focused panels and detail-page presentation here; mutation workflows should live in editor or API modules.

use crate::features::forms::api::load_form_detail;
use crate::features::forms::{
    FormDatasetSourceLink, FormDefinition, FormVersionsTable, FormWorkflowLink, RenderedForm,
};
use crate::features::forms::{
    form_attached_nodes, form_definition_scope_label, form_field_count_label, form_status_label,
    rendered_field_layout_label, rendered_field_type_label,
};
use crate::features::organization::{
    RelatedWorkPaginationFooter, active_form_definition_version, form_version_label,
};
use crate::features::shared::{FormAttachmentLink, status_badge_class};
use crate::features::workflows::{WorkflowSourceMarker, workflow_revision_label_from_option};
use crate::types::route_params::{FormRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    DataTable, EmptyState, InfoListTable, PageHeader, SearchableDataTable, Tabs, TabsContent,
    TabsList, TabsTrigger, Timestamp, empty_view,
};
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::{sentence_label, text_matches};
use leptos::prelude::*;

#[component]
/// Renders the forms detail page view.
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
/// Renders the form detail content view.
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
/// Renders the form attached nodes related table view.
pub(crate) fn FormAttachedNodesRelatedTable(nodes: Vec<FormAttachmentLink>) -> impl IntoView {
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
/// Renders the rendered form sections view.
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
/// Renders the form related links view.
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
/// Renders the form related workflows table view.
pub(crate) fn FormRelatedWorkflowsTable(workflows: Vec<FormWorkflowLink>) -> impl IntoView {
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
/// Renders the form related dataset sources table view.
pub(crate) fn FormRelatedDatasetSourcesTable(
    dataset_sources: Vec<FormDatasetSourceLink>,
) -> impl IntoView {
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
