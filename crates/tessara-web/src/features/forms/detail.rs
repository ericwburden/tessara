//! Detail view components for the Forms feature.
//!
//! Keep read-focused panels and detail-page presentation here; mutation workflows should live in editor or API modules.

use crate::features::forms::loaders::load_form_detail;
use crate::features::forms::{
    FormDatasetSourceLink, FormDefinition, FormVersionsTable, FormWorkflowLink, RenderedForm,
};
use crate::features::forms::{
    active_form_definition_version, form_attached_nodes, form_definition_scope_label,
    form_field_count_label, form_status_label, form_version_label, rendered_field_layout_label,
    rendered_field_type_label,
};
use crate::features::shared::{FormAttachmentLink, status_badge_class};
use crate::types::route_params::{FormRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    DataTable, EmptyState, InfoListTable, PageHeader, Tabs, TabsContent, TabsList, TabsTrigger,
    Timestamp, empty_view,
};
use leptos::prelude::*;

use super::attached_nodes::FormAttachedNodesRelatedTable;
use super::tables::{FormRelatedDatasetSourcesTable, FormRelatedWorkflowsTable};

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
