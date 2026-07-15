//! Authenticated, per-actor shell navigation projection.
//!
//! Navigation visibility is a usability projection only. Product routes keep
//! enforcing their own authorization, and a malformed catalog or policy
//! removes every contribution rather than broadening access.

use std::collections::{BTreeMap, BTreeSet};

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use tessara_module_contract::{ResourceOwner, SemanticDestination, SemanticRouteName};

use crate::{
    auth::{AccountContext, AuthenticatedRequest},
    db::AppState,
};

use super::{
    destination,
    dto::DestinationResolutionStatusV1,
    service::{self, NavigationPolicyEntry, NavigationPolicyReadModel},
};

const SHELL_NAVIGATION_SCHEMA_VERSION_V1: u16 = 1;
const MAIN_BETWEEN_ORGANIZATION_AND_OPERATIONS: &str = "main_between_organization_and_operations";
const MAIN_AFTER_OPERATIONS: &str = "main_after_operations";
const ADMIN_BETWEEN_ADMINISTRATION_AND_MODULE_MANAGEMENT: &str =
    "admin_between_administration_and_module_management";

#[derive(Clone, Copy)]
struct ContributionSpec {
    contribution_id: &'static str,
    definition_id: &'static str,
    key: &'static str,
    destination: &'static str,
    label: &'static str,
    group: &'static str,
    reorder_band: &'static str,
    source_order_hint: i32,
    default_policy_order: i32,
    required_capabilities_any_of: &'static [&'static str],
}

const CONTRIBUTIONS: [ContributionSpec; 6] = [
    ContributionSpec {
        contribution_id: "tessara.forms.navigation",
        definition_id: "tessara.forms",
        key: "forms",
        destination: "forms.directory",
        label: "Forms",
        group: "Main",
        reorder_band: MAIN_BETWEEN_ORGANIZATION_AND_OPERATIONS,
        source_order_hint: 20,
        default_policy_order: 0,
        required_capabilities_any_of: &["forms:read", "forms:manage"],
    },
    ContributionSpec {
        contribution_id: "tessara.workflows.navigation",
        definition_id: "tessara.workflows",
        key: "workflows",
        destination: "workflows.directory",
        label: "Workflows",
        group: "Main",
        reorder_band: MAIN_BETWEEN_ORGANIZATION_AND_OPERATIONS,
        source_order_hint: 30,
        default_policy_order: 1,
        required_capabilities_any_of: &["workflows:read", "workflows:manage"],
    },
    ContributionSpec {
        contribution_id: "tessara.responses.navigation",
        definition_id: "tessara.responses",
        key: "responses",
        destination: "responses.directory",
        label: "Responses",
        group: "Main",
        reorder_band: MAIN_BETWEEN_ORGANIZATION_AND_OPERATIONS,
        source_order_hint: 40,
        default_policy_order: 2,
        required_capabilities_any_of: &[
            "submissions:read_own",
            "submissions:respond",
            "submissions:manage",
        ],
    },
    ContributionSpec {
        contribution_id: "tessara.components.navigation",
        definition_id: "tessara.components",
        key: "components",
        destination: "components.directory",
        label: "Components",
        group: "Main",
        reorder_band: MAIN_AFTER_OPERATIONS,
        source_order_hint: 60,
        default_policy_order: 0,
        required_capabilities_any_of: &["components:read", "components:manage"],
    },
    ContributionSpec {
        contribution_id: "tessara.dashboards.navigation",
        definition_id: "tessara.dashboards",
        key: "dashboards",
        destination: "dashboards.directory",
        label: "Dashboards",
        group: "Main",
        reorder_band: MAIN_AFTER_OPERATIONS,
        source_order_hint: 70,
        default_policy_order: 1,
        required_capabilities_any_of: &["dashboards:read"],
    },
    ContributionSpec {
        contribution_id: "tessara.datasets.navigation",
        definition_id: "tessara.datasets",
        key: "datasets",
        destination: "datasets.directory",
        label: "Datasets",
        group: "Admin",
        reorder_band: ADMIN_BETWEEN_ADMINISTRATION_AND_MODULE_MANAGEMENT,
        source_order_hint: 20,
        default_policy_order: 0,
        required_capabilities_any_of: &["datasets:read", "datasets:manage"],
    },
];

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
    /// Present only when contribution composition failed. Core items remain.
    pub(crate) unavailable: Option<ShellNavigationUnavailableV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ShellNavigationGroupV1 {
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
) -> Json<ShellNavigationResponseV1> {
    Json(load_response(&state, &auth.account).await)
}

pub(super) async fn load_response(
    state: &AppState,
    account: &AccountContext,
) -> ShellNavigationResponseV1 {
    match service::load_navigation_policy(&state.pool).await {
        Ok(policy) => compose_response(policy, account),
        Err(error) => {
            tracing::warn!(
                code = error.stable_code(),
                "shell contribution navigation is unavailable"
            );
            unavailable_response(account, None)
        }
    }
}

fn compose_response(
    policy: NavigationPolicyReadModel,
    account: &AccountContext,
) -> ShellNavigationResponseV1 {
    let revision = Some(policy.revision);
    match compose_groups(&policy, account) {
        Ok(groups) => ShellNavigationResponseV1 {
            schema_version: SHELL_NAVIGATION_SCHEMA_VERSION_V1,
            policy_revision: revision,
            state: ShellNavigationStateV1::Available,
            groups,
            unavailable: None,
        },
        Err(()) => unavailable_response(account, revision),
    }
}

fn unavailable_response(
    account: &AccountContext,
    policy_revision: Option<i64>,
) -> ShellNavigationResponseV1 {
    ShellNavigationResponseV1 {
        schema_version: SHELL_NAVIGATION_SCHEMA_VERSION_V1,
        policy_revision,
        state: ShellNavigationStateV1::Unavailable,
        groups: core_groups(account),
        unavailable: Some(ShellNavigationUnavailableV1 {
            code: "shell_navigation_unavailable".to_string(),
            message: "Contribution navigation is temporarily unavailable.".to_string(),
        }),
    }
}

fn compose_groups(
    policy: &NavigationPolicyReadModel,
    account: &AccountContext,
) -> Result<Vec<ShellNavigationGroupV1>, ()> {
    validate_policy(policy)?;

    let mut main = Vec::new();
    let mut admin = Vec::new();
    push_core(&mut main, "home", account);
    push_core(&mut main, "organization", account);
    push_band(
        &mut main,
        MAIN_BETWEEN_ORGANIZATION_AND_OPERATIONS,
        policy,
        account,
    )?;
    push_core(&mut main, "operations", account);
    push_band(&mut main, MAIN_AFTER_OPERATIONS, policy, account)?;
    push_core(&mut admin, "administration", account);
    push_band(
        &mut admin,
        ADMIN_BETWEEN_ADMINISTRATION_AND_MODULE_MANAGEMENT,
        policy,
        account,
    )?;
    push_core(&mut admin, "module_management", account);

    Ok(groups(main, admin))
}

fn core_groups(account: &AccountContext) -> Vec<ShellNavigationGroupV1> {
    let mut main = Vec::new();
    let mut admin = Vec::new();
    for key in ["home", "organization", "operations"] {
        push_core(&mut main, key, account);
    }
    for key in ["administration", "module_management"] {
        push_core(&mut admin, key, account);
    }
    groups(main, admin)
}

fn groups(
    main: Vec<ShellNavigationItemV1>,
    admin: Vec<ShellNavigationItemV1>,
) -> Vec<ShellNavigationGroupV1> {
    let mut groups = vec![ShellNavigationGroupV1 {
        name: "Main".to_string(),
        items: main,
    }];
    if !admin.is_empty() {
        groups.push(ShellNavigationGroupV1 {
            name: "Admin".to_string(),
            items: admin,
        });
    }
    groups
}

