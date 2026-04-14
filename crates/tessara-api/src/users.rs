use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{
    auth::{self, DelegationSummary, ScopeNodeSummary, UiAccessProfile},
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::require_text,
};

#[derive(Serialize)]
pub struct RoleSummary {
    pub id: Uuid,
    pub name: String,
    pub capability_count: i64,
    pub account_count: i64,
}

#[derive(Serialize)]
pub struct CapabilitySummary {
    pub id: Uuid,
    pub key: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct AccountAssignmentSummary {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
}

#[derive(Serialize)]
pub struct RoleDetail {
    pub id: Uuid,
    pub name: String,
    pub capabilities: Vec<CapabilitySummary>,
    pub assigned_accounts: Vec<AccountAssignmentSummary>,
}

#[derive(Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub roles: Vec<RoleSummary>,
}

#[derive(Serialize)]
pub struct UserDetail {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_active: bool,
    pub ui_access_profile: UiAccessProfile,
    pub capabilities: Vec<String>,
    pub roles: Vec<RoleSummary>,
    pub scope_nodes: Vec<ScopeNodeSummary>,
    pub delegations: Vec<DelegationSummary>,
    pub delegated_by: Vec<DelegationSummary>,
}

#[derive(Serialize)]
pub struct UserAccessDetail {
    pub account_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub ui_access_profile: UiAccessProfile,
    pub capabilities: Vec<String>,
    pub scope_nodes: Vec<ScopeNodeSummary>,
    pub available_scope_nodes: Vec<ScopeNodeSummary>,
    pub delegations: Vec<DelegationSummary>,
    pub available_delegate_accounts: Vec<DelegationSummary>,
    pub scope_assignments_editable: bool,
    pub delegation_assignments_editable: bool,
}

#[derive(Serialize)]
pub struct IdResponse {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub display_name: String,
    pub password: String,
    pub is_active: bool,
    pub role_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub email: String,
    pub display_name: String,
    pub password: Option<String>,
    pub is_active: bool,
    pub role_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub capability_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub capability_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateUserAccessRequest {
    pub scope_node_ids: Vec<Uuid>,
    pub delegate_account_ids: Vec<Uuid>,
}

pub async fn list_capabilities(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<CapabilitySummary>>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;

    Ok(Json(load_capabilities(&state.pool).await?))
}

pub async fn list_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<RoleSummary>>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;

    let roles = sqlx::query(
        r#"
        SELECT
            roles.id,
            roles.name,
            COUNT(DISTINCT role_capabilities.capability_id) AS capability_count,
            COUNT(DISTINCT account_role_assignments.account_id) AS account_count
        FROM roles
        LEFT JOIN role_capabilities ON role_capabilities.role_id = roles.id
        LEFT JOIN account_role_assignments ON account_role_assignments.role_id = roles.id
        GROUP BY roles.id, roles.name
        ORDER BY name, id
        "#,
    )
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(RoleSummary {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            capability_count: row.try_get("capability_count")?,
            account_count: row.try_get("account_count")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(roles))
}

pub async fn create_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateRoleRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    require_text("role name", &payload.name)?;
    ensure_capability_ids_exist(&state.pool, &payload.capability_ids).await?;

    let mut tx = state.pool.begin().await?;
    ensure_role_name_unique(&mut tx, &payload.name, None).await?;

    let role_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO roles (name)
        VALUES ($1)
        RETURNING id
        "#,
    )
    .bind(payload.name.trim())
    .fetch_one(&mut *tx)
    .await?;

    replace_role_capabilities(&mut tx, role_id, &payload.capability_ids).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: role_id }))
}

pub async fn get_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(role_id): Path<Uuid>,
) -> ApiResult<Json<RoleDetail>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    Ok(Json(load_role_detail(&state.pool, role_id).await?))
}

