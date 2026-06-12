//! List and catalog loaders for administration user screens.

use crate::features::administration::models::{AdminCapabilitySummary, AdminUserSummary};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

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
