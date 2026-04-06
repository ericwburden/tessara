//! Database connection, migration, and seed wiring for the API service.

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::config::Config;

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

    let admin_role_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO roles (name)
        VALUES ('admin')
        ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .fetch_one(pool)
    .await?;

    let capabilities = [
        ("admin:all", "Full administration access"),
        (
            "hierarchy:write",
            "Manage hierarchy configuration and nodes",
        ),
        ("forms:write", "Manage form definitions and versions"),
        ("submissions:write", "Create and update submissions"),
        ("analytics:refresh", "Refresh analytics projections"),
        ("reports:write", "Manage report definitions"),
        ("reports:read", "Run report definitions"),
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

        sqlx::query(
            r#"
            INSERT INTO role_capabilities (role_id, capability_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(admin_role_id)
        .bind(capability_id)
        .execute(pool)
        .await?;
    }

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

    Ok(())
}
