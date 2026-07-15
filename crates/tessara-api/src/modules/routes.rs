//! Axum routes for Sprint 6A Core module discovery and platform adapters.

use axum::{
    Json, Router,
    extract::{Path, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use uuid::Uuid;

use crate::{auth::AuthenticatedRequest, db::AppState};

use super::{
    destination,
    dto::{
        ApplicationInstallationV1, CoreRuntimeObservationV1, CreateResourceReferenceRequestV1,
        ImmutableCoreNavigationItemV1, MODULE_HTTP_SCHEMA_VERSION_V1, ModuleDetailResponseV1,
        ModuleInventoryResponseV1, NavigationPolicyContributionV1, NavigationPolicyMutationV1,
        NavigationPolicyResponseV1, ResolveDestinationRequestV1, ResolveResourceReferenceRequestV1,
        UpdateNavigationPolicyRequestV1,
    },
    error::{ModuleHttpError, ModuleHttpResult},
    reference,
    service::{
        self, CatalogReadError, ModuleInventoryReadModel, NavigationPolicyReadModel,
        NavigationPolicyUpdateEntry, NavigationPolicyUpdateError,
    },
};

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/admin/modules", get(list_modules))
        .route("/api/admin/modules/{definition_id}", get(get_module))
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
}

async fn list_modules(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
) -> ModuleHttpResult<Json<ModuleInventoryResponseV1>> {
    require_global_read(&auth)?;
    let inventory = service::load_module_inventory(&state.pool)
        .await
        .map_err(map_catalog_error)?;
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
    let inventory = service::load_module_inventory(&state.pool)
        .await
        .map_err(map_catalog_error)?;
    let installation_id = inventory.installation_id;
    let entry = inventory
        .transitions
        .into_iter()
        .find(|entry| entry.definition_id == definition_id)
        .ok_or_else(module_not_found)?;
    Ok(Json(ModuleDetailResponseV1 {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        installation_id,
        entry: entry.normalized_projection,
    }))
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
) -> ModuleHttpResult<Json<NavigationPolicyResponseV1>> {
    require_global_read(&auth)?;
    let policy = service::load_navigation_policy(&state.pool)
        .await
        .map_err(map_catalog_error)?;
    Ok(Json(navigation_policy_response(
        policy,
        auth.account
            .has_global_capability("modules:manage_navigation"),
    )?))
}

async fn update_navigation_policy(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    payload: Result<Json<UpdateNavigationPolicyRequestV1>, JsonRejection>,
) -> ModuleHttpResult<Json<NavigationPolicyResponseV1>> {
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
    ensure_schema_v1(payload.schema_version)?;
    let entries = payload
        .contributions
        .into_iter()
        .map(policy_update_entry)
        .collect();
    let policy = service::update_navigation_policy(
        &state.pool,
        auth.account.account_id,
        correlation_id,
        payload.expected_revision,
        entries,
    )
    .await
    .map_err(map_policy_error)?;
    Ok(Json(navigation_policy_response(policy, true)?))
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
            "The navigation policy update is invalid.",
        ),
    }
}

pub(super) fn inventory_response(inventory: ModuleInventoryReadModel) -> ModuleInventoryResponseV1 {
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
            .map(|entry| entry.normalized_projection)
            .collect(),
    }
}

fn policy_update_entry(entry: NavigationPolicyMutationV1) -> NavigationPolicyUpdateEntry {
    NavigationPolicyUpdateEntry {
        contribution_id: entry.id,
        group: entry.group,
        reorder_band: entry.reorder_band,
        visible: entry.visible,
        order: entry.order,
    }
}

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
        navigation_policy_response,
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
