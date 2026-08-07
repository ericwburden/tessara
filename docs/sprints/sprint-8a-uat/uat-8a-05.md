# UAT-8A-05 — Dashboard lifecycle and outages

## 1. Test Script Summary

- System / Module: Tessara Dashboards consuming Components
- Requirement: Sprint 8A AC-11 and AC-18
- Environment: Frozen Sprint 8A UAT candidate
- User role: Operator, Component manager, Dashboard manager, and reader
- Scenario: Use the three exact predecessor-bound action placements to exercise Dashboard findings, Defer/Upgrade/Replace/Remove, restricted nondisclosure, Component-provider outage, and recovery.
- Acceptance: Existing Dashboard policy remains intact; each action has an independent executable fixture; the blocked placement is never disclosed; outage is contained to the exact authorized placements; and recovery converges to zero findings.
- Machine contract: `docs/sprints/sprint-8a-uat/scenario-contract.json` entry `UAT-8A-05`; its acceptance criteria, semantic predicates, role and actor bindings, preconditions, starting state, ordered steps, and evidence IDs, kinds, cardinalities, and capture metadata are exact.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Tester identity: tester ID __________; display name __________.
- Actor bindings: `operator` account __________; `component-manager` account __________; `dashboard-manager` account __________; `dashboard-reader` account __________.
- Required precondition IDs: `candidate-fingerprint`, `environment-fingerprint`, `preflight-receipt`, `sit-result-receipt`, `evidence-root`, `execution-start`.
- Required starting-state IDs: `reference-dashboard`, `lifecycle-placement-identities`, `predecessor-successor-binding`, `unrelated-route`.
- Evidence folder and execution start time: ____________________
- Select `Reference Operations` and record the `lifecycle-upgrade`,
  `lifecycle-replace`, `lifecycle-remove`, and blocked-scope placement IDs:
  ____________________
- Confirm all three action placements reference the same superseded/inactive
  Stat Card predecessor and that it declares the published/active successor.
- Ensure an unrelated route is available for containment checks.
- Structured evidence: retain at least one raw artifact for every exact
  `dependency-semantic-receipt` assertion, then use
  `Publish-Sprint8AManualUatStructuredEvidence`. The canonical producer and
  wrapper bind the attempt, candidate, environment, Start checkpoint, and
  execution lease; every assertion must be `passed` and name the same producer
  receipt plus its retained raw evidence.

## 3. Test Steps

| Step | User action | Expected result | Required evidence ID(s) | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|---|
| 1 | Refresh dependency health before any action. | Exactly three `lifecycle_unrenderable` findings appear for the three action placements. The blocked-scope placement is absent, while the authorized upgrade finding discloses the declared successor. | `lifecycle-findings-trace` | | | |
| 2 | Defer the upgrade fixture, refresh it at the returned finding revision, then choose Upgrade. | Defer advances the finding revision and remains actionable; Upgrade resolves it by using the declared successor. | `defer-upgrade-trace` | | | |
| 3 | On the independent fixtures, Replace with the authorized current Table reference and Remove the remove placement. | Replace and Remove each resolve only its selected placement; the three action paths do not consume one another's fixture. | `replace-remove-trace` | | | |
| 4 | Stop only the Sprint 8A Component provider, refresh the Dashboard, and open the unrelated route. | Exactly five remaining authorized placements report `provider_unavailable`; the blocked placement remains absent and the unrelated route stays healthy. | `provider-outage-trace` | | | |
| 5 | Restore Components, wait for readiness, and refresh. | Dashboard health is `healthy`, open/deferred counts are zero, and no visible finding remains. | `provider-recovery-trace` | | | |
| 6 | Record the structured dependency-semantic receipt and restore the canonical fresh seed as directed by the coordinator. | The authenticated JSON receipt contains every exact check/action identity and requires canonical reset; restoration is recorded without converting this manual scenario into rehearsal evidence. | `dependency-semantic-receipt` | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted (any defect makes this scenario Not Accepted)
