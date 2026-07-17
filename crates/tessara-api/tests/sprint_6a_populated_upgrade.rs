use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use tessara_api::{config::Config, db, router};
use tower::ServiceExt;
use uuid::Uuid;

#[path = "support/database_safety.rs"]
mod database_safety;

use database_safety::is_disposable_database_name;

const BASELINE: &[u8] = include_bytes!("../migrations/001_baseline.sql");
const DASHBOARD_CAPACITY: &[u8] =
    include_bytes!("../migrations/002_dashboard_placement_capacity.sql");
const POPULATED_SPRINT_5A: &str = include_str!("fixtures/sprint_5a_populated.sql");
const POPULATED_SPRINT_5A_SHA256: &str =
    "29db015ddcd7206a548c5839b958a937c03aab78d2c53047a55483a7aef31172";
const PRODUCT_TABLES: &[&str] = &[
    "node_types",
    "node_type_relationships",
    "node_metadata_field_definitions",
    "nodes",
    "node_metadata_values",
    "forms",
    "form_scope_nodes",
    "compatibility_groups",
    "form_versions",
    "form_sections",
    "form_fields",
    "choice_lists",
    "choice_list_items",
    "workflows",
    "workflow_available_nodes",
    "workflow_versions",
    "workflow_steps",
    "workflow_assignments",
    "workflow_instances",
    "submissions",
    "workflow_step_instances",
    "submission_values",
    "submission_value_multi",
    "submission_audit_events",
    "datasets",
    "dataset_scope_nodes",
    "dataset_tags",
    "dataset_revisions",
    "dataset_major_materializations",
    "dataset_sources",
    "dataset_fields",
    "components",
    "component_versions",
    "dashboards",
    "dashboard_scope_nodes",
    "dashboard_components",
];

// This literal inventory is independent of the fixture SQL and migration
// snapshot. Every source-of-truth table that existed after Sprint 5A has a
// non-empty representative family, so deleting an INSERT cannot leave the
// before/after equality vacuously green. The migration ledger is system-owned
// and asserted separately; the seven analytics tables and physical dataset
// outputs are reproducible projections and are explicitly classified below.
const PRE_SPRINT_6A_DURABLE_TABLE_COUNTS: &[(&str, i64)] = &[
    ("accounts", 2),
    ("account_credentials", 1),
    ("roles", 4),
    ("capabilities", 20),
    ("role_capabilities", 33),
    ("auth_sessions", 1),
    ("node_types", 2),
    ("node_type_relationships", 1),
    ("node_metadata_field_definitions", 1),
    ("nodes", 2),
    ("role_assignments", 3),
    ("account_delegations", 1),
    ("node_metadata_values", 1),
    ("forms", 1),
    ("form_scope_nodes", 1),
    ("compatibility_groups", 1),
    ("form_versions", 1),
    ("form_sections", 1),
    ("form_fields", 2),
    ("choice_lists", 1),
    ("choice_list_items", 2),
    ("workflows", 1),
    ("workflow_available_nodes", 1),
    ("workflow_versions", 1),
    ("workflow_steps", 1),
    ("workflow_assignments", 1),
    ("workflow_instances", 1),
    ("submissions", 1),
    ("workflow_step_instances", 1),
    ("submission_values", 1),
    ("submission_value_multi", 2),
    ("submission_audit_events", 1),
    ("datasets", 1),
    ("dataset_scope_nodes", 1),
    ("dataset_tags", 2),
    ("dataset_revisions", 1),
    ("dataset_major_materializations", 1),
    ("dataset_sources", 1),
    ("dataset_fields", 1),
    ("components", 1),
    ("component_versions", 1),
    ("dashboards", 1),
    ("dashboard_scope_nodes", 1),
    ("dashboard_components", 1),
];

const FULL_ROW_PRESERVATION_TABLES: &[&str] = &[
    "accounts",
    "roles",
    "node_types",
    "node_type_relationships",
    "node_metadata_field_definitions",
    "nodes",
    "role_assignments",
    "account_delegations",
    "node_metadata_values",
    "forms",
    "form_scope_nodes",
    "compatibility_groups",
    "form_versions",
    "form_sections",
    "form_fields",
    "choice_lists",
    "choice_list_items",
    "workflows",
    "workflow_available_nodes",
    "workflow_versions",
    "workflow_steps",
    "workflow_assignments",
    "workflow_instances",
    "submissions",
    "workflow_step_instances",
    "submission_values",
    "submission_value_multi",
    "submission_audit_events",
    "datasets",
    "dataset_scope_nodes",
    "dataset_tags",
    "dataset_revisions",
    "dataset_major_materializations",
    "dataset_sources",
    "dataset_fields",
    "components",
    "component_versions",
    "dashboards",
    "dashboard_scope_nodes",
    "dashboard_components",
];

const DERIVED_ANALYTICS_TABLES: &[&str] = &[
    "compatibility_group_dim",
    "field_dim",
    "form_dim",
    "form_version_dim",
    "node_dim",
    "submission_fact",
    "submission_value_fact",
];

const CONTROL_PLANE_TABLES: &[&str] = &[
    "application_installations",
    "core_runtime_observations",
    "module_definition_reservations",
    "transition_descriptor_sources",
    "transition_catalog_projections",
    "transition_catalog_current",
    "module_catalog_findings",
    "capability_provenance",
    "module_navigation_contributions",
    "navigation_policies",
    "navigation_policy_entries",
    "core_control_plane_audit_events",
    "_sqlx_migrations",
];

const SPRINT_5A_CAPABILITIES: &[(&str, &str)] = &[
    ("admin:all", "Full administration access"),
    ("analytics:refresh", "Refresh analytics projections"),
    ("components:manage", "Manage component definitions"),
    ("components:read", "Inspect component definitions"),
    ("dashboards:manage", "Manage dashboard definitions"),
    ("dashboards:read", "Inspect dashboard definitions"),
    ("datasets:manage", "Manage dataset definitions"),
    ("datasets:read", "Inspect dataset definitions"),
    (
        "datasets:read_confidential",
        "Read confidential and restricted dataset rows when dataset visibility allows access",
    ),
    (
        "datasets:read_restricted",
        "Read restricted dataset rows when dataset visibility allows access",
    ),
    ("forms:manage", "Manage form definitions and versions"),
    ("forms:read", "Browse top-level form records"),
    (
        "hierarchy:manage",
        "Manage hierarchy configuration and nodes",
    ),
    ("hierarchy:read", "Browse runtime hierarchy records"),
    (
        "operations:view",
        "Inspect workflow assignment and dataset readiness status",
    ),
    (
        "submissions:manage",
        "Manage submissions by hierarchy scope",
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
        "workflows:manage",
        "Manage workflow definitions and assignments",
    ),
    (
        "workflows:read",
        "Browse workflow definitions and assignments",
    ),
];

