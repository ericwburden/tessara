# UAT-6E-01 — Use Dashboard Routes Normally

## 1. Test Script Summary

- System / Module: Tessara Dashboards
- Requirement: Dashboard-owned same-origin documents
- Test environment: `http://127.0.0.1:8080`, Sprint 6E baseline `2.0.0`
- User role: Administrator / Dashboard author
- Business scenario: An administrator uses normal Tessara navigation to find,
  inspect, edit, and view an existing Dashboard after its UI is extracted from
  Core.
- Acceptance criteria: All five Dashboard route types open inside a coherent
  Tessara document with the expected Dashboard content and no unavailable
  fallback.

## 2. Before You Start

Preconditions:

1. The Sprint 6E stack is running on Dashboard `2.0.0`.
2. Sign in as `admin@tessara.local`.
3. At least one representative Dashboard exists.

Record Selection Criteria:

1. Select an existing Dashboard with at least one placement.
2. Prefer a Dashboard containing more than one Component presentation type.

Record Actually Tested:

- Dashboard name:
- Starting placement count:
- Browser/version:

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Sign in and select **Dashboards** from primary navigation. | The Dashboard directory opens and lists authorized Dashboards. |  |  |  |
| 2 | Open a representative Dashboard. | The detail page shows its name, description, visibility, placement count, and available actions. |  |  |  |
| 3 | Select **Edit**. | The editor opens with the saved placement layout and Dashboard metadata. |  |  |  |
| 4 | Return to detail, then select **View**. | Viewer mode opens and renders the Dashboard placements without authoring controls. |  |  |  |
| 5 | Open `/dashboards/new` directly. | The Dashboard creation document opens with authorized visibility choices. |  |  |  |
| 6 | Return to `/dashboards`. | Navigation remains coherent and the directory is still usable. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester:
- Test execution date:
- Defect IDs: None /
- Tester comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects

