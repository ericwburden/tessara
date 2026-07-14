//! Policy-neutral browser JSON transport for Tessara web crates.
//!
//! This crate owns request preparation, response decoding, API error-envelope
//! parsing, and failure classification. Feature crates retain endpoint
//! orchestration, navigation, and authentication redirect policy.

use std::fmt;

#[cfg(feature = "hydrate")]
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// The recovery policy associated with a browser request failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestErrorKind {
    /// The session is missing or no longer valid.
    Authentication,
    /// Repeating the request can reasonably succeed without changing it.
    Retryable,
    /// Repeating the same request is not expected to recover the failure.
    Terminal,
}

/// A classified browser request failure with user-safe display text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestError {
    kind: RequestErrorKind,
    message: String,
    status: Option<u16>,
}

impl RequestError {
    #[cfg(any(feature = "hydrate", test))]
    fn new(kind: RequestErrorKind, message: impl Into<String>, status: Option<u16>) -> Self {
        Self {
            kind,
            message: message.into(),
            status,
        }
    }

    #[cfg(any(feature = "hydrate", test))]
    fn authentication() -> Self {
        Self::new(
            RequestErrorKind::Authentication,
            "Authentication is required.",
            Some(401),
        )
    }

    #[cfg(any(feature = "hydrate", test))]
    fn retryable(message: impl Into<String>, status: Option<u16>) -> Self {
        Self::new(RequestErrorKind::Retryable, message, status)
    }

    #[cfg(any(feature = "hydrate", test))]
    fn terminal(message: impl Into<String>, status: Option<u16>) -> Self {
        Self::new(RequestErrorKind::Terminal, message, status)
    }

    /// Returns the failure's recovery classification.
    pub const fn kind(&self) -> RequestErrorKind {
        self.kind
    }

    /// Returns whether the caller should apply its authentication policy.
    pub const fn is_authentication(&self) -> bool {
        matches!(self.kind, RequestErrorKind::Authentication)
    }

    /// Returns whether the same request may be retried after a bounded delay.
    pub const fn is_retryable(&self) -> bool {
        matches!(self.kind, RequestErrorKind::Retryable)
    }

    /// Returns the HTTP status when a response was received.
    pub const fn status(&self) -> Option<u16> {
        self.status
    }

    /// Consumes the error and returns its user-safe display text.
    pub fn into_message(self) -> String {
        self.message
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RequestError {}

#[cfg(feature = "hydrate")]
#[derive(Deserialize)]
struct ApiErrorResponse {
    error: Option<String>,
    message: Option<String>,
}

/// Fetches and decodes one JSON response.
#[cfg(feature = "hydrate")]
pub async fn fetch_json<T>(url: &str, action: &str) -> Result<T, RequestError>
where
    T: DeserializeOwned,
{
    parse_json_result(gloo_net::http::Request::get(url).send().await, action).await
}

/// Serializes a value and sends it as a JSON request.
#[cfg(feature = "hydrate")]
pub async fn send_json<T, B>(
    builder: gloo_net::http::RequestBuilder,
    body: &B,
    action: &str,
) -> Result<T, RequestError>
where
    T: DeserializeOwned,
    B: Serialize + ?Sized,
{
    let body = serde_json::to_string(body).map_err(|_| {
        RequestError::terminal(format!("{action} request could not be prepared."), None)
    })?;
    send_json_text(builder, Some(body), action).await
}

/// Serializes and sends JSON when a successful response body is not needed.
#[cfg(feature = "hydrate")]
pub async fn send_json_without_response<B>(
    builder: gloo_net::http::RequestBuilder,
    body: &B,
    action: &str,
) -> Result<(), RequestError>
where
    B: Serialize + ?Sized,
{
    let body = serde_json::to_string(body).map_err(|_| {
        RequestError::terminal(format!("{action} request could not be prepared."), None)
    })?;
    let request = builder
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|_| {
            RequestError::terminal(format!("{action} request could not be prepared."), None)
        })?;
    parse_empty_result(request.send().await, action).await
}

/// Sends an optional pre-serialized JSON body and decodes the JSON response.
#[cfg(feature = "hydrate")]
pub async fn send_json_text<T>(
    builder: gloo_net::http::RequestBuilder,
    body: Option<String>,
    action: &str,
) -> Result<T, RequestError>
where
    T: DeserializeOwned,
{
    let response = if let Some(body) = body {
        builder
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|_| {
                RequestError::terminal(format!("{action} request could not be prepared."), None)
            })?
            .send()
            .await
    } else {
        builder.send().await
    };

