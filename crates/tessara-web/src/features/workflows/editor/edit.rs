//! Workflow edit page implementation.

use crate::features::forms::FormSummary;
use crate::features::organization::{NodeTypeCatalogEntry, OrganizationNode};
use crate::features::workflows::loaders::{load_workflow_create_options, load_workflow_detail};
use crate::features::workflows::types::{
    WorkflowDefinition, WorkflowSaveIntent, WorkflowStepDraft, WorkflowSummary,
};
use crate::types::route_params::{WorkflowRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;
use leptos::prelude::*;
use std::collections::HashSet;

use super::{WorkflowEditForm, prune_unavailable_workflow_steps, workflow_edit_initial_state};

#[component]
pub(crate) fn WorkflowsEditPage() -> impl IntoView {
    let params = require_route_params::<WorkflowRouteParams>();
    let workflow_id = params.workflow_id;
    let detail = RwSignal::new(None::<WorkflowDefinition>);
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let forms = RwSignal::new(Vec::<FormSummary>::new());
    let existing_workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let available_node_ids = RwSignal::new(HashSet::<String>::new());
    let description = RwSignal::new(String::new());
    let steps = RwSignal::new(Vec::<WorkflowStepDraft>::new());
    let original_steps = RwSignal::new(Vec::<WorkflowStepDraft>::new());
    let next_step_id = RwSignal::new(1_usize);
    let edit_version_id = RwSignal::new(None::<String>);
    let edit_version_label = RwSignal::new(String::new());
    let edit_version_status = RwSignal::new(String::new());
    let version_is_draft = RwSignal::new(false);
    let initialized = RwSignal::new(false);
    let detail_loading = RwSignal::new(true);
    let options_loading = RwSignal::new(true);
    let detail_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);
    let is_saving = RwSignal::new(false);
    let save_intent = RwSignal::new(None::<WorkflowSaveIntent>);

    {
        let workflow_id = workflow_id.clone();
        Effect::new(move |_| {
            load_workflow_detail(workflow_id.clone(), detail, detail_loading, detail_error);
        });
    }

    Effect::new(move |_| {
        load_workflow_create_options(
            node_types,
            organization_nodes,
            forms,
            existing_workflows,
            options_loading,
            message,
        );
    });

    Effect::new(move |_| {
        if initialized.get_untracked() {
            return;
        }
        let Some(workflow) = detail.get() else {
            return;
        };

        let requested_version_id = {
            #[cfg(feature = "hydrate")]
            {
                current_search_param("version_id")
            }
            #[cfg(not(feature = "hydrate"))]
            {
                None::<String>
            }
        };
        let initial_state = workflow_edit_initial_state(&workflow, requested_version_id);

        name.set(initial_state.name);
        slug.set(initial_state.slug);
        available_node_ids.set(initial_state.available_node_ids);
        description.set(initial_state.description);
        edit_version_id.set(initial_state.edit_version_id);
        edit_version_label.set(initial_state.edit_version_label);
        edit_version_status.set(initial_state.edit_version_status);
        version_is_draft.set(initial_state.version_is_draft);
        original_steps.set(initial_state.steps.clone());
        steps.set(initial_state.steps);
        next_step_id.set(initial_state.next_step_id);
        initialized.set(true);
    });

    Effect::new(move |_| {
        if !initialized.get() {
            return;
        }
        if options_loading.get() {
            return;
        }
        prune_unavailable_workflow_steps(&forms.get(), &node_types.get(), steps);
    });

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                    </BreadcrumbItem>
                    {move || detail.get().map(|workflow| view! {
                        <>
                            <BreadcrumbSeparator/>
                            <BreadcrumbItem>
                                <BreadcrumbLink href=format!("/workflows/{}", workflow.id)>{workflow.name}</BreadcrumbLink>
                            </BreadcrumbItem>
                        </>
                    })}
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Edit Workflow"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <section class="route-panel workflows-page workflow-edit-page">
                    <PageHeader title="Edit Workflow"/>

                    {move || {
                        if detail_loading.get() || options_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading workflow"</h3>
                                    <p>"Fetching workflow details and form versions."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(error) = detail_error.get() {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Workflow unavailable"</h3>
                                    <p>{error}</p>
                                </section>
                            }
                            .into_any()
                        } else {
                            view! {
                                <WorkflowEditForm
                                    workflow_id=workflow_id.clone()
                                    name
                                    slug
                                    available_node_ids
                                    description
                                    steps
                                    original_steps
                                    next_step_id
                                    edit_version_id
                                    edit_version_label
                                    edit_version_status
                                    version_is_draft
                                    node_types
                                    organization_nodes
                                    forms
                                    is_saving
                                    save_intent
                                    message
                                />
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}
