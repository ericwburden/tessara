---
name: tessara-uat
description: Execute and retain Tessara scripted and manual user acceptance testing for the exact candidate authorized by SIT. Use when running formal sprint UAT, exercising role, responsive, failure, recovery, upgrade, rollback, or business scenarios, collecting acceptance decisions, or producing the UAT receipt consumed by sprint validation and closeout.
---

# Tessara UAT

Run formal acceptance for the exact SIT-authorized candidate. Do not change
the acceptance inventory while executing it and do not authorize closeout
directly.

Before acting, read
[`../tessara-sprint-validation/references/validation-protocol.md`](../tessara-sprint-validation/references/validation-protocol.md)
completely.

## Prerequisites

Require parsed, passing:

- `preflight-result.json`
- `candidate.json`
- `sit-result.json`

Reject UAT if their candidate or environment fingerprints differ, any SIT lane
is incomplete, the UAT inventory changed, required accounts/fixtures are
missing, or the intended topology is not in its recorded starting state.

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

- product or tracked harness correction: refreeze and restart SIT
- shared environment/topology correction: rerun affected SIT/downstream work
  as directed by the coordinator
- scenario setup failure before actions: rerun that scenario after prerequisite
  reconfirmation when the coordinator permits it
- evidence finalization failure with intact raw results: rerun finalization
- assertion failure with no change: retain it, diagnose narrowly, and rerun the
  complete affected scenario set as directed

Never combine candidates or resume from an arbitrary failed step. Mark every
superseded attempt.

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
