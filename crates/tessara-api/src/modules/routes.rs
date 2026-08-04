//! Axum routes for Sprint 6A Core module discovery and platform adapters.

use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use tessara_module_contract::{DeploymentProfile, DeploymentReceiptV1};
use uuid::Uuid;

#[cfg(test)]
use super::dto::{
    ImmutableCoreNavigationItemV1, NavigationPolicyContributionV1, NavigationPolicyResponseV1,
};
#[cfg(test)]
use super::service::NavigationPolicyReadModel;

use crate::{auth::AuthenticatedRequest, db::AppState};

use super::{
    destination,
    dto::{
        ApplicationInstallationV1, CoreRuntimeObservationV1, CreateResourceReferenceRequestV1,
        IndependentConfigurationV1, IndependentDefinitionV1, IndependentDiagnosticsV1,
        IndependentInstanceV1, IndependentModuleEntryV1, IndependentReleaseV1,
        MODULE_HTTP_SCHEMA_VERSION_V1, ModuleDetailResponseV1, ModuleInventoryResponseV1,
        NAVIGATION_POLICY_SCHEMA_VERSION_V2, NavigationPolicyResponseV2,
        ResolveDestinationRequestV1, ResolveResourceReferenceRequestV1,
        UpdateNavigationPolicyRequestV2,
    },
    error::{ModuleHttpError, ModuleHttpResult},
    reference, repository,
    service::{
        self, CatalogReadError, ModuleInventoryReadModel, NavigationDestinationUpdateV2,
        NavigationGroupUpdateV2, NavigationPolicyReadModelV2, NavigationPolicyUpdateError,
    },
};

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/admin/modules", get(list_modules))
        .route("/api/admin/modules/{definition_id}", get(get_module))
        .route(
            "/api/internal/deployment-receipts",
            post(import_deployment_receipt),
        )
        .route(
            "/api/admin/deployment-receipts/{revision}",
            get(download_deployment_receipt),
        )
        .route(
            "/api/admin/modules/{definition_id}/descriptor",
            get(get_descriptor),
        )
        .route(
            "/api/admin/navigation-policy",
            get(get_navigation_policy).put(update_navigation_policy),
        )
        .route(
            "/api/platform/destinations/resolve",
            post(resolve_destination),
        )
        .route(
            "/api/platform/resource-references",
            post(create_resource_reference),
        )
        .route(
            "/api/platform/resource-references/resolve",
            post(resolve_resource_reference),
        )
        .route(
            "/api/platform/resource-observations/resolve",
            post(observe_resource_reference),
        )
}

async fn download_deployment_receipt(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(revision): Path<i64>,
) -> ModuleHttpResult<Response> {
    require_global_read(&auth)?;
    if revision <= 0 {
        return Err(ModuleHttpError::bad_request(
            "deployment_receipt_revision_invalid",
            "Receipt revision must be greater than zero.",
        ));
    }
    let installation_id = current_installation_id(&state).await?;
    let receipt = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT receipt FROM deployment_receipts WHERE installation_id = $1 AND revision = $2",
    )
    .bind(installation_id)
    .bind(revision)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        ModuleHttpError::not_found(
            "deployment_receipt_not_found",
            "The requested deployment receipt was not found.",
        )
    })?;
    let bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|_| ModuleHttpError::Internal("deployment receipt serialization failed"))?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=deployment-receipt-{revision}.json"
        ))
        .map_err(|_| ModuleHttpError::Internal("deployment receipt filename is invalid"))?,
    );
    Ok((headers, bytes).into_response())
}

async fn import_deployment_receipt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(receipt): Json<DeploymentReceiptV1>,
) -> ModuleHttpResult<StatusCode> {
    let expected = std::env::var("TESSARA_DEPLOY_TOKEN").map_err(|_| {
        ModuleHttpError::Internal("deployment receipt import token is not configured")
    })?;
    let presented = headers
        .get("x-tessara-deploy-token")
        .and_then(|value| value.to_str().ok());
    if presented != Some(expected.as_str()) {
        return Err(ModuleHttpError::forbidden(
            "deployment_receipt_import_forbidden",
            "A valid deployment receipt import token is required.",
        ));
    }
    if receipt.api_version != "tessara.io/deployment-receipt/v1" {
        return Err(ModuleHttpError::bad_request(
            "deployment_receipt_version_unsupported",
            "Only deployment receipt version 1 is supported.",
        ));
    }
    let installation_id = current_installation_id(&state).await?;
    if receipt.installation_id != installation_id {
        return Err(ModuleHttpError::conflict(
            "deployment_receipt_installation_mismatch",
            "The receipt belongs to a different installation.",
        ));
    }
    let applied_at = chrono::DateTime::parse_from_rfc3339(&receipt.applied_at)
        .map_err(|_| {
            ModuleHttpError::bad_request(
                "deployment_receipt_applied_at_invalid",
                "Receipt applied_at must be RFC 3339.",
            )
        })?
        .with_timezone(&chrono::Utc);
    let receipt_json = serde_json::to_value(&receipt)
        .map_err(|_| ModuleHttpError::Internal("deployment receipt serialization failed"))?;
    let revision = i64::try_from(receipt.revision).map_err(|_| {
        ModuleHttpError::bad_request(
            "deployment_receipt_revision_invalid",
            "Receipt revision is too large.",
        )
    })?;
    let previous_revision = receipt
        .previous_revision
        .map(i64::try_from)
        .transpose()
        .map_err(|_| {
            ModuleHttpError::bad_request(
                "deployment_receipt_revision_invalid",
                "Receipt previous revision is too large.",
            )
        })?;

    let mut tx = state.pool.begin().await?;
    let accepted = sqlx::query("INSERT INTO deployment_receipts (installation_id, revision, plan_digest, applied_at, operator_name, idempotency_key, previous_revision, receipt) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT (installation_id, revision) DO NOTHING")
        .bind(receipt.installation_id)
        .bind(revision)
        .bind(receipt.plan_digest.as_str())
        .bind(applied_at)
        .bind(&receipt.operator)
        .bind(&receipt.idempotency_key)
        .bind(previous_revision)
        .bind(receipt_json.clone())
        .execute(&mut *tx)
        .await?;
    if accepted.rows_affected() == 0 {
        let existing = sqlx::query_as::<_, (String, serde_json::Value)>(
            "SELECT idempotency_key, receipt FROM deployment_receipts WHERE installation_id = $1 AND revision = $2 FOR UPDATE",
        )
        .bind(receipt.installation_id)
        .bind(revision)
        .fetch_one(&mut *tx)
        .await?;
        tx.rollback().await?;
        if existing.0 == receipt.idempotency_key && existing.1 == receipt_json {
            return Ok(StatusCode::NO_CONTENT);
        }
        return Err(ModuleHttpError::conflict(
            "deployment_receipt_revision_conflict",
            "This deployment revision is already bound to different receipt evidence.",
        ));
    }

    // The accepted receipt is the complete current deployment projection.
    // Rebuild instances from that evidence so modules omitted by a later
    // receipt cannot remain visible as if they were still deployed.
    sqlx::query("DELETE FROM module_instances WHERE installation_id = $1")
        .bind(receipt.installation_id)
        .execute(&mut *tx)
        .await?;

    for module in &receipt.modules {
        let manifest = module.manifest.as_ref().ok_or_else(|| {
            ModuleHttpError::bad_request(
                "deployment_receipt_manifest_required",
                "Every curated module receipt must include its complete validated manifest.",
            )
        })?;
        let display_name = module
            .definition_id
            .as_str()
            .rsplit(['.', ':'])
            .next()
            .unwrap_or(module.definition_id.as_str())
            .replace(['-', '_'], " ");
        sqlx::query("INSERT INTO module_definition_reservations (definition_id, display_name) VALUES ($1, $2) ON CONFLICT (definition_id) DO NOTHING")
            .bind(module.definition_id.as_str()).bind(display_name).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO module_releases (id, definition_id, version, manifest_digest, manifest, runtime_image_digest, publisher, trust_state, compatibility_state) VALUES ($1,$2,$3,$4,$5,$6,$7,'curated','compatible') ON CONFLICT (definition_id, manifest_digest) DO UPDATE SET version=EXCLUDED.version, manifest=EXCLUDED.manifest, runtime_image_digest=EXCLUDED.runtime_image_digest, publisher=EXCLUDED.publisher, trust_state='curated', compatibility_state='compatible'")
            .bind(module.release_id).bind(module.definition_id.as_str()).bind(module.version.to_string()).bind(module.manifest_digest.as_str()).bind(sqlx::types::Json(manifest)).bind(module.runtime_image.as_str()).bind(module.publisher.as_str()).execute(&mut *tx).await?;
        for declaration in &manifest.security_capabilities {
            repository::ensure_declared_module_capability(
                &mut tx,
                declaration.id.as_str(),
                &declaration.description,
            )
            .await?;
        }
        for route in &manifest.browser_routes {
            sqlx::query(
                "INSERT INTO core_module_action_declarations
                 (target_definition_id,dependency_binding,functional_contract,action,operation,required_capability)
                 VALUES ($1,$2,$3,$4,'read',$5)
                 ON CONFLICT (target_definition_id,dependency_binding,functional_contract,action)
                 DO UPDATE SET operation=EXCLUDED.operation,required_capability=EXCLUDED.required_capability",
            )
            .bind(module.definition_id.as_str())
            .bind(route.dependency_binding.as_str())
            .bind(route.functional_contract.as_str())
            .bind(&route.authorization_action)
            .bind(route.required_capability.as_str())
            .execute(&mut *tx)
            .await?;
        }
        for route in &manifest.public_api_routes {
            let operation = match route.operation {
                tessara_module_contract::AuthorizationGrantOperationV1::Read => "read",
                tessara_module_contract::AuthorizationGrantOperationV1::Mutation => "mutation",
            };
            sqlx::query(
                "INSERT INTO core_module_action_declarations
                 (target_definition_id,dependency_binding,functional_contract,action,operation,required_capability)
                 VALUES ($1,$2,$3,$4,$5,$6)
                 ON CONFLICT (target_definition_id,dependency_binding,functional_contract,action)
                 DO UPDATE SET operation=EXCLUDED.operation,required_capability=EXCLUDED.required_capability",
            )
            .bind(module.definition_id.as_str())
            .bind(route.dependency_binding.as_str())
            .bind(route.functional_contract.as_str())
            .bind(&route.authorization_action)
            .bind(operation)
            .bind(route.required_capability.as_str())
            .execute(&mut *tx)
            .await?;
        }
        sqlx::query("INSERT INTO module_instances (id, installation_id, definition_id, release_id, identity_state, data_state, database_name, configuration, route_prefix, installed, deployed, configured, ready, enabled, healthy, last_observed_at) VALUES ($1,$2,$3,$4,'live','retained',$5,$6,$7,true,true,true,true,true,true,$8) ON CONFLICT (installation_id, definition_id) DO UPDATE SET release_id=EXCLUDED.release_id, identity_state='live', data_state='retained', database_name=EXCLUDED.database_name, configuration=EXCLUDED.configuration, route_prefix=EXCLUDED.route_prefix, installed=true, deployed=true, configured=true, ready=true, enabled=true, healthy=true, last_observed_at=EXCLUDED.last_observed_at")
            .bind(module.instance_id).bind(receipt.installation_id).bind(module.definition_id.as_str()).bind(module.release_id).bind(&module.database_name).bind(sqlx::types::Json(&module.configuration)).bind(&module.route_prefix).bind(applied_at).execute(&mut *tx).await?;
    }
    service::ensure_navigation_composition_v2(&mut tx, receipt.installation_id, Uuid::new_v4())
        .await
        .map_err(|error| match error {
            service::CatalogSyncError::Database(error) => ModuleHttpError::Database(error),
            other => ModuleHttpError::Integrity(other.stable_code()),
        })?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_modules(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
) -> ModuleHttpResult<Json<ModuleInventoryResponseV1>> {
    require_global_read(&auth)?;
    let mut inventory = service::load_module_inventory(&state.pool)
        .await
        .map_err(map_catalog_error)?;
    refresh_module_observations(&mut inventory).await;
    Ok(Json(inventory_response(inventory)))
}

async fn get_module(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(definition_id): Path<String>,
) -> ModuleHttpResult<Json<ModuleDetailResponseV1>> {
    // Authorization deliberately precedes lookup so unknown identities do not
    // create an unauthenticated or scoped-only definition oracle.
    require_global_read(&auth)?;
    let mut inventory = service::load_module_inventory(&state.pool)
        .await
        .map_err(map_catalog_error)?;
    refresh_module_observations(&mut inventory).await;
    let installation_id = inventory.installation_id;
    let entry = inventory
        .modules
        .into_iter()
        .map(independent_entry_value)
        .chain(
            inventory
                .transitions
                .into_iter()
                .map(|entry| entry.normalized_projection),
        )
        .find(|entry| {
            entry
                .pointer("/descriptor/reserved_definition_id")
                .or_else(|| entry.pointer("/definition/id"))
                .and_then(|value| value.as_str())
                == Some(definition_id.as_str())
        })
        .ok_or_else(module_not_found)?;
    Ok(Json(ModuleDetailResponseV1 {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        installation_id,
        entry,
    }))
}

pub(super) async fn refresh_module_observations(inventory: &mut ModuleInventoryReadModel) {
    let Some(endpoints) = configured_module_control_endpoints() else {
        return;
    };
    let Ok(client) = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_millis(750))
        .build()
    else {
        return;
    };

    for module in &mut inventory.modules {
        let Some(endpoint) = endpoints.get(&module.definition_id) else {
            continue;
        };
        let Some(manifest) = module.manifest.as_ref() else {
            continue;
        };
        let (readiness_path, liveness_path) = match &manifest.deployment {
            DeploymentProfile::TessaraOciV1(deployment) => (
                deployment.readiness_path.as_str(),
                deployment.liveness_path.as_str(),
            ),
        };
        let base_url = endpoint.trim_end_matches('/');
        let readiness_url = format!("{base_url}{readiness_path}");
        let liveness_url = format!("{base_url}{liveness_path}");
        let (ready, healthy) = tokio::join!(
            module_probe_passes(&client, &readiness_url),
            module_probe_passes(&client, &liveness_url),
        );
        let state_changed = module.ready != ready || module.healthy != healthy;
        module.ready = ready;
        module.healthy = healthy;
        if state_changed {
            module.observed_at = Utc::now();
        }
    }
}

