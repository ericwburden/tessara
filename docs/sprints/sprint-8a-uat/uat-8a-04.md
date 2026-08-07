# UAT-8A-04 — Dataset contract, joint scope, and outage

## 1. Test Script Summary

- System / Module: Tessara Dashboard, Components, and Dataset compatibility provider
- Requirement: Sprint 8A AC-09, AC-10, AC-18, and AC-19
- Environment: Frozen Sprint 8A UAT candidate
- User role: Operator, Component manager, dual-scope administrator, and restricted actors
- Scenario: Author and execute a Component through a typed Dataset reference, prove Dashboard and Component/Dataset scopes agree on one governing node, then observe restricted and unavailable provider states.
- Acceptance: Allowed and shared-node work succeeds; a Dashboard on scope A cannot disclose or render a Component/Dataset on disjoint scope B even for an actor authorized on both; outage is read-only without pending writes; and retry recovers with unsaved browser input retained.
- Machine contract: `docs/sprints/sprint-8a-uat/scenario-contract.json` entry `UAT-8A-04`; its acceptance criteria, semantic predicates, role and actor bindings, preconditions, starting state, ordered steps, and evidence IDs, kinds, cardinalities, and capture metadata are exact.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Tester identity: tester ID __________; display name __________.
- Actor bindings: `operator` account __________; `component-manager` account __________; `dual-scope-administrator` account __________; `out-of-scope-actor` account __________; `wrong-role-actor` account __________.
- Required precondition IDs: `candidate-fingerprint`, `environment-fingerprint`, `preflight-receipt`, `sit-result-receipt`, `evidence-root`, `execution-start`.
- Required starting-state IDs: `allowed-dataset`, `out-of-scope-dataset`, `mixed-dashboard`, `unsaved-component-draft`.
- Evidence folder and execution start time: ____________________
- Select an allowed Dataset and an out-of-scope Dataset; record their displayed names: ____________________
- Identify the prepared mixed Dashboard with its shared-node placement on scope A and disjoint Component/Dataset placement on scope B: ____________________
- Open a disposable Component draft with an unsaved visible change.

## 3. Test Steps

| Step | User action | Expected result | Required evidence ID(s) | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|---|
| 1 | Preview, save, publish, and execute using the allowed Dataset. | Operations succeed through the canonical typed reference. | `typed-dataset-execution-trace` | | | |
| 2 | As an administrator or another actor authorized on both scopes, open the mixed Dashboard. Render the shared scope-A placement, inspect the disjoint scope-B tile, and try its render action. | The shared placement discloses its exact Component identity and renders normally. The disjoint placement is generic and unavailable, discloses no Component/Dataset identity, scope, title, or data, and its render is denied. | `joint-scope-render-trace` | | | |
| 3 | Try the out-of-scope record and wrong-role account, then compare a known blocked direct identity with a generated random identity. | Access is denied without revealing whether a restricted resource exists; the known and random public outcomes are equivalent. | `known-random-nondisclosure-record` | | | |
| 4 | Stop or timeout the Dataset provider, then retry preview/save. | Existing metadata remains readable; dependent mutations do not persist; unsaved input remains. | `dataset-outage-trace` | | | |
| 5 | Restore the provider and retry. | Readiness returns and the valid operation succeeds once. | `dataset-recovery-trace` | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted (any defect makes this scenario Not Accepted)
