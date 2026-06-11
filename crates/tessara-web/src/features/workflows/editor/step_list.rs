//! Workflow step list and step card UI for workflow editing.

use super::options::{workflow_form_version_options, workflow_step_form_label};
use crate::features::forms::FormSummary;
use crate::features::organization::{
    NodeTypeCatalogEntry, workflow_step_form_version_id_by_id, workflow_step_title_by_id,
};
use crate::features::workflows::types::WorkflowStepDraft;
use icons::{ArrowDown, ArrowUp, Trash2};
use leptos::prelude::*;

#[component]
/// Renders the editable workflow step list.
pub(in crate::features::workflows) fn WorkflowStepList(
    forms: RwSignal<Vec<FormSummary>>,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
) -> impl IntoView {
    view! {
        <div class="workflow-step-list">
            <For
                each=move || {
                    steps.get().into_iter().enumerate().collect::<Vec<_>>()
                }
                key=|(_, step)| step.id
                children=move |(index, step)| {
                    let step_id = step.id;
                    let step_position = move || {
                        steps
                            .get()
                            .iter()
                            .position(|step| step.id == step_id)
                            .map(|index| index + 1)
                            .unwrap_or(index + 1)
                    };
                    view! {
                        <article class="workflow-step-card">
                            <header class="workflow-step-card__header">
                                <span class="workflow-step-card__position">{move || format!("Step {}", step_position())}</span>
                                <div class="workflow-step-card__actions">
                                    <button
                                        class="icon-button icon-button--control"
                                        type="button"
                                        title="Move step up"
                                        disabled=move || step_position() <= 1
                                        on:click=move |_| {
                                            steps.update(|steps| {
                                                if let Some(index) = steps.iter().position(|step| step.id == step_id)
                                                    && index > 0 {
                                                        steps.swap(index, index - 1);
                                                    }
                                            });
                                        }
                                    >
                                        <ArrowUp/>
                                    </button>
                                    <button
                                        class="icon-button icon-button--control"
                                        type="button"
                                        title="Move step down"
                                        disabled=move || {
                                            let step_count = steps.get().len();
                                            step_position() >= step_count
                                        }
                                        on:click=move |_| {
                                            steps.update(|steps| {
                                                if let Some(index) = steps.iter().position(|step| step.id == step_id)
                                                    && index + 1 < steps.len() {
                                                        steps.swap(index, index + 1);
                                                    }
                                            });
                                        }
                                    >
                                        <ArrowDown/>
                                    </button>
                                    <button
                                        class="icon-button icon-button--danger"
                                        type="button"
                                        title="Remove step"
                                        on:click=move |_| {
                                            steps.update(|steps| {
                                                steps.retain(|step| step.id != step_id);
                                            });
                                        }
                                    >
                                        <Trash2/>
                                    </button>
                                </div>
                            </header>
                            <div class="form-grid">
                                <label class="form-field">
                                    <span>"Step Title"</span>
                                    <input
                                        type="text"
                                        prop:value=move || {
                                            workflow_step_title_by_id(&steps.get(), step_id)
                                        }
                                        on:input=move |event| {
                                            let value = event_target_value(&event);
                                            steps.update(|steps| {
                                                if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                    step.title = value;
                                                }
                                            });
                                        }
                                    />
                                </label>
                                <label class="form-field">
                                    <span>"Form Version"</span>
                                    <select
                                        prop:value=move || {
                                            workflow_step_form_version_id_by_id(&steps.get(), step_id)
                                        }
                                        on:change=move |event| {
                                            let value = event_target_value(&event);
                                            steps.update(|steps| {
                                                if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                    step.form_version_id = value;
                                                }
                                            });
                                        }
                                    >
                                        <option value="">"Select form version"</option>
                                        {workflow_form_version_options(
                                            &forms.get(),
                                            &node_types.get(),
                                            "",
                                        )
                                            .into_iter()
                                            .map(|(id, label, _)| {
                                                view! {
                                                    <option value=id>{label}</option>
                                                }
                                            })
                                            .collect_view()}
                                    </select>
                                </label>
                            </div>
                            <div class="workflow-step-card__footer">
                                <span>{move || {
                                    let selected_form_version_id = steps
                                        .get()
                                        .into_iter()
                                        .find(|step| step.id == step_id)
                                        .map(|step| step.form_version_id)
                                        .unwrap_or_default();
                                    workflow_step_form_label(&forms.get(), &selected_form_version_id)
                                }}</span>
                            </div>
                        </article>
                    }
                }
            />
        </div>
    }
}
