//! Request-scoped Module Management bootstrap.

use leptos::context::use_context;
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use std::cell::Cell;

use super::models::{
    ModuleDetailResponseV1, ModuleInventoryResponseV1, ModuleManagementAccessV1,
    NavigationPolicyResponseV2,
};

pub const MODULE_MANAGEMENT_BOOTSTRAP_SCRIPT_ID: &str = "tessara-module-management-bootstrap";

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static INITIAL_BOOTSTRAP_AVAILABLE: Cell<bool> = const { Cell::new(true) };
}

/// Authorization-filtered SSR state for exactly one Module Management route.
///
/// Restricted and unavailable variants intentionally carry no inventory. A
/// restricted detail route therefore cannot reveal whether its requested
/// definition exists.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "route", rename_all = "snake_case", deny_unknown_fields)]
pub enum ModuleManagementRouteBootstrapV1 {
    Directory {
        access: ModuleManagementAccessV1,
        inventory: ModuleInventoryResponseV1,
        navigation_policy: NavigationPolicyBootstrapV1,
    },
    Detail {
        access: ModuleManagementAccessV1,
        detail: Box<ModuleDetailResponseV1>,
        navigation_policy: NavigationPolicyBootstrapV1,
    },
    Restricted {
        surface: ModuleManagementSurfaceV1,
    },
    NotFound {
        definition_id: String,
    },
    Unavailable {
        surface: ModuleManagementSurfaceV1,
        message: String,
    },
}

impl ModuleManagementRouteBootstrapV1 {
    pub fn directory(
        access: ModuleManagementAccessV1,
        inventory: ModuleInventoryResponseV1,
        navigation_policy: NavigationPolicyBootstrapV1,
    ) -> Self {
        if !access.may_read() {
            return Self::Restricted {
                surface: ModuleManagementSurfaceV1::Directory,
            };
        }
        Self::Directory {
            access,
            inventory,
            navigation_policy,
        }
    }

    pub fn detail(
        access: ModuleManagementAccessV1,
        detail: ModuleDetailResponseV1,
        navigation_policy: NavigationPolicyBootstrapV1,
    ) -> Self {
        if !access.may_read() {
            return Self::Restricted {
                surface: ModuleManagementSurfaceV1::Detail,
            };
        }
        Self::Detail {
            access,
            detail: Box::new(detail),
            navigation_policy,
        }
    }

    pub const fn restricted(surface: ModuleManagementSurfaceV1) -> Self {
        Self::Restricted { surface }
    }

    pub fn not_found(definition_id: impl Into<String>) -> Self {
        Self::NotFound {
            definition_id: definition_id.into(),
        }
    }

    pub fn unavailable(surface: ModuleManagementSurfaceV1, message: impl Into<String>) -> Self {
        Self::Unavailable {
            surface,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleManagementSurfaceV1 {
    Directory,
    Detail,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum NavigationPolicyBootstrapV1 {
    Ready { policy: NavigationPolicyResponseV2 },
    Unavailable { message: String },
}

impl NavigationPolicyBootstrapV1 {
    pub fn ready(policy: NavigationPolicyResponseV2) -> Self {
        Self::Ready { policy }
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::Unavailable {
            message: message.into(),
        }
    }
}

/// Returns the initial bootstrap if it still belongs to this hydration pass.
pub fn module_management_route_bootstrap() -> Option<ModuleManagementRouteBootstrapV1> {
    let bootstrap = use_context::<ModuleManagementRouteBootstrapV1>();
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        INITIAL_BOOTSTRAP_AVAILABLE.with(|available| if available.get() { bootstrap } else { None })
    }
    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    bootstrap
}

/// Prevents client-side navigation from reusing a request-specific payload.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub fn clear_module_management_route_bootstrap() {
    INITIAL_BOOTSTRAP_AVAILABLE.with(|available| available.set(false));
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
pub fn clear_module_management_route_bootstrap() {}

/// Serializes a bootstrap for an inert `application/json` script element.
pub fn escaped_module_management_bootstrap_json(
    bootstrap: &ModuleManagementRouteBootstrapV1,
) -> String {
    serde_json::to_string(bootstrap)
        .expect("Module Management route bootstrap should serialize")
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

/// Parses the inert request bootstrap during hydration.
pub fn parse_module_management_bootstrap_json(
    json: &str,
) -> Option<ModuleManagementRouteBootstrapV1> {
    serde_json::from_str(json).ok()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ModuleManagementRouteBootstrapV1, ModuleManagementSurfaceV1, NavigationPolicyBootstrapV1,
        escaped_module_management_bootstrap_json, parse_module_management_bootstrap_json,
    };
    use crate::features::modules::models::{
        ApplicationInstallationV1, CoreRuntimeObservationV1, ModuleInventoryResponseV1,
        ModuleManagementAccessV1,
    };

    #[test]
    fn restricted_bootstrap_carries_no_identifier_or_inventory() {
        let wire = serde_json::to_value(ModuleManagementRouteBootstrapV1::restricted(
            ModuleManagementSurfaceV1::Detail,
        ))
        .expect("serialize restricted bootstrap");

        assert_eq!(wire, json!({ "route": "restricted", "surface": "detail" }));
    }

    #[test]
    fn authorized_constructor_fails_closed_before_inventory_can_be_embedded() {
        let bootstrap = ModuleManagementRouteBootstrapV1::directory(
            ModuleManagementAccessV1::restricted(),
            ModuleInventoryResponseV1 {
                schema_version: 1,
                installation: ApplicationInstallationV1 {
                    id: "must-not-leak".into(),
                    created_at: "must-not-leak".into(),
                },
                core_runtime: CoreRuntimeObservationV1 {
                    provenance: "must-not-leak".into(),
                    observed_version: "must-not-leak".into(),
                    finding_code: "must-not-leak".into(),
                    observed_at: "must-not-leak".into(),
                },
                entries: Vec::new(),
            },
            NavigationPolicyBootstrapV1::unavailable("must-not-leak"),
        );
        let wire = serde_json::to_value(bootstrap).expect("serialize");

        assert_eq!(
            wire,
            json!({ "route": "restricted", "surface": "directory" })
        );
        assert!(!wire.to_string().contains("must-not-leak"));
    }

    #[test]
    fn directory_bootstrap_round_trips_one_authorized_projection() {
        let bootstrap = ModuleManagementRouteBootstrapV1::directory(
            ModuleManagementAccessV1::read_only(),
            ModuleInventoryResponseV1 {
                schema_version: 1,
                installation: ApplicationInstallationV1 {
                    id: "installation-1".into(),
                    created_at: "2026-07-14T12:00:00Z".into(),
                },
                core_runtime: CoreRuntimeObservationV1 {
                    provenance: "unresolved".into(),
                    observed_version: "0.1.0".into(),
                    finding_code: "core_release_provenance_unresolved".into(),
                    observed_at: "2026-07-14T12:00:01Z".into(),
                },
                entries: Vec::new(),
            },
            NavigationPolicyBootstrapV1::unavailable("Policy temporarily unavailable."),
        );
        let wire = serde_json::to_string(&bootstrap).expect("serialize");

        assert_eq!(
            parse_module_management_bootstrap_json(&wire).expect("deserialize"),
            bootstrap
        );
    }

    #[test]
    fn inert_bootstrap_json_cannot_terminate_its_script_element() {
        let bootstrap = ModuleManagementRouteBootstrapV1::unavailable(
            ModuleManagementSurfaceV1::Directory,
            "</script><script>alert('module')</script>&",
        );
        let json = escaped_module_management_bootstrap_json(&bootstrap);

        assert!(!json.contains('<'));
        assert!(!json.contains('&'));
        assert!(json.contains("\\u003c/script\\u003e"));
        assert_eq!(
            parse_module_management_bootstrap_json(&json),
            Some(bootstrap)
        );
    }
}
