---
name: tessara-validation-preflight
description: Verify Tessara sprint validation prerequisites and freeze one source-exact candidate without running SIT or UAT. Use when preparing a sprint for formal testing, auditing databases and environment variables, reconciling test and acceptance inventories, validating deployment or evidence configuration, or producing the prerequisite receipts consumed by tessara-sit.
---

# Tessara Validation Preflight

Prepare and freeze a candidate for `tessara-sit`. Do not execute SIT, deployed
acceptance smoke, scripted UAT, or manual UAT in this skill.

Before acting, read
[`../tessara-sprint-validation/references/validation-protocol.md`](../tessara-sprint-validation/references/validation-protocol.md)
completely. It defines receipt schemas, fingerprints, classifications, and
invalidation authority.

## Inputs

- sprint label and slug
- intended sprint worktree, branch, and implementation commit
- roadmap, sprint plan, and validation record
- planned SIT/UAT commands and acceptance inventory
- deployment profile, expected service slot, and handoff URL
- evidence directory

## Required execution order

1. Confirm repository instructions, worktree, branch, and sprint scope.
2. Audit all changes and require one clean implementation commit.
3. Reconcile every roadmap exit condition with automated, smoke, and manual
   UAT coverage in the validation record.
4. Discover required environment variables from the actual test and runner
   sources. Do not infer similarly named variables.
5. Validate database URLs, unique disposable identities, reachability,
   credentials, reset authorization, and actual migration-ledger tables.
6. Validate Rust, Playwright, smoke, scripted UAT, and manual UAT commands
   without running their product assertions. Use supported self-test or dry-run
   modes when present.
7. Validate Compose files, project/profile identity, ports, expected active
   slot, bootstrap/no-op commands, provenance label keys, and canonical
   restoration command.
8. Validate that every runner accepts its documented output paths and that the
   evidence directory is empty or intentionally replaceable.
9. Create the evidence inventory before SIT. Include every mandatory file,
   phase receipt, raw log, failure record, summary, and manifest path.
10. Record source commit/tree/dirty state, configuration and inventory hashes,
    migration identity, and expected provenance as the frozen candidate.
11. Write `preflight-result.json`, then `candidate.json`, only after all checks
    pass. Update the human verification record with the same identities.

## Executable preflight contract

Prefer one repository-owned preflight command. Until one exists, perform the
checks explicitly and retain their outputs. The contract must catch:

- missing reset acknowledgements
- missing or misspelled database variables
- shared or unsafe database identities
- absent Compose files or profiles
- occupied required ports and unexpected active projects
- wrong or missing image-label keys
- unsupported absolute/relative output-path forms
- missing test files, scripts, accounts, fixtures, or evidence destinations
- Markdown evidence that would fail repository link validation

Do not build product images merely to discover labels. Audit Dockerfiles and
deployment configuration for expected keys; SIT confirms the built values.

## Receipts

Write receipts under the sprint evidence directory using the shared protocol:

- `preflight-result.json`
- `candidate.json`
- `attempts/preflight-<attempt>.json` for every superseded attempt

The candidate fingerprint is immutable. A later SIT receipt may add observed
image IDs and labels but must not rewrite the source identity.

## Failure handling

Classify failures here as `preflight/setup` unless they reveal a product or
product-decision issue. Correct setup and rerun preflight. If a correction
changes tracked product, test, harness, migration, seed, manifest, bootstrap,
or deployment source, commit it before issuing a new candidate receipt.

Do not characterize a preflight failure as SIT. Do not write a passing
candidate receipt from partial checks.

## Finish criteria

Finish only when:

- the implementation commit is clean
- the acceptance inventory is complete and frozen
- all environment and deployment prerequisites pass
- evidence paths and required artifacts are declared
- preflight and candidate receipts parse and agree
- no SIT or UAT assertion has run
