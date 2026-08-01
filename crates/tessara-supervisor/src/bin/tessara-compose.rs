//! Composition operator CLI backed by the same contracts as Core and Supervisor.

use std::{env, fs, path::Path};

use anyhow::{Context, bail};
use serde::Serialize;
use serde_json::Value;
use tessara_composition::{
    ApplicationBlueprintV1, ApplicationLockfileV1, ApplyAuthorizationV1, ReleaseCatalogV1,
    canonical_digest, resolve, semantic_diff,
};
use tessara_module_contract::{
    ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1, PurposeBoundVerifyingKeyV1,
    SignedEnvelopeV1,
};
use tessara_supervisor::{SupervisorLedger, signature_purpose_name};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command, output] if command == "key-public" => {
            let signer = signer(ProtocolSignaturePurposeV1::ReleaseCatalog)?;
            fs::write(output, encode_hex(&signer.verifier().public_key_bytes()))?;
        }
        [command, input, output] if command == "catalog-sign" => {
            let catalog: ReleaseCatalogV1 = read_json(input)?;
            let signer = signer(ProtocolSignaturePurposeV1::ReleaseCatalog)?;
            write_json(output, &signer.sign(catalog)?)?;
            println!("signed catalog written to {output}");
        }
        [command, input, public_key] if command == "catalog-verify" => {
            let envelope: SignedEnvelopeV1<ReleaseCatalogV1> = read_json(input)?;
            verifier(
                &envelope.issuer,
                &envelope.key_id,
                envelope.purpose,
                public_key,
            )?
            .verify(&envelope)?;
            println!("{}", canonical_digest(&envelope.payload)?);
        }
        [command, blueprint, catalog, public_key, output] if command == "resolve" => {
            let blueprint: ApplicationBlueprintV1 = read_json(blueprint)?;
            let catalog: SignedEnvelopeV1<ReleaseCatalogV1> = read_json(catalog)?;
            verifier(
                &catalog.issuer,
                &catalog.key_id,
                catalog.purpose,
                public_key,
            )?
            .verify(&catalog)?;
            let lockfile = resolve(&blueprint, &catalog.payload).map_err(|error| {
                anyhow::anyhow!(
                    serde_json::to_string_pretty(&error.findings)
                        .unwrap_or_else(|_| "composition resolution failed".into())
                )
            })?;
            println!("{}", canonical_digest(&lockfile)?);
            write_json(output, &lockfile)?;
        }
        [command, input, output] if command == "resolved-sign" => {
            let lockfile: ApplicationLockfileV1 = read_json(input)?;
            let signer = signer(ProtocolSignaturePurposeV1::ResolvedComposition)?;
            write_json(output, &signer.sign(lockfile)?)?;
            println!("signed resolved composition written to {output}");
        }
        [command, input, public_key, output] if command == "resolved-verify" => {
            let envelope: SignedEnvelopeV1<ApplicationLockfileV1> = read_json(input)?;
            anyhow::ensure!(
                envelope.purpose == ProtocolSignaturePurposeV1::ResolvedComposition,
                "resolved composition envelope has the wrong signature purpose"
            );
            verifier(
                &envelope.issuer,
                &envelope.key_id,
                ProtocolSignaturePurposeV1::ResolvedComposition,
                public_key,
            )?
            .verify(&envelope)?;
            println!("{}", canonical_digest(&envelope.payload)?);
            write_json(output, &envelope.payload)?;
        }
        [command, current, desired] if command == "diff" => {
            let current: ApplicationLockfileV1 = read_json(current)?;
            let desired: ApplicationLockfileV1 = read_json(desired)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&semantic_diff(Some(&current), &desired))?
            );
        }
        [command, input] if command == "digest" => {
            let value: Value = read_json(input)?;
            println!("{}", canonical_digest(&value)?);
        }
        [command, input, output] if command == "authorization-sign" => {
            let authorization: ApplyAuthorizationV1 = read_json(input)?;
            let signer = signer(ProtocolSignaturePurposeV1::ApplyAuthorization)?;
            write_json(output, &signer.sign(authorization)?)?;
        }
        [command, ledger, installation_id] if command == "init" => {
            SupervisorLedger::open(ledger)?
                .initialize_installation(Uuid::parse_str(installation_id)?, chrono::Utc::now())?;
        }
        [command, ledger, issuer, key_id, purpose, public_key] if command == "trust-register" => {
            let purpose = parse_purpose(purpose)?;
            SupervisorLedger::open(ledger)?.register_trust_anchor(
                issuer,
                key_id,
                signature_purpose_name(purpose),
                &read_key(public_key)?,
                chrono::Utc::now(),
            )?;
        }
        [command, url, lockfile, authorization] if command == "apply" => {
            let body = serde_json::json!({
                "lockfile": read_json::<ApplicationLockfileV1>(lockfile)?,
                "authorization": read_json::<SignedEnvelopeV1<ApplyAuthorizationV1>>(authorization)?,
            });
            let response = reqwest::Client::new()
                .post(format!("{}/v1/apply", url.trim_end_matches('/')))
                .json(&body)
                .send()
                .await?;
            print_response(response).await?;
        }
        [command, url, operation_id] if command == "status" => {
            let response = reqwest::get(format!(
                "{}/v1/operations/{operation_id}",
                url.trim_end_matches('/')
            ))
            .await?;
            print_response(response).await?;
        }
        [command, url] if command == "read-back" => {
            let response =
                reqwest::get(format!("{}/v1/receipts/current", url.trim_end_matches('/'))).await?;
            print_response(response).await?;
        }
        _ => bail!(
            "usage: tessara-compose <catalog-sign|catalog-verify|resolve|resolved-sign|resolved-verify|diff|authorization-sign|init|trust-register|apply|status|read-back> ..."
        ),
    }
    Ok(())
}

fn signer(purpose: ProtocolSignaturePurposeV1) -> anyhow::Result<PurposeBoundSigningKeyV1> {
    let issuer = env::var("TESSARA_SIGNING_ISSUER").context("TESSARA_SIGNING_ISSUER")?;
    let key_id = env::var("TESSARA_SIGNING_KEY_ID").context("TESSARA_SIGNING_KEY_ID")?;
    let secret = env::var("TESSARA_SIGNING_SECRET_HEX").context("TESSARA_SIGNING_SECRET_HEX")?;
    PurposeBoundSigningKeyV1::from_secret_bytes(issuer, key_id, purpose, decode_hex_32(&secret)?)
        .map_err(Into::into)
}

fn verifier(
    issuer: &str,
    key_id: &str,
    purpose: ProtocolSignaturePurposeV1,
    public_key: &str,
) -> anyhow::Result<PurposeBoundVerifyingKeyV1> {
    PurposeBoundVerifyingKeyV1::from_public_bytes(issuer, key_id, purpose, read_key(public_key)?)
        .map_err(Into::into)
}

fn read_key(path: impl AsRef<Path>) -> anyhow::Result<[u8; 32]> {
    decode_hex_32(fs::read_to_string(path)?.trim())
}

fn decode_hex_32(value: &str) -> anyhow::Result<[u8; 32]> {
    if value.len() != 64 {
        bail!("Ed25519 keys must contain exactly 64 hexadecimal characters")
    }
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .context("invalid hexadecimal key")?;
    }
    Ok(bytes)
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_purpose(value: &str) -> anyhow::Result<ProtocolSignaturePurposeV1> {
    match value {
        "release_catalog" => Ok(ProtocolSignaturePurposeV1::ReleaseCatalog),
        "resolved_composition" => Ok(ProtocolSignaturePurposeV1::ResolvedComposition),
        "apply_authorization" => Ok(ProtocolSignaturePurposeV1::ApplyAuthorization),
        "installation_receipt" => Ok(ProtocolSignaturePurposeV1::InstallationReceipt),
        _ => bail!("unsupported trust-anchor purpose '{value}'"),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> anyhow::Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

async fn print_response(response: reqwest::Response) -> anyhow::Result<()> {
    let status = response.status();
    let body: Value = response.json().await?;
    println!("{}", serde_json::to_string_pretty(&body)?);
    if !status.is_success() {
        bail!("Supervisor returned {status}")
    }
    Ok(())
}
