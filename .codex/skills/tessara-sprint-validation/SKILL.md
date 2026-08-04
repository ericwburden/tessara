---
name: tessara-sprint-validation
description: Coordinate Tessara sprint validation across validation readiness, mutable candidate rehearsal, preflight and candidate freeze, SIT, deployed acceptance smoke, UAT, post-SIT defect harvesting and convergence, evidence integrity, scoped failure invalidation, and closeout authorization. Use when planning or executing the full validation regime, preparing a candidate for freeze, batching rehearsal or UAT defects, authorizing an impact cone or focused repair validation, deciding what must rerun after a failure, reconciling phase receipts, determining whether SIT or UAT passed, reopening validation from closeout, or authorizing an exact sprint candidate for closeout.
---

# Tessara Sprint Validation Coordinator

Own validation policy and authorization. Delegate phase execution to:

- `tessara-validation-preflight`
- `tessara-sit`
- `tessara-uat`

Before acting, read
[`references/validation-protocol.md`](references/validation-protocol.md)
completely. It is authoritative for receipts, fingerprints, classifications,
result collection, and invalidation scope.

Before creating a candidate, also read
[`references/validation-readiness.md`](references/validation-readiness.md)
completely. It is authoritative for the Test Readiness Gate and Candidate
Rehearsal. This coordinator owns both gates; phase skills must not duplicate
them.

After authoritative SIT has started, read
[`references/post-sit-defect-convergence.md`](references/post-sit-defect-convergence.md)
completely before handling a candidate-invalidating UAT failure or any
correction proposed from SIT/UAT findings. This coordinator alone owns defect
harvest status, the consolidated correction batch, impact-cone authorization,
convergence, and permission to return to full readiness/rehearsal.
Validate convergence records against
[`references/post-sit-defect-convergence.schema.json`](references/post-sit-defect-convergence.schema.json).

## State machine

```text
Test Readiness -> Candidate Rehearsal -> preflight/freeze -> SIT -> UAT
      ^                  |                    ^             ^      ^
      |__________________|____________________|_____________|______|
                   coordinator-selected invalidation boundary

SIT passed -> UAT failure -> diagnostic defect harvest -> mutable batch
                                      -> impact assessment
                                      -> focused repair validation
                                      -> final readiness/rehearsal -> new freeze
                                      -> complete SIT -> complete UAT
```

Preserve these invariants:

- UAT never starts before authoritative SIT passes.
- No candidate freezes until the complete readiness gate and rehearsal both
  pass cleanly against the same mutable source identity.
- Rehearsal is diagnostic and non-authoritative; it is never called SIT or UAT.
- Deployed acceptance smoke belongs to SIT.
- One candidate fingerprint covers all authoritative SIT and UAT evidence.
- Candidate-affecting corrections invalidate all SIT and UAT.
- Post-invalidation UAT may continue only as safe, explicitly
  non-authoritative diagnostic defect harvesting.
- Focused repair validation proves a mutable correction cone only; it never
  authorizes a candidate, SIT, UAT, or closeout.
- A successor candidate receives complete readiness, rehearsal, SIT, and UAT
  regardless of focused repair results.
- Closeout never originates an acceptance check.
- Product decisions pause for user direction.

Do not interpret every command failure as a candidate failure. Record its
stage and `assertions_started`, then apply the shared invalidation matrix.

## Establish the validation record

Copy `assets/sprint-validation-record.md` to
`docs/sprints/<sprint-slug>-verification.md` when absent. Preserve useful
existing content when present.

Before freeze, record:

- every roadmap exit-condition clause
- relevant product, authorization, lifecycle, deployment, migration,
  compatibility, recovery, and rollback risks
- automated, deployed-smoke, and manual UAT proof per clause
- exact commands, environments, accounts, fixtures, topology, and evidence
  paths
- the required receipt and evidence inventory, including planned conditional
  Post-SIT Defect Convergence record paths before validation begins
- candidate and environment fingerprint inputs

Require smoke, UAT, Playwright, fixtures, manifests, and deployment/bootstrap
coverage to change in the same candidate as the behavior that makes them
stale.

## Full-regime execution

1. Execute the complete Test Readiness Gate and retain
   `validation-readiness-result.json`.
2. Execute the complete non-authoritative Candidate Rehearsal and retain
   `candidate-rehearsal-result.json`.
3. Collect all safe-to-discover rehearsal defects into one batch. Correct the
   batch while source remains mutable, then repeat the complete readiness gate
   and complete rehearsal until both pass cleanly. A narrow reproducer may
   diagnose a defect but cannot satisfy either gate or replace an affected
   rehearsal lane.
4. Invoke `tessara-validation-preflight` and require passing
   `preflight-result.json` plus `candidate.json`.
5. Verify their hashes and immutable candidate fingerprint.
6. Invoke `tessara-sit` and require all authoritative lane receipts plus a
   passing `sit-result.json`.
