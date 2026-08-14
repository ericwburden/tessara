use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tessara_dataset_module::DatasetModuleError;
use tessara_dataset_module::dependency_dag::{
    rebuild_affected_published_closure_in_transaction, validate_candidate_sources_in_transaction,
};
use tessara_datasets_contract::{DatasetProductFieldV1, DatasetProductSourceV1};
use uuid::Uuid;

const DIGEST: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Clone, Copy)]
struct DatasetIds {
    dataset_id: Uuid,
    revision_id: Uuid,
}

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn revision_table(revision_id: Uuid) -> String {
    format!("dataset_{}", revision_id.simple())
}

fn major_table(dataset_id: Uuid, major: i32) -> String {
    format!("dataset_major_{}_v{major}", dataset_id.simple())
}

fn qualified_table(table: &str) -> String {
    format!("\"dataset_materialized\".\"{table}\"")
}

fn static_sql(row_id: &str, value: &str) -> String {
    format!(
        "SELECT '{row_id}'::text AS __row_id, 'public'::text AS __restriction_tier, \
         ARRAY['{}'::uuid]::uuid[] AS __scope_node_ids, '{value}'::text AS \"value\"",
        id(900)
    )
}

fn derived_revision_sql(source_revision_id: Uuid, suffix: &str) -> String {
    format!(
        "SELECT concat('{suffix}:', __row_id)::text AS __row_id, \
         __restriction_tier, __scope_node_ids, concat(\"value\", '-{suffix}')::text AS \"value\" \
         FROM {}",
        qualified_table(&revision_table(source_revision_id))
    )
}

fn derived_major_sql(source_dataset_id: Uuid, source_major: i32, suffix: &str) -> String {
    format!(
        "SELECT concat('{suffix}:', __row_id)::text AS __row_id, \
         __restriction_tier, __scope_node_ids, concat(\"value\", '-{suffix}')::text AS \"value\" \
         FROM {}",
        qualified_table(&major_table(source_dataset_id, source_major))
    )
}

async fn insert_published_dataset(pool: &PgPool, ids: DatasetIds, name: &str, generated_sql: &str) {
    let slug = name.to_ascii_lowercase().replace(' ', "-");
    sqlx::query("INSERT INTO datasets(id,name,slug,grain) VALUES($1,$2,$3,'submission')")
        .bind(ids.dataset_id)
        .bind(name)
        .bind(slug)
        .execute(pool)
        .await
        .unwrap();
    let fields = vec![DatasetProductFieldV1 {
        key: "value".into(),
        label: "Value".into(),
        source_alias: "source".into(),
        source_field_key: "value".into(),
        field_type: "text".into(),
        position: 0,
    }];
    sqlx::query(
        "INSERT INTO dataset_revisions
         (id,dataset_id,version_number,version_label,version_major,version_minor,version_patch,
          semantic_bump,started_new_major_line,status,generated_sql,output_fields,published_at)
         VALUES($1,$2,1,'Version 1',1,0,0,'initial',true,'published',$3,$4,now())",
    )
    .bind(ids.revision_id)
    .bind(ids.dataset_id)
    .bind(generated_sql)
    .bind(serde_json::to_value(fields).unwrap())
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_source_snapshot(
    pool: &PgPool,
    dependent_revision_id: Uuid,
    alias: &str,
    source: DatasetProductSourceV1,
    position: i32,
) {
    let source_kind = match &source {
        DatasetProductSourceV1::Form { .. } => "form_version",
        DatasetProductSourceV1::Dataset { .. } => "dataset_revision",
        DatasetProductSourceV1::DatasetMajor { .. } => "dataset_major_line",
    };
    sqlx::query(
        "INSERT INTO dataset_revision_sources
         (revision_id,source_alias,source_kind,source_reference,source_name,
          source_scope_revision,source_scope_digest,source_content_revision,
          source_content_digest,position)
         VALUES($1,$2,$3,$4,$5,'scope:1',$6,'content:1',$6,$7)",
    )
    .bind(dependent_revision_id)
    .bind(alias)
    .bind(source_kind)
    .bind(serde_json::to_value(source).unwrap())
    .bind(alias)
    .bind(DIGEST)
    .bind(position)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_chain(pool: &PgPool) -> (DatasetIds, DatasetIds, DatasetIds, DatasetIds) {
    let base = DatasetIds {
        dataset_id: id(1),
        revision_id: id(101),
    };
    let derived = DatasetIds {
        dataset_id: id(2),
        revision_id: id(102),
    };
    let second_hop = DatasetIds {
        dataset_id: id(3),
        revision_id: id(103),
    };
    let independent = DatasetIds {
        dataset_id: id(4),
        revision_id: id(104),
    };
    insert_published_dataset(pool, base, "Base", &static_sql("base", "old")).await;
    insert_published_dataset(
        pool,
        derived,
        "Derived",
        &derived_revision_sql(base.revision_id, "derived"),
    )
    .await;
    insert_published_dataset(
        pool,
        second_hop,
        "Second Hop",
        &derived_major_sql(derived.dataset_id, 1, "second"),
    )
    .await;
    insert_published_dataset(
        pool,
        independent,
        "Independent",
        &static_sql("independent", "unchanged"),
    )
    .await;
    insert_source_snapshot(
        pool,
        derived.revision_id,
        "base",
        DatasetProductSourceV1::Dataset {
            alias: "base".into(),
            dataset_id: base.dataset_id.to_string(),
            dataset_revision_id: base.revision_id.to_string(),
        },
        0,
    )
    .await;
    insert_source_snapshot(
        pool,
        second_hop.revision_id,
        "derived",
        DatasetProductSourceV1::DatasetMajor {
            alias: "derived".into(),
            dataset_id: derived.dataset_id.to_string(),
            version_major: 1,
        },
        0,
    )
    .await;
    (base, derived, second_hop, independent)
}

async fn materialized_value(pool: &PgPool, revision_id: Uuid) -> String {
    sqlx::query_scalar(&format!(
        "SELECT \"value\" FROM {}",
        qualified_table(&revision_table(revision_id))
    ))
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn candidate_sources_reject_a_transitive_cycle_before_any_sync_attempt(pool: PgPool) {
    let (base, _, second_hop, _) = insert_chain(&pool).await;
    let candidate_sources = vec![DatasetProductSourceV1::DatasetMajor {
        alias: "second".into(),
        dataset_id: second_hop.dataset_id.to_string(),
        version_major: 1,
    }];
    let mut transaction = pool.begin().await.unwrap();

    let error = validate_candidate_sources_in_transaction(
        &mut transaction,
        base.dataset_id,
        &candidate_sources,
    )
    .await
    .unwrap_err();

    assert!(matches!(error, DatasetModuleError::ValidationFailed(_)));
    let attempts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dataset_sync_attempts")
        .fetch_one(&mut *transaction)
        .await
        .unwrap();
    assert_eq!(attempts, 0);
    transaction.rollback().await.unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn rebuild_promotes_the_full_topological_closure_and_leaves_independent_state_exact(
    pool: PgPool,
) {
    let (base, derived, second_hop, independent) = insert_chain(&pool).await;
    let mut initial = pool.begin().await.unwrap();
    rebuild_affected_published_closure_in_transaction(
        &mut initial,
        &[base.dataset_id, independent.dataset_id],
    )
    .await
    .unwrap();
    initial.commit().await.unwrap();
    assert_eq!(
        materialized_value(&pool, second_hop.revision_id).await,
        "old-derived-second"
    );
    let independent_before: (String, i64, DateTime<Utc>, DateTime<Utc>, i64, i64) = sqlx::query_as(
        "SELECT r.materialized_table,r.materialized_row_count,r.materialized_at,
                m.materialized_at,r.resource_revision,d.resource_revision
         FROM dataset_revisions r
         JOIN datasets d ON d.id=r.dataset_id
         JOIN dataset_major_materializations m
           ON m.dataset_id=r.dataset_id AND m.version_major=r.version_major
         WHERE r.id=$1",
    )
    .bind(independent.revision_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let affected_revision_before: Vec<(Uuid, i64)> = sqlx::query_as(
        "SELECT id,resource_revision FROM dataset_revisions
         WHERE id = ANY($1) ORDER BY id",
    )
    .bind(vec![
        base.revision_id,
        derived.revision_id,
        second_hop.revision_id,
    ])
    .fetch_all(&pool)
    .await
    .unwrap();

    let mut refresh = pool.begin().await.unwrap();
    sqlx::query("UPDATE dataset_revisions SET generated_sql=$1 WHERE id=$2")
        .bind(static_sql("base", "new"))
        .bind(base.revision_id)
        .execute(&mut *refresh)
        .await
        .unwrap();
    let plan = rebuild_affected_published_closure_in_transaction(&mut refresh, &[base.dataset_id])
        .await
        .unwrap();
    assert_eq!(
        plan.targets
            .iter()
            .map(|target| target.dataset_id)
            .collect::<Vec<_>>(),
        vec![base.dataset_id, derived.dataset_id, second_hop.dataset_id]
    );
    refresh.commit().await.unwrap();

    assert_eq!(materialized_value(&pool, base.revision_id).await, "new");
    assert_eq!(
        materialized_value(&pool, derived.revision_id).await,
        "new-derived"
    );
    assert_eq!(
        materialized_value(&pool, second_hop.revision_id).await,
        "new-derived-second"
    );
    let independent_after: (String, i64, DateTime<Utc>, DateTime<Utc>, i64, i64) = sqlx::query_as(
        "SELECT r.materialized_table,r.materialized_row_count,r.materialized_at,
                m.materialized_at,r.resource_revision,d.resource_revision
         FROM dataset_revisions r
         JOIN datasets d ON d.id=r.dataset_id
         JOIN dataset_major_materializations m
           ON m.dataset_id=r.dataset_id AND m.version_major=r.version_major
         WHERE r.id=$1",
    )
    .bind(independent.revision_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(independent_after, independent_before);
    let affected_revision_after: Vec<(Uuid, i64)> = sqlx::query_as(
        "SELECT id,resource_revision FROM dataset_revisions
         WHERE id = ANY($1) ORDER BY id",
    )
    .bind(vec![
        base.revision_id,
        derived.revision_id,
        second_hop.revision_id,
    ])
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        affected_revision_after,
        affected_revision_before
            .into_iter()
            .map(|(revision_id, revision)| (revision_id, revision + 1))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        materialized_value(&pool, independent.revision_id).await,
        "unchanged"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn downstream_materialization_failure_rolls_back_every_rebuilt_table(pool: PgPool) {
    let (base, derived, second_hop, _) = insert_chain(&pool).await;
    let mut initial = pool.begin().await.unwrap();
    rebuild_affected_published_closure_in_transaction(&mut initial, &[base.dataset_id])
        .await
        .unwrap();
    initial.commit().await.unwrap();
    let before_base = materialized_value(&pool, base.revision_id).await;
    let before_derived = materialized_value(&pool, derived.revision_id).await;
    let before_second_hop = materialized_value(&pool, second_hop.revision_id).await;
    let metadata_before: Vec<(Uuid, i64, DateTime<Utc>, i64, i64)> = sqlx::query_as(
        "SELECT r.id,r.materialized_row_count,r.materialized_at,
                r.resource_revision,d.resource_revision
         FROM dataset_revisions r JOIN datasets d ON d.id=r.dataset_id
         WHERE r.id = ANY($1) ORDER BY r.id",
    )
    .bind(vec![
        base.revision_id,
        derived.revision_id,
        second_hop.revision_id,
    ])
    .fetch_all(&pool)
    .await
    .unwrap();

    let mut failed = pool.begin().await.unwrap();
    sqlx::query("UPDATE dataset_revisions SET generated_sql=$1 WHERE id=$2")
        .bind(static_sql("base", "must-not-promote"))
        .bind(base.revision_id)
        .execute(&mut *failed)
        .await
        .unwrap();
    sqlx::query("UPDATE dataset_revisions SET generated_sql=$1 WHERE id=$2")
        .bind("SELECT * FROM dataset_materialized.missing_rebuild_input")
        .bind(derived.revision_id)
        .execute(&mut *failed)
        .await
        .unwrap();
    let error = rebuild_affected_published_closure_in_transaction(&mut failed, &[base.dataset_id])
        .await
        .unwrap_err();
    assert!(matches!(error, DatasetModuleError::Database(_)));
    failed.rollback().await.unwrap();

    assert_eq!(
        materialized_value(&pool, base.revision_id).await,
        before_base
    );
    assert_eq!(
        materialized_value(&pool, derived.revision_id).await,
        before_derived
    );
    assert_eq!(
        materialized_value(&pool, second_hop.revision_id).await,
        before_second_hop
    );
    let metadata_after: Vec<(Uuid, i64, DateTime<Utc>, i64, i64)> = sqlx::query_as(
        "SELECT r.id,r.materialized_row_count,r.materialized_at,
                r.resource_revision,d.resource_revision
         FROM dataset_revisions r JOIN datasets d ON d.id=r.dataset_id
         WHERE r.id = ANY($1) ORDER BY r.id",
    )
    .bind(vec![
        base.revision_id,
        derived.revision_id,
        second_hop.revision_id,
    ])
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(metadata_after, metadata_before);
    let base_sql: String =
        sqlx::query_scalar("SELECT generated_sql FROM dataset_revisions WHERE id=$1")
            .bind(base.revision_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(base_sql, static_sql("base", "old"));
}
