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
        capabilities: &["submissions:read_own", "submissions:respond", "submissions:manage"],
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
        || capabilities.iter().any(|capability| capability == "admin:all")
        || item
            .capabilities
            .iter()
            .any(|required| capabilities.iter().any(|capability| capability == required))
}

pub fn nav_items_for_section(section: &'static str, capabilities: &[String]) -> Vec<&'static NavItem> {
    NAV_ITEMS
        .iter()
        .filter(move |item| item.section == section)
        .filter(|item| nav_item_is_allowed(item, capabilities))
        .collect::<Vec<_>>()
}
