use axum::{
    Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use chrono::Utc;
use sqlx::Row;
use tessara_module_runtime::SecurityStateProvider;
use tessara_responses_contract::{
    RESPONSE_START_RECONCILIATION_ACTION, RESPONSE_START_RECONCILIATION_BINDING_KEY,
    RESPONSE_START_RECONCILIATION_CONTRACT_ID, RESPONSE_START_RECONCILIATION_MEDIA_TYPE,
    RESPONSE_START_RECONCILIATION_PATH, RESPONSE_START_RECONCILIATION_SCHEMA_VERSION,
    ResponseLifecycleState, ResponseReference, ResponseStartReconciliationCommit,
    ResponseStartReconciliationRequest, ResponseStartReconciliationResponse,
    ResponseStartReconciliationState,
};

use crate::ResponseRuntime;

pub(crate) fn routes() -> Router<std::sync::Arc<ResponseRuntime>> {
    Router::new().route(RESPONSE_START_RECONCILIATION_PATH, post(reconcile))
}

async fn reconcile(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    reconcile_response(async {
        if headers
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            != Some(RESPONSE_START_RECONCILIATION_MEDIA_TYPE)
        {
            return Err(());
        }
        crate::private_provider_auth::authorize(
            &runtime,
            &headers,
            &body,
            crate::private_provider_auth::PrivateProviderContract {
                path: RESPONSE_START_RECONCILIATION_PATH,
                binding: RESPONSE_START_RECONCILIATION_BINDING_KEY,
                contract: RESPONSE_START_RECONCILIATION_CONTRACT_ID,
                action: RESPONSE_START_RECONCILIATION_ACTION,
                capability: "submissions:manage",
            },
        )
        .await?;
        let request: ResponseStartReconciliationRequest =
            serde_json::from_slice(&body).map_err(|_| ())?;
        request.validate().map_err(|_| ())?;
        let security = runtime.current_security_state().await.map_err(|_| ())?;
        let result = reconcile_start(
            &runtime.pool,
            security.installation_id,
            security.module_instance_id,
            &request,
        )
        .await?;
        contract_response(&result)
    })
    .await
}

async fn reconcile_start(
    pool: &sqlx::PgPool,
    installation_id: uuid::Uuid,
    module_instance_id: uuid::Uuid,
    request: &ResponseStartReconciliationRequest,
) -> Result<ResponseStartReconciliationResponse, ()> {
    let mut transaction = pool.begin().await.map_err(|_| ())?;
    let row = sqlx::query(
        "SELECT workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,
                    expires_at,state,response_id
             FROM response_start_claims WHERE one_use_nonce=$1 FOR UPDATE",
    )
    .bind(request.one_use_nonce)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| ())?;
    let result = match row {
        None => absent(),
        Some(row)
            if row.get::<uuid::Uuid, _>("workflow_assignment_id")
                != request.workflow_assignment_id
                || row.get::<uuid::Uuid, _>("workflow_instance_id")
                    != request.workflow_instance_id
                || row.get::<uuid::Uuid, _>("workflow_step_instance_id")
                    != request.workflow_step_instance_id =>
        {
            return Err(());
        }
        Some(row) => match row.get::<String, _>("state").as_str() {
            "pending" if row.get::<chrono::DateTime<Utc>, _>("expires_at") > Utc::now() => {
                pending()
            }
            "pending" => {
                sqlx::query(
                    "UPDATE response_start_claims
                         SET state='abandoned',finalized_at=now()
                         WHERE one_use_nonce=$1 AND state='pending'",
                )
                .bind(request.one_use_nonce)
                .execute(&mut *transaction)
                .await
                .map_err(|_| ())?;
                absent()
            }
            "abandoned" => absent(),
            "committed" => {
                let response_id = row.get::<Option<uuid::Uuid>, _>("response_id").ok_or(())?;
                let response = sqlx::query(
                    "SELECT status::text AS status,revision FROM responses WHERE id=$1",
                )
                .bind(response_id)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(|_| ())?
                .ok_or(())?;
                let lifecycle_state = match response.get::<String, _>("status").as_str() {
                    "draft" => ResponseLifecycleState::Draft,
                    "submitted" => ResponseLifecycleState::Submitted,
                    "deleted" => ResponseLifecycleState::Deleted,
                    _ => return Err(()),
                };
                ResponseStartReconciliationResponse {
                    schema_version: RESPONSE_START_RECONCILIATION_SCHEMA_VERSION,
                    state: ResponseStartReconciliationState::Committed,
                    commit: Some(ResponseStartReconciliationCommit {
                        response: ResponseReference::from_parts(
                            installation_id,
                            module_instance_id,
                            response_id,
                        )
                        .map_err(|_| ())?,
                        revision: u64::try_from(response.get::<i64, _>("revision"))
                            .map_err(|_| ())?,
                        lifecycle_state,
                    }),
                }
            }
            _ => return Err(()),
        },
    };
    result.validate().map_err(|_| ())?;
    transaction.commit().await.map_err(|_| ())?;
    Ok(result)
}

fn pending() -> ResponseStartReconciliationResponse {
    ResponseStartReconciliationResponse {
        schema_version: RESPONSE_START_RECONCILIATION_SCHEMA_VERSION,
        state: ResponseStartReconciliationState::Pending,
        commit: None,
    }
}

fn absent() -> ResponseStartReconciliationResponse {
    ResponseStartReconciliationResponse {
        schema_version: RESPONSE_START_RECONCILIATION_SCHEMA_VERSION,
        state: ResponseStartReconciliationState::Absent,
        commit: None,
    }
}

fn contract_response(value: &ResponseStartReconciliationResponse) -> Result<Response, ()> {
    Response::builder()
        .header(
            header::CONTENT_TYPE,
            RESPONSE_START_RECONCILIATION_MEDIA_TYPE,
        )
        .body(Body::from(serde_json::to_vec(value).map_err(|_| ())?))
        .map_err(|_| ())
}

async fn reconcile_response(future: impl Future<Output = Result<Response, ()>>) -> Response {
    future
        .await
        .unwrap_or_else(|()| StatusCode::NOT_FOUND.into_response())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn request(nonce: uuid::Uuid) -> ResponseStartReconciliationRequest {
        ResponseStartReconciliationRequest {
            schema_version: RESPONSE_START_RECONCILIATION_SCHEMA_VERSION,
            workflow_assignment_id: uuid::Uuid::from_u128(1),
            workflow_instance_id: uuid::Uuid::from_u128(2),
            workflow_step_instance_id: uuid::Uuid::from_u128(3),
            one_use_nonce: nonce,
        }
    }

    async fn insert_claim(
        pool: &sqlx::PgPool,
        nonce: uuid::Uuid,
        expires_at: chrono::DateTime<Utc>,
    ) {
        sqlx::query("INSERT INTO response_start_claims(one_use_nonce,workflow_assignment_id,workflow_instance_id,workflow_step_instance_id,actor_account_id,idempotency_key_digest,request_digest,expires_at,state) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'pending')")
            .bind(nonce)
            .bind(uuid::Uuid::from_u128(1))
            .bind(uuid::Uuid::from_u128(2))
            .bind(uuid::Uuid::from_u128(3))
            .bind(uuid::Uuid::from_u128(4))
            .bind(format!("sha256:{:064x}", nonce.as_u128()))
            .bind(format!(
                "sha256:{:064x}",
                nonce.as_u128() ^ u128::MAX
            ))
            .bind(expires_at)
            .execute(pool)
            .await
            .unwrap();
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn reconciliation_retains_live_claim_and_abandons_only_after_expiry(pool: sqlx::PgPool) {
        let live_nonce = uuid::Uuid::from_u128(10);
        insert_claim(&pool, live_nonce, Utc::now() + Duration::minutes(1)).await;
        let live = reconcile_start(
            &pool,
            uuid::Uuid::from_u128(20),
            uuid::Uuid::from_u128(21),
            &request(live_nonce),
        )
        .await
        .unwrap();
        assert_eq!(live.state, ResponseStartReconciliationState::Pending);

        let expired_nonce = uuid::Uuid::from_u128(11);
        insert_claim(&pool, expired_nonce, Utc::now() - Duration::seconds(1)).await;
        let expired = reconcile_start(
            &pool,
            uuid::Uuid::from_u128(20),
            uuid::Uuid::from_u128(21),
            &request(expired_nonce),
        )
        .await
        .unwrap();
        assert_eq!(expired.state, ResponseStartReconciliationState::Absent);
        let state: String =
            sqlx::query_scalar("SELECT state FROM response_start_claims WHERE one_use_nonce=$1")
                .bind(expired_nonce)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(state, "abandoned");

        let absent = reconcile_start(
            &pool,
            uuid::Uuid::from_u128(20),
            uuid::Uuid::from_u128(21),
            &request(uuid::Uuid::from_u128(12)),
        )
        .await
        .unwrap();
        assert_eq!(absent.state, ResponseStartReconciliationState::Absent);
    }
}
