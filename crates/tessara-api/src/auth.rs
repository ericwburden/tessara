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

#[derive(Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoleFamily {
    Admin,
    Operator,
    Respondent,
}

#[derive(Clone, Serialize)]
pub struct ScopeNodeSummary {
    pub node_id: Uuid,
    pub node_name: String,
    pub node_type_name: String,
    pub parent_node_id: Option<Uuid>,
    pub parent_node_name: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct RespondentSummary {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
}

#[derive(Clone, Serialize)]
pub struct AccountContext {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role_family: RoleFamily,
    pub capabilities: Vec<String>,
    pub scope_nodes: Vec<ScopeNodeSummary>,
    pub subordinate_respondents: Vec<RespondentSummary>,
}

impl AccountContext {
    pub fn is_admin(&self) -> bool {
        self.role_family == RoleFamily::Admin
    }

    pub fn is_operator(&self) -> bool {
        self.role_family == RoleFamily::Operator
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let account_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT accounts.id
        FROM accounts
        JOIN account_credentials ON account_credentials.account_id = accounts.id
        WHERE accounts.email = $1 AND account_credentials.password = $2
        "#,
    )
    .bind(&payload.email)
    .bind(&payload.password)
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
    Ok(Json(require_authenticated(&state.pool, &headers).await?))
}

pub async fn require_authenticated(
    pool: &PgPool,
    headers: &HeaderMap,
) -> ApiResult<AccountContext> {
    account_from_headers(pool, headers).await
}

pub async fn require_capability(
    pool: &PgPool,
    headers: &HeaderMap,
    required: &str,
) -> ApiResult<AccountContext> {
    let context = require_authenticated(pool, headers).await?;

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

    let role_names = sqlx::query_scalar::<_, String>(
        r#"
        SELECT roles.name
        FROM account_role_assignments
        JOIN roles ON roles.id = account_role_assignments.role_id
        WHERE account_role_assignments.account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    let role_family = if role_names.iter().any(|role| role == "admin") {
        RoleFamily::Admin
    } else if role_names.iter().any(|role| role == "operator") {
        RoleFamily::Operator
    } else {
        RoleFamily::Respondent
    };

    let scope_nodes = sqlx::query(
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
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let subordinate_respondents = sqlx::query(
        r#"
        SELECT
            accounts.id AS account_id,
            accounts.email,
            accounts.display_name
        FROM account_subordinate_relationships
        JOIN accounts ON accounts.id = account_subordinate_relationships.respondent_account_id
        WHERE account_subordinate_relationships.parent_account_id = $1
        ORDER BY accounts.display_name, accounts.email
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(RespondentSummary {
            account_id: row.try_get("account_id")?,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(AccountContext {
        account_id,
        email: row.try_get("email")?,
        display_name: row.try_get("display_name")?,
        role_family,
        capabilities,
        scope_nodes,
        subordinate_respondents,
    })
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

pub async fn resolve_accessible_respondent_account_id(
    pool: &PgPool,
    context: &AccountContext,
    requested_account_id: Option<Uuid>,
) -> ApiResult<Uuid> {
    let mut allowed = vec![context.account_id];
    allowed.extend(
        context
            .subordinate_respondents
            .iter()
            .map(|respondent| respondent.account_id),
    );

    let selected = requested_account_id.unwrap_or(context.account_id);
    if allowed.contains(&selected) {
        Ok(selected)
    } else {
        let _ = pool;
        Err(ApiError::Forbidden("responses:respondent-context".into()))
    }
}
