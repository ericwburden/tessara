# UAT-6F-01 — Bootstrap the Complete Reference Application

## 1. Test Script Summary

- System / Module: Tessara Application Composition
- Enhancement / Requirement: Bootstrap directly from an Application Blueprint
- Test environment: Frozen Sprint 6F UAT candidate at `http://127.0.0.1:8080`
- User role: Authorized local deployment operator and Tessara administrator
- Business scenario: An operator creates a fresh Tessara installation from the
  approved reference Blueprint. An administrator then confirms that the
  intended Core, Dashboard, Scoped Records, and declared starter content are
  available through the product.
- Acceptance criteria: The signed apply succeeds once, the composition receipt
  identifies the complete application, all intended experiences are healthy,
  and the declared starter content is visible without manual database work.

## 2. Before You Start

Preconditions:

1. The candidate is frozen and its source and image identities are recorded.
2. The Sprint 6F Compose project and its data volumes do not exist.
3. Docker is running and ports 8080 and 8095 are available.
4. The tester can sign in as an administrator after bootstrap.

Record Selection Criteria:

1. Use `deploy/sprint-6f/blueprints/reference.json` from the frozen candidate.
2. Use the signed catalog and trust key checked into the same candidate.

Record Actually Tested:

- Candidate commit/tree:
- Blueprint digest:
- Plan digest:
- Receipt revision/digest:
- Browser/version:

Input Values to Use During Test:

1. Composition: `reference`
2. Administrator email: `admin@tessara.local`
3. Administrator password: supplied through the UAT environment

Tester Instructions: Follow the steps in order and attach the apply receipt and
screenshots of the observed product content.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | From the frozen candidate worktree, run `.\scripts\bootstrap-sprint-6f-composition.ps1 -Composition reference`. | Catalog verification, Blueprint resolution, signed authorization, and Supervisor apply complete successfully; a receipt path is displayed. |  |  |  |
| 2 | Open the generated apply response and record the plan and receipt identities above. | The receipt reports the expected installation, Dashboard and Scoped Records enabled, and bootstrap receipts for Core and both modules. |  |  |  |
| 3 | Open `http://127.0.0.1:8080`, sign in as the administrator, and open **Application Composition**. | The native composition page opens and shows an observed receipt rather than an unmaterialized state. |  |  |  |
| 4 | Open **Dashboards**, then open **Reference Operations**. | The Dashboard is available with the metric-summary and records placements. |  |  |  |
| 5 | Open **Scoped Records** and inspect the available records. | One `Reference record` starter record is visible in the reference organization scope. |  |  |  |
| 6 | Return to normal Core pages and primary navigation. | Core remains usable and navigation includes the enabled reference modules without an unavailable fallback. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester name:
- Test execution date: YYYY-MM-DD
- Defect IDs: None /
- Tester comments:
- Business acceptance decision: Accepted / Not Accepted / Accepted with Defect(s)
- Business owner or reviewer:
- Review date: YYYY-MM-DD
