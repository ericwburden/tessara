# UAT-6E-05 — Contain Lifecycle And Provider Outages

## 1. Test Script Summary

- System / Module: Tessara Dashboards and Module Management
- Requirement: Core fallback ownership and provider degradation containment
- Test environment: `http://127.0.0.1:8080`, Dashboard `2.0.0`
- User role: Administrator / Dashboard author
- Business scenario: An administrator disables or temporarily disrupts a
  Dashboard dependency and confirms users receive contained states while Core
  and unrelated modules continue to operate.
- Acceptance criteria: Dashboard lifecycle or Components-provider failures do
  not interrupt Core, Scoped Records, or unrelated routes; restoring service
  restores normal Dashboard behavior without data loss.

## 2. Before You Start

Preconditions:

1. UAT-6E-02 has produced or identified a Dashboard with a Component placement.
2. The tester can use Module Management and the local deployment controls.
3. Record Dashboard metadata and placement count before disruption.

Record Actually Tested:

- Dashboard name/ID:
- Starting placement count:
- Unrelated Core route:
- Unrelated module route:

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Disable the Dashboard product route through Module Management. | Dashboard navigation is removed or disabled and direct access receives the Core-owned unavailable/disabled state. |  |  |  |
| 2 | While Dashboard is disabled, use a Core administration route and Scoped Records. | Both remain available and usable. |  |  |  |
| 3 | Re-enable Dashboard. | Navigation and direct Dashboard access recover without data or configuration loss. |  |  |  |
| 4 | Temporarily make the Components provider unavailable and reload editor/viewer. | The Dashboard remains available; affected placements show the approved contained degraded state. |  |  |  |
| 5 | Use an unaffected Dashboard action and an unrelated route during the provider outage. | Unaffected work and unrelated routes remain usable. |  |  |  |
| 6 | Restore the Components provider and reload. | Compatible placement metadata/rendering recover without recreating the Dashboard. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester:
- Test execution date:
- Defect IDs: None /
- Tester comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects

