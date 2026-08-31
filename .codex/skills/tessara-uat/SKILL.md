---
name: tessara-uat
description: Execute and retain Tessara scripted and manual user acceptance testing for the exact candidate authorized by SIT, including coordinator-directed fail-late diagnostic harvesting after invalidation and focused repair UAT checks. Use when running formal sprint UAT, exercising role, responsive, failure, recovery, upgrade, rollback, or business scenarios, collecting scenario dispositions or acceptance decisions, or producing UAT evidence consumed by sprint validation and closeout.
---

# Tessara UAT

Run formal acceptance for the exact SIT-authorized candidate. Do not change
the acceptance inventory while executing it and do not authorize closeout
directly.

Read
[`../tessara-sprint-validation/references/uat-scenario-classification.md`](../tessara-sprint-validation/references/uat-scenario-classification.md)
completely before reviewing or executing the scenario inventory.

Inspect the tracked sprint validation contract before loading a protocol. When
it declares `policy_version: tessara-validation-v2`, read
[`../tessara-sprint-validation/references/validation-policy-v2.md`](../tessara-sprint-validation/references/validation-policy-v2.md)
completely. Authenticate compact prerequisite certificates and publish UAT
through one phase-local evidence index plus a compact UAT certificate.

When it declares `policy_version: tessara-validation-v3`, read v2 and then
[`../tessara-sprint-validation/references/validation-policy-v3.md`](../tessara-sprint-validation/references/validation-policy-v3.md)
completely. Validate the successor-impact plan and immediate predecessor.
Execute every `execute` scripted/manual scenario through its platform adapter
lane; use the shared certifier for `inherit` items. A manual adapter action may
validate a human scenario receipt, but it cannot own phase lifecycle or
publication.

Only when the contract selects neither v2 nor v3, read
[`../tessara-sprint-validation/references/validation-protocol.md`](../tessara-sprint-validation/references/validation-protocol.md)
completely.

When a candidate-invalidating failure occurs or the coordinator assigns a
focused repair portion, also read
[`../tessara-sprint-validation/references/post-sit-defect-convergence.md`](../tessara-sprint-validation/references/post-sit-defect-convergence.md)
completely.

On any failed UAT script or scenario, also read
[`../tessara-sprint-validation/references/defect-provenance.md`](../tessara-sprint-validation/references/defect-provenance.md)
and emit a schema-valid `defect-provenance.json` beside the failed attempt
before correction or another broad scenario run.

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

Under v2, every scripted and manual UAT scenario executes for the exact
SIT-authorized candidate. A successor candidate cannot inherit UAT acceptance
from its predecessor.

Under v3, `uat-result.json` contains the exact complete scenario inventory with
an `executed` or `inherited_nonimpact` basis. Inherited scenarios retain the
immediate predecessor receipt and unchanged dependency/compatibility closure,
state explicit non-impact rationale, and have no successor timestamps. Missing
hashes, changed fixtures/environment/acceptance semantics, unknown paths, open
defects, or expectation ambiguity require complete UAT.

## Automated versus human boundary

Do not treat inspection of machine-readable evidence as human UAT. A scenario
whose pass/fail decision consists of parsing JSON, confirming exact fields,
counts, hashes, identities, release sequences, source absence, isolation
matrices, topology cleanup, API results, or other deterministic state must be a
scripted scenario or an earlier automated target/lane. The same applies to
deterministic browser behavior that a stable browser test can establish.

Manual UAT must exercise the actual product surface and state an irreducible
human acceptance question. For mixed coverage, require automation to establish
fixtures, provenance, and exact state first; the human receipt references that
passing automated evidence and records only the direct interaction and human
judgment. If no human question remains, manual coverage is not applicable.

If the frozen inventory labels an artifact-review checklist as manual UAT,
block execution and return the classification gap to the coordinator. Do not
ask a person to open a JSON file merely to manufacture manual evidence.

## Required execution order

1. Reconfirm candidate provenance, SIT authorization, active slot, health,
   fixtures, roles/accounts, browser configuration, and evidence paths.
2. Audit the scripted/manual classification, then hash the frozen inventory.
   Block rather than execute a misclassified machine-decidable manual scenario.
3. Under legacy/v2, run the contract's UAT command. Under v3, invoke each
   impact-plan `execute` scripted scenario through
   `Invoke-TessaraValidationLane`; the named script may be its adapter action.
4. Run every legacy/v2 manual scenario, or each v3 manual scenario marked
   `execute`; authenticate the remaining v3 inventory as inheritance.
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

For a manual scenario, also record the product surface, direct human
interaction, irreducible judgment question, and prerequisite automated receipt.
For a scripted scenario, retain the deterministic oracle and its declared
machine-readable inputs.

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
The defect-provenance record must compare the exact candidate behavior with
the frozen fixture, environment, harness, and acceptance contract and must
route ambiguity to a product decision rather than changing the expected
result.

- product or tracked harness correction: enter coordinator-owned convergence,
  then refreeze only after the required legacy complete pass, v2 affected-lane
  pre-freeze recertification, or v3 focused-repair/batch convergence and
  successor-impact plan passes
- shared environment/topology correction: rerun affected SIT/downstream work
  as directed by the coordinator
- scenario setup failure before actions: rerun that scenario after prerequisite
  reconfirmation when the coordinator permits it
- evidence finalization failure with intact raw results: rerun finalization
- assertion failure with no change: retain it, diagnose narrowly, and rerun the
  complete affected scenario set as directed

Never manually combine candidates or resume from an arbitrary failed step. V3
cross-candidate coverage is valid only when the shared certifier authenticates
the immediate predecessor and impact plan. Mark every superseded attempt.

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

For v2, routine verification consumes compact certificates and
`evidence-chain.json`. Raw UAT artifacts remain cold after the UAT phase index
is sealed; closeout performs the one required full integrity audit.

## Finish criteria

Finish only when scripted UAT and every manual scenario pass for the exact SIT
candidate as successor-executed coverage or v3-authenticated non-impact
inheritance, evidence is complete and hashed, no defect or product decision is
open, every UAT provenance record is verified or validly superseded, the
handoff topology is restored, and `uat-result.json` agrees with the human
verification record. No passing manual receipt may consist solely of reviewing
machine-readable evidence.

Diagnostic harvest or focused repair work finishes at its coordinator-defined
record boundary, not at this formal-UAT finish criterion.
