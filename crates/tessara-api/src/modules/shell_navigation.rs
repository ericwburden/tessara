//! Authenticated, per-actor shell navigation projection.
//!
//! The schema-v2 policy owns ordered groups and destination placement. Core's
//! catalog remains authoritative for labels, routes, ownership, protection,
//! and capability predicates. Route authorization remains independent.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json, Router,
    extract::State,
    http::{
        HeaderValue,
        header::{CACHE_CONTROL, VARY},
    },
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use sqlx::Row;
use tessara_module_contract::{
    ModuleManifest, NavigationContributionId, NavigationProjectionV1, ResourceOwner,
    SemanticDestination, SemanticRouteName,
};
use uuid::Uuid;

use crate::{
    auth::{AccountContext, AuthenticatedRequest},
    db::AppState,
    error::{ApiError, ApiResult},
};

use super::{
    destination,
    dto::DestinationResolutionStatusV1,
    navigation_catalog::{self, NavigationCatalogOwner},
    service::{self, NavigationPolicyReadModelV2},
};

const SHELL_NAVIGATION_SCHEMA_VERSION_V3: u16 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ShellNavigationStateV1 {
    Available,
    Unavailable,
}

/// One versioned model consumed unchanged by desktop and mobile shells.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ShellNavigationResponseV1 {
    pub(crate) schema_version: u16,
    pub(crate) policy_revision: Option<i64>,
    pub(crate) state: ShellNavigationStateV1,
    pub(crate) groups: Vec<ShellNavigationGroupV1>,
    pub(crate) unavailable: Option<ShellNavigationUnavailableV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ShellNavigationGroupV1 {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) items: Vec<ShellNavigationItemV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ShellNavigationItemOwnerV1 {
    Core,
    Contribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ShellNavigationModeV1 {
    Shell,
    Document,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ShellNavigationItemV1 {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) href: String,
    pub(crate) owner: ShellNavigationItemOwnerV1,
    pub(crate) contribution_id: Option<String>,
    pub(crate) navigation_mode: ShellNavigationModeV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ShellNavigationUnavailableV1 {
    pub(crate) code: String,
    pub(crate) message: String,
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/api/shell/navigation", get(get_shell_navigation))
}

async fn get_shell_navigation(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
) -> Response {
    let mut response = Json(load_response(&state, &auth.account).await).into_response();
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("private, no-store"));
    response
        .headers_mut()
        .insert(VARY, HeaderValue::from_static("Cookie, Authorization"));
    response
}

pub(super) async fn load_response(
    state: &AppState,
    account: &AccountContext,
) -> ShellNavigationResponseV1 {
    load_response_for_installation(state, account, None)
        .await
        .expect("an unbound shell-navigation request cannot have an installation mismatch")
}

/// Resolves the same Core-owned, actor-filtered navigation model for a
/// separately rendered module document that Core exposes to its own shell.
///
/// The expected installation binding prevents a module document from ever
/// receiving navigation composed for another installation. Policy/catalog
/// failures retain the same fail-closed Core-only projection used by the
/// first-party shell.
pub(crate) async fn load_context_navigation(
    state: &AppState,
    account: &AccountContext,
    expected_installation_id: Uuid,
) -> ApiResult<Vec<NavigationProjectionV1>> {
    let canonical_installation_id: Uuid =
        sqlx::query_scalar("SELECT id FROM application_installations WHERE singleton=true")
            .fetch_one(&state.pool)
            .await?;
    validate_installation_binding(expected_installation_id, canonical_installation_id).map_err(
        |()| {
            ApiError::Internal(anyhow::anyhow!(
                "module Shell Context installation does not match Core installation"
            ))
        },
    )?;
    let response = load_response_for_installation(state, account, Some(expected_installation_id))
        .await
        .map_err(|()| {
            ApiError::Internal(anyhow::anyhow!(
                "module Shell Context installation does not match Core navigation policy"
            ))
        })?;
    navigation_projection(&response).map_err(|()| {
        ApiError::Internal(anyhow::anyhow!(
            "Core shell navigation contains an invalid destination identity"
        ))
    })
}

fn validate_installation_binding(expected: Uuid, canonical: Uuid) -> Result<(), ()> {
    (expected == canonical).then_some(()).ok_or(())
}

async fn load_response_for_installation(
    state: &AppState,
    account: &AccountContext,
    expected_installation_id: Option<Uuid>,
) -> Result<ShellNavigationResponseV1, ()> {
    match service::load_navigation_policy_v2(&state.pool).await {
        Ok(policy) => {
            if expected_installation_id.is_some_and(|expected| expected != policy.installation_id) {
                return Err(());
            }
            let (installed_definitions, lifecycle_definitions) =
                browser_delivery_inventory(&state.pool, policy.installation_id)
                    .await
                    .unwrap_or_else(|error| {
                        tracing::warn!(%error, "module lifecycle inventory is unavailable; contribution navigation will use document fallback");
                        (BTreeSet::new(), BTreeSet::new())
                    });
            Ok(
                match compose_groups(
                    &policy,
                    account,
                    &installed_definitions,
                    &lifecycle_definitions,
                ) {
                    Ok(groups) => ShellNavigationResponseV1 {
                        schema_version: SHELL_NAVIGATION_SCHEMA_VERSION_V3,
                        policy_revision: Some(policy.revision),
                        state: ShellNavigationStateV1::Available,
                        groups,
                        unavailable: None,
                    },
                    Err(()) => unavailable_response(account, Some(policy.revision)),
                },
            )
        }
        Err(error) => {
            tracing::warn!(
                code = error.stable_code(),
                "shell navigation policy is unavailable"
            );
            Ok(unavailable_response(account, None))
        }
    }
}

fn navigation_projection(
    response: &ShellNavigationResponseV1,
) -> Result<Vec<NavigationProjectionV1>, ()> {
    response
        .groups
        .iter()
        .flat_map(|group| &group.items)
        .map(|item| {
            let destination_id = match item.contribution_id.as_deref() {
                Some(contribution_id) => contribution_id,
                None => navigation_catalog::DESTINATIONS
                    .iter()
                    .find(|destination| {
                        destination.owner == NavigationCatalogOwner::Core
                            && destination.key == item.key
                    })
                    .map(|destination| destination.id)
                    .ok_or(())?,
            };
            Ok(NavigationProjectionV1 {
                contribution_id: NavigationContributionId::new(destination_id).map_err(|_| ())?,
                label: item.label.clone(),
                href: item.href.clone(),
            })
        })
        .collect()
}

fn compose_groups(
    policy: &NavigationPolicyReadModelV2,
    account: &AccountContext,
    installed_definitions: &BTreeSet<String>,
    lifecycle_definitions: &BTreeSet<String>,
) -> Result<Vec<ShellNavigationGroupV1>, ()> {
    let mut groups = Vec::new();
    for group in &policy.groups {
        let mut destinations = policy
            .destinations
            .iter()
            .filter(|destination| destination.group_id == group.id)
            .collect::<Vec<_>>();
        destinations.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| left.id.cmp(&right.id))
        });

        let mut items = Vec::new();
        for destination in destinations {
            if !destination.visible
                || !destination.available
                || (!destination.required_capabilities_any_of.is_empty()
                    && !destination
                        .required_capabilities_any_of
                        .iter()
                        .any(|required| account.has_capability(required)))
            {
                continue;
            }

            let href = if navigation_catalog::is_frozen_destination(&destination.id)
                && let Some(route) = &destination.semantic_destination
            {
                let semantic_destination = SemanticDestination {
                    owner: ResourceOwner::CoreInstallation {
                        installation_id: policy.installation_id,
                    },
                    route: SemanticRouteName::new(route.clone()).map_err(|_| ())?,
                    parameters: BTreeMap::new(),
                };
                let resolution =
                    destination::resolve(&semantic_destination, policy.installation_id, account);
                if resolution.status != DestinationResolutionStatusV1::Resolved {
                    continue;
                }
                resolution.path.ok_or(())?
            } else {
                destination.route.clone()
            };
            if !is_same_origin_path(&href) {
                return Err(());
            }
            items.push(ShellNavigationItemV1 {
                key: destination.key.clone(),
                label: destination.label.clone(),
                href,
                owner: match destination.owner {
                    NavigationCatalogOwner::Core => ShellNavigationItemOwnerV1::Core,
                    NavigationCatalogOwner::Contribution => {
                        ShellNavigationItemOwnerV1::Contribution
                    }
                },
                contribution_id: (destination.owner == NavigationCatalogOwner::Contribution)
                    .then(|| destination.id.clone()),
                navigation_mode: match destination.definition_id.as_deref() {
                    None => ShellNavigationModeV1::Shell,
                    Some(definition) if lifecycle_definitions.contains(definition) => {
                        ShellNavigationModeV1::Shell
                    }
                    Some(definition) if installed_definitions.contains(definition) => {
                        ShellNavigationModeV1::Document
                    }
                    // Transitional contributions are still rendered by Core.
                    Some(_) => ShellNavigationModeV1::Shell,
                },
            });
        }
        if !items.is_empty() {
            groups.push(ShellNavigationGroupV1 {
                id: group.id.clone(),
                name: group.label.clone(),
                items,
            });
        }
    }
    Ok(groups)
}

