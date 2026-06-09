use std::collections::HashSet;
use icons::{PanelRight, Search, X};
use leptos::prelude::*;
use crate::features::shared::FilterHeader as SharedFilterHeader;
use super::*;
use crate::types::route_params::require_route_params;

#[component]
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
        let query = workflow_search.get();
        let selected_node_id = selected_node_id.get();
        let mut seen = HashSet::new();
        let mut workflows = candidates
            .get()
            .into_iter()
            .filter(|candidate| {
                (selected_node_id.is_empty() || candidate.node_id == selected_node_id)
                    && seen.insert(candidate.workflow_version_id.clone())
                && crate::utils::text::text_matches(
                        &query,
                        &[
                            candidate.workflow_name.as_str(),
                            candidate
                                .workflow_version_label
                                .as_deref()
                                .unwrap_or_default(),
                        ],
                    )
            })
            .collect::<Vec<_>>();
        workflows.sort_by(|left, right| {
            left.workflow_name
                .cmp(&right.workflow_name)
                .then(left.workflow_version_id.cmp(&right.workflow_version_id))
        });
        workflows
    };
    let filtered_node_candidates = move || {
        let query = node_search.get();
        let selected_workflow_version_id = selected_workflow_version_id.get();
        let mut seen = HashSet::new();
        let mut nodes = candidates
            .get()
            .into_iter()
            .filter(|candidate| {
                (selected_workflow_version_id.is_empty()
                    || candidate.workflow_version_id == selected_workflow_version_id)
                    && seen.insert(candidate.node_id.clone())
                && crate::utils::text::text_matches(
                        &query,
                        &[candidate.node_name.as_str(), candidate.node_path.as_str()],
                    )
            })
            .collect::<Vec<_>>();
        nodes.sort_by(|left, right| left.node_path.cmp(&right.node_path));
        nodes
    };
    let selected_pair_is_valid = move || {
        let workflow_version_id = selected_workflow_version_id.get();
        let node_id = selected_node_id.get();
        !workflow_version_id.is_empty()
            && !node_id.is_empty()
            && candidates.get().into_iter().any(|candidate| {
                candidate.workflow_version_id == workflow_version_id && candidate.node_id == node_id
            })
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
        let selected_id = selected_workflow_version_id.get();
        candidates
            .get()
            .into_iter()
            .find(|candidate| candidate.workflow_version_id == selected_id)
            .map(|candidate| {
                let revision =
                    workflow_assignment_revision_label(candidate.workflow_version_label.as_deref());
                (candidate.workflow_name, format!("Revision {revision}"))
            })
    };
    let selected_node_summary = move || {
        let selected_id = selected_node_id.get();
        candidates
            .get()
            .into_iter()
            .find(|candidate| candidate.node_id == selected_id)
            .map(|candidate| {
                let node_path = if candidate.node_path.trim().is_empty() {
                    candidate.node_name.clone()
                } else {
                    candidate.node_path.clone()
                };
                (candidate.node_name, node_path)
            })
    };
    let filtered_assignees = move || {
        let query = assignee_search.get();
        assignees
            .get()
            .into_iter()
            .filter(|assignee| {
                crate::utils::text::text_matches(
                    &query,
                    &[assignee.display_name.as_str(), assignee.email.as_str()],
                )
            })
            .collect::<Vec<_>>()
    };
    let filtered_assignments = move || {
        let query = assignment_search.get();
        let status = status_filter.get();
        let state = state_filter.get();
        let assignee = assignee_filter.get();
        assignments
            .get()
            .into_iter()
            .filter(|assignment| {
                let matches_status =
                    status == "all" || workflow_assignment_status_key(assignment) == status;
                let matches_state =
                    state == "all" || workflow_assignment_state(assignment) == state;
                let matches_assignee =
                    assignee == "all" || workflow_assignment_assignee_label(assignment) == assignee;
                matches_status
                    && matches_state
                    && matches_assignee
                    && crate::utils::text::text_matches(
                        &query,
                        &[
                            assignment.workflow_name.as_str(),
                            assignment.workflow_step_title.as_str(),
                            assignment.form_name.as_str(),
                            assignment.node_name.as_str(),
                            assignment.account_display_name.as_str(),
                            assignment.account_email.as_str(),
                            assignment.id.as_str(),
                        ],
                    )
            })
            .collect::<Vec<_>>()
    };
    let assignee_filter_options = move || {
        unique_filter_options(
            assignments
                .get()
                .iter()
                .map(workflow_assignment_assignee_label)
                .collect::<Vec<_>>(),
        )
    };
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

