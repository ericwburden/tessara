//! Assignments support for the Workflows feature.
//!
//! Keep functionality here when it is owned by Workflows and specifically supports the Assignments concern.

use crate::features::workflows::assignments::{
    WorkflowAssigneeOption, WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
    WorkflowAssignmentsList, assignee_filter_options, filtered_assignees, filtered_assignments,
    filtered_node_candidates, filtered_workflow_candidates, load_workflow_assignment_assignees,
    load_workflow_assignment_candidates, load_workflow_assignments,
    selected_node_summary as selected_node_assignment_summary,
    selected_workflow_summary as selected_workflow_assignment_summary,
    workflow_assignment_pair_is_valid,
};
use crate::features::workflows::{
    submit_workflow_assignment_bulk, workflow_assignee_label, workflow_assignment_candidate_key,
    workflow_assignment_revision_label,
};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader, empty_view,
};
use crate::utils::text::IntoNonemptyString;
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;
use icons::{Search, X};
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

    let filtered_workflow_candidates = move || {
        filtered_workflow_candidates(
            candidates.get(),
            &workflow_search.get(),
            &selected_node_id.get(),
        )
    };
    let filtered_node_candidates = move || {
        filtered_node_candidates(
            candidates.get(),
            &node_search.get(),
            &selected_workflow_version_id.get(),
        )
    };
    let selected_pair_is_valid = move || {
        workflow_assignment_pair_is_valid(
            &candidates.get(),
            &selected_workflow_version_id.get(),
            &selected_node_id.get(),
        )
    };
    let invalid_pair_message = move || {
        if selected_workflow_version_id.get().is_empty()
            || selected_node_id.get().is_empty()
            || selected_pair_is_valid()
        {
            None
        } else {
            Some("That workflow is not valid for the selected node.".to_string())
        }
    };
    let selected_workflow_summary = move || {
        selected_workflow_assignment_summary(candidates.get(), &selected_workflow_version_id.get())
    };
    let selected_node_summary =
        move || selected_node_assignment_summary(candidates.get(), &selected_node_id.get());
    let filtered_assignees = move || filtered_assignees(assignees.get(), &assignee_search.get());
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
    let can_create = move || {
        !is_saving.get()
            && !selected_candidate_id.get().is_empty()
            && !selected_account_ids.get().is_empty()
    };

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

                    <form
                        class="native-form workflow-assignment-create-form"
                        on:submit=move |event| {
                            event.prevent_default();
                            submit_workflow_assignment_bulk(
                                selected_candidate_id,
                                candidates,
                                selected_account_ids,
                                assignments,
                                assignments_loading,
                                assignments_error,
                                is_saving,
                                message,
                            );
                        }
                    >
                        <section class="form-section">
                            <div class="form-builder-section-card__header">
                                <h3>"Create Assignment"</h3>
                            </div>
                            <div class="workflow-assignment-create-grid">
                                <section class="workflow-assignment-pair-list" aria-labelledby="workflow-assignment-workflow-list">
                                    <div class="workflow-assignment-pair-list__header">
                                        <h4 id="workflow-assignment-workflow-list">"Workflow"</h4>
                                    </div>
                                    {move || {
                                        if let Some((workflow_name, version)) = selected_workflow_summary() {
                                            view! {
                                                <div class="workflow-assignment-selected-option">
                                                    <div>
                                                        <strong>{workflow_name}</strong>
                                                        <span>{version}</span>
                                                    </div>
                                                    <button
                                                        class="icon-button icon-button--control"
                                                        type="button"
                                                        aria-label="Clear selected workflow"
                                                        on:click=move |_| {
                                                            selected_workflow_version_id.set(String::new());
                                                            selected_candidate_id.set(String::new());
                                                            selected_account_ids.set(HashSet::new());
                                                        }
                                                    >
                                                        <X/>
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <label class="searchable-data-table__search searchable-data-table__control workflow-assignment-candidate-search">
                                                    <Search class="searchable-data-table__control-icon"/>
                                                    <span class="sr-only">"Search workflows"</span>
                                                    <input
                                                        type="search"
                                                        placeholder="Search workflows"
                                                        prop:value=move || workflow_search.get()
                                                        on:input=move |event| workflow_search.set(event_target_value(&event))
                                                    />
                                                </label>
                                                <div class="workflow-assignment-pair-list__options">
                                                    {move || {
                                                        let options = filtered_workflow_candidates();
                                                        if options.is_empty() {
                                                            view! { <p class="workflow-assignee-picker__empty">"No Workflows to Display"</p> }.into_any()
                                                        } else {
                                                            options.into_iter().map(|candidate| {
                                                                let workflow_version_id = candidate.workflow_version_id.clone();
                                                                let workflow_version_id_for_class = workflow_version_id.clone();
                                                                let revision = workflow_assignment_revision_label(candidate.workflow_version_label.as_deref());
                                                                view! {
                                                                    <button
                                                                        class=move || if selected_workflow_version_id.get() == workflow_version_id_for_class {
                                                                            "workflow-assignment-pair-option is-selected"
                                                                        } else {
                                                                            "workflow-assignment-pair-option"
                                                                        }
                                                                        type="button"
                                                                        disabled=move || candidates_loading.get()
                                                                        on:click=move |_| {
                                                                            let workflow_version_id = workflow_version_id.clone();
                                                                            selected_workflow_version_id.set(workflow_version_id.clone());
                                                                            let selected_node = selected_node_id.get_untracked();
                                                                            if !selected_node.is_empty()
                                                                                && !candidates.get_untracked().into_iter().any(|candidate| {
                                                                                    candidate.workflow_version_id == workflow_version_id
                                                                                        && candidate.node_id == selected_node
                                                                                })
                                                                            {
                                                                                selected_node_id.set(String::new());
                                                                            }
                                                                        }
                                                                    >
                                                                        <strong>{candidate.workflow_name}</strong>
                                                                        <span>{format!("Revision {revision}")}</span>
                                                                    </button>
                                                                }
                                                            }).collect_view().into_any()
                                                        }
                                                    }}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </section>
                                <section class="workflow-assignment-pair-list" aria-labelledby="workflow-assignment-node-list">
                                    <div class="workflow-assignment-pair-list__header">
                                        <h4 id="workflow-assignment-node-list">"Node"</h4>
                                    </div>
                                    {move || {
                                        if let Some((node_name, node_path)) = selected_node_summary() {
                                            view! {
                                                <div class="workflow-assignment-selected-option">
                                                    <div>
                                                        <strong>{node_name}</strong>
                                                        <span>{node_path}</span>
                                                    </div>
                                                    <button
                                                        class="icon-button icon-button--control"
                                                        type="button"
                                                        aria-label="Clear selected node"
                                                        on:click=move |_| {
                                                            selected_node_id.set(String::new());
                                                            selected_candidate_id.set(String::new());
                                                            selected_account_ids.set(HashSet::new());
                                                        }
                                                    >
                                                        <X/>
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <label class="searchable-data-table__search searchable-data-table__control workflow-assignment-candidate-search">
                                                    <Search class="searchable-data-table__control-icon"/>
                                                    <span class="sr-only">"Search nodes"</span>
                                                    <input
                                                        type="search"
                                                        placeholder="Search nodes"
                                                        prop:value=move || node_search.get()
                                                        on:input=move |event| node_search.set(event_target_value(&event))
                                                    />
                                                </label>
                                                <div class="workflow-assignment-pair-list__options">
                                                    {move || {
                                                        let options = filtered_node_candidates();
                                                        if options.is_empty() {
                                                            view! { <p class="workflow-assignee-picker__empty">"No Nodes to Display"</p> }.into_any()
                                                        } else {
                                                            options.into_iter().map(|candidate| {
                                                                let node_id = candidate.node_id.clone();
                                                                let node_id_for_class = node_id.clone();
                                                                view! {
                                                                    <button
                                                                        class=move || if selected_node_id.get() == node_id_for_class {
                                                                            "workflow-assignment-pair-option is-selected"
                                                                        } else {
                                                                            "workflow-assignment-pair-option"
                                                                        }
                                                                        type="button"
                                                                        disabled=move || candidates_loading.get()
                                                                        on:click=move |_| {
                                                                            let node_id = node_id.clone();
                                                                            selected_node_id.set(node_id.clone());
                                                                            let selected_workflow = selected_workflow_version_id.get_untracked();
                                                                            if !selected_workflow.is_empty()
                                                                                && !candidates.get_untracked().into_iter().any(|candidate| {
                                                                                    candidate.workflow_version_id == selected_workflow
                                                                                        && candidate.node_id == node_id
                                                                                })
                                                                            {
                                                                                selected_workflow_version_id.set(String::new());
                                                                            }
                                                                        }
                                                                    >
                                                                        <strong>{candidate.node_name}</strong>
                                                                        <span>{candidate.node_path}</span>
                                                                    </button>
                                                                }
                                                            }).collect_view().into_any()
                                                        }
                                                    }}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </section>
                            </div>
                            {move || invalid_pair_message().map(|message| view! {
                                <p class="form-message" role="alert">{message}</p>
                            })}
                            {move || {
                                if let Some(message) = candidates_error.get() {
                                    view! {
                                        <section class="organization-state is-error" role="alert">
                                            <h3>"Assignment candidates unavailable"</h3>
                                            <p>{message}</p>
                                        </section>
                                    }
                                    .into_any()
                                } else if candidates_loading.get() {
                                    view! {
                                        <section class="organization-state" aria-live="polite">
                                            <h3>"Loading candidates"</h3>
                                            <p>"Fetching eligible workflow and node combinations."</p>
                                        </section>
                                    }
                                    .into_any()
                                } else {
                                    empty_view()
                                }
                            }}
                            <div class="workflow-assignee-picker">
                                <h4>"Eligible Assignees"</h4>
                                {move || if selected_candidate_id.get().is_empty() {
                                    empty_view()
                                } else {
                                    view! {
                                        <label class="searchable-data-table__search searchable-data-table__control workflow-assignment-candidate-search">
                                            <Search class="searchable-data-table__control-icon"/>
                                            <span class="sr-only">"Search assignees"</span>
                                            <input
                                                type="search"
                                                placeholder="Search assignees"
                                                prop:value=move || assignee_search.get()
                                                on:input=move |event| assignee_search.set(event_target_value(&event))
                                            />
                                        </label>
                                    }.into_any()
                                }}
                                {move || {
                                    if selected_candidate_id.get().is_empty() {
                                        view! { <p class="workflow-assignee-picker__empty">"Select a candidate to load assignees."</p> }.into_any()
                                    } else if assignees_loading.get() {
                                        view! { <p class="workflow-assignee-picker__empty">"Loading assignees."</p> }.into_any()
                                    } else if let Some(message) = assignees_error.get() {
                                        view! { <p class="workflow-assignee-picker__empty">{message}</p> }.into_any()
                                    } else {
                                        let options = filtered_assignees();
                                        if options.is_empty() {
                                            view! { <p class="workflow-assignee-picker__empty">"No eligible assignees to display."</p> }.into_any()
                                        } else {
                                            options
                                                .into_iter()
                                                .map(|assignee| {
                                                    let account_id = assignee.account_id.clone();
                                                    let account_id_for_checked = account_id.clone();
                                                    let label = workflow_assignee_label(&assignee);
                                                    view! {
                                                        <label class="workflow-assignee-option">
                                                            <input
                                                                type="checkbox"
                                                                prop:checked=move || selected_account_ids.get().contains(&account_id_for_checked)
                                                                on:change=move |event| {
                                                                    let is_checked = event_target_checked(&event);
                                                                    let account_id = account_id.clone();
                                                                    selected_account_ids.update(|selected| {
                                                                        if is_checked {
                                                                            selected.insert(account_id);
                                                                        } else {
                                                                            selected.remove(&account_id);
                                                                        }
                                                                    });
                                                                }
                                                            />
                                                            <span>{label}</span>
                                                        </label>
                                                    }
                                                })
                                                .collect_view()
                                                .into_any()
                                        }
                                    }
                                }}
                            </div>
                            <div class="form-actions">
                                <button class="button button--secondary" type="submit" disabled=move || !can_create()>
                                    {move || if is_saving.get() { "Creating..." } else { "Create Assignments" }}
                                </button>
                            </div>
                        </section>
                    </form>

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
