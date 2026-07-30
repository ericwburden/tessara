# UAT-6D-05 — Authorization And Nondisclosure

## 1. Test Script Summary

- System / Module: Core authorization and Module SDK Reference
- Requirement: Signed capability/scope projection and nondisclosure
- Environment: `http://127.0.0.1:8080`
- User role: Administrator and constrained operator
- Business scenario: Authorized and constrained users attempt the reference
  root and Organization-scoped probe through the same Core origin.
- Acceptance criteria: Authorized requests succeed; unauthorized known and
  random Organization identifiers are indistinguishable and disclose no
  module or Organization details.

## 2. Before You Start

Preconditions:

1. UAT-6D-01 passed and demo users are seeded.
2. Use the administrator and constrained operator accounts.

Record selection criteria:

1. Select one Organization visible to the constrained operator.
2. Select one known Organization outside that scope.
3. Generate one random UUID.

Record actually tested:

- Authorized Organization:
- Unauthorized known Organization:
- Random UUID:

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | As administrator, open `/reference/module-sdk`. | The reference root succeeds. |  |  |  |
| 2 | As the constrained operator, open the root and the scoped probe for the authorized Organization. | Only granted destinations succeed and the scoped response names only the authorized Organization. |  |  |  |
| 3 | Request the scoped probe for the known unauthorized Organization. | Access is denied without disclosing whether the Organization exists. |  |  |  |
| 4 | Request the same route with the random UUID and compare status/body shape. | The result is indistinguishable from Step 3. |  |  |  |

## 4. Overall Test Result

- Overall result:
- Tester:
- Execution date:
- Defects: None /
- Comments:
- Business acceptance: Accepted / Not Accepted / Accepted with Defects
