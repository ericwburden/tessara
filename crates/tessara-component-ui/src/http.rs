//! Components-specific adaptation of the shared browser JSON transport.

#[cfg(feature = "hydrate")]
use serde::de::DeserializeOwned;

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_json_request<T>(url: &str, action: &str) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
{
    tessara_web_http::fetch_json(url, action)
        .await
        .map(Some)
        .map_err(tessara_web_http::RequestError::into_message)
}

#[cfg(feature = "hydrate")]
pub(crate) async fn send_json_request<T>(
    builder: gloo_net::http::RequestBuilder,
    body: String,
    action: &str,
) -> Result<T, String>
where
    T: DeserializeOwned,
{
    tessara_web_http::send_json_text(builder, Some(body), action)
        .await
        .map_err(tessara_web_http::RequestError::into_message)
}
