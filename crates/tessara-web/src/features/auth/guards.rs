//! Authentication guard helpers.
//!
//! Keep route/session gating helpers here so pages can delegate auth decisions to one module.

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use crate::features::auth::api;
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use leptos::task::spawn_local;

/// Evaluates direct-route authority independently from shell display policy.
///
/// Navigation visibility can be changed by an administrator, but it must never
/// grant or revoke access to a route. Unknown route keys retain the historical
/// authenticated-only behavior because their handlers remain authoritative.
#[cfg_attr(
    not(all(feature = "hydrate", target_arch = "wasm32")),
    allow(dead_code)
)]
pub(crate) fn route_is_allowed(active_route: &str, capabilities: &[String]) -> bool {
    let required_any_of: &[&str] = match active_route {
        "home" => return true,
        "organization" => &["hierarchy:read", "hierarchy:manage"],
        "forms" => &["forms:read", "forms:manage"],
        "workflows" => &["workflows:read", "workflows:manage"],
        "responses" => &[
            "submissions:read_own",
            "submissions:respond",
            "submissions:manage",
        ],
        "operations" => &["operations:view"],
        "components" => &["components:read", "components:manage"],
        "dashboards" => &["dashboards:read", "dashboards:manage"],
        "datasets" => &["datasets:read", "datasets:manage"],
        "administration" => &["admin:all"],
        "module_management" => &["modules:read", "modules:manage_navigation"],
        _ => return true,
    };

    capabilities.iter().any(|capability| {
        capability == "admin:all"
            || required_any_of
                .iter()
                .any(|required| capability == *required)
    })
}

#[cfg_attr(
    not(all(feature = "hydrate", target_arch = "wasm32")),
    allow(dead_code)
)]
fn renders_authenticated_denial_in_place(active_route: &str) -> bool {
    active_route == "module_management"
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub fn require_authenticated_route(active_route: &'static str) {
    if active_route == "home" {
        return;
    }

    spawn_local(async move {
        let session = api::fetch_session().await;
        let authenticated = session
            .as_ref()
            .map(|session| session.authenticated)
            .unwrap_or(false);

        if !authenticated {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/login");
            }
            return;
        }

        // Module Management deliberately renders one stable restricted state
        // for every authenticated actor without installation-global read.
        // The scope-aware API remains authoritative; flat session capability
        // strings cannot safely distinguish a global grant from a scoped one.
        if renders_authenticated_denial_in_place(active_route) {
            return;
        }

        let capabilities = session
            .and_then(|session| session.account)
            .map(|account| account.capabilities)
            .unwrap_or_default();
        if !route_is_allowed(active_route, &capabilities)
            && let Some(window) = web_sys::window()
        {
            let _ = window.location().set_href("/");
        }
    });
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
pub fn require_authenticated_route(active_route: &'static str) {
    let _ = active_route;
}

#[cfg(test)]
mod tests {
    use super::{renders_authenticated_denial_in_place, route_is_allowed};

    fn capabilities(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn direct_route_policy_preserves_product_manage_authority() {
        for (route, capability) in [
            ("organization", "hierarchy:manage"),
            ("forms", "forms:manage"),
            ("workflows", "workflows:manage"),
            ("responses", "submissions:manage"),
            ("components", "components:manage"),
            ("dashboards", "dashboards:manage"),
            ("datasets", "datasets:manage"),
        ] {
            assert!(
                route_is_allowed(route, &capabilities(&[capability])),
                "{capability} should authorize {route}"
            );
        }
    }

    #[test]
    fn module_manage_implies_module_read_route_access_only() {
        let manage = capabilities(&["modules:manage_navigation"]);
        assert!(route_is_allowed("module_management", &manage));
        assert!(!route_is_allowed("administration", &manage));

        let read = capabilities(&["modules:read"]);
        assert!(route_is_allowed("module_management", &read));
        assert!(!route_is_allowed("administration", &read));
        assert!(renders_authenticated_denial_in_place("module_management"));
        assert!(!renders_authenticated_denial_in_place("administration"));
    }

    #[test]
    fn display_policy_is_not_an_input_to_direct_route_authority() {
        assert!(route_is_allowed("forms", &capabilities(&["forms:read"])));
        assert!(!route_is_allowed(
            "forms",
            &capabilities(&["datasets:read"])
        ));
        assert!(route_is_allowed(
            "administration",
            &capabilities(&["admin:all"])
        ));
    }
}