async fn module_probe_passes(client: &reqwest::Client, url: &str) -> bool {
    client
        .get(url)
        .send()
        .await
        .is_ok_and(|response| response.status().is_success())
}

fn configured_module_control_endpoints() -> Option<BTreeMap<String, String>> {
    let value = std::env::var("TESSARA_MODULE_CONTROL_ENDPOINTS").ok()?;
    parse_module_control_endpoints(&value)
}

fn parse_module_control_endpoints(value: &str) -> Option<BTreeMap<String, String>> {
    let endpoints: BTreeMap<String, String> = serde_json::from_str(value).ok()?;
    endpoints
        .iter()
        .all(|(definition, endpoint)| {
            !definition.trim().is_empty()
                && (endpoint.starts_with("http://") || endpoint.starts_with("https://"))
        })
        .then_some(endpoints)
}

async fn get_descriptor(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(definition_id): Path<String>,
    request_headers: HeaderMap,
) -> ModuleHttpResult<Response> {
    require_global_read(&auth)?;
    let document = service::load_descriptor_document(&state.pool, &definition_id)
        .await
        .map_err(map_catalog_error)?
        .ok_or_else(module_not_found)?;

    let etag = HeaderValue::from_str(&format!("\"{}\"", document.source_digest))
        .map_err(|_| ModuleHttpError::Internal("descriptor digest is not a valid header"))?;
    if request_headers
        .get(header::IF_NONE_MATCH)
        .is_some_and(|value| if_none_match_matches(value, &document.source_digest))
    {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        response.headers_mut().insert(header::ETAG, etag);
        return Ok(response);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&document.content_type)
            .map_err(|_| ModuleHttpError::Internal("descriptor content type is invalid"))?,
    );
    headers.insert(header::ETAG, etag);
    Ok((headers, document.source_bytes).into_response())
}

fn if_none_match_matches(value: &HeaderValue, source_digest: &str) -> bool {
    let Ok(value) = value.to_str() else {
        return false;
    };
    value.split(',').any(|candidate| {
        let candidate = candidate.trim();
        if candidate == "*" {
            return true;
        }
        let candidate = candidate.strip_prefix("W/").unwrap_or(candidate).trim();
        candidate
            .strip_prefix('"')
            .and_then(|candidate| candidate.strip_suffix('"'))
            == Some(source_digest)
    })
}

async fn get_navigation_policy(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
) -> ModuleHttpResult<Json<NavigationPolicyResponseV2>> {
    require_global_read(&auth)?;
    let policy = service::load_navigation_policy_v2(&state.pool)
        .await
        .map_err(map_catalog_error)?;
    Ok(Json(navigation_policy_response_v2(
        policy,
        auth.account
            .has_global_capability("modules:manage_navigation"),
    )))
}

async fn update_navigation_policy(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    payload: Result<Json<UpdateNavigationPolicyRequestV2>, JsonRejection>,
) -> ModuleHttpResult<Json<NavigationPolicyResponseV2>> {
    let correlation_id = Uuid::new_v4();
    if let Err(error) = require_global_navigation_manage(&auth) {
        service::record_navigation_policy_authorization_denial(
            &state.pool,
            auth.account.account_id,
            correlation_id,
        )
        .await?;
        return Err(error);
    }
    let Json(payload) = strict_json(payload)?;
    ensure_navigation_schema_v2(payload.schema_version)?;
    let groups = payload
        .groups
        .into_iter()
        .map(|group| NavigationGroupUpdateV2 {
            id: group.id,
            label: group.label,
            order: group.order,
        })
        .collect();
    let destinations = payload
        .destinations
        .into_iter()
        .map(|destination| NavigationDestinationUpdateV2 {
            id: destination.id,
            group_id: destination.group_id,
            visible: destination.visible,
            order: destination.order,
        })
        .collect();
    let policy = service::update_navigation_policy_v2(
        &state.pool,
        auth.account.account_id,
        correlation_id,
        payload.expected_revision,
        groups,
        destinations,
    )
    .await
    .map_err(map_policy_error)?;
    Ok(Json(navigation_policy_response_v2(policy, true)))
}

async fn resolve_destination(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    payload: Result<Json<ResolveDestinationRequestV1>, JsonRejection>,
) -> ModuleHttpResult<Json<super::dto::DestinationResolutionResponseV1>> {
    let Json(payload) = strict_json(payload)?;
    ensure_schema_v1(payload.schema_version)?;
    let installation_id = current_installation_id(&state).await?;
    Ok(Json(destination::resolve(
        &payload.destination,
        installation_id,
        &auth.account,
    )))
}

async fn create_resource_reference(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    payload: Result<Json<CreateResourceReferenceRequestV1>, JsonRejection>,
) -> ModuleHttpResult<Json<super::dto::ResourceReferenceResponseV1>> {
    let Json(payload) = strict_json(payload)?;
    let installation_id = current_installation_id(&state).await?;
    Ok(Json(reference::construct(
        payload,
        installation_id,
        &auth.account,
    )?))
}

async fn resolve_resource_reference(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    payload: Result<Json<ResolveResourceReferenceRequestV1>, JsonRejection>,
) -> ModuleHttpResult<Json<tessara_module_contract::ResourceResolutionV1>> {
    let Json(payload) = strict_json(payload)?;
    ensure_schema_v1(payload.schema_version)?;
    let installation_id = current_installation_id(&state).await?;
    let resolution = reference::resolve(
        &state.pool,
        &payload.reference,
        installation_id,
        &auth.account,
    )
    .await?;
    Ok(Json(resolution))
}

