use std::{env, path::Path, process::ExitCode};

use anyhow::{Context, Result, bail};
use serde_json::json;
use tessara_deploy::{
    apply_curated_plan, publish_receipt, read_deployment, read_plan, read_receipt, rollback,
    write_json,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args
        .next()
        .context("usage: tessara-deploy <validate|plan|apply|status|rollback> ...")?;
    let rest: Vec<String> = args.collect();
    match command.as_str() {
        "validate" => {
            let desired = read_deployment(required_path(&rest, 0, "deployment.json")?)?;
            desired.validate().map_err(|error| {
                anyhow::anyhow!(serde_json::to_string_pretty(&error.findings).unwrap())
            })?;
            print_json(&json!({
                "status": "valid",
                "deployment_digest": tessara_module_contract::canonical_sha256(&desired)?,
            }))?;
        }
        "plan" => {
            let desired = read_deployment(required_path(&rest, 0, "deployment.json")?)?;
            let plan = desired.plan().map_err(|error| {
                anyhow::anyhow!(serde_json::to_string_pretty(&error.findings).unwrap())
            })?;
            write_json(required_path(&rest, 1, "plan.json")?, &plan)?;
            print_json(&json!({
                "status": "planned",
                "plan_digest": plan.digest(),
                "output": required_path(&rest, 1, "plan.json")?,
            }))?;
        }
        "apply" => {
            let desired = read_deployment(required_path(&rest, 0, "deployment.json")?)?;
            let plan = read_plan(required_path(&rest, 1, "plan.json")?)?;
            let receipt_path = required_path(&rest, 2, "receipt.json")?;
            let current = receipt_path
                .exists()
                .then(|| read_receipt(receipt_path))
                .transpose()?;
            let receipt = apply_curated_plan(
                &desired,
                &plan,
                current.as_ref(),
                required(&rest, 3, "operator")?.into(),
                required(&rest, 4, "applied-at")?.into(),
            )?;
            write_json(receipt_path, &receipt)?;
            if let Some(core_url) = rest.get(5) {
                publish_receipt(
                    core_url,
                    required(&rest, 6, "deployment receipt import token")?,
                    &receipt,
                )?;
            }
            print_json(&json!({
                "status": "applied",
                "revision": receipt.revision,
                "receipt": receipt_path,
            }))?;
        }
        "status" => print_json(&read_receipt(required_path(&rest, 0, "receipt.json")?)?)?,
        "rollback" => {
            let current = read_receipt(required_path(&rest, 0, "current-receipt.json")?)?;
            let target = read_receipt(required_path(&rest, 1, "target-receipt.json")?)?;
            let output = required_path(&rest, 2, "receipt.json")?;
            let receipt = rollback(
                &current,
                &target,
                required(&rest, 3, "operator")?.into(),
                required(&rest, 4, "applied-at")?.into(),
            )?;
            write_json(output, &receipt)?;
            if let Some(core_url) = rest.get(5) {
                publish_receipt(
                    core_url,
                    required(&rest, 6, "deployment receipt import token")?,
                    &receipt,
                )?;
            }
            print_json(&json!({
                "status": "rolled_back",
                "revision": receipt.revision,
                "target_revision": receipt.rollback_target_revision,
                "receipt": output,
            }))?;
        }
        _ => bail!("unknown command '{command}'"),
    }
    Ok(())
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn required<'a>(args: &'a [String], index: usize, name: &str) -> Result<&'a str> {
    args.get(index)
        .map(String::as_str)
        .with_context(|| format!("missing {name}"))
}
fn required_path<'a>(args: &'a [String], index: usize, name: &str) -> Result<&'a Path> {
    required(args, index, name).map(Path::new)
}
