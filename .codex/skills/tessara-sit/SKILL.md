---
name: tessara-sit
description: Execute and retain Tessara system integration testing for one preflight-approved frozen candidate, including static and boundary checks, isolated database-backed Rust tests, full Playwright, source-exact deployment, and integrated acceptance smoke; also execute coordinator-assigned diagnostic SIT portions during focused repair validation. Use when running or rerunning authoritative SIT, collecting complete phase results, diagnosing a SIT lane, executing a declared repair cone, or producing the SIT receipt required before UAT.
---

# Tessara SIT

Run authoritative SIT for one frozen candidate: complete coverage under v2 or
the authenticated executed/inherited v3 coverage plan. Do not run UAT or
authorize closeout.

Inspect the tracked sprint validation contract before loading a protocol. When
it declares `policy_version: tessara-validation-v2`, read
[`../tessara-sprint-validation/references/validation-policy-v2.md`](../tessara-sprint-validation/references/validation-policy-v2.md)
completely. Authenticate compact prerequisite certificates and write one
phase-local evidence index plus a compact SIT certificate.

When it declares `policy_version: tessara-validation-v3`, read v2 and then
[`../tessara-sprint-validation/references/validation-policy-v3.md`](../tessara-sprint-validation/references/validation-policy-v3.md)
completely. Validate the canonical successor-impact plan, execute only items
marked `execute` through `Invoke-TessaraValidationLane`, and authenticate
inherited items through the shared certifier and immediate predecessor. Never
invoke a sprint-owned phase runner or describe inherited assertions as run.

Only when the contract selects neither v2 nor v3, read
[`../tessara-sprint-validation/references/validation-protocol.md`](../tessara-sprint-validation/references/validation-protocol.md)
completely.

When the coordinator assigns a focused repair portion, also read
[`../tessara-sprint-validation/references/post-sit-defect-convergence.md`](../tessara-sprint-validation/references/post-sit-defect-convergence.md)
completely.

On any failed SIT lane, also read
[`../tessara-sprint-validation/references/defect-provenance.md`](../tessara-sprint-validation/references/defect-provenance.md)
and emit a schema-valid `defect-provenance.json` beside the failed attempt
before correction or another full lane launch.

## Prerequisites

Require parsed, passing `preflight-result.json` and `candidate.json`. Reject
the run when:

- their candidate fingerprints differ
- tracked source or the acceptance inventory changed
- the worktree is unexpectedly dirty
- required databases, reset authorization, ports, tools, evidence paths, or
  deployment inputs no longer match preflight
- preflight does not name valid passing Validation Readiness and Candidate
  Rehearsal receipts for the exact frozen source/environment identities

Do not repair a stale prerequisite silently. Return to
`tessara-sprint-validation` for the invalidation decision.

Do not relabel or reuse rehearsal output as SIT evidence. Every authoritative
SIT lane reruns after freeze and produces its own receipt.

Under v2, every SIT lane still executes for the frozen candidate. A successor
candidate cannot inherit a SIT lane from its predecessor, even when pre-freeze
Readiness or Rehearsal reused authenticated unaffected lanes.

Under v3, a bounded successor executes affected SIT lanes plus prerequisite
closure and may inherit only non-impact lanes whose dependency, fixture,
environment, acceptance, runner, platform, adapter, and recursive prerequisite
inheritance fingerprints are unchanged. Unknown paths, broad-risk domains,
open defects, expectation ambiguity, or a non-immediate predecessor require
complete SIT. `sit-result.json` lists executed and inherited lanes separately;
inherited lanes have no successor execution timestamps.

## Phase model

Execute every lane as:

```text
prepare -> execute -> finalize
```

Write `started` state before expensive work, append raw output continuously,
record heartbeats during long commands, and write `completed` atomically. If a
tool session disappears, inspect receipts and logs before rerunning anything.

Within a lane, run independent checks fail-late and collect every safe result.
Stop dependent or destructive checks when their prerequisite state failed.
Do not discard passing sibling results, but do not call the lane passed unless
every required assertion passed.

## Authoritative lanes

### 1. Static and boundaries

- formatting and compilation/static checks
- sprint-specific package, schema, manifest, boundary, and Compose checks
- Markdown links and evidence-contract self-tests
- migration baseline and expected provenance-key checks

