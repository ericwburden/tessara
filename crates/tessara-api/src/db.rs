//! Database connection, migration, and seed wiring for the API service.

use std::path::PathBuf;

use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};

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

    Migrator::new(migrations_dir().as_path())
        .await?
        .run(&pool)
        .await?;
    seed_dev_admin(&pool, config).await?;

    Ok(pool)
}

fn migrations_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("TESSARA_MIGRATIONS_DIR") {
        return PathBuf::from(path);
    }

    let workspace_path = PathBuf::from("crates/tessara-api/migrations");
    if workspace_path.exists() {
        return workspace_path;
    }

    PathBuf::from("migrations")
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
            "hierarchy:manage",
            "Manage hierarchy configuration and nodes",
        ),
        ("forms:read", "Browse top-level form records"),
        ("forms:manage", "Manage form definitions and versions"),
        (
            "workflows:read",
            "Browse workflow definitions and assignments",
        ),
        (
            "workflows:manage",
            "Manage workflow definitions and assignments",
        ),
        (
            "submissions:read_own",
            "Read own and delegated response work",
        ),
        (
            "submissions:respond",
            "Start and complete assigned response work",
        ),
        (
            "submissions:manage",
            "Manage submissions by hierarchy scope",
        ),
        ("analytics:refresh", "Refresh analytics projections"),
        (
            "operations:view",
            "Inspect workflow assignment and dataset readiness status",
        ),
        ("datasets:manage", "Manage dataset definitions"),
        ("datasets:read", "Inspect dataset definitions"),
        ("components:manage", "Manage component definitions"),
        ("components:read", "Inspect component definitions"),
        ("dashboards:manage", "Manage dashboard definitions"),
        ("dashboards:read", "Inspect dashboard definitions"),
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
        "workflows:manage",
        "submissions:respond",
        "submissions:manage",
        "operations:view",
        "components:read",
        "dashboards:read",
    ] {
        let capability_id: uuid::Uuid =
            sqlx::query_scalar("SELECT id FROM capabilities WHERE key = $1")
                .bind(capability_key)
                .fetch_one(pool)
                .await?;
        ensure_role_capability(pool, "operator", capability_id).await?;
    }

    for capability_key in ["submissions:read_own", "submissions:respond"] {
        let capability_id: uuid::Uuid =
            sqlx::query_scalar("SELECT id FROM capabilities WHERE key = $1")
                .bind(capability_key)
                .fetch_one(pool)
                .await?;
        ensure_role_capability(pool, "respondent", capability_id).await?;
    }

    let admin_role_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM roles WHERE name = 'admin'")
        .fetch_one(pool)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO role_assignments (account_id, role_id, node_id)
        VALUES ($1, $2, NULL)
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
