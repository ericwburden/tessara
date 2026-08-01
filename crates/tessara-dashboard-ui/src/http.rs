//! Dashboard-specific adaptation of the shared browser JSON transport.

#[cfg(feature = "hydrate")]
use serde::de::DeserializeOwned;

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_json<T>(url: &str, action: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    tessara_web_http::fetch_json(url, action)
        .await
        .map_err(tessara_web_http::RequestError::into_message)
}

#[cfg(feature = "hydrate")]
pub(crate) async fn send_json<T>(
    builder: gloo_net::http::RequestBuilder,
    body: &impl serde::Serialize,
    action: &str,
) -> Result<T, String>
where
    T: DeserializeOwned,
{
    tessara_web_http::send_json(builder, body, action)
        .await
        .map_err(tessara_web_http::RequestError::into_message)
}

#[cfg(feature = "hydrate")]
pub(crate) async fn send_without_response(
    builder: gloo_net::http::RequestBuilder,
    action: &str,
) -> Result<(), String> {
    tessara_web_http::send_without_response(builder, action)
        .await
        .map_err(tessara_web_http::RequestError::into_message)
}
