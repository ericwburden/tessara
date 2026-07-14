//! Feature-local browser HTTP helpers.

#[cfg(feature = "hydrate")]
use serde::Deserialize;

#[cfg(feature = "hydrate")]
#[derive(Debug, Deserialize)]
pub(crate) struct IdResponse {
    pub(crate) id: String,
}

#[cfg(feature = "hydrate")]
pub(crate) async fn send_json_request<T, B>(
    builder: gloo_net::http::RequestBuilder,
    body: &B,
    action: &str,
) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
    B: serde::Serialize + ?Sized,
{
    tessara_web_http::send_json(builder, body, action)
        .await
        .map_err(|error| {
            if error.is_authentication() {
                redirect_to_login();
            }
            error.into_message()
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
