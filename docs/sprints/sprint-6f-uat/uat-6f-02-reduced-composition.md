# UAT-6F-02 — Bootstrap the Signed Reduced Application

## 1. Test Script Summary

- System / Module: Tessara Application Composition
- Enhancement / Requirement: Detached, trusted bootstrap of a materially
  reduced composition
- Test environment: Isolated empty Sprint 6F UAT environment
- User role: Authorized local deployment operator and Tessara administrator
- Business scenario: An operator bootstraps a smaller Tessara application from
  a trusted, pre-resolved reduced lockfile and detached plan signature. The
  administrator confirms that omitted modules and their product experiences
  were not materialized.
- Acceptance criteria: The trusted detached input produces a healthy Core and
  gateway installation, while Dashboard and Scoped Records services,
  navigation, content, and bootstrap receipts remain absent.

## 2. Before You Start

Preconditions:

1. Use an empty installation isolated from UAT-6F-01.
2. Use the disposable `tessara-sprint-6f` Compose project at
   `http://127.0.0.1:8080`; Supervisor is `http://127.0.0.1:8095`. Run this
   scenario after tearing down the reference installation and its named volumes.
3. Resolve `deploy/sprint-6f/blueprints/reduced.json`, sign it with
   `tessara-compose resolved-sign`, then bootstrap with
   `scripts/bootstrap-sprint-6f-composition.ps1 -Composition reduced
   -ResolvedCompositionEnvelope <signed-envelope> -ReleaseCatalogEnvelope
   <signed-catalog-used-to-resolve-the-envelope> -ReplaceExisting`. The same
   signed catalog must be supplied so Core reproduces the detached lockfile's
   exact catalog digest.
4. The tester can sign in as an administrator after bootstrap.

Record Selection Criteria:

1. Use the candidate's reduced Blueprint resolution.
2. Confirm its locked module inventory is empty before applying it.

Record Actually Tested:

- Candidate commit/tree:
- Lockfile digest:
- Detached signature identity:
- Plan digest:
- Receipt revision/digest:

Input Values to Use During Test:

1. Composition: `reduced`
2. Omitted modules: `tessara.dashboards`, `tessara.reference.scoped-records`

Tester Instructions: Stop and mark the script Blocked if the detached input or
isolated environment details have not been supplied.

## 3. Test Steps

| Step | User Action | Expected Result | Actual Result | Pass/Fail | Notes or Defect ID |
| --- | --- | --- | --- | --- | --- |
| 1 | Verify the supplied detached signature using the supplied trusted key and record its identity. | Verification succeeds and binds the exact reduced lockfile or plan digest. |  |  |  |
| 2 | Inspect the reduced lockfile before apply. | Core and gateway are locked; Dashboard and Scoped Records are absent from releases, enablement, bootstrap, and actions. |  |  |  |
| 3 | Run the supplied signed reduced-composition apply command. | Supervisor accepts the separate authorization and produces a successful installation receipt. |  |  |  |
| 4 | Sign in and use normal Core navigation. | Core is healthy and usable; Dashboard and Scoped Records destinations are not shown. |  |  |  |
| 5 | Inspect the receipt and deployed service inventory with the UAT coordinator. | No omitted-module artifact, configuration, enablement, or bootstrap receipt is present, and no omitted-module service is running. |  |  |  |
| 6 | Open **Application Composition**. | The observed state identifies the reduced plan and does not imply that hidden navigation alone removed a deployed module. |  |  |  |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester name:
- Test execution date: YYYY-MM-DD
- Defect IDs: None /
- Tester comments:
- Business acceptance decision: Accepted / Not Accepted / Accepted with Defect(s)
- Business owner or reviewer:
- Review date: YYYY-MM-DD
