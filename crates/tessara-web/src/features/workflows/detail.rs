//! Detail view components for the Workflows feature.
//!
//! Keep read-focused panels and detail-page presentation here; route loading belongs in `pages`.

mod cards;

use crate::features::workflows::types::WorkflowDefinition;
use crate::features::workflows::{
    active_workflow_definition_version, workflow_available_nodes_label,
    workflow_definition_status_label, workflow_definition_version_label, workflow_source_label,
};
use crate::utils::text::nonempty_text;
use cards::{
    WorkflowActiveRevisionCard, WorkflowAssignmentsSection, WorkflowDetailsCard,
    WorkflowRevisionsSection, WorkflowStepsSection,
};
use leptos::prelude::*;

#[component]
pub(in crate::features::workflows) fn WorkflowDetailContent(
    workflow: WorkflowDefinition,
) -> impl IntoView {
    let active_version = active_workflow_definition_version(&workflow).cloned();
    let active_status = active_version
        .as_ref()
        .map(|version| version.status.clone())
        .unwrap_or_else(|| "none".to_string());
    let active_version_label = workflow_definition_version_label(active_version.as_ref());
    let active_status_label = workflow_definition_status_label(active_version.as_ref());
    let active_step_count = active_version
        .as_ref()
        .map(|version| version.step_count.to_string())
        .unwrap_or_else(|| "-".to_string());
    let published_at = active_version
        .as_ref()
        .and_then(|version| version.published_at.clone());
    let workflow_id = workflow.id.clone();
    let workflow_name = workflow.name.clone();
    let workflow_slug = workflow.slug.clone();
    let workflow_description = nonempty_text(Some(workflow.description.as_str()), "No description");
    let workflow_available_at = workflow_available_nodes_label(&workflow.available_nodes);
    let workflow_source = workflow_source_label(&workflow.source)
        .unwrap_or("Authored")
        .to_string();
    let revision_count = workflow.versions.len().to_string();
    let assignment_count = workflow.assignments.len().to_string();
    let steps = active_version
        .as_ref()
        .map(|version| version.steps.clone())
        .unwrap_or_default();
    let versions = workflow.versions.clone();
    let assignments = workflow.assignments.clone();

    view! {
        <div class="organization-detail-content workflow-detail-content">
            <header class="organization-detail-content__header">
                <p>"Workflow Detail"</p>
                <h2>{workflow_name}</h2>
            </header>

            <div class="organization-detail-content__grid">
                <WorkflowDetailsCard
                    slug=workflow_slug
                    description=workflow_description
                    available_at=workflow_available_at
                    source=workflow_source
                    revision_count=revision_count.clone()
                    assignment_count=assignment_count.clone()
                />

                <WorkflowActiveRevisionCard
                    active_status
                    active_status_label
                    active_step_count=active_step_count.clone()
                    active_version_label
                    published_at
                />

                <WorkflowStepsSection steps count=active_step_count/>
                <WorkflowRevisionsSection workflow_id versions count=revision_count/>
                <WorkflowAssignmentsSection assignments count=assignment_count/>
            </div>
        </div>
    }
}

pub(crate) use crate::features::workflows::pages::detail::WorkflowsDetailPage;
