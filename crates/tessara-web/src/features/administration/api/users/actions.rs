//! Signal-aware actions for administration user screens.

#[cfg(feature = "hydrate")]
use crate::features::administration::models::{
    UpdateAdminUserAccessPayload, UpdateAdminUserPayload,
};
#[cfg(feature = "hydrate")]
use crate::features::organization::IdResponse;
#[cfg(feature = "hydrate")]
use crate::http::{navigate_to_href, send_json_request};
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
/// Submits an administration user account update.
pub(crate) fn submit_update_admin_user(
    account_id: String,
    email: RwSignal<String>,
    display_name: RwSignal<String>,
    password: RwSignal<String>,
    is_active: RwSignal<bool>,
    selected_role_ids: RwSignal<Vec<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);
            let password_value = password.get().trim().to_string();
            let payload = UpdateAdminUserPayload {
                email: email.get().trim().to_string(),
                display_name: display_name.get().trim().to_string(),
                password: (!password_value.is_empty()).then_some(password_value),
                is_active: is_active.get(),
                role_ids: selected_role_ids.get(),
            };

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("User update could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_request::<IdResponse>(
                gloo_net::http::Request::put(&format!("/api/admin/users/{account_id}")),
                Some(body),
                "Update user",
            )
            .await
            {
                Ok(_) => navigate_to_href(&format!("/administration/users/{account_id}")),
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            account_id,
            email,
            display_name,
            password,
            is_active,
            selected_role_ids,
            is_saving,
            message,
        );
    }
}

/// Submits an administration user access update.
pub(crate) fn submit_update_admin_user_access(
    account_id: String,
    selected_scope_node_ids: RwSignal<Vec<String>>,
    selected_delegate_account_ids: RwSignal<Vec<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);
            let payload = UpdateAdminUserAccessPayload {
                scope_node_ids: selected_scope_node_ids.get(),
                delegate_account_ids: selected_delegate_account_ids.get(),
            };

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Permission update could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_request::<IdResponse>(
                gloo_net::http::Request::put(&format!("/api/admin/users/{account_id}/access")),
                Some(body),
                "Update permissions",
            )
            .await
            {
                Ok(_) => navigate_to_href(&format!("/administration/users/{account_id}")),
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            account_id,
            selected_scope_node_ids,
            selected_delegate_account_ids,
            is_saving,
            message,
        );
    }
}
