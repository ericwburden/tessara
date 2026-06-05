use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth,
    error::{ApiError, ApiResult},
};

pub(super) async fn ensure_demo_account(
    pool: &PgPool,
    email: &str,
    display_name: &str,
    role_name: &str,
    password: &str,
) -> ApiResult<Uuid> {
    let account_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO accounts (email, display_name)
        VALUES ($1, $2)
        ON CONFLICT (email)
        DO UPDATE SET display_name = EXCLUDED.display_name
        RETURNING id
        "#,
    )
    .bind(email)
    .bind(display_name)
    .fetch_one(pool)
    .await?;

    auth::store_password_hash(pool, account_id, password).await?;

    let role_id: Uuid = sqlx::query_scalar("SELECT id FROM roles WHERE name = $1")
        .bind(role_name)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("role {role_name}")))?;

    sqlx::query(
        r#"
        INSERT INTO role_assignments (account_id, role_id, node_id)
        VALUES ($1, $2, NULL)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(account_id)
    .bind(role_id)
    .execute(pool)
    .await?;

    Ok(account_id)
}

pub(super) async fn ensure_account_scope_assignment(
    pool: &PgPool,
    account_id: Uuid,
    node_id: Uuid,
) -> ApiResult<()> {
    let role_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT role_id
        FROM role_assignments
        WHERE account_id = $1
        GROUP BY role_id
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM role_assignments
        WHERE account_id = $1
          AND node_id IS NULL
          AND NOT EXISTS (
              SELECT 1
              FROM role_capabilities
              JOIN capabilities ON capabilities.id = role_capabilities.capability_id
              WHERE role_capabilities.role_id = role_assignments.role_id
                AND capabilities.key = 'admin:all'
          )
        "#,
    )
    .bind(account_id)
    .execute(pool)
    .await?;

    for role_id in role_ids {
        sqlx::query(
            r#"
            INSERT INTO role_assignments (account_id, role_id, node_id)
            SELECT $1, $2, $3
            WHERE NOT EXISTS (
                SELECT 1
                FROM role_capabilities
                JOIN capabilities ON capabilities.id = role_capabilities.capability_id
                WHERE role_capabilities.role_id = $2
                  AND capabilities.key = 'admin:all'
            )
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(account_id)
        .bind(role_id)
        .bind(node_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub(super) async fn ensure_account_delegation(
    pool: &PgPool,
    delegator_account_id: Uuid,
    delegate_account_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        INSERT INTO account_delegations (delegator_account_id, delegate_account_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(delegator_account_id)
    .bind(delegate_account_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn require_dev_admin_account(pool: &PgPool) -> ApiResult<Uuid> {
    sqlx::query_scalar("SELECT id FROM accounts WHERE email = 'admin@tessara.local' LIMIT 1")
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound("dev admin account".into()))
}
