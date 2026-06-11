//! State helpers for Administration feature interactions.

use super::display::{admin_user_role_names, admin_user_status_key};
use super::models::AdminUserSummary;
use crate::features::shared::unique_filter_options;
use crate::utils::text::text_matches;
use leptos::prelude::{RwSignal, Update};

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