async fn observe_resource_reference(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    payload: Result<Json<ResolveResourceReferenceRequestV1>, JsonRejection>,
) -> ModuleHttpResult<Json<super::dto::ResourceObservationResponseV1>> {
    let Json(payload) = strict_json(payload)?;
    ensure_schema_v1(payload.schema_version)?;
    let installation_id = current_installation_id(&state).await?;
    let (resolution, observation) = reference::observe(
        &state.pool,
        &payload.reference,
        installation_id,
        &auth.account,
    )
    .await?;
    Ok(Json(super::dto::ResourceObservationResponseV1 {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        resolution,
        observation,
    }))
}

async fn current_installation_id(state: &AppState) -> ModuleHttpResult<Uuid> {
    service::load_module_inventory(&state.pool)
        .await
        .map(|inventory| inventory.installation_id)
        .map_err(map_catalog_error)
}

fn strict_json<T>(payload: Result<Json<T>, JsonRejection>) -> ModuleHttpResult<Json<T>> {
    payload.map_err(|_| {
        ModuleHttpError::bad_request(
            "platform_request_invalid",
            "The request body does not match the versioned platform contract.",
        )
    })
}

fn ensure_schema_v1(schema_version: u16) -> ModuleHttpResult<()> {
    if schema_version == MODULE_HTTP_SCHEMA_VERSION_V1 {
        Ok(())
    } else {
        Err(ModuleHttpError::bad_request(
            "platform_schema_version_unsupported",
            "Only platform HTTP schema version 1 is supported.",
        ))
    }
}

fn ensure_navigation_schema_v2(schema_version: u16) -> ModuleHttpResult<()> {
    if schema_version == NAVIGATION_POLICY_SCHEMA_VERSION_V2 {
        Ok(())
    } else {
        Err(ModuleHttpError::bad_request(
            "platform_schema_version_unsupported",
            "Only navigation policy schema version 2 is supported.",
        ))
    }
}

fn require_global_read(auth: &AuthenticatedRequest) -> ModuleHttpResult<()> {
    if auth.account.has_global_capability("modules:read") {
        Ok(())
    } else {
        Err(ModuleHttpError::forbidden(
            "modules_read_global_required",
            "Installation-global modules:read authority is required.",
        ))
    }
}

fn require_global_navigation_manage(auth: &AuthenticatedRequest) -> ModuleHttpResult<()> {
    if auth
        .account
        .has_global_capability("modules:manage_navigation")
    {
        Ok(())
    } else {
        Err(ModuleHttpError::forbidden(
            "modules_manage_navigation_global_required",
            "Installation-global modules:manage_navigation authority is required.",
        ))
    }
}

fn module_not_found() -> ModuleHttpError {
    ModuleHttpError::not_found(
        "module_definition_not_found",
        "The requested Module Definition was not found.",
    )
}

fn map_catalog_error(error: CatalogReadError) -> ModuleHttpError {
    match error {
        CatalogReadError::Integrity { code } => ModuleHttpError::Integrity(code),
        CatalogReadError::Database(error) => ModuleHttpError::Database(error),
    }
}

fn map_policy_error(error: NavigationPolicyUpdateError) -> ModuleHttpError {
    match error {
        NavigationPolicyUpdateError::Database(error) => ModuleHttpError::Database(error),
        conflict @ NavigationPolicyUpdateError::RevisionConflict { .. } => {
            ModuleHttpError::conflict(
                conflict.stable_code(),
                "The navigation policy changed after the presented revision.",
            )
        }
        NavigationPolicyUpdateError::Integrity => {
            ModuleHttpError::Integrity("navigation_policy_integrity_mismatch")
        }
        rejected => ModuleHttpError::bad_request(
            rejected.stable_code(),
            navigation_policy_rejection_message(&rejected),
        ),
    }
}

fn navigation_policy_rejection_message(error: &NavigationPolicyUpdateError) -> &'static str {
    match error {
        NavigationPolicyUpdateError::NonEmptyGroupDeletion { .. } => {
            "Move every destination out of the custom group before deleting it."
        }
        NavigationPolicyUpdateError::ProtectedDestination { .. } => {
            "A protected destination cannot be hidden or moved from its required placement."
        }
        NavigationPolicyUpdateError::InvalidGroupCollection => {
            "Navigation groups must keep Main and Admin, with unique labels and consecutive order."
        }
        NavigationPolicyUpdateError::InvalidDestinationCollection => {
            "Every destination must have a valid group and consecutive order."
        }
        NavigationPolicyUpdateError::CoreItemImmutable { .. }
        | NavigationPolicyUpdateError::DuplicateContribution { .. }
        | NavigationPolicyUpdateError::UnknownContribution { .. }
        | NavigationPolicyUpdateError::MissingContribution { .. }
        | NavigationPolicyUpdateError::GroupChangeForbidden { .. }
        | NavigationPolicyUpdateError::BandChangeForbidden { .. }
        | NavigationPolicyUpdateError::InvalidBandOrder { .. }
        | NavigationPolicyUpdateError::InvalidRevision => {
            "The navigation policy update is invalid."
        }
        NavigationPolicyUpdateError::RevisionConflict { .. }
        | NavigationPolicyUpdateError::Integrity
        | NavigationPolicyUpdateError::Database(_) => {
            unreachable!("these errors are handled before request validation failures")
        }
    }
}

pub(super) fn inventory_response(inventory: ModuleInventoryReadModel) -> ModuleInventoryResponseV1 {
    let deployment = inventory.deployment;
    let deployment_history = inventory.deployment_history;
    let independent_definition_ids = inventory
        .modules
        .iter()
        .map(|module| module.definition_id.clone())
        .collect::<BTreeSet<_>>();
    ModuleInventoryResponseV1 {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        installation: ApplicationInstallationV1 {
            id: inventory.installation_id,
            created_at: inventory.installation_created_at,
        },
        core_runtime: CoreRuntimeObservationV1 {
            provenance: inventory.core_runtime.provenance,
            observed_version: inventory.core_runtime.observed_version,
            finding_code: inventory.core_runtime.finding_code,
            observed_at: inventory.core_runtime.observed_at,
        },
        entries: inventory
            .transitions
            .into_iter()
            .filter(|entry| !independent_definition_ids.contains(&entry.definition_id))
            .map(|entry| entry.normalized_projection)
            .chain(inventory.modules.into_iter().map(independent_entry_value))
            .collect(),
        deployment,
        deployment_history,
    }
}

pub(super) fn independent_entry_value(
    module: super::service::IndependentModuleReadModel,
) -> serde_json::Value {
    let manifest = module.manifest.map(|manifest| manifest.0);
    let configured_display_name = module
        .configuration
        .get("display_label")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| module.display_name.clone());
    let mut findings = Vec::new();
    if !module.configured {
        findings.push(serde_json::json!({
            "code": "module_configuration_invalid",
            "path": "instance.configured",
            "message": "The module configuration is not valid."
        }));
    }
    if !module.ready {
        findings.push(serde_json::json!({
            "code": "module_readiness_failed",
            "path": "instance.ready",
            "message": "The module readiness probe is not passing."
        }));
    }
    if !module.healthy {
        findings.push(serde_json::json!({
            "code": "module_health_check_failed",
            "path": "instance.healthy",
            "message": "The module liveness observation is unhealthy."
        }));
    }
    let (readiness_path, liveness_path) = manifest
        .as_ref()
        .map(|manifest| match &manifest.deployment {
            DeploymentProfile::TessaraOciV1(deployment) => (
                deployment.readiness_path.clone(),
                deployment.liveness_path.clone(),
            ),
        })
        .unwrap_or_else(|| ("Not reported".into(), "Not reported".into()));
    serde_json::to_value(IndependentModuleEntryV1::IndependentlyDeployed {
        definition: IndependentDefinitionV1 {
            id: module.definition_id.clone(),
            display_name: configured_display_name,
            description:
                "Independently deployed module observed from the current deployment receipt."
                    .into(),
        },
        release: IndependentReleaseV1 {
            id: module.release_id.to_string(),
            version: module.version,
            manifest_digest: module.manifest_digest,
            runtime_image: module.runtime_image,
            publisher: module.publisher,
            trust: module.trust,
            compatibility: module.compatibility,
        },
        instance: IndependentInstanceV1 {
            id: module.instance_id.to_string(),
            identity: module.identity,
            data: module.data,
            database_name: module.database_name,
            installed: module.installed,
            deployed: module.deployed,
            configured: module.configured,
            ready: module.ready,
            enabled: module.enabled,
            healthy: module.healthy,
            observed_at: module.observed_at.to_rfc3339(),
        },
        configuration: IndependentConfigurationV1 {
            declared: manifest.is_some(),
            valid: module.configured,
            values: module
                .configuration
                .as_object()
                .map(|details| {
                    details
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect()
                })
                .unwrap_or_default(),
        },
        diagnostics: IndependentDiagnosticsV1 {
            readiness_path,
            liveness_path,
            public_route: module.route_prefix.unwrap_or_else(|| "Not reported".into()),
            details: Default::default(),
        },
        manifest,
        findings,
    })
    .expect("typed independent module projection must serialize")
}

#[cfg(test)]
pub(super) fn navigation_policy_response(
    policy: NavigationPolicyReadModel,
    can_manage_navigation: bool,
) -> ModuleHttpResult<NavigationPolicyResponseV1> {
    let contributions = policy
        .entries
        .into_iter()
        .map(|entry| {
            let (before_core_anchor, after_core_anchor) = band_anchors(&entry.reorder_band)?;
            Ok(NavigationPolicyContributionV1 {
                id: entry.contribution_id,
                definition_id: entry.definition_id,
                label: entry.label,
                destination: entry.destination,
                group: entry.group,
                reorder_band: entry.reorder_band,
                before_core_anchor: before_core_anchor.to_string(),
                after_core_anchor: after_core_anchor.to_string(),
                visible: entry.visible,
                order: entry.order,
                required_capabilities_any_of: entry.required_capabilities_any_of,
            })
        })
        .collect::<ModuleHttpResult<Vec<_>>>()?;

    Ok(NavigationPolicyResponseV1 {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        installation_id: policy.installation_id,
        revision: policy.revision,
        can_manage_navigation,
        immutable_core_items: immutable_core_items(),
        contributions,
    })
}

