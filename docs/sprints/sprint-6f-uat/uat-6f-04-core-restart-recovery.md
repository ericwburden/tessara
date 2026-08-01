# UAT-6F-04 — Recover Composition Status After a Core Restart

## 1. Test Script Summary

- System / Module: Tessara Supervisor and Core read-back
- Enhancement / Requirement: Supervisor-owned apply survives Core restart
- Test environment: Healthy Sprint 6F UAT installation
- User role: Authorized deployment operator and Tessara administrator
- Business scenario: An authorized composition change is applying when Core is
  restarted. Supervisor retains ownership and the administrator later sees one
  final result in the recovered Core UI.
- Acceptance criteria: The operation completes exactly once, product bootstrap
  is not duplicated, and recovered Core reads the same final receipt while
  unrelated services remain usable.

## 2. Before You Start

Preconditions:

1. Capture the starting receipt, owner bootstrap counts, and container identities.
2. Prepare an approved, reversible composition change.
3. **NEEDS INFO:** Provide a candidate-supported apply scenario or fault hook
   that remains active long enough to restart Core before completion.
4. The operator is authorized to restart only the Core container.

Record Selection Criteria:

1. Use a change with an observable final configuration or enablement result.
2. Do not use destructive data removal.

Record Actually Tested:

- Starting receipt revision/digest:
- Operation ID:
- Core container identity before restart:
- Owner bootstrap counts before apply:

Input Values to Use During Test:

1. Restart target: `tessara-sprint-6f-core-1`

Tester Instructions: Coordinate the apply and restart timestamps and do not
restart Supervisor or its ledger.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Start the supplied approved composition apply and record the operation ID. | Supervisor shows an accepted or running operation owned outside Core. |  |  |  |
| 2 | While the operation is active, restart only Core using the supplied candidate command. | Core becomes temporarily unavailable; Supervisor and unrelated services remain running. |  |  |  |
| 3 | Monitor Supervisor status until the operation reaches a final state. | The original operation completes once; no second operation is created because Core restarted. |  |  |  |
| 4 | Wait for Core readiness, sign in, and open **Application Composition**. | Core recovers and displays the same final operation and receipt read back from Supervisor. |  |  |  |
| 5 | Inspect the changed business behavior. | The approved change is observable and healthy. |  |  |  |
| 6 | Compare owner bootstrap counts and unrelated service identities with the starting values. | No product record is duplicated and unrelated services were not replaced. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester name:
- Test execution date: YYYY-MM-DD
- Defect IDs: None /
- Tester comments:
- Business acceptance decision: Accepted / Not Accepted / Accepted with Defect(s)
- Business owner or reviewer:
- Review date: YYYY-MM-DD
