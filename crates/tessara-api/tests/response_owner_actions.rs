#[path = "response_owner_actions/support.rs"]
mod support;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::Row;
use tessara_module_contract::ProtocolSignaturePurposeV1;
use tessara_responses_contract::{
    RESPONSE_OWNER_ACTION_IDEMPOTENCY_HEADER, RESPONSE_OWNER_ACTION_MEDIA_TYPE,
    RESPONSE_OWNER_ACTION_PATH, ResponseOwnerActionResponse, SubmittedResponseChange,
};
use tokio::time::{Duration, sleep, timeout};
use uuid::Uuid;

use support::{TEST_DATABASE_LOCK, login_token, request_status_and_json, test_state};

#[tokio::test]
async fn owner_actions_are_atomic_replay_safe_and_commit_ordered() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let app = tessara_api::router(state.clone());
    let token = login_token(app.clone()).await;
    let fixture = seed_owner_graph(&state.pool).await;

    // Fresh-baseline draft INSERT must never evaluate OLD in the trigger and
    // must not publish a submitted-response envelope.
    let draft_id: Uuid = sqlx::query_scalar(
        "INSERT INTO submissions(form_version_id,node_id,workflow_assignment_id,status)
         VALUES($1,$2,$3,'draft'::submission_status) RETURNING id",
    )
    .bind(fixture.form_version_id)
    .bind(fixture.node_id)
    .bind(fixture.workflow_assignment_id)
    .fetch_one(&state.pool)
    .await
    .expect("draft INSERT must not read OLD");
    let draft_change_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM response_export_changes WHERE response_id=$1")
            .bind(draft_id)
            .fetch_one(&state.pool)
            .await
            .unwrap();
    assert_eq!(draft_change_count, 0);

    let create_body = json!({
        "schema_version": 1,
        "action": "create",
        "logical_key": "response.new",
        "form_version_id": fixture.form_version_id,
        "node_id": fixture.node_id,
        "values": {"answer": "created", "labels": ["a", "b"], "optional": null}
    });
    let (status, created) = owner_request(
        app.clone(),
        &token,
        "response-create-1",
        create_body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{created}");
    assert_eq!(created["replayed"], false);
    assert_eq!(
        created["signed_receipt"]["purpose"],
        "response_owner_action_receipt"
    );
    assert_eq!(created["signed_receipt"]["payload"]["action"], "create");
    assert_eq!(
        created["signed_receipt"]["payload"]["export"]["change_kind"],
        "upsert"
    );
    let response_id = uuid_at(&created, "/signed_receipt/payload/response_id");
    let mut sequences = vec![u64_at(
        &created,
        "/signed_receipt/payload/export/change_sequence",
    )];
    verify_signed_receipt(&created);
    assert_one_final_change(&state.pool, response_id, sequences[0], "upsert", None).await;
    assert_export_integrity(
        &state.pool,
        sequences[0],
        created["signed_receipt"]["payload"]["export"]["content_digest"].as_str(),
    )
    .await;

    let (status, replay) = owner_request(
        app.clone(),
        &token,
        "response-create-1",
        create_body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay["replayed"], true);
    assert_eq!(replay["signed_receipt"], created["signed_receipt"]);
    assert_one_final_change(&state.pool, response_id, sequences[0], "upsert", None).await;

    let mut mismatch_body = create_body;
    mismatch_body["values"]["answer"] = json!("different");
    let (status, mismatch) =
        owner_request(app.clone(), &token, "response-create-1", mismatch_body).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        mismatch["error"]["code"],
        "response_owner.idempotency_mismatch"
    );

    let correction = json!({
        "schema_version": 1,
        "action": "correct",
        "logical_key": "response.corrected",
        "response_id": response_id,
        "values": {"answer": "corrected", "labels": ["c"], "optional": "now-present"}
    });
    sequences.push(
        success_sequence(
            &state.pool,
            app.clone(),
            &token,
            "response-correct-1",
            correction,
            ("correct", "upsert", None),
        )
        .await,
    );
    sequences.push(
        success_sequence(
            &state.pool,
            app.clone(),
            &token,
            "response-status-out-1",
            json!({
                "schema_version":1,"action":"status_out","logical_key":"response.status-out",
                "response_id":response_id
            }),
            ("status_out", "tombstone", Some("status_excluded")),
        )
        .await,
    );
    sequences.push(
        success_sequence(
            &state.pool,
            app.clone(),
            &token,
            "response-status-in-1",
            json!({
                "schema_version":1,"action":"status_in","logical_key":"response.status-in",
                "response_id":response_id
            }),
            ("status_in", "upsert", None),
        )
        .await,
    );
    sequences.push(
        success_sequence(
            &state.pool,
            app.clone(),
            &token,
            "response-redact-1",
            json!({
                "schema_version":1,"action":"redact","logical_key":"response.redacted",
                "response_id":response_id
            }),
            ("redact", "tombstone", Some("redacted")),
        )
        .await,
    );

    let redacted_value_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM submission_values WHERE submission_id=$1")
            .bind(response_id)
            .fetch_one(&state.pool)
            .await
            .unwrap();
    assert_eq!(redacted_value_count, 0);

    let delete_target = create_response(
        &state.pool,
        app.clone(),
        &token,
        &fixture,
        "response.deleted",
        "response-create-delete-1",
    )
    .await;
    sequences.push(
        success_sequence(
            &state.pool,
            app,
            &token,
            "response-delete-1",
            json!({
                "schema_version":1,"action":"delete","logical_key":"response.deleted",
                "response_id":delete_target.0
            }),
            ("delete", "tombstone", Some("deleted")),
        )
        .await,
    );
    assert!(sequences.windows(2).all(|pair| pair[0] < pair[1]));
    let target_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM submissions WHERE id=$1)")
            .bind(delete_target.0)
            .fetch_one(&state.pool)
            .await
            .unwrap();
    assert!(!target_exists);

    let before_failed_sequence: i64 =
        sqlx::query_scalar("SELECT next_sequence FROM response_export_state WHERE singleton")
            .fetch_one(&state.pool)
            .await
            .unwrap();
    let before_failed_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM response_owner_action_receipts")
            .fetch_one(&state.pool)
            .await
            .unwrap();
    let (status, failed) = owner_request(
        tessara_api::router(state.clone()),
        &token,
        "response-invalid-complete-values",
        json!({
            "schema_version":1,"action":"create","logical_key":"response.invalid",
            "form_version_id":fixture.form_version_id,"node_id":fixture.node_id,
            "values":{"answer":"omits-two-fields"}
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(failed["error"]["code"], "response_owner.validation_failed");
    let after_failed_sequence: i64 =
        sqlx::query_scalar("SELECT next_sequence FROM response_export_state WHERE singleton")
            .fetch_one(&state.pool)
            .await
            .unwrap();
    let after_failed_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM response_owner_action_receipts")
            .fetch_one(&state.pool)
            .await
            .unwrap();
    assert_eq!(after_failed_sequence, before_failed_sequence);
    assert_eq!(after_failed_count, before_failed_count);
}

#[tokio::test]
async fn owner_scope_authorization_serializes_node_reparenting_until_commit() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let app = tessara_api::router(state.clone());
    let token = login_token(app.clone()).await;
    let fixture = seed_owner_graph(&state.pool).await;

    let mut form_blocker = state.pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM form_versions WHERE id=$1 FOR UPDATE")
        .bind(fixture.form_version_id)
        .fetch_one(&mut *form_blocker)
        .await
        .unwrap();

    let owner_fixture = OwnerFixture {
        form_version_id: fixture.form_version_id,
        node_id: fixture.node_id,
        workflow_assignment_id: fixture.workflow_assignment_id,
        workflow_version_id: fixture.workflow_version_id,
        workflow_step_id: fixture.workflow_step_id,
        account_id: fixture.account_id,
    };
    let owner_pool = state.pool.clone();
    let mut owner_task = tokio::spawn(async move {
        create_response(
            &owner_pool,
            app,
            &token,
            &owner_fixture,
            "response.scope-lock",
            "response-scope-lock-1",
        )
        .await
    });

    let mut owner_holds_hierarchy_lock = false;
    for _ in 0..100 {
        owner_holds_hierarchy_lock = sqlx::query_scalar(
            "SELECT EXISTS(
                 SELECT 1
                   FROM pg_locks
                  WHERE relation='nodes'::regclass
                    AND mode='ShareLock' AND granted
                    AND pid<>pg_backend_pid()
             )",
        )
        .fetch_one(&mut *form_blocker)
        .await
        .unwrap();
        if owner_holds_hierarchy_lock {
            break;
        }
        sleep(Duration::from_millis(20)).await;
    }
    assert!(
        owner_holds_hierarchy_lock,
        "owner authorization must hold the hierarchy mutation barrier before reading form scope"
    );

    let pool = state.pool.clone();
    let node_id = fixture.node_id;
    let mut reparent_task = tokio::spawn(async move {
        sqlx::query("UPDATE nodes SET parent_node_id=parent_node_id WHERE id=$1")
            .bind(node_id)
            .execute(&pool)
            .await
    });
    assert!(
        timeout(Duration::from_millis(250), &mut reparent_task)
            .await
            .is_err(),
        "concurrent hierarchy mutation must wait for the owner action transaction"
    );

    form_blocker.commit().await.unwrap();
    timeout(Duration::from_secs(5), &mut owner_task)
        .await
        .expect("owner action should finish after the form lock is released")
        .expect("owner action task should not panic");
    timeout(Duration::from_secs(5), &mut reparent_task)
        .await
        .expect("hierarchy mutation should resume after owner commit")
        .expect("reparent task should not panic")
        .expect("reparent mutation should succeed");
}

#[tokio::test]
async fn public_submit_emits_one_final_envelope_and_rolls_back_with_workflow_failure() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let app = tessara_api::router(state.clone());
    let token = login_token(app.clone()).await;
    let fixture = seed_owner_graph(&state.pool).await;
    let draft_id = insert_runtime_draft(&state.pool, &fixture).await;

    let (status, body) = support::request_status_and_json(
        app.clone(),
        support::authorized_request(
            "POST",
            &format!("/api/submissions/{draft_id}/submit"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let rows = sqlx::query(
        "SELECT change_sequence,change_kind,payload
           FROM response_export_changes WHERE response_id=$1 ORDER BY change_sequence",
    )
    .bind(draft_id)
    .fetch_all(&state.pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].try_get::<String, _>("change_kind").unwrap(),
        "upsert"
    );
    let payload: Value = rows[0].try_get("payload").unwrap();
    assert_eq!(payload["last_modified_by_user_name"], "Tessara Admin");
    let sequence = u64::try_from(rows[0].try_get::<i64, _>("change_sequence").unwrap()).unwrap();
    assert_export_integrity(&state.pool, sequence, None).await;

    let broken_id = insert_runtime_draft(&state.pool, &fixture).await;
    let (provider_epoch, before): (Uuid, i64) = sqlx::query_as(
        "SELECT provider_epoch,next_sequence FROM response_export_state WHERE singleton",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap();
    sqlx::query("DELETE FROM response_export_state WHERE singleton")
        .execute(&state.pool)
        .await
        .unwrap();
    let (status, _) = support::request_status_and_json(
        app,
        support::authorized_request(
            "POST",
            &format!("/api/submissions/{broken_id}/submit"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let status_after: String =
        sqlx::query_scalar("SELECT status::text FROM submissions WHERE id=$1")
            .bind(broken_id)
            .fetch_one(&state.pool)
            .await
            .unwrap();
    assert_eq!(status_after, "draft");
    let broken_change_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM response_export_changes WHERE response_id=$1")
            .bind(broken_id)
            .fetch_one(&state.pool)
            .await
            .unwrap();
    assert_eq!(broken_change_count, 0);
    sqlx::query(
        "INSERT INTO response_export_state(singleton,provider_epoch,next_sequence)
         VALUES(true,$1,$2)",
    )
    .bind(provider_epoch)
    .bind(before)
    .execute(&state.pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn raw_and_canonical_tampering_are_rejected_for_trigger_and_owner_rows() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let app = tessara_api::router(state.clone());
    let token = login_token(app.clone()).await;
    let fixture = seed_owner_graph(&state.pool).await;
    let (response_id, sequence) = create_response(
        &state.pool,
        app,
        &token,
        &fixture,
        "response.tamper",
        "response-create-tamper-1",
    )
    .await;
    assert_export_integrity(&state.pool, sequence, None).await;

    let trigger_response_id = insert_runtime_draft(&state.pool, &fixture).await;
    sqlx::query(
        "UPDATE submissions SET status='submitted'::submission_status,submitted_at=now()
          WHERE id=$1",
    )
    .bind(trigger_response_id)
    .execute(&state.pool)
    .await
    .unwrap();
    let trigger_sequence: i64 = sqlx::query_scalar(
        "SELECT change_sequence FROM response_export_changes WHERE response_id=$1",
    )
    .bind(trigger_response_id)
    .fetch_one(&state.pool)
    .await
    .unwrap();
    let trigger_sequence = u64::try_from(trigger_sequence).unwrap();
    assert_export_integrity(&state.pool, trigger_sequence, None).await;

    for (producer, target_sequence) in [
        ("owner action", sequence),
        ("ordinary trigger", trigger_sequence),
    ] {
        let target_sequence = i64::try_from(target_sequence).unwrap();
        let original_digest: String = sqlx::query_scalar(
            "SELECT content_digest FROM response_export_changes WHERE change_sequence=$1",
        )
        .bind(target_sequence)
        .fetch_one(&state.pool)
        .await
        .unwrap();
        sqlx::query(
            "UPDATE response_export_changes SET content_digest=$2 WHERE change_sequence=$1",
        )
        .bind(target_sequence)
        .bind(format!("sha256:{}", "0".repeat(64)))
        .execute(&state.pool)
        .await
        .unwrap();
        assert!(
            read_and_validate_export_row(&state.pool, target_sequence)
                .await
                .is_err_and(is_sanitized_export_rejection),
            "{producer} raw payload/digest tampering must fail closed"
        );

        sqlx::query(
            "UPDATE response_export_changes SET content_digest=$2 WHERE change_sequence=$1",
        )
        .bind(target_sequence)
        .bind(original_digest)
        .execute(&state.pool)
        .await
        .unwrap();
        sqlx::query(
            "UPDATE response_export_changes
                SET payload=payload || '{\"unexpected\":true}'::jsonb
              WHERE change_sequence=$1",
        )
        .bind(target_sequence)
        .execute(&state.pool)
        .await
        .unwrap();
        sqlx::query(
            "UPDATE response_export_changes
                SET content_digest='sha256:' || encode(
                    digest(convert_to(payload::text,'UTF8'),'sha256'),'hex')
              WHERE change_sequence=$1",
        )
        .bind(target_sequence)
        .execute(&state.pool)
        .await
        .unwrap();
        assert!(
            read_and_validate_export_row(&state.pool, target_sequence)
                .await
                .is_err_and(is_sanitized_export_rejection),
            "{producer} canonical-envelope tampering must fail closed even with a matching raw digest"
        );
    }

    assert_ne!(response_id, trigger_response_id);
}

struct OwnerFixture {
    form_version_id: Uuid,
    node_id: Uuid,
    workflow_assignment_id: Uuid,
    workflow_version_id: Uuid,
    workflow_step_id: Uuid,
    account_id: Uuid,
}

async fn seed_owner_graph(pool: &sqlx::PgPool) -> OwnerFixture {
    let account_id: Uuid =
        sqlx::query_scalar("SELECT id FROM accounts WHERE email='admin@tessara.local'")
            .fetch_one(pool)
            .await
            .unwrap();
    let node_type_id: Uuid = sqlx::query_scalar(
        "INSERT INTO node_types(name,slug) VALUES('Response Owner Node','response-owner-node') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let node_id: Uuid = sqlx::query_scalar(
        "INSERT INTO nodes(node_type_id,name) VALUES($1,'Response Owner Scope') RETURNING id",
    )
    .bind(node_type_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let form_id: Uuid = sqlx::query_scalar(
        "INSERT INTO forms(name,slug,scope_node_type_id) VALUES('Response Owner Form','response-owner-form',$1) RETURNING id",
    )
    .bind(node_type_id)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO form_scope_nodes(form_id,node_id) VALUES($1,$2)")
        .bind(form_id)
        .bind(node_id)
        .execute(pool)
        .await
        .unwrap();
    let form_version_id: Uuid = sqlx::query_scalar(
        "INSERT INTO form_versions(form_id,version_label,status,published_at)
         VALUES($1,'1.0.0','published'::form_version_status,now()) RETURNING id",
    )
    .bind(form_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let section_id: Uuid = sqlx::query_scalar(
        "INSERT INTO form_sections(form_version_id,title) VALUES($1,'Main') RETURNING id",
    )
    .bind(form_version_id)
    .fetch_one(pool)
    .await
    .unwrap();
    for (key, kind, required, position) in [
        ("answer", "text", true, 0),
        ("labels", "multi_choice", false, 1),
        ("optional", "text", false, 2),
    ] {
        sqlx::query(
            "INSERT INTO form_fields(form_version_id,section_id,key,label,field_type,required,position)
             VALUES($1,$2,$3,$3,$4::field_type,$5,$6)",
        )
        .bind(form_version_id)
        .bind(section_id)
        .bind(key)
        .bind(kind)
        .bind(required)
        .bind(position)
        .execute(pool)
        .await
        .unwrap();
    }
    let workflow_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflows(workflow_node_type_id,name,slug)
         VALUES($1,'Response Owner Workflow','response-owner-workflow') RETURNING id",
    )
    .bind(node_type_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let workflow_version_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_versions(workflow_id,version_label,status,published_at)
         VALUES($1,'1.0.0','published'::form_version_status,now()) RETURNING id",
    )
    .bind(workflow_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let workflow_step_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_steps(workflow_version_id,form_version_id,title,position)
         VALUES($1,$2,'Response',0) RETURNING id",
    )
    .bind(workflow_version_id)
    .bind(form_version_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let workflow_assignment_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_assignments(workflow_version_id,workflow_step_id,node_id,account_id)
         VALUES($1,$2,$3,$4) RETURNING id",
    )
    .bind(workflow_version_id)
    .bind(workflow_step_id)
    .bind(node_id)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .unwrap();
    OwnerFixture {
        form_version_id,
        node_id,
        workflow_assignment_id,
        workflow_version_id,
        workflow_step_id,
        account_id,
    }
}

async fn owner_request(
    app: axum::Router,
    token: &str,
    idempotency_key: &str,
    body: Value,
) -> (StatusCode, Value) {
    request_status_and_json(
        app,
        Request::builder()
            .method("POST")
            .uri(RESPONSE_OWNER_ACTION_PATH)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header(header::CONTENT_TYPE, RESPONSE_OWNER_ACTION_MEDIA_TYPE)
            .header(RESPONSE_OWNER_ACTION_IDEMPOTENCY_HEADER, idempotency_key)
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
}

async fn success_sequence(
    pool: &sqlx::PgPool,
    app: axum::Router,
    token: &str,
    key: &str,
    body: Value,
    expected: (&str, &str, Option<&str>),
) -> u64 {
    let (action, kind, reason) = expected;
    let (status, value) = owner_request(app, token, key, body).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value["signed_receipt"]["payload"]["action"], action);
    assert_eq!(
        value["signed_receipt"]["payload"]["export"]["change_kind"],
        kind
    );
    assert_eq!(
        value["signed_receipt"]["payload"]["export"]["tombstone_reason"],
        reason.map_or(Value::Null, |reason| json!(reason))
    );
    verify_signed_receipt(&value);
    let sequence = u64_at(&value, "/signed_receipt/payload/export/change_sequence");
    assert_export_integrity(
        pool,
        sequence,
        value["signed_receipt"]["payload"]["export"]["content_digest"].as_str(),
    )
    .await;
    sequence
}

async fn create_response(
    pool: &sqlx::PgPool,
    app: axum::Router,
    token: &str,
    fixture: &OwnerFixture,
    logical_key: &str,
    idempotency_key: &str,
) -> (Uuid, u64) {
    let (status, value) = owner_request(
        app,
        token,
        idempotency_key,
        json!({
            "schema_version":1,"action":"create","logical_key":logical_key,
            "form_version_id":fixture.form_version_id,"node_id":fixture.node_id,
            "values":{"answer":"delete-me","labels":[],"optional":null}
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    verify_signed_receipt(&value);
    let sequence = u64_at(&value, "/signed_receipt/payload/export/change_sequence");
    assert_export_integrity(
        pool,
        sequence,
        value["signed_receipt"]["payload"]["export"]["content_digest"].as_str(),
    )
    .await;
    (
        uuid_at(&value, "/signed_receipt/payload/response_id"),
        sequence,
    )
}

fn verify_signed_receipt(value: &Value) {
    let response: ResponseOwnerActionResponse = serde_json::from_value(value.clone()).unwrap();
    let signer = tessara_module_contract::PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "core-development-v1",
        ProtocolSignaturePurposeV1::ResponseOwnerActionReceipt,
        [12_u8; 32],
    )
    .unwrap();
    signer
        .verifier()
        .verify(&response.signed_receipt)
        .expect("Core Response owner receipt signature");
}

fn uuid_at(value: &Value, pointer: &str) -> Uuid {
    value
        .pointer(pointer)
        .unwrap()
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

fn u64_at(value: &Value, pointer: &str) -> u64 {
    value.pointer(pointer).unwrap().as_u64().unwrap()
}

async fn assert_one_final_change(
    pool: &sqlx::PgPool,
    response_id: Uuid,
    sequence: u64,
    kind: &str,
    reason: Option<&str>,
) {
    let rows = sqlx::query(
        "SELECT change_sequence,change_kind,payload FROM response_export_changes
          WHERE response_id=$1 ORDER BY change_sequence",
    )
    .bind(response_id)
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].try_get::<i64, _>("change_sequence").unwrap(),
        i64::try_from(sequence).unwrap()
    );
    assert_eq!(rows[0].try_get::<String, _>("change_kind").unwrap(), kind);
    let payload: Value = rows[0].try_get("payload").unwrap();
    assert_eq!(payload.get("reason").and_then(Value::as_str), reason);
}

async fn assert_export_integrity(
    pool: &sqlx::PgPool,
    sequence: u64,
    receipt_content_digest: Option<&str>,
) {
    let change = read_and_validate_export_row(pool, i64::try_from(sequence).unwrap())
        .await
        .expect("stored export row must pass raw and canonical integrity checks");
    assert!(change.validate_content_digest().is_ok());
    if let Some(receipt_content_digest) = receipt_content_digest {
        let canonical_digest = match &change {
            SubmittedResponseChange::Upsert(upsert) => &upsert.content_digest,
            SubmittedResponseChange::Tombstone { content_digest, .. } => content_digest,
        };
        assert_eq!(canonical_digest, receipt_content_digest);
    }
}

async fn read_and_validate_export_row(
    pool: &sqlx::PgPool,
    sequence: i64,
) -> tessara_api::error::ApiResult<SubmittedResponseChange> {
    let row = sqlx::query(
        "SELECT change_kind,payload::text AS payload_text,content_digest
           FROM response_export_changes WHERE change_sequence=$1",
    )
    .bind(sequence)
    .fetch_one(pool)
    .await?;
    let kind: String = row.try_get("change_kind")?;
    let payload_text: String = row.try_get("payload_text")?;
    let stored_digest: String = row.try_get("content_digest")?;
    tessara_api::validate_response_export_storage_row(&kind, &payload_text, &stored_digest)
}

fn is_sanitized_export_rejection(error: tessara_api::error::ApiError) -> bool {
    matches!(
        error,
        tessara_api::error::ApiError::NotFound(message)
            if message == "Response export is unavailable"
    )
}

async fn insert_runtime_draft(pool: &sqlx::PgPool, fixture: &OwnerFixture) -> Uuid {
    let submission_id: Uuid = sqlx::query_scalar(
        "INSERT INTO submissions(form_version_id,node_id,workflow_assignment_id,status)
         VALUES($1,$2,$3,'draft'::submission_status) RETURNING id",
    )
    .bind(fixture.form_version_id)
    .bind(fixture.node_id)
    .bind(fixture.workflow_assignment_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let field_id: Uuid = sqlx::query_scalar(
        "SELECT field_id FROM form_fields WHERE form_version_id=$1 AND key='answer'",
    )
    .bind(fixture.form_version_id)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO submission_values(submission_id,form_version_id,field_id,value)
         VALUES($1,$2,$3,'\"submitted\"'::jsonb)",
    )
    .bind(submission_id)
    .bind(fixture.form_version_id)
    .bind(field_id)
    .execute(pool)
    .await
    .unwrap();
    let workflow_instance_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_instances(
             workflow_assignment_id,workflow_version_id,node_id,assignee_account_id,started_by_account_id)
         VALUES($1,$2,$3,$4,$4) RETURNING id",
    )
    .bind(fixture.workflow_assignment_id)
    .bind(fixture.workflow_version_id)
    .bind(fixture.node_id)
    .bind(fixture.account_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let step_instance_id: Uuid = sqlx::query_scalar(
        "INSERT INTO workflow_step_instances(
             workflow_instance_id,workflow_step_id,submission_id,status)
         VALUES($1,$2,$3,'in_progress') RETURNING id",
    )
    .bind(workflow_instance_id)
    .bind(fixture.workflow_step_id)
    .bind(submission_id)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE submissions SET workflow_instance_id=$2,workflow_step_instance_id=$3 WHERE id=$1",
    )
    .bind(submission_id)
    .bind(workflow_instance_id)
    .bind(step_instance_id)
    .execute(pool)
    .await
    .unwrap();
    submission_id
}
