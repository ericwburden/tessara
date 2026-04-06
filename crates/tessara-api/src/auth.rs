use axum::{Json, extract::State, http::HeaderMap};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{ApiError, ApiResult},
};

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: Uuid,
}

#[derive(Clone, Serialize)]
pub struct AccountContext {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub capabilities: Vec<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    if payload.email != state.config.dev_admin_email
        || payload.password != state.config.dev_admin_password
    {
        return Err(ApiError::Unauthorized);
    }

    let account_id: Uuid = sqlx::query_scalar("SELECT id FROM accounts WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    let token: Uuid =
        sqlx::query_scalar("INSERT INTO auth_sessions (account_id) VALUES ($1) RETURNING token")
            .bind(account_id)
            .fetch_one(&state.pool)
            .await?;

    Ok(Json(LoginResponse { token }))
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<AccountContext>> {
    Ok(Json(
        require_capability(&state.pool, &headers, "admin:all").await?,
    ))
}

pub async fn require_capability(
    pool: &PgPool,
    headers: &HeaderMap,
    required: &str,
) -> ApiResult<AccountContext> {
    let context = account_from_headers(pool, headers).await?;

    if context
        .capabilities
        .iter()
        .any(|cap| cap == "admin:all" || cap == required)
    {
        return Ok(context);
    }

    Err(ApiError::Forbidden(required.to_string()))
}

pub async fn account_from_headers(pool: &PgPool, headers: &HeaderMap) -> ApiResult<AccountContext> {
    let auth = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;
    let token = auth
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized)?
        .parse::<Uuid>()
        .map_err(|_| ApiError::Unauthorized)?;

    let row = sqlx::query(
        r#"
        SELECT accounts.id, accounts.email, accounts.display_name
        FROM auth_sessions
        JOIN accounts ON accounts.id = auth_sessions.account_id
        WHERE auth_sessions.token = $1
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::Unauthorized)?;

    let account_id: Uuid = row.try_get("id")?;
    let capabilities = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT capabilities.key
        FROM account_role_assignments
        JOIN role_capabilities ON role_capabilities.role_id = account_role_assignments.role_id
        JOIN capabilities ON capabilities.id = role_capabilities.capability_id
        WHERE account_role_assignments.account_id = $1
        UNION
        SELECT capabilities.key
        FROM permission_grants
        JOIN capabilities ON capabilities.id = permission_grants.capability_id
        WHERE permission_grants.account_id = $1 AND permission_grants.is_allowed = true
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    Ok(AccountContext {
        account_id,
        email: row.try_get("email")?,
        display_name: row.try_get("display_name")?,
        capabilities,
    })
}
