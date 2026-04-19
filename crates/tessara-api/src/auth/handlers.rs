use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    http::header,
    response::{IntoResponse, Response},
};

use crate::{db::AppState, error::ApiResult};

use super::{
    AuthenticatedRequest, LoginRequest, LoginResponse, LogoutResponse, SessionStateResponse,
    service,
};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Response> {
    let (token, expires_at) = service::login(
        &state.pool,
        &state.config,
        &payload.email,
        &payload.password,
    )
    .await?;
    let cookie = service::build_session_cookie(
        &state.config,
        token,
        state.config.auth_session_ttl_hours * 60 * 60,
    )?;

    let mut response = Json(LoginResponse { token, expires_at }).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        cookie.parse().expect("session cookie should be valid"),
    );
    Ok(response)
}

pub async fn me(request: AuthenticatedRequest) -> ApiResult<Json<super::AccountContext>> {
    Ok(Json(request.account))
}

pub async fn session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<SessionStateResponse>> {
    let response = match service::authenticate_request(&state.pool, &state.config, &headers).await {
        Ok((account, _session)) => SessionStateResponse {
            authenticated: true,
            account: Some(account),
        },
        Err(crate::error::ApiError::Unauthorized)
        | Err(crate::error::ApiError::SessionExpired)
        | Err(crate::error::ApiError::SessionRevoked) => SessionStateResponse {
            authenticated: false,
            account: None,
        },
        Err(error) => return Err(error),
    };

    Ok(Json(response))
}

pub async fn logout(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Response> {
    let signed_out = service::logout(&state.pool, &request.session).await?;
    let mut response = Json(LogoutResponse { signed_out }).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        service::clear_session_cookie(&state.config)
            .parse()
            .expect("clear session cookie should be valid"),
    );
    Ok(response)
}
