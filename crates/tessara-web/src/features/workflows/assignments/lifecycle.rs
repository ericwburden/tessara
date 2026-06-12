//! Reactive lifecycle wiring for the workflow assignments page.

use super::state::WorkflowAssignmentsPageState;
use super::{
    load_workflow_assignment_assignees, load_workflow_assignment_candidates,
    load_workflow_assignments, workflow_assignment_candidate_key,
};
use crate::utils::text::IntoNonemptyString;
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;
use leptos::prelude::*;

pub(in crate::features::workflows) fn install_workflow_assignments_page_effects(
    state: WorkflowAssignmentsPageState,
) {
    Effect::new(move |_| {
        load_workflow_assignments(
            state.assignments,
            state.assignments_loading,
            state.assignments_error,
        );
        load_workflow_assignment_candidates(
            state.candidates,
            state.candidates_loading,
            state.candidates_error,
        );
        #[cfg(feature = "hydrate")]
        {
            if let Some(assignment_id) = current_search_param("assignment_id") {
                state.assignment_search.set(assignment_id);
            }
        }
    });

    Effect::new(move |_| {
        let available_candidates = state.candidates.get();
        let workflow_id = state
            .requested_workflow_id
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
        if workflow_id.is_empty()
            || !state
                .selected_workflow_version_id
                .get_untracked()
                .is_empty()
        {
            return;
        }

        if let Some(candidate) = available_candidates.into_iter().find(|candidate| {
            candidate.workflow_id == workflow_id || candidate.workflow_version_id == workflow_id
        }) {
            state
                .selected_workflow_version_id
                .set(candidate.workflow_version_id);
            state.workflow_search.set(String::new());
            state.requested_workflow_id.set(String::new());
        }
    });

    Effect::new(move |_| {
        let workflow_version_id = state.selected_workflow_version_id.get();
        let node_id = state.selected_node_id.get();
        let next_candidate_id = if workflow_version_id.is_empty() || node_id.is_empty() {
            String::new()
        } else {
            state
                .candidates
                .get()
                .into_iter()
                .find(|candidate| {
                    candidate.workflow_version_id == workflow_version_id
                        && candidate.node_id == node_id
                })
                .map(|candidate| workflow_assignment_candidate_key(&candidate))
                .unwrap_or_default()
        };

        if state.selected_candidate_id.get_untracked() != next_candidate_id {
            state.selected_candidate_id.set(next_candidate_id);
        }
    });

    Effect::new(move |_| {
        let selected_id = state.selected_candidate_id.get();
        state.selected_account_ids.set(Default::default());
        let selected_candidate = state
            .candidates
            .get()
            .into_iter()
            .find(|candidate| workflow_assignment_candidate_key(candidate) == selected_id);

        if let Some(candidate) = selected_candidate {
            load_workflow_assignment_assignees(
                candidate.workflow_version_id,
                candidate.node_id,
                state.assignees,
                state.assignees_loading,
                state.assignees_error,
            );
        } else {
            state.assignees.set(Vec::new());
            state.assignees_loading.set(false);
            state.assignees_error.set(None);
        }
    });
}
