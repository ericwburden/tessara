# UAT-8A-02 — From-empty seed and canonical references

## 1. Test Script Summary

- System / Module: Tessara reference application
- Requirement: Sprint 8A AC-03 through AC-05
- Environment: Disposable Sprint 8A validation topology
- User role: Operator and Dashboard reader
- Scenario: Build the reference application from empty owner databases and repeat the unchanged bootstrap.
- Acceptance: The recognizable seed is complete, Dashboard uses only the selected Component Module Instance/v3 references, and the second run is an exact no-op.

## 2. Before You Start

- Confirm explicit authorization for the exact `tessara-sprint-8a` disposable project.
- Record the resolved project, databases, and volumes: ____________________
- Ensure the candidate source identity is recorded.

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Run the approved from-empty materialization. | Owners initialize and seed in declared order; the topology becomes healthy. | | | |
| 2 | Open Components and a seeded Dashboard. | All supported Components are recognizable and Dashboard placements render. | | | |
| 3 | Inspect module inventory and navigation. | Dashboard appears once through its real Release/Instance; Core has exactly Forms, Workflows, Responses, Datasets, and Migration transitions. | | | |
| 4 | Repeat bootstrap unchanged. | The receipt reports an exact semantic no-op with no duplicate rows, revisions, references, roles, or configuration. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects

