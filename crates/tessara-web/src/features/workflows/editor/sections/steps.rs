//! Workflow editor step sections.

use crate::features::forms::FormSummary;
use crate::features::organization::NodeTypeCatalogEntry;
use crate::features::workflows::types::WorkflowStepDraft;
use crate::features::workflows::workflow_form_version_options;
use leptos::prelude::*;

use super::super::WorkflowStepList;

#[component]
pub(in crate::features::workflows) fn WorkflowCreateStepsSection(
    forms: RwSignal<Vec<FormSummary>>,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
    on_add_step: impl Fn(leptos::ev::MouseEvent) + 'static + Copy,
) -> impl IntoView {
    view! {
        <section class="form-section">
            <div class="form-builder-section-card__header">
                <h3>"Workflow Steps"</h3>
                <button
                    class="button button--secondary"
                    type="button"
                    disabled=move || {
                        workflow_form_version_options(
                            &forms.get(),
                            &node_types.get(),
                            "",
                        ).is_empty()
                    }
                    on:click=on_add_step
                >
                    "+ Add Step"
                </button>
            </div>
            {move || {
                let options = workflow_form_version_options(
                    &forms.get(),
                    &node_types.get(),
                    "",
                );
                if options.is_empty() {
                    return view! {
                        <section class="organization-state">
                            <h3>"No published forms available"</h3>
                            <p>"Publish at least one form version before creating a workflow."</p>
                        </section>
                    }
                    .into_any();
                }

                if steps.get().is_empty() {
                    return view! {
                        <section class="organization-state">
                            <h3>"No workflow steps yet"</h3>
                            <p>"Add one or more form steps to define the workflow."</p>
                        </section>
                    }
                    .into_any();
                }

                view! {
                    <WorkflowStepList forms=forms node_types=node_types steps=steps/>
                }
                .into_any()
            }}
        </section>
    }
}

#[component]
pub(in crate::features::workflows) fn WorkflowEditStepsSection(
    forms: RwSignal<Vec<FormSummary>>,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
    version_is_draft: RwSignal<bool>,
    on_add_step: impl Fn(leptos::ev::MouseEvent) + 'static + Copy,
) -> impl IntoView {
    view! {
        <section class="form-section">
            <div class="form-builder-section-card__header">
                <h3>"Workflow Steps"</h3>
                <button
                    class="button button--secondary"
                    type="button"
                    disabled=move || {
                        workflow_form_version_options(
                            &forms.get(),
                            &node_types.get(),
                            "",
                        )
                        .is_empty()
                    }
                    on:click=on_add_step
                >
                    "+ Add Step"
                </button>
            </div>

            {move || {
                if workflow_form_version_options(
                    &forms.get(),
                    &node_types.get(),
                    "",
                ).is_empty() {
                    return view! {
                        <section class="organization-state">
                            <h3>"No published forms available"</h3>
                            <p>"Publish at least one form version before editing workflow steps."</p>
                        </section>
                    }
                    .into_any();
                }

                if !version_is_draft.get() {
                    view! {
                        <p class="form-message" role="status">
                            "Step changes will create a new draft workflow revision."
                        </p>
                    }
                    .into_any()
                } else {
                    let _: () = view! { <></> };
                    ().into_any()
                }
            }}

            {move || {
                if steps.get().is_empty() {
                    return view! {
                        <section class="organization-state">
                            <h3>"No workflow steps"</h3>
                            <p>"This workflow revision does not have steps yet."</p>
                        </section>
                    }
                    .into_any();
                }

                view! {
                    <WorkflowStepList forms=forms node_types=node_types steps=steps/>
                }
                .into_any()
            }}
        </section>
    }
}
