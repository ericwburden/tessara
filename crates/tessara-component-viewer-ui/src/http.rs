//! Component viewer adaptation of the shared browser JSON transport.

#[cfg(feature = "hydrate")]
use serde::de::DeserializeOwned;

#[cfg(feature = "hydrate")]
pub(crate) use tessara_web_http::RequestError as ComponentRequestError;

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_json_request<T>(
    url: &str,
    action: &str,
) -> Result<Option<T>, ComponentRequestError>
where
    T: DeserializeOwned,
{
    tessara_web_http::fetch_json(url, action).await.map(Some)
}
