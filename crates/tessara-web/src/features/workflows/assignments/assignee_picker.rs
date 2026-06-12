//! Workflow assignment assignee picker.

use super::filtering::filtered_assignees;
use super::types::WorkflowAssigneeOption;
use crate::features::workflows::workflow_assignee_label;
use crate::ui::empty_view;
use icons::Search;
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub(in crate::features::workflows) fn WorkflowAssignmentAssigneePicker(
    assignees: RwSignal<Vec<WorkflowAssigneeOption>>,
    selected_candidate_id: RwSignal<String>,
    selected_account_ids: RwSignal<HashSet<String>>,
    assignee_search: RwSignal<String>,
    assignees_loading: RwSignal<bool>,
    assignees_error: RwSignal<Option<String>>,
) -> impl IntoView {
    let filtered_assignees = move || filtered_assignees(assignees.get(), &assignee_search.get());

    view! {
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
    }
}
