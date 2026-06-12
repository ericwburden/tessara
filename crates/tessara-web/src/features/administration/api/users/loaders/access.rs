//! Access detail loader for administration user screens.

use crate::features::administration::models::AdminUserAccessDetail;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

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
