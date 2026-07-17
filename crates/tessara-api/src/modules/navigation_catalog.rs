//! Authoritative Core catalog for every navigation destination.
//!
//! Installation policy owns only group instances, placement, visibility, and
//! order. Labels, routes/destinations, eligibility, ownership, and protection
//! flags remain Core-controlled here and are never accepted from mutation
//! payloads.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NavigationCatalogOwner {
    Core,
    Contribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NavigationCatalogDestination {
    pub(crate) id: &'static str,
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) route: &'static str,
    pub(crate) semantic_destination: Option<&'static str>,
    pub(crate) definition_id: Option<&'static str>,
    pub(crate) owner: NavigationCatalogOwner,
    pub(crate) required_capabilities_any_of: &'static [&'static str],
    pub(crate) default_group_id: &'static str,
    pub(crate) default_order: i32,
    pub(crate) can_hide: bool,
    pub(crate) can_move_between_groups: bool,
}

pub(crate) const DESTINATIONS: [NavigationCatalogDestination; 13] = [
    NavigationCatalogDestination {
        id: "core.home",
        key: "home",
        label: "Home",
        route: "/",
        semantic_destination: None,
        definition_id: None,
        owner: NavigationCatalogOwner::Core,
        required_capabilities_any_of: &[],
        default_group_id: "core.main",
        default_order: 0,
        can_hide: false,
        can_move_between_groups: false,
    },
    NavigationCatalogDestination {
        id: "core.organization",
        key: "organization",
        label: "Organization",
        route: "/organization",
        semantic_destination: None,
        definition_id: None,
        owner: NavigationCatalogOwner::Core,
        required_capabilities_any_of: &["hierarchy:read", "hierarchy:manage"],
        default_group_id: "core.main",
        default_order: 1,
        can_hide: true,
        can_move_between_groups: false,
    },
    contribution(ContributionSpec {
        id: "tessara.forms.navigation",
        key: "forms",
        label: "Forms",
        route: "/forms",
        semantic_destination: "forms.directory",
        definition_id: "tessara.forms",
        capabilities: &["forms:read", "forms:manage"],
        default_order: 2,
    }),
    contribution(ContributionSpec {
        id: "tessara.workflows.navigation",
        key: "workflows",
        label: "Workflows",
        route: "/workflows",
        semantic_destination: "workflows.directory",
        definition_id: "tessara.workflows",
        capabilities: &["workflows:read", "workflows:manage"],
        default_order: 3,
    }),
    contribution(ContributionSpec {
        id: "tessara.responses.navigation",
        key: "responses",
        label: "Responses",
        route: "/responses",
        semantic_destination: "responses.directory",
        definition_id: "tessara.responses",
        capabilities: &[
            "submissions:read_own",
            "submissions:respond",
            "submissions:manage",
        ],
        default_order: 4,
    }),
    NavigationCatalogDestination {
        id: "core.operations",
        key: "operations",
        label: "Operations",
        route: "/operations",
        semantic_destination: None,
        definition_id: None,
        owner: NavigationCatalogOwner::Core,
        required_capabilities_any_of: &["operations:view"],
        default_group_id: "core.main",
        default_order: 5,
        can_hide: true,
        can_move_between_groups: true,
    },
    contribution(ContributionSpec {
        id: "tessara.datasets.navigation",
        key: "datasets",
        label: "Datasets",
        route: "/datasets",
        semantic_destination: "datasets.directory",
        definition_id: "tessara.datasets",
        capabilities: &["datasets:read", "datasets:manage"],
        default_order: 6,
    }),
    contribution(ContributionSpec {
        id: "tessara.components.navigation",
        key: "components",
        label: "Components",
        route: "/components",
        semantic_destination: "components.directory",
        definition_id: "tessara.components",
        capabilities: &["components:read", "components:manage"],
        default_order: 7,
    }),
    contribution(ContributionSpec {
        id: "tessara.dashboards.navigation",
        key: "dashboards",
        label: "Dashboards",
        route: "/dashboards",
        semantic_destination: "dashboards.directory",
        definition_id: "tessara.dashboards",
        capabilities: &["dashboards:read"],
        default_order: 8,
    }),
    core_admin(
        "core.admin.users",
        "user_management",
        "User Management",
        "/administration/users",
        0,
    ),
    core_admin(
        "core.admin.roles",
        "roles_access",
        "Roles & Access",
        "/administration/roles",
        1,
    ),
    core_admin(
        "core.admin.node_types",
        "node_types",
        "Node Types",
        "/administration/node-types",
        2,
    ),
    NavigationCatalogDestination {
        id: "core.admin.modules",
        key: "module_management",
        label: "Module Management",
        route: "/administration/modules",
        semantic_destination: None,
        definition_id: None,
        owner: NavigationCatalogOwner::Core,
        required_capabilities_any_of: &["modules:read"],
        default_group_id: "core.admin",
        default_order: 3,
        can_hide: false,
        can_move_between_groups: false,
    },
];

struct ContributionSpec {
    id: &'static str,
    key: &'static str,
    label: &'static str,
    route: &'static str,
    semantic_destination: &'static str,
    definition_id: &'static str,
    capabilities: &'static [&'static str],
    default_order: i32,
}

const fn contribution(spec: ContributionSpec) -> NavigationCatalogDestination {
    NavigationCatalogDestination {
        id: spec.id,
        key: spec.key,
        label: spec.label,
        route: spec.route,
        semantic_destination: Some(spec.semantic_destination),
        definition_id: Some(spec.definition_id),
        owner: NavigationCatalogOwner::Contribution,
        required_capabilities_any_of: spec.capabilities,
        default_group_id: "core.main",
        default_order: spec.default_order,
        can_hide: true,
        can_move_between_groups: true,
    }
}

const fn core_admin(
    id: &'static str,
    key: &'static str,
    label: &'static str,
    route: &'static str,
    default_order: i32,
) -> NavigationCatalogDestination {
    NavigationCatalogDestination {
        id,
        key,
        label,
        route,
        semantic_destination: None,
        definition_id: None,
        owner: NavigationCatalogOwner::Core,
        required_capabilities_any_of: &["admin:all"],
        default_group_id: "core.admin",
        default_order,
        can_hide: false,
        can_move_between_groups: false,
    }
}

pub(crate) fn destination(id: &str) -> Option<NavigationCatalogDestination> {
    DESTINATIONS.iter().copied().find(|item| item.id == id)
}
