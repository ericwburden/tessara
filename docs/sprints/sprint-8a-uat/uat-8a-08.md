# UAT-8A-08 — Component-only upgrade and rollback

## 1. Test Script Summary

- System / Module: Tessara Components deployment lifecycle
- Requirement: Sprint 8A AC-16
- Environment: Frozen Sprint 8A UAT candidate
- User role: Operator and reviewer
- Scenario: Upgrade only Components, confirm state and behavior, roll back, and restore the intended release.
- Acceptance: Every switch is health-gated, Component state persists, unrelated service identities/restarts do not change, and final topology is healthy.

## 2. Before You Start

- Record Component and unrelated image digests, container identities, restart counts, instance, configuration, and a recognizable Component: ____________________
- Confirm immutable baseline, candidate, and intended-current image references.

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Upgrade only Components and wait for health. | The new Component release becomes healthy; recorded Component data/configuration remain. | | | |
| 2 | Exercise Component and Dashboard routes. | Product behavior and placements remain usable. | | | |
| 3 | Compare unrelated service identities and restart counts. | Core, gateway, Supervisor, Dashboard, Dataset compatibility, and other modules are unchanged. | | | |
| 4 | Roll back, verify health, then restore intended current release. | Both switches are health-gated; final state is intended current and healthy. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects

