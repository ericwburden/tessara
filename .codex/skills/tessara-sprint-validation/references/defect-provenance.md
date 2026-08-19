# Tessara Defect Provenance Gate

Use this gate for every failed implementation target, Validation Readiness
lane, Candidate Rehearsal lane, Preflight check, SIT lane, or UAT scenario.
It determines whether the failure originated in product implementation or in
the delivery process before any correction or broad rerun begins.

Validate each record against
[`defect-provenance.schema.json`](defect-provenance.schema.json). Retain the
record beside the failed attempt as `defect-provenance.json`; it is part of the
failure chronology and must be referenced by the next correction-impact
assessment and phase certificate.

## Required comparison

Compare the failed attempt with all of the following authenticated inputs:

- source commit, tree, dirty state, and changed dependency domains;
- the tracked validation contract and governing requirement clauses;
- implementation-readiness target coverage and exact commands;
- fixture and reference-topology identity;
- environment fingerprint and required database, service, port, and account
  bindings;
- acceptance inventory, test-change log, runner, and assertion version; and
- the failed receipt, raw output, and whether assertions or product actions
  started.

Never infer provenance from the command exit code alone. Collapse repeated
symptoms with one root cause into one finding while retaining every affected
test, lane, or scenario as evidence.

## Finding classifications

Classify every distinct finding as exactly one of:

- `product`: implementation violates an approved requirement under a valid
  fixture, environment, harness, and assertion;
- `fixture`: seed, reference topology, account, identity, or expected data is
  missing, stale, contradictory, or not produced through its canonical owner;
- `harness`: runner, setup, teardown, command, selector, evidence adapter, or
  test utility does not execute the declared contract correctly;
- `environment`: required tool, credential binding, database, port, service,
  or external state is absent or differs from the authenticated environment
  contract;
- `evidence-finalization`: assertions completed with immutable raw results but
  publication, hashing, indexing, or receipt finalization failed;
- `flaky`: a narrow reproducer demonstrates nondeterminism without a source,
  fixture, harness, or environment difference; suspicion alone is not enough;
- `contract-ambiguity`: the assertion and implementation disagree and no
  approved requirement unambiguously selects the intended behavior; or
- `product-decision`: correction requires a new or changed product decision.

Use `origin_boundary: implementation` when all findings are `product`. Use
`process` for fixture, harness, environment, or evidence-finalization findings.
Use `mixed` when both occur, and `unresolved` while any finding lacks a bounded
classification.

## Drift detection

Set `implementation_exit_gap` when formal validation discovers a product,
fixture, harness, environment, or acceptance-inventory failure that the exact
selected implementation target was required to exercise. This means the
implementation exit gate or its command coverage is incomplete even when the
underlying product also has a defect.

Set `process_drift` when the formal runner uses a different fixture,
environment, inventory, command, or assertion contract from the one proven at
implementation exit. Both flags may be true.

An unexplained test expectation change after a failed attempt is itself an
open `contract-ambiguity` finding. Record the old assertion, proposed new
assertion, governing authority, reason the prior assertion is superseded, and
equal-or-stronger replacement coverage. A test-change-log entry is required
before the finding can close.

## Automatic routing

The gate automates detection, classification, invalidation, rerun blocking,
and workflow routing. It never edits product code, fixtures, runners, tests,
or expected values and never decides that an assertion is obsolete.

- `product`: invalidate implementation readiness, return to focused
  implementation, and recertify every affected pre-freeze dependency.
- `fixture` or `harness`: quarantine the formal attempt, repair the owning
  process surface under implementation rules, prove its focused reproducer,
  and recertify every consuming lane.
- `environment`: restore or correct the declared environment, then rerun the
  affected setup/lane boundary only when fingerprints prove product and
  assertions unaffected; otherwise widen conservatively.
- `evidence-finalization`: rerun finalization only when immutable raw results
  authenticate and no assertion or environment input changed.
- `flaky`: block the broad rerun until the nondeterminism has a focused
  reproducer and an owned correction; retries are not a correction.
- `contract-ambiguity` or `product-decision`: pause and request the governing
  decision. Do not change the assertion or implementation while ambiguous.
- `mixed` or uncertain provenance: apply the union of affected domains and
  select the broader safe invalidation boundary.

The record's `routing.full_rerun_blocked` remains true until every required
focused reproducer passes, every expectation change has authority, the
correction is clean and committed, and the affected implementation targets
pass. After two consecutive failures of the same formal lane, do not launch it
again until the concentrated defect batch passes its focused reproducers.
After three, classify the lane as a validation-platform incident and require a
root-cause correction before another full launch.

## Phase boundaries

- Implementation exit must have no open provenance record and must prove the
  exact formal inventory, fixture, and environment contract it claims.
- A failed Readiness or Rehearsal attempt cannot authorize correction or
  recertification until its provenance record is complete.
- Preflight rejects unresolved provenance, undocumented expectation changes,
  or an implementation-exit gap not closed by a new passing readiness result.
- SIT and UAT retain the failed candidate evidence, emit provenance before
  correction, and return invalidation authority to the coordinator.
- Closeout requires every provenance record to be `verified` or `superseded`
  by a later authenticated record and includes their hashes in the final
  chronology audit.

## Completion

A record is `verified` only when its correction identifies exact changed paths
and dependency domains, all required focused reproducers pass, expectation
changes are authorized and logged, the required implementation targets pass
against clean committed source, and the coordinator records the next allowed
validation boundary. A correction may close process drift without changing
product source, but it may not reuse the failed formal attempt as a pass.
