# UAT-6E-02 — Create, Edit, View, And Retain A Dashboard

## 1. Test Script Summary

- System / Module: Tessara Dashboards
- Requirement: Product behavior and persistence preservation
- Test environment: `http://127.0.0.1:8080`, Sprint 6E baseline `2.0.0`
- User role: Administrator / Dashboard author
- Business scenario: An administrator creates a disposable Dashboard, updates
  its metadata and composition, and confirms the saved result remains visible
  after navigation and reload.
- Acceptance criteria: Saved Dashboard values and placements survive reload and
  can be viewed and deleted through the normal product flow.

## 2. Before You Start

Preconditions:

1. UAT-6E-01 passed.
2. The administrator can create and manage Dashboards.
3. At least one compatible Component version is available to place.

Record Selection Criteria:

1. Create a new disposable record; do not alter a business-owned Dashboard.

Record Actually Tested:

- Dashboard ID:
- Visibility selection:
- Component version placed:

Input Values to Use During Test:

1. Name: `Sprint 6E Manual UAT Dashboard`
2. Description: `Disposable Dashboard for Sprint 6E acceptance testing.`
3. Updated name: `Sprint 6E Manual UAT Dashboard — Saved`

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Create a Dashboard with the supplied name, description, and an authorized visibility node. | The Dashboard is created once and its detail or editor route opens. |  |  |  |
| 2 | Add one compatible Component placement and save. | The placement appears in the editor with valid grid geometry and a successful save result. |  |  |  |
| 3 | Change the Dashboard name to the supplied updated name and save. | The updated name appears without creating a duplicate Dashboard. |  |  |  |
| 4 | Reload the editor. | The updated metadata and placement remain present. |  |  |  |
| 5 | Open viewer mode. | The saved placement renders or shows its approved contained degraded state. |  |  |  |
| 6 | Return to the directory. | The updated Dashboard name appears once. |  |  |  |
| 7 | Delete the disposable Dashboard and confirm. | It is removed from the directory and a later direct load does not disclose retained details. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester:
- Test execution date:
- Defect IDs: None /
- Tester comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects

