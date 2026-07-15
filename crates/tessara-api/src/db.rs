//! Database connection, migration, and seed wiring for the API service.

use std::path::PathBuf;

use sha2::{Digest, Sha256};
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};

use crate::{auth, config::Config, modules};

/// Exact replaceable membership contract for the three Tessara-owned roles.
///
/// Only `role_capabilities` membership for these names is seed-owned. Existing
/// role rows/IDs, assignments, accounts, sessions, and user-managed roles are
/// never replaced by synchronization.
pub const BUILT_IN_ROLE_CAPABILITY_SEED: &[(&str, &[&str])] = &[
    // `admin:all` universally implies every product and module capability.
    // Storing only that global capability keeps the role single-mode without
    // changing its effective authority.
    ("admin", &["admin:all"]),
    (
        "operator",
        &[
            "hierarchy:read",
            "forms:read",
            "workflows:read",
            "workflows:manage",
            "submissions:respond",
            "submissions:manage",
            "operations:view",
            "datasets:read",
            "components:read",
            "dashboards:read",
        ],
    ),
    (
        "respondent",
        &["submissions:read_own", "submissions:respond"],
    ),
];

/// Stable review identifier for the exact built-in membership set above.
///
/// The suffix is the first twelve hexadecimal characters of
/// [`BUILT_IN_ROLE_CAPABILITY_SEED_SHA256`]. This coupling makes a membership
/// change require both a new digest and an intentional version change.
pub const BUILT_IN_ROLE_CAPABILITY_SEED_VERSION: &str =
    "sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68";

/// SHA-256 of [`built_in_role_capability_seed_canonical_bytes`].
pub const BUILT_IN_ROLE_CAPABILITY_SEED_SHA256: &str =
    "2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600";

/// Returns the canonical bytes covered by the built-in membership digest.
///
/// The wire is deliberately tiny and explicit: one `role=<name>` line followed
/// by its ordered `capability=<key>` lines, all UTF-8 with LF terminators.
pub fn built_in_role_capability_seed_canonical_bytes() -> Vec<u8> {
    let mut canonical = String::new();
    for &(role_name, capability_keys) in BUILT_IN_ROLE_CAPABILITY_SEED {
        canonical.push_str("role=");
        canonical.push_str(role_name);
        canonical.push('\n');
        for &capability_key in capability_keys {
            canonical.push_str("capability=");
            canonical.push_str(capability_key);
            canonical.push('\n');
        }
    }
    canonical.into_bytes()
}

fn verify_built_in_role_capability_seed_contract() -> anyhow::Result<()> {
    let actual_digest = sha256_hex(&built_in_role_capability_seed_canonical_bytes());
    anyhow::ensure!(
        actual_digest == BUILT_IN_ROLE_CAPABILITY_SEED_SHA256,
        "built-in role capability seed does not match its declared digest; bump the contract version, update the digest, tests, and Sprint test change log"
    );
    let expected_suffix = format!("+sha256.{}", &actual_digest[..12]);
    anyhow::ensure!(
        BUILT_IN_ROLE_CAPABILITY_SEED_VERSION.ends_with(&expected_suffix),
        "built-in role capability seed version must end with '{expected_suffix}'"
    );
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

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
    modules::synchronize_catalog(&pool).await?;

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
        (
            "datasets:read_restricted",
            "Read restricted dataset rows when dataset visibility allows access",
        ),
        (
            "datasets:read_confidential",
            "Read confidential and restricted dataset rows when dataset visibility allows access",
        ),
        ("components:manage", "Manage component definitions"),
        ("components:read", "Inspect component definitions"),
        ("dashboards:manage", "Manage dashboard definitions"),
        ("dashboards:read", "Inspect dashboard definitions"),
    ];

    for (key, description) in capabilities {
        let _capability_id: uuid::Uuid = sqlx::query_scalar(
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
    }

    synchronize_seed_role_capabilities(pool).await?;

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

async fn synchronize_seed_role_capabilities(pool: &PgPool) -> anyhow::Result<()> {
    verify_built_in_role_capability_seed_contract()?;

    let mut tx = pool.begin().await?;
    for &(role_name, _) in BUILT_IN_ROLE_CAPABILITY_SEED {
        sqlx::query(
            r#"
            INSERT INTO roles (name)
            VALUES ($1)
            ON CONFLICT (name) DO NOTHING
            "#,
        )
        .bind(role_name)
        .execute(&mut *tx)
        .await?;
    }

    let seed_role_names = BUILT_IN_ROLE_CAPABILITY_SEED
        .iter()
        .map(|(role_name, _)| (*role_name).to_string())
        .collect::<Vec<_>>();
    // Assignment writers lock every selected role in ascending UUID order.
    // Take the stronger seed locks in that same order so a concurrent access
    // rewrite and startup synchronization cannot form an inverted lock cycle.
    let locked_roles: Vec<(uuid::Uuid, String)> = sqlx::query_as(
        r#"
        SELECT id, name
        FROM roles
        WHERE name = ANY($1)
        ORDER BY id
        FOR UPDATE
        "#,
    )
    .bind(&seed_role_names)
    .fetch_all(&mut *tx)
    .await?;
    anyhow::ensure!(
        locked_roles.len() == BUILT_IN_ROLE_CAPABILITY_SEED.len(),
        "not every built-in seed role could be locked"
    );

    let mut role_ids = Vec::with_capacity(BUILT_IN_ROLE_CAPABILITY_SEED.len());
    for &(role_name, capability_keys) in BUILT_IN_ROLE_CAPABILITY_SEED {
        let role_id = locked_roles
            .iter()
            .find_map(|(role_id, stored_name)| (stored_name == role_name).then_some(*role_id))
            .ok_or_else(|| anyhow::anyhow!("built-in seed role '{role_name}' is missing"))?;
        for &capability_key in capability_keys {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM capabilities WHERE key = $1)")
                    .bind(capability_key)
                    .fetch_one(&mut *tx)
                    .await?;
            anyhow::ensure!(
                exists,
                "seed role '{role_name}' references unknown capability '{capability_key}'"
            );
        }
        role_ids.push((role_name, role_id, capability_keys));
    }

    // All references are validated before deleting any membership. The
    // replacement then occurs in the same transaction, so callers can observe
    // either the prior exact set or the new exact set, never a partial set.
    for (role_id, _) in &locked_roles {
        sqlx::query("DELETE FROM role_capabilities WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
    }
    for (role_name, role_id, capability_keys) in role_ids {
        for &capability_key in capability_keys {
            let result = sqlx::query(
                r#"
                INSERT INTO role_capabilities (role_id, capability_id)
                SELECT $1, id
                FROM capabilities
                WHERE key = $2
                "#,
            )
            .bind(role_id)
            .bind(capability_key)
            .execute(&mut *tx)
            .await?;
            anyhow::ensure!(
                result.rows_affected() == 1,
                "seed role '{role_name}' references unknown capability '{capability_key}'"
            );
        }
    }
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::{
        BUILT_IN_ROLE_CAPABILITY_SEED_SHA256, BUILT_IN_ROLE_CAPABILITY_SEED_VERSION,
        built_in_role_capability_seed_canonical_bytes,
        verify_built_in_role_capability_seed_contract,
    };

    const BASELINE: &[u8] = include_bytes!("../migrations/001_baseline.sql");
    const DASHBOARD_PLACEMENT_CAPACITY: &[u8] =
        include_bytes!("../migrations/002_dashboard_placement_capacity.sql");

    #[test]
    fn built_in_role_capability_seed_contract_is_exact_and_review_versioned() {
        assert_eq!(
            BUILT_IN_ROLE_CAPABILITY_SEED_VERSION,
            "sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68"
        );
        assert_eq!(
            BUILT_IN_ROLE_CAPABILITY_SEED_SHA256,
            "2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600"
        );
        assert_eq!(
            super::sha256_hex(&built_in_role_capability_seed_canonical_bytes()),
            BUILT_IN_ROLE_CAPABILITY_SEED_SHA256
        );
        verify_built_in_role_capability_seed_contract()
            .expect("checked-in membership contract should match its version and digest");
    }

    #[test]
    fn squashed_baseline_migration_remains_immutable() {
        assert_eq!(fnv1a(BASELINE), 0xb2f5_7278_25e1_d5b9);
        let baseline = std::str::from_utf8(BASELINE).expect("baseline migration is UTF-8");
        assert!(baseline.contains(
            "CREATE TYPE component_type AS ENUM ('table', 'bar', 'line', 'pie', 'donut', 'stat_card');"
        ));
        assert!(baseline.contains("component_versions_component_type_supported_chk"));
        for kind in ["table", "bar", "line", "pie", "donut", "stat_card"] {
            assert!(baseline.contains(&format!("'{kind}'::component_type")));
        }
        assert!(!baseline.contains("component_versions_component_type_table_chk"));
    }

    #[test]
    fn pre_control_plane_migration_bytes_remain_immutable() {
        assert_eq!(
            sha256_hex(BASELINE),
            "a61f5192ad8e14bdcbbd26203301030fd57b647a237218c1e5443936944e9ca0"
        );
        assert_eq!(
            sha256_hex(DASHBOARD_PLACEMENT_CAPACITY),
            "c26a100e7fcd7aba4a74622c03f6c8e809219022595206da3ba7ddc86313550e"
        );
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn fnv1a(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    }
}
