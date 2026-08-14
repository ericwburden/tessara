//! Policy-neutral scope and principal display catalogs for modules.

use std::collections::BTreeSet;

use axum::{
    Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::Response,
    routing::post,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::Row;
use tessara_control_plane_contract::{
    CONTROL_PLANE_SCHEMA_VERSION, ControlPlaneCatalogAction, PRINCIPAL_DISPLAY_ACTION,
    PRINCIPAL_DISPLAY_CONTRACT_ID, PRINCIPAL_DISPLAY_MEDIA_TYPE, PRINCIPAL_DISPLAY_PATH,
    PrincipalDisplayCatalogItem, PrincipalDisplayCatalogRequest, PrincipalDisplayCatalogResponse,
    SCOPE_CATALOG_ACTION, SCOPE_CATALOG_CONTRACT_ID, SCOPE_CATALOG_MEDIA_TYPE, SCOPE_CATALOG_PATH,
    ScopeCatalogNode, ScopeCatalogRequest, ScopeCatalogResponse,
};
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{ApiError, ApiResult},
};

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(SCOPE_CATALOG_PATH, post(scope_catalog))
        .route(PRINCIPAL_DISPLAY_PATH, post(principal_catalog))
}

async fn scope_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    require_media_type(&headers, SCOPE_CATALOG_MEDIA_TYPE)?;
    let grant = authorize(
        &state,
        &headers,
        SCOPE_CATALOG_CONTRACT_ID,
        SCOPE_CATALOG_ACTION,
        SCOPE_CATALOG_PATH,
        &body,
    )
    .await?;
    let request: ScopeCatalogRequest = serde_json::from_slice(&body).map_err(|_| restricted())?;
    let authorized = authorized_scope(&grant);
    let requested = requested_ids(request.action, request.node_ids, &authorized)?;
    let rows = sqlx::query(
        "SELECT n.id,
                CASE WHEN n.parent_node_id=ANY($1) THEN n.parent_node_id END AS parent_node_id,
                nt.slug AS node_type_key,nt.name AS node_type_name,n.name,
                CASE WHEN parent.id=ANY($1) THEN parent.name || ' / ' || n.name
                     ELSE n.name END AS node_path
         FROM nodes n
         JOIN node_types nt ON nt.id=n.node_type_id
         LEFT JOIN nodes parent ON parent.id=n.parent_node_id
         WHERE n.id=ANY($1) ORDER BY node_path,n.id",
    )
    .bind(requested.iter().copied().collect::<Vec<_>>())
    .fetch_all(&state.pool)
    .await?;
    if rows.len() != requested.len() {
        return Err(restricted());
    }
    let nodes = rows
        .into_iter()
        .map(|row| {
            Ok(ScopeCatalogNode {
                node_id: row.try_get("id")?,
                parent_node_id: row.try_get("parent_node_id")?,
                node_type_key: row.try_get("node_type_key")?,
                node_type_name: row.try_get("node_type_name")?,
                display_label: row.try_get("name")?,
                node_path: row.try_get("node_path")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let requested_set_digest = digest_json(&nodes)?;
    contract_response(
        &ScopeCatalogResponse {
            schema_version: CONTROL_PLANE_SCHEMA_VERSION,
            nodes,
            requested_set_revision: requested_set_digest.clone(),
            requested_set_digest,
        },
        SCOPE_CATALOG_MEDIA_TYPE,
    )
}

async fn principal_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    require_media_type(&headers, PRINCIPAL_DISPLAY_MEDIA_TYPE)?;
    let grant = authorize(
        &state,
        &headers,
        PRINCIPAL_DISPLAY_CONTRACT_ID,
        PRINCIPAL_DISPLAY_ACTION,
        PRINCIPAL_DISPLAY_PATH,
        &body,
    )
    .await?;
    let request: PrincipalDisplayCatalogRequest =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    let authorized = authorized_scope(&grant);
    if authorized.is_empty() {
        return Err(restricted());
    }
    let catalog_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT a.id
         FROM accounts a
         LEFT JOIN role_assignments ra ON ra.account_id=a.id
         WHERE a.is_active=true AND (a.id=$1 OR ra.node_id=ANY($2))
         ORDER BY a.id",
    )
    .bind(grant.payload.original_actor_id)
    .bind(authorized.iter().copied().collect::<Vec<_>>())
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .collect::<BTreeSet<_>>();
    let requested = requested_ids(request.action, request.principal_ids, &catalog_ids)?;
    let rows = sqlx::query(
        "SELECT id,display_name FROM accounts
         WHERE is_active=true AND id=ANY($1) ORDER BY id",
    )
    .bind(requested.iter().copied().collect::<Vec<_>>())
    .fetch_all(&state.pool)
    .await?;
    if rows.len() != requested.len() {
        return Err(restricted());
    }
    let principals = rows
        .into_iter()
        .map(|row| {
            Ok(PrincipalDisplayCatalogItem {
                principal_id: row.try_get("id")?,
                display_name: row.try_get("display_name")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let requested_set_digest = digest_json(&principals)?;
    contract_response(
        &PrincipalDisplayCatalogResponse {
            schema_version: CONTROL_PLANE_SCHEMA_VERSION,
            principals,
            requested_set_revision: requested_set_digest.clone(),
            requested_set_digest,
        },
        PRINCIPAL_DISPLAY_MEDIA_TYPE,
    )
}

fn requested_ids(
    action: ControlPlaneCatalogAction,
    requested: Vec<Uuid>,
    authorized: &BTreeSet<Uuid>,
) -> ApiResult<BTreeSet<Uuid>> {
    match action {
        ControlPlaneCatalogAction::Catalog if requested.is_empty() => Ok(authorized.clone()),
        ControlPlaneCatalogAction::ResolveRequestedSet
            if !requested.is_empty() && requested.iter().all(|id| authorized.contains(id)) =>
        {
            Ok(requested.into_iter().collect())
        }
        _ => Err(restricted()),
    }
}

async fn authorize(
    state: &AppState,
    headers: &HeaderMap,
    contract: &'static str,
    action: &'static str,
    path: &'static str,
    body: &[u8],
) -> ApiResult<crate::module_service_requests::CoreProviderAuthorizationV1> {
    crate::module_service_requests::authorize_core_provider(
        state,
        headers,
        contract,
        action,
        path,
        body,
        "Control-plane catalog is unavailable",
    )
    .await
}

fn authorized_scope(
    grant: &crate::module_service_requests::CoreProviderAuthorizationV1,
) -> BTreeSet<Uuid> {
    grant
        .payload
        .capability_scope_bindings
        .iter()
        .filter(|binding| binding.capability.as_str() == "datasets:manage")
        .flat_map(|binding| {
            std::iter::once(binding.organization_root_id)
                .chain(binding.authorized_organization_ids.iter().copied())
        })
        .collect()
}

fn require_media_type(headers: &HeaderMap, expected: &'static str) -> ApiResult<()> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(expected)
    {
        return Err(restricted());
    }
    Ok(())
}

fn contract_response<T: Serialize>(value: &T, media_type: &'static str) -> ApiResult<Response> {
    let body = serde_json::to_vec(value)
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media_type)
        .body(Body::from(body))
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))
}

fn digest_json<T: Serialize>(value: &T) -> ApiResult<String> {
    let encoded = serde_json::to_vec(value)
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))?;
    Ok(format!("sha256:{:x}", Sha256::digest(encoded)))
}

fn restricted() -> ApiError {
    ApiError::NotFound("Control-plane catalog is unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_set_never_drops_an_unauthorized_identity() {
        let allowed = BTreeSet::from([Uuid::from_u128(1)]);
        assert_eq!(
            requested_ids(ControlPlaneCatalogAction::Catalog, Vec::new(), &allowed).unwrap(),
            allowed
        );
        assert!(
            requested_ids(
                ControlPlaneCatalogAction::ResolveRequestedSet,
                vec![Uuid::from_u128(1), Uuid::from_u128(2)],
                &allowed,
            )
            .is_err()
        );
    }
}
