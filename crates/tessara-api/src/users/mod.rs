//! Admin user, role, scope, and delegation management endpoints.
//!
//! This module owns the administrator-facing access-management API. The public
//! DTOs live in `dto`, while the handlers below keep the current route contract
//! and coordinate validation, persistence, and effective-access projections.

use std::collections::{BTreeMap, HashSet};

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

mod dto;

pub use dto::{
    AccountAssignmentSummary, CapabilityProvenanceSourceKind, CapabilityProvenanceSummary,
    CapabilityProviderState, CapabilityScopeMode, CapabilitySummary, CreateRoleRequest,
    CreateUserRequest, IdResponse, RoleDetail, RoleSummary, UpdateRoleRequest,
    UpdateUserAccessRequest, UpdateUserRequest, UserAccessDetail, UserDetail, UserSummary,
};

use crate::{
    auth::{self, AuthenticatedRequest, DelegationSummary, ScopeNodeSummary},
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::require_text,
};

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/admin/capabilities", get(list_capabilities))
        .route("/api/admin/roles", get(list_roles).post(create_role))
        .route("/api/admin/roles/{role_id}", get(get_role).put(update_role))
        .route("/api/admin/users", get(list_users).post(create_user))
        .route(
            "/api/admin/users/{account_id}",
            get(get_user).put(update_user),
        )
        .route(
            "/api/admin/users/{account_id}/access",
            get(get_user_access).put(update_user_access),
        )
}

/// Lists the capability catalog admins use to compose role bundles.
pub async fn list_capabilities(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<Vec<CapabilitySummary>>> {
    request.require_capability("admin:all")?;

    Ok(Json(load_capabilities(&state.pool).await?))
}

/// Lists reusable role bundles and the number of assigned accounts/capabilities.
pub async fn list_roles(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<Vec<RoleSummary>>> {
    request.require_capability("admin:all")?;

    let roles = sqlx::query(
        r#"
        SELECT
            roles.id,
            roles.name,
            COUNT(DISTINCT role_capabilities.capability_id) AS capability_count,
            COUNT(DISTINCT role_assignments.account_id) AS account_count
        FROM roles
        LEFT JOIN role_capabilities ON role_capabilities.role_id = roles.id
        LEFT JOIN role_assignments ON role_assignments.role_id = roles.id
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
            assigned_at: None,
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(Json(roles))
}

/// Creates an administrator-managed role bundle from selected capabilities.
pub async fn create_role(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Json(payload): Json<CreateRoleRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
    require_text("role name", &payload.name)?;

    let mut tx = state.pool.begin().await?;
    ensure_capability_ids_exist(&mut tx, &payload.capability_ids).await?;
    capability_bundle_scope_mode(&mut tx, &payload.capability_ids).await?;
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

/// Loads a role bundle with its capabilities and assigned accounts.
pub async fn get_role(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(role_id): Path<Uuid>,
) -> ApiResult<Json<RoleDetail>> {
    request.require_capability("admin:all")?;
    Ok(Json(load_role_detail(&state.pool, role_id).await?))
}

/// Replaces the capabilities assigned to an existing role bundle.
pub async fn update_role(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(role_id): Path<Uuid>,
    Json(payload): Json<UpdateRoleRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
    let mut tx = state.pool.begin().await?;
    let role_name: String = sqlx::query_scalar("SELECT name FROM roles WHERE id = $1 FOR UPDATE")
        .bind(role_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("role {role_id}")))?;
    ensure_capability_ids_exist(&mut tx, &payload.capability_ids).await?;
    let proposed_scope_mode =
        capability_bundle_scope_mode(&mut tx, &payload.capability_ids).await?;

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
    .fetch_one(&mut *tx)
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
        .fetch_one(&mut *tx)
        .await?;
        if !has_admin_all {
            return Err(ApiError::BadRequest(format!(
                "role '{}' must keep admin:all while it grants administrative access",
                role_name
            )));
        }
    }

    if proposed_scope_mode == RoleScopeMode::InstallationGlobal
        && role_has_scoped_assignments(&mut tx, role_id).await?
    {
        return Err(ApiError::GlobalCapabilityRequiresGlobalRoleAssignment);
    }

    replace_role_capabilities(&mut tx, role_id, &payload.capability_ids).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: role_id }))
}

/// Lists local accounts with their assigned role bundles.
pub async fn list_users(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<Vec<UserSummary>>> {
    request.require_capability("admin:all")?;

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

/// Loads a local account with effective capabilities, scopes, and delegations.
pub async fn get_user(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(account_id): Path<Uuid>,
) -> ApiResult<Json<UserDetail>> {
    request.require_capability("admin:all")?;
    Ok(Json(load_user_detail(&state.pool, account_id).await?))
}

/// Creates a local account, credential, and initial role assignments.
pub async fn create_user(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Json(payload): Json<CreateUserRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
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

    let password_hash = auth::hash_password_for_storage(payload.password.trim())?;
    sqlx::query(
        r#"
        INSERT INTO account_credentials (account_id, password_hash, password_scheme, password_updated_at)
        VALUES ($1, $2, $3, now())
        "#,
    )
    .bind(account_id)
    .bind(password_hash)
    .bind(auth::password_scheme())
    .execute(&mut *tx)
    .await?;

    replace_role_assignments(&mut tx, account_id, &payload.role_ids).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: account_id }))
}

/// Updates local account identity, active state, optional password, and roles.
pub async fn update_user(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
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
            let password_hash = auth::hash_password_for_storage(trimmed)?;
            sqlx::query(
                r#"
                INSERT INTO account_credentials (account_id, password_hash, password_scheme, password_updated_at)
                VALUES ($1, $2, $3, now())
                ON CONFLICT (account_id) DO UPDATE SET
                    password_hash = EXCLUDED.password_hash,
                    password_scheme = EXCLUDED.password_scheme,
                    password_updated_at = now()
                "#,
            )
            .bind(account_id)
            .bind(password_hash)
            .bind(auth::password_scheme())
            .execute(&mut *tx)
            .await?;
        }
    }

    replace_role_assignments(&mut tx, account_id, &payload.role_ids).await?;
    tx.commit().await?;

    Ok(Json(IdResponse { id: account_id }))
}

/// Loads editable scope/delegation assignment state for a local account.
pub async fn get_user_access(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(account_id): Path<Uuid>,
) -> ApiResult<Json<UserAccessDetail>> {
    request.require_capability("admin:all")?;
    require_account_exists(&state.pool, account_id).await?;

    let capabilities = auth::load_effective_capabilities(&state.pool, account_id).await?;
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
        capabilities,
        scope_nodes: auth::load_scope_nodes(&state.pool, account_id).await?,
        available_scope_nodes: load_all_scope_nodes(&state.pool).await?,
        delegations: auth::load_delegations(&state.pool, account_id).await?,
        available_delegate_accounts: load_delegate_accounts(&state.pool, account_id).await?,
        scope_assignments_editable: true,
        delegation_assignments_editable: true,
    }))
}

/// Replaces scope and delegation assignments for a local account.
pub async fn update_user_access(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<UpdateUserAccessRequest>,
) -> ApiResult<Json<IdResponse>> {
    request.require_capability("admin:all")?;
    require_account_exists(&state.pool, account_id).await?;
    ensure_node_ids_exist(&state.pool, &payload.scope_node_ids).await?;
    ensure_delegate_account_ids_exist(&state.pool, account_id, &payload.delegate_account_ids)
        .await?;

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

    Ok(UserDetail {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
        display_name: row.try_get("display_name")?,
        is_active: row.try_get("is_active")?,
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
            COUNT(DISTINCT all_assignments.account_id) AS account_count,
            MIN(role_assignments.created_at) AS assigned_at
        FROM role_assignments
        JOIN roles ON roles.id = role_assignments.role_id
        LEFT JOIN role_capabilities ON role_capabilities.role_id = roles.id
        LEFT JOIN role_assignments AS all_assignments ON all_assignments.role_id = roles.id
        WHERE role_assignments.account_id = $1
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
            assigned_at: Some(row.try_get("assigned_at")?),
        })
    })
    .collect::<Result<Vec<_>, sqlx::Error>>()?)
}

async fn load_capabilities(pool: &PgPool) -> ApiResult<Vec<CapabilitySummary>> {
    let rows = sqlx::query(
        r#"
        SELECT
            capabilities.id,
            capabilities.key,
            capabilities.description,
            capabilities.scope_mode,
            capability_provenance.id AS provenance_id,
            capability_provenance.source_kind,
            capability_provenance.source_key,
            capability_provenance.definition_id,
            capability_provenance.provider_state,
            module_definition_reservations.display_name AS definition_display_name,
            transition_descriptor_sources.source_digest
        FROM capabilities
        LEFT JOIN capability_provenance
            ON capability_provenance.capability_id = capabilities.id
        LEFT JOIN module_definition_reservations
            ON module_definition_reservations.definition_id = capability_provenance.definition_id
        LEFT JOIN transition_descriptor_sources
            ON transition_descriptor_sources.id = capability_provenance.descriptor_source_id
        ORDER BY
            capabilities.key,
            capabilities.id,
            CASE capability_provenance.source_kind
                WHEN 'core' THEN 0
                WHEN 'transition_contribution' THEN 1
                ELSE 2
            END,
            capability_provenance.source_key,
            capability_provenance.id
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut capabilities = Vec::<CapabilitySummary>::new();
    for row in rows {
        let capability_id: Uuid = row.try_get("id")?;
        if capabilities
            .last()
            .is_none_or(|capability| capability.id != capability_id)
        {
            let scope_mode: String = row.try_get("scope_mode")?;
            capabilities.push(CapabilitySummary {
                id: capability_id,
                key: row.try_get("key")?,
                description: row.try_get("description")?,
                scope_mode: parse_capability_scope_mode(&scope_mode)?,
                provenance: Vec::new(),
            });
        }

        let provenance_id: Option<Uuid> = row.try_get("provenance_id")?;
        if provenance_id.is_none() {
            continue;
        }
        let source_kind: String = row.try_get("source_kind")?;
        let provider_state: String = row.try_get("provider_state")?;
        capabilities
            .last_mut()
            .expect("the current capability row has a summary")
            .provenance
            .push(CapabilityProvenanceSummary {
                source_kind: parse_provenance_source_kind(&source_kind)?,
                source_key: row.try_get("source_key")?,
                definition_id: row.try_get("definition_id")?,
                definition_display_name: row.try_get("definition_display_name")?,
                provider_state: parse_capability_provider_state(&provider_state)?,
                source_digest: row.try_get("source_digest")?,
            });
    }
    Ok(capabilities)
}

fn parse_capability_scope_mode(value: &str) -> ApiResult<CapabilityScopeMode> {
    match value {
        "scope_aware" => Ok(CapabilityScopeMode::ScopeAware),
        "installation_global" => Ok(CapabilityScopeMode::InstallationGlobal),
        unexpected => Err(ApiError::Internal(anyhow::anyhow!(
            "stored capability scope mode '{unexpected}' is unsupported"
        ))),
    }
}

fn parse_provenance_source_kind(value: &str) -> ApiResult<CapabilityProvenanceSourceKind> {
    match value {
        "core" => Ok(CapabilityProvenanceSourceKind::Core),
        "transition_contribution" => Ok(CapabilityProvenanceSourceKind::TransitionContribution),
        unexpected => Err(ApiError::Internal(anyhow::anyhow!(
            "stored capability provenance source kind '{unexpected}' is unsupported"
        ))),
    }
}

fn parse_capability_provider_state(value: &str) -> ApiResult<CapabilityProviderState> {
    match value {
        "core_authoritative" => Ok(CapabilityProviderState::CoreAuthoritative),
        "transitional_in_process" => Ok(CapabilityProviderState::TransitionalInProcess),
        unexpected => Err(ApiError::Internal(anyhow::anyhow!(
            "stored capability provider state '{unexpected}' is unsupported"
        ))),
    }
}

async fn load_role_detail(pool: &PgPool, role_id: Uuid) -> ApiResult<RoleDetail> {
    let row = sqlx::query("SELECT id, name FROM roles WHERE id = $1")
        .bind(role_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("role {role_id}")))?;

    let assigned_capability_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT capabilities.id
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
    .collect::<HashSet<_>>();
    let capabilities = load_capabilities(pool)
        .await?
        .into_iter()
        .filter(|capability| assigned_capability_ids.contains(&capability.id))
        .collect();

    let assigned_accounts = sqlx::query(
        r#"
        SELECT accounts.id AS account_id, accounts.email, accounts.display_name
        FROM role_assignments
        JOIN accounts ON accounts.id = role_assignments.account_id
        WHERE role_assignments.role_id = $1
        GROUP BY accounts.id, accounts.email, accounts.display_name
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RoleScopeMode {
    ScopeAware,
    InstallationGlobal,
}

async fn ensure_capability_ids_exist(
    tx: &mut Transaction<'_, Postgres>,
    capability_ids: &[Uuid],
) -> ApiResult<()> {
    for capability_id in capability_ids {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM capabilities WHERE id = $1)",
        )
        .bind(capability_id)
        .fetch_one(&mut **tx)
        .await?;
        if !exists {
            return Err(ApiError::BadRequest(format!(
                "unknown capability {capability_id}"
            )));
        }
    }
    Ok(())
}

async fn capability_bundle_scope_mode(
    tx: &mut Transaction<'_, Postgres>,
    capability_ids: &[Uuid],
) -> ApiResult<RoleScopeMode> {
    let capabilities = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT key, scope_mode
        FROM capabilities
        WHERE id = ANY($1)
        ORDER BY key
        "#,
    )
    .bind(capability_ids)
    .fetch_all(&mut **tx)
    .await?;

    classify_role_capability_scope_modes(
        capabilities
            .iter()
            .map(|(key, scope_mode)| (key.as_str(), scope_mode.as_str())),
    )
}

fn classify_role_capability_scope_modes<'a>(
    capabilities: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> ApiResult<RoleScopeMode> {
    let mut has_scope_aware = false;
    let mut has_installation_global = false;
    let mut has_admin_all = false;
    for (key, scope_mode) in capabilities {
        has_admin_all |= key == "admin:all";
        match scope_mode {
            "scope_aware" => has_scope_aware = true,
            "installation_global" => has_installation_global = true,
            unexpected => {
                return Err(ApiError::Internal(anyhow::anyhow!(
                    "stored capability scope mode '{unexpected}' is unsupported"
                )));
            }
        }
    }

    if has_scope_aware && has_installation_global && !has_admin_all {
        Err(ApiError::MixedCapabilityScopeModes)
    } else if has_installation_global {
        // `admin:all` is the sole mixed-mode exception. Its universal grant is
        // installation-global, so redundant product rows never change the
        // role's assignment classification.
        Ok(RoleScopeMode::InstallationGlobal)
    } else {
        // Empty roles retain the historical scope-aware assignment behavior.
        Ok(RoleScopeMode::ScopeAware)
    }
}

async fn role_scope_mode(
    tx: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
) -> ApiResult<RoleScopeMode> {
    // Hold a lock through assignment rewriting. This serializes against
    // `update_role`'s FOR UPDATE lock, preventing a scoped row from being
    // inserted after that role has been converted to installation-global.
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM roles WHERE id = $1 FOR KEY SHARE")
        .bind(role_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("role {role_id}")))?;

    let capabilities = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT capabilities.key, capabilities.scope_mode
        FROM role_capabilities
        JOIN capabilities ON capabilities.id = role_capabilities.capability_id
        WHERE role_capabilities.role_id = $1
        ORDER BY capabilities.key
        "#,
    )
    .bind(role_id)
    .fetch_all(&mut **tx)
    .await?;

    classify_role_capability_scope_modes(
        capabilities
            .iter()
            .map(|(key, scope_mode)| (key.as_str(), scope_mode.as_str())),
    )
}

async fn role_has_scoped_assignments(
    tx: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
) -> ApiResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM role_assignments
            WHERE role_id = $1
              AND node_id IS NOT NULL
        )
        "#,
    )
    .bind(role_id)
    .fetch_one(&mut **tx)
    .await?)
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
    // Role edits preserve the account's existing scope roots. Each role is
    // classified before mutation so an invalid stored bundle fails atomically.
    let scoped_node_ids = current_scope_node_ids(tx, account_id).await?;
    let existing_assignment_dates = current_role_assignment_dates(tx, account_id).await?;
    let mut ordered_role_ids = role_ids.to_vec();
    ordered_role_ids.sort_unstable();
    ordered_role_ids.dedup();
    let mut roles = Vec::with_capacity(ordered_role_ids.len());
    for role_id in ordered_role_ids {
        roles.push((role_id, role_scope_mode(tx, role_id).await?));
    }

    sqlx::query("DELETE FROM role_assignments WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut **tx)
        .await?;

    for (role_id, scope_mode) in roles {
        insert_role_assignment_rows(
            tx,
            account_id,
            role_id,
            scope_mode,
            &scoped_node_ids,
            existing_assignment_dates.get(&role_id).copied(),
        )
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
    // Scope edits preserve selected roles and rewrite each role assignment to
    // either global or per-node rows according to the capability bundle.
    let role_ids = current_role_ids(tx, account_id).await?;
    let existing_assignment_dates = current_role_assignment_dates(tx, account_id).await?;
    let mut roles = Vec::with_capacity(role_ids.len());
    for role_id in role_ids {
        roles.push((role_id, role_scope_mode(tx, role_id).await?));
    }

    let has_scope_aware_role = roles
        .iter()
        .any(|(_, scope_mode)| *scope_mode == RoleScopeMode::ScopeAware);
    let has_installation_global_role = roles
        .iter()
        .any(|(_, scope_mode)| *scope_mode == RoleScopeMode::InstallationGlobal);
    if !node_ids.is_empty() && has_installation_global_role && !has_scope_aware_role {
        return Err(ApiError::GlobalCapabilityRequiresGlobalRoleAssignment);
    }

    sqlx::query("DELETE FROM role_assignments WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut **tx)
        .await?;

    for (role_id, scope_mode) in roles {
        insert_role_assignment_rows(
            tx,
            account_id,
            role_id,
            scope_mode,
            node_ids,
            existing_assignment_dates.get(&role_id).copied(),
        )
        .await?;
    }

    Ok(())
}

async fn current_role_ids(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> ApiResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar(
        r#"
        SELECT DISTINCT role_id
        FROM role_assignments
        WHERE account_id = $1
        ORDER BY role_id
        "#,
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await?)
}

async fn current_role_assignment_dates(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> ApiResult<BTreeMap<Uuid, DateTime<Utc>>> {
    let rows = sqlx::query(
        r#"
        SELECT role_id, MIN(created_at) AS assigned_at
        FROM role_assignments
        WHERE account_id = $1
        GROUP BY role_id
        ORDER BY role_id
        "#,
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await?;

    rows.into_iter()
        .map(|row| Ok((row.try_get("role_id")?, row.try_get("assigned_at")?)))
        .collect::<Result<BTreeMap<_, _>, sqlx::Error>>()
        .map_err(Into::into)
}

async fn current_scope_node_ids(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
) -> ApiResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar(
        r#"
        SELECT DISTINCT node_id
        FROM role_assignments
        WHERE account_id = $1
          AND node_id IS NOT NULL
          AND NOT EXISTS (
              SELECT 1
              FROM role_capabilities
              JOIN capabilities ON capabilities.id = role_capabilities.capability_id
              WHERE role_capabilities.role_id = role_assignments.role_id
                AND capabilities.scope_mode = 'installation_global'
          )
        ORDER BY node_id
        "#,
    )
    .bind(account_id)
    .fetch_all(&mut **tx)
    .await?)
}

async fn insert_role_assignment_rows(
    tx: &mut Transaction<'_, Postgres>,
    account_id: Uuid,
    role_id: Uuid,
    scope_mode: RoleScopeMode,
    scoped_node_ids: &[Uuid],
    assigned_at: Option<DateTime<Utc>>,
) -> ApiResult<()> {
    // A role assignment with a NULL node is global. Empty scope selections
    // intentionally mean unrestricted product scope. Installation-global roles
    // always use NULL independently of the account's product-scope selections.
    if scoped_node_ids.is_empty() || scope_mode == RoleScopeMode::InstallationGlobal {
        sqlx::query(
            r#"
            INSERT INTO role_assignments (account_id, role_id, node_id, created_at)
            VALUES ($1, $2, NULL, COALESCE($3, now()))
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(account_id)
        .bind(role_id)
        .bind(assigned_at)
        .execute(&mut **tx)
        .await?;
        return Ok(());
    }

    for node_id in scoped_node_ids {
        sqlx::query(
            r#"
            INSERT INTO role_assignments (account_id, role_id, node_id, created_at)
            VALUES ($1, $2, $3, COALESCE($4, now()))
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(account_id)
        .bind(role_id)
        .bind(node_id)
        .bind(assigned_at)
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

#[cfg(test)]
mod tests {
    use super::{
        CapabilityProvenanceSourceKind, CapabilityProviderState, CapabilityScopeMode,
        RoleScopeMode, classify_role_capability_scope_modes, parse_capability_provider_state,
        parse_capability_scope_mode, parse_provenance_source_kind,
    };
    use crate::error::ApiError;

    #[test]
    fn stored_capability_metadata_uses_closed_wire_vocabularies() {
        assert_eq!(
            parse_capability_scope_mode("scope_aware").expect("scope mode"),
            CapabilityScopeMode::ScopeAware
        );
        assert_eq!(
            parse_provenance_source_kind("transition_contribution").expect("source kind"),
            CapabilityProvenanceSourceKind::TransitionContribution
        );
        assert_eq!(
            parse_capability_provider_state("core_authoritative").expect("provider state"),
            CapabilityProviderState::CoreAuthoritative
        );
        assert!(parse_capability_scope_mode("sometimes_global").is_err());
        assert!(parse_provenance_source_kind("module_guess").is_err());
        assert!(parse_capability_provider_state("healthy").is_err());
    }

    #[test]
    fn role_scope_mode_classification_allows_only_the_admin_all_mixed_exception() {
        assert_eq!(
            classify_role_capability_scope_modes([("forms:read", "scope_aware")])
                .expect("scope-aware role"),
            RoleScopeMode::ScopeAware
        );
        assert_eq!(
            classify_role_capability_scope_modes([("modules:read", "installation_global")])
                .expect("global role"),
            RoleScopeMode::InstallationGlobal
        );
        assert!(matches!(
            classify_role_capability_scope_modes([
                ("forms:read", "scope_aware"),
                ("modules:read", "installation_global"),
            ]),
            Err(ApiError::MixedCapabilityScopeModes)
        ));
        assert_eq!(
            classify_role_capability_scope_modes([
                ("admin:all", "installation_global"),
                ("forms:read", "scope_aware"),
            ])
            .expect("admin:all exception"),
            RoleScopeMode::InstallationGlobal
        );
    }
}
