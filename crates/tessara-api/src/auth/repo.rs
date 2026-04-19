use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::ApiResult;

use super::dto::{DelegationSummary, ScopeNodeSummary};

#[derive(Clone)]
pub struct AccountCredentialRow {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub password_hash: Option<String>,
    pub password_scheme: Option<String>,
    pub legacy_password: Option<String>,
}

#[derive(Clone)]
pub struct SessionAccountRow {
    pub token: Uuid,
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn find_account_credential_by_email(
    pool: &PgPool,
    email: &str,
) -> ApiResult<Option<AccountCredentialRow>> {
    let row = sqlx::query(
        r#"
        SELECT
            accounts.id,
            accounts.email,
            accounts.display_name,
            accounts.is_active,
            account_credentials.password_hash,
            account_credentials.password_scheme,
            account_credentials.legacy_password
        FROM accounts
        LEFT JOIN account_credentials ON account_credentials.account_id = accounts.id
        WHERE accounts.email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok(AccountCredentialRow {
            account_id: row.try_get("id")?,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
            is_active: row.try_get("is_active")?,
            password_hash: row.try_get("password_hash")?,
            password_scheme: row.try_get("password_scheme")?,
            legacy_password: row.try_get("legacy_password")?,
        })
    })
    .transpose()
}

pub async fn list_legacy_credentials(pool: &PgPool) -> ApiResult<Vec<(Uuid, String)>> {
    Ok(sqlx::query(
        r#"
        SELECT account_id, legacy_password
        FROM account_credentials
        WHERE password_hash IS NULL
          AND legacy_password IS NOT NULL
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| Ok((row.try_get("account_id")?, row.try_get("legacy_password")?)))
    .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

pub async fn upsert_password_hash(
    pool: &PgPool,
    account_id: Uuid,
    password_hash: &str,
    password_scheme: &str,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        INSERT INTO account_credentials (account_id, legacy_password, password_hash, password_scheme, password_updated_at)
        VALUES ($1, NULL, $2, $3, now())
        ON CONFLICT (account_id) DO UPDATE SET
            legacy_password = NULL,
            password_hash = EXCLUDED.password_hash,
            password_scheme = EXCLUDED.password_scheme,
            password_updated_at = now()
        "#,
    )
    .bind(account_id)
    .bind(password_hash)
    .bind(password_scheme)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn create_session(
    pool: &PgPool,
    account_id: Uuid,
    expires_at: DateTime<Utc>,
) -> ApiResult<Uuid> {
    sqlx::query_scalar(
        r#"
        INSERT INTO auth_sessions (account_id, expires_at, last_seen_at)
        VALUES ($1, $2, now())
        RETURNING token
        "#,
    )
    .bind(account_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn find_session_account_by_token(
    pool: &PgPool,
    token: Uuid,
) -> ApiResult<Option<SessionAccountRow>> {
    let row = sqlx::query(
        r#"
        SELECT
            auth_sessions.token,
            auth_sessions.account_id,
            auth_sessions.expires_at,
            auth_sessions.last_seen_at,
            auth_sessions.revoked_at,
            accounts.email,
            accounts.display_name,
            accounts.is_active
        FROM auth_sessions
        JOIN accounts ON accounts.id = auth_sessions.account_id
        WHERE auth_sessions.token = $1
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok(SessionAccountRow {
            token: row.try_get("token")?,
            account_id: row.try_get("account_id")?,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
            is_active: row.try_get("is_active")?,
            expires_at: row.try_get("expires_at")?,
            last_seen_at: row.try_get("last_seen_at")?,
            revoked_at: row.try_get("revoked_at")?,
        })
    })
    .transpose()
}

pub async fn touch_session(pool: &PgPool, token: Uuid, seen_at: DateTime<Utc>) -> ApiResult<()> {
    sqlx::query("UPDATE auth_sessions SET last_seen_at = $2 WHERE token = $1")
        .bind(token)
        .bind(seen_at)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn revoke_session(pool: &PgPool, token: Uuid, revoked_at: DateTime<Utc>) -> ApiResult<bool> {
    let result = sqlx::query(
        r#"
        UPDATE auth_sessions
        SET revoked_at = COALESCE(revoked_at, $2)
        WHERE token = $1
        "#,
    )
    .bind(token)
    .bind(revoked_at)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn load_effective_capabilities(
    pool: &PgPool,
    account_id: Uuid,
) -> ApiResult<Vec<String>> {
    Ok(sqlx::query_scalar::<_, String>(
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
    .await?)
}

pub async fn load_scope_nodes(pool: &PgPool, account_id: Uuid) -> ApiResult<Vec<ScopeNodeSummary>> {
    Ok(sqlx::query(
        r#"
        SELECT
            nodes.id AS node_id,
            nodes.name AS node_name,
            node_types.name AS node_type_name,
            nodes.parent_node_id,
            parent_nodes.name AS parent_node_name
        FROM account_node_scope_assignments
        JOIN nodes ON nodes.id = account_node_scope_assignments.node_id
        JOIN node_types ON node_types.id = nodes.node_type_id
        LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
        WHERE account_node_scope_assignments.account_id = $1
        ORDER BY nodes.name, nodes.id
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(ScopeNodeSummary {
            node_id: row.try_get("node_id")?,
            node_name: row.try_get("node_name")?,
            node_type_name: row.try_get("node_type_name")?,
            parent_node_id: row.try_get("parent_node_id")?,
            parent_node_name: row.try_get("parent_node_name")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

pub async fn load_role_names(pool: &PgPool, account_id: Uuid) -> ApiResult<Vec<String>> {
    Ok(sqlx::query_scalar(
        r#"
        SELECT roles.name
        FROM account_role_assignments
        JOIN roles ON roles.id = account_role_assignments.role_id
        WHERE account_role_assignments.account_id = $1
        ORDER BY roles.name, roles.id
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?)
}

pub async fn load_delegations(
    pool: &PgPool,
    account_id: Uuid,
) -> ApiResult<Vec<DelegationSummary>> {
    Ok(sqlx::query(
        r#"
        SELECT
            accounts.id AS account_id,
            accounts.email,
            accounts.display_name
        FROM account_delegations
        JOIN accounts ON accounts.id = account_delegations.delegate_account_id
        WHERE account_delegations.delegator_account_id = $1
        ORDER BY accounts.display_name, accounts.email
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(DelegationSummary {
            account_id: row.try_get("account_id")?,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

pub async fn effective_scope_node_ids(pool: &PgPool, account_id: Uuid) -> ApiResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar(
        r#"
        WITH RECURSIVE scoped(node_id) AS (
            SELECT node_id
            FROM account_node_scope_assignments
            WHERE account_id = $1
            UNION
            SELECT nodes.id
            FROM nodes
            JOIN scoped ON nodes.parent_node_id = scoped.node_id
        )
        SELECT DISTINCT node_id
        FROM scoped
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?)
}
