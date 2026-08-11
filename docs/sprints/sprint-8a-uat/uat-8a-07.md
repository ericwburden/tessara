# UAT-8A-07 — Failed materialization and clean successor

## 1. Test Script Summary

- System / Module: Sprint 8A deployment workflow
- Requirement: Sprint 8A AC-13
- Environment: Frozen Sprint 8A UAT candidate on the coordinator-authorized disposable topology
- User role: Operator
- Scenario: Induce a bounded owner-bootstrap failure, review retained evidence, and create a new healthy attempt from empty.
- Acceptance: Raw failure evidence is retained before exact teardown; no partial topology remains; the successor starts empty and becomes healthy.
- Machine contract: `docs/sprints/sprint-8a-uat/scenario-contract.json` entry `UAT-8A-07`; its acceptance criteria, semantic predicates, role and actor bindings, preconditions, starting state, ordered steps, and evidence IDs, kinds, cardinalities, and capture metadata are exact.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Tester identity: tester ID __________; display name __________.
- Actor binding: `operator` account __________.
- Required precondition IDs: `candidate-fingerprint`, `environment-fingerprint`, `preflight-receipt`, `sit-result-receipt`, `evidence-root`, `execution-start`.
- Required starting-state IDs: `disposable-project-authorization`, `bounded-fault`, `expected-failing-prerequisite`.
- Evidence folder and execution start time: ____________________
- Confirm the approved bounded fault and exact disposable project authorization.
- Record the fault and expected failing prerequisite: ____________________

## 3. Test Steps

| Step | User action | Expected result | Required evidence ID(s) | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|---|
| 1 | Run materialization with the approved fault. | The attempt fails at the intended point and retains raw apply response and service logs. | `bounded-failure-record`, `failure-containment-receipt` | | | |
| 2 | Inspect containers, named volumes, and networks after teardown. | No resource belonging to `tessara-sprint-8a` remains. | `exact-teardown-record` | | | |
| 3 | Remove the fault and run a complete new materialization. | A new empty attempt reaches canonical health; no partial state is reused. | `empty-successor-record` | | | |
| 4 | Repeat bootstrap unchanged. | The successor reports an exact semantic no-op. | `successor-no-op-record` | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted (any defect makes this scenario Not Accepted)
