//! Canonical Sprint 6A transition sources and deterministic normalization.

use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tessara_module_contract::{
    DependencyEvaluationInput, DependencyRelationshipKind, InventoryEntry, TransitionAvailability,
    TransitionalContributionDescriptorV1, evaluate_functional_dependency,
};
use uuid::Uuid;

const FROZEN_CATALOG: [FrozenCatalogEntry; 7] = [
    FrozenCatalogEntry {
        name: "Forms",
        definition_id: "tessara.forms",
        navigation: Some(("main_between_organization_and_operations", 0)),
    },
    FrozenCatalogEntry {
        name: "Workflows",
        definition_id: "tessara.workflows",
        navigation: Some(("main_between_organization_and_operations", 1)),
    },
    FrozenCatalogEntry {
        name: "Responses",
        definition_id: "tessara.responses",
        navigation: Some(("main_between_organization_and_operations", 2)),
    },
    FrozenCatalogEntry {
        name: "Datasets",
        definition_id: "tessara.datasets",
        navigation: Some(("admin_between_administration_and_module_management", 0)),
    },
    FrozenCatalogEntry {
        name: "Components",
        definition_id: "tessara.components",
        navigation: Some(("main_after_operations", 0)),
    },
    FrozenCatalogEntry {
        name: "Dashboards",
        definition_id: "tessara.dashboards",
        navigation: Some(("main_after_operations", 1)),
    },
    FrozenCatalogEntry {
        name: "Migration",
        definition_id: "tessara.migration",
        navigation: None,
    },
];

#[derive(Clone, Copy)]
struct FrozenCatalogEntry {
    name: &'static str,
    definition_id: &'static str,
    navigation: Option<(&'static str, i32)>,
}

#[derive(Clone, Debug)]
pub(crate) struct CatalogInput {
    pub(crate) name: String,
    pub(crate) definition_id: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) expected_digest: String,
    pub(crate) navigation_defaults: Option<NavigationDefaults>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NavigationDefaults {
    pub(crate) reorder_band: String,
    pub(crate) policy_order: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct PreparedCatalogSource {
    pub(crate) definition_id: String,
    pub(crate) display_name: String,
    pub(crate) source_bytes: Vec<u8>,
    pub(crate) source_digest: String,
    pub(crate) descriptor: TransitionalContributionDescriptorV1,
    pub(crate) findings: Vec<CatalogFinding>,
    pub(crate) navigation_defaults: Option<NavigationDefaults>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CatalogFinding {
    pub(crate) code: String,
    pub(crate) path: String,
    pub(crate) message: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CatalogContractError {
    #[error("catalog source '{name}' violates the canonical byte contract: {reason}")]
    InvalidSourceBytes { name: String, reason: String },
    #[error("catalog source '{name}' has invalid expected digest '{digest}'")]
    InvalidExpectedDigest { name: String, digest: String },
    #[error("catalog source '{name}' digest mismatch: expected {expected}, found {actual}")]
    DigestMismatch {
        name: String,
        expected: String,
        actual: String,
    },
    #[error("catalog source '{name}' is not valid transition JSON: {message}")]
    Decode { name: String, message: String },
    #[error("catalog source '{name}' failed semantic validation: {message}")]
    Validation { name: String, message: String },
    #[error(
        "catalog source '{name}' reserved definition '{actual}' instead of frozen identity '{expected}'"
    )]
    DefinitionMismatch {
        name: String,
        expected: String,
        actual: String,
    },
    #[error("catalog source '{name}' does not match its frozen navigation shape")]
    NavigationShape { name: String },
    #[error("catalog inputs do not match the seven frozen Sprint 6A sources")]
    CatalogShape,
    #[error(
        "catalog source '{name}' display name '{actual}' does not match frozen name '{expected}'"
    )]
    DisplayNameMismatch {
        name: String,
        expected: String,
        actual: String,
    },
    #[error(
        "catalog source change for '{definition_id}' changes frozen identifiers or declarations"
    )]
    IncompatibleSourceChange { definition_id: String },
    #[error("catalog projection serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl CatalogContractError {
    pub(crate) const fn stable_code(&self) -> &'static str {
        match self {
            Self::InvalidSourceBytes { .. } => "transition_source_bytes_invalid",
            Self::InvalidExpectedDigest { .. } => "transition_source_digest_invalid",
            Self::DigestMismatch { .. } => "transition_source_digest_mismatch",
            Self::Decode { .. } => "transition_source_decode_failed",
            Self::Validation { .. } => "transition_source_validation_failed",
            Self::DefinitionMismatch { .. } => "transition_definition_identity_mismatch",
            Self::NavigationShape { .. } => "transition_navigation_shape_mismatch",
            Self::CatalogShape => "transition_catalog_shape_mismatch",
            Self::DisplayNameMismatch { .. } => "transition_display_name_mismatch",
            Self::IncompatibleSourceChange { .. } => "transition_source_change_incompatible",
            Self::Serialization(_) => "transition_projection_serialization_failed",
        }
    }
}

pub(crate) fn canonical_inputs() -> Vec<CatalogInput> {
    vec![
        canonical_input(
            "Forms",
            "tessara.forms",
            include_bytes!(
                "../../../tessara-module-contract/tests/fixtures/transition-forms-v1.json"
            ),
            include_str!(
                "../../../tessara-module-contract/tests/fixtures/transition-forms-v1.json.sha256"
            ),
            Some(("main_between_organization_and_operations", 0)),
        ),
        canonical_input(
            "Workflows",
            "tessara.workflows",
            include_bytes!(
                "../../../tessara-module-contract/tests/fixtures/transition-workflows-v1.json"
            ),
            include_str!(
                "../../../tessara-module-contract/tests/fixtures/transition-workflows-v1.json.sha256"
            ),
            Some(("main_between_organization_and_operations", 1)),
        ),
        canonical_input(
            "Responses",
            "tessara.responses",
            include_bytes!(
                "../../../tessara-module-contract/tests/fixtures/transition-responses-v1.json"
            ),
            include_str!(
                "../../../tessara-module-contract/tests/fixtures/transition-responses-v1.json.sha256"
            ),
            Some(("main_between_organization_and_operations", 2)),
        ),
        canonical_input(
            "Datasets",
            "tessara.datasets",
            include_bytes!(
                "../../../tessara-module-contract/tests/fixtures/transition-datasets-v1.json"
            ),
            include_str!(
                "../../../tessara-module-contract/tests/fixtures/transition-datasets-v1.json.sha256"
            ),
            Some(("admin_between_administration_and_module_management", 0)),
        ),
        canonical_input(
            "Components",
            "tessara.components",
            include_bytes!(
                "../../../tessara-module-contract/tests/fixtures/transition-components-v1.json"
            ),
            include_str!(
                "../../../tessara-module-contract/tests/fixtures/transition-components-v1.json.sha256"
            ),
            Some(("main_after_operations", 0)),
        ),
        canonical_input(
            "Dashboards",
            "tessara.dashboards",
            include_bytes!(
                "../../../tessara-module-contract/tests/fixtures/transition-dashboards-v1.json"
            ),
            include_str!(
                "../../../tessara-module-contract/tests/fixtures/transition-dashboards-v1.json.sha256"
            ),
            Some(("main_after_operations", 1)),
        ),
        canonical_input(
            "Migration",
            "tessara.migration",
            include_bytes!(
                "../../../tessara-module-contract/tests/fixtures/transition-migration-v1.json"
            ),
            include_str!(
                "../../../tessara-module-contract/tests/fixtures/transition-migration-v1.json.sha256"
            ),
            None,
        ),
    ]
}

pub(crate) fn frozen_definition_ids() -> Vec<String> {
    FROZEN_CATALOG
        .iter()
        .map(|entry| entry.definition_id.to_string())
        .collect()
}

pub(crate) fn frozen_source_input(
    definition_id: &str,
    bytes: Vec<u8>,
    expected_digest: String,
) -> Option<CatalogInput> {
    let entry = FROZEN_CATALOG
        .iter()
        .find(|entry| entry.definition_id == definition_id)?;
    Some(CatalogInput {
        name: entry.name.to_string(),
        definition_id: entry.definition_id.to_string(),
        bytes,
        expected_digest,
        navigation_defaults: entry.navigation.map(|(reorder_band, policy_order)| {
            NavigationDefaults {
                reorder_band: reorder_band.to_string(),
                policy_order,
            }
        }),
    })
}

fn canonical_input(
    name: &str,
    definition_id: &str,
    bytes: &[u8],
    expected_digest: &str,
    navigation_defaults: Option<(&str, i32)>,
) -> CatalogInput {
    CatalogInput {
        name: name.to_string(),
        definition_id: definition_id.to_string(),
        bytes: bytes.to_vec(),
        expected_digest: expected_digest.trim_end_matches('\n').to_string(),
        navigation_defaults: navigation_defaults.map(|(reorder_band, policy_order)| {
            NavigationDefaults {
                reorder_band: reorder_band.to_string(),
                policy_order,
            }
        }),
    }
}

