//! Forms-specific adaptation of shared browser transport and route policy.

#[cfg(feature = "hydrate")]
use serde::{Deserialize, de::DeserializeOwned};

#[cfg(feature = "hydrate")]
#[derive(Debug, Deserialize)]
pub(crate) struct IdResponse {
    pub(crate) id: String,
}

#[cfg(feature = "hydrate")]
async fn send_json_request<T>(
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
pub(crate) async fn send_json_id_request(
    builder: gloo_net::http::RequestBuilder,
    body: Option<String>,
    action: &str,
) -> Result<IdResponse, String> {
    send_json_request(builder, body, action).await
}

#[cfg(feature = "hydrate")]
pub(crate) fn redirect_to_login() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}
