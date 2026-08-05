# UAT-8A-07 — Failed materialization and clean successor

## 1. Test Script Summary

- System / Module: Sprint 8A deployment workflow
- Requirement: Sprint 8A AC-15
- Environment: Disposable Sprint 8A validation topology
- User role: Operator
- Scenario: Induce a bounded owner-bootstrap failure, review retained evidence, and create a new healthy attempt from empty.
- Acceptance: Raw failure evidence is retained before exact teardown; no partial topology remains; the successor starts empty and becomes healthy.

## 2. Before You Start

- Confirm the approved bounded fault and exact disposable project authorization.
- Record the fault and expected failing prerequisite: ____________________

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Run materialization with the approved fault. | The attempt fails at the intended point and retains raw apply response and service logs. | | | |
| 2 | Inspect containers, named volumes, and networks after teardown. | No resource belonging to `tessara-sprint-8a` remains. | | | |
| 3 | Remove the fault and run a complete new materialization. | A new empty attempt reaches canonical health; no partial state is reused. | | | |
| 4 | Repeat bootstrap unchanged. | The successor reports an exact semantic no-op. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects

