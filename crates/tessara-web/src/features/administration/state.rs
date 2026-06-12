//! State helpers for Administration feature interactions.

use super::display::{admin_user_role_names, admin_user_status_key};
use super::models::{
    AdminCapabilitySummary, AdminUserAccessDetail, AdminUserDetail, AdminUserSummary,
};
use crate::features::organization::AdminRoleSummary;
use crate::features::shared::unique_filter_options;
use crate::utils::text::text_matches;
use leptos::prelude::{RwSignal, Update};

#[derive(Clone, Copy)]
pub(crate) struct AdminUserAccessState {
    pub(crate) detail: RwSignal<Option<AdminUserAccessDetail>>,
    pub(crate) capability_catalog: RwSignal<Vec<AdminCapabilitySummary>>,
    pub(crate) selected_scope_node_ids: RwSignal<Vec<String>>,
    pub(crate) selected_delegate_account_ids: RwSignal<Vec<String>>,
    pub(crate) is_loading: RwSignal<bool>,
    pub(crate) is_saving: RwSignal<bool>,
    pub(crate) load_error: RwSignal<Option<String>>,
    pub(crate) message: RwSignal<Option<String>>,
}

impl AdminUserAccessState {
    pub(crate) fn new() -> Self {
        Self {
            detail: RwSignal::new(None::<AdminUserAccessDetail>),
            capability_catalog: RwSignal::new(Vec::<AdminCapabilitySummary>::new()),
            selected_scope_node_ids: RwSignal::new(Vec::<String>::new()),
            selected_delegate_account_ids: RwSignal::new(Vec::<String>::new()),
            is_loading: RwSignal::new(true),
            is_saving: RwSignal::new(false),
            load_error: RwSignal::new(None::<String>),
            message: RwSignal::new(None::<String>),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AdminUserEditState {
    pub(crate) detail: RwSignal<Option<AdminUserDetail>>,
    pub(crate) roles: RwSignal<Vec<AdminRoleSummary>>,
    pub(crate) email: RwSignal<String>,
    pub(crate) display_name: RwSignal<String>,
    pub(crate) password: RwSignal<String>,
    pub(crate) is_active: RwSignal<bool>,
    pub(crate) selected_role_ids: RwSignal<Vec<String>>,
    pub(crate) is_loading: RwSignal<bool>,
    pub(crate) is_saving: RwSignal<bool>,
    pub(crate) load_error: RwSignal<Option<String>>,
    pub(crate) message: RwSignal<Option<String>>,
}

impl AdminUserEditState {
    pub(crate) fn new() -> Self {
        Self {
            detail: RwSignal::new(None::<AdminUserDetail>),
            roles: RwSignal::new(Vec::<AdminRoleSummary>::new()),
            email: RwSignal::new(String::new()),
            display_name: RwSignal::new(String::new()),
            password: RwSignal::new(String::new()),
            is_active: RwSignal::new(true),
            selected_role_ids: RwSignal::new(Vec::<String>::new()),
            is_loading: RwSignal::new(true),
            is_saving: RwSignal::new(false),
            load_error: RwSignal::new(None::<String>),
            message: RwSignal::new(None::<String>),
        }
    }
}

pub(crate) fn filtered_admin_users(
    users: Vec<AdminUserSummary>,
    query: &str,
    status: &str,
    role: &str,
) -> Vec<AdminUserSummary> {
    users
        .into_iter()
        .filter(|user| {
            let status_key = admin_user_status_key(user);
            let role_names = admin_user_role_names(user);
            let matches_status = status == "all" || status == status_key;
            let matches_role =
                role == "all" || user.roles.iter().any(|user_role| user_role.name == role);
            matches_status
                && matches_role
                && text_matches(
                    query,
                    &[
                        user.display_name.as_str(),
                        user.email.as_str(),
                        status_key,
                        role_names.as_str(),
                    ],
                )
        })
        .collect()
}

pub(crate) fn admin_user_role_filter_options(users: &[AdminUserSummary]) -> Vec<String> {
    unique_filter_options(users.iter().flat_map(|user| {
        user.roles
            .iter()
            .map(|role| role.name.clone())
            .collect::<Vec<_>>()
    }))
}

/// Toggles a string value in a selected values signal.
pub(crate) fn toggle_string_selection(
    selection: RwSignal<Vec<String>>,
    value: String,
    selected: bool,
) {
    selection.update(|values| {
        if selected {
            if !values.iter().any(|existing| existing == &value) {
                values.push(value);
            }
        } else {
            values.retain(|existing| existing != &value);
        }
    });
}