pub async fn update_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(role_id): Path<Uuid>,
    Json(payload): Json<UpdateRoleRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    ensure_capability_ids_exist(&state.pool, &payload.capability_ids).await?;

    let role_name: String = sqlx::query_scalar("SELECT name FROM roles WHERE id = $1")
        .bind(role_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("role {role_id}")))?;

    let role_currently_grants_admin = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM role_capabilities
            JOIN capabilities ON capabilities.id = role_capabilities.capability_id
            WHERE role_capabilities.role_id = $1
              AND capabilities.key = 'admin:all'
        )
        "#,
    )
    .bind(role_id)
    .fetch_one(&state.pool)
    .await?;

    if role_currently_grants_admin {
        let has_admin_all = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM capabilities
                WHERE id = ANY($1)
                  AND key = 'admin:all'
            )
            "#,
        )
        .bind(&payload.capability_ids)
        .fetch_one(&state.pool)
        .await?;
        if !has_admin_all {
            return Err(ApiError::BadRequest(format!(
                "role '{}' must keep admin:all while it grants administrative access",
                role_name
            )));
        }
    }

    let mut tx = state.pool.begin().await?;
    replace_role_capabilities(&mut tx, role_id, &payload.capability_ids).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: role_id }))
}

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<UserSummary>>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;

    let rows = sqlx::query(
        r#"
        SELECT id, email, display_name, is_active
        FROM accounts
        ORDER BY display_name, email, id
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    let mut users = Vec::with_capacity(rows.len());
    for row in rows {
        let account_id: Uuid = row.try_get("id")?;
        users.push(UserSummary {
            id: account_id,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
            is_active: row.try_get("is_active")?,
            roles: load_roles_for_account(&state.pool, account_id).await?,
        });
    }

    Ok(Json(users))
}

pub async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(account_id): Path<Uuid>,
) -> ApiResult<Json<UserDetail>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    Ok(Json(load_user_detail(&state.pool, account_id).await?))
}

pub async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateUserRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    validate_user_payload(
        &payload.email,
        &payload.display_name,
        Some(&payload.password),
        &payload.role_ids,
    )?;

    let mut tx = state.pool.begin().await?;
    ensure_email_unique(&mut tx, &payload.email, None).await?;
    ensure_role_ids_exist(&mut tx, &payload.role_ids).await?;

    let account_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO accounts (email, display_name, is_active)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(payload.email.trim())
    .bind(payload.display_name.trim())
    .bind(payload.is_active)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO account_credentials (account_id, password)
        VALUES ($1, $2)
        "#,
    )
    .bind(account_id)
    .bind(payload.password.trim())
    .execute(&mut *tx)
    .await?;

    replace_role_assignments(&mut tx, account_id, &payload.role_ids).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: account_id }))
}

pub async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    require_account_exists(&state.pool, account_id).await?;
    validate_user_payload(
        &payload.email,
        &payload.display_name,
        payload.password.as_deref(),
        &payload.role_ids,
    )?;

    let mut tx = state.pool.begin().await?;
    ensure_email_unique(&mut tx, &payload.email, Some(account_id)).await?;
    ensure_role_ids_exist(&mut tx, &payload.role_ids).await?;

    sqlx::query(
        r#"
        UPDATE accounts
        SET email = $2,
            display_name = $3,
            is_active = $4
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .bind(payload.email.trim())
    .bind(payload.display_name.trim())
    .bind(payload.is_active)
    .execute(&mut *tx)
    .await?;

    if let Some(password) = payload.password.as_deref() {
        let trimmed = password.trim();
        if !trimmed.is_empty() {
            sqlx::query(
                r#"
                INSERT INTO account_credentials (account_id, password)
                VALUES ($1, $2)
                ON CONFLICT (account_id) DO UPDATE SET password = EXCLUDED.password
                "#,
            )
            .bind(account_id)
            .bind(trimmed)
            .execute(&mut *tx)
            .await?;
        }
    }

    replace_role_assignments(&mut tx, account_id, &payload.role_ids).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: account_id }))
}

pub async fn get_user_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(account_id): Path<Uuid>,
) -> ApiResult<Json<UserAccessDetail>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    require_account_exists(&state.pool, account_id).await?;

    let capabilities = auth::load_effective_capabilities(&state.pool, account_id).await?;
    let ui_access_profile = auth::derive_ui_access_profile(&capabilities);
    let row = sqlx::query(
        r#"
        SELECT id, email, display_name
        FROM accounts
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(UserAccessDetail {
        account_id,
        email: row.try_get("email")?,
        display_name: row.try_get("display_name")?,
        ui_access_profile: ui_access_profile.clone(),
        capabilities,
        scope_nodes: auth::load_scope_nodes(&state.pool, account_id).await?,
        available_scope_nodes: load_all_scope_nodes(&state.pool).await?,
        delegations: auth::load_delegations(&state.pool, account_id).await?,
        available_delegate_accounts: load_delegate_accounts(&state.pool, account_id).await?,
        scope_assignments_editable: auth::scope_assignments_are_meaningful(&ui_access_profile),
        delegation_assignments_editable: true,
    }))
}

pub async fn update_user_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<UpdateUserAccessRequest>,
) -> ApiResult<Json<IdResponse>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    require_account_exists(&state.pool, account_id).await?;
    ensure_node_ids_exist(&state.pool, &payload.scope_node_ids).await?;
    ensure_delegate_account_ids_exist(&state.pool, account_id, &payload.delegate_account_ids)
        .await?;

    let capabilities = auth::load_effective_capabilities(&state.pool, account_id).await?;
    let ui_access_profile = auth::derive_ui_access_profile(&capabilities);
    if !auth::scope_assignments_are_meaningful(&ui_access_profile)
        && !payload.scope_node_ids.is_empty()
    {
        return Err(ApiError::BadRequest(
            "scope assignments are only meaningful for operator-style access".into(),
        ));
    }

    let mut tx = state.pool.begin().await?;
    replace_scope_assignments(&mut tx, account_id, &payload.scope_node_ids).await?;
    replace_delegations(&mut tx, account_id, &payload.delegate_account_ids).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: account_id }))
}

async fn load_user_detail(pool: &PgPool, account_id: Uuid) -> ApiResult<UserDetail> {
    let row = sqlx::query(
        r#"
        SELECT id, email, display_name, is_active
        FROM accounts
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("account {account_id}")))?;
    let capabilities = auth::load_effective_capabilities(pool, account_id).await?;
    let ui_access_profile = auth::derive_ui_access_profile(&capabilities);

    Ok(UserDetail {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
        display_name: row.try_get("display_name")?,
        is_active: row.try_get("is_active")?,
        ui_access_profile,
        capabilities,
        roles: load_roles_for_account(pool, account_id).await?,
        scope_nodes: auth::load_scope_nodes(pool, account_id).await?,
        delegations: auth::load_delegations(pool, account_id).await?,
        delegated_by: load_delegated_by(pool, account_id).await?,
    })
}

