# UAT-6F-03 — Keep Planning, Approval, and Restricted Access Separate

## 1. Test Script Summary

- System / Module: Tessara Application Composition authorization
- Enhancement / Requirement: Separate read, plan, approval, and apply authority
- Test environment: Healthy Sprint 6F UAT installation
- User role: Read-only planner, authorized approver, and constrained non-admin
- Business scenario: Different users inspect, plan, approve, and apply a
  composition according to their assigned authority. A plan remains inert
  until an authorized approval is explicitly submitted.
- Acceptance criteria: Read-only and constrained users cannot approve or apply;
  an approver sees and approves exact effects; no role exposes secrets or
  restricted module details.

## 2. Before You Start

Preconditions:

1. A healthy installation has a draft Blueprint that differs safely from the
   current receipt and does not remove data.
2. Before UAT, use Roles & Access to create `UAT Composition Reader`
   (`composition:read`), `UAT Composition Planner` (`composition:read` and
   `composition:plan`), and `UAT Constrained User` (no composition capability),
   then create one temporary account for each role. The coordinator supplies
   one-time passwords out of band. Use `admin@tessara.local` as the approver.
3. Record the starting receipt revision and plan digest.

Record Selection Criteria:

1. Use a non-secret, reversible configuration or navigation change.
2. Do not use a change that requires destructive authority.

Record Actually Tested:

- Blueprint revision:
- Starting receipt revision:
- Planner account:
- Approver account:
- Constrained account:

Input Values to Use During Test:

1. Approval reason: `Sprint 6F UAT approval separation`

Tester Instructions: Sign out completely between roles and record any action
that remains visible but is rejected when selected.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Sign in as the read-only composition user and open **Application Composition**. | Permitted composition state is visible; planning, approval, and apply actions are unavailable or denied. |  |  |  |
| 2 | Sign in as the planner and resolve the prepared draft. | A deterministic plan and exact effects are shown, but the plan remains pending and cannot be applied by this user. |  |  |  |
| 3 | Sign in as the constrained non-admin and revisit the page. | Only authorized state is disclosed; approval, apply, secrets, and restricted module/resource details are unavailable. |  |  |  |
| 4 | Sign in as the approver, inspect the plan, and select **Approve current plan**. | The approval binds the current Blueprint revision, lockfile, plan, and exact effects separately from planning. |  |  |  |
| 5 | Select **Apply approved plan**. | Supervisor accepts a short-lived authorization derived from that exact approval and completes the operation. |  |  |  |
| 6 | Reload the composition page and record the receipt. | Observed state advances to the approved plan; no secret values appear in the UI, response, or retained receipt. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester name:
- Test execution date: YYYY-MM-DD
- Defect IDs: None /
- Tester comments:
- Business acceptance decision: Accepted / Not Accepted / Accepted with Defect(s)
- Business owner or reviewer:
- Review date: YYYY-MM-DD
