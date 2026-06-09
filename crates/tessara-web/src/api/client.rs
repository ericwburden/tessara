//! Browser HTTP transport helpers shared by Leptos feature screens.
//! Kept in `crate::api::client` for a clearer frontend infrastructure boundary.

#[cfg(feature = "hydrate")]
use serde::{Deserialize, de::DeserializeOwned};

#[cfg(feature = "hydrate")]
#[derive(Deserialize)]
struct ApiErrorResponse {
    error: Option<String>,
    message: Option<String>,
}

#[cfg(feature = "hydrate")]
pub(crate) async fn send_json_request<T>(
    builder: gloo_net::http::RequestBuilder,
    body: Option<String>,
    action: &str,
) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let response = if let Some(body) = body {
        builder
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|_| format!("{action} request could not be prepared."))?
            .send()
            .await
    } else {
        builder.send().await
    };

    match response {
        Ok(response) if response.status() == 401 => {
            redirect_to_login();
            Err("Authentication is required.".into())
        }
        Ok(response) if response.ok() => response
            .json::<T>()
            .await
            .map_err(|_| format!("{action} response could not be read.")),
        Ok(response) => {
            let status = response.status();
            if let Ok(body) = response.json::<ApiErrorResponse>().await {
                let message = body.message.or(body.error).unwrap_or_default();
                if !message.trim().is_empty() {
                    return Err(message);
                }
            }
            Err(format!("{action} failed with status {status}."))
        }
        Err(_) => Err(format!("Could not reach the {action} API.")),
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn redirect_to_login() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}