#[component]
fn WorkflowAssignmentsList(
    assignments: Vec<WorkflowAssignmentSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    state_filter: RwSignal<String>,
    assignee_filter: RwSignal<String>,
    assignee_options: Vec<String>,
    assignments_signal: RwSignal<Vec<WorkflowAssignmentSummary>>,
    assignments_loading: RwSignal<bool>,
    assignments_error: RwSignal<Option<String>>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let mut table_assignments = assignments.clone();
    table_assignments.sort_by(|left, right| {
        left.workflow_name
            .to_lowercase()
            .cmp(&right.workflow_name.to_lowercase())
            .then(
                left.account_display_name
                    .to_lowercase()
                    .cmp(&right.account_display_name.to_lowercase()),
            )
            .then(left.id.cmp(&right.id))
    });
    let card_assignments = table_assignments.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_assignments.len();
    let page_count = move || {
        if total_count == 0 {
            1
        } else {
            ((total_count + page_size.get() - 1) / page_size.get()).max(1)
        }
    };
    let current_page = move || page_index.get().min(page_count() - 1);
    let page_start = move || {
        if total_count == 0 {
            0
        } else {
            current_page() * page_size.get()
        }
    };
    let page_end = move || (page_start() + page_size.get()).min(total_count);
    let page_summary = move || {
        if total_count == 0 {
            "No workflow assignments to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} workflow assignments",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };
    let selected_detail = RwSignal::new(None::<WorkflowAssignmentSummary>);
    let close_detail = move |_| selected_detail.set(None);

    view! {
        <div class="forms-list forms-list-responsive-table workflow-assignments-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search assignments"</span>
                        <input
                            type="search"
                            placeholder="Search assignments"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Workflow"</th>
                            <th scope="col">
                                <SharedFilterHeader
                                    label="Assignee"
                                    all_label="All Assignees"
                                    filter=assignee_filter
                                    options=assignee_options
                                    always_searchable=true
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <SharedFilterHeader
                                    label="Work State"
                                    all_label="All States"
                                    filter=state_filter
                                    options=vec!["pending".into(), "draft".into(), "submitted".into()]
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <SharedFilterHeader
                                    label="Status"
                                    all_label="All Statuses"
                                    filter=status_filter
                                    options=vec!["active".into(), "inactive".into()]
                                />
                            </th>
                            <th scope="col">"Assigned"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || if table_assignments.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="6">"No Workflow Assignments to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_assignments
                                .iter()
                                .skip(page_start())
                                .take(page_size.get())
                                .cloned()
                                .map(|assignment| {
                                    let workflow_href = format!("/workflows/{}", assignment.workflow_id);
                                    let state_label = workflow_assignment_state_label(&assignment);
                                    let state_key = workflow_assignment_state(&assignment);
                                    let status_key = workflow_assignment_status_key(&assignment);
                                    let status_label = workflow_assignment_status_label(&assignment);
                                    let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                                    let assignment_for_toggle = assignment.clone();
                                    let assignment_for_detail = assignment.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=workflow_href>{assignment.workflow_name.clone()}</a>
                                            </th>
                                            <td>
                                                <span>{assignment.account_display_name}</span>
                                                <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                            </td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(state_key)>{state_label}</span>
                                            </td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(status_key)>{status_label}</span>
                                            </td>
                                            <td><Timestamp value=assignment.created_at/></td>
                                            <td class="data-table__cell--center">
                                                <DropdownMenu label=format!("Open actions for {}", assignment.workflow_name)>
                                                    <button
                                                        class="dropdown-menu__item"
                                                        type="button"
                                                        role="menuitem"
                                                        on:click=move |_| selected_detail.set(Some(assignment_for_detail.clone()))
                                                    >
                                                        <PanelRight class="dropdown-menu__item-icon"/>
                                                        <span>"View Details"</span>
                                                    </button>
                                                    <button
                                                        class="dropdown-menu__item"
                                                        type="button"
                                                        role="menuitem"
                                                        on:click=move |_| {
                                                            toggle_workflow_assignment(
                                                                assignment_for_toggle.clone(),
                                                                assignments_signal,
                                                                assignments_loading,
                                                                assignments_error,
                                                                message,
                                                            );
                                                        }
                                                    >
                                                        <X class="dropdown-menu__item-icon"/>
                                                        <span>{action_label}</span>
                                                    </button>
                                                </DropdownMenu>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }}
                    </tbody>
                </DataTable>
                <div class="directory-table-pagination" aria-label="Workflow assignments table pagination">
                    <p>{move || page_summary()}</p>
                    <div class="directory-table-pagination__actions">
                        <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                            <span>"Rows"</span>
                            <select
                                prop:value=move || page_size.get().to_string()
                                on:change=move |event| {
                                    if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                        page_size.set(size);
                                        page_index.set(0);
                                    }
                                }
                            >
                                <option value="10">"10"</option>
                                <option value="25">"25"</option>
                                <option value="50">"50"</option>
                            </select>
                        </label>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || current_page() == 0
                            on:click=move |_| {
                                page_index.update(|page| *page = page.saturating_sub(1));
                            }
                        >
                            "Previous"
                        </button>
                        <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || { current_page() + 1 >= page_count() }
                            on:click=move |_| {
                                let last_page = page_count().saturating_sub(1);
                                page_index.update(|page| *page = (*page + 1).min(last_page));
                            }
                        >
                            "Next"
                        </button>
                    </div>
                </div>
            </div>
            <div class="forms-list-mobile-cards workflow-assignment-mobile-cards">
                {move || if card_assignments.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Workflow Assignments to Display"</p> }.into_any()
                } else {
                    card_assignments
                        .iter()
                        .skip(page_start())
                        .take(page_size.get())
                        .cloned()
                        .map(|assignment| {
                            let workflow_href = format!("/workflows/{}", assignment.workflow_id);
                            let node_href = format!("/organization/{}", assignment.node_id);
                            let state_label = workflow_assignment_state_label(&assignment);
                            let state_key = workflow_assignment_state(&assignment);
                            let status_key = workflow_assignment_status_key(&assignment);
                            let status_label = workflow_assignment_status_label(&assignment);
                            let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                            let assignment_for_toggle = assignment.clone();
                            let assignment_for_detail = assignment.clone();
                            view! {
                                <article class="forms-list-mobile-card workflow-assignment-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div class="forms-list-mobile-card__title-row">
                                            <h3><a href=workflow_href>{assignment.workflow_name}</a></h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Assignee"</dt>
                                            <dd>
                                                <span>{assignment.account_display_name}</span>
                                                <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Form"</dt>
                                            <dd>{assignment.form_name}</dd>
                                        </div>
                                        <div>
                                            <dt>"Node"</dt>
                                            <dd><a href=node_href>{assignment.node_name}</a></dd>
                                        </div>
                                        <div>
                                            <dt>"Step"</dt>
                                            <dd>{assignment.workflow_step_title}</dd>
                                        </div>
                                        <div>
                                            <dt>"Work State"</dt>
                                            <dd><span class=status_badge_class(state_key)>{state_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(status_key)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Assigned"</dt>
                                            <dd><Timestamp value=assignment.created_at/></dd>
                                        </div>
                                    </dl>
                                    <div class="workflow-assignment-mobile-card__actions">
                                        <button
                                            class="button button--compact"
                                            type="button"
                                            on:click=move |_| selected_detail.set(Some(assignment_for_detail.clone()))
                                        >
                                            "View Details"
                                        </button>
                                        <button
                                            class="button button--compact"
                                            type="button"
                                            on:click=move |_| {
                                                toggle_workflow_assignment(
                                                    assignment_for_toggle.clone(),
                                                    assignments_signal,
                                                    assignments_loading,
                                                    assignments_error,
                                                    message,
                                                );
                                            }
                                        >
                                            {action_label}
                                        </button>
                                    </div>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
        {move || selected_detail.get().map(|assignment| {
            let workflow_href = format!("/workflows/{}", assignment.workflow_id);
            let node_href = format!("/organization/{}", assignment.node_id);
            let state_key = workflow_assignment_state(&assignment);
            let state_label = workflow_assignment_state_label(&assignment);
            let status_key = workflow_assignment_status_key(&assignment);
            let status_label = workflow_assignment_status_label(&assignment);

            view! {
                <Portal>
                    <section class="sheet-overlay workflow-assignment-detail-overlay" aria-label="Workflow assignment detail">
                        <button class="sheet-overlay__scrim" type="button" aria-label="Close assignment details" on:click=close_detail></button>
                        <aside class="sheet-panel blurred-surface workflow-assignment-detail-sheet" role="dialog" aria-modal="true" aria-label="Workflow assignment details">
                            <div class="sheet-panel__actions">
                                <button class="icon-button sheet-panel__close" type="button" aria-label="Close assignment details" title="Close assignment details" on:click=close_detail>
                                    <X class="icon-button__icon"/>
                                </button>
                            </div>
                            <header class="sheet-panel__header">
                                <p>"Assignment Detail"</p>
                                <h2>{assignment.workflow_name.clone()}</h2>
                            </header>
                            <section class="sheet-panel__section">
                                <h3>"Workflow"</h3>
                                <table class="info-list-table">
                                    <tbody>
                                        <tr>
                                            <th scope="row">"Workflow"</th>
                                                <td><a class="data-table__primary-link" href=workflow_href.clone()>{assignment.workflow_name.clone()}</a></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Revision"</th>
                                            <td>{workflow_assignment_revision_label(assignment.workflow_version_label.as_deref())}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Step"</th>
                                            <td>{assignment.workflow_step_title.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Form"</th>
                                            <td>{assignment.form_name.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Form Version"</th>
                                            <td>{nonempty_text(assignment.form_version_label.as_deref(), "-")}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </section>
                            <section class="sheet-panel__section">
                                <h3>"Assignment"</h3>
                                <table class="info-list-table">
                                    <tbody>
                                        <tr>
                                            <th scope="row">"Node"</th>
                                                <td><a class="data-table__primary-link" href=node_href.clone()>{assignment.node_name.clone()}</a></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Assignee"</th>
                                            <td>{assignment.account_display_name.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Email"</th>
                                            <td>{assignment.account_email.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Work State"</th>
                                            <td><span class=status_badge_class(state_key)>{state_label}</span></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Status"</th>
                                            <td><span class=status_badge_class(status_key)>{status_label}</span></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Assigned"</th>
                                            <td><Timestamp value=assignment.created_at.clone()/></td>
                                        </tr>
                                    </tbody>
                                </table>
                            </section>
                        </aside>
                    </section>
                </Portal>
            }
        })}
    }
}