async fn load_roles_for_account(pool: &PgPool, account_id: Uuid) -> ApiResult<Vec<RoleSummary>> {
    Ok(sqlx::query(
        r#"
        SELECT
            roles.id,
            roles.name,
            COUNT(DISTINCT role_capabilities.capability_id) AS capability_count,
            COUNT(DISTINCT all_assignments.account_id) AS account_count
        FROM account_role_assignments
        JOIN roles ON roles.id = account_role_assignments.role_id
        LEFT JOIN role_capabilities ON role_capabilities.role_id = roles.id
        LEFT JOIN account_role_assignments AS all_assignments ON all_assignments.role_id = roles.id
        WHERE account_role_assignments.account_id = $1
        GROUP BY roles.id, roles.name
        ORDER BY roles.name, roles.id
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(RoleSummary {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            capability_count: row.try_get("capability_count")?,
            account_count: row.try_get("account_count")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

async fn load_capabilities(pool: &PgPool) -> ApiResult<Vec<CapabilitySummary>> {
    Ok(sqlx::query(
        r#"
        SELECT id, key, description
        FROM capabilities
        ORDER BY key, id
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        Ok(CapabilitySummary {
            id: row.try_get("id")?,
            key: row.try_get("key")?,
            description: row.try_get("description")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

async fn load_role_detail(pool: &PgPool, role_id: Uuid) -> ApiResult<RoleDetail> {
    let row = sqlx::query("SELECT id, name FROM roles WHERE id = $1")
        .bind(role_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("role {role_id}")))?;

    let capabilities = sqlx::query(
        r#"
        SELECT capabilities.id, capabilities.key, capabilities.description
        FROM role_capabilities
        JOIN capabilities ON capabilities.id = role_capabilities.capability_id
        WHERE role_capabilities.role_id = $1
        ORDER BY capabilities.key, capabilities.id
        "#,
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|capability_row| {
        Ok(CapabilitySummary {
            id: capability_row.try_get("id")?,
            key: capability_row.try_get("key")?,
            description: capability_row.try_get("description")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    let assigned_accounts = sqlx::query(
        r#"
        SELECT accounts.id AS account_id, accounts.email, accounts.display_name
        FROM account_role_assignments
        JOIN accounts ON accounts.id = account_role_assignments.account_id
        WHERE account_role_assignments.role_id = $1
        ORDER BY accounts.display_name, accounts.email
        "#,
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|account_row| {
        Ok(AccountAssignmentSummary {
            account_id: account_row.try_get("account_id")?,
            email: account_row.try_get("email")?,
            display_name: account_row.try_get("display_name")?,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(RoleDetail {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        capabilities,
        assigned_accounts,
    })
}

async fn load_all_scope_nodes(pool: &PgPool) -> ApiResult<Vec<ScopeNodeSummary>> {
    Ok(sqlx::query(
        r#"
        SELECT
            nodes.id AS node_id,
            nodes.name AS node_name,
            node_types.name AS node_type_name,
            nodes.parent_node_id,
            parent_nodes.name AS parent_node_name
        FROM nodes
        JOIN node_types ON node_types.id = nodes.node_type_id
        LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
        ORDER BY nodes.name, nodes.id
        "#,
    )
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

async fn load_delegate_accounts(
    pool: &PgPool,
    account_id: Uuid,
) -> ApiResult<Vec<DelegationSummary>> {
    Ok(sqlx::query(
        r#"
        SELECT id AS account_id, email, display_name
        FROM accounts
        WHERE id <> $1
        ORDER BY display_name, email, id
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

async fn load_delegated_by(pool: &PgPool, account_id: Uuid) -> ApiResult<Vec<DelegationSummary>> {
    Ok(sqlx::query(
        r#"
        SELECT
            accounts.id AS account_id,
            accounts.email,
            accounts.display_name
        FROM account_delegations
        JOIN accounts ON accounts.id = account_delegations.delegator_account_id
        WHERE account_delegations.delegate_account_id = $1
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

fn validate_user_payload(
    email: &str,
    display_name: &str,
    password: Option<&str>,
    role_ids: &[Uuid],
) -> ApiResult<()> {
    require_text("account email", email)?;
    require_text("display name", display_name)?;
    if let Some(password) = password {
        require_text("password", password)?;
    }
    if role_ids.is_empty() {
        return Err(ApiError::BadRequest(
            "at least one role must be selected".into(),
        ));
    }
    Ok(())
}

async fn ensure_email_unique(
    tx: &mut Transaction<'_, Postgres>,
    email: &str,
    exclude_account_id: Option<Uuid>,
) -> ApiResult<()> {
    let existing = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM accounts
            WHERE email = $1
              AND ($2::uuid IS NULL OR id <> $2)
        )
        "#,
    )
    .bind(email.trim())
    .bind(exclude_account_id)
    .fetch_one(&mut **tx)
    .await?;

    if existing {
        Err(ApiError::BadRequest(format!(
            "account email '{}' is already in use",
            email.trim()
        )))
    } else {
        Ok(())
    }
}

async fn ensure_role_name_unique(
    tx: &mut Transaction<'_, Postgres>,
    role_name: &str,
    exclude_role_id: Option<Uuid>,
) -> ApiResult<()> {
    let existing = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM roles
            WHERE name = $1
              AND ($2::uuid IS NULL OR id <> $2)
        )
        "#,
    )
    .bind(role_name.trim())
    .bind(exclude_role_id)
    .fetch_one(&mut **tx)
    .await?;

    if existing {
        Err(ApiError::BadRequest(format!(
            "role '{}' already exists",
            role_name.trim()
        )))
    } else {
        Ok(())
    }
}

async fn ensure_role_ids_exist(
    tx: &mut Transaction<'_, Postgres>,
    role_ids: &[Uuid],
) -> ApiResult<()> {
    for role_id in role_ids {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM roles WHERE id = $1)")
                .bind(role_id)
                .fetch_one(&mut **tx)
                .await?;
        if !exists {
            return Err(ApiError::BadRequest(format!("unknown role {role_id}")));
        }
    }
    Ok(())
}

async fn ensure_capability_ids_exist(pool: &PgPool, capability_ids: &[Uuid]) -> ApiResult<()> {
    for capability_id in capability_ids {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM capabilities WHERE id = $1)",
        )
        .bind(capability_id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(ApiError::BadRequest(format!(
                "unknown capability {capability_id}"
            )));
        }
    }
    Ok(())
}

async fn ensure_node_ids_exist(pool: &PgPool, node_ids: &[Uuid]) -> ApiResult<()> {
    for node_id in node_ids {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1)")
                .bind(node_id)
                .fetch_one(pool)
                .await?;
        if !exists {
            return Err(ApiError::BadRequest(format!("unknown node {node_id}")));
        }
    }
    Ok(())
}

async fn ensure_delegate_account_ids_exist(
    pool: &PgPool,
    account_id: Uuid,
    delegate_account_ids: &[Uuid],
) -> ApiResult<()> {
    for delegate_account_id in delegate_account_ids {
        if *delegate_account_id == account_id {
            return Err(ApiError::BadRequest(
                "an account cannot delegate to itself".into(),
            ));
        }

        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1)")
                .bind(delegate_account_id)
                .fetch_one(pool)
                .await?;
        if !exists {
            return Err(ApiError::BadRequest(format!(
                "unknown account {delegate_account_id}"
            )));
        }
    }
    Ok(())
}

async fn replace_role_assignments(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    role_ids: &[Uuid],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM account_role_assignments WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut **tx)
        .await?;

    for role_id in role_ids {
        sqlx::query(
            r#"
            INSERT INTO account_role_assignments (account_id, role_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(account_id)
        .bind(role_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn replace_role_capabilities(
    tx: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
    capability_ids: &[Uuid],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM role_capabilities WHERE role_id = $1")
        .bind(role_id)
        .execute(&mut **tx)
        .await?;

    for capability_id in capability_ids {
        sqlx::query(
            r#"
            INSERT INTO role_capabilities (role_id, capability_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(role_id)
        .bind(capability_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn replace_scope_assignments(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    node_ids: &[Uuid],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM account_node_scope_assignments WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut **tx)
        .await?;

    for node_id in node_ids {
        sqlx::query(
            r#"
            INSERT INTO account_node_scope_assignments (account_id, node_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(account_id)
        .bind(node_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn replace_delegations(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    delegate_account_ids: &[Uuid],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM account_delegations WHERE delegator_account_id = $1")
        .bind(account_id)
        .execute(&mut **tx)
        .await?;

    for delegate_account_id in delegate_account_ids {
        sqlx::query(
            r#"
            INSERT INTO account_delegations (delegator_account_id, delegate_account_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(account_id)
        .bind(delegate_account_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn require_account_exists(pool: &PgPool, account_id: Uuid) -> ApiResult<()> {
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1)")
            .bind(account_id)
            .fetch_one(pool)
            .await?;

    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("account {account_id}")))
    }
}
