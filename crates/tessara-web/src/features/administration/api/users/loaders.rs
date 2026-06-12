//! Signal-aware loaders for administration user screens.

use crate::features::administration::models::{
    AdminCapabilitySummary, AdminUserAccessDetail, AdminUserDetail, AdminUserSummary,
};
use crate::features::organization::AdminRoleSummary;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

/// Loads the administration users list.
pub(crate) fn load_admin_users(
    users: RwSignal<Vec<AdminUserSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/admin/users")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    users.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<AdminUserSummary>>().await {
                        Ok(loaded_users) => {
                            users.set(loaded_users);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            users.set(Vec::new());
                            load_error.set(Some(format!("Unable to parse users: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    users.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load users. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    users.set(Vec::new());
                    load_error.set(Some(format!("Unable to load users: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (users, is_loading, load_error);
    }
}

/// Loads the administration capability catalog.
pub(crate) fn load_admin_capability_catalog(capabilities: RwSignal<Vec<AdminCapabilitySummary>>) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match gloo_net::http::Request::get("/api/admin/capabilities")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    if let Ok(items) = response.json::<Vec<AdminCapabilitySummary>>().await {
                        capabilities.set(items);
                    }
                }
                _ => {}
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = capabilities;
    }
}

/// Loads an administration user's access detail.
pub(crate) fn load_admin_user_access(
    account_id: String,
    detail: RwSignal<Option<AdminUserAccessDetail>>,
    selected_scope_node_ids: RwSignal<Vec<String>>,
    selected_delegate_account_ids: RwSignal<Vec<String>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get(&format!("/api/admin/users/{account_id}/access"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<AdminUserAccessDetail>().await {
                        Ok(loaded_detail) => {
                            selected_scope_node_ids.set(
                                loaded_detail
                                    .scope_nodes
                                    .iter()
                                    .map(|node| node.node_id.clone())
                                    .collect(),
                            );
                            selected_delegate_account_ids.set(
                                loaded_detail
                                    .delegations
                                    .iter()
                                    .map(|delegation| delegation.account_id.clone())
                                    .collect(),
                            );
                            detail.set(Some(loaded_detail));
                            is_loading.set(false);
                        }
                        Err(error) => {
                            detail.set(None);
                            load_error
                                .set(Some(format!("Unable to parse user permissions: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load user permissions. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    load_error.set(Some(format!("Unable to load user permissions: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            account_id,
            detail,
            selected_scope_node_ids,
            selected_delegate_account_ids,
            is_loading,
            load_error,
        );
    }
}

#[allow(clippy::too_many_arguments)]
/// Loads an administration user's edit context.
pub(crate) fn load_admin_user_edit_context(
    account_id: String,
    detail: RwSignal<Option<AdminUserDetail>>,
    roles: RwSignal<Vec<AdminRoleSummary>>,
    email: RwSignal<String>,
    display_name: RwSignal<String>,
    is_active: RwSignal<bool>,
    selected_role_ids: RwSignal<Vec<String>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            let user_response =
                gloo_net::http::Request::get(&format!("/api/admin/users/{account_id}"))
                    .send()
                    .await;
            let roles_response = gloo_net::http::Request::get("/api/admin/roles")
                .send()
                .await;

            match (user_response, roles_response) {
                (Ok(user_response), _) if user_response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(user_response), Ok(roles_response))
                    if user_response.ok() && roles_response.ok() =>
                {
                    let loaded_user = user_response.json::<AdminUserDetail>().await;
                    let loaded_roles = roles_response.json::<Vec<AdminRoleSummary>>().await;
                    match (loaded_user, loaded_roles) {
                        (Ok(user), Ok(available_roles)) => {
                            email.set(user.email.clone());
                            display_name.set(user.display_name.clone());
                            is_active.set(user.is_active);
                            selected_role_ids
                                .set(user.roles.iter().map(|role| role.id.clone()).collect());
                            detail.set(Some(user));
                            roles.set(available_roles);
                            is_loading.set(false);
                        }
                        (Err(error), _) => {
                            load_error.set(Some(format!("Unable to parse user: {error}")));
                            is_loading.set(false);
                        }
                        (_, Err(error)) => {
                            load_error.set(Some(format!("Unable to parse roles: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                (Ok(user_response), _) if !user_response.ok() => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load user. Server returned {}.",
                        user_response.status()
                    )));
                    is_loading.set(false);
                }
                (_, Ok(roles_response)) if !roles_response.ok() => {
                    roles.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load roles. Server returned {}.",
                        roles_response.status()
                    )));
                    is_loading.set(false);
                }
                (Err(error), _) => {
                    load_error.set(Some(format!("Unable to load user: {error}")));
                    is_loading.set(false);
                }
                (_, Err(error)) => {
                    load_error.set(Some(format!("Unable to load roles: {error}")));
                    is_loading.set(false);
                }
                _ => {
                    load_error.set(Some("Unable to load user edit context.".into()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            account_id,
            detail,
            roles,
            email,
            display_name,
            is_active,
            selected_role_ids,
            is_loading,
            load_error,
        );
    }
}
