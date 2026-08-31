---
name: tessara-validation-preflight
description: Audit passing Tessara Validation Readiness and Candidate Rehearsal receipts, any required post-SIT convergence authorization, and unchanged source/environment prerequisites, then freeze one source-exact candidate without running rehearsal, SIT, or UAT. Use after the mandatory mutable readiness/rehearsal cycle passes, when freezing an initial or successor sprint candidate, or when producing the prerequisite receipts consumed by tessara-sit.
---

# Tessara Validation Preflight

Prepare and freeze a candidate for `tessara-sit`. Do not execute SIT, deployed
acceptance smoke, scripted UAT, manual UAT, Test Readiness, or Candidate
Rehearsal in this skill.

Inspect the tracked sprint validation contract before loading a protocol. When
it declares `policy_version: tessara-validation-v2`, read
[`../tessara-sprint-validation/references/validation-policy-v2.md`](../tessara-sprint-validation/references/validation-policy-v2.md)
completely. Consume compact Readiness/Rehearsal certificates and an
authenticated impact assessment. Do not reopen their raw evidence or reject
them merely because an unrelated repository commit changed.

When it declares `policy_version: tessara-validation-v3`, read v2 and then
[`../tessara-sprint-validation/references/validation-policy-v3.md`](../tessara-sprint-validation/references/validation-policy-v3.md)
completely. Validate the canonical successor-impact plan when this is a
successor freeze. Require its exact predecessor/successor source and candidate
fingerprints, diff digest, owned-domain mapping, affected Readiness/Rehearsal/
Preflight closure, correction-batch convergence, and fallback decision.

Only when the contract selects neither v2 nor v3, read
[`../tessara-sprint-validation/references/validation-protocol.md`](../tessara-sprint-validation/references/validation-protocol.md)
completely. It defines the legacy receipt schemas, fingerprints,
classifications, and invalidation authority.

When an earlier candidate was invalidated after SIT, also read
[`../tessara-sprint-validation/references/post-sit-defect-convergence.md`](../tessara-sprint-validation/references/post-sit-defect-convergence.md)
completely and validate its records against
[`../tessara-sprint-validation/references/post-sit-defect-convergence.schema.json`](../tessara-sprint-validation/references/post-sit-defect-convergence.schema.json).

When any prior v2 target, lane, or scenario failed, also read
[`../tessara-sprint-validation/references/defect-provenance.md`](../tessara-sprint-validation/references/defect-provenance.md)
and validate every retained record against its schema.

Read
[`../tessara-sprint-validation/references/uat-scenario-classification.md`](../tessara-sprint-validation/references/uat-scenario-classification.md)
before freezing the acceptance inventory.

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
4. When an earlier candidate was invalidated after SIT, parse and hash the
   defect-harvest, defect-batch, correction-impact, focused repair,
   restoration, and final-certification-entry records. Require the coordinator
   to have authorized return to the legacy complete pass, v2 affected-lane
   pre-freeze recertification, or v3 successor-impact selection, and require
   the resulting certificates to bind the final corrected source and
   authenticated inherited coverage.
5. Audit all changes and require one clean implementation commit. Require
   every defect-provenance record to be verified or validly superseded. Reject
   an open implementation-exit gap, unresolved provenance, blocked broad
   rerun, or test expectation change without approved authority and a
   test-change-log entry.
6. Reconcile every roadmap exit condition with automated evidence and smoke
   where applicable. Audit the exact screen/feature touch inventory and require
   every sprint-delivered user-facing surface to map to a human exploratory UAT
   scenario with automated prerequisites; purely technical clauses may record
   manual coverage as not applicable.
7. Discover required environment variables from the actual test and runner
   sources. Do not infer similarly named variables.
8. Validate database URLs, unique disposable identities, reachability,
   credentials, reset authorization, and actual migration-ledger tables.
9. Audit the readiness evidence that validated Rust, Playwright, smoke,
   scripted UAT, and manual UAT commands without running product assertions.
10. Validate Compose files, project/profile identity, ports, expected active
   slot, bootstrap/no-op commands, provenance label keys, and canonical
   restoration command.
11. Audit the readiness evidence that every runner accepts its documented
    output paths, then validate that the evidence directory is empty or
    intentionally replaceable.
12. Create the evidence inventory before SIT. Include every mandatory file,
   phase receipt, raw log, failure record, summary, manifest path, and the
   planned conditional Post-SIT Defect Convergence records.
13. Record source commit/tree/dirty state, configuration and inventory hashes,
   migration identity, and expected provenance as the frozen candidate.
14. Write `preflight-result.json`, then `candidate.json`, only after all checks
    pass. Update the human verification record with the same identities.

For v2, step 3 validates the current dependency-domain fingerprints and the
coordinator's impact decision instead of requiring unrelated whole-tree
identity equality. An intersecting Readiness/Rehearsal dependency requires the
corresponding affected-lane recertification certificate; a Preflight-only
runner change does not reopen either upstream phase. Unknown or unauthenticated
impact still returns to complete affected-phase execution.

## Executable preflight contract

For v3, execute the declared Preflight lane through
`Invoke-TessaraValidationLane`; a repository script may be its product action
but cannot own phase orchestration, topology, cleanup, publication, or
certificates. For retained legacy/v2 contracts, use only the runner named by
that contract. The contract must catch:

- missing reset acknowledgements
- missing or misspelled database variables
- shared or unsafe database identities
- absent Compose files or profiles
- occupied required ports and unexpected active projects
- wrong or missing image-label keys
- unsupported absolute/relative output-path forms
- missing test files, scripts, accounts, fixtures, or evidence destinations
- unresolved defect provenance, process drift, or implementation-exit gaps
- changed test expectations without exact authority and replacement coverage
- Markdown evidence that would fail repository link validation

Do not build product images merely to discover labels. Audit Dockerfiles and
deployment configuration for expected keys; SIT confirms the built values.

Do not repeat the complete readiness gate or rehearsal here. Perform only the
freeze-boundary audit and inexpensive prerequisite reconfirmation needed to
prove nothing changed since their passing receipts.

For v2, validate only the compact certificate documents, prerequisite hashes,
declared coverage, dependency fingerprints, and sealed phase evidence-index
hashes. Raw evidence remains cold unless a certificate fails authentication or
an explicit audit is requested.

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

For v2, emit and validate `defect-provenance.json` before correcting a failed
Preflight check. A bounded setup/environment record may authorize the narrow
Preflight rerun; product, fixture, harness, ambiguity, or mixed provenance
returns to the coordinator-selected earlier boundary.

## Finish criteria

Finish only when:

- the implementation commit is clean
- the acceptance inventory is complete and frozen
- every delivered user-facing screen/feature has planned human exploratory
  touch coverage and no manual scenario is a machine-evidence review checklist
- all environment and deployment prerequisites pass
- evidence paths and required artifacts are declared
- the defect-provenance chronology is complete and resolved
- preflight and candidate receipts parse and agree
- no SIT or UAT assertion has run
