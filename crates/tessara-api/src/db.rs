//! Database connection, migration, and seed wiring for the API service.

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{auth, config::Config};

/// Shared application state injected into Axum handlers.
///
/// The state currently holds the PostgreSQL pool and immutable runtime config.
/// Keeping it small makes handler dependencies explicit and easy to test.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool for OLTP and analytics projection queries.
    pub pool: PgPool,
    /// Runtime configuration used by handlers such as the development login.
    pub config: Config,
}

/// Connects to PostgreSQL, applies embedded migrations, and seeds the
/// development administrator role/capability graph.
///
/// This is the primary startup entry point for both the API server and
/// command-line maintenance modes such as `seed-demo`.
pub async fn connect_and_prepare(config: &Config) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    seed_dev_admin(&pool, config).await?;
    auth::backfill_legacy_password_hashes(&pool).await?;

    Ok(pool)
}

async fn seed_dev_admin(pool: &PgPool, config: &Config) -> anyhow::Result<()> {
    let admin_account_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO accounts (email, display_name)
        VALUES ($1, 'Tessara Admin')
        ON CONFLICT (email) DO UPDATE SET display_name = EXCLUDED.display_name
        RETURNING id
        "#,
    )
    .bind(&config.dev_admin_email)
    .fetch_one(pool)
    .await?;

    let capabilities = [
        ("admin:all", "Full administration access"),
        ("hierarchy:read", "Browse runtime hierarchy records"),
        (
            "hierarchy:write",
            "Manage hierarchy configuration and nodes",
        ),
        ("forms:read", "Browse top-level form records"),
        ("forms:write", "Manage form definitions and versions"),
        (
            "workflows:read",
            "Browse workflow definitions and assignments",
        ),
        (
            "workflows:write",
            "Manage workflow definitions and assignments",
        ),
        ("submissions:write", "Create and update submissions"),
        ("analytics:refresh", "Refresh analytics projections"),
        ("datasets:write", "Manage dataset definitions"),
        ("datasets:read", "Inspect dataset definitions"),
        ("reports:write", "Manage report definitions"),
        ("reports:read", "Run report definitions"),
        ("aggregations:write", "Manage aggregation definitions"),
        ("aggregations:read", "Run aggregation definitions"),
    ];

    for (key, description) in capabilities {
        let capability_id: uuid::Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO capabilities (key, description)
            VALUES ($1, $2)
            ON CONFLICT (key) DO UPDATE SET description = EXCLUDED.description
            RETURNING id
            "#,
        )
        .bind(key)
        .bind(description)
        .fetch_one(pool)
        .await?;

        ensure_role_capability(pool, "admin", capability_id).await?;
    }

    for capability_key in [
        "hierarchy:read",
        "forms:read",
        "workflows:read",
        "workflows:write",
        "submissions:write",
        "reports:read",
    ] {
        let capability_id: uuid::Uuid =
            sqlx::query_scalar("SELECT id FROM capabilities WHERE key = $1")
                .bind(capability_key)
                .fetch_one(pool)
                .await?;
        ensure_role_capability(pool, "operator", capability_id).await?;
    }

    let respondent_capability_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM capabilities WHERE key = 'submissions:write'")
            .fetch_one(pool)
            .await?;
    ensure_role_capability(pool, "respondent", respondent_capability_id).await?;

    let admin_role_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM roles WHERE name = 'admin'")
        .fetch_one(pool)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO account_role_assignments (account_id, role_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(admin_account_id)
    .bind(admin_role_id)
    .execute(pool)
    .await?;

    auth::store_password_hash(pool, admin_account_id, &config.dev_admin_password).await?;

    Ok(())
}

async fn ensure_role_capability(
    pool: &PgPool,
    role_name: &str,
    capability_id: uuid::Uuid,
) -> anyhow::Result<()> {
    let role_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO roles (name)
        VALUES ($1)
        ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(role_name)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO role_capabilities (role_id, capability_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(role_id)
    .bind(capability_id)
    .execute(pool)
    .await?;

    Ok(())
}
