# UAT-6F-05 — Adopt and Reconcile Composition Drift

## 1. Test Script Summary

- System / Module: Tessara Application Composition drift management
- Enhancement / Requirement: Detect, adopt, and reconcile deliberate UI change
- Test environment: Healthy lockfile-matched Sprint 6F UAT installation
- User role: Tessara administrator with composition approval authority
- Business scenario: An administrator changes a non-secret setting outside the
  active Blueprint. Tessara identifies the difference and lets the
  administrator either adopt it into desired state or reconcile it back.
- Acceptance criteria: Drift is explicit, each disposition is authorized and
  observable, adoption updates desired state, and reconciliation restores the
  approved value without hiding or silently accepting the difference.

## 2. Before You Start

Preconditions:

1. Desired and observed composition state currently match.
2. Select one reversible module configuration or navigation setting.
3. Change **Display label** on a Module Management configuration page, then
   open **Application Composition → Drift**. Refresh once to run live owner
   read-back; each finding shows desired/observed JSON plus **Adopt as new
   draft** and **Restore desired**.

Record Selection Criteria:

1. Choose a non-secret display or navigation setting visible to an administrator.
2. Record its desired and observed starting value.

Record Actually Tested:

- Setting/path:
- Starting desired value:
- Starting observed value:
- Starting Blueprint/receipt revision:

Input Values to Use During Test:

1. First changed value for adoption: `Sprint 6F UAT adopted value`
2. Second changed value for reconciliation: `Sprint 6F UAT reconcile value`

Tester Instructions: Capture the drift view before selecting each disposition.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Change the selected setting to the adoption value through its normal administration UI. | The business setting changes, and Application Composition reports desired-versus-observed drift. |  |  |  |
| 2 | Open the drift detail and record its path and values. | The finding clearly distinguishes desired and observed values without exposing secrets. |  |  |  |
| 3 | Select **Adopt** and confirm the authorized action. | The finding is marked adopted and the changed value becomes part of a new desired revision. |  |  |  |
| 4 | Change the same setting to the reconciliation value. | A new open drift finding appears against the newly adopted desired value. |  |  |  |
| 5 | Select **Reconcile** and approve the resulting plan if prompted. | Supervisor restores the adopted desired value through a new authorized operation. |  |  |  |
| 6 | Reload the business setting and composition page. | Desired and observed values match again, and both drift dispositions remain auditable. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester name:
- Test execution date: YYYY-MM-DD
- Defect IDs: None /
- Tester comments:
- Business acceptance decision: Accepted / Not Accepted / Accepted with Defect(s)
- Business owner or reviewer:
- Review date: YYYY-MM-DD