const SPRINT_5A_OPERATOR_CAPABILITIES: &[&str] = &[
    "components:read",
    "dashboards:read",
    "datasets:read",
    "forms:read",
    "hierarchy:read",
    "operations:view",
    "submissions:manage",
    "submissions:respond",
    "workflows:manage",
    "workflows:read",
];

const SPRINT_5A_RESPONDENT_CAPABILITIES: &[&str] = &["submissions:read_own", "submissions:respond"];

const SPRINT_5A_SEED_VERSION: &str = "sprint-5a-role-capabilities-v1+sha256.7725e889996a";
const SPRINT_5A_SEED_SHA256: &str =
    "7725e889996a73a5655c57106aca6e12d9a5f95e9103f14d7b0fd50fbac96988";

const CURRENT_SEED_VERSION: &str = "sprint-6a-role-capabilities-v1+sha256.2c21a9ebed68";
const CURRENT_SEED_SHA256: &str =
    "2c21a9ebed6870c0245a2b1b131e2b053533b0cbae698e8594295eeba92be600";

const FIXTURE_ACCOUNT_ID: &str = "60000000-0000-0000-0000-000000000002";
const FIXTURE_SESSION_TOKEN: &str = "60000000-0000-0000-0000-000000000301";
const DESTRUCTIVE_RESET_ACKNOWLEDGEMENT_ENV: &str = "SPRINT_6A_CONFIRM_DESTRUCTIVE_UPGRADE_RESET";
const DESTRUCTIVE_RESET_ACKNOWLEDGEMENT: &str = "I_UNDERSTAND_THIS_DATABASE_WILL_BE_RESET";
const FRESH_DATABASE_URL_ENV: &str = "SPRINT_6A_FRESH_DATABASE_URL";
const REQUIRED_DATABASE_URL_ENVS: [&str; 3] = [
    "TEST_DATABASE_URL",
    "SPRINT_6A_UPGRADE_DATABASE_URL",
    FRESH_DATABASE_URL_ENV,
];

#[test]
fn disposable_database_names_require_token_boundaries() {
    for accepted in [
        "tessara_sprint6a_test",
        "tessara_sprint6a_upgrade_test",
        "tessara-clone-01",
        "ROLLBACK_snapshot",
        "tessara-tests-01",
        "tessara_testing_01",
        "tessara-sprint-6a-fresh",
    ] {
        assert!(
            is_disposable_database_name(accepted),
            "'{accepted}' should be recognized as explicitly disposable"
        );
    }

    for rejected in [
        "latest",
        "contest",
        "attested",
        "production_upgradeable",
        "sprint6atest",
        "production",
        "",
    ] {
        assert!(
            !is_disposable_database_name(rejected),
            "'{rejected}' must not pass through substring matching"
        );
    }
}

#[tokio::test]
async fn populated_sprint_5a_upgrade_preserves_invariants_and_replaces_seed_atomically() {
    assert_declared_current_seed_contract();
    assert_populated_sprint_5a_fixture_identity();
    let database_url = std::env::var("SPRINT_6A_UPGRADE_DATABASE_URL").expect(
        "SPRINT_6A_UPGRADE_DATABASE_URL is required; the destructive populated upgrade proof must never skip",
    );
    assert!(
        !database_url.trim().is_empty(),
        "SPRINT_6A_UPGRADE_DATABASE_URL must not be empty"
    );
    assert_destructive_upgrade_reset_acknowledged();
    let config = Config {
        database_url: database_url.clone(),
        bind_addr: "127.0.0.1:0".to_string(),
        dev_admin_email: "admin@tessara.local".to_string(),
        dev_admin_password: "sprint-6a-upgrade-proof".to_string(),
        auth_cookie_name: "tessara_session".to_string(),
        auth_cookie_secure: false,
        auth_session_ttl_hours: 12,
    };

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("test database should be reachable");
    assert_disposable_proof_database(&pool).await;
    assert_required_proof_databases_are_pairwise_distinct().await;
    reset_database(&pool).await;

    let pre_control_plane_migrations = PreControlPlaneMigrations::create();
    Migrator::new(pre_control_plane_migrations.path())
        .await
        .expect("migrations 1 and 2 should load")
        .run(&pool)
        .await
        .expect("migrations 1 and 2 should apply");
    sqlx::raw_sql(POPULATED_SPRINT_5A)
        .execute(&pool)
        .await
        .expect("representative Sprint 5A data should load");

    assert_eq!(applied_migrations(&pool).await, vec![1, 2]);
    assert_pre_sprint_6a_table_inventory(&pool).await;
    assert_sprint_5a_seed_precondition(&pool).await;
    // The Sprint 6A binary cannot serve against the Sprint 5A schema because
    // its auth query intentionally consumes migration-3 scope metadata. Use
    // exact Sprint 5A-equivalent queries for the pre-startup side, then compare
    // them with real session/product HTTP and resolved navigation after startup.
    let pre_upgrade_actor = pre_control_plane_actor_snapshot(&pool).await;
    assert_preserved_actor_contract(&pre_upgrade_actor);
    let pre_upgrade_navigation = pre_control_plane_navigation(&pre_upgrade_actor);
    let pre_control_plane_ledger = migration_ledger_snapshot(&pool, 2).await;
    let product_counts_before = product_counts(&pool).await;
    let fixture_before = fixture_snapshot(&pool).await;
    assert!(!fixture_before.is_empty(), "fixture snapshot is meaningful");

    pool.close().await;
    let _working_directory = WorkspaceWorkingDirectory::enter();

    let upgraded = db::connect_and_prepare(&config)
        .await
        .expect("migrations 3 and 4 plus the transition catalog should apply");
    assert_eq!(applied_migrations(&upgraded).await, vec![1, 2, 3, 4]);
    let upgraded_preexisting_ledger = migration_ledger_snapshot(&upgraded, 2).await;
    assert_eq!(
        upgraded_preexisting_ledger, pre_control_plane_ledger,
        "migration 3 must append to, not rewrite, the migration 1/2 ledger"
    );
    assert_eq!(product_counts(&upgraded).await, product_counts_before);
    assert_eq!(fixture_snapshot(&upgraded).await, fixture_before);
    assert_seed_role_updates(&upgraded).await;
    assert_control_plane_shape(&upgraded).await;
    let upgraded_http = preserved_actor_http_snapshot(&upgraded, &config).await;
    assert_eq!(
        upgraded_http, pre_upgrade_actor,
        "the preserved fixture session must retain its account and product HTTP behavior"
    );
    let upgraded_navigation =
        fixture_request_json(&upgraded, &config, "/api/shell/navigation").await;
    assert_available_operator_navigation(&upgraded_navigation);
    assert_eq!(
        pre_upgrade_navigation.admin,
        vec!["datasets"],
        "the characterized Sprint 5A fixture must prove the old Admin placement before migration"
    );
    let stable_catalog = control_plane_snapshot(&upgraded).await;
    upgraded.close().await;

    for restart in 1..=2 {
        let restarted = db::connect_and_prepare(&config)
            .await
            .unwrap_or_else(|error| panic!("restart {restart} should succeed: {error:#}"));
        assert_eq!(product_counts(&restarted).await, product_counts_before);
        assert_eq!(fixture_snapshot(&restarted).await, fixture_before);
        assert_seed_role_updates(&restarted).await;
        assert_eq!(control_plane_snapshot(&restarted).await, stable_catalog);
        restarted.close().await;
    }

    let (left, right) = tokio::join!(
        db::connect_and_prepare(&config),
        db::connect_and_prepare(&config)
    );
    let left = left.expect("first concurrent startup should succeed");
    let right = right.expect("second concurrent startup should succeed");
    assert_eq!(product_counts(&left).await, product_counts_before);
    assert_eq!(fixture_snapshot(&left).await, fixture_before);
    assert_seed_role_updates(&left).await;
    assert_seed_role_updates(&right).await;
    assert_eq!(control_plane_snapshot(&left).await, stable_catalog);
    assert_eq!(control_plane_snapshot(&right).await, stable_catalog);
    left.close().await;
    right.close().await;

    // Prove exact-set replacement rather than additive seeding. Introduce both
    // a stale membership and a missing required membership, then restart. The
    // transaction must restore the declared set without changing any role row,
    // role ID, assignment, account, session, or user-managed membership.
    let drifted = PgPoolOptions::new()
        .max_connections(2)
        .connect(&config.database_url)
        .await
        .expect("upgrade database should remain reachable");
    let invariant_snapshot_before_repair = fixture_snapshot(&drifted).await;
    sqlx::query(
        r#"
        INSERT INTO role_capabilities (role_id, capability_id)
        SELECT roles.id, capabilities.id
        FROM roles
        CROSS JOIN capabilities
        WHERE roles.name = 'admin' AND capabilities.key = 'forms:read'
        ON CONFLICT DO NOTHING
        "#,
    )
    .execute(&drifted)
    .await
    .expect("stale built-in membership should be injectable");
    sqlx::query(
        r#"
        DELETE FROM role_capabilities
        USING roles, capabilities
        WHERE role_capabilities.role_id = roles.id
          AND role_capabilities.capability_id = capabilities.id
          AND roles.name = 'operator'
          AND capabilities.key = 'dashboards:read'
        "#,
    )
    .execute(&drifted)
    .await
    .expect("required built-in membership should be removable for repair proof");
    assert_ne!(
        seeded_role_capability_snapshot(&drifted).await,
        expected_current_seed_role_capabilities(),
        "repair setup must actually differ from the declared seed contract"
    );
    drifted.close().await;

    let repaired = db::connect_and_prepare(&config)
        .await
        .expect("seed replacement should repair the complete set atomically");
    assert_seed_role_updates(&repaired).await;
    assert_eq!(
        fixture_snapshot(&repaired).await,
        invariant_snapshot_before_repair,
        "seed replacement may change built-in membership only"
    );
    assert_eq!(control_plane_snapshot(&repaired).await, stable_catalog);
    repaired.close().await;

    // Deliberately leave this representative populated fixture at migration 4
    // for invariant and CompatibilityOnUpgraded inspection. Closing-build
    // smoke, UAT, and browser acceptance use a separate Sprint 5A demo clone:
    // restore it, prove OriginalAfterRestore, then let the closing startup apply
    // migrations 3 and 4 with demo seeding disabled.
}

#[tokio::test]
async fn fresh_startup_and_seed_assignment_lock_order_use_a_separate_database() {
    assert_declared_current_seed_contract();
    assert_destructive_upgrade_reset_acknowledged();
    let database_url = std::env::var(FRESH_DATABASE_URL_ENV).unwrap_or_else(|_| {
        panic!(
            "{FRESH_DATABASE_URL_ENV} is required; fresh-start and lock-order proof must never reset the populated upgrade clone"
        )
    });
    assert!(
        !database_url.trim().is_empty(),
        "{FRESH_DATABASE_URL_ENV} must not be empty"
    );
    let config = Config {
        database_url: database_url.clone(),
        bind_addr: "127.0.0.1:0".to_string(),
        dev_admin_email: "admin@tessara.local".to_string(),
        dev_admin_password: "sprint-6a-fresh-proof".to_string(),
        auth_cookie_name: "tessara_session".to_string(),
        auth_cookie_secure: false,
        auth_session_ttl_hours: 12,
    };
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("fresh proof database should be reachable");
    assert_disposable_proof_database(&pool).await;
    assert_required_proof_databases_are_pairwise_distinct().await;
    pool.close().await;

    prove_seed_and_assignment_lock_order_is_deadlock_free(&config).await;
    let fresh = db::connect_and_prepare(&config)
        .await
        .expect("fresh Sprint 6A startup should remain healthy after lock-order proof");
    assert_eq!(applied_migrations(&fresh).await, vec![1, 2, 3, 4]);
    assert_seed_role_updates(&fresh).await;
    assert_control_plane_shape(&fresh).await;
    fresh.close().await;
}

