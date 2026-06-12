//! Workflow edit form composition.

use crate::features::forms::FormSummary;
use crate::features::organization::{NodeTypeCatalogEntry, OrganizationNode};
use crate::features::workflows::types::{WorkflowSaveIntent, WorkflowStepDraft};
use leptos::prelude::*;
use std::collections::HashSet;

use super::{
    WorkflowActiveRevisionSection, WorkflowAvailabilitySection, WorkflowEditStepsSection,
    WorkflowIdentityFields, add_workflow_step, can_submit_workflow_editor, submit_update_workflow,
    workflow_step_signature,
};

#[allow(clippy::too_many_arguments)]
#[component]
pub(in crate::features::workflows) fn WorkflowEditForm(
    workflow_id: String,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    available_node_ids: RwSignal<HashSet<String>>,
    description: RwSignal<String>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
    original_steps: RwSignal<Vec<WorkflowStepDraft>>,
    next_step_id: RwSignal<usize>,
    edit_version_id: RwSignal<Option<String>>,
    edit_version_label: RwSignal<String>,
    edit_version_status: RwSignal<String>,
    version_is_draft: RwSignal<bool>,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    organization_nodes: RwSignal<Vec<OrganizationNode>>,
    forms: RwSignal<Vec<FormSummary>>,
    is_saving: RwSignal<bool>,
    save_intent: RwSignal<Option<WorkflowSaveIntent>>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let workflow_id_for_href = workflow_id.clone();
    let workflow_id_for_submit = workflow_id.clone();
    let workflow_id_for_publish = workflow_id.clone();
    let workflow_href = format!("/workflows/{workflow_id_for_href}");
    let add_step = move |_| add_workflow_step(next_step_id, steps);
    let can_submit = move || can_submit_workflow_editor(is_saving, name, available_node_ids, steps);
    let has_step_changes = move || {
        workflow_step_signature(&steps.get()) != workflow_step_signature(&original_steps.get())
    };

    view! {
        <form
            class="native-form workflow-create-form"
            on:submit=move |event| {
                event.prevent_default();
                submit_update_workflow(
                    workflow_id_for_submit.clone(),
                    edit_version_id.get_untracked(),
                    version_is_draft.get_untracked(),
                    name,
                    slug,
                    available_node_ids,
                    steps,
                    original_steps,
                    description,
                    is_saving,
                    save_intent,
                    message,
                    WorkflowSaveIntent::Draft,
                );
            }
        >
            <WorkflowIdentityFields name=name description=description/>

            <WorkflowAvailabilitySection
                organization_nodes=organization_nodes
                available_node_ids=available_node_ids
            />

            <WorkflowActiveRevisionSection
                edit_version_label=edit_version_label
                edit_version_status=edit_version_status
            />

            <WorkflowEditStepsSection
                forms=forms
                node_types=node_types
                steps=steps
                version_is_draft=version_is_draft
                on_add_step=add_step
            />

            {move || message.get().map(|message| view! {
                <p class="form-message" role="status">{message}</p>
            })}

            <div class="form-actions">
                <a class="button" href=workflow_href>"Cancel"</a>
                <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                    {move || {
                        if save_intent.get() == Some(WorkflowSaveIntent::Draft) {
                            "Saving..."
                        } else if has_step_changes() {
                            "Save as Draft"
                        } else {
                            "Save Changes"
                        }
                    }}
                </button>
                <button
                    class="button button--secondary"
                    type="button"
                    disabled=move || {
                        !can_submit()
                            || (!version_is_draft.get() && !has_step_changes())
                    }
                    on:click=move |_| {
                        submit_update_workflow(
                            workflow_id_for_publish.clone(),
                            edit_version_id.get_untracked(),
                            version_is_draft.get_untracked(),
                            name,
                            slug,
                            available_node_ids,
                            steps,
                            original_steps,
                            description,
                            is_saving,
                            save_intent,
                            message,
                            WorkflowSaveIntent::Publish,
                        );
                    }
                >
                    {move || {
                        if save_intent.get() == Some(WorkflowSaveIntent::Publish) {
                            "Publishing..."
                        } else {
                            "Save and Publish"
                        }
                    }}
                </button>
            </div>
        </form>
    }
}
