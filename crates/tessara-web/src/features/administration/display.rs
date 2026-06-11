//! Display helpers for Administration feature screens.

use crate::features::administration::models::AdminUserSummary;

/// Returns the filter key for an admin user's active status.
pub(crate) fn admin_user_status_key(user: &AdminUserSummary) -> &'static str {
    if user.is_active { "active" } else { "inactive" }
}

/// Returns the visible label for an admin user's active status.
pub(crate) fn admin_user_status_label(user: &AdminUserSummary) -> &'static str {
    if user.is_active { "Active" } else { "Inactive" }
}

/// Formats the role names assigned to an admin user.
pub(crate) fn admin_user_role_names(user: &AdminUserSummary) -> String {
    if user.roles.is_empty() {
        "No roles".to_string()
    } else {
        user.roles
            .iter()
            .map(|role| role.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Returns the visible label for an editable administration scope.
pub(crate) fn admin_editable_label(is_editable: bool) -> &'static str {
    if is_editable {
        "Editable"
    } else {
        "Not editable"
    }
}
