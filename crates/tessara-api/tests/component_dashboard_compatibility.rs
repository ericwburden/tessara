#[allow(dead_code)]
mod support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{
    TEST_DATABASE_LOCK, authorized_request, login_token, request_json, request_status_and_json,
    test_app,
};
use uuid::Uuid;

const COMPATIBILITY_ERROR: &str =
    "This Component update would make a pinned Dashboard layout undisplayable.";

#[tokio::test]
async fn published_kind_update_preserves_every_pinned_dashboard_layout_atomically() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let seed_version_id = seed["component_version_id"]
        .as_str()
        .expect("seeded Component version id")
        .parse::<Uuid>()
        .expect("seeded Component version UUID");
    let pool = test_pool().await;
    let (dataset_id, dataset_version_major): (Uuid, i32) = sqlx::query_as(
        r#"
        SELECT dataset_id, dataset_version_major
        FROM component_versions
        WHERE id = $1
        "#,
    )
    .bind(seed_version_id)
    .fetch_one(&pool)
    .await
    .expect("seeded Component Dataset binding");

    let (_, occupier_version_id) =
        insert_published_stat_card(&pool, dataset_id, dataset_version_major, "row-occupier").await;
    let (target_component_id, target_version_id) = insert_published_stat_card(
        &pool,
        dataset_id,
        dataset_version_major,
        "kind-update-target",
    )
    .await;
    let dashboard_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO dashboards (name, description)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind("Published Component compatibility guard")
    .bind("Exercises kind-aware placement validation for mutable published payloads.")
    .fetch_one(&pool)
    .await
    .expect("compatibility Dashboard fixture");
    sqlx::query(
        r#"
        INSERT INTO dashboard_components
            (dashboard_id, component_version_id, position, config)
        VALUES
            ($1, $2, 0, $4),
            ($1, $3, 1, $5)
        "#,
    )
    .bind(dashboard_id)
    .bind(occupier_version_id)
    .bind(target_version_id)
    .bind(placement_config(1, 1, 1, 240))
    .bind(placement_config(1, 2, 1, 1))
    .execute(&pool)
    .await
    .expect("pinned placement fixtures");

    let update_uri = format!(
        "/api/admin/components/{target_component_id}/versions/{target_version_id}/published"
    );
    let update_payload = json!({
        "dataset_id": dataset_id,
        "dataset_version_major": dataset_version_major,
        "component_type": "table",
        "version_note": "Candidate Table payload.",
        "config": {}
    });
    let version_before = load_version_payload(&pool, target_version_id).await;
    let (rejected_status, rejected_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PATCH",
            &update_uri,
            &admin_token,
            Some(update_payload.clone()),
        ),
    )
    .await;

    assert_eq!(rejected_status, StatusCode::BAD_REQUEST);
    assert_eq!(rejected_body["code"], "bad_request");
    assert_eq!(rejected_body["message"], COMPATIBILITY_ERROR);
    assert_eq!(
        load_version_payload(&pool, target_version_id).await,
        version_before,
        "the rejected candidate payload must roll back in full"
    );

    sqlx::query(
        r#"
        UPDATE dashboard_components
        SET config = $2
        WHERE dashboard_id = $1
          AND component_version_id = $3
        "#,
    )
    .bind(dashboard_id)
    .bind(placement_config(1, 2, 6, 4))
    .bind(target_version_id)
    .execute(&pool)
    .await
    .expect("make the target placement Table-compatible");

    let accepted = request_json(
        app,
        authorized_request("PATCH", &update_uri, &admin_token, Some(update_payload)),
    )
    .await;
    assert_eq!(accepted["id"], target_version_id.to_string());
    let version_after = load_version_payload(&pool, target_version_id).await;
    assert_eq!(version_after.0, "table");
    assert_eq!(version_after.1, json!({}));
    assert_eq!(version_after.2, "Candidate Table payload.");
}

async fn insert_published_stat_card(
    pool: &PgPool,
    dataset_id: Uuid,
    dataset_version_major: i32,
    fixture_name: &str,
) -> (Uuid, Uuid) {
    let component_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO components (name, slug) VALUES ($1, $2) RETURNING id",
    )
    .bind(format!("Compatibility fixture {fixture_name}"))
    .bind(format!("compatibility-{fixture_name}-{}", Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("compatibility Component fixture");
    let version_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO component_versions (
            component_id,
            dataset_id,
            dataset_version_major,
            component_type,
            version_number,
            version_label,
            status,
            config,
            published_at
        )
        VALUES (
            $1,
            $2,
            $3,
            'stat_card'::component_type,
            1,
            '1',
            'published'::component_version_status,
            $4,
            now()
        )
        RETURNING id
        "#,
    )
    .bind(component_id)
    .bind(dataset_id)
    .bind(dataset_version_major)
    .bind(json!({
        "summary_field": "",
        "summary_type": "row_count"
    }))
    .fetch_one(pool)
    .await
    .expect("published Stat Card fixture");
    (component_id, version_id)
}

fn placement_config(row: i32, column: i32, width: i32, height: i32) -> Value {
    json!({
        "schema_version": 1,
        "grid_row": row,
        "grid_column": column,
        "grid_width": width,
        "grid_height": height
    })
}

async fn load_version_payload(pool: &PgPool, version_id: Uuid) -> (String, Value, String) {
    sqlx::query_as(
        r#"
        SELECT component_type::text, config, version_note
        FROM component_versions
        WHERE id = $1
        "#,
    )
    .bind(version_id)
    .fetch_one(pool)
    .await
    .expect("Component version payload")
}

async fn test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL").expect("test database url");
    PgPool::connect(&database_url)
        .await
        .expect("test database connection")
}
