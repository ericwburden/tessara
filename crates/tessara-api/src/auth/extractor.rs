use std::ops::Deref;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{db::AppState, error::ApiError};

use super::{AccountContext, SessionContext, service};

#[derive(Clone)]
pub struct AuthenticatedRequest {
    pub account: AccountContext,
    pub session: SessionContext,
}

impl AuthenticatedRequest {
    pub fn require_capability(&self, required: &str) -> Result<&AccountContext, ApiError> {
        service::ensure_capability(&self.account, required)?;
        Ok(&self.account)
    }
}

impl Deref for AuthenticatedRequest {
    type Target = AccountContext;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl<S> FromRequestParts<S> for AuthenticatedRequest
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let (account, session) =
            service::authenticate_request(&app_state.pool, &app_state.config, &parts.headers)
                .await?;
        Ok(Self { account, session })
    }
}
