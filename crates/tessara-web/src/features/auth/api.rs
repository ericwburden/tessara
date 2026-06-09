#[cfg(feature = "hydrate")]
use gloo_net::http::Request;

use leptos::prelude::RwSignal;
#[cfg(feature = "hydrate")]
use leptos::prelude::Set;
#[cfg(feature = "hydrate")]
use leptos::task::spawn_local;

use crate::features::auth::types::{SessionStateResponse, ShellAccountSummary};

#[cfg(feature = "hydrate")]
pub async fn fetch_session() -> Option<SessionStateResponse> {
    let response = Request::get("/api/auth/session").send().await;

    match response {
        Ok(response) if response.ok() => response.json::<SessionStateResponse>().await.ok(),
        _ => None,
    }
}

#[cfg(not(feature = "hydrate"))]
pub async fn fetch_session() -> Option<SessionStateResponse> {
    None
}

#[cfg(feature = "hydrate")]
pub fn load_shell_account(account: RwSignal<Option<ShellAccountSummary>>) {
    spawn_local(async move {
        let response = Request::get("/api/auth/session").send().await;

        match response {
            Ok(response) if response.ok() => {
                let session = response.json::<SessionStateResponse>().await.ok();
                account.set(session.and_then(|session| {
                    if !session.authenticated {
                        return None;
                    }
                    session.account.map(Into::into)
                }));
            }
            _ => account.set(None),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub fn load_shell_account(account: RwSignal<Option<ShellAccountSummary>>) {
    let _ = account;
}

#[cfg(feature = "hydrate")]
pub fn submit_logout() {
    spawn_local(async move {
        let _ = Request::delete("/api/auth/logout").send().await;

        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/login");
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub fn submit_logout() {}