fn push_core(items: &mut Vec<ShellNavigationItemV1>, key: &'static str, account: &AccountContext) {
    let (label, href, visible) = match key {
        "home" => ("Home", "/", true),
        "organization" => (
            "Organization",
            "/organization",
            account.has_capability("hierarchy:read"),
        ),
        "operations" => (
            "Operations",
            "/operations",
            account.has_capability("operations:view"),
        ),
        "administration" => (
            "Administration",
            "/administration",
            account.has_capability("admin:all"),
        ),
        "module_management" => (
            "Module Management",
            "/administration/modules",
            account.has_global_capability("modules:read"),
        ),
        _ => unreachable!("only frozen Core navigation keys are composed"),
    };
    if visible {
        items.push(ShellNavigationItemV1 {
            key: key.to_string(),
            label: label.to_string(),
            href: href.to_string(),
            owner: ShellNavigationItemOwnerV1::Core,
            contribution_id: None,
        });
    }
}

fn push_band(
    items: &mut Vec<ShellNavigationItemV1>,
    band: &str,
    policy: &NavigationPolicyReadModel,
    account: &AccountContext,
) -> Result<(), ()> {
    let mut entries = policy
        .entries
        .iter()
        .filter(|entry| entry.reorder_band == band)
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.contribution_id.cmp(&right.contribution_id))
    });

    for entry in entries {
        if !entry.visible {
            continue;
        }
        let spec = contribution_spec(&entry.contribution_id).ok_or(())?;
        if !entry
            .required_capabilities_any_of
            .iter()
            .any(|required| account.has_capability(required))
        {
            continue;
        }

        let semantic_destination = SemanticDestination {
            owner: ResourceOwner::CoreInstallation {
                installation_id: policy.installation_id,
            },
            route: SemanticRouteName::new(entry.destination.clone()).map_err(|_| ())?,
            parameters: BTreeMap::new(),
        };
        let resolution =
            destination::resolve(&semantic_destination, policy.installation_id, account);
        if resolution.status != DestinationResolutionStatusV1::Resolved {
            return Err(());
        }
        let href = resolution.path.ok_or(())?;
        if !is_same_origin_path(&href) {
            return Err(());
        }
        items.push(ShellNavigationItemV1 {
            key: spec.key.to_string(),
            label: entry.label.clone(),
            href,
            owner: ShellNavigationItemOwnerV1::Contribution,
            contribution_id: Some(entry.contribution_id.clone()),
        });
    }
    Ok(())
}

fn validate_policy(policy: &NavigationPolicyReadModel) -> Result<(), ()> {
    if policy.revision < 0 || policy.entries.len() != CONTRIBUTIONS.len() {
        return Err(());
    }

    let mut seen = BTreeSet::new();
    for entry in &policy.entries {
        let spec = contribution_spec(&entry.contribution_id).ok_or(())?;
        if !seen.insert(entry.contribution_id.as_str())
            || entry.definition_id != spec.definition_id
            || entry.destination != spec.destination
            || entry.label != spec.label
            || entry.group != spec.group
            || entry.reorder_band != spec.reorder_band
            || entry.source_order_hint != spec.source_order_hint
            || entry.default_policy_order != spec.default_policy_order
            || entry.order < 0
            || !capabilities_match(entry, spec)
        {
            return Err(());
        }
    }

    for band in [
        MAIN_BETWEEN_ORGANIZATION_AND_OPERATIONS,
        MAIN_AFTER_OPERATIONS,
        ADMIN_BETWEEN_ADMINISTRATION_AND_MODULE_MANAGEMENT,
    ] {
        let mut orders = policy
            .entries
            .iter()
            .filter(|entry| entry.reorder_band == band)
            .map(|entry| entry.order)
            .collect::<Vec<_>>();
        orders.sort_unstable();
        if orders
            .iter()
            .copied()
            .ne((0..orders.len()).map(|order| order as i32))
        {
            return Err(());
        }
    }
    Ok(())
}

fn capabilities_match(entry: &NavigationPolicyEntry, spec: ContributionSpec) -> bool {
    entry.required_capabilities_any_of.len() == spec.required_capabilities_any_of.len()
        && entry
            .required_capabilities_any_of
            .iter()
            .map(String::as_str)
            .eq(spec.required_capabilities_any_of.iter().copied())
}

fn contribution_spec(contribution_id: &str) -> Option<ContributionSpec> {
    CONTRIBUTIONS
        .iter()
        .copied()
        .find(|spec| spec.contribution_id == contribution_id)
}

fn is_same_origin_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains('?')
        && !path.contains('#')
        && !path.chars().any(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::auth::CapabilityScope;

    use super::*;

    type NavigationCase<'a> = (&'a str, &'a [(&'a str, bool)], &'a [&'a str], &'a [&'a str]);

    #[test]
    fn default_navigation_sequences_are_exact_for_named_actors() {
        let policy = default_policy();
        let cases: &[NavigationCase<'_>] = &[
            (
                "admin_all",
                &[("admin:all", true)],
                &[
                    "home",
                    "organization",
                    "forms",
                    "workflows",
                    "responses",
                    "operations",
                    "components",
                    "dashboards",
                ],
                &["administration", "datasets", "module_management"],
            ),
            (
                "operator",
                &[
                    ("hierarchy:read", false),
                    ("forms:read", false),
                    ("workflows:read", false),
                    ("submissions:respond", false),
                    ("operations:view", false),
                    ("datasets:read", false),
                    ("components:read", false),
                    ("dashboards:read", false),
                ],
                &[
                    "home",
                    "organization",
                    "forms",
                    "workflows",
                    "responses",
                    "operations",
                    "components",
                    "dashboards",
                ],
                &["datasets"],
            ),
            (
                "respondent",
                &[
                    ("submissions:read_own", false),
                    ("submissions:respond", false),
                ],
                &["home", "responses"],
                &[],
            ),
            (
                "dashboards_manage_only",
                &[("dashboards:manage", false)],
                &["home"],
                &[],
            ),
            (
                "global_modules_read",
                &[("modules:read", true)],
                &["home"],
                &["module_management"],
            ),
            (
                "global_modules_manage_without_read_row",
                &[("modules:manage_navigation", true)],
                &["home"],
                &["module_management"],
            ),
            ("no_access", &[], &["home"], &[]),
        ];

        for (name, scopes, expected_main, expected_admin) in cases {
            let response = compose_response(policy.clone(), &account(scopes));
            assert_eq!(response.state, ShellNavigationStateV1::Available, "{name}");
            assert_eq!(keys(&response, "Main"), *expected_main, "{name} Main");
            assert_eq!(keys(&response, "Admin"), *expected_admin, "{name} Admin");
        }

        let admin = compose_response(policy, &account(&[("admin:all", true)]));
        let exact = admin
            .groups
            .iter()
            .flat_map(|group| {
                group.items.iter().map(move |item| {
                    (
                        group.name.as_str(),
                        item.key.as_str(),
                        item.label.as_str(),
                        item.href.as_str(),
                    )
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            exact,
            vec![
                ("Main", "home", "Home", "/"),
                ("Main", "organization", "Organization", "/organization"),
                ("Main", "forms", "Forms", "/forms"),
                ("Main", "workflows", "Workflows", "/workflows"),
                ("Main", "responses", "Responses", "/responses"),
                ("Main", "operations", "Operations", "/operations"),
                ("Main", "components", "Components", "/components"),
                ("Main", "dashboards", "Dashboards", "/dashboards"),
                (
                    "Admin",
                    "administration",
                    "Administration",
                    "/administration"
                ),
                ("Admin", "datasets", "Datasets", "/datasets"),
                (
                    "Admin",
                    "module_management",
                    "Module Management",
                    "/administration/modules",
                ),
            ]
        );
        assert!(
            exact
                .iter()
                .all(|(_, _, _, href)| is_same_origin_path(href))
        );
    }

    #[test]
    fn hidden_contribution_does_not_revoke_authorized_destination() {
        let mut policy = default_policy();
        policy
            .entries
            .iter_mut()
            .find(|entry| entry.contribution_id == "tessara.forms.navigation")
            .expect("Forms contribution")
            .visible = false;
        let account = account(&[("forms:read", false)]);

        let response = compose_response(policy.clone(), &account);
        assert_eq!(keys(&response, "Main"), ["home"]);
        assert_eq!(response.state, ShellNavigationStateV1::Available);

        let direct = destination::resolve(
            &SemanticDestination {
                owner: ResourceOwner::CoreInstallation {
                    installation_id: policy.installation_id,
                },
                route: SemanticRouteName::new("forms.directory").expect("fixed route"),
                parameters: BTreeMap::new(),
            },
            policy.installation_id,
            &account,
        );
        assert_eq!(direct.status, DestinationResolutionStatusV1::Resolved);
        assert_eq!(direct.path.as_deref(), Some("/forms"));
    }

    #[test]
    fn malformed_band_falls_back_to_filtered_core_and_marks_unavailable() {
        let mut policy = default_policy();
        policy.entries[0].reorder_band = MAIN_AFTER_OPERATIONS.to_string();
        let response = compose_response(policy, &account(&[("admin:all", true)]));

        assert_eq!(response.state, ShellNavigationStateV1::Unavailable);
        assert!(response.unavailable.is_some());
        assert_eq!(
            keys(&response, "Main"),
            ["home", "organization", "operations"]
        );
        assert_eq!(
            keys(&response, "Admin"),
            ["administration", "module_management"]
        );
        assert!(
            response
                .groups
                .iter()
                .flat_map(|group| &group.items)
                .all(|item| {
                    item.owner == ShellNavigationItemOwnerV1::Core && item.contribution_id.is_none()
                })
        );
    }

    #[test]
    fn scoped_module_capabilities_never_expose_module_management() {
        for capability in ["modules:read", "modules:manage_navigation"] {
            let response = compose_response(default_policy(), &account(&[(capability, false)]));
            assert_eq!(keys(&response, "Main"), ["home"]);
            assert!(keys(&response, "Admin").is_empty());
        }
    }

    #[test]
    fn dashboard_directory_resolution_preserves_read_only_exception() {
        let policy = default_policy();
        let manage_only = account(&[("dashboards:manage", false)]);
        let direct = destination::resolve(
            &SemanticDestination {
                owner: ResourceOwner::CoreInstallation {
                    installation_id: policy.installation_id,
                },
                route: SemanticRouteName::new("dashboards.directory").expect("fixed route"),
                parameters: BTreeMap::new(),
            },
            policy.installation_id,
            &manage_only,
        );

        assert_eq!(direct.status, DestinationResolutionStatusV1::Rejected);
        assert_eq!(
            keys(&compose_response(policy, &manage_only), "Main"),
            ["home"]
        );
    }

    fn default_policy() -> NavigationPolicyReadModel {
        NavigationPolicyReadModel {
            installation_id: Uuid::parse_str("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa")
                .expect("fixed installation id"),
            revision: 0,
            entries: CONTRIBUTIONS
                .iter()
                .map(|spec| NavigationPolicyEntry {
                    contribution_id: spec.contribution_id.to_string(),
                    definition_id: spec.definition_id.to_string(),
                    destination: spec.destination.to_string(),
                    label: spec.label.to_string(),
                    group: spec.group.to_string(),
                    reorder_band: spec.reorder_band.to_string(),
                    source_order_hint: spec.source_order_hint,
                    default_policy_order: spec.default_policy_order,
                    required_capabilities_any_of: spec
                        .required_capabilities_any_of
                        .iter()
                        .map(|capability| (*capability).to_string())
                        .collect(),
                    visible: true,
                    order: spec.default_policy_order,
                })
                .collect(),
        }
    }

    fn account(scopes: &[(&str, bool)]) -> AccountContext {
        AccountContext {
            account_id: Uuid::nil(),
            email: "shell-navigation@example.test".to_string(),
            display_name: "Shell Navigation".to_string(),
            is_active: true,
            roles: Vec::new(),
            capabilities: scopes
                .iter()
                .map(|(capability, _)| (*capability).to_string())
                .collect(),
            capability_scopes: scopes
                .iter()
                .map(|(capability, global)| CapabilityScope {
                    capability: (*capability).to_string(),
                    global: *global,
                    node_ids: Vec::new(),
                })
                .collect(),
            scope_nodes: Vec::new(),
            delegations: Vec::new(),
        }
    }

    fn keys<'a>(response: &'a ShellNavigationResponseV1, group: &str) -> Vec<&'a str> {
        response
            .groups
            .iter()
            .find(|candidate| candidate.name == group)
            .map(|group| group.items.iter().map(|item| item.key.as_str()).collect())
            .unwrap_or_default()
    }
}