pub(crate) fn prepare_catalog(
    inputs: &[CatalogInput],
) -> Result<Vec<PreparedCatalogSource>, CatalogContractError> {
    if inputs.len() != FROZEN_CATALOG.len()
        || inputs
            .iter()
            .zip(FROZEN_CATALOG)
            .any(|(input, frozen)| !matches_frozen_entry(input, frozen))
    {
        return Err(CatalogContractError::CatalogShape);
    }
    inputs.iter().map(prepare_source).collect()
}

fn matches_frozen_entry(input: &CatalogInput, frozen: FrozenCatalogEntry) -> bool {
    let expected_navigation =
        frozen
            .navigation
            .map(|(reorder_band, policy_order)| NavigationDefaults {
                reorder_band: reorder_band.to_string(),
                policy_order,
            });
    input.name == frozen.name
        && input.definition_id == frozen.definition_id
        && input.navigation_defaults == expected_navigation
}

pub(crate) fn prepare_source(
    input: &CatalogInput,
) -> Result<PreparedCatalogSource, CatalogContractError> {
    validate_source_bytes(input)?;
    validate_expected_digest(input)?;

    let actual_digest = source_digest(&input.bytes);
    if actual_digest != input.expected_digest {
        return Err(CatalogContractError::DigestMismatch {
            name: input.name.clone(),
            expected: input.expected_digest.clone(),
            actual: actual_digest,
        });
    }

    let descriptor: TransitionalContributionDescriptorV1 = serde_json::from_slice(&input.bytes)
        .map_err(|error| CatalogContractError::Decode {
            name: input.name.clone(),
            message: error.to_string(),
        })?;
    descriptor
        .validate()
        .map_err(|error| CatalogContractError::Validation {
            name: input.name.clone(),
            message: error.to_string(),
        })?;

    let actual_definition_id = descriptor.reserved_definition_id.as_str();
    if actual_definition_id != input.definition_id {
        return Err(CatalogContractError::DefinitionMismatch {
            name: input.name.clone(),
            expected: input.definition_id.clone(),
            actual: actual_definition_id.to_string(),
        });
    }
    if descriptor.display_name != input.name {
        return Err(CatalogContractError::DisplayNameMismatch {
            name: input.name.clone(),
            expected: input.name.clone(),
            actual: descriptor.display_name.clone(),
        });
    }

    match (&input.navigation_defaults, descriptor.navigation.as_slice()) {
        (Some(_), [_]) | (None, []) => {}
        _ => {
            return Err(CatalogContractError::NavigationShape {
                name: input.name.clone(),
            });
        }
    }

    let mut findings = descriptor
        .dependencies
        .iter()
        .enumerate()
        .map(|(index, dependency)| {
            let evaluation = evaluate_functional_dependency(DependencyEvaluationInput {
                evaluation_requested: true,
                relationship: DependencyRelationshipKind::TransitionInternal,
                consumer_definition_id: &descriptor.reserved_definition_id,
                dependency,
                candidates: &[],
                resolved_bindings: &[],
            });
            let finding_code = evaluation
                .finding_code()
                .expect("a transition-internal relationship always produces a finding");
            CatalogFinding {
                code: finding_code.stable_code().to_string(),
                path: format!("dependencies[{index}]"),
                message: format!(
                    "Dependency binding '{}' describes current in-process coupling and cannot be satisfied by a transition contribution provider.",
                    dependency.binding_key.as_str()
                ),
            }
        })
        .collect::<Vec<_>>();
    if descriptor.availability == TransitionAvailability::Retired {
        findings.push(CatalogFinding {
            code: "transition_destination_retired".to_string(),
            path: "availability".to_string(),
            message:
                "The former transition destination was deliberately withdrawn and has no live route."
                    .to_string(),
        });
    }

    Ok(PreparedCatalogSource {
        definition_id: input.definition_id.clone(),
        display_name: descriptor.display_name.clone(),
        source_bytes: input.bytes.clone(),
        source_digest: input.expected_digest.clone(),
        descriptor,
        findings,
        navigation_defaults: input.navigation_defaults.clone(),
    })
}

fn validate_source_bytes(input: &CatalogInput) -> Result<(), CatalogContractError> {
    let reason = if input.bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        Some("UTF-8 byte-order marks are forbidden")
    } else if input.bytes.contains(&b'\r') {
        Some("line endings must be LF")
    } else if !input.bytes.ends_with(b"\n") {
        Some("the source must end with one LF")
    } else if std::str::from_utf8(&input.bytes).is_err() {
        Some("the source must be UTF-8")
    } else {
        None
    };

    if let Some(reason) = reason {
        return Err(CatalogContractError::InvalidSourceBytes {
            name: input.name.clone(),
            reason: reason.to_string(),
        });
    }
    Ok(())
}

fn validate_expected_digest(input: &CatalogInput) -> Result<(), CatalogContractError> {
    let digest = input.expected_digest.as_bytes();
    let valid = digest.len() == 71
        && digest.starts_with(b"sha256:")
        && digest[7..]
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte));
    if !valid {
        return Err(CatalogContractError::InvalidExpectedDigest {
            name: input.name.clone(),
            digest: input.expected_digest.clone(),
        });
    }
    Ok(())
}

pub(crate) fn source_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub(crate) fn ensure_compatible_source_change(
    previous: &TransitionalContributionDescriptorV1,
    replacement: &TransitionalContributionDescriptorV1,
) -> Result<(), CatalogContractError> {
    if stable_shape(previous)? != stable_shape(replacement)? {
        return Err(CatalogContractError::IncompatibleSourceChange {
            definition_id: replacement.reserved_definition_id.as_str().to_string(),
        });
    }
    Ok(())
}

fn stable_shape(
    descriptor: &TransitionalContributionDescriptorV1,
) -> Result<Value, CatalogContractError> {
    Ok(json!({
        "definition_id": descriptor.reserved_definition_id.as_str(),
        "availability": availability_name(descriptor.availability),
        "features": descriptor.features.iter().map(|value| {
            json!({
                "id": value.id.as_str(),
                "contracts": value.contracts.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "resource_types": value.resource_types.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "destinations": value.destinations.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "capabilities": value.capabilities.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
                "configuration_pointers": &value.configuration_pointers,
            })
        }).collect::<Vec<_>>(),
        "contracts": descriptor.provided_contracts.iter().map(|value| {
            json!({
                "id": value.id.as_str(),
                "version": value.version.to_string(),
                "kind": serde_json::to_value(value.kind).expect("contract kind serializes"),
            })
        }).collect::<Vec<_>>(),
        "dependencies": descriptor.dependencies.iter().map(serde_json::to_value).collect::<Result<Vec<_>, _>>()?,
        "resource_types": descriptor.resource_types.iter().map(|value| value.id.as_str()).collect::<Vec<_>>(),
        "routes": serde_json::to_value(&descriptor.routes)?,
        "navigation": serde_json::to_value(&descriptor.navigation)?,
        "capabilities": descriptor.security_capabilities.iter().map(|value| value.id.as_str()).collect::<Vec<_>>(),
        "configuration_schema": &descriptor.configuration_schema,
    }))
}

fn availability_name(availability: TransitionAvailability) -> &'static str {
    match availability {
        TransitionAvailability::ActiveInProcess => "active_in_process",
        TransitionAvailability::Unavailable => "unavailable",
        TransitionAvailability::Retired => "retired",
    }
}

