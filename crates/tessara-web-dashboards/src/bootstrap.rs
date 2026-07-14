use leptos::context::use_context;
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use std::cell::Cell;

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static INITIAL_BOOTSTRAP_AVAILABLE: Cell<bool> = const { Cell::new(true) };
}

use crate::types::{
    Dashboard, DashboardComposition, DashboardSummary, SessionAccount, VisibilityNodeOption,
};

/// Request-scoped, authorization-filtered initial state for one Dashboard route.
///
/// The payload deliberately uses web-owned projections. It contains no session
/// token, database records, Dataset metadata, or binding identifiers for a
/// redacted placement. A bootstrap is valid only for the route (and, where
/// applicable, Dashboard id) represented by its variant.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "route", rename_all = "snake_case")]
pub enum DashboardRouteBootstrap {
    Directory {
        account: SessionAccount,
        dashboards: Vec<DashboardSummary>,
    },
    Create {
        account: SessionAccount,
        visibility_nodes: Vec<VisibilityNodeOption>,
    },
    Detail {
        account: SessionAccount,
        dashboard: Dashboard,
    },
    Editor {
        account: SessionAccount,
        composition: DashboardComposition,
        visibility_nodes: Vec<VisibilityNodeOption>,
    },
    Viewer {
        account: SessionAccount,
        dashboard: Dashboard,
    },
}

impl DashboardRouteBootstrap {
    pub fn directory(account: SessionAccount, dashboards: Vec<DashboardSummary>) -> Self {
        Self::Directory {
            account,
            dashboards,
        }
    }

    pub fn create(account: SessionAccount, visibility_nodes: Vec<VisibilityNodeOption>) -> Self {
        Self::Create {
            account,
            visibility_nodes,
        }
    }

    pub fn detail(account: SessionAccount, dashboard: Dashboard) -> Self {
        Self::Detail { account, dashboard }
    }

    pub fn editor(
        account: SessionAccount,
        composition: DashboardComposition,
        visibility_nodes: Vec<VisibilityNodeOption>,
    ) -> Self {
        Self::Editor {
            account,
            composition,
            visibility_nodes,
        }
    }

    pub fn viewer(account: SessionAccount, dashboard: Dashboard) -> Self {
        Self::Viewer { account, dashboard }
    }
}

/// Returns the request bootstrap supplied by root `tessara-web`, when it
/// matches the currently rendered application request.
///
/// Dashboard pages should pattern-match the expected route variant and fall
/// back to their REST loader when this returns `None` or a different variant.
pub fn dashboard_route_bootstrap() -> Option<DashboardRouteBootstrap> {
    let bootstrap = use_context::<DashboardRouteBootstrap>();
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        return INITIAL_BOOTSTRAP_AVAILABLE.with(
            |available| {
                if available.get() { bootstrap } else { None }
            },
        );
    }
    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    bootstrap
}

/// Prevents client-side route changes from reusing the request-scoped payload
/// after every component participating in the initial hydration pass has had
/// an opportunity to read it.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub fn clear_dashboard_route_bootstrap() {
    INITIAL_BOOTSTRAP_AVAILABLE.with(|available| available.set(false));
}

#[cfg(all(feature = "hydrate", not(target_arch = "wasm32")))]
pub fn clear_dashboard_route_bootstrap() {}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::DashboardRouteBootstrap;
    use crate::types::{Dashboard, DashboardPlacement, SessionAccount};

    fn account() -> SessionAccount {
        SessionAccount {
            capabilities: vec!["dashboards:read".into()],
        }
    }

    fn dashboard() -> Dashboard {
        Dashboard {
            id: "dashboard-42".into(),
            name: "Delivery health".into(),
            description: None,
            visibility_nodes: Vec::new(),
            placement_count: 1,
            can_manage: false,
            placements: vec![DashboardPlacement {
                placement_id: "placement-1".into(),
                position: 0,
                grid_row: 2,
                grid_column: 7,
                grid_width: 6,
                grid_height: 3,
                availability: crate::types::DashboardPlacementAvailability::Unavailable,
                config_state: None,
                title: None,
                component: None,
                allowed_operations: None,
            }],
        }
    }

    #[test]
    fn serializes_route_payload_and_redacted_geometry() {
        let value = serde_json::to_value(DashboardRouteBootstrap::viewer(account(), dashboard()))
            .expect("serialize");

        assert_eq!(value["route"], json!("viewer"));
        assert_eq!(value["dashboard"]["id"], json!("dashboard-42"));
        assert_eq!(value["dashboard"]["placements"][0]["grid_column"], 7);
        assert_eq!(
            value["dashboard"]["placements"][0]["availability"],
            "unavailable"
        );
        assert!(
            value["dashboard"]["placements"][0]
                .as_object()
                .expect("placement object")
                .get("component")
                .is_none()
        );
    }

    #[test]
    fn route_variants_do_not_alias_between_surfaces() {
        let dashboard = dashboard();
        assert_ne!(
            DashboardRouteBootstrap::detail(account(), dashboard.clone()),
            DashboardRouteBootstrap::viewer(account(), dashboard)
        );
    }
}
