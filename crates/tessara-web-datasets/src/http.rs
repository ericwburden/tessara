//! Datasets-specific browser transport policy.

#[cfg(feature = "hydrate")]
use serde::de::DeserializeOwned;

#[cfg(feature = "hydrate")]
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
pub(crate) async fn fetch_json_request<T>(url: &str, action: &str) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
{
    match tessara_web_http::fetch_json(url, action).await {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.is_authentication() => {
            redirect_to_login();
            Ok(None)
        }
        Err(error) => Err(error.into_message()),
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn redirect_to_login() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}
