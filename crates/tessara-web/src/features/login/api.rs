//! Transport calls for login.

#[cfg(feature = "hydrate")]
pub(super) enum LoginApiError {
    InvalidCredentials,
    Unreachable,
}

#[cfg(feature = "hydrate")]
pub(super) async fn submit_login_request(email: &str, password: &str) -> Result<(), LoginApiError> {
    let body = serde_json::json!({
        "email": email,
        "password": password,
    })
    .to_string();

    let response = match gloo_net::http::Request::post("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(body)
    {
        Ok(request) => request.send().await,
        Err(_) => return Err(LoginApiError::Unreachable),
    };

    match response {
        Ok(response) if response.ok() => Ok(()),
        Ok(_) => Err(LoginApiError::InvalidCredentials),
        Err(_) => Err(LoginApiError::Unreachable),
    }
}
