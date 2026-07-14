//! Transport calls for login.

#[cfg(feature = "hydrate")]
pub(super) enum LoginApiError {
    InvalidCredentials,
    Unreachable,
}

#[cfg(feature = "hydrate")]
pub(super) async fn submit_login_request(email: &str, password: &str) -> Result<(), LoginApiError> {
    let result = tessara_web_http::send_json_without_response(
        gloo_net::http::Request::post("/api/auth/login"),
        &serde_json::json!({
            "email": email,
            "password": password,
        }),
        "Sign in",
    )
    .await;

    match result {
        Ok(()) => Ok(()),
        Err(error) if error.status().is_some() => Err(LoginApiError::InvalidCredentials),
        Err(_) => Err(LoginApiError::Unreachable),
    }
}
