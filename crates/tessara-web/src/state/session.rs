use crate::features::auth::ShellAccountSummary;
#[cfg(feature = "hydrate")]
use leptos::context::{provide_context, use_context};
use leptos::prelude::RwSignal;

#[cfg(feature = "hydrate")]
#[derive(Clone)]
pub(crate) struct ShellSessionState {
    shell_account: RwSignal<Option<ShellAccountSummary>>,
}

#[cfg(feature = "hydrate")]
pub(crate) fn shell_session_account() -> RwSignal<Option<ShellAccountSummary>> {
    use_context::<ShellSessionState>()
        .map(|state| state.shell_account)
        .unwrap_or_else(|| RwSignal::new(None))
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn shell_session_account() -> RwSignal<Option<ShellAccountSummary>> {
    RwSignal::new(None)
}

#[cfg(feature = "hydrate")]
pub(crate) fn provide_shell_session() -> RwSignal<Option<ShellAccountSummary>> {
    let shell_account = RwSignal::new(None::<ShellAccountSummary>);
    crate::features::auth::load_shell_account(shell_account);
    provide_context(ShellSessionState { shell_account });
    shell_account
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn provide_shell_session() -> RwSignal<Option<ShellAccountSummary>> {
    RwSignal::new(None)
}