impl PreparedCatalogSource {
    pub(crate) fn normalized_projection(
        &self,
        installation_id: Uuid,
    ) -> Result<Value, CatalogContractError> {
        let inventory = InventoryEntry::TransitionalInProcess {
            descriptor: self.descriptor.clone(),
        };
        let mut projection = serde_json::to_value(inventory)?;
        let object = projection
            .as_object_mut()
            .expect("inventory entries serialize as objects");
        object.insert(
            "source_digest".to_string(),
            Value::String(self.source_digest.clone()),
        );
        object.insert(
            "resource_owner".to_string(),
            json!({
                "kind": "core_installation",
                "installation_id": installation_id,
            }),
        );
        object.insert("provider_eligible".to_string(), Value::Bool(false));
        object.insert("supervisor_materializable".to_string(), Value::Bool(false));
        object.insert(
            "findings".to_string(),
            serde_json::to_value(&self.findings)?,
        );
        Ok(projection)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::{
        CatalogContractError, canonical_inputs, ensure_compatible_source_change, prepare_catalog,
    };

    #[test]
    fn canonical_catalog_prepares_exact_sources_and_projection_findings() {
        let prepared = prepare_catalog(&canonical_inputs()).expect("canonical catalog prepares");
        assert_eq!(prepared.len(), 7);
        assert_eq!(
            prepared
                .iter()
                .map(|source| source.definition_id.as_str())
                .collect::<Vec<_>>(),
            [
                "tessara.forms",
                "tessara.workflows",
                "tessara.responses",
                "tessara.datasets",
                "tessara.components",
                "tessara.dashboards",
                "tessara.migration",
            ]
        );
        assert!(prepared.iter().all(|source| {
            source.source_digest.starts_with("sha256:") && source.source_bytes.ends_with(b"\n")
        }));

        let migration = prepared
            .iter()
            .find(|source| source.definition_id == "tessara.migration")
            .expect("Migration source");
        assert_eq!(migration.findings.len(), 1);
        assert_eq!(migration.findings[0].code, "transition_destination_retired");
        assert!(migration.navigation_defaults.is_none());

        let dependency_finding_count = prepared
            .iter()
            .flat_map(|source| &source.findings)
            .filter(|finding| finding.code == "transition_internal_only")
            .count();
        assert_eq!(dependency_finding_count, 7);

        let response_findings = &prepared
            .iter()
            .find(|source| source.definition_id == "tessara.responses")
            .expect("Responses source")
            .findings;
        assert_eq!(
            response_findings
                .iter()
                .map(|finding| (
                    finding.code.as_str(),
                    finding.path.as_str(),
                    finding.message.as_str()
                ))
                .collect::<Vec<_>>(),
            [
                (
                    "transition_internal_only",
                    "dependencies[0]",
                    "Dependency binding 'tessara.responses.workflow-version' describes current in-process coupling and cannot be satisfied by a transition contribution provider.",
                ),
                (
                    "transition_internal_only",
                    "dependencies[1]",
                    "Dependency binding 'tessara.responses.form-version' describes current in-process coupling and cannot be satisfied by a transition contribution provider.",
                ),
            ]
        );
        assert_eq!(
            (
                migration.findings[0].path.as_str(),
                migration.findings[0].message.as_str(),
            ),
            (
                "availability",
                "The former transition destination was deliberately withdrawn and has no live route.",
            )
        );

        let projection = migration
            .normalized_projection(Uuid::nil())
            .expect("Migration projection serializes");
        assert_eq!(projection["kind"], "transitional_in_process");
        assert_eq!(projection["descriptor"]["availability"], "retired");
        assert_eq!(projection["provider_eligible"], false);
        assert_eq!(projection["supervisor_materializable"], false);
        assert_eq!(
            projection["resource_owner"],
            json!({"kind": "core_installation", "installation_id": Uuid::nil()})
        );
        assert_eq!(
            projection["findings"][0]["code"],
            "transition_destination_retired"
        );
    }

    #[test]
    fn catalog_collection_and_core_navigation_defaults_are_frozen() {
        let mut inputs = canonical_inputs();
        inputs.pop();
        assert!(matches!(
            prepare_catalog(&inputs),
            Err(CatalogContractError::CatalogShape)
        ));

        let mut inputs = canonical_inputs();
        inputs[0]
            .navigation_defaults
            .as_mut()
            .expect("Forms navigation")
            .policy_order = 1;
        assert!(matches!(
            prepare_catalog(&inputs),
            Err(CatalogContractError::CatalogShape)
        ));

        let mut inputs = canonical_inputs();
        inputs.swap(0, 1);
        assert!(matches!(
            prepare_catalog(&inputs),
            Err(CatalogContractError::CatalogShape)
        ));
    }

    #[test]
    fn compatible_source_change_preserves_feature_realization_links() {
        let prepared = prepare_catalog(&canonical_inputs()).expect("canonical catalog prepares");
        let previous = &prepared[0].descriptor;

        let mut narrative_only = previous.clone();
        narrative_only
            .description
            .push_str(" Additional support context.");
        narrative_only.features[0]
            .description
            .push_str(" Additional support context.");
        ensure_compatible_source_change(previous, &narrative_only)
            .expect("narrative-only source changes preserve the stable shape");

        let mut changed_realization = previous.clone();
        changed_realization.features[0].destinations.pop();
        assert!(matches!(
            ensure_compatible_source_change(previous, &changed_realization),
            Err(CatalogContractError::IncompatibleSourceChange { .. })
        ));
    }
}
