//! Role page state and filtering helpers.

use crate::features::administration::models::{AdminCapabilitySummary, AdminRoleDetail};
use crate::features::organization::AdminRoleSummary;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub(super) struct AdministrationRolesPageState {
    pub(super) roles: RwSignal<Vec<AdminRoleSummary>>,
    pub(super) capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    pub(super) selected_role_id: RwSignal<Option<String>>,
    pub(super) selected_role_detail: RwSignal<Option<AdminRoleDetail>>,
    pub(super) search: RwSignal<String>,
    pub(super) is_loading: RwSignal<bool>,
    pub(super) detail_loading: RwSignal<bool>,
    pub(super) is_saving: RwSignal<bool>,
    pub(super) message: RwSignal<Option<String>>,
    pub(super) sheet_open: RwSignal<bool>,
    pub(super) editing_role_id: RwSignal<Option<String>>,
    pub(super) role_name: RwSignal<String>,
    pub(super) selected_capability_ids: RwSignal<Vec<String>>,
    pub(super) capability_search: RwSignal<String>,
}

impl AdministrationRolesPageState {
    pub(super) fn new() -> Self {
        Self {
            roles: RwSignal::new(Vec::new()),
            capabilities: RwSignal::new(Vec::new()),
            selected_role_id: RwSignal::new(None),
            selected_role_detail: RwSignal::new(None),
            search: RwSignal::new(String::new()),
            is_loading: RwSignal::new(true),
            detail_loading: RwSignal::new(false),
            is_saving: RwSignal::new(false),
            message: RwSignal::new(None),
            sheet_open: RwSignal::new(false),
            editing_role_id: RwSignal::new(None),
            role_name: RwSignal::new(String::new()),
            selected_capability_ids: RwSignal::new(Vec::new()),
            capability_search: RwSignal::new(String::new()),
        }
    }
}

pub(super) fn filtered_admin_roles(
    roles: RwSignal<Vec<AdminRoleSummary>>,
    search: RwSignal<String>,
) -> Vec<AdminRoleSummary> {
    let query = search.get().trim().to_lowercase();
    roles
        .get()
        .into_iter()
        .filter(|role| query.is_empty() || role.name.to_lowercase().contains(&query))
        .collect()
}