Use a non-overlapping static command when the repository provides one. Do not
run a monolithic suite here and then duplicate it in the Rust lane without an
explicit reason in the validation record.

### 2. Rust workspace

- run `cargo test --workspace --locked` by default
- explicitly set every database variable discovered by preflight
- reset pairwise-distinct disposable databases before the lane
- retain intentional ignored-test explanations
- include sprint-specific release, timing, migration, upgrade, or rollback
  proofs not covered by the workspace command

### 3. Playwright

- prepare the documented source-exact topology and fixtures
- audit built image commit/tree/dirty labels using preflight's exact keys
- run `npm --prefix .\end2end test` from the repository root
- retain the repository-owned evidence wrapper when required
- record passed, failed, skipped, and did-not-run counts

Never use bare root-level `npx playwright test`.

### 4. Deployed acceptance smoke

Build or reuse only images proven to represent the frozen source candidate.
Run `scripts/smoke.ps1` or the sprint-specific equivalent and cover changed
integration contracts, including relevant:

- health, gateway, slot, and source provenance
- login/session and constrained non-admin access
- routes, complete documents, navigation, manifests, and lifecycle
- contained module or Supervisor failure while Core stays usable
- bootstrap/materialization first apply, no-op, restart, recovery, and UAT data

Restore the documented canonical handoff topology in `finally` behavior and
retain final health and routing evidence.

## Evidence and result

Retain per-lane receipts, logs, durations, commands, environment fingerprint,
assertion-start markers, and corrections. Produce `sit-result.json` only after
all four lanes pass for the same candidate and environment contract.

Generate or update the evidence manifest during SIT. Do not postpone discovery
of missing required evidence until closeout.

For v2, hash artifacts as they are published, seal one SIT-attempt
`evidence-index.json`, and add only the compact SIT certificate/index hashes to
`evidence-chain.json`. Do not rebuild a sprint-wide raw-file manifest.

## Failure handling

Stop downstream dependent lanes after a failed lane. Record the command,
candidate, environment, stage, whether assertions started, and raw evidence.
Use the narrowest safe reproducer for diagnosis, but never substitute it for
the authoritative lane.

Compare the failure with the frozen fixture, environment, acceptance
inventory, and implementation-readiness proof. Route product, fixture,
harness, environment, evidence, flaky, and ambiguous findings through the
provenance record. Do not change source or tests inside authoritative SIT and
do not rerun broadly while `routing.full_rerun_blocked` is true.

Do not decide that all earlier phases are invalid merely because a command
returned nonzero. Apply the shared invalidation matrix through
`tessara-sprint-validation`:

- candidate-affecting correction: under v2 refreeze and restart all SIT; under
  v3 finish focused repair/batch convergence, refreeze, and follow the
  authenticated successor-impact plan
- before any successor freeze, the coordinator must complete the legacy full
  Readiness/Rehearsal cycle or v2 affected-lane pre-freeze recertification
- shared-environment correction: rerun affected and downstream lanes
- lane setup failure before assertions: rerun that lane
- evidence finalization failure with intact raw results: rerun finalization
- assertion failure without a source change: diagnose, then rerun the complete
  failed lane; the coordinator decides whether upstream receipts remain valid

Mark superseded attempts explicitly. Never merge evidence across candidate
fingerprints.

## Focused repair assignment

After a post-SIT correction, execute only the static, integration,
Playwright, deployment, conformance, nondisclosure, smoke, or recovery checks
in the coordinator-authorized correction-impact cone. Label them `focused
repair validation` and `authoritative: false`. Bind them to the mutable source
and environment identities, not a candidate fingerprint, and never emit or
update `sit-result.json`.

Run safe independent checks fail-late, retain exact blocked dependencies, and
return newly discovered defects to the coordinator. Do not select or reduce
the cone, decide convergence, or authorize readiness/rehearsal. A narrow
reproducer remains diagnostic and cannot satisfy a declared cone item. Even a
passing focused SIT portion must be rerun in the complete authoritative SIT
suite after the successor candidate freezes.

## Finish criteria

Finish only when every authoritative lane passed, evidence parses and hashes,
the candidate fingerprint is unchanged, the canonical topology is restored,
every SIT provenance record is verified or validly superseded, and
`sit-result.json` authorizes `tessara-uat`—not closeout.
