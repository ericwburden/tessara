# UAT-8A-03 — Configuration and diagnostics

## 1. Test Script Summary

- System / Module: Tessara Module Management / Components
- Requirement: Sprint 8A AC-08 and AC-13
- Environment: Frozen Sprint 8A UAT candidate
- User role: Global Module Management manager and reader
- Scenario: Configure the Components display label and Dataset timeout and review sanitized diagnostics.
- Acceptance: Only navigation/admin display uses the label, timeout range is 1–30 with default 5, authority is correct, and diagnostics disclose no product data or secrets.

## 2. Before You Start

- Record the current display label and timeout: ____________________
- Use temporary label `Visual Parts` and timeout `6`.

## 3. Test Steps

| Step | User action | Expected result | Actual result | Pass/Fail | Notes or defect ID |
|---|---|---|---|---|---|
| 1 | Save `Visual Parts` and timeout `6`. | Navigation/admin show the label; product headings remain Components; diagnostics show redacted effective values. | | | |
| 2 | Try blank/81-character labels, timeouts 0/31, an unknown field, and unsupported schema version. | Every invalid value is rejected without clamping or partial save. | | | |
| 3 | Repeat as a reader and as a Components manager without global configuration authority. | Read access follows role; unauthorized writes are denied. | | | |
| 4 | Restore the recorded settings. | Original configuration and navigation are restored. | | | |

## 4. Overall Test Result

- Overall result: Pass / Fail / Blocked
- Tester / date: ____________________
- Defect IDs or comments: ____________________
- Acceptance decision: Accepted / Not Accepted / Accepted with defects