7. Verify SIT used the preflight candidate and declared environment contract.
8. Invoke `tessara-uat`. If it passes, require passing scripted/manual
   evidence plus `uat-result.json`.
9. If UAT finds a candidate-invalidating defect, do not create a passing
   `uat-result.json`. Execute the Post-SIT Defect Convergence Cycle, retain all
   required structured records, and return to step 1 only after the focused
   repair cone passes with no open defect.
10. Audit the failure chronology and every invalidation decision.
11. Validate the complete evidence manifest, canonical handoff topology,
   provenance, health, and acceptance mapping.
12. Write `closeout-authorization.json` and update the human verification
   record only when every requirement passes.

If an in-flight frozen candidate fails, apply the existing invalidation matrix
before changing the process. When a candidate-affecting correction is needed,
retain and classify the failed evidence, restore the canonical environment,
invalidate the candidate, and stop formal testing. Then perform the readiness
and rehearsal cycle before freezing its successor.

When the user requests only one phase, route to that phase skill but still
enforce prerequisite receipts. A phase skill cannot bypass this coordinator's
authority boundaries.

## Post-SIT defect convergence

On the first candidate-invalidating UAT failure, retain the failure, mark the
candidate invalid, and forbid a passing `uat-result.json`. Direct UAT to finish
all safe independent scenarios as non-authoritative diagnostic harvesting;
record unsafe or dependent scenarios as blocked with exact reasons.

After harvesting, consolidate every product, harness, fixture, environment,
acceptance-inventory, and evidence defect into one mutable correction batch.
Require Tessara implementation rules for tracked corrections. Produce and
authorize an evidence-based correction-impact assessment before any focused
rerun. Default identity, authorization, protocol, migration, shared fixture,
shared environment, and cross-module changes to a broad cone. If the cone
cannot be bounded confidently, require the complete Candidate Rehearsal.

Run the declared affected SIT and automated/manual UAT portions as `focused
repair validation`. Do not issue a fingerprint or authoritative phase result.
Collect newly exposed defects fail-late, correct them as one next batch,
reassess the cone, and repeat until every declared check passes and no defect
is open. Then record the coordinator's decision to enter final certification,
rerun complete readiness and rehearsal, freeze a new candidate, and rerun
complete authoritative SIT and UAT from the beginning.

## Failure and invalidation decision

For every failure:

1. Retain the failed receipt and raw evidence.
2. Classify it using the protocol vocabulary.
3. Establish whether assertions or product actions began.
4. Run the narrowest safe reproducer for diagnosis.
5. Identify any tracked-source or shared-environment change.
6. Select the minimum safe invalidation boundary from the matrix.
7. Record the decision and rationale in the receipt and verification record.
8. Mark invalidated attempts superseded before resuming.

Examples:

- A missing database variable found before assertions reruns preparation and
  the affected lane, not unrelated completed lanes, when fingerprints remain
  valid.
- A wrong evidence output path after immutable raw results reruns finalization.
- A changed test, harness, fixture, or product source creates a new candidate
  only after the complete readiness and rehearsal gates pass, then restarts
  all SIT.
- A flaky assertion requires narrow diagnosis and a complete authoritative
  rerun of its lane; upstream reuse requires matching fingerprints and an
  explicit non-impact rationale.

Never choose a narrower scope merely to avoid expensive work.

## Result collection and recovery

- Prefer repository-owned phase runners over ad hoc compound commands.
- Run independent sibling checks fail-late within a lane or isolated scenario
  set and aggregate their results.
- Retain start/completion receipts, append-only logs, heartbeats, durations,
  and completion sentinels for long-running work.
- When a controlling tool session disappears, inspect retained completion
  state before relaunching.
- Keep authoritative results distinct from diagnostic and superseded attempts.

## Closeout authorization

Authorize `tessara-sprint-closeout` only when:

- preflight passed before SIT
- one immutable candidate fingerprint covers all authoritative receipts
- every SIT lane and deployed acceptance smoke passed
- scripted and every manual UAT scenario passed after SIT
- every roadmap clause maps to automated and manual evidence
- all invalidation decisions were satisfied
- no required evidence is missing, stale, malformed, or unhashed
- no product decision or open acceptance defect remains
- the intended candidate route, topology, provenance, and health are restored

Write `closeout-authorization.json` with hashes of the prerequisite receipts
and evidence manifest. Name the evidence-source commit separately from later
documentation-only commits.

If closeout discovers missing coverage or executable evidence, reopen at the
boundary chosen by this coordinator. Documentation-only corrections may stay
in closeout when they cannot alter executable behavior or test interpretation.

## Finish criteria

Do not report validation complete unless the receipt chain parses and hashes,
all authoritative phases passed, failure invalidations are satisfied, the
verification record explicitly authorizes closeout, and the application is in
the intended healthy handoff state.
