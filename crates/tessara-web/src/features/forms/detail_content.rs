//! Form detail presentation components.

use crate::features::forms::{
    FormDatasetSourceLink, FormDefinition, FormVersionsTable, FormWorkflowLink, RenderedForm,
    active_form_definition_version, form_attached_nodes, form_definition_scope_label,
    form_field_count_label, form_status_label, form_version_label,
};
use crate::features::shared::{FormAttachmentLink, status_badge_class};
use crate::ui::{InfoListTable, Tabs, TabsContent, TabsList, TabsTrigger, Timestamp, empty_view};
use leptos::prelude::*;

use super::attached_nodes::FormAttachedNodesRelatedTable;
use super::components::RenderedFormSections;
use super::tables::{FormRelatedDatasetSourcesTable, FormRelatedWorkflowsTable};

#[component]
pub(in crate::features::forms) fn FormDetailContent(
    form: FormDefinition,
    rendered_form: Option<RenderedForm>,
) -> impl IntoView {
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
