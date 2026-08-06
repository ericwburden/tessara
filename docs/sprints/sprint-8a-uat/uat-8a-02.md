# UAT-8A-02 — From-empty seed and canonical references

## 1. Test Script Summary

- System / Module: Tessara reference application
- Requirement: Sprint 8A AC-03, AC-04, AC-05, and AC-16
- Environment: Frozen Sprint 8A UAT candidate on the coordinator-authorized disposable topology
- User role: Operator and Dashboard reader
- Scenario: Build the reference application from empty owner databases and repeat the unchanged bootstrap.
- Acceptance: The recognizable seed has the exact seven-shell/eight-version/seven-placement topology, Dashboard uses only the selected Component Module Instance/v3 references, and the second run is an exact no-op.

## 2. Before You Start

- Coordinator bindings: candidate fingerprint __________; environment fingerprint __________; preflight receipt SHA-256 __________; SIT result receipt SHA-256 __________.
- Evidence folder and execution start time: ____________________
- Confirm explicit authorization for the exact `tessara-sprint-8a` disposable project.
- Record the resolved project, databases, and volumes: ____________________
- Ensure the candidate source identity is recorded.

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Run the approved from-empty materialization. | Owners initialize and seed in declared order; the topology becomes healthy. | | | |
| 2 | Inspect owner read-back and open Components plus the seeded Dashboard. | Exactly seven Component shells and eight versions exist: the Stat Card `1.0.0` predecessor is superseded/inactive and names the published/active `2.0.0` successor; the other six versions are published/active. | | | |
| 3 | Inspect the seeded Dashboard placements and their resolved references. | Exactly seven placements exist: four normal/blocked-scope fixtures plus independent `lifecycle-upgrade`, `lifecycle-replace`, and `lifecycle-remove` placements bound to the inactive predecessor; every reference is selected-instance Components v3. | | | |
| 4 | Inspect module inventory and navigation. | Dashboard appears once through its real Release/Instance; Core has exactly Forms, Workflows, Responses, Datasets, and Migration transitions. | | | |
| 5 | Repeat bootstrap unchanged. | The receipt reports an exact semantic no-op with no duplicate rows, revisions, references, roles, or configuration. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Evidence paths and execution end time: ____________________
- Cleanup/restoration result: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects
