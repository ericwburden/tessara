# UAT-6D-06 — Outage, Recovery, And Retained-Product Regression

## 1. Test Script Summary

- System / Module: Core, Module SDK Reference, Dashboard, and Scoped Records
- Requirement: Contained module outage with unchanged retained products
- Environment: `http://127.0.0.1:8080`
- User role: Administrator and deployment operator
- Business scenario: The reference module becomes unavailable while Core and
  other modules remain available, then returns without losing its state.
- Acceptance criteria: Core renders its branded `503` fallback, unrelated
  routes continue working, restart restores the reference and retained
  configuration, shutdown is bounded, and Dashboard/Scoped Records behavior
  remains intact.

## 2. Before You Start

Preconditions:

1. UAT-6D-04 passed with display label `SDK Reference UAT`.
2. The administrator is signed in.

Record actually tested:

- Reference container:
- Shutdown duration:
- Dashboard selected:
- Scoped Record selected:

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Stop the reference service and open `/reference/module-sdk`. | Core returns the branded authenticated `503` fallback with Home and Module Management recovery actions. |  |  |  |
| 2 | While it is stopped, open Core, Module Management, a Dashboard, and Scoped Records. | Unrelated routes remain available and their retained product behavior is unchanged. |  |  |  |
| 3 | Start the reference service and wait for readiness, then reload its route. | The normal reference document returns and the saved label is retained. |  |  |  |
| 4 | Stop the reference service normally and record elapsed time. | The process exits within the 30-second Compose grace period without corrupting state. |  |  |  |
| 5 | Start it again and rerun the canonical boundary/conformance commands. | The module is healthy; conformance passes; the sole Dashboard root-web finding remains assigned to Sprint 6E and has not expanded. |  |  |  |

## 4. Overall Test Result

- Overall result:
- Tester:
- Execution date:
- Defects: None /
- Comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects
