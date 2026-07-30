//! Authenticated, per-actor shell navigation projection.
//!
//! The schema-v2 policy owns ordered groups and destination placement. Core's
//! catalog remains authoritative for labels, routes, ownership, protection,
//! and capability predicates. Route authorization remains independent.

use std::collections::BTreeMap;

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
use tessara_module_contract::{ResourceOwner, SemanticDestination, SemanticRouteName};

use crate::{
    auth::{AccountContext, AuthenticatedRequest},
    db::AppState,
};

use super::{
    destination,
    dto::DestinationResolutionStatusV1,
    navigation_catalog::{self, NavigationCatalogOwner},
    service::{self, NavigationPolicyReadModelV2},
};

const SHELL_NAVIGATION_SCHEMA_VERSION_V2: u16 = 2;

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ShellNavigationItemV1 {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) href: String,
    pub(crate) owner: ShellNavigationItemOwnerV1,
    pub(crate) contribution_id: Option<String>,
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
    match service::load_navigation_policy_v2(&state.pool).await {
        Ok(policy) => match compose_groups(&policy, account) {
            Ok(groups) => ShellNavigationResponseV1 {
                schema_version: SHELL_NAVIGATION_SCHEMA_VERSION_V2,
                policy_revision: Some(policy.revision),
                state: ShellNavigationStateV1::Available,
                groups,
                unavailable: None,
            },
            Err(()) => unavailable_response(account, Some(policy.revision)),
        },
        Err(error) => {
            tracing::warn!(
                code = error.stable_code(),
                "shell navigation policy is unavailable"
            );
            unavailable_response(account, None)
        }
    }
}

fn compose_groups(
    policy: &NavigationPolicyReadModelV2,
    account: &AccountContext,
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
        schema_version: SHELL_NAVIGATION_SCHEMA_VERSION_V2,
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

fn is_same_origin_path(path: &str) -> bool {
    path.starts_with('/') && !path.starts_with("//") && !path.contains(['\r', '\n'])
}

#[cfg(test)]
mod tests {
    use super::{is_same_origin_path, navigation_catalog};

    #[test]
    fn shell_paths_remain_same_origin() {
        assert!(is_same_origin_path("/administration/modules"));
        assert!(!is_same_origin_path("https://example.invalid"));
        assert!(!is_same_origin_path("//example.invalid"));
        assert!(!is_same_origin_path("/safe\nset-cookie: unsafe"));
    }

    #[test]
    fn only_frozen_destinations_require_the_transition_route_resolver() {
        assert!(navigation_catalog::is_frozen_destination(
            "tessara.dashboards.navigation"
        ));
        assert!(!navigation_catalog::is_frozen_destination(
            "tessara.reference.module-sdk.navigation"
        ));
    }
}
