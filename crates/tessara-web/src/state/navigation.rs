//! Shell navigation model and capability filtering.
//!
//! Keep route labels, navigation sections, icon selection, and permission checks here; feature modules should not duplicate sidebar metadata.

#[derive(Clone, Copy)]
pub struct NavItem {
    pub key: &'static str,
    pub href: &'static str,
    pub label: &'static str,
    pub section: &'static str,
    pub capabilities: &'static [&'static str],
}

pub const NAV_ITEMS: [NavItem; 10] = [
    NavItem {
        key: "home",
        href: "/",
        label: "Home",
        section: "Main",
        capabilities: &[],
    },
    NavItem {
        key: "organization",
        href: "/organization",
        label: "Organization",
        section: "Main",
        capabilities: &["hierarchy:read", "hierarchy:manage"],
    },
    NavItem {
        key: "forms",
        href: "/forms",
        label: "Forms",
        section: "Main",
        capabilities: &["forms:read", "forms:manage"],
    },
    NavItem {
        key: "workflows",
        href: "/workflows",
        label: "Workflows",
        section: "Main",
        capabilities: &["workflows:read", "workflows:manage"],
    },
    NavItem {
        key: "responses",
        href: "/responses",
        label: "Responses",
        section: "Main",
        capabilities: &[
            "submissions:read_own",
            "submissions:respond",
            "submissions:manage",
        ],
    },
    NavItem {
        key: "operations",
        href: "/operations",
        label: "Operations",
        section: "Main",
        capabilities: &["operations:view"],
    },
    NavItem {
        key: "components",
        href: "/components",
        label: "Components",
        section: "Main",
        capabilities: &["components:read", "components:manage"],
    },
    NavItem {
        key: "dashboards",
        href: "/dashboards",
        label: "Dashboards",
        section: "Main",
        capabilities: &["dashboards:read", "dashboards:manage"],
    },
    NavItem {
        key: "administration",
        href: "/administration",
        label: "Administration",
        section: "Admin",
        capabilities: &["admin:all"],
    },
    NavItem {
        key: "datasets",
        href: "/datasets",
        label: "Datasets",
        section: "Admin",
        capabilities: &["datasets:read", "datasets:manage"],
    },
];

pub fn nav_item_for_route(route_key: &str) -> Option<&'static NavItem> {
    NAV_ITEMS.iter().find(|item| item.key == route_key)
}

pub fn nav_item_is_allowed(item: &NavItem, capabilities: &[String]) -> bool {
    item.capabilities.is_empty()
        || capabilities
            .iter()
            .any(|capability| capability == "admin:all")
        || item
            .capabilities
            .iter()
            .any(|required| capabilities.iter().any(|capability| capability == required))
}

/// Whether a permitted route also has a useful directory entrypoint.
///
/// Object-scoped Dashboard managers can open an editor URL issued for a
/// Dashboard they manage, but `/dashboards` itself is a reader directory. Do
/// not advertise that reader link when the account only has manage access.
pub fn nav_item_is_visible(item: &NavItem, capabilities: &[String]) -> bool {
    if item.key == "dashboards" {
        return capabilities
            .iter()
            .any(|capability| capability == "dashboards:read" || capability == "admin:all");
    }
    nav_item_is_allowed(item, capabilities)
}

pub fn nav_items_for_section(
    section: &'static str,
    capabilities: &[String],
) -> Vec<&'static NavItem> {
    NAV_ITEMS
        .iter()
        .filter(move |item| item.section == section)
        .filter(|item| nav_item_is_visible(item, capabilities))
        .collect::<Vec<_>>()
}

/// A shell navigation group supported by the Sprint 6A composition contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NavigationSection {
    Main,
    Admin,
}

impl NavigationSection {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Main => "Main",
            Self::Admin => "Admin",
        }
    }
}

/// A Core-owned ordering band. Contributions may move only inside their band.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NavigationBand {
    MainBetweenOrganizationAndOperations,
    MainAfterOperations,
    AdminBetweenAdministrationAndModuleManagement,
}

impl NavigationBand {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MainBetweenOrganizationAndOperations => {
                "main_between_organization_and_operations"
            }
            Self::MainAfterOperations => "main_after_operations",
            Self::AdminBetweenAdministrationAndModuleManagement => {
                "admin_between_administration_and_module_management"
            }
        }
    }

    const fn section(self) -> NavigationSection {
        match self {
            Self::MainBetweenOrganizationAndOperations | Self::MainAfterOperations => {
                NavigationSection::Main
            }
            Self::AdminBetweenAdministrationAndModuleManagement => NavigationSection::Admin,
        }
    }
}

/// Whether the contribution's owning module can currently supply product navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleNavigationAvailability {
    Available,
    Unavailable,
}

/// Owned contribution data supplied by the Core shell-navigation projection.
///
/// `href` is the already-resolved, same-origin semantic destination. The shell
/// does not persist or manufacture deployment URLs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributedNavigationItem {
    pub contribution_id: String,
    pub key: String,
    pub href: String,
    pub label: String,
    pub section: NavigationSection,
    pub band: NavigationBand,
    pub default_order: u32,
    pub required_capabilities_any_of: Vec<String>,
    pub module_availability: ModuleNavigationAvailability,
}

/// One complete, revisioned administrator policy member.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationPolicyEntry {
    pub contribution_id: String,
    pub visible: bool,
    pub order: u32,
}

/// The policy projection is an atomic collection replacement.
///
/// A supplied policy must contain exactly one entry for every contribution.
/// The resolver accepts no policy only when the catalog has no contributions;
/// a missing policy for a non-empty catalog is a composition failure, not
/// permission to expose catalog defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationPolicy {
    pub revision: u64,
    pub entries: Vec<NavigationPolicyEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationItemOwner {
    Core,
    Contribution,
}

/// The owned item consumed by both desktop and mobile shell renderers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedNavigationItem {
    pub key: String,
    pub href: String,
    pub label: String,
    pub section: NavigationSection,
    pub owner: NavigationItemOwner,
    pub contribution_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationCompositionError {
    InvalidContribution { contribution_id: String },
    DuplicateContributionId { contribution_id: String },
    DuplicateItemKey { key: String },
    DuplicateItemHref { href: String },
    ContributionCollidesWithCore { key: String },
    ContributionInWrongBand { contribution_id: String },
    MissingPolicy,
    DuplicatePolicyEntry { contribution_id: String },
    UnknownPolicyContribution { contribution_id: String },
    MissingPolicyContribution { contribution_id: String },
    PolicyTargetsCore { key: String },
}

/// A successful composition or a Core-only, fail-closed fallback.
///
/// On malformed catalog/policy input, contributions are omitted wholesale and
/// `unavailable` carries the reason so the shell can render an explicit state.
/// Route authorization remains independent of this model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedNavigation {
    pub items: Vec<ResolvedNavigationItem>,
    pub unavailable: Option<NavigationCompositionError>,
}

const CORE_NAV_ITEMS: [NavItem; 5] = [
    NAV_ITEMS[0],
    NAV_ITEMS[1],
    NAV_ITEMS[5],
    NAV_ITEMS[8],
    NavItem {
        key: "module_management",
        href: "/administration/modules",
        label: "Module Management",
        section: "Admin",
        capabilities: &["modules:read"],
    },
];

/// Compose policy, module state, and actor display eligibility into one model.
///
/// Invalid contribution or policy data never becomes navigation. Instead, the
/// resolver returns capability-filtered permanent Core destinations and marks
/// the contribution model unavailable.
pub fn resolve_navigation(
    contributions: &[ContributedNavigationItem],
    policy: Option<&NavigationPolicy>,
    capabilities: &[String],
) -> ResolvedNavigation {
    match validate_composition(contributions, policy) {
        Ok(()) => ResolvedNavigation {
            items: compose_navigation(contributions, policy, capabilities),
            unavailable: None,
        },
        Err(error) => ResolvedNavigation {
            items: core_navigation(capabilities),
            unavailable: Some(error),
        },
    }
}

fn validate_composition(
    contributions: &[ContributedNavigationItem],
    policy: Option<&NavigationPolicy>,
) -> Result<(), NavigationCompositionError> {
    for (index, contribution) in contributions.iter().enumerate() {
        if !contribution_is_well_formed(contribution) {
            return Err(NavigationCompositionError::InvalidContribution {
                contribution_id: contribution.contribution_id.clone(),
            });
        }
        if contribution.section != contribution.band.section() {
            return Err(NavigationCompositionError::ContributionInWrongBand {
                contribution_id: contribution.contribution_id.clone(),
            });
        }
        if core_item_for_key(&contribution.key).is_some() {
            return Err(NavigationCompositionError::ContributionCollidesWithCore {
                key: contribution.key.clone(),
            });
        }
        if CORE_NAV_ITEMS
            .iter()
            .any(|core_item| core_item.href == contribution.href)
        {
            return Err(NavigationCompositionError::ContributionCollidesWithCore {
                key: contribution.key.clone(),
            });
        }

        for other in &contributions[..index] {
            if other.contribution_id == contribution.contribution_id {
                return Err(NavigationCompositionError::DuplicateContributionId {
                    contribution_id: contribution.contribution_id.clone(),
                });
            }
            if other.key == contribution.key {
                return Err(NavigationCompositionError::DuplicateItemKey {
                    key: contribution.key.clone(),
                });
            }
            if other.href == contribution.href {
                return Err(NavigationCompositionError::DuplicateItemHref {
                    href: contribution.href.clone(),
                });
            }
        }
    }

    let Some(policy) = policy else {
        return if contributions.is_empty() {
            Ok(())
        } else {
            Err(NavigationCompositionError::MissingPolicy)
        };
    };

    for (index, entry) in policy.entries.iter().enumerate() {
        if core_item_for_key(&entry.contribution_id).is_some() {
            return Err(NavigationCompositionError::PolicyTargetsCore {
                key: entry.contribution_id.clone(),
            });
        }
        if policy.entries[..index]
            .iter()
            .any(|other| other.contribution_id == entry.contribution_id)
        {
            return Err(NavigationCompositionError::DuplicatePolicyEntry {
                contribution_id: entry.contribution_id.clone(),
            });
        }
        if !contributions
            .iter()
            .any(|contribution| contribution.contribution_id == entry.contribution_id)
        {
            return Err(NavigationCompositionError::UnknownPolicyContribution {
                contribution_id: entry.contribution_id.clone(),
            });
        }
    }

    if let Some(missing) = contributions.iter().find(|contribution| {
        !policy
            .entries
            .iter()
            .any(|entry| entry.contribution_id == contribution.contribution_id)
    }) {
        return Err(NavigationCompositionError::MissingPolicyContribution {
            contribution_id: missing.contribution_id.clone(),
        });
    }

    Ok(())
}

