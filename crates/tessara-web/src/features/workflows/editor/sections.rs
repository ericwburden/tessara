//! Workflow editor form sections.

use crate::features::forms::FormSummary;
use crate::features::organization::{NodeTypeCatalogEntry, OrganizationNode};
use crate::features::shared::status_badge_class;
use crate::features::workflows::types::WorkflowStepDraft;
use crate::features::workflows::workflow_form_version_options;
use leptos::prelude::*;
use std::collections::HashSet;

use super::{WorkflowAvailableNodesPicker, WorkflowStepList};

#[component]
/// Renders the workflow identity editor fields.
pub(in crate::features::workflows) fn WorkflowIdentityFields(
    name: RwSignal<String>,
    description: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="form-grid">
            <label class="form-field">
                <span>"Workflow Name"</span>
                <input
                    type="text"
                    value=move || name.get()
                    on:input=move |event| {
                        name.set(event_target_value(&event));
                    }
                />
            </label>
            <label class="form-field">
                <span>"Description"</span>
                <textarea
                    prop:value=move || description.get()
                    on:input=move |event| {
                        description.set(event_target_value(&event));
                    }
                ></textarea>
            </label>
        </div>
    }
}

#[component]
/// Renders the workflow availability editor section.
pub(in crate::features::workflows) fn WorkflowAvailabilitySection(
    organization_nodes: RwSignal<Vec<OrganizationNode>>,
    available_node_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    view! {
        <section class="form-section">
            <h3>"Available At"</h3>
            <WorkflowAvailableNodesPicker
                nodes=organization_nodes.get()
                selected_node_ids=available_node_ids
            />
        </section>
    }
}

#[component]
/// Renders the create workflow steps editor section.
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
/// Renders the edit workflow active revision section.
pub(in crate::features::workflows) fn WorkflowActiveRevisionSection(
    edit_version_label: RwSignal<String>,
    edit_version_status: RwSignal<String>,
) -> impl IntoView {
    view! {
        <section class="form-section">
            <h3>"Active Revision"</h3>
            <table class="info-list-table">
                <tbody>
                    <tr>
                        <th scope="row">"Revision"</th>
                        <td>{move || edit_version_label.get()}</td>
                    </tr>
                    <tr>
                        <th scope="row">"Status"</th>
                        <td>{move || {
                            let status = edit_version_status.get();
                            let key = status.to_lowercase().replace(' ', "-");
                            view! { <span class=status_badge_class(&key)>{status}</span> }
                        }}</td>
                    </tr>
                </tbody>
            </table>
        </section>
    }
}

#[component]
/// Renders the edit workflow steps editor section.
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
