//! Signal state for the workflow assignments page.

use super::types::{
    WorkflowAssigneeOption, WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};
use leptos::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(crate) struct WorkflowAssignmentsPageState {
    pub(crate) assignments: RwSignal<Vec<WorkflowAssignmentSummary>>,
    pub(crate) candidates: RwSignal<Vec<WorkflowAssignmentCandidate>>,
    pub(crate) assignees: RwSignal<Vec<WorkflowAssigneeOption>>,
    pub(crate) selected_candidate_id: RwSignal<String>,
    pub(crate) selected_workflow_version_id: RwSignal<String>,
    pub(crate) selected_node_id: RwSignal<String>,
    pub(crate) requested_workflow_id: RwSignal<String>,
    pub(crate) selected_account_ids: RwSignal<HashSet<String>>,
    pub(crate) workflow_search: RwSignal<String>,
    pub(crate) node_search: RwSignal<String>,
    pub(crate) assignee_search: RwSignal<String>,
    pub(crate) assignment_search: RwSignal<String>,
    pub(crate) status_filter: RwSignal<String>,
    pub(crate) state_filter: RwSignal<String>,
    pub(crate) assignee_filter: RwSignal<String>,
    pub(crate) assignments_loading: RwSignal<bool>,
    pub(crate) assignments_error: RwSignal<Option<String>>,
    pub(crate) candidates_loading: RwSignal<bool>,
    pub(crate) candidates_error: RwSignal<Option<String>>,
    pub(crate) assignees_loading: RwSignal<bool>,
    pub(crate) assignees_error: RwSignal<Option<String>>,
    pub(crate) is_saving: RwSignal<bool>,
    pub(crate) message: RwSignal<Option<String>>,
}

impl WorkflowAssignmentsPageState {
    pub(crate) fn new() -> Self {
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
