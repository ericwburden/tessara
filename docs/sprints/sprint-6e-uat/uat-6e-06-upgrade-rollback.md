# UAT-6E-06 — Upgrade And Roll Back Only Dashboard

## 1. Test Script Summary

- System / Module: Tessara Dashboard deployment
- Requirement: Health-gated Dashboard-only upgrade and rollback
- Test environment: Sprint 6E Compose profile
- User role: Release operator and administrator
- Business scenario: A release operator activates Dashboard `2.0.1`, verifies
  normal product behavior, and restores `2.0.0` without replacing unrelated
  services or losing Dashboard state.
- Acceptance criteria: Release identity changes only for Dashboard, the route
  switches only to a healthy candidate, rollback restores baseline identity,
  and saved data remains unchanged.

## 2. Before You Start

Preconditions:

1. Dashboard `2.0.0` is healthy and active.
2. Candidate image `2.0.1` is available but not active.
3. Record image IDs, container IDs, restart counts, receipt revision, Dashboard
   configuration, and representative data before switching.

Record Actually Tested:

- Baseline image/container:
- Candidate image/container:
- Receipt revision:
- Dashboard record/data checkpoint:

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Attempt to switch to a candidate that is not healthy. | The switch is refused and the active baseline route is unchanged. |  |  |  |
| 2 | Start `2.0.1`, wait for health, and activate the candidate slot. | Normal Dashboard routes serve `2.0.1`; diagnostics, metadata, assets, and image labels agree. |  |  |  |
| 3 | Open the representative Dashboard and perform a read plus a reversible save. | Product behavior remains usable and the saved value persists. |  |  |  |
| 4 | Compare unrelated service image IDs, container IDs, and restart counts with the checkpoint. | Core, gateway, installation control, Scoped Records, reference SDK, and PostgreSQL are unchanged. |  |  |  |
| 5 | Activate the baseline slot and stop the candidate. | Normal routes return to `2.0.0`; candidate is no longer serving. |  |  |  |
| 6 | Reload the representative Dashboard and compare configuration, data, migrations, identity, and receipt revision. | All retained state matches the expected post-save values and no unrelated state regressed. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester:
- Test execution date:
- Defect IDs: None /
- Tester comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects
