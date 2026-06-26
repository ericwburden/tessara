//! Authentication guard helpers.
//!
//! Keep route/session gating helpers here so pages can delegate auth decisions to one module.

#[cfg(feature = "hydrate")]
use crate::features::auth::api;
#[cfg(feature = "hydrate")]
use crate::state::navigation;
#[cfg(feature = "hydrate")]
use leptos::task::spawn_local;

#[cfg(feature = "hydrate")]
pub fn require_authenticated_route(active_route: &'static str) {
    if active_route == "home" {
        return;
    }

    spawn_local(async move {
        let session = api::fetch_session().await;
        let authenticated = session
            .as_ref()
            .map(|session| session.authenticated)
            .unwrap_or(false);

        if !authenticated {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/login");
            }
            return;
        }

        if let Some(item) = navigation::nav_item_for_route(active_route) {
            let capabilities = session
                .and_then(|session| session.account)
                .map(|account| account.capabilities)
                .unwrap_or_default();
            if !navigation::nav_item_is_allowed(item, &capabilities)
                && let Some(window) = web_sys::window()
            {
                let _ = window.location().set_href("/");
            }
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub fn require_authenticated_route(active_route: &'static str) {
    let _ = active_route;
}
