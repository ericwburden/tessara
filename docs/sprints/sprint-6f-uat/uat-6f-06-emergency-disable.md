# UAT-6F-06 — Emergency-Disable and Restore a Module

## 1. Test Script Summary

- System / Module: Tessara Application Composition emergency override
- Enhancement / Requirement: Constrained, audited, optionally expiring disable
- Test environment: Healthy Sprint 6F UAT reference installation
- User role: Authorized Tessara administrator
- Business scenario: An administrator immediately disables one unhealthy or
  unsafe module without deleting its data. The override remains visible as
  drift until the administrator adopts or reconciles it.
- Acceptance criteria: Only the selected module becomes unavailable, the
  reason and expiry are auditable, unrelated experiences remain usable, and
  restoration does not lose module data.

## 2. Before You Start

Preconditions:

1. Dashboard and Scoped Records are enabled and contain their reference data.
2. Use **Application Composition → Emergency module disable**. Enter a module
   definition ID and reason; the candidate creates a one-hour override.
3. Use `tessara.reference.scoped-records`. Restore it with **Restore desired**
   on its enablement drift finding, which reapplies the separately approved
   Blueprint and reconciles the Supervisor override.
4. Record the selected module's starting receipt, data, and enabled state.

Record Selection Criteria:

1. Select one enabled reference module with retained non-destructive data.
2. Do not select Core or gateway.

Record Actually Tested:

- Module:
- Starting enabled state:
- Starting record/dashboard used to verify retention:
- Starting receipt revision:

Input Values to Use During Test:

1. Reason: `Sprint 6F UAT emergency-disable verification`
2. Expiry: shortest candidate-supported future expiry

Tester Instructions: Do not use any destructive option and do not manually
stop the module container.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Open **Application Composition**, choose the approved module, and select **Emergency disable**. | A constrained confirmation requests a reason and supported expiry; no data-removal effect is offered. |  |  |  |
| 2 | Enter the supplied reason and expiry, then confirm. | Supervisor records an authorized disable override and only the selected module becomes disabled. |  |  |  |
| 3 | Use primary navigation and open unrelated Core/module experiences. | Unrelated experiences remain healthy; the selected module has a clear contained unavailable state. |  |  |  |
| 4 | Reopen **Application Composition**. | The override, reason, expiry, desired-versus-observed drift, and available adopt/reconcile choices are visible. |  |  |  |
| 5 | Select **Restore desired** on the enablement drift finding. | The approved Blueprint is reapplied, the selected module returns to enabled, healthy state, and the override is reconciled. Expiry alone ends the authorization window but intentionally leaves the disable as visible drift. |  |  |  |
| 6 | Reopen the selected module and inspect its starting record or Dashboard. | Previously retained data remains available and no duplicate starter content was created. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester name:
- Test execution date: YYYY-MM-DD
- Defect IDs: None /
- Tester comments:
- Business acceptance decision: Accepted / Not Accepted / Accepted with Defect(s)
- Business owner or reviewer:
- Review date: YYYY-MM-DD
