//! Owns the state::session module behavior.

#[cfg(feature = "hydrate")]
use crate::features::auth;
use crate::features::auth::ShellAccountSummary;
#[cfg(feature = "hydrate")]
use leptos::context::{provide_context, use_context};
use leptos::prelude::RwSignal;
#[cfg(feature = "hydrate")]
use leptos::prelude::Set;
#[cfg(feature = "hydrate")]
use leptos::task::spawn_local;

#[cfg(feature = "hydrate")]
#[derive(Clone)]
pub(crate) struct ShellSessionState {
    shell_account: RwSignal<Option<ShellAccountSummary>>,
}

#[cfg(feature = "hydrate")]
/// Handles the shell session account behavior.
pub(crate) fn shell_session_account() -> RwSignal<Option<ShellAccountSummary>> {
    use_context::<ShellSessionState>()
        .map(|state| state.shell_account)
        .unwrap_or_else(|| RwSignal::new(None))
}

#[cfg(not(feature = "hydrate"))]
/// Handles the shell session account behavior.
pub(crate) fn shell_session_account() -> RwSignal<Option<ShellAccountSummary>> {
    RwSignal::new(None)
}

#[cfg(feature = "hydrate")]
/// Handles the provide shell session behavior.
pub(crate) fn provide_shell_session() -> RwSignal<Option<ShellAccountSummary>> {
    let shell_account = RwSignal::new(None::<ShellAccountSummary>);
    load_shell_account(shell_account);
    provide_context(ShellSessionState { shell_account });
    shell_account
}

#[cfg(not(feature = "hydrate"))]
/// Handles the provide shell session behavior.
pub(crate) fn provide_shell_session() -> RwSignal<Option<ShellAccountSummary>> {
    RwSignal::new(None)
}

#[cfg(feature = "hydrate")]
/// Loads the load shell account data.
pub(crate) fn load_shell_account(account: RwSignal<Option<ShellAccountSummary>>) {
    spawn_local(async move {
        let session = auth::fetch_session().await;
        account.set(session.and_then(|session| {
            if !session.authenticated {
                return None;
            }
            session.account.map(Into::into)
        }));
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(dead_code)]
/// Loads the load shell account data.
pub(crate) fn load_shell_account(account: RwSignal<Option<ShellAccountSummary>>) {
    let _ = account;
}

#[cfg(feature = "hydrate")]
/// Submits the submit logout request.
pub(crate) fn submit_logout() {
    spawn_local(async move {
        auth::api::submit_logout().await;

        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/login");
        }
    });
}

#[cfg(not(feature = "hydrate"))]
/// Submits the submit logout request.
pub(crate) fn submit_logout() {}