fn contribution_is_well_formed(contribution: &ContributedNavigationItem) -> bool {
    is_navigation_identifier(&contribution.contribution_id)
        && is_route_key(&contribution.key)
        && !contribution.label.trim().is_empty()
        && contribution.label == contribution.label.trim()
        && is_same_origin_path(&contribution.href)
        && !contribution.required_capabilities_any_of.is_empty()
        && contribution
            .required_capabilities_any_of
            .iter()
            .all(|capability| {
                !capability.is_empty()
                    && !capability.chars().any(char::is_whitespace)
                    && capability.contains(':')
            })
        && contribution
            .required_capabilities_any_of
            .iter()
            .enumerate()
            .all(|(index, capability)| {
                !contribution.required_capabilities_any_of[..index].contains(capability)
            })
}

fn is_navigation_identifier(value: &str) -> bool {
    if value.is_empty() || matches!(value.chars().next(), Some('.' | ':' | '_' | '-')) {
        return false;
    }
    if matches!(value.chars().last(), Some('.' | ':' | '_' | '-')) {
        return false;
    }

    let mut previous_was_separator = false;
    for character in value.chars() {
        let is_separator = matches!(character, '.' | ':' | '_' | '-');
        if !(character.is_ascii_lowercase() || character.is_ascii_digit() || is_separator)
            || (is_separator && previous_was_separator)
        {
            return false;
        }
        previous_was_separator = is_separator;
    }
    true
}

fn is_route_key(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}

fn is_same_origin_path(value: &str) -> bool {
    value.starts_with('/')
        && !value.starts_with("//")
        && !value.contains('?')
        && !value.contains('#')
        && !value.chars().any(char::is_whitespace)
}

fn compose_navigation(
    contributions: &[ContributedNavigationItem],
    policy: Option<&NavigationPolicy>,
    capabilities: &[String],
) -> Vec<ResolvedNavigationItem> {
    let mut items = Vec::new();
    push_core_if_visible(&mut items, "home", capabilities);
    push_core_if_visible(&mut items, "organization", capabilities);
    push_contribution_band(
        &mut items,
        NavigationBand::MainBetweenOrganizationAndOperations,
        contributions,
        policy,
        capabilities,
    );
    push_core_if_visible(&mut items, "operations", capabilities);
    push_contribution_band(
        &mut items,
        NavigationBand::MainAfterOperations,
        contributions,
        policy,
        capabilities,
    );
    push_core_if_visible(&mut items, "administration", capabilities);
    push_contribution_band(
        &mut items,
        NavigationBand::AdminBetweenAdministrationAndModuleManagement,
        contributions,
        policy,
        capabilities,
    );
    push_core_if_visible(&mut items, "module_management", capabilities);
    items
}

fn core_navigation(capabilities: &[String]) -> Vec<ResolvedNavigationItem> {
    let mut items = Vec::new();
    for key in [
        "home",
        "organization",
        "operations",
        "administration",
        "module_management",
    ] {
        push_core_if_visible(&mut items, key, capabilities);
    }
    items
}

fn push_core_if_visible(
    items: &mut Vec<ResolvedNavigationItem>,
    key: &'static str,
    capabilities: &[String],
) {
    let item = core_item_for_key(key).expect("fixed Core navigation key must exist");
    if core_item_is_visible(item, capabilities) {
        items.push(ResolvedNavigationItem {
            key: item.key.to_string(),
            href: item.href.to_string(),
            label: item.label.to_string(),
            section: match item.section {
                "Main" => NavigationSection::Main,
                "Admin" => NavigationSection::Admin,
                _ => unreachable!("fixed Core navigation section must be supported"),
            },
            owner: NavigationItemOwner::Core,
            contribution_id: None,
        });
    }
}

fn push_contribution_band(
    items: &mut Vec<ResolvedNavigationItem>,
    band: NavigationBand,
    contributions: &[ContributedNavigationItem],
    policy: Option<&NavigationPolicy>,
    capabilities: &[String],
) {
    let mut in_band = contributions
        .iter()
        .filter(|contribution| contribution.band == band)
        .map(|contribution| {
            let policy_entry = policy.and_then(|policy| {
                policy
                    .entries
                    .iter()
                    .find(|entry| entry.contribution_id == contribution.contribution_id)
            });
            let order = policy_entry
                .map(|entry| entry.order)
                .unwrap_or(contribution.default_order);
            (order, contribution)
        })
        .collect::<Vec<_>>();

    in_band.sort_by(|(left_order, left), (right_order, right)| {
        left_order
            .cmp(right_order)
            .then_with(|| left.contribution_id.cmp(&right.contribution_id))
    });

    items.extend(
        in_band
            .into_iter()
            .filter(|(_, contribution)| {
                let policy_visible = policy
                    .and_then(|policy| {
                        policy
                            .entries
                            .iter()
                            .find(|entry| entry.contribution_id == contribution.contribution_id)
                    })
                    .map(|entry| entry.visible)
                    .unwrap_or(true);
                policy_visible
                    && contribution.module_availability == ModuleNavigationAvailability::Available
                    && contribution
                        .required_capabilities_any_of
                        .iter()
                        .any(|required| actor_has_effective_capability(capabilities, required))
            })
            .map(|(_, contribution)| ResolvedNavigationItem {
                key: contribution.key.clone(),
                href: contribution.href.clone(),
                label: contribution.label.clone(),
                section: contribution.section,
                owner: NavigationItemOwner::Contribution,
                contribution_id: Some(contribution.contribution_id.clone()),
            }),
    );
}

fn core_item_for_key(key: &str) -> Option<&'static NavItem> {
    CORE_NAV_ITEMS.iter().find(|item| item.key == key)
}

fn core_item_is_visible(item: &NavItem, capabilities: &[String]) -> bool {
    item.capabilities.is_empty()
        || item
            .capabilities
            .iter()
            .any(|required| actor_has_effective_capability(capabilities, required))
}

fn actor_has_effective_capability(capabilities: &[String], required: &str) -> bool {
    capabilities.iter().any(|capability| {
        capability == "admin:all"
            || capability == required
            || (required == "modules:read" && capability == "modules:manage_navigation")
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ContributedNavigationItem, ModuleNavigationAvailability, NAV_ITEMS, NavigationBand,
        NavigationCompositionError, NavigationItemOwner, NavigationPolicy, NavigationPolicyEntry,
        NavigationSection, ResolvedNavigation, nav_item_for_route, nav_item_is_allowed,
        nav_item_is_visible, nav_items_for_section, resolve_navigation,
    };

    struct NavigationActorCase {
        name: &'static str,
        capabilities: &'static [&'static str],
        main: &'static [&'static str],
        admin: &'static [&'static str],
    }

    fn owned_capabilities(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|key| (*key).to_string()).collect()
    }

    fn visible_keys(section: &'static str, capabilities: &[String]) -> Vec<&'static str> {
        nav_items_for_section(section, capabilities)
            .into_iter()
            .map(|item| item.key)
            .collect()
    }

    fn contributed_item(
        contribution_id: &str,
        key: &str,
        label: &str,
        section: NavigationSection,
        band: NavigationBand,
        default_order: u32,
        required_capabilities_any_of: &[&str],
    ) -> ContributedNavigationItem {
        ContributedNavigationItem {
            contribution_id: contribution_id.to_string(),
            key: key.to_string(),
            href: format!("/{key}"),
            label: label.to_string(),
            section,
            band,
            default_order,
            required_capabilities_any_of: owned_capabilities(required_capabilities_any_of),
            module_availability: ModuleNavigationAvailability::Available,
        }
    }

    fn default_contributions() -> Vec<ContributedNavigationItem> {
        vec![
            contributed_item(
                "tessara.forms.navigation",
                "forms",
                "Forms",
                NavigationSection::Main,
                NavigationBand::MainBetweenOrganizationAndOperations,
                0,
                &["forms:read", "forms:manage"],
            ),
            contributed_item(
                "tessara.workflows.navigation",
                "workflows",
                "Workflows",
                NavigationSection::Main,
                NavigationBand::MainBetweenOrganizationAndOperations,
                1,
                &["workflows:read", "workflows:manage"],
            ),
            contributed_item(
                "tessara.responses.navigation",
                "responses",
                "Responses",
                NavigationSection::Main,
                NavigationBand::MainBetweenOrganizationAndOperations,
                2,
                &[
                    "submissions:read_own",
                    "submissions:respond",
                    "submissions:manage",
                ],
            ),
            contributed_item(
                "tessara.components.navigation",
                "components",
                "Components",
                NavigationSection::Main,
                NavigationBand::MainAfterOperations,
                0,
                &["components:read", "components:manage"],
            ),
            contributed_item(
                "tessara.dashboards.navigation",
                "dashboards",
                "Dashboards",
                NavigationSection::Main,
                NavigationBand::MainAfterOperations,
                1,
                &["dashboards:read"],
            ),
            contributed_item(
                "tessara.datasets.navigation",
                "datasets",
                "Datasets",
                NavigationSection::Admin,
                NavigationBand::AdminBetweenAdministrationAndModuleManagement,
                0,
                &["datasets:read", "datasets:manage"],
            ),
        ]
    }

    fn default_policy(contributions: &[ContributedNavigationItem]) -> NavigationPolicy {
        NavigationPolicy {
            revision: 1,
            entries: contributions
                .iter()
                .map(|contribution| NavigationPolicyEntry {
                    contribution_id: contribution.contribution_id.clone(),
                    visible: true,
                    order: contribution.default_order,
                })
                .collect(),
        }
    }

    fn resolved_keys(navigation: &ResolvedNavigation, section: NavigationSection) -> Vec<&str> {
        navigation
            .items
            .iter()
            .filter(|item| item.section == section)
            .map(|item| item.key.as_str())
            .collect()
    }

    fn resolve_defaults(
        contributions: &[ContributedNavigationItem],
        capabilities: &[String],
    ) -> ResolvedNavigation {
        let policy = default_policy(contributions);
        resolve_navigation(contributions, Some(&policy), capabilities)
    }

    #[test]
    fn static_navigation_contract_freezes_labels_routes_groups_order_and_capabilities() {
        let actual = NAV_ITEMS
            .iter()
            .map(|item| {
                (
                    item.key,
                    item.href,
                    item.label,
                    item.section,
                    item.capabilities,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                ("home", "/", "Home", "Main", &[][..]),
                (
                    "organization",
                    "/organization",
                    "Organization",
                    "Main",
                    &["hierarchy:read", "hierarchy:manage"][..],
                ),
                (
                    "forms",
                    "/forms",
                    "Forms",
                    "Main",
                    &["forms:read", "forms:manage"][..],
                ),
                (
                    "workflows",
                    "/workflows",
                    "Workflows",
                    "Main",
                    &["workflows:read", "workflows:manage"][..],
                ),
                (
                    "responses",
                    "/responses",
                    "Responses",
                    "Main",
                    &[
                        "submissions:read_own",
                        "submissions:respond",
                        "submissions:manage",
                    ][..],
                ),
                (
                    "operations",
                    "/operations",
                    "Operations",
                    "Main",
                    &["operations:view"][..],
                ),
                (
                    "components",
                    "/components",
                    "Components",
                    "Main",
                    &["components:read", "components:manage"][..],
                ),
                (
                    "dashboards",
                    "/dashboards",
                    "Dashboards",
                    "Main",
                    &["dashboards:read", "dashboards:manage"][..],
                ),
                (
                    "administration",
                    "/administration",
                    "Administration",
                    "Admin",
                    &["admin:all"][..],
                ),
                (
                    "datasets",
                    "/datasets",
                    "Datasets",
                    "Admin",
                    &["datasets:read", "datasets:manage"][..],
                ),
            ]
        );
    }

    #[test]
    fn named_actor_navigation_sequences_are_frozen_before_dynamic_navigation() {
        let cases = [
            NavigationActorCase {
                name: "admin_all",
                capabilities: &["admin:all"],
                main: &[
                    "home",
                    "organization",
                    "forms",
                    "workflows",
                    "responses",
                    "operations",
                    "components",
                    "dashboards",
                ],
                admin: &["administration", "datasets"],
            },
            NavigationActorCase {
                name: "operator",
                capabilities: &[
                    "hierarchy:read",
                    "forms:read",
                    "workflows:read",
                    "workflows:manage",
                    "submissions:respond",
                    "submissions:manage",
                    "operations:view",
                    "datasets:read",
                    "components:read",
                    "dashboards:read",
                ],
                main: &[
                    "home",
                    "organization",
                    "forms",
                    "workflows",
                    "responses",
                    "operations",
                    "components",
                    "dashboards",
                ],
                admin: &["datasets"],
            },
            NavigationActorCase {
                name: "respondent",
                capabilities: &["submissions:read_own", "submissions:respond"],
                main: &["home", "responses"],
                admin: &[],
            },
            NavigationActorCase {
                name: "forms_manage_only",
                capabilities: &["forms:manage"],
                main: &["home", "forms"],
                admin: &[],
            },
            NavigationActorCase {
                name: "workflows_manage_only",
                capabilities: &["workflows:manage"],
                main: &["home", "workflows"],
                admin: &[],
            },
            NavigationActorCase {
                name: "submissions_manage_only",
                capabilities: &["submissions:manage"],
                main: &["home", "responses"],
                admin: &[],
            },
            NavigationActorCase {
                name: "components_manage_only",
                capabilities: &["components:manage"],
                main: &["home", "components"],
                admin: &[],
            },
            NavigationActorCase {
                name: "dashboards_manage_only",
                capabilities: &["dashboards:manage"],
                main: &["home"],
                admin: &[],
            },
            NavigationActorCase {
                name: "datasets_manage_only",
                capabilities: &["datasets:manage"],
                main: &["home"],
                admin: &["datasets"],
            },
            NavigationActorCase {
                name: "no_access",
                capabilities: &[],
                main: &["home"],
                admin: &[],
            },
        ];

        for case in cases {
            let capabilities = owned_capabilities(case.capabilities);
            assert_eq!(
                visible_keys("Main", &capabilities),
                case.main,
                "{} Main navigation changed",
                case.name
            );
            assert_eq!(
                visible_keys("Admin", &capabilities),
                case.admin,
                "{} Admin navigation changed",
                case.name
            );
        }
    }

    #[test]
    fn manage_only_dashboard_access_keeps_route_permission_without_reader_link() {
        let dashboards = nav_item_for_route("dashboards").expect("Dashboard nav item");
        let capabilities = vec!["dashboards:manage".to_string()];

        assert!(nav_item_is_allowed(dashboards, &capabilities));
        assert!(!nav_item_is_visible(dashboards, &capabilities));
    }

    #[test]
    fn dashboard_readers_and_admins_receive_the_directory_link() {
        let dashboards = nav_item_for_route("dashboards").expect("Dashboard nav item");

        assert!(nav_item_is_visible(
            dashboards,
            &["dashboards:read".to_string()]
        ));
        assert!(nav_item_is_visible(dashboards, &["admin:all".to_string()]));
    }

    #[test]
    fn product_manage_capabilities_allow_routes_and_keep_current_link_rules() {
        for (route_key, capability, expected_visible) in [
            ("organization", "hierarchy:manage", true),
            ("forms", "forms:manage", true),
            ("workflows", "workflows:manage", true),
            ("responses", "submissions:manage", true),
            ("components", "components:manage", true),
            ("dashboards", "dashboards:manage", false),
            ("datasets", "datasets:manage", true),
        ] {
            let item = nav_item_for_route(route_key).expect("characterized route item");
            let capabilities = vec![capability.to_string()];

            assert!(
                nav_item_is_allowed(item, &capabilities),
                "{capability} should continue allowing the {route_key} route family"
            );
            assert_eq!(
                nav_item_is_visible(item, &capabilities),
                expected_visible,
                "{capability} visibility changed for {route_key}"
            );
        }
    }

    #[test]
    fn a_visible_product_link_does_not_authorize_an_unrelated_actor() {
        let forms = nav_item_for_route("forms").expect("Forms nav item");
        let unrelated_capabilities = vec!["datasets:read".to_string()];

        assert!(!nav_item_is_allowed(forms, &unrelated_capabilities));
        assert!(!nav_item_is_visible(forms, &unrelated_capabilities));
    }

    #[test]
    fn default_dynamic_composition_preserves_every_old_item_and_appends_module_management() {
        let contributions = default_contributions();
        let navigation = resolve_defaults(&contributions, &["admin:all".to_string()]);

        assert_eq!(navigation.unavailable, None);
        assert_eq!(
            resolved_keys(&navigation, NavigationSection::Main),
            [
                "home",
                "organization",
                "forms",
                "workflows",
                "responses",
                "operations",
                "components",
                "dashboards",
            ]
        );
        assert_eq!(
            resolved_keys(&navigation, NavigationSection::Admin),
            ["administration", "datasets", "module_management"]
        );

        let actual = navigation
            .items
            .iter()
            .map(|item| {
                (
                    item.key.as_str(),
                    item.href.as_str(),
                    item.label.as_str(),
                    item.section,
                    item.owner,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            vec![
                (
                    "home",
                    "/",
                    "Home",
                    NavigationSection::Main,
                    NavigationItemOwner::Core
                ),
                (
                    "organization",
                    "/organization",
                    "Organization",
                    NavigationSection::Main,
                    NavigationItemOwner::Core,
                ),
                (
                    "forms",
                    "/forms",
                    "Forms",
                    NavigationSection::Main,
                    NavigationItemOwner::Contribution,
                ),
                (
                    "workflows",
                    "/workflows",
                    "Workflows",
                    NavigationSection::Main,
                    NavigationItemOwner::Contribution,
                ),
                (
                    "responses",
                    "/responses",
                    "Responses",
                    NavigationSection::Main,
                    NavigationItemOwner::Contribution,
                ),
                (
                    "operations",
                    "/operations",
                    "Operations",
                    NavigationSection::Main,
                    NavigationItemOwner::Core,
                ),
                (
                    "components",
                    "/components",
                    "Components",
                    NavigationSection::Main,
                    NavigationItemOwner::Contribution,
                ),
                (
                    "dashboards",
                    "/dashboards",
                    "Dashboards",
                    NavigationSection::Main,
                    NavigationItemOwner::Contribution,
                ),
                (
                    "administration",
                    "/administration",
                    "Administration",
                    NavigationSection::Admin,
                    NavigationItemOwner::Core,
                ),
                (
                    "datasets",
                    "/datasets",
                    "Datasets",
                    NavigationSection::Admin,
                    NavigationItemOwner::Contribution,
                ),
                (
                    "module_management",
                    "/administration/modules",
                    "Module Management",
                    NavigationSection::Admin,
                    NavigationItemOwner::Core,
                ),
            ]
        );
    }

    #[test]
    fn no_contributions_retains_capability_filtered_core_navigation() {
        let admin = resolve_navigation(&[], None, &["admin:all".to_string()]);
        assert_eq!(
            resolved_keys(&admin, NavigationSection::Main),
            ["home", "organization", "operations"]
        );
        assert_eq!(
            resolved_keys(&admin, NavigationSection::Admin),
            ["administration", "module_management"]
        );

        let no_access = resolve_navigation(&[], None, &[]);
        assert_eq!(resolved_keys(&no_access, NavigationSection::Main), ["home"]);
        assert!(resolved_keys(&no_access, NavigationSection::Admin).is_empty());
        assert_eq!(admin.unavailable, None);
        assert_eq!(no_access.unavailable, None);
    }

    #[test]
    fn policy_reorders_only_within_each_core_assigned_band() {
        let contributions = default_contributions();
        let mut policy = default_policy(&contributions);
        for entry in &mut policy.entries {
            entry.order = match entry.contribution_id.as_str() {
                "tessara.responses.navigation" | "tessara.dashboards.navigation" => 0,
                "tessara.workflows.navigation" | "tessara.components.navigation" => 1,
                "tessara.forms.navigation" => 2,
                "tessara.datasets.navigation" => 0,
                _ => unreachable!("known default contribution"),
            };
        }

        let navigation =
            resolve_navigation(&contributions, Some(&policy), &["admin:all".to_string()]);

        assert_eq!(
            resolved_keys(&navigation, NavigationSection::Main),
            [
                "home",
                "organization",
                "responses",
                "workflows",
                "forms",
                "operations",
                "dashboards",
                "components",
            ]
        );
        assert_eq!(
            resolved_keys(&navigation, NavigationSection::Admin),
            ["administration", "datasets", "module_management"]
        );
    }

    #[test]
    fn equal_policy_orders_use_contribution_id_as_a_deterministic_tie_breaker() {
        let contributions = default_contributions();
        let mut policy = default_policy(&contributions);
        for entry in &mut policy.entries {
            entry.order = 0;
        }

        let navigation =
            resolve_navigation(&contributions, Some(&policy), &["admin:all".to_string()]);

        assert_eq!(
            resolved_keys(&navigation, NavigationSection::Main),
            [
                "home",
                "organization",
                "forms",
                "responses",
                "workflows",
                "operations",
                "components",
                "dashboards",
            ]
        );
    }

    #[test]
    fn dynamic_actor_filtering_preserves_product_and_dashboard_display_rules() {
        let contributions = default_contributions();
        let cases = [
            NavigationActorCase {
                name: "operator",
                capabilities: &[
                    "hierarchy:read",
                    "forms:read",
                    "workflows:manage",
                    "submissions:respond",
                    "operations:view",
                    "components:read",
                    "dashboards:read",
                    "datasets:read",
                ],
                main: &[
                    "home",
                    "organization",
                    "forms",
                    "workflows",
                    "responses",
                    "operations",
                    "components",
                    "dashboards",
                ],
                admin: &["datasets"],
            },
            NavigationActorCase {
                name: "respondent",
                capabilities: &["submissions:read_own", "submissions:respond"],
                main: &["home", "responses"],
                admin: &[],
            },
            NavigationActorCase {
                name: "forms_manage_only",
                capabilities: &["forms:manage"],
                main: &["home", "forms"],
                admin: &[],
            },
            NavigationActorCase {
                name: "workflows_manage_only",
                capabilities: &["workflows:manage"],
                main: &["home", "workflows"],
                admin: &[],
            },
            NavigationActorCase {
                name: "submissions_manage_only",
                capabilities: &["submissions:manage"],
                main: &["home", "responses"],
                admin: &[],
            },
            NavigationActorCase {
                name: "components_manage_only",
                capabilities: &["components:manage"],
                main: &["home", "components"],
                admin: &[],
            },
            NavigationActorCase {
                name: "dashboards_manage_only",
                capabilities: &["dashboards:manage"],
                main: &["home"],
                admin: &[],
            },
            NavigationActorCase {
                name: "datasets_manage_only",
                capabilities: &["datasets:manage"],
                main: &["home"],
                admin: &["datasets"],
            },
            NavigationActorCase {
                name: "unrelated_product",
                capabilities: &["datasets:read"],
                main: &["home"],
                admin: &["datasets"],
            },
            NavigationActorCase {
                name: "no_access",
                capabilities: &[],
                main: &["home"],
                admin: &[],
            },
        ];

        for case in cases {
            let navigation =
                resolve_defaults(&contributions, &owned_capabilities(case.capabilities));
            assert_eq!(
                resolved_keys(&navigation, NavigationSection::Main),
                case.main,
                "{} Main navigation changed",
                case.name
            );
            assert_eq!(
                resolved_keys(&navigation, NavigationSection::Admin),
                case.admin,
                "{} Admin navigation changed",
                case.name
            );
        }
    }

    #[test]
    fn contribution_requires_module_availability_policy_visibility_and_actor_capability() {
        let mut contributions = default_contributions();
        contributions
            .iter_mut()
            .find(|contribution| contribution.key == "forms")
            .expect("Forms contribution")
            .module_availability = ModuleNavigationAvailability::Unavailable;
        let mut policy = default_policy(&contributions);
        policy
            .entries
            .iter_mut()
            .find(|entry| entry.contribution_id == "tessara.workflows.navigation")
            .expect("Workflows policy")
            .visible = false;

        let navigation = resolve_navigation(
            &contributions,
            Some(&policy),
            &owned_capabilities(&["forms:read", "workflows:read", "submissions:read_own"]),
        );

        assert_eq!(
            resolved_keys(&navigation, NavigationSection::Main),
            ["home", "responses"]
        );
        assert_eq!(navigation.unavailable, None);
    }

    #[test]
    fn manage_navigation_implies_module_read_without_implying_administration() {
        let contributions = default_contributions();

        for capability in ["modules:read", "modules:manage_navigation"] {
            let navigation = resolve_defaults(&contributions, &[capability.to_string()]);
            assert_eq!(
                resolved_keys(&navigation, NavigationSection::Admin),
                ["module_management"],
                "{capability} should expose the fixed Module Management item only"
            );
        }

        let product_only = resolve_defaults(&contributions, &["forms:read".to_string()]);
        assert!(
            !resolved_keys(&product_only, NavigationSection::Admin).contains(&"module_management")
        );

        let admin = resolve_defaults(&contributions, &["admin:all".to_string()]);
        assert_eq!(
            resolved_keys(&admin, NavigationSection::Admin),
            ["administration", "datasets", "module_management"]
        );
    }

    #[test]
    fn policy_cannot_target_or_displace_any_fixed_core_item() {
        let contributions = default_contributions();

        for core_key in [
            "home",
            "organization",
            "operations",
            "administration",
            "module_management",
        ] {
            let policy = NavigationPolicy {
                revision: 7,
                entries: vec![NavigationPolicyEntry {
                    contribution_id: core_key.to_string(),
                    visible: false,
                    order: u32::MAX,
                }],
            };
            let navigation =
                resolve_navigation(&contributions, Some(&policy), &["admin:all".to_string()]);

            assert_eq!(
                navigation.unavailable,
                Some(NavigationCompositionError::PolicyTargetsCore {
                    key: core_key.to_string(),
                })
            );
            assert_eq!(
                resolved_keys(&navigation, NavigationSection::Main),
                ["home", "organization", "operations"]
            );
            assert_eq!(
                resolved_keys(&navigation, NavigationSection::Admin),
                ["administration", "module_management"]
            );
            assert!(
                navigation
                    .items
                    .iter()
                    .all(|item| item.owner == NavigationItemOwner::Core)
            );
        }
    }

    #[test]
    fn missing_policy_for_a_non_empty_catalog_fails_closed() {
        let contributions = default_contributions();
        let navigation = resolve_navigation(&contributions, None, &["admin:all".to_string()]);

        assert_eq!(
            navigation.unavailable,
            Some(NavigationCompositionError::MissingPolicy)
        );
        assert_eq!(
            resolved_keys(&navigation, NavigationSection::Main),
            ["home", "organization", "operations"]
        );
        assert_eq!(
            resolved_keys(&navigation, NavigationSection::Admin),
            ["administration", "module_management"]
        );
    }

    #[test]
    fn supplied_policy_must_be_a_complete_unique_known_collection() {
        let contributions = default_contributions();
        let mut partial = default_policy(&contributions);
        let missing = partial.entries.pop().expect("policy member");
        let navigation =
            resolve_navigation(&contributions, Some(&partial), &["admin:all".to_string()]);
        assert_eq!(
            navigation.unavailable,
            Some(NavigationCompositionError::MissingPolicyContribution {
                contribution_id: missing.contribution_id,
            })
        );

        let mut duplicate = default_policy(&contributions);
        duplicate.entries.push(duplicate.entries[0].clone());
        let duplicate_id = duplicate.entries[0].contribution_id.clone();
        let navigation =
            resolve_navigation(&contributions, Some(&duplicate), &["admin:all".to_string()]);
        assert_eq!(
            navigation.unavailable,
            Some(NavigationCompositionError::DuplicatePolicyEntry {
                contribution_id: duplicate_id,
            })
        );

        let mut unknown = default_policy(&contributions);
        unknown.entries.push(NavigationPolicyEntry {
            contribution_id: "tessara.unknown.navigation".to_string(),
            visible: true,
            order: 0,
        });
        let navigation =
            resolve_navigation(&contributions, Some(&unknown), &["admin:all".to_string()]);
        assert_eq!(
            navigation.unavailable,
            Some(NavigationCompositionError::UnknownPolicyContribution {
                contribution_id: "tessara.unknown.navigation".to_string(),
            })
        );
    }

    #[test]
    fn malformed_or_core_colliding_contributions_fail_closed_as_a_collection() {
        let baseline = default_contributions();
        let mut malformed_cases = Vec::new();

        let mut malformed_id = baseline.clone();
        malformed_id[0].contribution_id = "Tessara.Forms".to_string();
        malformed_cases.push(malformed_id);

        let mut wrong_band = baseline.clone();
        wrong_band[0].section = NavigationSection::Admin;
        malformed_cases.push(wrong_band);

        let mut no_capability = baseline.clone();
        no_capability[0].required_capabilities_any_of.clear();
        malformed_cases.push(no_capability);

        let mut deployment_url = baseline.clone();
        deployment_url[0].href = "https://forms.example.invalid/forms".to_string();
        malformed_cases.push(deployment_url);

        let mut padded_label = baseline.clone();
        padded_label[0].label = " Forms ".to_string();
        malformed_cases.push(padded_label);

        let mut core_key_collision = baseline.clone();
        core_key_collision[0].key = "home".to_string();
        core_key_collision[0].href = "/forms".to_string();
        malformed_cases.push(core_key_collision);

        let mut core_href_collision = baseline.clone();
        core_href_collision[0].href = "/operations".to_string();
        malformed_cases.push(core_href_collision);

        let mut duplicate = baseline.clone();
        duplicate.push(duplicate[0].clone());
        malformed_cases.push(duplicate);

        for contributions in malformed_cases {
            let navigation = resolve_navigation(&contributions, None, &["admin:all".to_string()]);

            assert!(navigation.unavailable.is_some());
            assert_eq!(
                resolved_keys(&navigation, NavigationSection::Main),
                ["home", "organization", "operations"]
            );
            assert_eq!(
                resolved_keys(&navigation, NavigationSection::Admin),
                ["administration", "module_management"]
            );
            assert!(
                navigation
                    .items
                    .iter()
                    .all(|item| item.owner == NavigationItemOwner::Core)
            );
        }
    }
}
