//! Expandable workflow detail table sections.

use crate::features::workflows::assignments::WorkflowAssignmentSummary;
use crate::features::workflows::types::{WorkflowStepSummary, WorkflowVersionSummary};
use crate::features::workflows::{
    WorkflowDetailAssignmentsTable, WorkflowStepsTable, WorkflowVersionsTable,
};
use crate::ui::empty_view;
use leptos::prelude::*;

#[component]
pub(super) fn WorkflowStepsSection(
    steps: Vec<WorkflowStepSummary>,
    count: String,
) -> impl IntoView {
    let expanded = RwSignal::new(false);

    view! {
        <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
            <header class="form-detail-disclosure-header">
                <h3>"Steps"</h3>
                <button
                    class="link-button form-detail-disclosure-toggle"
                    type="button"
                    aria-expanded=move || expanded.get().to_string()
                    on:click=move |_| expanded.update(|expanded| *expanded = !*expanded)
                >
                    {move || {
                        if expanded.get() {
                            "Hide Steps".to_string()
                        } else {
                            format!("Show {count} Steps")
                        }
                    }}
                </button>
            </header>
            {move || {
                if expanded.get() {
                    view! { <WorkflowStepsTable steps=steps.clone()/> }.into_any()
                } else {
                    empty_view()
                }
            }}
        </section>
    }
}

#[component]
pub(super) fn WorkflowRevisionsSection(
    workflow_id: String,
    versions: Vec<WorkflowVersionSummary>,
    count: String,
) -> impl IntoView {
    let expanded = RwSignal::new(false);

    view! {
        <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
            <header class="form-detail-disclosure-header">
                <h3>"Revisions"</h3>
                <button
                    class="link-button form-detail-disclosure-toggle"
                    type="button"
                    aria-expanded=move || expanded.get().to_string()
                    on:click=move |_| expanded.update(|expanded| *expanded = !*expanded)
                >
                    {move || {
                        if expanded.get() {
                            "Hide Revisions".to_string()
                        } else {
                            format!("Show {count} Revisions")
                        }
                    }}
                </button>
            </header>
            {move || {
                if expanded.get() {
                    view! { <WorkflowVersionsTable workflow_id=workflow_id.clone() versions=versions.clone()/> }.into_any()
                } else {
                    empty_view()
                }
            }}
        </section>
    }
}

#[component]
pub(super) fn WorkflowAssignmentsSection(
    assignments: Vec<WorkflowAssignmentSummary>,
    count: String,
) -> impl IntoView {
    let expanded = RwSignal::new(false);

    view! {
        <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card workflow-detail-assignments-card">
            <header class="form-detail-disclosure-header">
                <h3>"Assignments"</h3>
                <button
                    class="link-button form-detail-disclosure-toggle"
                    type="button"
                    aria-expanded=move || expanded.get().to_string()
                    on:click=move |_| expanded.update(|expanded| *expanded = !*expanded)
                >
                    {move || {
                        if expanded.get() {
                            "Hide Assignments".to_string()
                        } else {
                            format!("Show {count} Assignments")
                        }
                    }}
                </button>
            </header>
            {move || {
                if expanded.get() {
                    view! { <WorkflowDetailAssignmentsTable assignments=assignments.clone()/> }.into_any()
                } else {
                    empty_view()
                }
            }}
        </section>
    }
}
