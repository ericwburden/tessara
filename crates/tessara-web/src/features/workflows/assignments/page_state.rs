//! Signal state for the workflow assignments page.

use super::types::{
    WorkflowAssigneeOption, WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};
use leptos::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(in crate::features::workflows) struct WorkflowAssignmentsPageState {
    pub(in crate::features::workflows) assignments: RwSignal<Vec<WorkflowAssignmentSummary>>,
    pub(in crate::features::workflows) candidates: RwSignal<Vec<WorkflowAssignmentCandidate>>,
    pub(in crate::features::workflows) assignees: RwSignal<Vec<WorkflowAssigneeOption>>,
    pub(in crate::features::workflows) selected_candidate_id: RwSignal<String>,
    pub(in crate::features::workflows) selected_workflow_version_id: RwSignal<String>,
    pub(in crate::features::workflows) selected_node_id: RwSignal<String>,
    pub(in crate::features::workflows) requested_workflow_id: RwSignal<String>,
    pub(in crate::features::workflows) selected_account_ids: RwSignal<HashSet<String>>,
    pub(in crate::features::workflows) workflow_search: RwSignal<String>,
    pub(in crate::features::workflows) node_search: RwSignal<String>,
    pub(in crate::features::workflows) assignee_search: RwSignal<String>,
    pub(in crate::features::workflows) assignment_search: RwSignal<String>,
    pub(in crate::features::workflows) status_filter: RwSignal<String>,
    pub(in crate::features::workflows) state_filter: RwSignal<String>,
    pub(in crate::features::workflows) assignee_filter: RwSignal<String>,
    pub(in crate::features::workflows) assignments_loading: RwSignal<bool>,
    pub(in crate::features::workflows) assignments_error: RwSignal<Option<String>>,
    pub(in crate::features::workflows) candidates_loading: RwSignal<bool>,
    pub(in crate::features::workflows) candidates_error: RwSignal<Option<String>>,
    pub(in crate::features::workflows) assignees_loading: RwSignal<bool>,
    pub(in crate::features::workflows) assignees_error: RwSignal<Option<String>>,
    pub(in crate::features::workflows) is_saving: RwSignal<bool>,
    pub(in crate::features::workflows) message: RwSignal<Option<String>>,
}

impl WorkflowAssignmentsPageState {
    pub(in crate::features::workflows) fn new() -> Self {
        Self {
            assignments: RwSignal::new(Vec::new()),
            candidates: RwSignal::new(Vec::new()),
            assignees: RwSignal::new(Vec::new()),
            selected_candidate_id: RwSignal::new(String::new()),
            selected_workflow_version_id: RwSignal::new(String::new()),
            selected_node_id: RwSignal::new(String::new()),
            requested_workflow_id: RwSignal::new(String::new()),
            selected_account_ids: RwSignal::new(HashSet::new()),
            workflow_search: RwSignal::new(String::new()),
            node_search: RwSignal::new(String::new()),
            assignee_search: RwSignal::new(String::new()),
            assignment_search: RwSignal::new(String::new()),
            status_filter: RwSignal::new("all".to_string()),
            state_filter: RwSignal::new("all".to_string()),
            assignee_filter: RwSignal::new("all".to_string()),
            assignments_loading: RwSignal::new(true),
            assignments_error: RwSignal::new(None),
            candidates_loading: RwSignal::new(true),
            candidates_error: RwSignal::new(None),
            assignees_loading: RwSignal::new(false),
            assignees_error: RwSignal::new(None),
            is_saving: RwSignal::new(false),
            message: RwSignal::new(None),
        }
    }
}
