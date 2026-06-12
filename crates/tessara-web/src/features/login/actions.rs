//! Signal-aware login actions.

#[cfg(feature = "hydrate")]
use super::api::{LoginApiError, submit_login_request};
use leptos::prelude::*;

pub(super) fn submit_login(
    email: String,
    password: String,
    error_message: RwSignal<Option<String>>,
    is_submitting: RwSignal<bool>,
) {
    error_message.set(None);
    is_submitting.set(true);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match submit_login_request(&email, &password).await {
                Ok(()) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Err(LoginApiError::InvalidCredentials) => {
                    error_message.set(Some("Email or password did not match.".into()));
                    is_submitting.set(false);
                }
                Err(LoginApiError::Unreachable) => {
                    error_message.set(Some("Could not reach Tessara. Try again.".into()));
                    is_submitting.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (email, password, error_message, is_submitting);
    }
}
