//! Assignments support for the Workflows feature.
//!
//! Keep functionality here when it is owned by Workflows and specifically supports the Assignments concern.

use crate::features::workflows::assignments::{
    WorkflowAssigneeOption, WorkflowAssignmentCandidate, WorkflowAssignmentCreateForm,
    WorkflowAssignmentSummary, WorkflowAssignmentsList, assignee_filter_options,
    filtered_assignments, load_workflow_assignment_assignees, load_workflow_assignment_candidates,
    load_workflow_assignments,
};
use crate::features::workflows::workflow_assignment_candidate_key;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
use crate::utils::text::IntoNonemptyString;
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
/// Renders the workflow assignments page view.
pub fn WorkflowAssignmentsPage() -> impl IntoView {
    let assignments = RwSignal::new(Vec::<WorkflowAssignmentSummary>::new());
    let candidates = RwSignal::new(Vec::<WorkflowAssignmentCandidate>::new());
    let assignees = RwSignal::new(Vec::<WorkflowAssigneeOption>::new());
    let selected_candidate_id = RwSignal::new(String::new());
    let selected_workflow_version_id = RwSignal::new(String::new());
    let selected_node_id = RwSignal::new(String::new());
    let requested_workflow_id = RwSignal::new(String::new());
    let selected_account_ids = RwSignal::new(HashSet::<String>::new());
    let workflow_search = RwSignal::new(String::new());
    let node_search = RwSignal::new(String::new());
    let assignee_search = RwSignal::new(String::new());
    let assignment_search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let state_filter = RwSignal::new("all".to_string());
    let assignee_filter = RwSignal::new("all".to_string());
    let assignments_loading = RwSignal::new(true);
    let assignments_error = RwSignal::new(None::<String>);
    let candidates_loading = RwSignal::new(true);
    let candidates_error = RwSignal::new(None::<String>);
    let assignees_loading = RwSignal::new(false);
    let assignees_error = RwSignal::new(None::<String>);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflow_assignments(assignments, assignments_loading, assignments_error);
        load_workflow_assignment_candidates(candidates, candidates_loading, candidates_error);
        #[cfg(feature = "hydrate")]
        {
            if let Some(assignment_id) = current_search_param("assignment_id") {
                assignment_search.set(assignment_id);
            }
        }
    });

    Effect::new(move |_| {
        let available_candidates = candidates.get();
        let workflow_id = requested_workflow_id
            .get()
            .into_nonempty()
            .or({
                #[cfg(feature = "hydrate")]
                {
                    current_search_param("workflow_id")
                }
                #[cfg(not(feature = "hydrate"))]
                {
                    None
                }
            })
            .unwrap_or_default();
        if workflow_id.is_empty() || !selected_workflow_version_id.get_untracked().is_empty() {
            return;
        }

        if let Some(candidate) = available_candidates.into_iter().find(|candidate| {
            candidate.workflow_id == workflow_id || candidate.workflow_version_id == workflow_id
        }) {
            selected_workflow_version_id.set(candidate.workflow_version_id);
            workflow_search.set(String::new());
            requested_workflow_id.set(String::new());
        }
    });

    Effect::new(move |_| {
        let workflow_version_id = selected_workflow_version_id.get();
        let node_id = selected_node_id.get();
        let next_candidate_id = if workflow_version_id.is_empty() || node_id.is_empty() {
            String::new()
        } else {
            candidates
                .get()
                .into_iter()
                .find(|candidate| {
                    candidate.workflow_version_id == workflow_version_id
                        && candidate.node_id == node_id
                })
                .map(|candidate| workflow_assignment_candidate_key(&candidate))
                .unwrap_or_default()
        };

        if selected_candidate_id.get_untracked() != next_candidate_id {
            selected_candidate_id.set(next_candidate_id);
        }
    });

    Effect::new(move |_| {
        let selected_id = selected_candidate_id.get();
        selected_account_ids.set(HashSet::new());
        let selected_candidate = candidates
            .get()
            .into_iter()
            .find(|candidate| workflow_assignment_candidate_key(candidate) == selected_id);

        if let Some(candidate) = selected_candidate {
            load_workflow_assignment_assignees(
                candidate.workflow_version_id,
                candidate.node_id,
                assignees,
                assignees_loading,
                assignees_error,
            );
        } else {
            assignees.set(Vec::new());
            assignees_loading.set(false);
            assignees_error.set(None);
        }
    });

    let filtered_assignments = move || {
        filtered_assignments(
            assignments.get(),
            &assignment_search.get(),
            &status_filter.get(),
            &state_filter.get(),
            &assignee_filter.get(),
        )
    };
    let assignee_filter_options = move || assignee_filter_options(&assignments.get());

    view! {
        <AppShell active_route="workflows" title="Workflow Assignments">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Assignments"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <section class="route-panel workflows-page workflow-assignments-page">
                    <PageHeader title="Workflow Assignments"/>

                    <WorkflowAssignmentCreateForm
                        assignments=assignments
                        candidates=candidates
                        assignees=assignees
                        selected_candidate_id=selected_candidate_id
                        selected_workflow_version_id=selected_workflow_version_id
                        selected_node_id=selected_node_id
                        selected_account_ids=selected_account_ids
                        workflow_search=workflow_search
                        node_search=node_search
                        assignee_search=assignee_search
                        assignments_loading=assignments_loading
                        assignments_error=assignments_error
                        candidates_loading=candidates_loading
                        candidates_error=candidates_error
                        assignees_loading=assignees_loading
                        assignees_error=assignees_error
                        is_saving=is_saving
                        message=message
                    />

                    {move || message.get().map(|message| view! {
                        <p class="form-message" role="status">{message}</p>
                    })}

                    {move || {
                        if assignments_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading assignments"</h3>
                                    <p>"Fetching workflow assignment records."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(message) = assignments_error.get() {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Workflow assignments unavailable"</h3>
                                    <p>{message}</p>
                                </section>
                            }
                            .into_any()
                        } else {
                            view! {
                                <WorkflowAssignmentsList
                                    assignments=filtered_assignments()
                                    search=assignment_search
                                    status_filter=status_filter
                                    state_filter=state_filter
                                    assignee_filter=assignee_filter
                                    assignee_options=assignee_filter_options()
                                    assignments_signal=assignments
                                    assignments_loading=assignments_loading
                                    assignments_error=assignments_error
                                    message=message
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
