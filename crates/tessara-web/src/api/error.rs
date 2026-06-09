//! Shared API error surface for frontend request/response failures.

#[derive(Debug, Clone)]
pub(crate) enum ApiError {
    /// Request preparation or execution failed before parsing a response body.
    Network(String),
    /// A response body could not be parsed into the expected DTO type.
    Parse(String),
    /// Request completed with an HTTP failure status.
    Http(u16, String),
    /// Operation not available under the current feature profile.
    Unavailable(String),
}

impl ApiError {
    pub(crate) fn kind_and_message(&self) -> (&'static str, &str) {
        match self {
            ApiError::Network(message) => ("network", message),
            ApiError::Parse(message) => ("parse", message),
            ApiError::Http(_, message) => ("http", message),
            ApiError::Unavailable(message) => ("unavailable", message),
        }
    }

    pub(crate) fn message(&self) -> &str {
        self.kind_and_message().1
    }
}

impl core::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ApiError::Network(message) => write!(formatter, "Network error: {message}"),
            ApiError::Parse(message) => write!(formatter, "Parse error: {message}"),
            ApiError::Http(status, message) => {
                write!(formatter, "HTTP {status} error: {message}")
            }
            ApiError::Unavailable(message) => write!(formatter, "Unavailable: {message}"),
        }
    }
}

impl std::error::Error for ApiError {}
