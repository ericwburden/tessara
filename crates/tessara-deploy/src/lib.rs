//! Host-side deployment orchestration primitives for curated Tessara releases.
//! Docker Compose remains the external container-lifecycle authority.

use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use tessara_module_contract::{
    AppliedComponentV1, AppliedModuleV1, DeploymentPlanV1, DeploymentReceiptV1, TessaraDeploymentV1,
};
use uuid::Uuid;

pub fn read_deployment(path: &Path) -> Result<TessaraDeploymentV1> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let deployment = serde_json::from_slice(&bytes).context("parse deployment document")?;
    Ok(deployment)
}

pub fn read_plan(path: &Path) -> Result<DeploymentPlanV1> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).context("parse deployment plan")
}

pub fn read_receipt(path: &Path) -> Result<DeploymentReceiptV1> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).context("parse deployment receipt")
}

pub fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let parent = path.parent().context("output path has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("write {}", temporary.display()))?;
    fs::rename(&temporary, path).with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}

/// Publishes a sanitized receipt to the same-host Core import boundary.
pub fn publish_receipt(core_url: &str, token: &str, receipt: &DeploymentReceiptV1) -> Result<()> {
    let authority_and_path = core_url
        .strip_prefix("http://")
        .context("Core receipt import currently requires a local http:// URL")?;
    let (authority, base_path) = authority_and_path
        .split_once('/')
        .unwrap_or((authority_and_path, ""));
    let (host, port) = authority
        .split_once(':')
        .map(|(host, port)| Ok((host, port.parse::<u16>()?)))
        .unwrap_or_else(|| Ok::<_, anyhow::Error>((authority, 80)))?;
    let path = format!(
        "/{}/api/internal/deployment-receipts",
        base_path.trim_matches('/')
    )
    .replace("//", "/");
    let body = serde_json::to_vec(receipt)?;
    let mut stream = TcpStream::connect((host, port))
        .with_context(|| format!("connect to Core at {authority}"))?;
    write!(
        stream,
        "POST {path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/json\r\nX-Tessara-Deploy-Token: {token}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(&body)?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    let status = response.lines().next().unwrap_or_default();
    if !status.contains(" 204 ") {
        bail!("Core rejected deployment receipt: {status}");
    }
    Ok(())
}

pub fn apply_curated_plan(
    desired: &TessaraDeploymentV1,
    plan: &DeploymentPlanV1,
    current: Option<&DeploymentReceiptV1>,
    operator: String,
    applied_at: String,
) -> Result<DeploymentReceiptV1> {
    desired
        .validate()
        .map_err(|error| anyhow::anyhow!(serde_json::to_string_pretty(&error.findings).unwrap()))?;
    let expected = desired
        .plan()
        .map_err(|error| anyhow::anyhow!(serde_json::to_string_pretty(&error.findings).unwrap()))?;
    if &expected != plan {
        bail!("plan does not match the exact desired deployment document");
    }
    let applied_time = DateTime::parse_from_rfc3339(&applied_at)
        .context("applied_at must be RFC 3339")?
        .with_timezone(&Utc);
    let expiry = DateTime::parse_from_rfc3339(&desired.expires_at)
        .context("expires_at must be RFC 3339")?
        .with_timezone(&Utc);
    if Utc::now() > expiry || applied_time > expiry {
        bail!("deployment plan is expired");
    }
    if let Some(current) = current {
        if current.installation_id != desired.installation_id {
            bail!("current receipt belongs to a different installation");
        }
        if desired.revision <= current.revision {
            bail!("revision must advance beyond the current receipt");
        }
    }
    let plan_digest = plan.digest();
    let components = std::iter::once(AppliedComponentV1 {
        name: "Core".into(),
        artifact: desired.core_image.clone(),
        runtime: "compose:core".into(),
        healthy: true,
    })
    .chain(std::iter::once(AppliedComponentV1 {
        name: "Gateway".into(),
        artifact: desired.gateway_image.clone(),
        runtime: "compose:gateway".into(),
        healthy: true,
    }))
    .chain(std::iter::once(AppliedComponentV1 {
        name: "PostgreSQL".into(),
        artifact: desired.database_image.clone(),
        runtime: "compose:postgres".into(),
        healthy: true,
    }))
    .chain(desired.modules.iter().map(|module| AppliedComponentV1 {
        name: module.definition_id.to_string(),
        artifact: module.runtime_image.clone(),
        runtime: format!("compose:{}", service_name(module.definition_id.as_str())),
        healthy: true,
    }))
    .collect();
    let modules = desired
        .modules
        .iter()
        .map(|module| AppliedModuleV1 {
            definition_id: module.definition_id.clone(),
            manifest: Some(module.manifest.clone()),
            manifest_digest: module.manifest_digest.clone(),
            runtime_image: module.runtime_image.clone(),
            publisher: module.publisher.clone(),
            release_id: stable_uuid(
                format!(
                    "release:{}:{}",
                    module.definition_id, module.manifest_digest
                )
                .as_bytes(),
            ),
            instance_id: current
                .and_then(|receipt| {
                    receipt
                        .modules
                        .iter()
                        .find(|existing| existing.definition_id == module.definition_id)
                        .map(|existing| existing.instance_id)
                })
                .unwrap_or_else(|| {
                    stable_uuid(
                        format!(
                            "instance:{}:{}",
                            desired.installation_id, module.definition_id
                        )
                        .as_bytes(),
                    )
                }),
            version: module.version.clone(),
            database_name: module.database_name.clone(),
            route_prefix: Some(module.route_prefix.clone()),
            configuration: module.configuration.clone(),
        })
        .collect();
    Ok(DeploymentReceiptV1 {
        api_version: "tessara.io/deployment-receipt/v1".into(),
        installation_id: desired.installation_id,
        revision: desired.revision,
        plan_digest,
        applied_at,
        operator,
        idempotency_key: format!("apply-{}-{}", desired.installation_id, desired.revision),
        previous_revision: current.map(|receipt| receipt.revision),
        rollback_target_revision: None,
        components,
        modules,
    })
}

pub fn rollback(
    current: &DeploymentReceiptV1,
    target: &DeploymentReceiptV1,
    operator: String,
    applied_at: String,
) -> Result<DeploymentReceiptV1> {
    if current.installation_id != target.installation_id {
        bail!("rollback target belongs to a different installation");
    }
    if target.revision >= current.revision {
        bail!("rollback target must be older than the current revision");
    }
    if current.modules.iter().any(|module| {
        !target.modules.iter().any(|candidate| {
            candidate.definition_id == module.definition_id
                && candidate.instance_id == module.instance_id
        })
    }) {
        bail!("rollback target does not preserve all durable module instance identities");
    }
    let mut receipt = target.clone();
    receipt.revision = current.revision + 1;
    receipt.previous_revision = Some(current.revision);
    receipt.rollback_target_revision = Some(target.revision);
    receipt.applied_at = applied_at;
    receipt.operator = operator;
    receipt.idempotency_key = format!(
        "rollback-{}-{}-to-{}",
        current.installation_id, receipt.revision, target.revision
    );
    Ok(receipt)
}

fn stable_uuid(bytes: &[u8]) -> Uuid {
    let digest = Sha256::digest(bytes);
    let mut id = [0_u8; 16];
    id.copy_from_slice(&digest[..16]);
    id[6] = (id[6] & 0x0f) | 0x50;
    id[8] = (id[8] & 0x3f) | 0x80;
    Uuid::from_bytes(id)
}

fn service_name(definition: &str) -> String {
    definition
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;
    use std::collections::BTreeMap;
    use tessara_module_contract::{
        ArtifactDigest, DeploymentProfile, DesiredModuleV1, ModuleManifest,
    };

    fn digest(c: char) -> ArtifactDigest {
        ArtifactDigest::new(format!("sha256:{}", c.to_string().repeat(64))).unwrap()
    }
    fn desired(revision: u64, version: Version) -> TessaraDeploymentV1 {
        let manifest_digest = if version.major == 0 {
            digest('c')
        } else {
            digest('e')
        };
        let mut manifest: ModuleManifest = serde_json::from_str(include_str!(
            "../../tessara-module-contract/tests/fixtures/valid-manifest.json"
        ))
        .unwrap();
        manifest.release_version = version.clone();
        let DeploymentProfile::TessaraOciV1(deployment) = &mut manifest.deployment;
        deployment.runtime_image.digest = digest('d');
        deployment.runtime_image.image_reference = format!(
            "registry.example/tessara/forms@{}",
            deployment.runtime_image.digest
        );
        TessaraDeploymentV1 {
            api_version: "tessara.io/deployment/v1".into(),
            installation_id: Uuid::nil(),
            revision,
            expires_at: "2030-01-01T00:00:00Z".into(),
            core_version: Version::new(1, 0, 0),
            core_image: digest('a'),
            gateway_image: digest('b'),
            database_image: digest('f'),
            modules: vec![DesiredModuleV1 {
                definition_id: manifest.definition_id.clone(),
                version,
                manifest,
                manifest_digest,
                runtime_image: digest('d'),
                publisher: tessara_module_contract::PublisherId::new("tessara.first_party")
                    .unwrap(),
                database_name: Some("tessara_module_scoped_records".into()),
                route_prefix: "/reference/scoped-records".into(),
                configuration: BTreeMap::new(),
            }],
        }
    }
    #[test]
    fn upgrade_preserves_instance_identity() {
        let first_desired = desired(1, Version::new(0, 9, 0));
        let first = apply_curated_plan(
            &first_desired,
            &first_desired.plan().unwrap(),
            None,
            "test".into(),
            "2026-07-22T18:00:00Z".into(),
        )
        .unwrap();
        let second_desired = desired(2, Version::new(1, 0, 0));
        let second = apply_curated_plan(
            &second_desired,
            &second_desired.plan().unwrap(),
            Some(&first),
            "test".into(),
            "2026-07-22T18:10:00Z".into(),
        )
        .unwrap();
        assert_eq!(first.modules[0].instance_id, second.modules[0].instance_id);
        assert_ne!(first.modules[0].release_id, second.modules[0].release_id);
    }

    #[test]
    fn rollback_is_a_new_revision_with_an_explicit_target() {
        let first_desired = desired(1, Version::new(0, 9, 0));
        let first = apply_curated_plan(
            &first_desired,
            &first_desired.plan().unwrap(),
            None,
            "test".into(),
            "2026-07-22T18:00:00Z".into(),
        )
        .unwrap();
        let second_desired = desired(2, Version::new(1, 0, 0));
        let second = apply_curated_plan(
            &second_desired,
            &second_desired.plan().unwrap(),
            Some(&first),
            "test".into(),
            "2026-07-22T18:10:00Z".into(),
        )
        .unwrap();

        let rolled_back = rollback(
            &second,
            &first,
            "test".into(),
            "2026-07-22T18:20:00Z".into(),
        )
        .unwrap();

        assert_eq!(rolled_back.revision, 3);
        assert_eq!(rolled_back.previous_revision, Some(2));
        assert_eq!(rolled_back.rollback_target_revision, Some(1));
        assert_eq!(
            rolled_back.modules[0].instance_id,
            first.modules[0].instance_id
        );
        assert_eq!(
            rolled_back.modules[0].database_name,
            first.modules[0].database_name
        );
    }
}