pub(super) fn navigation_policy_response_v2(
    policy: NavigationPolicyReadModelV2,
    can_manage_navigation: bool,
) -> NavigationPolicyResponseV2 {
    use super::dto::{
        NavigationDestinationOwnerV2, NavigationDestinationV2, NavigationGroupOwnerV2,
        NavigationGroupV2,
    };
    use super::navigation_catalog::NavigationCatalogOwner;
    use super::service::NavigationGroupOwnerV2 as ServiceGroupOwner;

    NavigationPolicyResponseV2 {
        schema_version: NAVIGATION_POLICY_SCHEMA_VERSION_V2,
        installation_id: policy.installation_id,
        revision: policy.revision,
        can_manage_navigation,
        groups: policy
            .groups
            .into_iter()
            .map(|group| {
                let is_custom = group.owner == ServiceGroupOwner::Custom;
                NavigationGroupV2 {
                    id: group.id,
                    label: group.label,
                    order: group.order,
                    owner: if is_custom {
                        NavigationGroupOwnerV2::Custom
                    } else {
                        NavigationGroupOwnerV2::Core
                    },
                    can_rename: can_manage_navigation && is_custom,
                    can_move: can_manage_navigation,
                    can_delete: can_manage_navigation && is_custom,
                }
            })
            .collect(),
        destinations: policy
            .destinations
            .into_iter()
            .map(|destination| NavigationDestinationV2 {
                id: destination.id,
                key: destination.key,
                label: destination.label,
                route: destination.route,
                semantic_destination: destination.semantic_destination,
                definition_id: destination.definition_id,
                owner: match destination.owner {
                    NavigationCatalogOwner::Core => NavigationDestinationOwnerV2::Core,
                    NavigationCatalogOwner::Contribution => {
                        NavigationDestinationOwnerV2::Contribution
                    }
                },
                required_capabilities_any_of: destination.required_capabilities_any_of,
                group_id: destination.group_id,
                visible: destination.visible,
                order: destination.order,
                available: destination.available,
                can_hide: can_manage_navigation && destination.can_hide,
                can_move_between_groups: can_manage_navigation
                    && destination.can_move_between_groups,
                can_reorder: can_manage_navigation,
            })
            .collect(),
    }
}

#[cfg(test)]
fn band_anchors(reorder_band: &str) -> ModuleHttpResult<(&'static str, &'static str)> {
    match reorder_band {
        "main_between_organization_and_operations" => Ok(("operations", "organization")),
        "main_after_operations" => Ok(("main_group_end", "operations")),
        "admin_between_administration_and_module_management" => {
            Ok(("module_management", "administration"))
        }
        _ => Err(ModuleHttpError::Integrity(
            "navigation_policy_unknown_reorder_band",
        )),
    }
}