fn unavailable_response(
    account: &AccountContext,
    policy_revision: Option<i64>,
) -> ShellNavigationResponseV1 {
    ShellNavigationResponseV1 {
        schema_version: SHELL_NAVIGATION_SCHEMA_VERSION_V3,
        policy_revision,
        state: ShellNavigationStateV1::Unavailable,
        groups: fail_closed_core_groups(account),
        unavailable: Some(ShellNavigationUnavailableV1 {
            code: "shell_navigation_unavailable".to_string(),
            message: "Configured navigation is temporarily unavailable.".to_string(),
        }),
    }
}

fn fail_closed_core_groups(account: &AccountContext) -> Vec<ShellNavigationGroupV1> {
    [("core.main", "Main"), ("core.admin", "Admin")]
        .into_iter()
        .filter_map(|(group_id, label)| {
            let items = navigation_catalog::DESTINATIONS
                .iter()
                .filter(|destination| {
                    destination.owner == NavigationCatalogOwner::Core
                        && destination.default_group_id == group_id
                        && (destination.required_capabilities_any_of.is_empty()
                            || destination
                                .required_capabilities_any_of
                                .iter()
                                .any(|required| account.has_capability(required)))
                })
                .map(|destination| ShellNavigationItemV1 {
                    key: destination.key.to_string(),
                    label: destination.label.to_string(),
                    href: destination.route.to_string(),
                    owner: ShellNavigationItemOwnerV1::Core,
                    contribution_id: None,
                    navigation_mode: ShellNavigationModeV1::Shell,
                })
                .collect::<Vec<_>>();
            (!items.is_empty()).then(|| ShellNavigationGroupV1 {
                id: group_id.to_string(),
                name: label.to_string(),
                items,
            })
        })
        .collect()
}

async fn browser_delivery_inventory(
    pool: &sqlx::PgPool,
    installation_id: Uuid,
) -> Result<(BTreeSet<String>, BTreeSet<String>), sqlx::Error> {
    let rows = sqlx::query(
        "SELECT releases.manifest
         FROM module_instances instances
         JOIN module_releases releases ON releases.id=instances.release_id
         WHERE instances.installation_id=$1
           AND instances.identity_state='live' AND instances.installed
           AND instances.deployed AND instances.configured AND instances.enabled
           AND instances.ready AND instances.healthy
           AND releases.manifest IS NOT NULL",
    )
    .bind(installation_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| row.try_get::<sqlx::types::Json<ModuleManifest>, _>("manifest"))
        .collect::<Result<Vec<_>, _>>()
        .map(|manifests| {
            let mut installed = BTreeSet::new();
            let mut lifecycle = BTreeSet::new();
            for manifest in manifests {
                let manifest = manifest.0;
                let definition = manifest.definition_id.as_str().to_string();
                installed.insert(definition.clone());
                if manifest.browser_lifecycle.is_some() {
                    lifecycle.insert(definition);
                }
            }
            (installed, lifecycle)
        })
}

fn is_same_origin_path(path: &str) -> bool {
    path.starts_with('/') && !path.starts_with("//") && !path.contains(['\r', '\n'])
}

#[cfg(test)]
mod tests {
    use super::{
        ShellNavigationGroupV1, ShellNavigationItemOwnerV1, ShellNavigationItemV1,
        ShellNavigationModeV1, ShellNavigationResponseV1, ShellNavigationStateV1,
        is_same_origin_path, navigation_catalog, navigation_projection, unavailable_response,
        validate_installation_binding,
    };
    use crate::auth::AccountContext;
    use uuid::Uuid;

