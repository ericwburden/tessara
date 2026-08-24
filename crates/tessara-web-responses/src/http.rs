//! Responses-specific adaptation of shared browser transport and route policy.

#[cfg(feature = "hydrate")]
use serde::de::DeserializeOwned;

#[cfg(feature = "hydrate")]
pub(crate) async fn send_json_request<T>(
    builder: gloo_net::http::RequestBuilder,
    body: Option<String>,
    action: &str,
) -> Result<T, tessara_web_http::RequestError>
where
    T: DeserializeOwned,
{
    tessara_web_http::send_json_text(builder, body, action)
        .await
        .inspect_err(|error| {
            if error.is_authentication() {
                redirect_to_login();
            }
        })
}

#[cfg(feature = "hydrate")]
pub(crate) fn redirect_to_login() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn navigate_to_href(href: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(href);
    }
}
