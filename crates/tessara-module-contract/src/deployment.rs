//! Versioned single-host deployment intent, plan, and receipt contracts.

use std::collections::{BTreeMap, BTreeSet};

use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{ArtifactDigest, ModuleDefinitionId, PublisherId};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TessaraDeploymentV1 {
    pub api_version: String,
    pub installation_id: Uuid,
    pub revision: u64,
    pub expires_at: String,
    pub core_version: Version,
    pub core_image: ArtifactDigest,
    pub gateway_image: ArtifactDigest,
    pub database_image: ArtifactDigest,
    #[serde(default)]
    pub modules: Vec<DesiredModuleV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesiredModuleV1 {
    pub definition_id: ModuleDefinitionId,
    pub version: Version,
    pub manifest_digest: ArtifactDigest,
    pub runtime_image: ArtifactDigest,
    pub publisher: PublisherId,
    pub database_name: String,
    pub route_prefix: String,
    #[serde(default)]
    pub configuration: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentFindingV1 {
    pub code: String,
    pub severity: DeploymentFindingSeverityV1,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentFindingSeverityV1 {
    Error,
    Warning,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum DeploymentActionV1 {
    InstallCore {
        version: Version,
        image: ArtifactDigest,
    },
    InstallGateway {
        image: ArtifactDigest,
    },
    InstallDatabase {
        image: ArtifactDigest,
    },
    InstallModule {
        definition_id: ModuleDefinitionId,
        version: Version,
        runtime_image: ArtifactDigest,
        database_name: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentPlanV1 {
    pub api_version: String,
    pub installation_id: Uuid,
    pub revision: u64,
    pub deployment_digest: ArtifactDigest,
    pub expires_at: String,
    pub actions: Vec<DeploymentActionV1>,
    pub findings: Vec<DeploymentFindingV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedComponentV1 {
    pub name: String,
    pub artifact: ArtifactDigest,
    pub runtime: String,
    pub healthy: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedModuleV1 {
    pub definition_id: ModuleDefinitionId,
    pub manifest_digest: ArtifactDigest,
    pub runtime_image: ArtifactDigest,
    pub publisher: PublisherId,
    pub release_id: Uuid,
    pub instance_id: Uuid,
    pub version: Version,
    pub database_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentReceiptV1 {
    pub api_version: String,
    pub installation_id: Uuid,
    pub revision: u64,
    pub plan_digest: ArtifactDigest,
    pub applied_at: String,
    pub operator: String,
    pub idempotency_key: String,
    pub previous_revision: Option<u64>,
    #[serde(default)]
    pub rollback_target_revision: Option<u64>,
    pub components: Vec<AppliedComponentV1>,
    pub modules: Vec<AppliedModuleV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeploymentChangeV1 {
    pub operation: DeploymentOperationV1,
    pub modules: Vec<AppliedModuleChangeV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeploymentOperationV1 {
    Installed,
    Applied,
    RolledBack { target_revision: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedModuleChangeV1 {
    pub definition_id: ModuleDefinitionId,
    pub release: ReleaseChangeV1,
    pub instance: IdentityChangeV1,
    pub database: IdentityChangeV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReleaseChangeV1 {
    Installed { version: Version },
    Unchanged { version: Version },
    Changed { from: Version, to: Version },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityChangeV1 {
    Created { value: String },
    Preserved { value: String },
    Replaced { from: String, to: String },
}

impl DeploymentReceiptV1 {
    pub fn classify_change(&self, previous: Option<&Self>) -> DeploymentChangeV1 {
        let operation = if let Some(target_revision) = self.rollback_target_revision {
            DeploymentOperationV1::RolledBack { target_revision }
        } else if previous.is_some() || self.previous_revision.is_some() {
            DeploymentOperationV1::Applied
        } else {
            DeploymentOperationV1::Installed
        };
        let modules = self
            .modules
            .iter()
            .map(|module| {
                let previous_module = previous.and_then(|receipt| {
                    receipt
                        .modules
                        .iter()
                        .find(|candidate| candidate.definition_id == module.definition_id)
                });
                let release = match previous_module {
                    None => ReleaseChangeV1::Installed {
                        version: module.version.clone(),
                    },
                    Some(previous) if previous.version == module.version => {
                        ReleaseChangeV1::Unchanged {
                            version: module.version.clone(),
                        }
                    }
                    Some(previous) => ReleaseChangeV1::Changed {
                        from: previous.version.clone(),
                        to: module.version.clone(),
                    },
                };
                let instance = classify_identity(
                    previous_module.map(|previous| previous.instance_id.to_string()),
                    module.instance_id.to_string(),
                );
                let database = classify_identity(
                    previous_module.map(|previous| previous.database_name.clone()),
                    module.database_name.clone(),
                );
                AppliedModuleChangeV1 {
                    definition_id: module.definition_id.clone(),
                    release,
                    instance,
                    database,
                }
            })
            .collect();
        DeploymentChangeV1 { operation, modules }
    }
}

fn classify_identity(previous: Option<String>, current: String) -> IdentityChangeV1 {
    match previous {
        None => IdentityChangeV1::Created { value: current },
        Some(previous) if previous == current => IdentityChangeV1::Preserved { value: current },
        Some(from) => IdentityChangeV1::Replaced { from, to: current },
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("deployment document failed validation")]
pub struct DeploymentValidationError {
    pub findings: Vec<DeploymentFindingV1>,
}

impl TessaraDeploymentV1 {
    pub fn validate(&self) -> Result<(), DeploymentValidationError> {
        let mut findings = Vec::new();
        if self.api_version != "tessara.io/deployment/v1" {
            findings.push(error(
                "deployment_api_version_unsupported",
                "/api_version",
                "api_version must be tessara.io/deployment/v1",
            ));
        }
        if self.revision == 0 {
            findings.push(error(
                "deployment_revision_invalid",
                "/revision",
                "revision must be greater than zero",
            ));
        }
        if !looks_like_rfc3339(&self.expires_at) {
            findings.push(error(
                "deployment_expiry_invalid",
                "/expires_at",
                "expires_at must be an RFC 3339 timestamp",
            ));
        }

        let mut definitions = BTreeSet::new();
        let mut databases = BTreeSet::new();
        let mut routes = BTreeSet::new();
        for (index, module) in self.modules.iter().enumerate() {
            let base = format!("/modules/{index}");
            if !definitions.insert(module.definition_id.as_str()) {
                findings.push(error(
                    "deployment_module_duplicate",
                    format!("{base}/definition_id"),
                    "module definition may appear only once",
                ));
            }
            if !valid_database_name(&module.database_name) {
                findings.push(error(
                    "deployment_database_name_invalid",
                    format!("{base}/database_name"),
                    "database_name must be a lower-case PostgreSQL identifier",
                ));
            } else if !databases.insert(module.database_name.as_str()) {
                findings.push(error(
                    "deployment_database_duplicate",
                    format!("{base}/database_name"),
                    "each module must own a distinct database",
                ));
            }
            if !valid_route_prefix(&module.route_prefix) {
                findings.push(error(
                    "deployment_route_prefix_invalid",
                    format!("{base}/route_prefix"),
                    "route_prefix must be an absolute non-root path without traversal",
                ));
            } else if !routes.insert(module.route_prefix.as_str()) {
                findings.push(error(
                    "deployment_route_prefix_duplicate",
                    format!("{base}/route_prefix"),
                    "each module must own a distinct route prefix",
                ));
            }
        }
        findings.sort_by(|left, right| (&left.path, &left.code).cmp(&(&right.path, &right.code)));
        if findings.is_empty() {
            Ok(())
        } else {
            Err(DeploymentValidationError { findings })
        }
    }

    pub fn plan(&self) -> Result<DeploymentPlanV1, DeploymentValidationError> {
        self.validate()?;
        let mut modules = self.modules.clone();
        modules.sort_by(|left, right| left.definition_id.cmp(&right.definition_id));
        let mut actions = vec![
            DeploymentActionV1::InstallCore {
                version: self.core_version.clone(),
                image: self.core_image.clone(),
            },
            DeploymentActionV1::InstallGateway {
                image: self.gateway_image.clone(),
            },
            DeploymentActionV1::InstallDatabase {
                image: self.database_image.clone(),
            },
        ];
        actions.extend(
            modules
                .into_iter()
                .map(|module| DeploymentActionV1::InstallModule {
                    definition_id: module.definition_id,
                    version: module.version,
                    runtime_image: module.runtime_image,
                    database_name: module.database_name,
                }),
        );
        Ok(DeploymentPlanV1 {
            api_version: "tessara.io/deployment-plan/v1".into(),
            installation_id: self.installation_id,
            revision: self.revision,
            deployment_digest: canonical_sha256(self)
                .expect("deployment serialization cannot fail"),
            expires_at: self.expires_at.clone(),
            actions,
            findings: Vec::new(),
        })
    }
}

impl DeploymentPlanV1 {
    pub fn digest(&self) -> ArtifactDigest {
        canonical_sha256(self).expect("plan serialization cannot fail")
    }
}

pub fn canonical_sha256(value: &impl Serialize) -> Result<ArtifactDigest, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    let digest = Sha256::digest(bytes);
    Ok(ArtifactDigest::new(format!("sha256:{digest:x}")).expect("SHA-256 output is valid"))
}

fn error(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> DeploymentFindingV1 {
    DeploymentFindingV1 {
        code: code.into(),
        severity: DeploymentFindingSeverityV1::Error,
        path: path.into(),
        message: message.into(),
    }
}

fn looks_like_rfc3339(value: &str) -> bool {
    value.len() >= 20
        && value.contains('T')
        && (value.ends_with('Z') || value.rfind(['+', '-']).is_some_and(|index| index > 10))
}

fn valid_database_name(value: &str) -> bool {
    value.len() <= 63
        && value.starts_with(|c: char| c.is_ascii_lowercase())
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn valid_route_prefix(value: &str) -> bool {
    value.starts_with('/')
        && value.len() > 1
        && !value.ends_with('/')
        && !value.contains("//")
        && !value.contains("..")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(character: char) -> ArtifactDigest {
        ArtifactDigest::new(format!("sha256:{}", character.to_string().repeat(64))).unwrap()
    }

    fn deployment() -> TessaraDeploymentV1 {
        TessaraDeploymentV1 {
            api_version: "tessara.io/deployment/v1".into(),
            installation_id: Uuid::nil(),
            revision: 4,
            expires_at: "2026-07-22T19:00:00Z".into(),
            core_version: Version::new(1, 0, 0),
            core_image: digest('a'),
            gateway_image: digest('b'),
            database_image: digest('f'),
            modules: vec![DesiredModuleV1 {
                definition_id: ModuleDefinitionId::new("tessara.reference.scoped-records").unwrap(),
                version: Version::new(1, 0, 0),
                manifest_digest: digest('c'),
                runtime_image: digest('d'),
                publisher: PublisherId::new("tessara.first_party").unwrap(),
                database_name: "tessara_module_scoped_records".into(),
                route_prefix: "/reference/scoped-records".into(),
                configuration: BTreeMap::new(),
            }],
        }
    }

    #[test]
    fn plan_is_deterministic_and_digest_bound() {
        let desired = deployment();
        let first = desired.plan().unwrap();
        let second = desired.plan().unwrap();
        assert_eq!(first, second);
        assert_eq!(first.digest(), second.digest());
        assert_eq!(first.deployment_digest, canonical_sha256(&desired).unwrap());
    }

    #[test]
    fn duplicate_database_and_route_findings_have_stable_paths() {
        let mut desired = deployment();
        let mut duplicate = desired.modules[0].clone();
        duplicate.definition_id = ModuleDefinitionId::new("tessara.reference.second").unwrap();
        desired.modules.push(duplicate);
        let error = desired.validate().unwrap_err();
        assert_eq!(
            error
                .findings
                .iter()
                .map(|finding| finding.code.as_str())
                .collect::<Vec<_>>(),
            vec![
                "deployment_database_duplicate",
                "deployment_route_prefix_duplicate"
            ]
        );
    }

    fn receipt(revision: u64, instance_id: Uuid, database_name: &str) -> DeploymentReceiptV1 {
        DeploymentReceiptV1 {
            api_version: "tessara.io/deployment-receipt/v1".into(),
            installation_id: Uuid::nil(),
            revision,
            plan_digest: digest('e'),
            applied_at: "2026-07-22T19:00:00Z".into(),
            operator: "local:test".into(),
            idempotency_key: format!("apply-{revision}"),
            previous_revision: (revision > 1).then_some(revision - 1),
            rollback_target_revision: None,
            components: Vec::new(),
            modules: vec![AppliedModuleV1 {
                definition_id: ModuleDefinitionId::new("tessara.reference.scoped-records").unwrap(),
                manifest_digest: digest('c'),
                runtime_image: digest('d'),
                publisher: PublisherId::new("tessara.first_party").unwrap(),
                release_id: Uuid::from_u128(10),
                instance_id,
                version: Version::new(1, 0, 0),
                database_name: database_name.into(),
            }],
        }
    }

    #[test]
    fn receipt_change_classification_verifies_identity_preservation() {
        let previous = receipt(1, Uuid::from_u128(20), "tessara_module_original");
        let preserved = receipt(2, Uuid::from_u128(20), "tessara_module_original");
        let preserved_change = preserved.classify_change(Some(&previous));
        assert!(matches!(
            preserved_change.modules[0].instance,
            IdentityChangeV1::Preserved { .. }
        ));
        assert!(matches!(
            preserved_change.modules[0].database,
            IdentityChangeV1::Preserved { .. }
        ));

        let replaced = receipt(2, Uuid::from_u128(21), "tessara_module_rebound");
        let replaced_change = replaced.classify_change(Some(&previous));
        assert!(matches!(
            replaced_change.modules[0].instance,
            IdentityChangeV1::Replaced { .. }
        ));
        assert!(matches!(
            replaced_change.modules[0].database,
            IdentityChangeV1::Replaced { .. }
        ));
    }
}