    fn item(
        key: &str,
        label: &str,
        href: &str,
        contribution_id: Option<&str>,
    ) -> ShellNavigationItemV1 {
        ShellNavigationItemV1 {
            key: key.to_string(),
            label: label.to_string(),
            href: href.to_string(),
            owner: if contribution_id.is_some() {
                ShellNavigationItemOwnerV1::Contribution
            } else {
                ShellNavigationItemOwnerV1::Core
            },
            contribution_id: contribution_id.map(str::to_string),
            navigation_mode: ShellNavigationModeV1::Document,
        }
    }

    #[test]
    fn shell_paths_remain_same_origin() {
        assert!(is_same_origin_path("/administration/modules"));
        assert!(!is_same_origin_path("https://example.invalid"));
        assert!(!is_same_origin_path("//example.invalid"));
        assert!(!is_same_origin_path("/safe\nset-cookie: unsafe"));
    }

    #[test]
    fn extracted_module_destinations_do_not_use_the_transition_route_resolver() {
        assert!(navigation_catalog::is_frozen_destination(
            "tessara.forms.navigation"
        ));
        assert!(!navigation_catalog::is_frozen_destination(
            "tessara.dashboards.navigation"
        ));
        assert!(!navigation_catalog::is_frozen_destination(
            "tessara.reference.module-sdk.navigation"
        ));
    }

    #[test]
    fn module_shell_context_uses_the_complete_core_composed_navigation_order() {
        let response = ShellNavigationResponseV1 {
            schema_version: 3,
            policy_revision: Some(7),
            state: ShellNavigationStateV1::Available,
            groups: vec![
                ShellNavigationGroupV1 {
                    id: "core.main".into(),
                    name: "Main".into(),
                    items: vec![
                        item("home", "Home", "/", None),
                        item("forms", "Forms", "/forms", Some("tessara.forms.navigation")),
                        item(
                            "scoped_records",
                            "Scoped Records",
                            "/reference/scoped-records",
                            Some("tessara.reference.scoped-records.navigation"),
                        ),
                        item(
                            "components",
                            "Components",
                            "/components",
                            Some("tessara.components.navigation"),
                        ),
                        item(
                            "dashboards",
                            "Dashboard",
                            "/dashboards",
                            Some("tessara.dashboards.navigation"),
                        ),
                    ],
                },
                ShellNavigationGroupV1 {
                    id: "core.admin".into(),
                    name: "Admin".into(),
                    items: vec![item(
                        "module_management",
                        "Module Management",
                        "/administration/modules",
                        None,
                    )],
                },
            ],
            unavailable: None,
        };

        let projection = navigation_projection(&response).expect("projection is valid");
        let identities = projection
            .iter()
            .map(|item| item.contribution_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            identities,
            vec![
                "core.home",
                "tessara.forms.navigation",
                "tessara.reference.scoped-records.navigation",
                "tessara.components.navigation",
                "tessara.dashboards.navigation",
                "core.admin.modules",
            ]
        );
        assert_eq!(
            identities
                .iter()
                .filter(|identity| **identity == "tessara.dashboards.navigation")
                .count(),
            1,
            "Dashboard is projected exactly once from its real manifest identity"
        );
    }

    #[test]
    fn module_shell_context_retains_exact_fail_closed_core_projection() {
        let account = AccountContext {
            account_id: Uuid::nil(),
            email: "actor@example.test".into(),
            display_name: "Actor".into(),
            is_active: true,
            roles: Vec::new(),
            capabilities: Vec::new(),
            capability_scopes: Vec::new(),
            scope_nodes: Vec::new(),
            delegations: Vec::new(),
        };
        let response = unavailable_response(&account, None);
        let projection = navigation_projection(&response).expect("fallback projection is valid");

        assert_eq!(response.state, ShellNavigationStateV1::Unavailable);
        assert_eq!(projection.len(), 1);
        assert_eq!(projection[0].contribution_id.as_str(), "core.home");
        assert_eq!(projection[0].href, "/");
    }

    #[test]
    fn module_shell_context_rejects_another_installation_identity() {
        let canonical = Uuid::new_v4();
        assert_eq!(validate_installation_binding(canonical, canonical), Ok(()));
        assert_eq!(
            validate_installation_binding(Uuid::new_v4(), canonical),
            Err(())
        );
    }
}
