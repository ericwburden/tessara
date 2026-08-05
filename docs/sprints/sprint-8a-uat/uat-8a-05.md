# UAT-8A-05 — Dashboard lifecycle and outages

## 1. Test Script Summary

- System / Module: Tessara Dashboards consuming Components
- Requirement: Sprint 8A AC-11 and AC-12
- Environment: Frozen Sprint 8A UAT candidate
- User role: Component manager, Dashboard manager, and reader
- Scenario: Change a placed Component revision/lifecycle and exercise Dashboard findings, actions, provider outage, and recovery.
- Acceptance: Existing Dashboard policy remains intact, degradation is contained, and recovery converges.

## 2. Before You Start

- Select a seeded Dashboard with a Component placement; record names/status: ____________________
- Ensure an unrelated route is available for containment checks.

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Publish a successor Component revision and refresh the Dashboard. | A clear finding appears and supported defer/Upgrade/Replace/Remove actions behave as before. | | | |
| 2 | Stop Components and reload Dashboard plus an unrelated route. | Placement/dependency state degrades coherently; unrelated route remains healthy. | | | |
| 3 | Restore Components and refresh. | Dashboard converges without duplicate findings or disclosure. | | | |
| 4 | Repeat the outage/recovery observation for the Dataset provider. | Downstream state is coherent and final health returns. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects

