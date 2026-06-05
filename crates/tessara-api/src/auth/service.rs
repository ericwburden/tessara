use std::fmt::Write as _;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use axum::http::{HeaderMap, header};
use chrono::{Duration, Utc};
use rand_core::OsRng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    error::{ApiError, ApiResult},
};

use super::{
    dto::{AccountContext, CapabilityBoundary, CapabilityScope, SessionContext, SessionTransport},
    repo,
};

const PASSWORD_SCHEME: &str = "argon2id-v1";
const DEFAULT_AUTH_COOKIE_NAME: &str = "tessara_session";

pub fn password_scheme() -> &'static str {
    PASSWORD_SCHEME
}

pub async fn store_password_hash(pool: &PgPool, account_id: Uuid, password: &str) -> ApiResult<()> {
    let hash = hash_password_for_storage(password)?;
    repo::upsert_password_hash(pool, account_id, &hash, PASSWORD_SCHEME).await
}

pub async fn login(
    pool: &PgPool,
    config: &Config,
    email: &str,
    password: &str,
) -> ApiResult<(Uuid, chrono::DateTime<Utc>)> {
    let credential = repo::find_account_credential_by_email(pool, email)
        .await?
        .ok_or(ApiError::InvalidCredentials)?;

    if !credential.is_active {
        return Err(ApiError::InvalidCredentials);
    }

    let Some(password_hash) = credential.password_hash.as_deref() else {
        return Err(ApiError::InvalidCredentials);
    };

    verify_password(password_hash, password)?;

    let expires_at = Utc::now() + Duration::hours(config.auth_session_ttl_hours);
    let token = repo::create_session(pool, credential.account_id, expires_at).await?;
    Ok((token, expires_at))
}

pub async fn authenticate_request(
    pool: &PgPool,
    config: &Config,
    headers: &HeaderMap,
) -> ApiResult<(AccountContext, SessionContext)> {
    authenticate_request_with_cookie_name(pool, headers, &config.auth_cookie_name).await
}

pub async fn require_authenticated(
    pool: &PgPool,
    headers: &HeaderMap,
) -> ApiResult<AccountContext> {
    let (account, _) =
        authenticate_request_with_cookie_name(pool, headers, DEFAULT_AUTH_COOKIE_NAME).await?;
    Ok(account)
}

pub async fn require_capability(
    pool: &PgPool,
    headers: &HeaderMap,
    required: &str,
) -> ApiResult<AccountContext> {
    let account = require_authenticated(pool, headers).await?;
    ensure_capability(&account, required)?;
    Ok(account)
}

async fn authenticate_request_with_cookie_name(
    pool: &PgPool,
    headers: &HeaderMap,
    cookie_name: &str,
) -> ApiResult<(AccountContext, SessionContext)> {
    let (token, _transport) = extract_session_token(headers, cookie_name)?;
    let session_row = repo::find_session_account_by_token(pool, token)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    if !session_row.is_active {
        return Err(ApiError::Unauthorized);
    }
    if session_row.revoked_at.is_some() {
        return Err(ApiError::SessionRevoked);
    }
    if session_row.expires_at <= Utc::now() {
        let _ = repo::revoke_session(pool, token, Utc::now()).await;
        return Err(ApiError::SessionExpired);
    }

    let touched_at = Utc::now();
    repo::touch_session(pool, token, touched_at).await?;

    let capability_scopes = repo::load_capability_scopes(pool, session_row.account_id).await?;
    let capabilities = capability_keys(&capability_scopes);
    let account = AccountContext {
        account_id: session_row.account_id,
        email: session_row.email,
        display_name: session_row.display_name,
        is_active: session_row.is_active,
        roles: repo::load_role_names(pool, session_row.account_id).await?,
        capabilities,
        capability_scopes,
        scope_nodes: repo::load_scope_nodes(pool, session_row.account_id).await?,
        delegations: repo::load_delegations(pool, session_row.account_id).await?,
    };

    Ok((account, SessionContext { token }))
}

pub async fn logout(pool: &PgPool, session: &SessionContext) -> ApiResult<bool> {
    repo::revoke_session(pool, session.token, Utc::now()).await
}

pub fn ensure_capability(account: &AccountContext, required: &str) -> ApiResult<()> {
    if account.has_capability(required) {
        Ok(())
    } else {
        Err(ApiError::Forbidden(required.to_string()))
    }
}

pub async fn capability_boundary(
    pool: &PgPool,
    account: &AccountContext,
    required: &str,
) -> ApiResult<CapabilityBoundary> {
    let Some(scope) = account.capability_scope(required) else {
        return Ok(CapabilityBoundary::None);
    };

    if scope.global {
        return Ok(CapabilityBoundary::Global);
    }

    let capability = if scope.capability == "admin:all" {
        "admin:all"
    } else {
        required
    };
    Ok(CapabilityBoundary::Scoped(
        repo::effective_scope_node_ids_for_capability(pool, account.account_id, capability).await?,
    ))
}

pub async fn capability_allows_node(
    pool: &PgPool,
    account: &AccountContext,
    required: &str,
    node_id: Uuid,
) -> ApiResult<bool> {
    Ok(match capability_boundary(pool, account, required).await? {
        CapabilityBoundary::None => false,
        CapabilityBoundary::Global => true,
        CapabilityBoundary::Scoped(scope_ids) => scope_ids.contains(&node_id),
    })
}

fn capability_keys(scopes: &[CapabilityScope]) -> Vec<String> {
    let mut capabilities = scopes
        .iter()
        .map(|scope| scope.capability.clone())
        .collect::<Vec<_>>();
    capabilities.sort();
    capabilities.dedup();
    capabilities
}

pub async fn resolve_accessible_delegate_account_id(
    _pool: &PgPool,
    context: &AccountContext,
    requested_account_id: Option<Uuid>,
) -> ApiResult<Uuid> {
    let mut allowed = vec![context.account_id];
    allowed.extend(
        context
            .delegations
            .iter()
            .map(|delegate| delegate.account_id),
    );

    let selected = requested_account_id.unwrap_or(context.account_id);
    if allowed.contains(&selected) {
        Ok(selected)
    } else {
        Err(ApiError::Forbidden("responses:delegate-context".into()))
    }
}

pub fn build_session_cookie(
    config: &Config,
    token: Uuid,
    max_age_seconds: i64,
) -> ApiResult<String> {
    let mut cookie = String::new();
    write!(
        &mut cookie,
        "{}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_seconds}",
        config.auth_cookie_name
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
    if config.auth_cookie_secure {
        cookie.push_str("; Secure");
    }
    Ok(cookie)
}

pub fn clear_session_cookie(config: &Config) -> String {
    let mut cookie = format!(
        "{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
        config.auth_cookie_name
    );
    if config.auth_cookie_secure {
        cookie.push_str("; Secure");
    }
    cookie
}

pub fn hash_password_for_storage(password: &str) -> ApiResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| ApiError::Internal(anyhow::anyhow!(error.to_string())))
}

fn verify_password(password_hash: &str, password: &str) -> ApiResult<()> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|_| ApiError::InvalidCredentials)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::InvalidCredentials)
}

fn extract_session_token(
    headers: &HeaderMap,
    cookie_name: &str,
) -> ApiResult<(Uuid, SessionTransport)> {
    if let Some(token) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .and_then(|raw| raw.parse::<Uuid>().ok())
    {
        return Ok((token, SessionTransport::Bearer));
    }

    let cookie_header = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;

    for pair in cookie_header.split(';') {
        let mut parts = pair.trim().splitn(2, '=');
        let Some(name) = parts.next() else { continue };
        let Some(value) = parts.next() else { continue };
        if name == cookie_name {
            let token = value.parse::<Uuid>().map_err(|_| ApiError::Unauthorized)?;
            return Ok((token, SessionTransport::Cookie));
        }
    }

    Err(ApiError::Unauthorized)
}
