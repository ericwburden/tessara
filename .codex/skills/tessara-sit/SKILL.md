---
name: tessara-sit
description: Execute and retain Tessara system integration testing for one preflight-approved frozen candidate, including static and boundary checks, isolated database-backed Rust tests, full Playwright, source-exact deployment, and integrated acceptance smoke. Use when running or rerunning SIT, collecting complete phase results, diagnosing a SIT lane, or producing the SIT receipt required before UAT.
---

# Tessara SIT

Run the complete SIT phase for one frozen candidate. Do not run UAT or
authorize closeout.

Before acting, read
[`../tessara-sprint-validation/references/validation-protocol.md`](../tessara-sprint-validation/references/validation-protocol.md)
completely.

## Prerequisites

Require parsed, passing `preflight-result.json` and `candidate.json`. Reject
the run when:

- their candidate fingerprints differ
- tracked source or the acceptance inventory changed
- the worktree is unexpectedly dirty
- required databases, reset authorization, ports, tools, evidence paths, or
  deployment inputs no longer match preflight

Do not repair a stale prerequisite silently. Return to
`tessara-sprint-validation` for the invalidation decision.

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

## Failure handling

Stop downstream dependent lanes after a failed lane. Record the command,
candidate, environment, stage, whether assertions started, and raw evidence.
Use the narrowest safe reproducer for diagnosis, but never substitute it for
the authoritative lane.

Do not decide that all earlier phases are invalid merely because a command
returned nonzero. Apply the shared invalidation matrix through
`tessara-sprint-validation`:

- candidate-affecting correction: refreeze and restart all SIT
- shared-environment correction: rerun affected and downstream lanes
- lane setup failure before assertions: rerun that lane
- evidence finalization failure with intact raw results: rerun finalization
- assertion failure without a source change: diagnose, then rerun the complete
  failed lane; the coordinator decides whether upstream receipts remain valid

Mark superseded attempts explicitly. Never merge evidence across candidate
fingerprints.

## Finish criteria

Finish only when every authoritative lane passed, evidence parses and hashes,
the candidate fingerprint is unchanged, the canonical topology is restored,
and `sit-result.json` authorizes `tessara-uat`—not closeout.
