//! Workflow creation page implementation.

use crate::features::forms::FormSummary;
use crate::features::organization::{NodeTypeCatalogEntry, OrganizationNode};
use crate::features::workflows::loaders::load_workflow_create_options;
use crate::features::workflows::types::{WorkflowStepDraft, WorkflowSummary};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;
use leptos::prelude::*;
use std::collections::HashSet;

use super::{
    WorkflowAvailabilitySection, WorkflowCreateStepsSection, WorkflowIdentityFields,
    add_workflow_step, can_submit_workflow_editor, prune_unavailable_workflow_steps,
    submit_create_workflow,
};

#[component]
pub(crate) fn WorkflowsNewPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let forms = RwSignal::new(Vec::<FormSummary>::new());
    let existing_workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let seeded_from_form = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let available_node_ids = RwSignal::new(HashSet::<String>::new());
    let steps = RwSignal::new(Vec::<WorkflowStepDraft>::new());
    let next_step_id = RwSignal::new(1_usize);
    let description = RwSignal::new(String::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflow_create_options(
            node_types,
            organization_nodes,
            forms,
            existing_workflows,
            is_loading,
            message,
        );
    });

    Effect::new(move |_| {
        if is_loading.get() || seeded_from_form.get_untracked() {
            return;
        }

        let form_id: Option<String> = {
            #[cfg(feature = "hydrate")]
            {
                current_search_param("form_id")
            }
            #[cfg(not(feature = "hydrate"))]
            {
                None
            }
        };
        let Some(form_id) = form_id else {
            seeded_from_form.set(true);
            return;
        };

        let available_forms = forms.get();
        let Some(form) = available_forms.iter().find(|form| form.id == form_id) else {
            seeded_from_form.set(true);
            return;
        };
        let Some(version) = form
            .versions
            .iter()
            .find(|version| version.status == "published")
        else {
            seeded_from_form.set(true);
            return;
        };

        name.set(format!("{} Workflow", form.name));
        description.set(format!("Workflow for {}.", form.name));
        steps.set(vec![WorkflowStepDraft {
            id: 1,
            title: format!("{} Response", form.name),
            form_version_id: version.id.clone(),
        }]);
        next_step_id.set(2);
        seeded_from_form.set(true);
    });

    Effect::new(move |_| {
        if is_loading.get() {
            return;
        }
        prune_unavailable_workflow_steps(&forms.get(), &node_types.get(), steps);
    });

    let add_step = move |_| add_workflow_step(next_step_id, steps);

    let can_submit = move || can_submit_workflow_editor(is_saving, name, available_node_ids, steps);

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Create Workflow"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <section class="route-panel workflows-page">
                    <PageHeader title="Create Workflow"/>

                    {move || {
                        if is_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading workflow options"</h3>
                                    <p>"Fetching forms and workflow names."</p>
                                </section>
                            }
                            .into_any()
                        } else {
                            view! {
                                <form
                                    class="native-form workflow-create-form"
                                    on:submit=move |event| {
                                        event.prevent_default();
                                        submit_create_workflow(
                                            name,
                                            available_node_ids,
                                            steps,
                                            description,
                                            existing_workflows,
                                            is_saving,
                                            message,
                                        );
                                    }
                                >
                                    <WorkflowIdentityFields name=name description=description/>

                                    <WorkflowAvailabilitySection
                                        organization_nodes=organization_nodes
                                        available_node_ids=available_node_ids
                                    />

                                    <WorkflowCreateStepsSection
                                        forms=forms
                                        node_types=node_types
                                        steps=steps
                                        on_add_step=add_step
                                    />

                                    {move || message.get().map(|message| view! {
                                        <p class="form-message" role="status">{message}</p>
                                    })}

                                    <div class="form-actions">
                                        <a class="button" href="/workflows">"Cancel"</a>
                                        <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                            {move || if is_saving.get() { "Creating..." } else { "Create Workflow" }}
                                        </button>
                                    </div>
                                </form>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}
