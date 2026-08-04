---
name: tessara-uat
description: Execute and retain Tessara scripted and manual user acceptance testing for the exact candidate authorized by SIT, including coordinator-directed fail-late diagnostic harvesting after invalidation and focused repair UAT checks. Use when running formal sprint UAT, exercising role, responsive, failure, recovery, upgrade, rollback, or business scenarios, collecting scenario dispositions or acceptance decisions, or producing UAT evidence consumed by sprint validation and closeout.
---

# Tessara UAT

Run formal acceptance for the exact SIT-authorized candidate. Do not change
the acceptance inventory while executing it and do not authorize closeout
directly.

Before acting, read
[`../tessara-sprint-validation/references/validation-protocol.md`](../tessara-sprint-validation/references/validation-protocol.md)
completely.

When a candidate-invalidating failure occurs or the coordinator assigns a
focused repair portion, also read
[`../tessara-sprint-validation/references/post-sit-defect-convergence.md`](../tessara-sprint-validation/references/post-sit-defect-convergence.md)
completely.

## Prerequisites

Require parsed, passing:

- `preflight-result.json`
- `candidate.json`
- `sit-result.json`

Reject UAT if their candidate or environment fingerprints differ, any SIT lane
is incomplete, the UAT inventory changed, required accounts/fixtures are
missing, or the intended topology is not in its recorded starting state.

Also require the receipt chain to include the passing pre-freeze Validation
Readiness and Candidate Rehearsal receipts audited by preflight. Rehearsal's
automated UAT diagnostics are not formal UAT evidence and cannot replace any
scripted or manual scenario below.

## Required execution order

1. Reconfirm candidate provenance, SIT authorization, active slot, health,
   fixtures, roles/accounts, browser configuration, and evidence paths.
2. Hash the frozen scripted and manual UAT inventory.
3. Run `scripts/uat-sprint.ps1 -BaseUrl "http://localhost:8080"` or the
   documented sprint equivalent.
4. Run every recorded manual business scenario.
5. Include role/scope, responsive, failure containment, restart/recovery,
   upgrade, and rollback scenarios when their contracts changed.
6. Restore the intended canonical handoff topology and verify health.
7. Validate all UAT JSON, links, screenshots/log references, and hashes.
8. Write `uat-result.json` only when scripted and manual UAT pass.

## Scenario execution

Record for every scenario:

- candidate and environment fingerprints
- role and exact starting state
- action and expected visible result
- actual result and pass/fail decision
- evidence paths and timestamps
- cleanup and restored state

Run independent scenarios to completion when their state is isolated and safe,
even if a sibling fails, so the phase collects useful results. Stop scenarios
that depend on corrupted, destructive, or unknown state. Never convert a
partial scenario set into a pass.

Do not improvise new acceptance scope during execution. If coverage is
missing, record the gap and return to the coordinator; adding or changing a
script or scenario changes the candidate/inventory fingerprint.

## Failure handling

Record the stage and whether product actions began. Use a narrow safe check to
classify the cause, then ask `tessara-sprint-validation` for invalidation scope.

- product or tracked harness correction: enter coordinator-owned convergence,
  then refreeze only after the final complete readiness/rehearsal passes
- shared environment/topology correction: rerun affected SIT/downstream work
  as directed by the coordinator
- scenario setup failure before actions: rerun that scenario after prerequisite
  reconfirmation when the coordinator permits it
- evidence finalization failure with intact raw results: rerun finalization
- assertion failure with no change: retain it, diagnose narrowly, and rerun the
  complete affected scenario set as directed

Never combine candidates or resume from an arbitrary failed step. Mark every
superseded attempt.

### Fail-late harvest after invalidation

On the first candidate-invalidating failure, retain it immediately, mark the
candidate invalid as directed by the coordinator, and do not create a passing
`uat-result.json`. Reconfirm prerequisites where necessary, then continue each
remaining safe independent inventory scenario as `diagnostic defect harvest`
with `authoritative: false`.

Do not run a scenario whose prerequisite failed, whose shared state is
corrupted or unknown, or whose security failure or unresolved product decision
makes the result unreliable. Record it as `blocked` with the exact dependency
reason; do not silently skip it. Preserve every executed result and report all
product, harness, fixture, environment, acceptance-inventory, and evidence
findings to the coordinator for `uat-defect-harvest.json` and the consolidated
defect batch. Restore the canonical environment when harvesting finishes.

### Focused repair assignment

Execute only the automated/manual UAT diagnostic scenarios in the
coordinator-authorized impact cone. Label every result `focused repair
validation` and `authoritative: false`; do not issue or reuse a candidate
fingerprint or `uat-result.json`. Run safe independent scenarios fail-late and
return new defects and blocked dependencies to the coordinator. Never reduce
the cone or authorize entry to final certification.

## Result boundary

`uat-result.json` states that UAT passed; it does not independently authorize
closeout. The coordinator verifies the full receipt chain, acceptance mapping,
failure chronology, manifest, and final topology before writing closeout
authorization.

## Finish criteria

Finish only when scripted UAT and every manual scenario pass for the exact SIT
candidate, evidence is complete and hashed, no defect or product decision is
open, the handoff topology is restored, and `uat-result.json` agrees with the
human verification record.

Diagnostic harvest or focused repair work finishes at its coordinator-defined
record boundary, not at this formal-UAT finish criterion.