#[cfg(test)]
fn immutable_core_items() -> Vec<ImmutableCoreNavigationItemV1> {
    [
        ("home", "Home", "Main", "/"),
        ("organization", "Organization", "Main", "/organization"),
        ("operations", "Operations", "Main", "/operations"),
        (
            "administration",
            "Administration",
            "Admin",
            "/administration",
        ),
        (
            "module_management",
            "Module Management",
            "Admin",
            "/administration/modules",
        ),
    ]
    .into_iter()
    .map(|(id, label, group, route)| ImmutableCoreNavigationItemV1 {
        id: id.to_string(),
        label: label.to_string(),
        group: group.to_string(),
        route: route.to_string(),
        policy_mutable: false,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use uuid::Uuid;

    use super::{
        band_anchors, if_none_match_matches, immutable_core_items, map_policy_error,
        navigation_policy_response, parse_module_control_endpoints,
    };
    use crate::auth::{AccountContext, AuthenticatedRequest, CapabilityScope, SessionContext};
    use crate::modules::{
        error::ModuleHttpError,
        service::{NavigationPolicyEntry, NavigationPolicyReadModel, NavigationPolicyUpdateError},
    };

    #[test]
    fn descriptor_conditionals_accept_exact_weak_quoted_list_and_wildcard_tags() {
        let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        for value in [
            format!("\"{digest}\""),
            format!("W/\"{digest}\""),
            format!("\"other\", \"{digest}\""),
            "*".to_string(),
        ] {
            assert!(
                if_none_match_matches(&value.parse().expect("header"), digest),
                "{value}"
            );
        }
        assert!(!if_none_match_matches(
            &"\"sha256:other\"".parse().expect("header"),
            digest
        ));
        assert!(!if_none_match_matches(
            &digest.parse().expect("syntactically representable header"),
            digest
        ));
    }

    #[test]
    fn live_observation_registry_is_definition_driven_and_rejects_non_http_endpoints() {
        let endpoints = parse_module_control_endpoints(
            r#"{
                "tessara.dashboards": "http://dashboards:8091",
                "example.future-module": "https://future-module.internal"
            }"#,
        )
        .expect("valid endpoint registry");
        assert_eq!(
            endpoints.get("example.future-module").map(String::as_str),
            Some("https://future-module.internal")
        );
        assert!(
            parse_module_control_endpoints(r#"{"example.future-module":"file:///etc/tessara"}"#)
                .is_none()
        );
    }

    #[test]
    fn policy_projection_exposes_fixed_anchors_and_never_makes_core_items_mutable() {
        let policy = NavigationPolicyReadModel {
            installation_id: Uuid::nil(),
            revision: 7,
            entries: vec![NavigationPolicyEntry {
                contribution_id: "tessara.datasets.navigation".to_string(),
                definition_id: "tessara.datasets".to_string(),
                destination: "datasets.directory".to_string(),
                label: "Datasets".to_string(),
                group: "Admin".to_string(),
                reorder_band: "admin_between_administration_and_module_management".to_string(),
                source_order_hint: 20,
                default_policy_order: 0,
                required_capabilities_any_of: vec!["datasets:read".to_string()],
                visible: true,
                order: 0,
            }],
        };

        let response = navigation_policy_response(policy, false).expect("policy response");
        assert!(!response.can_manage_navigation);
        assert_eq!(
            response.contributions[0].before_core_anchor,
            "module_management"
        );
        assert_eq!(
            response.contributions[0].after_core_anchor,
            "administration"
        );
        assert!(
            response
                .immutable_core_items
                .iter()
                .all(|item| !item.policy_mutable)
        );
        assert!(response.immutable_core_items.iter().any(|item| {
            item.id == "module_management"
                && item.group == "Admin"
                && item.route == "/administration/modules"
        }));
    }

    #[test]
    fn all_approved_bands_have_explicit_core_anchor_context() {
        assert_eq!(
            band_anchors("main_between_organization_and_operations").expect("band"),
            ("operations", "organization")
        );
        assert_eq!(
            band_anchors("main_after_operations").expect("band"),
            ("main_group_end", "operations")
        );
        assert_eq!(
            band_anchors("admin_between_administration_and_module_management").expect("band"),
            ("module_management", "administration")
        );
        assert!(band_anchors("caller_selected_band").is_err());
    }

    #[test]
    fn policy_errors_keep_the_approved_stable_codes() {
        let cases = [
            (
                NavigationPolicyUpdateError::RevisionConflict {
                    presented: 1,
                    current: 2,
                },
                StatusCode::CONFLICT,
                "navigation_policy_revision_conflict",
            ),
            (
                NavigationPolicyUpdateError::CoreItemImmutable {
                    contribution_id: "module_management".to_string(),
                },
                StatusCode::BAD_REQUEST,
                "navigation_policy_core_item_immutable",
            ),
            (
                NavigationPolicyUpdateError::BandChangeForbidden {
                    contribution_id: "tessara.forms.navigation".to_string(),
                },
                StatusCode::BAD_REQUEST,
                "navigation_policy_band_change_forbidden",
            ),
        ];

        for (error, expected_status, expected_code) in cases {
            let mapped = map_policy_error(error);
            match mapped {
                ModuleHttpError::Rejected { status, code, .. } => {
                    assert_eq!(status, expected_status);
                    assert_eq!(code, expected_code);
                }
                other => panic!("unexpected mapping: {other:?}"),
            }
        }

        assert!(
            immutable_core_items().iter().any(|item| {
                item.id == "module_management" && item.label == "Module Management"
            })
        );
    }

    #[test]
    fn module_http_authority_requires_global_scope_and_manage_implies_read() {
        let scoped_read = authenticated("modules:read", false);
        let global_read = authenticated("modules:read", true);
        let global_manage = authenticated("modules:manage_navigation", true);

        assert_eq!(
            super::require_global_read(&scoped_read)
                .expect_err("scoped-only read fails")
                .code(),
            "modules_read_global_required"
        );
        super::require_global_read(&global_read).expect("global read succeeds");
        assert_eq!(
            super::require_global_navigation_manage(&global_read)
                .expect_err("read does not imply manage")
                .code(),
            "modules_manage_navigation_global_required"
        );
        super::require_global_read(&global_manage).expect("manage implies read");
        super::require_global_navigation_manage(&global_manage).expect("global manage succeeds");
    }

    fn authenticated(capability: &str, global: bool) -> AuthenticatedRequest {
        AuthenticatedRequest {
            account: AccountContext {
                account_id: Uuid::nil(),
                email: "module-http@example.test".to_string(),
                display_name: "Module HTTP".to_string(),
                is_active: true,
                roles: Vec::new(),
                capabilities: vec![capability.to_string()],
                capability_scopes: vec![CapabilityScope {
                    capability: capability.to_string(),
                    global,
                    node_ids: Vec::new(),
                }],
                scope_nodes: Vec::new(),
                delegations: Vec::new(),
            },
            session: SessionContext { token: Uuid::nil() },
        }
    }
}
