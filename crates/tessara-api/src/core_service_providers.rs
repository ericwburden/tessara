//! Core-owned declarations for still-in-process providers that participate in
//! the generic downstream service authorization boundary.

use semver::Version;
use tessara_module_contract::{AuthorizationGrantOperationV1, ServiceActionMethod};

pub(crate) const DATASET_MAJOR_LINE_CONTRACT: &str = "tessara.datasets.dataset-major-line";
pub(crate) const DATASET_MAJOR_LINE_CONTRACT_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CoreServiceAction {
    pub(crate) path: &'static str,
    pub(crate) method: ServiceActionMethod,
    pub(crate) authorization_action: &'static str,
    pub(crate) operation: AuthorizationGrantOperationV1,
    pub(crate) required_capability: &'static str,
    pub(crate) functional_contract: &'static str,
}

const DATASET_ACTIONS: [CoreServiceAction; 6] = [
    CoreServiceAction {
        path: tessara_datasets_contract::DATASET_BOOTSTRAP_VALIDATION_PATH,
        method: ServiceActionMethod::Post,
        authorization_action: tessara_datasets_contract::DATASET_BOOTSTRAP_VALIDATION_ACTION,
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:read",
        functional_contract: DATASET_MAJOR_LINE_CONTRACT,
    },
    CoreServiceAction {
        path: "/api/private/datasets/catalog",
        method: ServiceActionMethod::Post,
        authorization_action: "datasets.catalog",
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:read",
        functional_contract: DATASET_MAJOR_LINE_CONTRACT,
    },
    CoreServiceAction {
        path: "/api/private/datasets/schema",
        method: ServiceActionMethod::Post,
        authorization_action: "datasets.schema",
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:read",
        functional_contract: DATASET_MAJOR_LINE_CONTRACT,
    },
    CoreServiceAction {
        path: "/api/private/datasets/distinct-values",
        method: ServiceActionMethod::Post,
        authorization_action: "datasets.distinct_values",
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:read",
        functional_contract: DATASET_MAJOR_LINE_CONTRACT,
    },
    CoreServiceAction {
        path: "/api/private/datasets/compatibility",
        method: ServiceActionMethod::Post,
        authorization_action: "datasets.compatibility",
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:read",
        functional_contract: DATASET_MAJOR_LINE_CONTRACT,
    },
    CoreServiceAction {
        path: "/api/private/datasets/execute",
        method: ServiceActionMethod::Post,
        authorization_action: "datasets.execute",
        operation: AuthorizationGrantOperationV1::Read,
        required_capability: "datasets:read",
        functional_contract: DATASET_MAJOR_LINE_CONTRACT,
    },
];

pub(crate) fn resolve_service_action(
    functional_contract: &str,
    authorization_action: &str,
) -> Option<CoreServiceAction> {
    DATASET_ACTIONS.iter().copied().find(|declaration| {
        declaration.functional_contract == functional_contract
            && declaration.authorization_action == authorization_action
    })
}

pub(crate) fn contract_version(functional_contract: &str) -> Option<Version> {
    (functional_contract == DATASET_MAJOR_LINE_CONTRACT)
        .then(|| Version::parse(DATASET_MAJOR_LINE_CONTRACT_VERSION).expect("static version"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitional_dataset_provider_actions_are_exact_and_module_neutral() {
        let identities = DATASET_ACTIONS
            .iter()
            .map(|action| {
                (
                    action.path,
                    action.authorization_action,
                    action.operation,
                    action.required_capability,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            identities,
            vec![
                (
                    tessara_datasets_contract::DATASET_BOOTSTRAP_VALIDATION_PATH,
                    tessara_datasets_contract::DATASET_BOOTSTRAP_VALIDATION_ACTION,
                    AuthorizationGrantOperationV1::Read,
                    "datasets:read"
                ),
                (
                    "/api/private/datasets/catalog",
                    "datasets.catalog",
                    AuthorizationGrantOperationV1::Read,
                    "datasets:read"
                ),
                (
                    "/api/private/datasets/schema",
                    "datasets.schema",
                    AuthorizationGrantOperationV1::Read,
                    "datasets:read"
                ),
                (
                    "/api/private/datasets/distinct-values",
                    "datasets.distinct_values",
                    AuthorizationGrantOperationV1::Read,
                    "datasets:read"
                ),
                (
                    "/api/private/datasets/compatibility",
                    "datasets.compatibility",
                    AuthorizationGrantOperationV1::Read,
                    "datasets:read"
                ),
                (
                    "/api/private/datasets/execute",
                    "datasets.execute",
                    AuthorizationGrantOperationV1::Read,
                    "datasets:read"
                ),
            ]
        );
        assert!(
            identities
                .iter()
                .all(|(path, _, _, _)| !path.contains("component"))
        );
    }

    #[test]
    fn sprint_8a_catalog_binds_each_current_manifest_digest() {
        let catalog: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8a/catalogs/local-release-catalog.json"
        )))
        .unwrap();
        for (definition_id, manifest) in [
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
}
