//! Root-shell adaptation of shared browser transport and navigation policy.

#[cfg(feature = "hydrate")]
use serde::{Deserialize, de::DeserializeOwned};

#[cfg(feature = "hydrate")]
#[derive(Debug, Deserialize)]
pub(crate) struct IdResponse {
    pub(crate) id: String,
}

#[cfg(feature = "hydrate")]
/// Sends a browser JSON request and applies the root shell's auth policy.
pub(crate) async fn send_json_request<T>(
    builder: gloo_net::http::RequestBuilder,
    body: Option<String>,
    action: &str,
) -> Result<T, String>
where
    T: DeserializeOwned,
{
    tessara_web_http::send_json_text(builder, body, action)
        .await
        .map_err(|error| {
            if error.is_authentication() {
                redirect_to_login();
            }
            error.into_message()
        })
}

#[cfg(feature = "hydrate")]
/// Sends a browser JSON request that returns an ID payload.
pub(crate) async fn send_json_id_request(
    builder: gloo_net::http::RequestBuilder,
    body: Option<String>,
    action: &str,
) -> Result<IdResponse, String> {
    send_json_request(builder, body, action).await
}

#[cfg(feature = "hydrate")]
/// Redirects the browser to the login route after an authentication failure.
pub(crate) fn redirect_to_login() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}

#[cfg(feature = "hydrate")]
/// Navigates the browser to the provided application href.
pub(crate) fn navigate_to_href(href: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(href);
    }
}
