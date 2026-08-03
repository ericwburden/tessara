---
name: tessara-validation-preflight
description: Audit passing Tessara Validation Readiness and Candidate Rehearsal receipts, verify unchanged source and environment prerequisites, and freeze one source-exact candidate without running rehearsal, SIT, or UAT. Use after the mandatory mutable readiness/rehearsal cycle passes, when freezing a sprint candidate or producing the prerequisite receipts consumed by tessara-sit.
---

# Tessara Validation Preflight

Prepare and freeze a candidate for `tessara-sit`. Do not execute SIT, deployed
acceptance smoke, scripted UAT, manual UAT, Test Readiness, or Candidate
Rehearsal in this skill.

Before acting, read
[`../tessara-sprint-validation/references/validation-protocol.md`](../tessara-sprint-validation/references/validation-protocol.md)
completely. It defines receipt schemas, fingerprints, classifications, and
invalidation authority.

## Inputs

- passing `validation-readiness-result.json` and
  `candidate-rehearsal-result.json`
- sprint label and slug
- intended sprint worktree, branch, and implementation commit
- roadmap, sprint plan, and validation record
- planned SIT/UAT commands and acceptance inventory
- deployment profile, expected service slot, and handoff URL
- evidence directory

## Required execution order

1. Parse and hash the readiness and rehearsal receipts. Require both to pass,
   require rehearsal to name the readiness receipt, and reject any
   authoritative SIT/UAT claim in rehearsal evidence.
2. Confirm repository instructions, worktree, branch, and sprint scope.
3. Verify current clean commit/tree, acceptance inventory, deployment inputs,
   environment, and source provenance exactly match the passing rehearsal.
   Any mismatch returns to the coordinator for a complete new readiness and
   rehearsal cycle; preflight never patches or partially refreshes them.
4. Audit all changes and require one clean implementation commit.
5. Reconcile every roadmap exit condition with automated, smoke, and manual
   UAT coverage in the validation record.
6. Discover required environment variables from the actual test and runner
   sources. Do not infer similarly named variables.
7. Validate database URLs, unique disposable identities, reachability,
   credentials, reset authorization, and actual migration-ledger tables.
8. Audit the readiness evidence that validated Rust, Playwright, smoke,
   scripted UAT, and manual UAT commands without running product assertions.
9. Validate Compose files, project/profile identity, ports, expected active
   slot, bootstrap/no-op commands, provenance label keys, and canonical
   restoration command.
10. Audit the readiness evidence that every runner accepts its documented
    output paths, then validate that the evidence directory is empty or
    intentionally replaceable.
11. Create the evidence inventory before SIT. Include every mandatory file,
   phase receipt, raw log, failure record, summary, and manifest path.
12. Record source commit/tree/dirty state, configuration and inventory hashes,
   migration identity, and expected provenance as the frozen candidate.
13. Write `preflight-result.json`, then `candidate.json`, only after all checks
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

Do not repeat the complete readiness gate or rehearsal here. Perform only the
freeze-boundary audit and inexpensive prerequisite reconfirmation needed to
prove nothing changed since their passing receipts.

## Receipts

Write receipts under the sprint evidence directory using the shared protocol:

- `preflight-result.json`
- `candidate.json`
- `attempts/preflight-<attempt>.json` for every superseded attempt

The candidate fingerprint is immutable. A later SIT receipt may add observed
image IDs and labels but must not rewrite the source identity.

Both passing pre-freeze receipts and hashes are prerequisites of
`preflight-result.json`; `candidate.json` names and hashes preflight as usual.

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
