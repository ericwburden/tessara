//! Core-owned declarations for still-in-process providers that participate in
//! the generic downstream service authorization boundary.

use semver::Version;
use tessara_module_contract::{AuthorizationGrantOperationV1, ServiceActionMethod};

pub(crate) const RESPONSE_EXPORT_CONTRACT: &str =
    tessara_responses_contract::RESPONSE_EXPORT_CONTRACT_ID;
pub(crate) const RESPONSE_EXPORT_CONTRACT_VERSION: &str =
    tessara_responses_contract::RESPONSE_EXPORT_CONTRACT_VERSION;
pub(crate) const FORM_VERSION_SCHEMA_CONTRACT: &str =
    tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_ID;
pub(crate) const FORM_VERSION_SCHEMA_CONTRACT_VERSION: &str =
    tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_VERSION;
pub(crate) const SCOPE_CATALOG_CONTRACT: &str =
    tessara_control_plane_contract::SCOPE_CATALOG_CONTRACT_ID;
pub(crate) const SCOPE_CATALOG_CONTRACT_VERSION: &str =
    tessara_control_plane_contract::SCOPE_CATALOG_CONTRACT_VERSION;
pub(crate) const PRINCIPAL_DISPLAY_CONTRACT: &str =
    tessara_control_plane_contract::PRINCIPAL_DISPLAY_CONTRACT_ID;
pub(crate) const PRINCIPAL_DISPLAY_CONTRACT_VERSION: &str =
    tessara_control_plane_contract::PRINCIPAL_DISPLAY_CONTRACT_VERSION;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CoreServiceAction {
    pub(crate) path: &'static str,
    pub(crate) method: ServiceActionMethod,
    pub(crate) authorization_action: &'static str,
    pub(crate) operation: AuthorizationGrantOperationV1,
    pub(crate) required_capability: &'static str,
    pub(crate) functional_contract: &'static str,
}

const RESPONSE_EXPORT_ACTIONS: [CoreServiceAction; 3] = [
    CoreServiceAction {
        path: tessara_responses_contract::RESPONSE_EXPORT_CHECKPOINT_PATH,
        method: ServiceActionMethod::Post,
        authorization_action: tessara_responses_contract::RESPONSE_EXPORT_CHECKPOINT_ACTION,
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:manage",
        functional_contract: RESPONSE_EXPORT_CONTRACT,
    },
    CoreServiceAction {
        path: tessara_responses_contract::RESPONSE_EXPORT_START_PATH,
        method: ServiceActionMethod::Post,
        authorization_action: tessara_responses_contract::RESPONSE_EXPORT_START_ACTION,
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:manage",
        functional_contract: RESPONSE_EXPORT_CONTRACT,
    },
    CoreServiceAction {
        path: tessara_responses_contract::RESPONSE_EXPORT_PAGE_PATH,
        method: ServiceActionMethod::Post,
        authorization_action: tessara_responses_contract::RESPONSE_EXPORT_PAGE_ACTION,
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:manage",
        functional_contract: RESPONSE_EXPORT_CONTRACT,
    },
];

const FORM_VERSION_SCHEMA_ACTIONS: [CoreServiceAction; 2] = [
    CoreServiceAction {
        path: tessara_forms_contract::FORM_VERSION_CATALOG_PATH,
        method: ServiceActionMethod::Post,
        authorization_action: tessara_forms_contract::FORM_VERSION_CATALOG_ACTION,
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:manage",
        functional_contract: FORM_VERSION_SCHEMA_CONTRACT,
    },
    CoreServiceAction {
        path: tessara_forms_contract::FORM_VERSION_SCHEMA_PATH,
        method: ServiceActionMethod::Post,
        authorization_action: tessara_forms_contract::FORM_VERSION_SCHEMA_ACTION,
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:manage",
        functional_contract: FORM_VERSION_SCHEMA_CONTRACT,
    },
];

const CONTROL_PLANE_ACTIONS: [CoreServiceAction; 2] = [
    CoreServiceAction {
        path: tessara_control_plane_contract::SCOPE_CATALOG_PATH,
        method: ServiceActionMethod::Post,
        authorization_action: tessara_control_plane_contract::SCOPE_CATALOG_ACTION,
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:manage",
        functional_contract: SCOPE_CATALOG_CONTRACT,
    },
    CoreServiceAction {
        path: tessara_control_plane_contract::PRINCIPAL_DISPLAY_PATH,
        method: ServiceActionMethod::Post,
        authorization_action: tessara_control_plane_contract::PRINCIPAL_DISPLAY_ACTION,
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:manage",
        functional_contract: PRINCIPAL_DISPLAY_CONTRACT,
    },
];

pub(crate) fn resolve_service_action(
    functional_contract: &str,
    authorization_action: &str,
) -> Option<CoreServiceAction> {
    RESPONSE_EXPORT_ACTIONS
        .iter()
        .chain(FORM_VERSION_SCHEMA_ACTIONS.iter())
        .chain(CONTROL_PLANE_ACTIONS.iter())
        .copied()
        .find(|declaration| {
            declaration.functional_contract == functional_contract
                && declaration.authorization_action == authorization_action
        })
}

pub(crate) fn contract_version(functional_contract: &str) -> Option<Version> {
    match functional_contract {
        RESPONSE_EXPORT_CONTRACT => {
            Some(Version::parse(RESPONSE_EXPORT_CONTRACT_VERSION).expect("static version"))
        }
        FORM_VERSION_SCHEMA_CONTRACT => {
            Some(Version::parse(FORM_VERSION_SCHEMA_CONTRACT_VERSION).expect("static version"))
        }
        SCOPE_CATALOG_CONTRACT => {
            Some(Version::parse(SCOPE_CATALOG_CONTRACT_VERSION).expect("static version"))
        }
        PRINCIPAL_DISPLAY_CONTRACT => {
            Some(Version::parse(PRINCIPAL_DISPLAY_CONTRACT_VERSION).expect("static version"))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_authoring_provider_actions_are_exact_and_versioned() {
        let expected = [
            (
                FORM_VERSION_SCHEMA_CONTRACT,
                tessara_forms_contract::FORM_VERSION_CATALOG_ACTION,
                tessara_forms_contract::FORM_VERSION_CATALOG_PATH,
            ),
            (
                FORM_VERSION_SCHEMA_CONTRACT,
                tessara_forms_contract::FORM_VERSION_SCHEMA_ACTION,
                tessara_forms_contract::FORM_VERSION_SCHEMA_PATH,
            ),
            (
                SCOPE_CATALOG_CONTRACT,
                tessara_control_plane_contract::SCOPE_CATALOG_ACTION,
                tessara_control_plane_contract::SCOPE_CATALOG_PATH,
            ),
            (
                PRINCIPAL_DISPLAY_CONTRACT,
                tessara_control_plane_contract::PRINCIPAL_DISPLAY_ACTION,
                tessara_control_plane_contract::PRINCIPAL_DISPLAY_PATH,
            ),
        ];
        for (contract, action, path) in expected {
            let declaration = resolve_service_action(contract, action).expect("provider action");
            assert_eq!(declaration.path, path);
            assert_eq!(declaration.method, ServiceActionMethod::Post);
            assert_eq!(declaration.operation, AuthorizationGrantOperationV1::Read);
            assert_eq!(declaration.required_capability, "datasets:manage");
            assert_eq!(contract_version(contract).unwrap(), Version::new(1, 0, 0));
        }
    }

    #[test]
    fn sprint_8b_catalog_binds_each_current_manifest_digest() {
        let catalog: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8b/catalogs/local-release-catalog.json"
        )))
        .unwrap();
        for (definition_id, manifest) in [
            (
                "tessara.datasets",
                include_str!("../../tessara-dataset-module/manifest.json"),
            ),
            (
                "tessara.components",
                include_str!("../../tessara-component-module/manifest.json"),
            ),
            (
                "tessara.dashboards",
                include_str!("../../tessara-dashboard-module/manifest.json"),
            ),
            (
                "tessara.reference.scoped-records",
                include_str!("../../tessara-reference-scoped-records/manifest.json"),
            ),
        ] {
            let manifest: tessara_module_contract::ModuleManifest =
                serde_json::from_str(manifest).unwrap();
            let actual = tessara_composition::canonical_digest(&manifest)
                .unwrap()
                .to_string();
            let expected = catalog["module_releases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|release| release["definition_id"] == definition_id)
                .unwrap()["manifest_digest"]
                .as_str()
                .unwrap();
            assert_eq!(actual, expected, "{definition_id}");
        }
    }

    #[test]
    fn sprint_8a_component_release_identity_stays_frozen() {
        let catalog: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8a/catalogs/local-release-catalog.json"
        )))
        .unwrap();
        let component = catalog["module_releases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|release| release["definition_id"] == "tessara.components")
            .unwrap();
        assert_eq!(component["version"], "1.0.1");
        assert_eq!(
            component["manifest_digest"],
            "sha256:59a78aa01356c5119cc23801b85ba47463940b6bd4237c528ecac7f4824f9d48"
        );
    }
}