#[derive(Debug, PartialEq)]
struct PreservedActorSnapshot {
    account_id: String,
    email: String,
    roles: Vec<String>,
    capabilities: Vec<String>,
    scope_node_ids: Vec<String>,
    readable_form_ids: Vec<String>,
}

async fn pre_control_plane_actor_snapshot(pool: &PgPool) -> PreservedActorSnapshot {
    let account_id =
        Uuid::parse_str(FIXTURE_ACCOUNT_ID).expect("fixture account ID should be valid");
    let session_token =
        Uuid::parse_str(FIXTURE_SESSION_TOKEN).expect("fixture session token should be valid");
    let session: (String, String) = sqlx::query_as(
        r#"
        SELECT accounts.id::text, accounts.email
        FROM auth_sessions
        JOIN accounts ON accounts.id = auth_sessions.account_id
        WHERE auth_sessions.token = $1
          AND auth_sessions.revoked_at IS NULL
          AND auth_sessions.expires_at > now()
          AND accounts.is_active
        "#,
    )
    .bind(session_token)
    .fetch_one(pool)
    .await
    .expect("the preserved Sprint 5A session should authenticate by durable query");
    let roles = sqlx::query_scalar(
        r#"
        SELECT DISTINCT roles.name
        FROM role_assignments
        JOIN roles ON roles.id = role_assignments.role_id
        WHERE role_assignments.account_id = $1
        ORDER BY roles.name
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .expect("preserved Sprint 5A roles should be queryable");
    let mut capabilities: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT capabilities.key
        FROM role_assignments
        JOIN role_capabilities ON role_capabilities.role_id = role_assignments.role_id
        JOIN capabilities ON capabilities.id = role_capabilities.capability_id
        WHERE role_assignments.account_id = $1
        ORDER BY capabilities.key
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .expect("preserved Sprint 5A capabilities should be queryable");
    let implied_reads = capabilities
        .iter()
        .filter_map(|capability| {
            capability
                .strip_suffix(":manage")
                .filter(|domain| *domain != "dashboards")
                .map(|domain| format!("{domain}:read"))
        })
        .collect::<Vec<_>>();
    capabilities.extend(implied_reads);
    capabilities.sort();
    capabilities.dedup();
    let scope_node_ids = sqlx::query_scalar(
        r#"
        SELECT DISTINCT node_id::text
        FROM role_assignments
        WHERE account_id = $1
          AND node_id IS NOT NULL
        ORDER BY node_id::text
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .expect("preserved Sprint 5A scope roots should be queryable");
    let readable_form_ids = sqlx::query_scalar(
        r#"
        SELECT DISTINCT forms.id::text
        FROM forms
        JOIN form_scope_nodes ON form_scope_nodes.form_id = forms.id
        WHERE EXISTS (
            SELECT 1
            FROM role_assignments
            JOIN role_capabilities
              ON role_capabilities.role_id = role_assignments.role_id
            JOIN capabilities
              ON capabilities.id = role_capabilities.capability_id
            WHERE role_assignments.account_id = $1
              AND capabilities.key IN ('forms:read', 'forms:manage')
              AND (
                  role_assignments.node_id IS NULL
                  OR role_assignments.node_id = form_scope_nodes.node_id
              )
        )
        ORDER BY forms.id::text
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .expect("preserved Sprint 5A readable forms should be queryable");

    PreservedActorSnapshot {
        account_id: session.0,
        email: session.1,
        roles,
        capabilities,
        scope_node_ids,
        readable_form_ids,
    }
}

async fn preserved_actor_http_snapshot(pool: &PgPool, config: &Config) -> PreservedActorSnapshot {
    let session = fixture_request_json(pool, config, "/api/auth/session").await;
    assert_eq!(session["authenticated"], true);
    let account = &session["account"];
    let readable_forms = fixture_request_json(pool, config, "/api/forms").await;
    PreservedActorSnapshot {
        account_id: account["account_id"]
            .as_str()
            .expect("session account ID should be a string")
            .to_string(),
        email: account["email"]
            .as_str()
            .expect("session email should be a string")
            .to_string(),
        roles: json_owned_string_array(&account["roles"]),
        capabilities: json_owned_string_array(&account["capabilities"]),
        scope_node_ids: account["scope_nodes"]
            .as_array()
            .expect("session scope nodes should be an array")
            .iter()
            .map(|node| {
                node["node_id"]
                    .as_str()
                    .expect("session scope node ID should be a string")
                    .to_string()
            })
            .collect(),
        readable_form_ids: readable_forms
            .as_array()
            .expect("readable forms response should be an array")
            .iter()
            .map(|form| {
                form["id"]
                    .as_str()
                    .expect("form ID should be a string")
                    .to_string()
            })
            .collect(),
    }
}

fn assert_preserved_actor_contract(snapshot: &PreservedActorSnapshot) {
    assert_eq!(snapshot.account_id, FIXTURE_ACCOUNT_ID);
    assert_eq!(snapshot.email, "existing.user@tessara.local");
    assert_eq!(
        snapshot.roles,
        vec!["existing-custom-role".to_string(), "operator".to_string()]
    );
    assert_eq!(
        snapshot.capabilities,
        vec![
            "components:read".to_string(),
            "dashboards:read".to_string(),
            "datasets:read".to_string(),
            "forms:read".to_string(),
            "hierarchy:read".to_string(),
            "operations:view".to_string(),
            "submissions:manage".to_string(),
            "submissions:read".to_string(),
            "submissions:respond".to_string(),
            "workflows:manage".to_string(),
            "workflows:read".to_string(),
        ]
    );
    assert_eq!(
        snapshot.scope_node_ids,
        vec!["60000000-0000-0000-0000-000000000402".to_string()]
    );
    assert_eq!(
        snapshot.readable_form_ids,
        vec!["60000000-0000-0000-0000-000000000601".to_string()]
    );
}

#[derive(Debug, PartialEq)]
struct NavigationSnapshot {
    main: Vec<String>,
    admin: Vec<String>,
}

fn pre_control_plane_navigation(actor: &PreservedActorSnapshot) -> NavigationSnapshot {
    let has = |capability: &str| actor.capabilities.iter().any(|key| key == capability);
    let has_any = |capabilities: &[&str]| capabilities.iter().any(|capability| has(capability));
    let mut main = vec!["home".to_string()];
    if has_any(&["hierarchy:read", "hierarchy:manage"]) {
        main.push("organization".to_string());
    }
    if has_any(&["forms:read", "forms:manage"]) {
        main.push("forms".to_string());
    }
    if has_any(&["workflows:read", "workflows:manage"]) {
        main.push("workflows".to_string());
    }
    if has_any(&[
        "submissions:read_own",
        "submissions:respond",
        "submissions:manage",
    ]) {
        main.push("responses".to_string());
    }
    if has("operations:view") {
        main.push("operations".to_string());
    }
    if has_any(&["components:read", "components:manage"]) {
        main.push("components".to_string());
    }
    if has("dashboards:read") {
        main.push("dashboards".to_string());
    }

    let mut admin = Vec::new();
    if has("admin:all") {
        admin.push("administration".to_string());
    }
    if has_any(&["datasets:read", "datasets:manage"]) {
        admin.push("datasets".to_string());
    }
    NavigationSnapshot { main, admin }
}

fn assert_available_operator_navigation(response: &Value) {
    assert_eq!(response["schema_version"], 2);
    assert_eq!(response["state"], "available");
    assert_eq!(response["policy_revision"], 0);
    assert!(response["unavailable"].is_null());
    assert_eq!(
        navigation_keys(response, "Main"),
        vec![
            "home",
            "organization",
            "forms",
            "workflows",
            "responses",
            "operations",
            "datasets",
            "components",
            "dashboards",
        ]
    );
    assert!(navigation_keys(response, "Admin").is_empty());
}

fn navigation_keys<'a>(response: &'a Value, group_name: &str) -> Vec<&'a str> {
    response["groups"]
        .as_array()
        .expect("navigation groups should be an array")
        .iter()
        .find(|group| group["name"] == group_name)
        .and_then(|group| group["items"].as_array())
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    item["key"]
                        .as_str()
                        .expect("navigation key should be a string")
                })
                .collect()
        })
        .unwrap_or_default()
}

fn json_owned_string_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("value should be an array")
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .expect("array entry should be a string")
                .to_string()
        })
        .collect()
}

async fn fixture_request_json(pool: &PgPool, config: &Config, uri: &str) -> Value {
    let app = router(db::AppState {
        pool: pool.clone(),
        config: config.clone(),
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .header(
                    header::COOKIE,
                    format!("{}={FIXTURE_SESSION_TOKEN}", config.auth_cookie_name),
                )
                .body(Body::empty())
                .expect("fixture HTTP request should be valid"),
        )
        .await
        .expect("fixture HTTP response should be produced");
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "fixture session request to {uri} should succeed"
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("fixture HTTP response body should be readable");
    serde_json::from_slice(&body).unwrap_or_else(|error| {
        panic!(
            "fixture HTTP response from {uri} should be JSON ({error}): {}",
            String::from_utf8_lossy(&body)
        )
    })
}

async fn prove_seed_and_assignment_lock_order_is_deadlock_free(config: &Config) {
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&config.database_url)
        .await
        .expect("upgrade database should remain reachable for lock-order proof");
    reset_database(&pool).await;
    let migrations = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
    Migrator::new(migrations)
        .await
        .expect("Sprint 6A migrations should load for lock-order proof")
        .run(&pool)
        .await
        .expect("Sprint 6A migrations should apply for lock-order proof");

    sqlx::query(
        r#"
        INSERT INTO roles (id, name)
        VALUES
            ('60000000-0000-0000-0000-000000009103', 'admin'),
            ('60000000-0000-0000-0000-000000009102', 'operator'),
            ('60000000-0000-0000-0000-000000009101', 'respondent')
        "#,
    )
    .execute(&pool)
    .await
    .expect("adversarially ordered built-in role IDs should be insertable");
    let ordered_roles: Vec<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT id, name
        FROM roles
        WHERE name IN ('admin', 'operator', 'respondent')
        ORDER BY id
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("adversarial role IDs should be readable");
    assert_eq!(
        ordered_roles
            .iter()
            .map(|(_, name)| name.as_str())
            .collect::<Vec<_>>(),
        vec!["respondent", "operator", "admin"],
        "the lock-order fixture must invert the declared seed order"
    );

    // Match the assignment writer exactly: acquire FOR KEY SHARE role locks
    // one at a time in UUID order. Holding the first lock forces startup into
    // a lock wait. An implementation that then takes seed locks in name order
    // forms respondent->operator and admin->respondent edges and deadlocks;
    // UUID-ordered seed locking waits without retaining an inverted role lock.
    let mut assignment_writer = pool
        .begin()
        .await
        .expect("assignment writer transaction should begin");
    lock_role_for_assignment(&mut assignment_writer, ordered_roles[0].0).await;

    let startup_config = config.clone();
    let startup = tokio::spawn(async move { db::connect_and_prepare(&startup_config).await });
    wait_for_startup_role_lock_wait(&pool).await;

    for (role_id, _) in ordered_roles.iter().skip(1) {
        lock_role_for_assignment(&mut assignment_writer, *role_id).await;
    }
    assignment_writer
        .commit()
        .await
        .expect("assignment writer lock transaction should commit without deadlock");

    let seeded = tokio::time::timeout(Duration::from_secs(10), startup)
        .await
        .expect("startup should finish promptly after assignment locks release")
        .expect("startup task should not panic")
        .expect("seed synchronization should not deadlock with assignment role locks");
    assert_seed_role_updates(&seeded).await;
    seeded.close().await;
    pool.close().await;
}

async fn lock_role_for_assignment(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, role_id: Uuid) {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM roles WHERE id = $1 FOR KEY SHARE")
        .bind(role_id)
        .fetch_one(&mut **tx)
        .await
        .unwrap_or_else(|error| panic!("assignment role lock {role_id} should succeed: {error}"));
}

async fn wait_for_startup_role_lock_wait(pool: &PgPool) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let startup_is_waiting: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM pg_stat_activity
                    WHERE datname = current_database()
                      AND pid <> pg_backend_pid()
                      AND wait_event_type = 'Lock'
                      AND query ILIKE '%roles%'
                      AND query ILIKE '%FOR UPDATE%'
                )
                "#,
            )
            .fetch_one(pool)
            .await
            .expect("startup lock wait should be observable");
            if startup_is_waiting {
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("startup should reach the deliberately blocked seed role lock");
}

fn assert_declared_current_seed_contract() {
    assert_eq!(
        db::BUILT_IN_ROLE_CAPABILITY_SEED_VERSION,
        CURRENT_SEED_VERSION,
        "changing built-in membership requires an intentional version and test-log update"
    );
    assert_eq!(
        db::BUILT_IN_ROLE_CAPABILITY_SEED_SHA256,
        CURRENT_SEED_SHA256,
        "changing built-in membership requires an intentional digest and test-log update"
    );
    let actual_digest = sha256_hex(&db::built_in_role_capability_seed_canonical_bytes());
    assert_eq!(actual_digest, CURRENT_SEED_SHA256);
}

fn assert_populated_sprint_5a_fixture_identity() {
    assert_eq!(
        sha256_hex(POPULATED_SPRINT_5A.as_bytes()),
        POPULATED_SPRINT_5A_SHA256,
        "the populated Sprint 5A fixture is durable test evidence; review every fixture change and update its pinned digest intentionally"
    );
}

async fn assert_pre_sprint_6a_table_inventory(pool: &PgPool) {
    let actual_public_tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename",
    )
    .fetch_all(pool)
    .await
    .expect("Sprint 5A public table inventory should be readable");
    let mut expected_public_tables = PRE_SPRINT_6A_DURABLE_TABLE_COUNTS
        .iter()
        .map(|(table, _)| (*table).to_string())
        .chain(std::iter::once("_sqlx_migrations".to_string()))
        .collect::<Vec<_>>();
    expected_public_tables.sort();
    assert_eq!(
        actual_public_tables, expected_public_tables,
        "every pre-Sprint-6A public table must be classified as durable or as the separately asserted migration ledger"
    );

    for &(table, expected_count) in PRE_SPRINT_6A_DURABLE_TABLE_COUNTS {
        assert!(
            expected_count > 0,
            "the literal fixture contract must keep {table} non-empty"
        );
        assert_eq!(
            count(pool, table).await,
            expected_count,
            "the pinned Sprint 5A fixture row count for {table} changed"
        );
    }

    let actual_analytics_tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'analytics' ORDER BY tablename",
    )
    .fetch_all(pool)
    .await
    .expect("Sprint 5A analytics table inventory should be readable");
    assert_eq!(
        actual_analytics_tables,
        DERIVED_ANALYTICS_TABLES
            .iter()
            .map(|table| (*table).to_string())
            .collect::<Vec<_>>(),
        "every analytics table must remain explicitly classified as a rebuildable projection"
    );
    for table in DERIVED_ANALYTICS_TABLES {
        assert_eq!(
            count(pool, &format!("analytics.{table}")).await,
            0,
            "the populated-upgrade fixture intentionally excludes rebuildable analytics projection {table}"
        );
    }

    let dataset_materialized_relations: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'dataset_materialized'
        ORDER BY table_name
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("physical dataset output inventory should be readable");
    assert!(
        dataset_materialized_relations.is_empty(),
        "physical dataset outputs are rebuildable derivatives and intentionally excluded from the durable preservation snapshot"
    );
    assert_eq!(
        count(pool, "_sqlx_migrations").await,
        2,
        "the migration ledger is system-owned and must contain exactly migrations 1 and 2 before upgrade"
    );
}

async fn assert_sprint_5a_seed_precondition(pool: &PgPool) {
    let capabilities: Vec<(String, String)> =
        sqlx::query_as("SELECT key, description FROM capabilities ORDER BY key")
            .fetch_all(pool)
            .await
            .expect("Sprint 5A capability catalog should be readable");
    let expected_capabilities = SPRINT_5A_CAPABILITIES
        .iter()
        .map(|(key, description)| ((*key).to_string(), (*description).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(
        capabilities, expected_capabilities,
        "the pre-upgrade fixture must reproduce the complete Sprint 5A startup capability catalog"
    );

    let built_in_mappings = seeded_role_capability_snapshot(pool).await;
    let mut expected_built_in_mappings = SPRINT_5A_CAPABILITIES
        .iter()
        .map(|(key, _)| ("admin".to_string(), (*key).to_string()))
        .chain(
            SPRINT_5A_OPERATOR_CAPABILITIES
                .iter()
                .map(|key| ("operator".to_string(), (*key).to_string())),
        )
        .chain(
            SPRINT_5A_RESPONDENT_CAPABILITIES
                .iter()
                .map(|key| ("respondent".to_string(), (*key).to_string())),
        )
        .collect::<Vec<_>>();
    expected_built_in_mappings.sort();
    assert_eq!(
        built_in_mappings, expected_built_in_mappings,
        "the pre-upgrade fixture must reproduce Sprint 5A admin-all, operator-10, and respondent-2 memberships exactly"
    );
    let historical_digest =
        sha256_hex(&role_capability_mapping_canonical_bytes(&built_in_mappings));
    assert_eq!(
        SPRINT_5A_SEED_VERSION,
        "sprint-5a-role-capabilities-v1+sha256.7725e889996a"
    );
    assert_eq!(historical_digest, SPRINT_5A_SEED_SHA256);
    assert!(
        SPRINT_5A_SEED_VERSION.ends_with(&format!("+sha256.{}", &historical_digest[..12])),
        "historical seed version must remain coupled to its canonical mapping digest"
    );

    let custom_mappings: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT roles.name, capabilities.key
        FROM role_capabilities
        JOIN roles ON roles.id = role_capabilities.role_id
        JOIN capabilities ON capabilities.id = role_capabilities.capability_id
        WHERE roles.name = 'existing-custom-role'
        ORDER BY capabilities.key
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("user-managed fixture membership should be readable");
    assert_eq!(
        custom_mappings,
        vec![("existing-custom-role".to_string(), "forms:read".to_string())],
        "the independent user-managed membership is an exact upgrade invariant"
    );
}

fn role_capability_mapping_canonical_bytes(mappings: &[(String, String)]) -> Vec<u8> {
    let mut canonical = String::new();
    let mut previous_role: Option<&str> = None;
    for (role, capability) in mappings {
        if previous_role != Some(role.as_str()) {
            canonical.push_str("role=");
            canonical.push_str(role);
            canonical.push('\n');
            previous_role = Some(role.as_str());
        }
        canonical.push_str("capability=");
        canonical.push_str(capability);
        canonical.push('\n');
    }
    canonical.into_bytes()
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn assert_control_plane_shape(pool: &PgPool) {
    let counts = BTreeMap::from([
        (
            "installations",
            count(pool, "application_installations").await,
        ),
        (
            "reservations",
            count(pool, "module_definition_reservations").await,
        ),
        (
            "sources",
            count(pool, "transition_descriptor_sources").await,
        ),
        (
            "projections",
            count(pool, "transition_catalog_projections").await,
        ),
        ("current", count(pool, "transition_catalog_current").await),
        (
            "navigation_contributions",
            count(pool, "module_navigation_contributions").await,
        ),
        ("policies", count(pool, "navigation_policies").await),
        (
            "policy_entries",
            count(pool, "navigation_policy_entries").await,
        ),
        ("groups", count(pool, "navigation_groups").await),
        (
            "placements",
            count(pool, "navigation_destination_placements").await,
        ),
        (
            "sync_audits",
            count(pool, "core_control_plane_audit_events").await,
        ),
    ]);
    assert_eq!(counts["installations"], 1);
    assert_eq!(counts["reservations"], 7);
    assert_eq!(counts["sources"], 7);
    assert_eq!(counts["projections"], 7);
    assert_eq!(counts["current"], 7);
    assert_eq!(counts["navigation_contributions"], 6);
    assert_eq!(counts["policies"], 1);
    assert_eq!(counts["policy_entries"], 6);
    assert_eq!(counts["groups"], 2);
    assert_eq!(counts["placements"], 13);
    assert_eq!(counts["sync_audits"], 2);

    let module_capabilities: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT key, scope_mode
        FROM capabilities
        WHERE key IN ('modules:read', 'modules:manage_navigation')
        ORDER BY key
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("module capabilities should be readable");
    assert_eq!(
        module_capabilities,
        vec![
            (
                "modules:manage_navigation".to_string(),
                "installation_global".to_string(),
            ),
            (
                "modules:read".to_string(),
                "installation_global".to_string(),
            ),
        ]
    );

    for table in ["module_releases", "module_instances"] {
        let exists: bool = sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(format!("public.{table}"))
            .fetch_one(pool)
            .await
            .expect("table existence should be queryable");
        assert!(!exists, "Sprint 6A must not persist {table}");
    }
}

async fn applied_migrations(pool: &PgPool) -> Vec<i64> {
    sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE success ORDER BY version")
        .fetch_all(pool)
        .await
        .expect("migration ledger should be readable")
}

async fn migration_ledger_snapshot(pool: &PgPool, through_version: i64) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT to_jsonb(m)::text FROM _sqlx_migrations m WHERE version <= $1 ORDER BY version",
    )
    .bind(through_version)
    .fetch_all(pool)
    .await
    .expect("migration ledger snapshot should be readable")
}

async fn product_counts(pool: &PgPool) -> BTreeMap<String, i64> {
    let mut counts = BTreeMap::new();
    for table in PRODUCT_TABLES {
        counts.insert((*table).to_string(), count(pool, table).await);
    }
    counts
}

async fn count(pool: &PgPool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| panic!("{table} should be countable: {error}"))
}

async fn fixture_snapshot(pool: &PgPool) -> Vec<String> {
    // This preservation snapshot covers every durable public table that
    // existed in Sprint 5A. Most tables compare every column of every row.
    // Four startup-owned surfaces require narrow, explicit normalization:
    //
    // - the development administrator credential is salted and intentionally
    //   replaced on each startup; all user-managed credentials remain exact;
    // - migration 3 adds capability.scope_mode and two module capabilities,
    //   while every pre-6A capability column remains exact;
    // - built-in role membership is versioned seed data proved separately,
    //   while every user-managed role_capability row remains exact;
    // - auth_sessions.last_seen_at is activity metadata changed by the real
    //   session HTTP proof, while every other column of every session is exact.
    let mut queries = FULL_ROW_PRESERVATION_TABLES
        .iter()
        .map(|table| {
            (
                (*table).to_string(),
                format!("SELECT to_jsonb(t)::text FROM {table} t"),
            )
        })
        .collect::<Vec<_>>();
    queries.extend([
        (
            "account_credentials".to_string(),
            "SELECT to_jsonb(t)::text FROM account_credentials t WHERE account_id <> '60000000-0000-0000-0000-000000000001'::uuid".to_string(),
        ),
        (
            "capabilities".to_string(),
            "SELECT to_jsonb(t)::text FROM (SELECT id, key, description FROM capabilities WHERE key NOT IN ('modules:read', 'modules:manage_navigation')) t".to_string(),
        ),
        (
            "role_capabilities".to_string(),
            "SELECT to_jsonb(rc)::text FROM role_capabilities rc JOIN roles r ON r.id = rc.role_id WHERE r.name NOT IN ('admin', 'operator', 'respondent')".to_string(),
        ),
        (
            "auth_sessions".to_string(),
            "SELECT to_jsonb(t)::text FROM (SELECT token, account_id, created_at, expires_at, revoked_at FROM auth_sessions) t".to_string(),
        ),
    ]);

    let expected_tables = PRE_SPRINT_6A_DURABLE_TABLE_COUNTS
        .iter()
        .map(|(table, _)| *table)
        .collect::<BTreeSet<_>>();
    let snapshotted_tables = queries
        .iter()
        .map(|(table, _)| table.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        queries.len(),
        expected_tables.len(),
        "each durable pre-Sprint-6A table must have exactly one preservation query"
    );
    assert_eq!(
        snapshotted_tables, expected_tables,
        "every durable pre-Sprint-6A table must have exactly one preservation snapshot query"
    );

    let borrowed = queries
        .iter()
        .map(|(label, query)| (label.as_str(), query.as_str()))
        .collect::<Vec<_>>();
    snapshot(pool, &borrowed).await
}
async fn seeded_role_capability_snapshot(pool: &PgPool) -> Vec<(String, String)> {
    sqlx::query_as(
        r#"
        SELECT roles.name, capabilities.key
        FROM role_capabilities
        JOIN roles ON roles.id = role_capabilities.role_id
        JOIN capabilities ON capabilities.id = role_capabilities.capability_id
        WHERE roles.name IN ('admin', 'operator', 'respondent')
        ORDER BY roles.name, capabilities.key
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("seeded role capability mappings should be readable")
}

async fn assert_seed_role_updates(upgraded_pool: &PgPool) {
    let mappings_after = seeded_role_capability_snapshot(upgraded_pool).await;
    assert_eq!(
        mappings_after,
        expected_current_seed_role_capabilities(),
        "versioned seed roles may update, but only to the declared current seed contract"
    );
}

fn expected_current_seed_role_capabilities() -> Vec<(String, String)> {
    let mut expected = db::BUILT_IN_ROLE_CAPABILITY_SEED
        .iter()
        .flat_map(|(role, capabilities)| {
            capabilities
                .iter()
                .map(move |capability| ((*role).to_string(), (*capability).to_string()))
        })
        .collect::<Vec<_>>();
    expected.sort();
    expected
}

async fn control_plane_snapshot(pool: &PgPool) -> Vec<String> {
    let mut queries = CONTROL_PLANE_TABLES
        .iter()
        .map(|table| (*table, format!("SELECT to_jsonb(t)::text FROM {table} t")))
        .collect::<Vec<_>>();
    queries.push((
        "module_capabilities",
        "SELECT to_jsonb(t)::text FROM capabilities t WHERE key IN ('admin:all', 'modules:read', 'modules:manage_navigation')".to_string(),
    ));
    queries.push((
        "module_role_capabilities",
        "SELECT to_jsonb(rc)::text FROM role_capabilities rc JOIN capabilities c ON c.id = rc.capability_id WHERE c.key IN ('modules:read', 'modules:manage_navigation')".to_string(),
    ));
    let borrowed = queries
        .iter()
        .map(|(label, query)| (*label, query.as_str()))
        .collect::<Vec<_>>();
    snapshot(pool, &borrowed).await
}

async fn snapshot(pool: &PgPool, queries: &[(&str, &str)]) -> Vec<String> {
    let mut result = Vec::new();
    for (label, query) in queries {
        let rows: Vec<String> = sqlx::query_scalar(query)
            .fetch_all(pool)
            .await
            .unwrap_or_else(|error| panic!("{label} snapshot should be readable: {error}"));
        result.extend(rows.into_iter().map(|row| format!("{label}|{row}")));
    }
    result.sort();
    result
}

async fn reset_database(pool: &PgPool) {
    assert_destructive_upgrade_reset_acknowledged();
    assert_disposable_proof_database(pool).await;

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename",
    )
    .fetch_all(pool)
    .await
    .expect("public tables should be listable");
    for table in tables {
        let quoted = table.replace('"', "\"\"");
        sqlx::query(&format!("DROP TABLE IF EXISTS public.\"{quoted}\" CASCADE"))
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("{table} should be droppable: {error}"));
    }
    for schema in ["analytics", "dataset_materialized"] {
        sqlx::query(&format!("DROP SCHEMA IF EXISTS {schema} CASCADE"))
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("{schema} should be droppable: {error}"));
    }
    for type_name in [
        "field_type",
        "form_version_status",
        "submission_status",
        "dataset_revision_status",
        "component_type",
        "component_version_status",
        "missing_data_policy",
    ] {
        sqlx::query(&format!("DROP TYPE IF EXISTS {type_name} CASCADE"))
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("{type_name} should be droppable: {error}"));
    }
}

async fn assert_disposable_proof_database(pool: &PgPool) {
    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .expect("current database should be readable");
    assert!(
        is_disposable_database_name(&database_name),
        "Sprint 6A destructive proof URLs must point at a database with a token-bounded disposable name marker before reset; got '{database_name}'"
    );
}

fn assert_destructive_upgrade_reset_acknowledged() {
    let acknowledgement = std::env::var(DESTRUCTIVE_RESET_ACKNOWLEDGEMENT_ENV).unwrap_or_default();
    assert_eq!(
        acknowledgement, DESTRUCTIVE_RESET_ACKNOWLEDGEMENT,
        "{DESTRUCTIVE_RESET_ACKNOWLEDGEMENT_ENV} must equal '{DESTRUCTIVE_RESET_ACKNOWLEDGEMENT}' before the populated upgrade proof may reset its database"
    );
}

async fn assert_required_proof_databases_are_pairwise_distinct() {
    let mut database_names = BTreeMap::<String, String>::new();
    let mut configured_names = BTreeSet::new();

    for database_url_env in REQUIRED_DATABASE_URL_ENVS {
        let database_url = std::env::var(database_url_env).unwrap_or_else(|_| {
            panic!("{database_url_env} is required for every destructive Sprint 6A database proof")
        });
        assert!(
            !database_url.trim().is_empty(),
            "{database_url_env} must not be empty"
        );
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .unwrap_or_else(|error| panic!("{database_url_env} should be reachable: {error}"));
        let database_name: String = sqlx::query_scalar("SELECT current_database()")
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|error| {
                panic!("{database_url_env} database identity should be readable: {error}")
            });
        assert!(
            is_disposable_database_name(&database_name),
            "{database_url_env} must resolve to a token-bounded disposable database; got '{database_name}'"
        );
        if let Some(first_env) =
            database_names.insert(database_name.clone(), database_url_env.into())
        {
            panic!(
                "{first_env} and {database_url_env} both resolve to database '{database_name}'; all three proof databases must be distinct"
            );
        }
        configured_names.insert(database_name);
        pool.close().await;
    }

    assert_eq!(
        configured_names.len(),
        REQUIRED_DATABASE_URL_ENVS.len(),
        "Sprint 6A proof databases must be pairwise distinct"
    );
}

struct PreControlPlaneMigrations {
    path: PathBuf,
}

impl PreControlPlaneMigrations {
    fn create() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tessara-sprint-6a-pre-control-plane-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        fs::create_dir(&path).expect("temporary migration directory should be creatable");
        fs::write(path.join("001_baseline.sql"), BASELINE)
            .expect("baseline migration should be writable");
        fs::write(
            path.join("002_dashboard_placement_capacity.sql"),
            DASHBOARD_CAPACITY,
        )
        .expect("dashboard capacity migration should be writable");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for PreControlPlaneMigrations {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct WorkspaceWorkingDirectory {
    previous: PathBuf,
}

impl WorkspaceWorkingDirectory {
    fn enter() -> Self {
        let previous = std::env::current_dir().expect("current directory should be readable");
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("API crate should be inside the workspace")
            .to_path_buf();
        std::env::set_current_dir(&workspace)
            .expect("upgrade proof should enter the workspace root");
        Self { previous }
    }
}

impl Drop for WorkspaceWorkingDirectory {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}