    parse_json_result(response, action).await
}

/// Sends a request whose successful response body is intentionally ignored.
#[cfg(feature = "hydrate")]
pub async fn send_without_response(
    builder: gloo_net::http::RequestBuilder,
    action: &str,
) -> Result<(), RequestError> {
    parse_empty_result(builder.send().await, action).await
}

/// Classifies and decodes an already-started JSON request.
///
/// This keeps parallel endpoint orchestration feature-local while sharing the
/// response contract and error-envelope handling.
#[cfg(feature = "hydrate")]
pub async fn parse_json_result<T>(
    response: Result<gloo_net::http::Response, gloo_net::Error>,
    action: &str,
) -> Result<T, RequestError>
where
    T: DeserializeOwned,
{
    match response {
        Ok(response) => parse_json_response(response, action).await,
        Err(_) => Err(RequestError::retryable(
            format!("Could not reach the {action} API."),
            None,
        )),
    }
}

/// Classifies and decodes an already-received JSON response.
#[cfg(feature = "hydrate")]
pub async fn parse_json_response<T>(
    response: gloo_net::http::Response,
    action: &str,
) -> Result<T, RequestError>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if status == 401 {
        return Err(RequestError::authentication());
    }
    if response.ok() {
        return response.json::<T>().await.map_err(|_| {
            RequestError::terminal(
                format!("{action} response could not be read."),
                Some(status),
            )
        });
    }
    Err(http_error(response, action, status).await)
}

#[cfg(feature = "hydrate")]
async fn parse_empty_result(
    response: Result<gloo_net::http::Response, gloo_net::Error>,
    action: &str,
) -> Result<(), RequestError> {
    match response {
        Ok(response) if response.status() == 401 => Err(RequestError::authentication()),
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => {
            let status = response.status();
            Err(http_error(response, action, status).await)
        }
        Err(_) => Err(RequestError::retryable(
            format!("Could not reach the {action} API."),
            None,
        )),
    }
}

#[cfg(feature = "hydrate")]
async fn http_error(response: gloo_net::http::Response, action: &str, status: u16) -> RequestError {
    let fallback = format!("{action} failed with status {status}.");
    let message = response
        .json::<ApiErrorResponse>()
        .await
        .ok()
        .and_then(|body| body.message.or(body.error))
        .filter(|message| !message.trim().is_empty())
        .unwrap_or(fallback);

    if is_retryable_status(status) {
        RequestError::retryable(message, Some(status))
    } else {
        RequestError::terminal(message, Some(status))
    }
}

#[cfg(any(feature = "hydrate", test))]
const fn is_retryable_status(status: u16) -> bool {
    matches!(status, 408 | 425 | 429) || status >= 500
}

#[cfg(test)]
mod tests {
    use super::{RequestError, RequestErrorKind, is_retryable_status};

    #[test]
    fn error_classification_is_independent_from_display_text() {
        let authentication = RequestError::authentication();
        assert!(authentication.is_authentication());
        assert!(!authentication.is_retryable());
        assert_eq!(authentication.status(), Some(401));

        let retryable = RequestError::retryable("temporarily unavailable", Some(503));
        assert_eq!(retryable.kind(), RequestErrorKind::Retryable);
        assert_eq!(retryable.into_message(), "temporarily unavailable");

        let terminal = RequestError::terminal("invalid request", Some(422));
        assert_eq!(terminal.kind(), RequestErrorKind::Terminal);
    }

    #[test]
    fn only_transient_http_statuses_are_retryable() {
        for status in [408, 425, 429, 500, 503] {
            assert!(is_retryable_status(status), "status {status}");
        }
        for status in [400, 401, 403, 404, 409, 422] {
            assert!(!is_retryable_status(status), "status {status}");
        }
    }
}
