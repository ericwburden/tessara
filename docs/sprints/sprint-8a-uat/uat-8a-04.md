# UAT-8A-04 — Dataset contract, joint scope, and outage

## 1. Test Script Summary

- System / Module: Tessara Dashboard, Components, and Dataset compatibility provider
- Requirement: Sprint 8A AC-09 and AC-10
- Environment: Frozen Sprint 8A UAT candidate
- User role: Component manager plus scoped and out-of-scope actors
- Scenario: Author and execute a Component through a typed Dataset reference, prove Dashboard and Component/Dataset scopes agree on one governing node, then observe restricted and unavailable provider states.
- Acceptance: Allowed and shared-node work succeeds; a Dashboard on scope A cannot disclose or render a Component/Dataset on disjoint scope B even for an actor authorized on both; outage is read-only without pending writes; and retry recovers with unsaved browser input retained.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Evidence folder and execution start time: ____________________
- Select an allowed Dataset and an out-of-scope Dataset; record their displayed names: ____________________
- Identify the prepared mixed Dashboard with its shared-node placement on scope A and disjoint Component/Dataset placement on scope B: ____________________
- Open a disposable Component draft with an unsaved visible change.

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Preview, save, publish, and execute using the allowed Dataset. | Operations succeed through the canonical typed reference. | | | |
| 2 | As an administrator or another actor authorized on both scopes, open the mixed Dashboard. Render the shared scope-A placement, inspect the disjoint scope-B tile, and try its render action. | The shared placement discloses its exact Component identity and renders normally. The disjoint placement is generic and unavailable, discloses no Component/Dataset identity, scope, title, or data, and its render is denied. | | | |
| 3 | Try the out-of-scope record and wrong-role account, then compare a known blocked direct identity with a generated random identity. | Access is denied without revealing whether a restricted resource exists; the known and random public outcomes are equivalent. | | | |
| 4 | Stop or timeout the Dataset provider, then retry preview/save. | Existing metadata remains readable; dependent mutations do not persist; unsaved input remains. | | | |
| 5 | Restore the provider and retry. | Readiness returns and the valid operation succeeds once. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects
