//! Shell session context management.
//!
//! This module owns loading, providing, reading, and clearing the current account summary used by the application shell.

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use crate::features::auth;
use crate::features::auth::ShellAccountSummary;
use crate::state::shell_navigation::{ShellNavigationLoadState, ShellNavigationResponseV1};
use leptos::context::{provide_context, use_context};
use leptos::prelude::RwSignal;
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use leptos::prelude::Set;
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use leptos::task::spawn_local;

#[derive(Clone)]
pub(crate) struct ShellSessionState {
    shell_account: RwSignal<Option<ShellAccountSummary>>,
    shell_navigation: RwSignal<ShellNavigationLoadState>,
}

pub(crate) fn shell_session_account() -> RwSignal<Option<ShellAccountSummary>> {
    use_context::<ShellSessionState>()
        .map(|state| state.shell_account)
        .unwrap_or_else(|| RwSignal::new(None))
}

pub(crate) fn shell_navigation_state() -> RwSignal<ShellNavigationLoadState> {
    use_context::<ShellSessionState>()
        .map(|state| state.shell_navigation)
        .unwrap_or_else(|| RwSignal::new(ShellNavigationLoadState::Loading))
}

pub(crate) fn provide_shell_session(
    initial_navigation: Option<ShellNavigationResponseV1>,
) -> RwSignal<Option<ShellAccountSummary>> {
    let shell_account = RwSignal::new(None::<ShellAccountSummary>);
    let shell_navigation = RwSignal::new(
        initial_navigation
            .filter(ShellNavigationResponseV1::is_supported)
            .map_or(
                ShellNavigationLoadState::Loading,
                ShellNavigationLoadState::Ready,
            ),
    );
    load_shell_state(shell_account, shell_navigation);
    provide_context(ShellSessionState {
        shell_account,
        shell_navigation,
    });
    shell_account
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
/// Loads the authenticated shell account, then its actor-filtered navigation.
pub(crate) fn load_shell_state(
    account: RwSignal<Option<ShellAccountSummary>>,
    navigation: RwSignal<ShellNavigationLoadState>,
) {
    spawn_local(async move {
        let session = auth::fetch_session().await;
        let Some(session) = session else {
            account.set(None);
            navigation.set(ShellNavigationLoadState::Failed);
            return;
        };
        if !session.authenticated {
            account.set(None);
            navigation.set(ShellNavigationLoadState::Loading);
            return;
        }
        let Some(summary) = session.account.map(Into::into) else {
            account.set(None);
            navigation.set(ShellNavigationLoadState::Failed);
            return;
        };
        account.set(Some(summary));
        let projection = tessara_web_http::fetch_json::<ShellNavigationResponseV1>(
            "/api/shell/navigation",
            "Shell navigation",
        )
        .await;
        navigation.set(match projection {
            Ok(projection) if projection.is_supported() => {
                ShellNavigationLoadState::Ready(projection)
            }
            Ok(_) | Err(_) => ShellNavigationLoadState::Failed,
        });
    });
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
#[allow(dead_code)]
pub(crate) fn load_shell_state(
    account: RwSignal<Option<ShellAccountSummary>>,
    navigation: RwSignal<ShellNavigationLoadState>,
) {
    let _ = account;
    let _ = navigation;
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
/// Submits the submit logout request.
pub(crate) fn submit_logout() {
    spawn_local(async move {
        auth::api::submit_logout().await;

        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/login");
        }
    });
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
/// Submits the submit logout request.
pub(crate) fn submit_logout() {}
