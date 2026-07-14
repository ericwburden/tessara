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

#[cfg(test)]
mod tests {
    use super::{nav_item_for_route, nav_item_is_allowed, nav_item_is_visible};

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
}
